use std::sync::Arc;

use crate::sync_block_dag::SyncBlockDag;
use anyhow::Result;
use consensus::blockdag::BlockDAG;
use consensus_types::{
    blockhash::ORIGIN,
    header::{ConsensusHeader, Header},
};
use database::prelude::open_db;
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, ServiceContext, ServiceFactory};
use starcoin_types::block::BlockHeader;

pub struct ChainDagService {
    dag: Arc<BlockDAG>,
}

impl ChainDagService {
    fn new_dag_for_test() -> BlockDAG {
        let genesis = Header::new(BlockHeader::random(), vec![HashValue::new(ORIGIN)]);
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

        dag
    }
    /// for testing only
    pub fn new_for_testing() -> Self {
        ChainDagService {
            dag: Arc::new(Self::new_dag_for_test()),
        }
    }
}

impl ServiceFactory<Self> for ChainDagService {
    fn create(ctx: &mut ServiceContext<ChainDagService>) -> Result<ChainDagService> {
        Ok(Self::new_for_testing())
    }
}

/// the code below should be implemented in starcoin's NetworkActorService
/// for now,
impl ActorService for ChainDagService {
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
