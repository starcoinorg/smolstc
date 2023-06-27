
use std::{sync::Arc, collections::HashSet};

use consensus_types::blockhash::{BlockHashes, KType};
use database::prelude::{DB, StoreError};
use ghostdag::ghostdata::{DbGhostdagStore, GhostdagStore, GhostdagData, GhostdagStoreReader};
use reachability::relations::{DbRelationsStore, RelationsStore, RelationsStoreReader};
use starcoin_crypto::HashValue as Hash;
use std::cmp::Ordering::{Equal, Greater, Less};

#[derive(Debug, PartialEq, Eq, Ord)]
pub struct FlexiBlock {
    pub hash: Hash,
    pub score: u64,
}

impl std::cmp::PartialOrd for FlexiBlock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.score != other.score {
            other.score.partial_cmp(&self.score)
        } else {
            self.hash.partial_cmp(&other.hash)
        }
    }

    fn lt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Less))
    }

    fn le(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Less | Equal))
    }

    fn gt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Greater))
    }

    fn ge(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Greater | Equal))
    }
}

pub struct FlexiDagConsensus {
    relation_store: DbRelationsStore,
    flexi_strore: DbGhostdagStore,
    k: KType,
    pub bmax: Hash,
    pub bmax_score: u64,
}

impl FlexiDagConsensus {
    /// the testing code for generating the testing data
    pub fn setup_for_test() -> Self {
        let path = "./";
        let destroy_options = rocksdb::Options::default();
        DB::destroy(&destroy_options, path.clone()).unwrap();

        let db = Arc::new(DB::open_default(path).unwrap());

        let mut relation_store = DbRelationsStore::new(db.clone(), 0.into(), 1024 * 1024 * 10);

        let genesis = 0.into();
        relation_store.insert(genesis, BlockHashes::new([0.into()].to_vec())).unwrap();

        let (b, c, d, e) = (Hash::sha3_256_of(b"B"), 
                                                Hash::sha3_256_of(b"C"),
                                                Hash::sha3_256_of(b"D"),
                                                Hash::sha3_256_of(b"E"),
                                                );

        let parent = BlockHashes::new([genesis].to_vec());
        relation_store.insert(b, parent.clone()).unwrap();
        relation_store.insert(c, parent.clone()).unwrap();
        relation_store.insert(d, parent.clone()).unwrap();
        relation_store.insert(e, parent.clone()).unwrap();

        let f = Hash::sha3_256_of(b"F");                                                
        let parent = BlockHashes::new([b, c].to_vec());
        relation_store.insert(f, parent).unwrap();

        let h = Hash::sha3_256_of(b"H");                                                
        let parent = BlockHashes::new([c, d, e].to_vec());
        relation_store.insert(h, parent).unwrap();

        let i = Hash::sha3_256_of(b"I");                                                
        let parent = BlockHashes::new([e].to_vec());
        relation_store.insert(i, parent).unwrap();

        let j = Hash::sha3_256_of(b"J");                                                
        let parent = BlockHashes::new([f, h].to_vec());
        relation_store.insert(j, parent).unwrap();

        let k = Hash::sha3_256_of(b"K");                                                
        let parent = BlockHashes::new([h, i].to_vec());
        relation_store.insert(k, parent).unwrap();

        let l = Hash::sha3_256_of(b"L");                                                
        let parent = BlockHashes::new([d, i].to_vec());
        relation_store.insert(l, parent).unwrap();

        let m = Hash::sha3_256_of(b"M");                                                
        let parent = BlockHashes::new([f, k].to_vec());
        relation_store.insert(m, parent).unwrap();

        println!("b = {}", b);
        println!("c = {}", c);
        println!("d = {}", d);
        println!("e = {}", e);
        println!("f = {}", f);
        println!("h = {}", h);
        println!("i = {}", i);
        println!("j = {}", j);
        println!("k = {}", k);
        println!("l = {}", l);
        println!("m = {}", m);

        let flexi_strore = DbGhostdagStore::new(db.clone(), 0.into(), 1024 * 1024 * 10); 
        let k = 3;
        let mut genesis_block = GhostdagData::new_with_selected_parent(genesis, k);
        genesis_block.blue_score = 0; 
        flexi_strore.insert(genesis, Arc::new(genesis_block)).expect("insert muse be successful!");
        FlexiDagConsensus {
            relation_store, 
            flexi_strore,
            k,
            bmax: Hash::zero(),
            bmax_score: 0,
        }
    }

