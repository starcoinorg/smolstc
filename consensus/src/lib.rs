mod dag;

use std::collections::{HashMap, HashSet};

pub trait BlockDag<Block: BlockTrait> {
    // Add a new block to the BlockDAG
    fn add_block(&mut self, block: Block) -> Result<(), String>;

    // get the block dag order
    fn get_topological_order(&self) -> Vec<Block>;
}


// Define a BlockTrait trait for handling basic operations on blocks
pub trait BlockTrait: Clone {
    // Get the hash value of the block
    fn get_hash(&self) -> String;

    // Get the hash value list of the parent nodes of the block
    fn get_parent_hashes(&self) -> Vec<String>;

    // Get the work of the block
    fn get_pow_work(&self) -> u64;

    // Get the hash value list of all the blocks in the anti-cone of the block
    fn get_anticone_hashes(&self) -> HashSet<String>;
}


pub struct GhostDAG<Block: BlockTrait> {
    genesis_block: Option<Block>,
    blocks: HashMap<String, Block>,
    blue_nodes: HashSet<String>,
    blue_score: u64,
    blue_work: HashMap<String, u64>,
}


impl<Block: BlockTrait> BlockDag<Block> for GhostDAG<Block> {
    fn add_block(&mut self, block: Block) -> Result<(), String> {
        let hash = block.get_hash();
        if self.blocks.contains_key(&hash) {
            return Err("Block already exists in the GhostDAG".to_string());
        }
        self.blocks.insert(hash.clone(), block.clone());
        //TODO: get parents from Dag tips.
        let parents = block.get_parent_hashes().iter().filter_map(|hash| self.blocks.get(hash)).collect();

        let is_blue = self.is_block_blue(&block, &parents);

        if is_blue {
            self.blue_nodes.insert(hash.clone());
            self.blue_score += block.get_pow_work();
            self.update_blue_anticone_sizes(&block);
            self.update_mergesets(&block);
        }

        self.update_anticone_sizes(&block, &parents);
        self.update_red_mergesets(&block, &parents);

        Ok(())
    }

    fn get_topological_order(&self) -> Vec<Block> {
        unimplemented!()
    }
}

impl<Block: BlockTrait> GhostDAG<Block> {
    pub fn blue_work(&self, parents: &[&Block]) -> u64 {
        let mut work = 0;
        for block in parents {
            work += self.blue_score(&block);
        }
        work
    }
    pub fn blue_score(&self, block: &Block) -> u64 {
        let hash = block.get_hash();
        if self.blue_work.contains_key(&hash) {
            self.blue_work[&hash]
        } else {
            0
        }
    }
    pub fn is_block_blue(&self,block:&Block, tips:&[&Block]) -> bool {
        if parents.is_empty() {
            return true;
        }

        // 判断是否满足工作量证明要求
        let pow_work = block.get_pow_work();
        let blue_work = self.blue_score(&block.get_parent_hashes());
        if pow_work <= blue_work {
            return false;
        }

        // 判断是否满足蓝色子图大小要求
        let blues_anticone_sizes = self.blues_anticone_sizes(&parents);
        let mut blue_size = 0;
        for hash in &block.get_anticone_hashes() {
            if blues_anticone_sizes.contains_key(hash) {
                blue_size += blues_anticone_sizes[hash];
            }
        }
        let k = self.k_param();
        if blue_size < k {
            return false;
        }

        true
    }
}