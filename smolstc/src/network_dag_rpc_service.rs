use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    chain_dag_service::ChainDagService,
    network_dag_data::Status,
    network_dag_rpc::{gen_server::NetworkDagRpc, NetworkDagRpcImpl},
};
use bcs_ext::BCSCodec;
use network_p2p::Event;
use network_p2p_core::{server::NetworkRpcServer, RawRpcServer};
use network_p2p_types::{OutgoingResponse, ProtocolRequest};
use sc_peerset::PeerId;
use starcoin_service_registry::{ActorService, EventHandler, ServiceFactory};

pub struct NetworkDagRpcService {
    rpc_server: Arc<NetworkRpcServer>,
}

impl NetworkDagRpcService {
    pub fn new(ctx: &mut starcoin_service_registry::ServiceContext<NetworkDagRpcService>) -> Self {
        let rpc_impl =
            NetworkDagRpcImpl::new(ctx.service_ref::<ChainDagService>().unwrap().clone());
        let rpc_server = NetworkRpcServer::new(rpc_impl.to_delegate());
        NetworkDagRpcService {
            rpc_server: Arc::new(rpc_server),
        }
    }
}

impl ActorService for NetworkDagRpcService {}

impl ServiceFactory<NetworkDagRpcService> for NetworkDagRpcService {
    fn create(
        ctx: &mut starcoin_service_registry::ServiceContext<NetworkDagRpcService>,
    ) -> anyhow::Result<NetworkDagRpcService> {
        anyhow::Result::Ok(NetworkDagRpcService::new(ctx))
    }
}

impl EventHandler<Self, ProtocolRequest> for NetworkDagRpcService {
    fn handle_event(
        &mut self,
        msg: ProtocolRequest,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) {
        let rpc_server = self.rpc_server.clone();
        ctx.spawn(async move {
            let protocol = msg.protocol;
            let rpc_path = protocol.strip_prefix("/starcoin/rpc/").unwrap().to_string();
            let peer = msg.request.peer.into();
            let result = rpc_server
                .handle_raw_request(peer, rpc_path.into(), msg.request.payload)
                .await;

            let resp = bcs_ext::to_bytes(&result).expect("NetRpc Result must encode success.");
            //TODO: update reputation_changes
            if let Err(e) = msg.request.pending_response.send(OutgoingResponse {
                result: Ok(resp),
                reputation_changes: vec![],
            }) {
                println!("Send response to rpc call failed:{:?}", e);
            }
        });
    }
}