    pub fn scoring_from_genesis(&mut self) -> anyhow::Result<()> {
        self.scoring(0.into())
    }

    pub fn scoring(&mut self, begin: Hash) -> anyhow::Result<()> {
        let mut children = vec![begin];
        while !children.is_empty() {
            let mut next_children = vec![];
            children.into_iter().for_each(|child| {
                let result = self.scoring_child(child);
                match result {
                    Ok(sub_children) => {
                        let set = next_children.iter().cloned().collect::<HashSet<_>>();
                        next_children.extend(sub_children.into_iter().filter(|item| !set.contains(item)));
                    }
                    Err(error) => {
                        println!("failed to score child: {}", error.to_string());
                    }
                }
            });
            children = next_children;
        }
        return Ok(())
    }

    fn insert_block(&mut self, child: Hash, result_max_parent: anyhow::Result<(u64, Hash, Vec<Hash>)>) -> anyhow::Result<u64> {
        match result_max_parent {
            Ok((score, selected_parent, sub_parents)) => {
                let mut block = GhostdagData::new_with_selected_parent(selected_parent, self.k);
                block.mergeset_blues = Arc::new(sub_parents); 
                block.blue_score = score + 1;
                if self.bmax_score < block.blue_score {
                    self.bmax_score = block.blue_score;
                    self.bmax = child.clone();
                } else if self.bmax_score == score && self.bmax.cmp(&child) == std::cmp::Ordering::Greater {
                    self.bmax = child.clone();
                }
                let max_score = block.blue_score;
                println!("{} score {}", child, block.blue_score);
                self.flexi_strore.insert(child.clone(), Arc::new(block)).expect("insert a block should be successful"); 
                return anyhow::Result::Ok(max_score);
            },
            Err(error) => {
                panic!("some exception happened when selecting a parent: {}", error.to_string())
            },
        }
    }

    fn scoring_child(&mut self, begin: Hash) -> anyhow::Result<Vec<Hash>> {
        let result_children = self.relation_store.get_children(begin);
        match result_children {
            std::result::Result::Ok(children) => {
                let mut children = (*children).clone();
                children.retain(|item| item != &0.into());
                children.iter().for_each(|child| {
                    match self.flexi_strore.has(child.clone()) {
                        Ok(has) => {
                            if !has {
                                let result_max_parent = self.scoring_by_parent(child.clone());
                                self.insert_block(child.clone(), result_max_parent).expect("the insertion of a child must be successful");
                            }
                        }
                        Err(error) => {
                            panic!("some exception happened when trying to get a score: {}", error.to_string())
                        }
                    }
               });
                return Ok(children.into_iter().collect::<Vec<_>>());
            },
            Err(error) => {
                println!("some exception happened when getting children: {}", error.to_string());
                if let StoreError::KeyNotFound(_) = &error {
                    return Ok([].into()); // for end of the loop
                }
                return Err(anyhow::anyhow!("some exception happened when getting children: {}", error.to_string()));
            },
        }
    }

    fn scoring_by_parent(&mut self, hash: Hash) -> anyhow::Result<(u64, Hash, Vec<Hash>)> {
        let parents = self.relation_store.get_parents(hash)?;

        if parents.is_empty() {
            return Err(anyhow::anyhow!("the block must have parent(s)"));
        }

        let mut candidate_parents = vec![]; 
        parents.iter().for_each(|hash| {
            candidate_parents.push(FlexiBlock {
                hash: hash.clone(),
                score: self.ensure_parent_score(hash.clone()).expect("for now, the parent should exist!"),
            });
        });

        candidate_parents.sort();

        let mut max_score = 0u64;
        let selected_parent = candidate_parents.get(0).expect("for now, the parent should exist!");
        let init_score = selected_parent.score;
        max_score += init_score;

        let mut index = 1;
        let mut sub_parents = vec![];
        for block in &candidate_parents[1..] {
            if block.score == init_score && index <= self.k {
                max_score += init_score;
                index += 1;
                sub_parents.push(block.hash);
            } else {
                break;
            }
        }

        return Ok((max_score, selected_parent.hash, sub_parents));
    }

