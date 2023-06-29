mod block_id_fetcher;
mod find_ancestor_task;
mod network_dag_data;
mod network_dag_handle;
mod network_dag_rpc;
mod network_dag_rpc_service;
mod network_dag_service;
mod network_dag_trait;
mod network_dag_verified_client;
mod network_dag_worker;
mod sync_dag_service;
mod sync_task_error_handle;

use anyhow::Ok;
use async_std::path::PathBuf;
use network_dag_rpc_service::NetworkDagRpcService;
// use flexi_dag::{FlexiBlock, FlexiDagConsensus};
use network_dag_service::{NetworkDagService, NetworkDagServiceFactory, NetworkMultiaddr};
use starcoin_crypto::HashValue as Hash;
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_types::block::BlockHeader;
use sync_dag_service::{CheckSync, SyncConnectToPeers, SyncDagService, SyncInitVerifiedClient};

// dag
use consensus::blockdag::BlockDAG;
use consensus_types::{
    blockhash::ORIGIN,
    header::{ConsensusHeader, Header},
};
use database::prelude::*;

async fn run_sync(
    registry: &ServiceRef<RegistryService>,
    peers: Vec<String>,
) -> anyhow::Result<()> {
    let sync_service = registry.service_ref::<SyncDagService>().await.unwrap();

    async_std::task::spawn(async move {
        /// to wait the services start`
        async_std::task::sleep(std::time::Duration::from_secs(3)).await;

        /// connect to the other node
        let _ = sync_service.send(SyncInitVerifiedClient).await.unwrap();
        let _ = sync_service
            .send(SyncConnectToPeers { peers })
            .await
            .unwrap();

        /// run the sync procedure
        let result = sync_service.send(CheckSync).await.unwrap();
    });

    return Ok(());
}

async fn run_server(registry: &ServiceRef<RegistryService>) -> anyhow::Result<()> {
    let network_service = registry
        .service_ref::<NetworkDagService>()
        .await
        .unwrap()
        .clone();
    async_std::task::spawn(async move {
        /// to wait the services start`
        async_std::task::sleep(std::time::Duration::from_secs(3)).await;

        let result = network_service.send(NetworkMultiaddr).await.unwrap();
        result.peers.into_iter().for_each(|peer| {
            println!("{}", peer);
        });
    });
    return Ok(());
}

fn main() {
    async_std::task::block_on(async {
        let system = actix::prelude::System::new();

        /// init services: network service and sync service
        /// Actix services are initialized in parallel.
        /// Therefore, if there are dependencies among them,
        /// we must first initialize the Actix services and then
        /// initialize the objects related to the dependencies.
        let registry = RegistryService::launch();
        registry.register::<NetworkDagRpcService>().await.unwrap();
        registry
            .register_by_factory::<NetworkDagService, NetworkDagServiceFactory>()
            .await
            .unwrap();
        registry.register::<SyncDagService>().await.unwrap();

        /// to see if sync task or server task?
        let op = std::env::args().collect::<Vec<_>>();
        if op.len() > 1 {
            let cmd = op.get(1).unwrap();
            if "sync" == cmd {
                run_sync(&registry, op[2..].to_vec()).await.unwrap();
            } else if "server" == cmd {
                run_server(&registry).await.unwrap();
            }
        }
        system.run().unwrap();
    });
}
