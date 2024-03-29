use crate::{
    network_dag_rpc::{SyncDagBlockInfo, TargetAccumulatorLeaf, TargetAccumulatorLeafDetail},
    sync_dag_protocol_trait::PeerSynDagAccumulator,
};
use anyhow::{bail, ensure, format_err, Result, Ok};
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
pub struct SyncDagBlockTask {
    accumulator: Arc<MerkleAccumulator>,
    start_index: u64,
    batch_size: u64,
    target: AccumulatorInfo,
    fetcher: Arc<crate::network_dag_verified_client::VerifiedDagRpcClient>,
}
impl SyncDagBlockTask {
    pub fn new(
        accumulator: Arc<MerkleAccumulator>,
        start_index: u64,
        batch_size: u64,
        target: AccumulatorInfo,
        fetcher: Arc<crate::network_dag_verified_client::VerifiedDagRpcClient>,
    ) -> Self {
        SyncDagBlockTask {
            accumulator,
            start_index,
            batch_size,
            target,
            fetcher,
        }
    }
}

impl TaskState for SyncDagBlockTask {
    type Item = SyncDagBlockInfo;

    fn new_sub_task(self) -> futures_core::future::BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let dag_info: Vec<SyncDagBlockInfo> = match self.fetcher.get_dag_block_info(None, self.start_index, self.batch_size).await {
                anyhow::Result::Ok(result) => result.unwrap_or_else(|| {
                    println!("failed to get the sync dag block info, result is None");
                    [].to_vec()
                }),
                Err(error) => {
                    println!("failed to get the sync dag block info, error: {:?}", error);
                    [].to_vec()
                },
            };
            Ok(dag_info)
        }.boxed()
    }

    fn next(&self) -> Option<Self> {
        let next_number = self.start_index.saturating_add(self.batch_size);
        if next_number > self.target.num_leaves {
            return None;
        }
        Some(Self {
            accumulator: self.accumulator.clone(),
            start_index: next_number,
            batch_size: self.batch_size,
            target: self.target.clone(),
            fetcher: self.fetcher.clone(),
        })
    }
}

pub struct SyncDagBlockCollector {
}

impl SyncDagBlockCollector {
    pub fn new(
    ) -> Self {
        Self {
        }
    }
}

impl TaskResultCollector<SyncDagBlockInfo> for SyncDagBlockCollector {
    type Output = ();

    fn collect(&mut self, mut item: SyncDagBlockInfo) -> anyhow::Result<CollectorState> {
        Ok(CollectorState::Enough)
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(())
    }
}
