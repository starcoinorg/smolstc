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
mod sync_dag_protocol;
mod sync_dag_types;
mod chain_dag_service;
mod sync_block_dag;

use anyhow::Ok;
use chain_dag_service::ChainDagService;
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

        /// wait for the client's initialization
        async_std::task::sleep(std::time::Duration::from_secs(3)).await;
        let _ = sync_service
            .send(SyncConnectToPeers { peers })
            .await
            .unwrap();

        /// wait for the connection initialization
        async_std::task::sleep(std::time::Duration::from_secs(3)).await;
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

fn init_dag(registry: &ServiceRef<RegistryService>) {
    let genesis = Header::new(BlockHeader::random(), vec![Hash::new(ORIGIN)]);
    let genesis_hash = genesis.hash();

    let k = 16;
    let db_path = std::env::temp_dir().join("smolstc");
    std::fs::remove_dir_all(db_path.clone()).expect("Failed to delete temporary directory");
    println!("db path:{}", db_path.to_string_lossy());

    let db = open_db(db_path, true, 1);

    let mut dag = BlockDAG::new(genesis, k, db, 1024);

    let block = Header::new(
        starcoin_types::block::BlockHeader::random(),
        vec![genesis_hash],
    );
    dag.commit_header(block);

    async_std::task::block_on(registry.put_shared(dag)).unwrap();
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
        registry.register::<ChainDagService>().await.unwrap();

        init_dag(&registry);

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
