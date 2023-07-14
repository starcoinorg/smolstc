use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    network_dag_data::Status,
    network_dag_rpc::{gen_server::NetworkDagRpc, NetworkDagRpcImpl}, chain_dag_service::ChainDagService,
};
use bcs_ext::BCSCodec;
use network_p2p::Event;
use network_p2p_core::server::NetworkRpcServer;
use network_p2p_types::ProtocolRequest;
use sc_peerset::PeerId;
use starcoin_service_registry::{ActorService, EventHandler, ServiceFactory};

pub struct NetworkDagRpcService {
    rpc_server: Arc<NetworkRpcServer>,
}

impl NetworkDagRpcService {
    pub fn new(ctx: &mut starcoin_service_registry::ServiceContext<NetworkDagRpcService>) -> Self {
        let rpc_impl = NetworkDagRpcImpl::new(ctx.service_ref::<ChainDagService>().unwrap().clone());
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
        todo!()
    }
}
