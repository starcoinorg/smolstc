use crate::block_id_fetcher::BlockIdFetcher;
use anyhow::{format_err, Result};
use starcoin_types::block::{BlockIdAndNumber, BlockNumber};
use std::sync::Arc;
use stream_task::{CollectorState, TaskResultCollector, TaskState};

#[derive(Clone)]
pub struct FindAncestorTask {
    start_number: BlockNumber,
    batch_size: u64,
    fetcher: Arc<dyn BlockIdFetcher>,
}
impl FindAncestorTask {
    pub(crate) fn new(
        current_block_number: BlockNumber,
        target_block_number: BlockNumber,
        arg: i32,
        fetcher: Arc<crate::network_dag_verified_client::VerifiedDagRpcClient>,
    ) -> Self {
        FindAncestorTask { 
            start_number: std::cmp::min(current_block_number, target_block_number), 
            batch_size: 10, 
            fetcher 
        }
    }
}

impl TaskState for FindAncestorTask {
    type Item = BlockIdAndNumber;

    fn new_sub_task(self) -> futures_core::future::BoxFuture<'static, Result<Vec<Self::Item>>> {
        todo!()
    }

    fn next(&self) -> Option<Self> {
        todo!()
    }
}


pub struct AncestorCollector {
    // accumulator: Arc<MerkleAccumulator>, // accumulator is temporarily commented.
    ancestor: Option<BlockIdAndNumber>,
}

impl AncestorCollector {
    pub fn new() -> Self {
        Self {
            // accumulator,
            ancestor: None,
        }
    }
}

impl TaskResultCollector<BlockIdAndNumber> for AncestorCollector {
    type Output = BlockIdAndNumber;

    fn collect(&mut self, item: BlockIdAndNumber) -> anyhow::Result<CollectorState> {
        if self.ancestor.is_some() {
            return Ok(CollectorState::Enough);
        }

        // here it does not consider the verification of a block info
        // fixme
        self.ancestor = Some(item);
        Ok(CollectorState::Enough)
    }

    fn finish(mut self) -> Result<Self::Output> {
        self.ancestor
            .take()
            .ok_or_else(|| format_err!("Unexpect state, collector finished by ancestor is None"))
    }
}
