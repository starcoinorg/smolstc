use crate::network_dag_rpc::{TargetAccumulatorLeaf, TargetAccumulatorLeafDetail};
use anyhow::Result;
use futures_core::future::BoxFuture;
use network_p2p_core::PeerId;
use starcoin_crypto::HashValue;

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
}
