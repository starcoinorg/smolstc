use std::{collections::HashSet, sync::Arc};

use bcs_ext::{BCSCodec, Sample};
use consensus::blockdag::BlockDAG;
use consensus_types::{
    blockhash::ORIGIN,
    header::{ConsensusHeader, Header},
};
use database::prelude::{FlexiDagStorage, FlexiDagStorageConfig};
use serde::{Deserialize, Serialize};
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_crypto::HashValue;
use starcoin_storage::{
    flexi_dag::{SyncFlexiDagSnapshot, SyncFlexiDagSnapshotStorage},
    storage::CodecKVStore,
    Storage, Store, SyncFlexiDagStore,
};
use starcoin_types::{
    account_address::AccountAddress,
    block::{BlockHeader, BlockHeaderExtra},
    genesis_config::ChainId,
};

pub struct SyncBlockDag {
    pub dag: Arc<BlockDAG>,
    pub accumulator: MerkleAccumulator,
    pub accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
}

#[derive(Clone, Debug, Hash, Eq, PartialOrd, Ord, PartialEq, Serialize, Deserialize)]
pub struct RelationshipPair {
    pub parent: HashValue,
    pub child: HashValue,
}

impl SyncBlockDag {
    fn new_header_test(nonce: u32) -> BlockHeader {
        BlockHeader::new(
            HashValue::zero(),
            0,
            0,
            AccountAddress::ZERO,
            HashValue::zero(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0.into(),
            HashValue::zero(),
            ChainId::test(),
            nonce,
            BlockHeaderExtra::default(),
        )
    }

    fn new_dag_db_test() -> FlexiDagStorage {
        let path = std::path::PathBuf::from("./sync_test_db");
        std::fs::remove_dir_all(path.clone()).unwrap_or(());

        let config = FlexiDagStorageConfig::create_with_params(1, 0, 1024);
        let db = FlexiDagStorage::create_from_path(path, config)
            .expect("Failed to create flexidag storage");

        return db;
    }

    fn new_basic_dag_test() -> (Vec<Header>, BlockDAG) {
        let genesis = Header::new(BlockHeader::sample(), vec![HashValue::new(ORIGIN)]);
        let genesis_hash = genesis.hash();

        let db = Self::new_dag_db_test();

        let k = 16;
        let mut dag = BlockDAG::new(genesis, k, db);

        let b = Header::new(Self::new_header_test(1), vec![genesis_hash]);

        let c = Header::new(Self::new_header_test(2), vec![genesis_hash]);

        let d = Header::new(Self::new_header_test(3), vec![genesis_hash]);

        let e = Header::new(Self::new_header_test(4), vec![genesis_hash]);

        let f = Header::new(Self::new_header_test(5), vec![b.hash(), c.hash()]);
        let h = Header::new(Self::new_header_test(6), vec![e.hash(), d.hash(), c.hash()]);
        let i = Header::new(Self::new_header_test(7), vec![e.hash()]);
        let k = Header::new(Self::new_header_test(8), vec![b.hash(), h.hash(), i.hash()]);
        let l = Header::new(Self::new_header_test(9), vec![d.hash(), i.hash()]);
        let j = Header::new(Self::new_header_test(10), vec![f.hash(), h.hash()]);
        let m = Header::new(Self::new_header_test(11), vec![f.hash(), k.hash()]);
        let p = Header::new(Self::new_header_test(12), vec![j.hash(), m.hash()]);
        let v = Header::new(
            Self::new_header_test(13),
            vec![k.hash(), c.hash(), l.hash()],
        );

        let mut result = vec![];

        result.push(b);
        result.push(c);
        result.push(d);
        result.push(e);
        result.push(f);
        result.push(h);
        result.push(i);
        result.push(j);
        result.push(k);
        result.push(l);
        result.push(m);
        result.push(p);
        result.push(v);

        (result, dag)
    }

    fn new_dag_diff_half_leaf_test() -> (Vec<Header>, BlockDAG) {
        let genesis = Header::new(BlockHeader::sample(), vec![HashValue::new(ORIGIN)]);
        let genesis_hash = genesis.hash();

        let db = Self::new_dag_db_test();

        let k = 16;
        let mut dag = BlockDAG::new(genesis, k, db);

        let diff_b = Header::new(Self::new_header_test(1), vec![genesis_hash]);

        let diff_c = Header::new(Self::new_header_test(2), vec![genesis_hash]);

        let diff_d = Header::new(Self::new_header_test(3), vec![genesis_hash]);

        let diff_e = Header::new(Self::new_header_test(4), vec![genesis_hash]);

        let diff_f = Header::new(
            Self::new_header_test(1005),
            vec![diff_b.hash(), diff_c.hash()],
        );
        let diff_h = Header::new(
            Self::new_header_test(1006),
            vec![diff_e.hash(), diff_d.hash(), diff_c.hash()],
        );
        let diff_i = Header::new(Self::new_header_test(1007), vec![diff_e.hash()]);
        let diff_k = Header::new(
            Self::new_header_test(1008),
            vec![diff_b.hash(), diff_h.hash(), diff_i.hash()],
        );
        let diff_l = Header::new(
            Self::new_header_test(1009),
            vec![diff_d.hash(), diff_i.hash()],
        );
        let diff_j = Header::new(
            Self::new_header_test(1010),
            vec![diff_f.hash(), diff_h.hash()],
        );
        let diff_m = Header::new(
            Self::new_header_test(1011),
            vec![diff_f.hash(), diff_k.hash()],
        );
        let diff_p = Header::new(
            Self::new_header_test(1012),
            vec![diff_j.hash(), diff_m.hash()],
        );
        let diff_v = Header::new(
            Self::new_header_test(1013),
            vec![diff_k.hash(), diff_c.hash(), diff_l.hash()],
        );

        let mut result = vec![];

        result.push(diff_b);
        result.push(diff_c);
        result.push(diff_d);
        result.push(diff_e);
        result.push(diff_f);
        result.push(diff_h);
        result.push(diff_i);
        result.push(diff_j);
        result.push(diff_k);
        result.push(diff_l);
        result.push(diff_m);
        result.push(diff_p);
        result.push(diff_v);

        (result, dag)
    }

    fn new_dag_diff_leaf_test() -> (Vec<Header>, BlockDAG) {
        let genesis = Header::new(BlockHeader::sample(), vec![HashValue::new(ORIGIN)]);
        let genesis_hash = genesis.hash();

        let db = Self::new_dag_db_test();

        let k = 16;
        let mut dag = BlockDAG::new(genesis, k, db);

        let diff_b = Header::new(Self::new_header_test(1001), vec![genesis_hash]);

        let diff_c = Header::new(Self::new_header_test(1002), vec![genesis_hash]);

        let diff_d = Header::new(Self::new_header_test(1003), vec![genesis_hash]);

        let diff_e = Header::new(Self::new_header_test(1004), vec![genesis_hash]);

        let diff_f = Header::new(
            Self::new_header_test(1005),
            vec![diff_b.hash(), diff_c.hash()],
        );
        let diff_h = Header::new(
            Self::new_header_test(1006),
            vec![diff_e.hash(), diff_d.hash(), diff_c.hash()],
        );
        let diff_i = Header::new(Self::new_header_test(1007), vec![diff_e.hash()]);
        let diff_k = Header::new(
            Self::new_header_test(1008),
            vec![diff_b.hash(), diff_h.hash(), diff_i.hash()],
        );
        let diff_l = Header::new(
            Self::new_header_test(1009),
            vec![diff_d.hash(), diff_i.hash()],
        );
        let diff_j = Header::new(
            Self::new_header_test(1010),
            vec![diff_f.hash(), diff_h.hash()],
        );
        let diff_m = Header::new(
            Self::new_header_test(1011),
            vec![diff_f.hash(), diff_k.hash()],
        );
        let diff_p = Header::new(
            Self::new_header_test(1012),
            vec![diff_j.hash(), diff_m.hash()],
        );
        let diff_v = Header::new(
            Self::new_header_test(1013),
            vec![diff_k.hash(), diff_c.hash(), diff_l.hash()],
        );

        let mut result = vec![];

        result.push(diff_b);
        result.push(diff_c);
        result.push(diff_d);
        result.push(diff_e);
        result.push(diff_f);
        result.push(diff_h);
        result.push(diff_i);
        result.push(diff_j);
        result.push(diff_k);
        result.push(diff_l);
        result.push(diff_m);
        result.push(diff_p);
        result.push(diff_v);

        (result, dag)
    }

    fn new_dag_half_diff_full_for_test() -> BlockDAG {
        let (headers, mut dag) = Self::new_dag_diff_half_leaf_test();
        headers.into_iter().for_each(|header| {
            dag.commit_header(&header);
        });
        dag
    }

    fn new_dag_diff_full_for_test() -> BlockDAG {
        let (headers, mut dag) = Self::new_dag_diff_leaf_test();
        headers.into_iter().for_each(|header| {
            dag.commit_header(&header);
        });
        dag
    }

    fn new_dag_lay1_for_test() -> BlockDAG {
        let (headers, mut dag) = Self::new_basic_dag_test();
        headers.get(0..4).unwrap().into_iter().for_each(|header| {
            dag.commit_header(&header);
        });
        dag
    }

    fn new_dag_genesis_for_test() -> BlockDAG {
        let (_, mut dag) = Self::new_basic_dag_test();
        dag
    }

    fn new_dag_full_for_test_2() -> BlockDAG {
        let (headers, mut dag) = Self::new_basic_dag_test();
        headers.into_iter().for_each(|header| {
            dag.commit_header(&header);
        });
        dag
    }

    pub fn build_sync_block_dag(store: Arc<Storage>) -> Self {
        let dag = Self::new_dag_half_diff_full_for_test();
        let accumulator_store = store.get_accumulator_store(AccumulatorStoreType::SyncDag);
        let accumulator = MerkleAccumulator::new_empty(accumulator_store.clone());
        let accumulator_snapshot = store.get_accumulator_snapshot_storage();

        let mut next_parents = HashSet::new();
        let genesis_hash = dag.get_genesis_hash();
        let genesis_leaf = HashValue::sha3_256_of(&[genesis_hash].encode().unwrap());
        next_parents.insert(genesis_hash);
        accumulator
            .append(&[genesis_leaf])
            .expect("appending genesis for sync accumulator must be successful");
        accumulator_snapshot
            .put(
                genesis_leaf,
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
                    Ok(children) => {
                        children.iter().for_each(|child| {
                            let result_parents = dag.get_parents(*child);
                            match result_parents {
                                Ok(parents) => {
                                    relationship_set.extend(parents.into_iter().map(|parent| {
                                        return RelationshipPair {
                                            parent,
                                            child: *child,
                                        };
                                    }));
                                }
                                Err(error) => {
                                    panic!("failed to get the parents when building sync accumulator, error message: {}", error.to_string());
                                }
                            }
                        });
                        children_set.extend(children.clone().into_iter());
                        // relationship_set.extend(children.into_iter().map(|child| {
                        //     return RelationshipPair {
                        //         parent,
                        //         child,
                        //     }
                        // }).collect::<HashSet<RelationshipPair>>());
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
                println!(
                    "insert a node, leaf hash = {}, accumulator root = {}",
                    accumulator_leaf,
                    accumulator.get_info().accumulator_root
                );
                next_parents = children_set;
            }
        }

        accumulator.flush().unwrap();

        println!(
            "finish to build accumulator, its info is: {:?}",
            accumulator.get_info()
        );

        return SyncBlockDag {
            dag: Arc::new(dag),
            accumulator,
            accumulator_snapshot: accumulator_snapshot.clone(),
        };
    }
}
