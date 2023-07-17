use crate::{
    network_dag_rpc::{TargetAccumulatorLeaf, TargetAccumulatorLeafDetail},
    sync_dag_protocol_trait::PeerSynDagAccumulator,
};
use anyhow::{bail, ensure, format_err, Result};
use bcs_ext::BCSCodec;
use futures::FutureExt;
use starcoin_accumulator::{accumulator_info::AccumulatorInfo, Accumulator, MerkleAccumulator};
use starcoin_crypto::HashValue;
use starcoin_storage::{
    flexi_dag::{SyncFlexiDagSnapshot, SyncFlexiDagSnapshotStorage},
    storage::CodecKVStore,
};
use std::sync::Arc;
use stream_task::{CollectorState, TaskResultCollector, TaskState};

#[derive(Clone)]
pub struct SyncDagAccumulatorTask {
    leaf_index: u64,
    batch_size: u64,
    target_index: u64,
    fetcher: Arc<crate::network_dag_verified_client::VerifiedDagRpcClient>,
}
impl SyncDagAccumulatorTask {
    pub(crate) fn new(
        leaf_index: u64,
        batch_size: u64,
        target_index: u64,
        fetcher: Arc<crate::network_dag_verified_client::VerifiedDagRpcClient>,
    ) -> Self {
        SyncDagAccumulatorTask {
            leaf_index,
            batch_size,
            target_index,
            fetcher,
        }
    }
}

impl TaskState for SyncDagAccumulatorTask {
    type Item = TargetAccumulatorLeafDetail;

    fn new_sub_task(self) -> futures_core::future::BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let target_details = self
                .fetcher
                .get_accumulator_leaf_detail(None, self.leaf_index, self.batch_size)
                .await?;
            Ok(target_details)
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        //this should never happen, because all node's genesis block should same.
        if self.leaf_index == 0 {
            // it is genesis
            return None;
        }

        let next_number = self.leaf_index.saturating_add(self.batch_size);
        if next_number > self.target_index {
            return None;
        }
        Some(Self {
            fetcher: self.fetcher.clone(),
            leaf_index: next_number,
            batch_size: self.batch_size,
            target_index: self.target_index,
        })
    }
}

pub struct SyncDagAccumulatorCollector {
    accumulator: MerkleAccumulator,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
    target: AccumulatorInfo,
    start_leaf_index: u64,
}

impl SyncDagAccumulatorCollector {
    pub fn new(
        accumulator: MerkleAccumulator,
        accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
        target: AccumulatorInfo,
        start_leaf_index: u64,
    ) -> Self {
        Self {
            accumulator,
            accumulator_snapshot,
            target,
            start_leaf_index,
        }
    }
}

impl TaskResultCollector<TargetAccumulatorLeafDetail> for SyncDagAccumulatorCollector {
    type Output = (u64, MerkleAccumulator);

    fn collect(&mut self, mut item: TargetAccumulatorLeafDetail) -> anyhow::Result<CollectorState> {
        item.relationship_pair.sort();
        let accumulator_leaf = HashValue::sha3_256_of(
            &item
                .encode()
                .expect("encoding the sorted relatship set must be successful"),
        );
        self.accumulator.append(&[accumulator_leaf])?;
        self.accumulator.flush()?;

        let accumulator_info = self.accumulator.get_info();
        if accumulator_info.accumulator_root != item.accumulator_root {
            bail!("sync occurs error for the accumulator root differs from other!")
        }

        let num_leaves = accumulator_info.num_leaves;
        self.accumulator_snapshot.put(
            accumulator_leaf,
            SyncFlexiDagSnapshot {
                child_hashes: item
                    .relationship_pair
                    .into_iter()
                    .map(|pair| pair.child)
                    .collect::<Vec<_>>(),
                accumulator_info,
            },
        )?;

        if num_leaves == self.target.num_leaves {
            Ok(CollectorState::Enough)
        } else {
            Ok(CollectorState::Need)
        }
    }

    fn finish(self) -> Result<Self::Output> {
        let accumulator_info = self.accumulator.get_info();

        ensure!(
            accumulator_info == self.target,
            "local accumulator info: {:?}, peer's: {:?}",
            accumulator_info,
            self.target
        );

        Ok((self.start_leaf_index, self.accumulator))
    }
}
