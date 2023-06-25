use super::DagCache;
use starcoin_storage::{cache_storage::CacheStorage, storage::InnerStore, FLEXI_DAG_PREFIX_NAME};
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Clone)]
pub struct Cache<TKey> {
    cf_name: &'static str,
    cache: Arc<CacheStorage>,
    _phantom: PhantomData<TKey>,
}

impl<TKey: Clone + std::hash::Hash + Eq + Send + Sync + AsRef<[u8]>> DagCache for Cache<TKey> {
    type TKey = TKey;
    type TData = Vec<u8>;

    fn new_with_capacity(size: u64) -> Self {
        Self {
            cf_name: FLEXI_DAG_PREFIX_NAME,
            cache: Arc::new(CacheStorage::new_with_capacity(size as usize, None)),
            _phantom: Default::default(),
        }
    }

    fn get(&self, key: &Self::TKey) -> Option<Self::TData> {
        self.cache
            .get(self.cf_name, key.as_ref().to_vec())
            .expect("Failed on cache read")
    }

    fn contains_key(&self, key: &Self::TKey) -> bool {
        self.get(key).is_some()
    }

    fn insert(&self, key: Self::TKey, data: Self::TData) {
        self.cache
            .put(self.cf_name, key.as_ref().to_vec(), data)
            .expect("Failed to write cache");
    }

    fn remove(&self, key: &Self::TKey) {
        self.cache
            .remove(self.cf_name, key.as_ref().to_vec())
            .expect("Failed to remove from cache")
    }

    fn remove_many(&self, key_iter: &mut impl Iterator<Item = Self::TKey>) {
        key_iter.for_each(|k| self.remove(&k));
    }

    fn remove_all(&self) {
        self.cache.remove_all();
    }
}
