use std::{collections::HashSet, sync::Arc};

use bcs_ext::BCSCodec;
use consensus::blockdag::BlockDAG;
use consensus_types::blockhash::ORIGIN;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_crypto::HashValue;
use starcoin_storage::{
    flexi_dag::{SyncFlexiDagSnapshot, SyncFlexiDagSnapshotStorage},
    storage::CodecKVStore,
    Storage, SyncFlexiDagStore,
};

pub struct SyncBlockDag {
    accumulator: MerkleAccumulator,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
}

#[derive(Clone, Debug, Hash, Eq, PartialOrd, Ord, PartialEq, Serialize, Deserialize)]
struct RelationshipPair {
    pub parent: HashValue,
    pub child: HashValue,
}

impl SyncBlockDag {
    pub fn build_sync_block_dag(
        dag: Arc<BlockDAG>,
        store: Arc<Storage>,
    ) -> (MerkleAccumulator, Arc<SyncFlexiDagSnapshotStorage>) {
        let accumulator = MerkleAccumulator::new_empty(store.get_accumulator_storage());
        let accumulator_snapshot = store.get_accumulator_snapshot_storage();

        let mut next_parents = HashSet::new();
        next_parents.insert(HashValue::new(ORIGIN));
        accumulator
            .append(&[HashValue::new(ORIGIN)])
            .expect("appending genesis for sync accumulator must be successful");
        loop {
            let mut children_set = HashSet::new();
            let mut relationship_set = HashSet::new();
            next_parents.into_iter().for_each(|parent| {
            let result_children = dag.get_children(parent);
            match result_children {
                Ok(mut children) => {
                    if !children.is_empty() {
                        children_set.extend(children.clone().into_iter());
                        relationship_set.extend(children.into_iter().map(|child| {
                            return RelationshipPair {
                                parent,
                                child,
                            }
                        }).collect::<HashSet<RelationshipPair>>());
                    }
                }
                Err(error) => {
                    panic!("failed to get the children when building sync accumulator, error message: {}", error.to_string());
                }
            }
            });
            if children_set.is_empty() {
                break;
            } else {
                let mut sorted_children = children_set.iter().cloned().collect::<Vec<_>>();
                sorted_children.sort();

                let mut sorted_relationship_set =
                    relationship_set.iter().cloned().collect::<Vec<_>>();
                sorted_relationship_set.sort();

                let accumulator_leaf = HashValue::sha3_256_of(
                    &sorted_relationship_set
                        .encode()
                        .expect("encoding the sorted relatship set must be successful"),
                );
                accumulator
                    .append(&[accumulator_leaf])
                    .expect("appending accumulator leaf for sync must be successful");
                accumulator_snapshot
                    .put(
                        accumulator_leaf,
                        SyncFlexiDagSnapshot {
                            child_hashes: sorted_children,
                            accumulator_info: accumulator.get_info(),
                        },
                    )
                    .expect("putting accumulator snapshot must be successful");
                next_parents = children_set;
            }
        }

        return (accumulator, accumulator_snapshot);
    }
}
