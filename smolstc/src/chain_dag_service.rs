use std::sync::Arc;

use crate::sync_block_dag::SyncBlockDag;
use anyhow::Result;
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, Accumulator};
use starcoin_service_registry::{
    ActorService, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
use starcoin_storage::Storage;

pub struct ChainDagService {
    dag: SyncBlockDag,
}

impl ChainDagService {}

impl ServiceFactory<Self> for ChainDagService {
    fn create(ctx: &mut ServiceContext<ChainDagService>) -> Result<ChainDagService> {
        // for testing only
        return Ok(ChainDagService {
            dag: SyncBlockDag::build_sync_block_dag(
                ctx.get_shared::<Arc<Storage>>().unwrap().clone(),
            ),
        });
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

#[derive(Debug)]
pub struct GetAccumulatorInfo;
impl ServiceRequest for GetAccumulatorInfo {
    type Response = AccumulatorInfo;
}

impl ServiceHandler<Self, GetAccumulatorInfo> for ChainDagService {
    fn handle(
        &mut self,
        msg: GetAccumulatorInfo,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> <GetAccumulatorInfo as ServiceRequest>::Response {
        // this is for test
        self.dag.accumulator.get_info()
    }
}