    fn ensure_parent_score(&mut self, hash: Hash) -> anyhow::Result<u64> {
        match self.flexi_strore.has(hash) {
            Ok(has) => {
                if has {
                    return anyhow::Result::Ok(self.flexi_strore.get_blue_score(hash).expect("for now, the parent should exist!"));
                } else {
                    let result_max_parent = self.scoring_by_parent(hash.clone());
                    return self.insert_block(hash, result_max_parent);
                }

            }
            Err(error) => {
                panic!("failed to having-query the db: {}", error.to_string());
            }
        }
    }

    pub fn chain(&mut self) -> anyhow::Result<Vec<Hash>> {
        self.scoring_from_genesis()?;

        let mut chain = vec![self.bmax];

        let mut next_block = self.bmax;
        loop {
            let result_block = self.flexi_strore.get_data(next_block);
            match result_block {
                Ok(block) => {
                    chain.push(block.selected_parent);
                    chain.extend((*block.mergeset_blues).clone());
                    next_block = block.selected_parent;
                }
                Err(error) => {
                    return Err(anyhow::anyhow!("failed to get data from dag db: {}", error.to_string()));
                }
            }
            if next_block == 0.into() {
                break;
            }
        }

        return self.chain_uncle_block(chain);
    }

    fn chain_uncle_block(&self, mut chain: Vec<Hash>) -> anyhow::Result<Vec<Hash>> {
        let mut blue_uncle_blocks = vec![];
        chain.iter().for_each(|blue_block| {
            let result_parents = self.relation_store.get_parents(blue_block.clone());
            match result_parents {
                Ok(parents) => {
                    parents.iter().for_each(|parent| {
                        if chain.contains(parent) {
                            return;
                        }

                        let connected_blocks = self.collect_connected_blocks(parent.clone()).expect("failed to collect connected blocks!");
                        let mut blue_count = 0;
                        connected_blocks.iter().for_each(|block| {
                            if chain.contains(block) {
                                blue_count += 1;
                            }
                        });
                        if self.k <= blue_count {
                            blue_uncle_blocks.push(parent.clone());
                        }
                    });
                }
                Err(error) => {
                    panic!("some exeption happened when chaining uncle block: {}", error.to_string());
                }
            }
        });

        chain.extend(blue_uncle_blocks);
        let mut flexi_chain = chain.into_iter().map(|hash| {
            FlexiBlock {
                hash: hash.clone(),
                score: self.flexi_strore.get_blue_score(hash.clone()).expect("blue score must exist"),
            }
        })
        .collect::<Vec<_>>();

        flexi_chain.sort();

        Ok(flexi_chain.into_iter().map(|block| block.hash).collect::<Vec<_>>())
    }

    fn collect_connected_blocks(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        let mut connected_blocks = vec![];

        // parents
        let mut next_blocks = vec![hash];
        while !next_blocks.is_empty() {
            let mut parents = vec![];
            next_blocks.into_iter().for_each(|block| {
                let result_parents = self.relation_store.get_parents(block).expect("the parent must exist");
                parents.extend(result_parents.iter().cloned().collect::<HashSet<_>>().into_iter().filter(|block| block != &0.into()));
            });
            next_blocks = parents.clone();
            connected_blocks.extend(parents);
        }

        // children
        let mut next_blocks = vec![hash];
        while !next_blocks.is_empty() {
            let mut children = vec![];
            next_blocks.into_iter().for_each(|block| {
                let result_children = self.relation_store.get_children(block);
                match result_children {
                    Ok(connected_children) => {
                        children.extend(connected_children.iter().cloned().collect::<HashSet<_>>().into_iter().filter(|block| block != &0.into()));
                    }
                    Err(error) => {
                        panic!("failed to get the children: {}", error.to_string());
                    }
                }
            });
            next_blocks = children.clone();
            connected_blocks.extend(children);
        }

        return Ok(connected_blocks);
    }
}