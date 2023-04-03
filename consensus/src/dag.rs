use std::sync::Arc;
use starcoin_crypto::HashValue;

pub trait DAGStoreReader {
    fn has(hash: HashValue) -> bool;
    fn get_children(hash: HashValue);
    fn get_parent(hash: HashValue);
}

pub type BlockHashes = Vec<HashValue>;

pub trait DAGStoreWriter {
    fn insert(&mut self, hash: HashValue, parents: BlockHashes);
}

#[derive(clone)]
pub struct PhantomDAG<T: DAGStoreReader> {
    genesis_hash: HashValue,
    dag_store_reader: Arc<T>,
}