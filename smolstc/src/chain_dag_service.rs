use std::sync::Arc;

use crate::sync_block_dag::SyncBlockDag;
use anyhow::Result;
use async_std::path::PathBuf;
use consensus::blockdag::BlockDAG;
use consensus_types::{
    blockhash::ORIGIN,
    header::{ConsensusHeader, Header},
};
use database::prelude::open_db;
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, ServiceContext, ServiceFactory};
use starcoin_storage::Storage;
use starcoin_types::block::BlockHeader;

pub struct ChainDagService {
    dag: Arc<BlockDAG>,
}

impl ChainDagService {
    fn new_dag_for_test() -> BlockDAG {
        let genesis = Header::new(BlockHeader::random(), vec![HashValue::new(ORIGIN)]);
        let genesis_hash = genesis.hash();

        let k = 16;
        let path = std::path::PathBuf::from("./sync_test_db");
        std::fs::remove_dir_all(path.clone()).unwrap_or(());

        let db = open_db(path.clone(), true, 1);

        let mut dag = BlockDAG::new(genesis, k, db, 1024);

        let block = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![genesis_hash],
        );
        dag.commit_header(block);

        dag
    }
    fn new_dag_for_test_2() -> BlockDAG {
        let genesis = Header::new(BlockHeader::random(), vec![HashValue::new(ORIGIN)]);
        let genesis_hash = genesis.hash();

        let k = 16;
        let path = std::path::PathBuf::from("./sync_test_db");
        std::fs::remove_dir_all(path.clone()).unwrap_or(());

        let db = open_db(path.clone(), true, 1);

        let mut dag = BlockDAG::new(genesis, k, db, 1024);

       let b = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![genesis_hash],
        );
 
        let c = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![genesis_hash],
        );
 
        let d = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![genesis_hash],
        );
 
        let e = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![genesis_hash],
        );

        let f = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![b.hash(), c.hash()],
        );
        let h = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![e.hash(), d.hash(), c.hash()],
        );
        let i = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![e.hash()],
        );
        let k = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![b.hash(), h.hash(), i.hash()],
        );
        let l = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![d.hash(), i.hash()],
        );
        let j = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![f.hash(), h.hash()],
        );
        let m = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![f.hash(), k.hash()],
        );
 
        dag.commit_header(b);
        dag.commit_header(c);
        dag.commit_header(d);
        dag.commit_header(e);
        dag.commit_header(f);
        dag.commit_header(h);
        dag.commit_header(i);
        dag.commit_header(k);
        dag.commit_header(l);
        dag.commit_header(j);
        dag.commit_header(m);

        dag
    }
}

impl ServiceFactory<Self> for ChainDagService {
    fn create(ctx: &mut ServiceContext<ChainDagService>) -> Result<ChainDagService> {
        // for testing only
        let dag = Arc::new(Self::new_dag_for_test());
        ctx.put_shared(dag.clone()).unwrap();
        let sync_block_dag = SyncBlockDag::build_sync_block_dag(
            dag.clone(),
            ctx.get_shared::<Arc<Storage>>().unwrap().clone(),
        );
        ctx.put_shared(sync_block_dag.clone()).unwrap();

        return Ok(ChainDagService { dag: dag.clone() });
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
