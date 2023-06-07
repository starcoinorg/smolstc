use starcoin_config::RocksdbConfig;
pub(crate) use starcoin_storage::{db_storage::DBStorage, FLEXI_DAG_PREFIX_NAME};
use std::{path::PathBuf, sync::Arc};

/// The DB type used for Kaspad stores
pub type DB = DBStorage;

/// Creates or loads an existing DB from the provided directory path.
pub fn open_db(db_path: PathBuf, parallelism: usize) -> Arc<DB> {
    let mut config = RocksdbConfig::default();
    config.parallelism = parallelism as u64;

    let db = Arc::new(
        DB::open_with_cfs(
            db_path.as_path(),
            vec![FLEXI_DAG_PREFIX_NAME],
            false,
            config,
            None,
        )
        .unwrap(),
    );
    db
}

///// Deletes an existing DB if it exists
//pub fn delete_db(db_dir: PathBuf) {
//    if !db_dir.exists() {
//        return;
//    }
//    let options = rocksdb::Options::default();
//    let path = db_dir.to_str().unwrap();
//    DB::destroy(&options, path).expect("DB is expected to be deletable");
//}
