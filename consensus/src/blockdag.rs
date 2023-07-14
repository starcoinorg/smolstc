use anyhow::bail;
use consensus_types::blockhash::BlockHashes;
use consensus_types::blockhash::KType;
use consensus_types::blockhash::ORIGIN;
use consensus_types::header::ConsensusHeader;
use consensus_types::header::{DbHeadersStore, Header, HeaderStore};
use database::prelude::DB;
use ghostdag::ghostdata::DbGhostdagStore;
use ghostdag::ghostdata::GhostdagStore;
use ghostdag::protocol::GhostdagManager;
use parking_lot::RwLock;
use reachability::inquirer;
use reachability::reachability::ReachabilityStoreReader;
use reachability::reachability::DbReachabilityStore;
use reachability::reachability_service::MTReachabilityService;
use reachability::relations::DbRelationsStore;
use reachability::relations::{RelationsStore, RelationsStoreReader};
use starcoin_crypto::HashValue as Hash;
use std::sync::Arc;
pub type DbGhostdagManager = GhostdagManager<
    DbGhostdagStore,
    DbRelationsStore,
    MTReachabilityService<DbReachabilityStore>,
    DbHeadersStore,
>;

#[derive(Clone)]
pub struct BlockDAG {
    genesis: Header,
    ghostdag_manager: DbGhostdagManager,
    relations_store: DbRelationsStore,
    reachability_store: DbReachabilityStore,
    ghostdag_store: DbGhostdagStore,
    header_store: DbHeadersStore,
}

impl BlockDAG {
    pub fn new(genesis: Header, k: KType, db: Arc<DB>, cache_size: u64) -> Self {
        let ghostdag_store = DbGhostdagStore::new(db.clone(), 0, cache_size);
        let relations_store = DbRelationsStore::new(db.clone(), 0, cache_size);
        let header_store = DbHeadersStore::new(db.clone(), cache_size);
        let mut reachability_store = DbReachabilityStore::new(db, cache_size);
        inquirer::init(&mut reachability_store).unwrap();
        let reachability_service =
            MTReachabilityService::new(Arc::new(RwLock::new(reachability_store.clone())));
        let ghostdag_manager = DbGhostdagManager::new(
            genesis.hash(),
            k,
            Arc::new(ghostdag_store.clone()),
            relations_store.clone(),
            Arc::new(header_store.clone()),
            reachability_service,
        );

        let mut dag = Self {
            genesis,
            ghostdag_manager,
            relations_store,
            reachability_store,
            ghostdag_store,
            header_store,
        };
        dag.init_with_genesis();
        dag
    }

    pub fn init_with_genesis(&mut self) {
        if self.relations_store.has(Hash::new(ORIGIN)).unwrap() {
            return;
        }
        self.relations_store
            .insert(Hash::new(ORIGIN), BlockHashes::new(vec![]))
            .unwrap();
        self.commit_header(self.genesis.clone())
    }

    pub fn commit_header(&mut self, header: Header) {
        // Generate ghostdag data

        let parents_hash = header.parents_hash();
        let ghostdag_data = if header.hash() != self.genesis.hash() {
            self.ghostdag_manager.ghostdag(&parents_hash)
        } else {
            self.ghostdag_manager.genesis_ghostdag_data()
        };
        // Store ghostdata
        self.ghostdag_store
            .insert(header.hash(), Arc::new(ghostdag_data.clone()))
            .unwrap();

        // Update reachability store
        let mut reachability_store = self.reachability_store.clone();
        let mut merge_set = ghostdag_data
            .unordered_mergeset_without_selected_parent()
            .filter(|hash| self.reachability_store.has(*hash).unwrap());

        inquirer::add_block(
            &mut reachability_store,
            header.hash(),
            ghostdag_data.selected_parent,
            &mut merge_set,
        )
        .unwrap();

        // store relations
        self.relations_store
            .insert(header.hash(), BlockHashes::new(parents_hash.to_vec()))
            .unwrap();
        // Store header store
        self.header_store
            .insert(header.hash(), Arc::new(header), 0)
            .unwrap();
    }

    pub fn get_parents(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        match self.relations_store.get_parents(hash) {
            Ok(parents) => anyhow::Result::Ok((*parents).clone()),
            Err(error) => {
                println!("failed to get parents by hash: {}", error.to_string());
                bail!("failed to get parents by hash: {}", error.to_string());
            }
        }
    }

    pub fn get_children(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        match self.relations_store.get_children(hash) {
            Ok(children) => anyhow::Result::Ok((*children).clone()),
            Err(error) => {
                println!("failed to get parents by hash: {}", error.to_string());
                bail!("failed to get parents by hash: {}", error.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::prelude::open_db;
    use starcoin_types::block::BlockHeader;
    use std::env;
    use std::fs;
    #[test]
    fn base_test() {
        let genesis = Header::new(BlockHeader::random(), vec![Hash::new(ORIGIN)]);
        let genesis_hash = genesis.hash();
        let k = 16;
        let db_path = env::temp_dir().join("smolstc");
        fs::remove_dir_all(db_path.clone()).expect("Failed to delete temporary directory");
        println!("db path:{}", db_path.to_string_lossy());
        let db = open_db(db_path, true, 1);
        let mut dag = BlockDAG::new(genesis, k, db, 1024);

        let block = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![genesis_hash],
        );
        dag.commit_header(block);
    }
}
