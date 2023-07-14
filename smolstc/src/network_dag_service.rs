use crate::{
    network_dag_data::{ChainInfo, Status},
    network_dag_handle::DagDataHandle,
    network_dag_rpc::gen_client,
    network_dag_rpc_service::NetworkDagRpcService,
    network_dag_trait::NetworkDag,
    network_dag_verified_client::NetworkDagServiceRef,
    network_dag_worker::build_worker,
};
use anyhow::Result;
use bcs_ext::BCSCodec;
use consensus::blockdag::BlockDAG;
use futures::{channel::mpsc::channel, FutureExt, StreamExt};
use futures_core::future::BoxFuture;
use network_p2p::{config, config::RequestResponseConfig, Event, NetworkService, NetworkWorker};
use network_p2p_types::{Multiaddr, ProtocolRequest};
use sc_peerset::PeerId;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef,
    ServiceRequest,
};
use std::{borrow::Cow, collections::HashMap, net::Ipv4Addr, sync::Arc, time::Duration};

const MAX_REQUEST_SIZE: u64 = 1024 * 1024;
const MAX_RESPONSE_SIZE: u64 = 1024 * 1024 * 64;
const REQUEST_BUFFER_SIZE: usize = 128;

// notify
pub const PROTOCOL_NAME_NOTIFY: &str = "/starcoin/notify/1";

// request-response
pub const PROTOCOL_NAME_REQRES_1: &str = "/starcoin/request_response/1";
pub const PROTOCOL_NAME_REQRES_2: &str = "/starcoin/request_response/2";

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

#[derive(Debug)]
pub struct GetBestChainInfo;
impl ServiceRequest for GetBestChainInfo {
    type Response = ChainInfo;
}

pub struct NetworkDagService {
    worker: Option<NetworkWorker<DagDataHandle>>,
    network_inner_service: Arc<NetworkService>,
    peer_set: HashMap<PeerId, ChainInfo>,
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
            ctx,
            NetworkType::InP2P(
                // notify
                vec![Cow::from(PROTOCOL_NAME_NOTIFY)],
                // listen addr
                vec![listen_addr],
                // request response
                gen_client::get_rpc_info()
                    .iter()
                    .cloned()
                    .map(|path| {
                        let protocol_name: Cow<'static, str> =
                            format!("{}{}", "/starcoin/rpc/", path).into();
                        protocol_name
                    })
                    .collect::<Vec<Cow<'static, str>>>(),
            ),
        );
        let network_async_service: NetworkDagServiceRef =
            NetworkDagServiceRef::new(network_service.network_service());
        ctx.put_shared(network_async_service)?;
        Ok(network_service)
    }
}

impl NetworkDagService {
    pub fn new(
        ctx: &mut starcoin_service_registry::ServiceContext<NetworkDagService>,
        nt: NetworkType,
    ) -> Self {
        let rpc_service = ctx.service_ref::<NetworkDagRpcService>().unwrap().clone();
        let worker = match nt {
            NetworkType::InMemory(notifications, listen_addrs, request_responses) => build_worker(
                config::NetworkConfiguration {
                    notifications_protocols: notifications,
                    listen_addresses: listen_addrs,
                    transport: config::TransportConfig::MemoryOnly,
                    request_response_protocols:
                        NetworkDagService::generate_request_response_protocol(
                            rpc_service,
                            request_responses,
                        ),
                    ..config::NetworkConfiguration::new_local()
                },
                ctx,
            ),
            NetworkType::InP2P(notifications, listen_addrs, request_responses) => build_worker(
                config::NetworkConfiguration {
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
                },
                ctx,
            ),
        };
        let network_inner_service = worker.service().clone();
        NetworkDagService {
            worker: Some(worker),
            network_inner_service: network_inner_service.clone(),
            peer_set: HashMap::new(),
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
        let event_stream = self.network_inner_service.event_stream("network");
        ctx.add_stream(event_stream);

        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        Ok(())
    }

    fn service_name() -> &'static str {
        std::any::type_name::<Self>()
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

impl EventHandler<Self, network_p2p::Event> for NetworkDagService {
    fn handle_event(
        &mut self,
        msg: Event,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) {
        println!("p2p event...");
        match msg {
            Event::NotificationStreamOpened {
                remote,
                protocol,
                generic_data,
                notif_protocols,
                rpc_protocols,
                version_string,
            } => {
                let chain_info = ChainInfo::decode(&generic_data).expect("failed to decode generic data for status");
                self.peer_set.entry(remote).or_insert(chain_info);
                println!("a peer is put into the peer set, {:?}", remote);
            }
            _ => (),
        }
    }
}

impl ServiceHandler<Self, GetBestChainInfo> for NetworkDagService {
    fn handle(
        &mut self,
        msg: GetBestChainInfo,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> <GetBestChainInfo as ServiceRequest>::Response {
        // this is for test
        let first = self.peer_set.iter().next().unwrap().1;
        first.clone()
    }
}
