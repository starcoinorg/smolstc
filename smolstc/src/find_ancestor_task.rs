use crate::{block_id_fetcher::BlockIdFetcher, sync_dag_types::DagBlockIdAndNumber};
use anyhow::{format_err, Result};
use futures::FutureExt;
use starcoin_crypto::HashValue;
use starcoin_types::block::{BlockIdAndNumber, BlockNumber};
use std::{sync::Arc, hash::Hash};
use stream_task::{CollectorState, TaskResultCollector, TaskState};

#[derive(Clone)]
pub struct FindAncestorTask {
    tips_hash: Vec<HashValue>, // tips hash in accumulator in chain service
    fetcher: Arc<dyn BlockIdFetcher>,
}
impl FindAncestorTask {
    pub(crate) fn new(
        current_tips: HashValue,
        target_tips: HashValue,
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
    type Item = DagBlockIdAndNumber;

    fn new_sub_task(self) -> futures_core::future::BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let current_number = self.start_number;
            let block_ids = self
                .fetcher
                .fetch_block_ids(None, current_number, true, self.batch_size)
                .await?;
            let id_and_numbers = block_ids
                .into_iter()
                .enumerate()
                .map(|(idx, block_info)| DagBlockIdAndNumber {
                    id: block_info.block_hash,
                    number: current_number.saturating_sub(idx as u64),
                    parents: block_info.parents,
                })
                .collect();
            Ok(id_and_numbers)
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        //this should never happen, because all node's genesis block should same.
        if self.start_number == 0 {
            return None;
        }

        let next_number = self.start_number.saturating_sub(self.batch_size);
        Some(Self {
            start_number: next_number,
            batch_size: self.batch_size,
            fetcher: self.fetcher.clone(),
        })
    }
}


pub struct AncestorCollector {
    // accumulator: Arc<MerkleAccumulator>, // accumulator is temporarily commented.
    ancestor: Option<DagBlockIdAndNumber>,
}

impl AncestorCollector {
    pub fn new() -> Self {
        Self {
            // accumulator,
            ancestor: None,
        }
    }
}

impl TaskResultCollector<DagBlockIdAndNumber> for AncestorCollector {
    type Output = DagBlockIdAndNumber;

    fn collect(&mut self, item: DagBlockIdAndNumber) -> anyhow::Result<CollectorState> {
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
