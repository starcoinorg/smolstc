mod network_dag_service;
mod network_dag_handle;
mod network_dag_trait;
mod network_dag_worker;
mod network_dag_data;
mod network_dag_rpc_service;
mod network_dag_rpc;
mod network_dag_verified_client;
mod sync_dag_service;

use std::{path::Path, sync::Arc};

use network_dag_rpc_service::NetworkDagRpcService;
use network_dag_service::{NetworkDagService, NetworkDagServiceFactory};
use starcoin_service_registry::{RegistryService, RegistryAsyncService};
use sync_dag_service::SyncDagService;
use ghostdag::protocol::GhostdagManager;
use ghostdag::ghostdata::{MemoryGhostdagStore, GhostdagStore};
use reachability::{reachability::MemoryReachabilityStore, relations::MemoryRelationsStore};
use reachability::reachability_service::MTReachabilityService;
use consensus_types::header::DbHeadersStore;
use database::prelude::{*};
use parking_lot::RwLock;

fn main() {
    // let system = actix::prelude::System::new();
    // let rt = tokio::runtime::Runtime::new().unwrap();
    // rt.block_on(async {
    //     let registry = RegistryService::launch(); 
    //     registry.register::<NetworkDagRpcService>().await.unwrap();
    //     registry.register_by_factory::<NetworkDagService, NetworkDagServiceFactory>().await.unwrap();
    //     registry.register::<SyncDagService>().await.unwrap();
    // });
    // system.run().unwrap();
    let memory_strore = Arc::new(MemoryGhostdagStore::new());
    let relation_reader = MemoryRelationsStore::new();

    let db = DB::open_default(Path::new("./jack_db")).unwrap();
    let header_store: Arc<DbHeadersStore> = Arc::new(DbHeadersStore::new(Arc::new(db), 128)); 

    let memory_reach_store = MemoryReachabilityStore::new();
    let reach_service = MTReachabilityService::new(Arc::new(RwLock::new(memory_reach_store)));

    let ghost_manager = GhostdagManager::new(0.into(), 
                                        3, 
                                        Arc::clone(&memory_strore), 
                                        relation_reader, 
                                        Arc::clone(&header_store), 
                                        reach_service);

                                        println!("success!");

    let ghost = Arc::new(ghost_manager.genesis_ghostdag_data());
    memory_strore.insert(0.into(), Arc::clone(&ghost)).unwrap();
    header_store.insert_batch(batch, hash, header, block_level)

    let dag = ghost_manager.ghostdag(&[0.into()]);
}
