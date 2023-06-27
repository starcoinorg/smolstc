use crate::{
    network_dag_handle::DagDataHandle, network_dag_rpc_service::NetworkDagRpcService,
    network_dag_trait::NetworkDag, network_dag_verified_client::NetworkDagServiceRef,
    network_dag_worker::build_worker,
};
use anyhow::Result;
use futures::{channel::mpsc::channel, FutureExt, StreamExt};
use futures_core::future::BoxFuture;
use network_p2p::{config, config::RequestResponseConfig, Event, NetworkService, NetworkWorker};
use network_p2p_types::{Multiaddr, ProtocolRequest};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef,
    ServiceRequest,
};
use std::{borrow::Cow, net::Ipv4Addr, sync::Arc, time::Duration};

const MAX_REQUEST_SIZE: u64 = 1024 * 1024;
const MAX_RESPONSE_SIZE: u64 = 1024 * 1024 * 64;
const REQUEST_BUFFER_SIZE: usize = 128;

// notify
const PROTOCOL_NAME_NOTIFY: &str = "/starcoin/notify/1";

// request-response
const PROTOCOL_NAME_REQRES_1: &str = "/starcoin/request_response/1";
const PROTOCOL_NAME_REQRES_2: &str = "/starcoin/request_response/2";

// broadcast
const PROTOCOL_NAME_BROADCAST: &str = "/starcoin/request_response/2";

pub enum NetworkType {
    InMemory(
        Vec<Cow<'static, str>>,
        Vec<Multiaddr>,
        Vec<Cow<'static, str>>,
    ),

    /// protocol: notification, listen addr, request-response
    InP2P(
        Vec<Cow<'static, str>>,
        Vec<Multiaddr>,
        Vec<Cow<'static, str>>,
    ),
}

#[derive(Debug)]
pub struct NetworkMultiaddr;
impl ServiceRequest for NetworkMultiaddr {
    type Response = NetworkMultiaddrInfo;
}
#[derive(Debug)]
pub struct NetworkMultiaddrInfo {
    pub peers: Vec<String>,
}

pub struct NetworkDagService {
    worker: Option<NetworkWorker<DagDataHandle>>,
    network_inner_service: Arc<NetworkService>,
}

impl NetworkDagService {
    pub fn network_service(&self) -> Arc<NetworkService> {
        match &self.worker {
            Some(w) => {
                return w.service().clone();
            }
            None => {
                panic!("the worker must be initialized firstly");
            }
        }
    }
}

pub struct NetworkDagServiceFactory;
impl ServiceFactory<NetworkDagService> for NetworkDagServiceFactory {
    fn create(
        ctx: &mut starcoin_service_registry::ServiceContext<NetworkDagService>,
    ) -> anyhow::Result<NetworkDagService> {
        let network_dag_rpc_service = ctx.service_ref::<NetworkDagRpcService>()?.clone();
        let localhost = Ipv4Addr::new(127, 0, 0, 1);
        let listen_addr = config::build_multiaddr![Ip4(localhost), Tcp(0_u16)];
        let network_service = NetworkDagService::new(
            network_dag_rpc_service,
            NetworkType::InP2P(
                // notify
                vec![Cow::from(PROTOCOL_NAME_NOTIFY)],
                // listen addr
                vec![listen_addr],
                // request response
                vec![
                    Cow::from(PROTOCOL_NAME_REQRES_1),
                    Cow::from(PROTOCOL_NAME_REQRES_2),
                ],
            ),
        );
        let network_async_service: NetworkDagServiceRef =
            NetworkDagServiceRef::new(network_service.network_service());
        ctx.put_shared(network_async_service)?;
        Ok(network_service)
    }
}

impl NetworkDagService {
    pub fn new(rpc_service: ServiceRef<NetworkDagRpcService>, nt: NetworkType) -> Self {
        let worker = match nt {
            NetworkType::InMemory(notifications, listen_addrs, request_responses) => {
                build_worker(config::NetworkConfiguration {
                    notifications_protocols: notifications,
                    listen_addresses: listen_addrs,
                    transport: config::TransportConfig::MemoryOnly,
                    request_response_protocols:
                        NetworkDagService::generate_request_response_protocol(
                            rpc_service,
                            request_responses,
                        ),
                    ..config::NetworkConfiguration::new_local()
                })
            }
            NetworkType::InP2P(notifications, listen_addrs, request_responses) => {
                build_worker(config::NetworkConfiguration {
                    notifications_protocols: notifications,
                    listen_addresses: listen_addrs,
                    transport: config::TransportConfig::Normal {
                        enable_mdns: true,
                        allow_private_ip: true,
                    },
                    request_response_protocols:
                        NetworkDagService::generate_request_response_protocol(
                            rpc_service,
                            request_responses,
                        ),
                    ..config::NetworkConfiguration::new_local()
                })
            }
        };
        let network_inner_service = worker.service().clone();
        NetworkDagService {
            worker: Some(worker),
            network_inner_service: network_inner_service.clone(),
        }
    }

    fn generate_request_response_protocol(
        rpc_service: ServiceRef<NetworkDagRpcService>,
        request_responses: Vec<Cow<'static, str>>,
    ) -> Vec<RequestResponseConfig> {
        request_responses
            .into_iter()
            .fold(vec![], |mut result_vec, name| {
                let (sender, receiver) = channel(REQUEST_BUFFER_SIZE);
                let protocol_name = name.clone();
                let stream = receiver.map(move |request| ProtocolRequest {
                    protocol: protocol_name.clone(),
                    request,
                });
                rpc_service.add_event_stream(stream).unwrap();
                let result = RequestResponseConfig {
                    name: name.clone(),
                    max_request_size: MAX_REQUEST_SIZE,
                    max_response_size: MAX_RESPONSE_SIZE,
                    request_timeout: Duration::from_secs(30),
                    inbound_queue: Some(sender),
                };

                result_vec.push(result);
                result_vec
            })
    }
}

impl NetworkDag for NetworkDagService {
    fn broadcast_message(&self, message: Vec<u8>) -> Result<()> {
        todo!()
    }

    fn register_handshaking(&self, fut: BoxFuture<()>) {
        todo!()
    }
}

/// the code below should be implemented in starcoin's NetworkActorService
/// for now,
impl ActorService for NetworkDagService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        let worker = self.worker.take().unwrap();

        async_std::task::spawn(async move {
            let _ = worker.await;
        });
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        Ok(())
    }

    fn service_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl EventHandler<NetworkDagService, Event> for NetworkDagService {
    fn handle_event(
        &mut self,
        msg: Event,
        ctx: &mut starcoin_service_registry::ServiceContext<NetworkDagService>,
    ) {
        todo!()
    }
}

impl ServiceHandler<Self, NetworkMultiaddr> for NetworkDagService {
    fn handle(
        &mut self,
        msg: NetworkMultiaddr,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> <NetworkMultiaddr as ServiceRequest>::Response {
        let result = async_std::task::block_on(self.network_inner_service.network_state());
        match result {
            Ok(state) => {
                let mut peers = vec![];
                let peer_id = state.peer_id;
                state.listened_addresses.iter().for_each(|addr| {
                    peers.push(format!("{}/p2p/{}", addr.to_string(), peer_id));
                });
                return NetworkMultiaddrInfo { peers };
            }
            Err(error) => {
                return NetworkMultiaddrInfo { peers: [].to_vec() };
            }
        }
    }
}