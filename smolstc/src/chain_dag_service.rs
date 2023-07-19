use std::sync::Arc;

use crate::{
    network_dag_rpc::{SyncDagBlockInfo, TargetAccumulatorLeaf, TargetAccumulatorLeafDetail},
    sync_block_dag::{RelationshipPair, SyncBlockDag},
};
use anyhow::Result;
use network_p2p_types::identity::error;
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, Accumulator};
use starcoin_crypto::HashValue;
use starcoin_service_registry::{
    ActorService, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
use starcoin_storage::{storage::CodecKVStore, Storage};
use stream_task::TaskResultCollector;

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

#[derive(Debug)]
pub struct GetDagAccumulatorLeafDetails {
    pub start_index: u64,
    pub batch_size: u64,
}

impl ServiceRequest for GetDagAccumulatorLeafDetails {
    type Response = Option<Vec<TargetAccumulatorLeafDetail>>;
}

impl ServiceHandler<Self, GetDagAccumulatorLeafDetails> for ChainDagService {
    fn handle(
        &mut self,
        msg: GetDagAccumulatorLeafDetails,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> <GetDagAccumulatorLeafDetails as ServiceRequest>::Response {
        let end_index = std::cmp::min(
            msg.start_index + msg.batch_size,
            self.dag.accumulator.get_info().num_leaves - 1,
        );
        let mut details = [].to_vec();
        for index in msg.start_index..=end_index {
            let leaf_hash = self
                .dag
                .accumulator
                .get_leaf(index)
                .unwrap_or(None)
                .expect("leaf hash should not be None");
            let snapshot = self
                .dag
                .accumulator_snapshot
                .get(leaf_hash)
                .unwrap_or(None)
                .expect("the snapshot should not be None");
            let mut relationship_pair = [].to_vec();
            relationship_pair.extend(
                snapshot
                    .child_hashes
                    .into_iter()
                    .fold([].to_vec(), |mut pairs, child| {
                        let parents = self
                            .dag
                            .dag
                            .get_parents(child)
                            .expect("a child must have parents");
                        parents.into_iter().for_each(|parent| {
                            pairs.push(RelationshipPair { parent, child });
                        });
                        pairs
                    })
                    .into_iter(),
            );

            details.push(TargetAccumulatorLeafDetail {
                accumulator_root: snapshot.accumulator_info.accumulator_root,
                relationship_pair,
            });
        }
        Some(details)
    }
}

#[derive(Debug)]
pub struct GetDagBlockInfo {
    pub start_index: u64,
    pub batch_size: u64,
}

impl ServiceRequest for GetDagBlockInfo {
    type Response = Option<Vec<SyncDagBlockInfo>>;
}

impl ServiceHandler<Self, GetDagBlockInfo> for ChainDagService {
    fn handle(
        &mut self,
        msg: GetDagBlockInfo,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> <GetDagBlockInfo as ServiceRequest>::Response {
        let end_index = std::cmp::min(
            msg.start_index + msg.batch_size,
            self.dag.accumulator.get_info().num_leaves,
        );
        let leaves = match self
            .dag
            .accumulator
            .get_leaves(msg.start_index, false, msg.batch_size)
        {
            Ok(leaves) => leaves,
            Err(error) => {
                println!(
                    "an error occured when getting the leaves of the accumulator, {}",
                    error.to_string()
                );
                [].to_vec()
            }
        };
        let snapshots = leaves
            .iter()
            .map(|hash| match self.dag.accumulator_snapshot.get(*hash) {
                Ok(children) => children.expect("children must exist"),
                Err(error) => {
                    panic!(
                        "error occured when query the accumulator snapshot: {}",
                        error.to_string()
                    );
                }
            })
            .collect::<Vec<_>>();
        let info = snapshots
            .into_iter()
            .map(|snapshot| {
                let headers =
                    snapshot
                        .child_hashes
                        .into_iter()
                        .fold([].to_vec(), |mut headers, child| {
                            let header = match self.dag.dag.get_block_header(child) {
                                Ok(header) => header,
                                Err(error) => {
                                    panic!(
                                        "error occured when query the block header: {}",
                                        error.to_string()
                                    );
                                }
                            };
                            headers.push(header);
                            headers
                        });
                SyncDagBlockInfo {
                    block_headers: headers,
                    accumulator_info: snapshot.accumulator_info,
                }
            })
            .collect::<Vec<_>>();
        Some(info)
    }
}
