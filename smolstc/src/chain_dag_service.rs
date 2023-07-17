use std::sync::Arc;

use crate::{network_dag_rpc::TargetAccumulatorLeaf, sync_block_dag::SyncBlockDag};
use anyhow::Result;
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, Accumulator};
use starcoin_crypto::HashValue;
use starcoin_service_registry::{
    ActorService, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
use starcoin_storage::{storage::CodecKVStore, Storage};

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

#[derive(Debug)]
pub struct GetAccumulatorLeaves {
    pub start_index: u64,
    pub batch_size: u64,
}
impl ServiceRequest for GetAccumulatorLeaves {
    type Response = Vec<TargetAccumulatorLeaf>;
}

impl ServiceHandler<Self, GetAccumulatorLeaves> for ChainDagService {
    fn handle(
        &mut self,
        msg: GetAccumulatorLeaves,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> <GetAccumulatorLeaves as ServiceRequest>::Response {
        match self
            .dag
            .accumulator
            .get_leaves(msg.start_index, true, msg.batch_size)
        {
            Ok(leaves) => leaves
                .into_iter()
                .enumerate()
                .map(
                    |(index, leaf)| match self.dag.accumulator_snapshot.get(leaf) {
                        Ok(op_snapshot) => {
                            let snapshot = op_snapshot.expect("snapshot must exist");
                            TargetAccumulatorLeaf {
                                accumulator_root: snapshot.accumulator_info.accumulator_root,
                                leaf_index: msg.start_index.saturating_sub(index as u64),
                            }
                        }
                        Err(error) => {
                            panic!(
                                "error occured when query the accumulator snapshot: {}",
                                error.to_string()
                            );
                        }
                    },
                )
                .collect(),
            Err(error) => {
                println!(
                    "an error occured when getting the leaves of the accumulator, {}",
                    error.to_string()
                );
                [].to_vec()
            }
        }
    }
}
