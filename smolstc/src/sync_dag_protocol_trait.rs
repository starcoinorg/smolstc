use crate::network_dag_rpc::{
    SyncDagBlockInfo, TargetAccumulatorLeaf, TargetAccumulatorLeafDetail,
};
use anyhow::Result;
use futures_core::future::BoxFuture;
use network_p2p_core::PeerId;

pub trait PeerSynDagAccumulator: Send + Sync {
    fn get_sync_dag_asccumulator_leaves(
        &self,
        peer_id: Option<PeerId>,
        leaf_index: u64,
        batch_size: u64,
    ) -> BoxFuture<Result<Vec<TargetAccumulatorLeaf>>>;

    fn get_accumulator_leaf_detail(
        &self,
        peer_id: Option<PeerId>,
        leaf_index: u64,
        batch_size: u64,
    ) -> BoxFuture<Result<Option<Vec<TargetAccumulatorLeafDetail>>>>;

    fn get_dag_block_info(
        &self,
        peer: Option<PeerId>,
        leaf_index: u64,
        batch_size: u64,
    ) -> BoxFuture<Result<Option<Vec<SyncDagBlockInfo>>>>;
}
