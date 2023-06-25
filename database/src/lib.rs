mod access;
mod cache;
mod db;
mod errors;
mod item;
mod key;
mod writer;

pub mod prelude {
    use crate::{db, errors};

    pub use super::{
        access::CachedDbAccess,
        cache::Cache,
        item::CachedDbItem,
        key::{DbKey, SEP, SEP_SIZE},
        writer::{BatchDbWriter, DbWriter, DirectDbWriter},
    };
    pub use db::{open_db, DB};
    pub use errors::{StoreError, StoreResult, StoreResultEmptyTuple, StoreResultExtensions};
}
