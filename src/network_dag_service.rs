use std::{borrow::Cow, time::Duration, net::Ipv4Addr};
use futures::{channel::mpsc::channel, StreamExt};
use futures_core::future::BoxFuture;
use network_p2p::{NetworkWorker, Event, config, config::RequestResponseConfig};
use network_p2p_types::{Multiaddr, ProtocolRequest};
use starcoin_service_registry::{ActorService, ServiceContext, EventHandler, ServiceRef, ServiceFactory};
use crate::{network_dag_handle::DagDataHandle, network_dag_trait::NetworkDag, network_dag_worker::build_worker, network_dag_rpc_service::NetworkDagRpcService};
use anyhow::Result;

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
  InMemory,

  /// protocol: notification, listen addr, request-response 
  InP2P(Vec<Cow<'static, str>>, Vec<Multiaddr>, Vec<Cow<'static, str>>)
}

pub struct NetworkDagService {
  worker: Option<NetworkWorker<DagDataHandle>>,
}

pub struct NetworkDagServiceFactory;
impl ServiceFactory<NetworkDagService> for NetworkDagServiceFactory {
  fn create(ctx: &mut starcoin_service_registry::ServiceContext<NetworkDagService>) -> anyhow::Result<NetworkDagService> {
      let network_dag_rpc_service = ctx.service_ref::<NetworkDagRpcService>()?.clone();
      let localhost = Ipv4Addr::new(127, 0, 0, 1);
      let listen_addr = config::build_multiaddr![Ip4(localhost), Tcp(0_u16)];
      Ok(NetworkDagService::new(network_dag_rpc_service, NetworkType::InP2P(
          // notify
          vec![Cow::from(PROTOCOL_NAME_NOTIFY)], 
          
          // listen addr
          vec![listen_addr],
          
          // request response
          vec![Cow::from(PROTOCOL_NAME_REQRES_1), Cow::from(PROTOCOL_NAME_REQRES_2)])))
  }
}

impl NetworkDagService {
    pub fn new(rpc_service: ServiceRef<NetworkDagRpcService>, nt: NetworkType) -> Self {
      match nt {
          NetworkType::InMemory => {
              NetworkDagService {
                worker: None,
              }
          },
          NetworkType::InP2P(notifications, listen_addrs, request_responses) => {
              let worker = build_worker(config::NetworkConfiguration {
                  notifications_protocols: notifications,
                  listen_addresses: listen_addrs,
                  transport: config::TransportConfig::Normal { enable_mdns: true, allow_private_ip: true },
                  request_response_protocols: NetworkDagService::generate_request_response_protocol(rpc_service, request_responses),
                  ..config::NetworkConfiguration::new_local()
              });
              NetworkDagService {
                  worker: Some(worker),
              }
          },
      }
    } 

    fn generate_request_response_protocol(rpc_service: ServiceRef<NetworkDagRpcService>,  request_responses: Vec<Cow<'static, str>>) -> Vec<RequestResponseConfig> {
        request_responses.into_iter().fold(vec![], |mut result_vec, name| {
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
    fn handle_event(&mut self, msg: Event, ctx: &mut starcoin_service_registry::ServiceContext<NetworkDagService>) {
        todo!()
    }
}