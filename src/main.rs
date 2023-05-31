mod network_dag_service;
mod network_dag_handle;
mod network_dag_trait;
mod network_dag_worker;
mod network_dag_data;
mod network_dag_rpc_service;
mod network_dag_rpc;

use futures::FutureExt;
use futures_core::future::BoxFuture;
use network_dag_rpc_service::NetworkDagRpcService;
use network_p2p_core::{server::NetworkRpcServer, InmemoryRpcClient};
use serde::{Deserialize, Serialize};
use network_p2p_derive::net_rpc; 
use network_p2p_types::peer_id::PeerId;
use anyhow::Result;
use network_dag_service::{NetworkDagService, NetworkDagServiceFactory};
use starcoin_service_registry::{RegistryService, RegistryAsyncService};

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let registry = RegistryService::launch(); 
        registry.register::<NetworkDagRpcService>().await.unwrap();
        registry.register_by_factory::<NetworkDagService, NetworkDagServiceFactory>().await.unwrap();
    })
}
