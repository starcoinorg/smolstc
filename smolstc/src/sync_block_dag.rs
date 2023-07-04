use std::{sync::Arc, collections::HashSet};

use consensus::blockdag::BlockDAG;
use consensus_types::{header::{Header, ConsensusHeader}, blockhash::ORIGIN};
use database::prelude::open_db;
use starcoin_accumulator::{MerkleAccumulator, Accumulator, node::AccumulatorStoreType};
use starcoin_config::RocksdbConfig;
use starcoin_crypto::HashValue;
use starcoin_storage::{accumulator::AccumulatorStorage, storage::StorageInstance, Storage, cache_storage::CacheStorage, db_storage::DBStorage, StorageVersion, Store};
use starcoin_types::block::BlockHeader;



pub struct SyncBlockDag {
    dag: BlockDAG,
    accumulator: MerkleAccumulator,
}

impl SyncBlockDag {
    pub fn new_for_test() -> Self {
        let genesis = Header::new(BlockHeader::random(), vec![HashValue::new(ORIGIN)]);
        let genesis_hash = genesis.hash();
    
        let k = 16;
        let db_path = std::env::temp_dir().join("smolstc");
        std::fs::remove_dir_all(db_path.clone()).expect("Failed to delete temporary directory");
        println!("db path:{}", db_path.to_string_lossy());
    
        let db = open_db(db_path, true, 1);
    
        let mut dag = BlockDAG::new(genesis, k, db, 1024);
    
        let block = Header::new(
            starcoin_types::block::BlockHeader::random(),
            vec![genesis_hash],
        );
        dag.commit_header(block);

        SyncBlockDag { 
            dag,
            accumulator: init_dag_accumulator_for_test(&dag),
        }
    }

    pub fn hash_blocks(hashes: &[HashValue]) -> HashValue {
        HashValue::sha3_256_of(&hashes.into_iter().fold([].to_vec(), |mut collect, hash| {
            collect.extend(hash.into_iter());
            collect
        }))
    }
}


fn init_dag_accumulator_for_test(dag: &BlockDAG) -> MerkleAccumulator {
    let db_storage = DBStorage::open_with_cfs(
      "./accumulator/starcoindb".to_string(),
      StorageVersion::current_version()
          .get_column_family_names()
          .to_vec(),
      true,
      Default::default(),
      None,
    ).unwrap();
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,)).unwrap());
    let mut accumulator = MerkleAccumulator::new_empty(storage.get_accumulator_store(AccumulatorStoreType::Block),);

    let mut next_parents = HashSet::new();
    next_parents.insert(HashValue::new(ORIGIN));
    accumulator.append(&[HashValue::new(ORIGIN)]).unwrap();
    loop {
        let mut children_set = HashSet::new();
        let mut have_children = false;
        next_parents.into_iter().for_each(|parent| {
          let result_children = dag.get_children(parent);
          match result_children {
              Ok(children) => {
                  if children.is_empty() {
                      children_set.insert(parent);
                  } else {
                      have_children = true;
                      children_set.extend(children.into_iter());
                  }
              }
              Err(error) => {
                  panic!("failed to get the children, {}", error.to_string());
              }
          }
        });
        if !have_children {
            break;
        } else {
            have_children = false;
            accumulator.append(&[SyncBlockDag::hash_blocks(&children_set.iter().cloned().collect::<Vec<_>>())]);
            next_parents = children_set;
        }
    }

    return accumulator;
}