use rocksdb::WriteBatch;
use starcoin_storage::storage::InnerStore;

use crate::db::FLEXI_DAG_PREFIX_NAME;
use crate::prelude::DB;

/// Abstraction over direct/batched DB writing
pub trait DbWriter {
    fn put(&mut self, key: &[u8], value: Vec<u8>) -> Result<(), rocksdb::Error>;
    fn delete(&mut self, key: &[u8]) -> Result<(), rocksdb::Error>;
}

pub struct DirectDbWriter<'a> {
    db: &'a DB,
}

impl<'a> DirectDbWriter<'a> {
    pub fn new(db: &'a DB) -> Self {
        Self { db }
    }
}

impl DbWriter for DirectDbWriter<'_> {
    fn put(&mut self, key: &[u8], value: Vec<u8>) -> Result<(), rocksdb::Error> {
        self.db
            .put(FLEXI_DAG_PREFIX_NAME, key.to_owned(), value)
            .unwrap();
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), rocksdb::Error> {
        self.db
            .remove(FLEXI_DAG_PREFIX_NAME, key.to_owned())
            .unwrap();
        Ok(())
    }
}

pub struct BatchDbWriter<'a> {
    batch: &'a mut WriteBatch,
}

impl<'a> BatchDbWriter<'a> {
    pub fn new(batch: &'a mut WriteBatch) -> Self {
        Self { batch }
    }
}

impl DbWriter for BatchDbWriter<'_> {
    fn put(&mut self, key: &[u8], value: Vec<u8>) -> Result<(), rocksdb::Error> {
        self.batch.put(key, value);
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), rocksdb::Error> {
        self.batch.delete(key);
        Ok(())
    }
}

impl<T: DbWriter> DbWriter for &mut T {
    #[inline]
    fn put(&mut self, key: &[u8], value: Vec<u8>) -> Result<(), rocksdb::Error> {
        (*self).put(key, value)
    }

    #[inline]
    fn delete(&mut self, key: &[u8]) -> Result<(), rocksdb::Error> {
        (*self).delete(key)
    }
}
