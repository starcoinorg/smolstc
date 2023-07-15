use std::{collections::HashSet, sync::Arc};

use bcs_ext::BCSCodec;
use consensus::blockdag::BlockDAG;
use consensus_types::{
    blockhash::ORIGIN,
    header::{ConsensusHeader, Header},
};
use database::prelude::open_db;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, node::AccumulatorStoreType, Accumulator,
    AccumulatorTreeStore, MerkleAccumulator,
};
use starcoin_crypto::HashValue;
use starcoin_storage::{
    flexi_dag::{SyncFlexiDagSnapshot, SyncFlexiDagSnapshotStorage},
    storage::CodecKVStore,
    Storage, Store, SyncFlexiDagStore,
};
use starcoin_types::block::BlockHeader;

pub struct SyncBlockDag {
    pub dag: Arc<BlockDAG>,
    pub accumulator: MerkleAccumulator,
    pub accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
}

#[derive(Clone, Debug, Hash, Eq, PartialOrd, Ord, PartialEq, Serialize, Deserialize)]
struct RelationshipPair {
    pub parent: HashValue,
    pub child: HashValue,
}

impl SyncBlockDag {
    fn new_dag_for_test() -> BlockDAG {
        let genesis = Header::new(BlockHeader::random(), vec![HashValue::new(ORIGIN)]);
        let genesis_hash = genesis.hash();

        let k = 16;
        let path = std::path::PathBuf::from("./sync_test_db");
        std::fs::remove_dir_all(path.clone()).unwrap_or(());

        let db = open_db(path.clone(), true, 1);

        let mut dag = BlockDAG::new(genesis, k, db, 1024);

        let block = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![genesis_hash],
        );
        dag.commit_header(block);

        dag
    }
    fn new_dag_for_test_2() -> BlockDAG {
        let genesis = Header::new(BlockHeader::random(), vec![HashValue::new(ORIGIN)]);
        let genesis_hash = genesis.hash();

        let k = 16;
        let path = std::path::PathBuf::from("./sync_test_db");
        std::fs::remove_dir_all(path.clone()).unwrap_or(());

        let db = open_db(path.clone(), true, 1);

        let mut dag = BlockDAG::new(genesis, k, db, 1024);

        let b = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![genesis_hash],
        );

        let c = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![genesis_hash],
        );

        let d = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![genesis_hash],
        );

        let e = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![genesis_hash],
        );

        let f = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![b.hash(), c.hash()],
        );
        let h = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![e.hash(), d.hash(), c.hash()],
        );
        let i = Header::new(starcoin_types::block::BlockHeader::random(), vec![e.hash()]);
        let k = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![b.hash(), h.hash(), i.hash()],
        );
        let l = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![d.hash(), i.hash()],
        );
        let j = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![f.hash(), h.hash()],
        );
        let m = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![f.hash(), k.hash()],
        );

        dag.commit_header(b);
        dag.commit_header(c);
        dag.commit_header(d);
        dag.commit_header(e);
        dag.commit_header(f);
        dag.commit_header(h);
        dag.commit_header(i);
        dag.commit_header(k);
        dag.commit_header(l);
        dag.commit_header(j);
        dag.commit_header(m);

        dag
    }

    pub fn build_sync_block_dag(store: Arc<Storage>) -> Self {
        let dag = Self::new_dag_for_test();
        let accumulator_store = store.get_accumulator_store(AccumulatorStoreType::SyncDag);
        let accumulator = MerkleAccumulator::new_empty(accumulator_store.clone());
        let accumulator_snapshot = store.get_accumulator_snapshot_storage();

        let mut next_parents = HashSet::new();
        let genesis_hash = HashValue::new(ORIGIN);
        next_parents.insert(genesis_hash);
        accumulator
            .append(&[HashValue::new(ORIGIN)])
            .expect("appending genesis for sync accumulator must be successful");
        accumulator_snapshot
            .put(
                HashValue::sha3_256_of(&[genesis_hash].encode().unwrap()),
                SyncFlexiDagSnapshot {
                    child_hashes: [genesis_hash].to_vec(),
                    accumulator_info: accumulator.get_info(),
                },
            )
            .expect("putting accumulator snapshot must be successful");
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

        accumulator.flush().unwrap();
        accumulator.flush().unwrap();
        return SyncBlockDag {
            dag: Arc::new(dag),
            accumulator,
            accumulator_snapshot: accumulator_snapshot.clone(),
        };
    }
}
