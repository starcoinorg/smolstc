use crate::{sync_dag_protocol_trait::PeerSynDagAccumulator, sync_dag_types::DagBlockIdAndNumber};
use anyhow::{format_err, Result};
use futures::FutureExt;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_storage::accumulator;
use std::sync::Arc;
use stream_task::{CollectorState, TaskResultCollector, TaskState};

#[derive(Clone)]
pub struct FindAncestorTask {
    start_leaf_number: u64,
    fetcher: Arc<dyn PeerSynDagAccumulator>,
    batch_size: u64,
}
impl FindAncestorTask {
    pub(crate) fn new(
        current_leaf_numeber: u64,
        target_leaf_numeber: u64,
        fetcher: Arc<crate::network_dag_verified_client::VerifiedDagRpcClient>,
    ) -> Self {
        FindAncestorTask {
            start_leaf_number: std::cmp::min(current_leaf_numeber, target_leaf_numeber),
            fetcher,
            batch_size: 10,
        }
    }
}

impl TaskState for FindAncestorTask {
    type Item = DagBlockIdAndNumber;

    fn new_sub_task(self) -> futures_core::future::BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let current_number = self.start_leaf_number;
            let leaf_hashes = self
                .fetcher
                .get_sync_dag_asccumulator_leaves(None, self.start_leaf_number, self.batch_size)
                .await?;
            let id_and_numbers: Vec<DagBlockIdAndNumber> = leaf_hashes
                .into_iter()
                .enumerate()
                .map(|(index, accumulator_leaf)| DagBlockIdAndNumber {
                    accumulator_leaf,
                    number: current_number.saturating_sub(index as u64),
                })
                .collect();
            Ok(id_and_numbers)
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        //this should never happen, because all node's genesis block should same.
        if self.start_leaf_number == 0 {
            return None;
        }

        let next_number = self.start_leaf_number.saturating_sub(self.batch_size);
        Some(Self {
            start_leaf_number: next_number,
            batch_size: self.batch_size,
            fetcher: self.fetcher.clone(),
        })
    }
}

pub struct AncestorCollector {
    accumulator: Arc<MerkleAccumulator>,
    ancestor: Option<DagBlockIdAndNumber>,
}

impl AncestorCollector {
    pub fn new(accumulator: Arc<MerkleAccumulator>) -> Self {
        Self {
            accumulator,
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

        let accumulator_leaf = self.accumulator.get_leaf(item.number)?.ok_or_else(|| {
            format_err!("Cannot find accumulator leaf by number: {}", item.number)
        })?;

        if item.accumulator_leaf == accumulator_leaf {
            self.ancestor = Some(item);
            return anyhow::Result::Ok(CollectorState::Enough);
        } else {
            Ok(CollectorState::Need)
        }
    }

    fn finish(mut self) -> Result<Self::Output> {
        self.ancestor
            .take()
            .ok_or_else(|| format_err!("Unexpect state, collector finished by ancestor is None"))
    }
}
