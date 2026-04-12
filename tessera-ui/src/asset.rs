//! Asset reading abstractions for generated resource handles.
//!
//! ## Usage
//!
//! Use generated asset constants and call [`AssetExt::read`] to load raw bytes.

use std::{
    any::TypeId,
    collections::HashMap,
    io,
    sync::{Arc, OnceLock, RwLock, RwLockWriteGuard},
};

type AssetCacheKey = (TypeId, u64);
type AssetCacheMap = HashMap<AssetCacheKey, AssetCacheEntry>;

const CACHE_MAX_ENTRIES: usize = 4096;
const CACHE_MAX_BYTES: usize = 64 * 1024 * 1024;

static ASSET_BYTES_CACHE: OnceLock<RwLock<AssetLruCache>> = OnceLock::new();

#[derive(Clone)]
struct AssetCacheEntry {
    bytes: Arc<[u8]>,
    len: usize,
    last_access_tick: u64,
}

#[derive(Default)]
struct AssetLruCache {
    entries: AssetCacheMap,
    total_bytes: usize,
    next_tick: u64,
}

impl AssetLruCache {
    fn get(&mut self, key: AssetCacheKey) -> Option<Arc<[u8]>> {
        let tick = self.bump_tick();
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.last_access_tick = tick;
            return Some(entry.bytes.clone());
        }
        None
    }

    fn insert(&mut self, key: AssetCacheKey, bytes: Arc<[u8]>) {
        let tick = self.bump_tick();
        let entry_len = bytes.len();
        let entry = AssetCacheEntry {
            len: entry_len,
            bytes,
            last_access_tick: tick,
        };

        if let Some(previous) = self.entries.insert(key, entry) {
            self.total_bytes = self.total_bytes.saturating_sub(previous.len);
        }
        self.total_bytes = self.total_bytes.saturating_add(entry_len);
        self.evict_if_needed();
    }

    fn bump_tick(&mut self) -> u64 {
        self.next_tick = self.next_tick.wrapping_add(1);
        self.next_tick
    }

    fn evict_if_needed(&mut self) {
        while (self.total_bytes > CACHE_MAX_BYTES || self.entries.len() > CACHE_MAX_ENTRIES)
            && self.entries.len() > 1
        {
            let Some(victim_key) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_access_tick)
                .map(|(key, _)| *key)
            else {
                break;
            };

            if let Some(removed) = self.entries.remove(&victim_key) {
                self.total_bytes = self.total_bytes.saturating_sub(removed.len);
            }
        }
    }
}

/// Trait implemented by generated asset handle types.
pub trait AssetExt: Copy {
    /// Read raw bytes for this asset.
    fn read(self) -> io::Result<Arc<[u8]>>;
}

/// Shared helper for generated asset readers that adds an in-memory LRU cache.
///
/// Each generated asset type should call this function from `AssetExt::read`,
/// passing its `index` as `asset_id` and a backend-specific loader closure.
#[doc(hidden)]
pub fn read_with_lru_cache<T, F>(asset_id: u64, loader: F) -> io::Result<Arc<[u8]>>
where
    T: 'static,
    F: FnOnce() -> io::Result<Arc<[u8]>>,
{
    let cache_key = (TypeId::of::<T>(), asset_id);

    {
        let mut cache = cache_write();
        if let Some(bytes) = cache.get(cache_key) {
            return Ok(bytes);
        }
    }

    let loaded = loader()?;

    let mut cache = cache_write();
    if let Some(bytes) = cache.get(cache_key) {
        return Ok(bytes);
    }

    cache.insert(cache_key, loaded.clone());
    Ok(loaded)
}

fn cache_instance() -> &'static RwLock<AssetLruCache> {
    ASSET_BYTES_CACHE.get_or_init(|| RwLock::new(AssetLruCache::default()))
}

fn cache_write() -> RwLockWriteGuard<'static, AssetLruCache> {
    match cache_instance().write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
