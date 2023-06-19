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

use consensus_types::blockhash;
use network_dag_rpc_service::NetworkDagRpcService;
use network_dag_service::{NetworkDagService, NetworkDagServiceFactory};
use reachability::interval::Interval;
use reachability::reachability::ReachabilityStore;
use reachability::relations::RelationsStore;
use starcoin_service_registry::{RegistryService, RegistryAsyncService};
use sync_dag_service::SyncDagService;

// dag
use ghostdag::protocol::GhostdagManager;
use ghostdag::ghostdata::{MemoryGhostdagStore, GhostdagStore, GhostdagData};
use reachability::{reachability::MemoryReachabilityStore, relations::MemoryRelationsStore};
use reachability::reachability_service::MTReachabilityService;
use consensus_types::header::DbHeadersStore;
use database::prelude::{*};
use parking_lot::RwLock;
use consensus_types::header::Header;
use starcoin_crypto::HashValue as Hash;

fn build_header_for_test(hash: Hash, parents: Vec<Hash>) -> Header {
    let mut header = Header::from_precomputed_hash(hash, parents);

    header
}

fn main() {
    let system = actix::prelude::System::new();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let registry = RegistryService::launch(); 
        registry.register::<NetworkDagRpcService>().await.unwrap();
        registry.register_by_factory::<NetworkDagService, NetworkDagServiceFactory>().await.unwrap();
        registry.register::<SyncDagService>().await.unwrap();
    });
    system.run().unwrap();
    // let (B, C, D, E, F, H, I, J, K, L, M) = (
    //     Hash::sha3_256_of(b"B"),
    //     Hash::sha3_256_of(b"C"),
    //     Hash::sha3_256_of(b"D"),
    //     Hash::sha3_256_of(b"E"),
    //     Hash::sha3_256_of(b"F"),
    //     Hash::sha3_256_of(b"H"),
    //     Hash::sha3_256_of(b"I"),
    //     Hash::sha3_256_of(b"J"),
    //     Hash::sha3_256_of(b"K"),
    //     Hash::sha3_256_of(b"L"),
    //     Hash::sha3_256_of(b"M"),
    // );
    // let ghost_strore = Arc::new(MemoryGhostdagStore::new());
    // let mut relation_store = MemoryRelationsStore::new();
    // relation_store.insert(Hash::new(blockhash::ORIGIN), Arc::new(vec![])).unwrap();
    // relation_store.insert(B, Arc::new(vec![Hash::new(blockhash::ORIGIN)])).unwrap();
    // relation_store.insert(C, Arc::new(vec![Hash::new(blockhash::ORIGIN)])).unwrap();
    // relation_store.insert(D, Arc::new(vec![Hash::new(blockhash::ORIGIN)])).unwrap();
    // relation_store.insert(E, Arc::new(vec![Hash::new(blockhash::ORIGIN)])).unwrap();
    // relation_store.insert(H, Arc::new(vec![C, D, E])).unwrap();
    // relation_store.insert(I, Arc::new(vec![E])).unwrap();
    // relation_store.insert(K, Arc::new(vec![B, H, I])).unwrap();
    // relation_store.insert(F, Arc::new(vec![B, C])).unwrap();
    // relation_store.insert(M, Arc::new(vec![F, K])).unwrap();
    // relation_store.insert(J, Arc::new(vec![H, F])).unwrap();
    // relation_store.insert(L, Arc::new(vec![D, I])).unwrap();

    // let db = DB::open_default(Path::new("./jack_db")).unwrap();
    // let header_store: Arc<DbHeadersStore> = Arc::new(DbHeadersStore::new(Arc::new(db), 128)); 

    // let mut inner_reach_store = MemoryReachabilityStore::new();
    // (&mut inner_reach_store as &mut dyn ReachabilityStore).init(Hash::new(blockhash::ORIGIN), Interval::new(1, 3)).unwrap();
    // (&mut inner_reach_store as &mut dyn ReachabilityStore).insert(B, Hash::new(blockhash::ORIGIN), Interval::new(1, 3), 1).unwrap();
    // (&mut inner_reach_store as &mut dyn ReachabilityStore).insert(C, Hash::new(blockhash::ORIGIN), Interval::new(1, 3), 1).unwrap();
    // (&mut inner_reach_store as &mut dyn ReachabilityStore).insert(D, Hash::new(blockhash::ORIGIN), Interval::new(1, 3), 1).unwrap();
    // (&mut inner_reach_store as &mut dyn ReachabilityStore).insert(E, Hash::new(blockhash::ORIGIN), Interval::new(1, 3), 1).unwrap();
    // (&mut inner_reach_store as &mut dyn ReachabilityStore).append_child(C, H).unwrap();
    // (&mut inner_reach_store as &mut dyn ReachabilityStore).append_child(D, H).unwrap();
    // (&mut inner_reach_store as &mut dyn ReachabilityStore).append_child(E, H).unwrap();
    // (&mut inner_reach_store as &mut dyn ReachabilityStore).insert(H, D, Interval::new(1, 3), 1).unwrap();
    // (&mut inner_reach_store as &mut dyn ReachabilityStore).append_child(H, K).unwrap();
    // (&mut inner_reach_store as &mut dyn ReachabilityStore).insert(K, H, Interval::new(1, 3), 1).unwrap();
    // (&mut inner_reach_store as &mut dyn ReachabilityStore).append_child(B, K).unwrap();
    // // (&mut inner_reach_store as &mut dyn ReachabilityStore).insert(2.into(), Hash::new(blockhash::ORIGIN), Interval::new(1, 3), 1).unwrap();
    // // (&mut inner_reach_store as &mut dyn ReachabilityStore).insert(3.into(), 2.into(), Interval::new(1, 3), 2).unwrap();
    // // (&mut inner_reach_store as &mut dyn ReachabilityStore).insert(4.into(), 3.into(), Interval::new(1, 3), 3).unwrap();
    // // (&mut inner_reach_store as &mut dyn ReachabilityStore).append_child(3.into(), 4.into()).unwrap();

    // let mut memory_reach_store = Arc::new(RwLock::new(inner_reach_store));
    // let reach_service = MTReachabilityService::new(memory_reach_store);

    // let k = 3;
    // let ghost_manager = GhostdagManager::new(Hash::new(blockhash::ORIGIN), 
    //                                     k, 
    //                                     Arc::clone(&ghost_strore), 
    //                                     relation_store, 
    //                                     Arc::clone(&header_store), 
    //                                     reach_service);

    // let dag = Arc::new(ghost_manager.genesis_ghostdag_data());
    // ghost_strore.insert(Hash::new(blockhash::ORIGIN), Arc::clone(&dag)).unwrap();

    // let dag = ghost_manager.ghostdag(&[Hash::new(blockhash::ORIGIN)]);
    // ghost_strore.insert(D, Arc::new(dag.clone())).unwrap();

    // ghost_strore.insert(C, Arc::new(dag.clone())).unwrap();
    // ghost_strore.insert(E, Arc::new(dag.clone())).unwrap();

    // let dag = ghost_manager.ghostdag(&[C, D, E]);
    // ghost_strore.insert(H, Arc::new(dag.clone())).unwrap();

    // let dag = ghost_manager.ghostdag(&[H]);
    // ghost_strore.insert(K, Arc::new(dag.clone())).unwrap();

    // let dag = ghost_manager.ghostdag(&[Hash::new(blockhash::ORIGIN)]);
    // ghost_strore.insert(B, Arc::new(dag.clone())).unwrap();


    // // let dag = ghost_manager.ghostdag(&[Hash::new(blockhash::ORIGIN)]);
    // // ghost_strore.insert(2.into(), Arc::new(dag)).unwrap();

    // // let dag = ghost_manager.ghostdag(&[2.into()]);
    // // ghost_strore.insert(3.into(), Arc::new(dag)).unwrap();

    // // let dag = ghost_manager.ghostdag(&[3.into()]);
    // // ghost_strore.insert(4.into(), Arc::new(dag)).unwrap();
    // // ghost_strore.insert(2.into(), Arc::new(GhostdagData::new_with_selected_parent(Hash::new(blockhash::ORIGIN), k))).unwrap();
    // // // ghost_strore.insert(3.into(), Arc::new(GhostdagData::new_with_selected_parent(2.into(), k))).unwrap();
    // // ghost_strore.insert(4.into(), Arc::new(GhostdagData::new_with_selected_parent(3.into(), k))).unwrap();

    // // // let dag = Arc::new(GhostdagData::new_with_selected_parent(3.into(), k));
    // // // ghost_strore.insert(4.into(), Arc::clone(&dag)).unwrap();


    // // let result = dag.consensus_ordered_mergeset(ghost_strore.as_ref()).collect::<Vec<_>>();
    
    // println!("success!");
}
