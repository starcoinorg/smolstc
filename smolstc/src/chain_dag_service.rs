use consensus::blockdag::BlockDAG;
use consensus_types::{blockhash::ORIGIN, header::{Header, ConsensusHeader}};
use database::prelude::open_db;
use starcoin_service_registry::{ActorService, ServiceContext, ServiceFactory};
use anyhow::Result;
use starcoin_types::block::BlockHeader;
use starcoin_crypto::HashValue as Hash;

use crate::sync_block_dag::SyncBlockDag;



pub struct ChainDagService {
    sync_dag: SyncBlockDag,
}

impl ChainDagService {
    /// for testing only
    pub fn new_testing_db() -> Self {
    }
}

impl ServiceFactory<Self> for ChainDagService {
  fn create(ctx: &mut ServiceContext<ChainDagService>) -> Result<ChainDagService> {
      Ok(Self::new_testing_db())
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