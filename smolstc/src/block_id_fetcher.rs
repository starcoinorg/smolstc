use std::collections::HashMap;

use anyhow::Result;
use futures::{FutureExt, TryFutureExt};
use futures_core::future::BoxFuture;
use network_p2p_core::PeerId;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;

use crate::sync_dag_protocol::SyncBlockIds;

pub trait BlockIdFetcher: Send + Sync {
    fn fetch_block_ids(
        &self,
        peer: Option<PeerId>,
        block_hash: Vec<HashValue>,
        reverse: bool,
        depth: u64,
    ) -> BoxFuture<Result<Vec<SyncBlockIds>>>;

    fn fetch_block_id(
        &self,
        peer: Option<PeerId>,
        block_hash: Vec<HashValue>,
    ) -> BoxFuture<Result<Option<SyncBlockIds>>> {
        self.fetch_block_ids(peer, block_hash, false, 1)
            .and_then(|mut ids| async move { Ok(ids.pop()) })
            .boxed()
    }
}
