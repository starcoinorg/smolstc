use crate::blockhash::{BlockLevel, ORIGIN};
use database::prelude::DB;
use database::prelude::{BatchDbWriter, CachedDbAccess, DirectDbWriter};
use database::prelude::{StoreError, StoreResult};
use rocksdb::WriteBatch;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue as Hash;
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use std::sync::Arc;
pub trait ConsensusHeader {
    fn parents_hash(&self) -> &[Hash];
    fn difficulty(&self) -> U256;
    fn hash(&self) -> Hash;
    fn timestamp(&self) -> u64;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    block_header: BlockHeader,
    parents_hash: Vec<Hash>,
}
impl Header {
    pub fn new(block_header: BlockHeader, parents_hash: Vec<Hash>) -> Self {
        Self {
            block_header,
            parents_hash,
        }
    }

    pub fn genesis_hash(&self) -> Hash {
        Hash::new(ORIGIN)
    }
}

impl ConsensusHeader for Header {
    fn parents_hash(&self) -> &[Hash] {
        &self.parents_hash
    }
    fn difficulty(&self) -> U256 {
        self.block_header.difficulty()
    }
    fn hash(&self) -> Hash {
        self.block_header.id()
    }

    fn timestamp(&self) -> u64 {
        self.block_header.timestamp()
    }
}

pub trait HeaderStoreReader {
    fn get_daa_score(&self, hash: Hash) -> Result<u64, StoreError>;
    fn get_blue_score(&self, hash: Hash) -> Result<u64, StoreError>;
    fn get_timestamp(&self, hash: Hash) -> Result<u64, StoreError>;
    fn get_difficulty(&self, hash: Hash) -> Result<U256, StoreError>;
    fn get_header(&self, hash: Hash) -> Result<Arc<Header>, StoreError>;
    fn get_header_with_block_level(&self, hash: Hash) -> Result<HeaderWithBlockLevel, StoreError>;
    fn get_compact_header_data(&self, hash: Hash) -> Result<CompactHeaderData, StoreError>;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HeaderWithBlockLevel {
    pub header: Arc<Header>,
    pub block_level: BlockLevel,
}

pub trait HeaderStore: HeaderStoreReader {
    // This is append only
    fn insert(
        &self,
        hash: Hash,
        header: Arc<Header>,
        block_level: BlockLevel,
    ) -> Result<(), StoreError>;
}

const HEADERS_STORE_PREFIX: &[u8] = b"headers";
const COMPACT_HEADER_DATA_STORE_PREFIX: &[u8] = b"compact-header-data";

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct CompactHeaderData {
    pub timestamp: u64,
    pub difficulty: U256,
}

/// A db + cache implementation of `HeaderStore` trait, with concurrency support.
#[derive(Clone)]
pub struct DbHeadersStore {
    db: Arc<DB>,
    compact_headers_access: CachedDbAccess<Hash, CompactHeaderData>,
    headers_access: CachedDbAccess<Hash, HeaderWithBlockLevel>,
}

impl DbHeadersStore {
    pub fn new(db: Arc<DB>, cache_size: u64) -> Self {
        Self {
            db: Arc::clone(&db),
            compact_headers_access: CachedDbAccess::new(
                Arc::clone(&db),
                cache_size,
                COMPACT_HEADER_DATA_STORE_PREFIX.to_vec(),
            ),
            headers_access: CachedDbAccess::new(db, cache_size, HEADERS_STORE_PREFIX.to_vec()),
        }
    }

    pub fn clone_with_new_cache(&self, cache_size: u64) -> Self {
        Self::new(Arc::clone(&self.db), cache_size)
    }

    pub fn has(&self, hash: Hash) -> StoreResult<bool> {
        self.headers_access.has(hash)
    }

    pub fn insert_batch(
        &self,
        batch: &mut WriteBatch,
        hash: Hash,
        header: Arc<Header>,
        block_level: BlockLevel,
    ) -> Result<(), StoreError> {
        if self.headers_access.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        self.headers_access.write(
            BatchDbWriter::new(batch),
            hash,
            HeaderWithBlockLevel {
                header: header.clone(),
                block_level,
            },
        )?;
        self.compact_headers_access.write(
            BatchDbWriter::new(batch),
            hash,
            CompactHeaderData {
                timestamp: header.timestamp(),
                difficulty: header.difficulty(),
            },
        )?;
        Ok(())
    }
}

impl HeaderStoreReader for DbHeadersStore {
    fn get_daa_score(&self, hash: Hash) -> Result<u64, StoreError> {
        unimplemented!()
    }

    fn get_blue_score(&self, hash: Hash) -> Result<u64, StoreError> {
        unimplemented!()
    }

    fn get_timestamp(&self, hash: Hash) -> Result<u64, StoreError> {
        if let Some(header_with_block_level) = self.headers_access.read_from_cache(hash) {
            return Ok(header_with_block_level.header.timestamp());
        }
        Ok(self.compact_headers_access.read(hash)?.timestamp)
    }

    fn get_difficulty(&self, hash: Hash) -> Result<U256, StoreError> {
        if let Some(header_with_block_level) = self.headers_access.read_from_cache(hash) {
            return Ok(header_with_block_level.header.difficulty());
        }
        Ok(self.compact_headers_access.read(hash)?.difficulty)
    }

    fn get_header(&self, hash: Hash) -> Result<Arc<Header>, StoreError> {
        Ok(self.headers_access.read(hash)?.header)
    }

    fn get_header_with_block_level(&self, hash: Hash) -> Result<HeaderWithBlockLevel, StoreError> {
        self.headers_access.read(hash)
    }

    fn get_compact_header_data(&self, hash: Hash) -> Result<CompactHeaderData, StoreError> {
        if let Some(header_with_block_level) = self.headers_access.read_from_cache(hash) {
            return Ok(CompactHeaderData {
                timestamp: header_with_block_level.header.timestamp(),
                difficulty: header_with_block_level.header.difficulty(),
            });
        }
        self.compact_headers_access.read(hash)
    }
}

impl HeaderStore for DbHeadersStore {
    fn insert(&self, hash: Hash, header: Arc<Header>, block_level: u8) -> Result<(), StoreError> {
        if self.headers_access.has(hash)? {
            return Err(StoreError::KeyAlreadyExists(hash.to_string()));
        }
        self.compact_headers_access.write(
            DirectDbWriter::new(&self.db),
            hash,
            CompactHeaderData {
                timestamp: header.timestamp(),
                difficulty: header.difficulty(),
            },
        )?;
        self.headers_access.write(
            DirectDbWriter::new(&self.db),
            hash,
            HeaderWithBlockLevel {
                header,
                block_level,
            },
        )?;
        Ok(())
    }
}
