
use std::sync::Arc;

use consensus_types::blockhash::BlockHashes;
use database::prelude::DB;
use ghostdag::ghostdata::{DbGhostdagStore, GhostdagStore, GhostdagData};
use reachability::relations::{DbRelationsStore, RelationsStore, RelationsStoreReader};
use starcoin_crypto::HashValue as Hash;
use anyhow::{Result, Result::Ok, Ok};

struct FlexiDagConsensus {
    relation_store: DbRelationsStore,
    flexi_strore: DbGhostdagStore,
}

impl FlexiDagConsensus {
    /// the testing code for generating the testing data
    pub fn setup_for_test() -> Self {
        let db = Arc::new(DB::open_default(std::env::current_dir().unwrap().to_owned().to_str().unwrap()).unwrap());
        let mut relation_store = DbRelationsStore::new(db.clone(), 0.into(), 1024 * 1024 * 10);

        let genesis = 0.into();
        relation_store.insert(genesis, BlockHashes::new([0.into()].to_vec())).unwrap();

        let (b, c, d, e) = (Hash::from_slice(b"B").unwrap(), 
                                                Hash::from_slice(b"C").unwrap(),
                                                Hash::from_slice(b"D").unwrap(),
                                                Hash::from_slice(b"E").unwrap(),
                                                );

        let parent = BlockHashes::new([genesis].to_vec());
        relation_store.insert(b, parent.clone()).unwrap();
        relation_store.insert(c, parent.clone()).unwrap();
        relation_store.insert(d, parent.clone()).unwrap();
        relation_store.insert(e, parent.clone()).unwrap();

        let f = Hash::from_slice(b"F").unwrap();                                                
        let parent = BlockHashes::new([b, c].to_vec());
        relation_store.insert(f, parent).unwrap();

        let h = Hash::from_slice(b"H").unwrap();                                                
        let parent = BlockHashes::new([c, d, e].to_vec());
        relation_store.insert(h, parent).unwrap();

        let i = Hash::from_slice(b"I").unwrap();                                                
        let parent = BlockHashes::new([e].to_vec());
        relation_store.insert(i, parent).unwrap();

        let j = Hash::from_slice(b"J").unwrap();                                                
        let parent = BlockHashes::new([f, h].to_vec());
        relation_store.insert(j, parent).unwrap();

        let k = Hash::from_slice(b"K").unwrap();                                                
        let parent = BlockHashes::new([h, i].to_vec());
        relation_store.insert(k, parent).unwrap();

        let l = Hash::from_slice(b"L").unwrap();                                                
        let parent = BlockHashes::new([d, i].to_vec());
        relation_store.insert(l, parent).unwrap();

        let m = Hash::from_slice(b"M").unwrap();                                                
        let parent = BlockHashes::new([f, k].to_vec());
        relation_store.insert(m, parent).unwrap();

        let mut flexi_strore = DbGhostdagStore::new(db.clone(), 0.into(), 1024 * 1024 * 10); 
        let k = 3;
        let mut genesis_node = GhostdagData::new_with_selected_parent(genesis, k);
        genesis_node.blue_score = 0; 
        flexi_strore.insert(genesis, Arc::new(genesis_node));

        FlexiDagConsensus {
            relation_store, 
            flexi_strore,
        }
    }

    pub fn get_bmax(&self, begin: Hash) -> anyhow::Result<Hash> {
        let result_children = self.relation_store.get_children(begin);
        match result_children {
            std::result::Result::Ok(children) => {

            },
            Err(_) => {

            },

        }
       
        Ok(0.into())
    }
}