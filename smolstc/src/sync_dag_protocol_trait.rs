use std::hash::Hash;

use anyhow::Result;
use futures_core::future::BoxFuture;
use network_p2p_core::PeerId;
use starcoin_crypto::HashValue;

pub trait PeerSynDagAccumulator: Send + Sync {
    fn get_sync_dag_asccumulator_leaves(
        &self,
        peer_id: Option<PeerId>,
        leaf_index: u64,
    ) -> BoxFuture<Result<Vec<HashValue>>>;
}
