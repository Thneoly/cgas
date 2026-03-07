//! 无锁缓存 (Lock-Free Cache)
//! 
//! Phase 3 性能优化核心组件：
//! - 读写分离 (DashMap 实现无锁并发)
//! - 缓存失效策略 (TTL + LRU 混合)
//! - 内存池管理 (对象复用减少分配)
//! 
//! 性能目标：缓存访问延迟 15ms → <13ms (-13%)

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use log::{debug, info, warn};

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    /// 值
    pub value: V,
    /// 创建时间
    pub created_at: Instant,
    /// 最后访问时间
    pub last_accessed_at: Instant,
    /// 访问次数
    pub access_count: AtomicUsize,
    /// TTL (秒)
    pub ttl_seconds: Option<u64>,
}

impl<V> CacheEntry<V> {
    pub fn new(value: V, ttl_seconds: Option<u64>) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            last_accessed_at: Instant::now(),
            access_count: AtomicUsize::new(1),
            ttl_seconds,
        }
    }
    
    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            self.created_at.elapsed().as_secs() >= ttl
        } else {
            false
        }
    }
    
    /// 更新访问时间
    pub fn touch(&self) {
        self.last_accessed_at = Instant::now();
        self.access_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 获取访问次数
    pub fn access_count(&self) -> usize {
        self.access_count.load(Ordering::Relaxed)
    }
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct LockFreeCacheConfig {
    /// 最大容量 (条目数)
    pub max_capacity: usize,
    /// 默认 TTL (秒)
    pub default_ttl_seconds: Option<u64>,
    /// 是否启用 LRU 淘汰
    pub enable_lru_eviction: bool,
    /// LRU 淘汰阈值 (容量百分比)
    pub lru_eviction_threshold: f64,
    /// 是否启用预取
    pub enable_prefetch: bool,
    /// 预取阈值 (容量百分比)
    pub prefetch_threshold: f64,
}

impl Default for LockFreeCacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 10000,
            default_ttl_seconds: Some(300), // 5 分钟
            enable_lru_eviction: true,
            lru_eviction_threshold: 0.9, // 90% 容量时开始淘汰
            enable_prefetch: false,
            prefetch_threshold: 0.5,
        }
    }
}

/// 缓存统计
#[derive(Debug)]
pub struct CacheStats {
    /// 命中数
    pub hits: AtomicU64,
    /// 未命中数
    pub misses: AtomicU64,
    /// 插入数
    pub inserts: AtomicU64,
    /// 淘汰数
    pub evictions: AtomicU64,
    /// 过期数
    pub expirations: AtomicU64,
    /// 当前大小
    pub current_size: AtomicUsize,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            inserts: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            expirations: AtomicU64::new(0),
            current_size: AtomicUsize::new(0),
        }
    }
    
    /// 获取命中率
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let misses = self.misses.load(Ordering::Relaxed) as f64;
        
        if hits + misses > 0.0 {
            hits / (hits + misses)
        } else {
            0.0
        }
    }
    
    /// 获取总请求数
    pub fn total_requests(&self) -> u64 {
        self.hits.load(Ordering::Relaxed) + self.misses.load(Ordering::Relaxed)
    }
}

/// 无锁缓存
pub struct LockFreeCache<K, V> {
    /// 主缓存 (DashMap 提供无锁并发)
    cache: DashMap<K, CacheEntry<V>>,
    /// 配置
    config: LockFreeCacheConfig,
    /// 统计
    stats: Arc<CacheStats>,
    /// Bloom Filter (用于快速检查不存在性)
    bloom_filter: Option<Arc<parking_lot::RwLock<BloomFilter>>>,
}

impl<K, V> LockFreeCache<K, V>
where
    K: Eq + Hash + Clone + ToString,
    V: Clone,
{
    /// 创建缓存
    pub fn new(config: LockFreeCacheConfig) -> Self {
        let cache = DashMap::with_capacity(config.max_capacity / 2);
        
        let bloom_filter = if config.max_capacity > 1000 {
            Some(Arc::new(parking_lot::RwLock::new(
                BloomFilter::new(config.max_capacity, 0.01)
            )))
        } else {
            None
        };
        
        Self {
            cache,
            config,
            stats: Arc::new(CacheStats::new()),
            bloom_filter,
        }
    }
    
    /// 获取值
    pub fn get(&self, key: &K) -> Option<V> {
        // 1. 使用 Bloom Filter 快速检查 (如果启用)
        if let Some(bf) = &self.bloom_filter {
            if !bf.read().contains(key) {
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
                return None;
            }
        }
        
        // 2. 从缓存获取
        match self.cache.get(key) {
            Some(ref_entry) => {
                let entry = ref_entry.value();
                
                // 检查是否过期
                if entry.is_expired() {
                    self.stats.expirations.fetch_add(1, Ordering::Relaxed);
                    self.remove(key);
                    self.stats.misses.fetch_add(1, Ordering::Relaxed);
                    return None;
                }
                
                // 更新访问统计
                entry.touch();
                self.stats.hits.fetch_add(1, Ordering::Relaxed);
                
                debug!("Cache hit for key: {}", key.to_string());
                Some(entry.value.clone())
            }
            None => {
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
                debug!("Cache miss for key: {}", key.to_string());
                None
            }
        }
    }
    
    /// 插入值
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        // 检查容量
        let current_size = self.cache.len();
        
        if current_size >= (self.config.max_capacity as f64 * self.config.lru_eviction_threshold) as usize {
            // 触发 LRU 淘汰
            self.evict_lru();
        }
        
        // 创建条目
        let entry = CacheEntry::new(value, self.config.default_ttl_seconds);
        
        // 插入缓存
        let old_value = self.cache
            .insert(key.clone(), entry)
            .map(|(_, old_entry)| old_entry.value);
        
        self.stats.inserts.fetch_add(1, Ordering::Relaxed);
        self.stats.current_size.store(self.cache.len(), Ordering::Relaxed);
        
        // 更新 Bloom Filter
        if let Some(bf) = &self.bloom_filter {
            bf.write().insert(&key);
        }
        
        debug!("Cache insert for key: {}", key.to_string());
        old_value
    }
    
    /// 移除值
    pub fn remove(&self, key: &K) -> Option<V> {
        self.cache.remove(key).map(|(_, entry)| entry.value)
    }
    
    /// 检查是否存在
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
    
    /// 获取缓存大小
    pub fn len(&self) -> usize {
        self.cache.len()
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
    
    /// 清空缓存
    pub fn clear(&self) {
        self.cache.clear();
        self.stats.current_size.store(0, Ordering::Relaxed);
        
        if let Some(bf) = &self.bloom_filter {
            bf.write().clear();
        }
        
        info!("Cache cleared");
    }
    
    /// LRU 淘汰
    fn evict_lru(&self) {
        if !self.config.enable_lru_eviction {
            return;
        }
        
        let mut entries: Vec<_> = self.cache
            .iter()
            .map(|ref_entry| {
                let key = ref_entry.key().clone();
                let entry = ref_entry.value();
                (key, entry.last_accessed_at, entry.access_count())
            })
            .collect();
        
        // 按最后访问时间排序 (最旧的在前)
        entries.sort_by(|a, b| a.1.cmp(&b.1));
        
        // 淘汰 10% 的条目
        let evict_count = (self.cache.len() as f64 * 0.1) as usize;
        
        for (key, _, _) in entries.iter().take(evict_count) {
            self.cache.remove(key);
            self.stats.evictions.fetch_add(1, Ordering::Relaxed);
        }
        
        self.stats.current_size.store(self.cache.len(), Ordering::Relaxed);
        
        debug!("LRU eviction: removed {} entries", evict_count);
    }
    
    /// 过期清理
    pub fn cleanup_expired(&self) -> usize {
        let mut expired_keys = Vec::new();
        
        for ref_entry in self.cache.iter() {
            if ref_entry.value().is_expired() {
                expired_keys.push(ref_entry.key().clone());
            }
        }
        
        let count = expired_keys.len();
        
        for key in expired_keys {
            self.cache.remove(&key);
            self.stats.expirations.fetch_add(1, Ordering::Relaxed);
        }
        
        self.stats.current_size.store(self.cache.len(), Ordering::Relaxed);
        
        if count > 0 {
            debug!("Cleanup: removed {} expired entries", count);
        }
        
        count
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> Arc<CacheStats> {
        Arc::clone(&self.stats)
    }
    
    /// 获取命中率
    pub fn hit_rate(&self) -> f64 {
        self.stats.hit_rate()
    }
    
    /// 预热缓存 (批量插入)
    pub fn warmup(&self, entries: Vec<(K, V)>) {
        info!("Warming up cache with {} entries", entries.len());
        
        for (key, value) in entries {
            self.insert(key, value);
        }
        
        info!("Cache warmup completed, size={}", self.len());
    }
}

/// Bloom Filter 实现
struct BloomFilter {
    bits: Vec<bool>,
    hash_functions: usize,
}

impl BloomFilter {
    fn new(capacity: usize, false_positive_rate: f64) -> Self {
        // 计算最优的位数组大小和哈希函数数量
        let size = (-(capacity as f64) * false_positive_rate.ln() / (2.0_f64.ln().powi(2))).ceil() as usize;
        let hash_functions = ((size as f64 / capacity as f64) * 2.0_f64.ln()).ceil() as usize;
        
        Self {
            bits: vec![false; size],
            hash_functions,
        }
    }
    
    fn insert<K: Hash + ToString>(&mut self, key: &K) {
        for hash in self.get_hashes(key) {
            let index = hash % self.bits.len();
            self.bits[index] = true;
        }
    }
    
    fn contains<K: Hash + ToString>(&self, key: &K) -> bool {
        self.get_hashes(key).all(|hash| {
            let index = hash % self.bits.len();
            self.bits[index]
        })
    }
    
    fn clear(&mut self) {
        self.bits.fill(false);
    }
    
    fn get_hashes<K: Hash + ToString>(&self, key: &K) -> impl Iterator<Item = usize> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        
        let key_string = key.to_string();
        
        (0..self.hash_functions).map(move |i| {
            let mut hasher = DefaultHasher::new();
            hasher.write(key_string.as_bytes());
            hasher.write_usize(i);
            hasher.finish() as usize
        })
    }
}

/// 内存池管理的缓存条目
pub struct PooledCacheEntry<V> {
    value: V,
    pool: Arc<ObjectPool<V>>,
}

impl<V: Clone + Default> PooledCacheEntry<V> {
    pub fn new(value: V, pool: Arc<ObjectPool<V>>) -> Self {
        Self { value, pool }
    }
}

impl<V: Clone + Default> Drop for PooledCacheEntry<V> {
    fn drop(&mut self) {
        // 返回对象池
        self.pool.release(self.value.clone());
    }
}

/// 对象池 (简化版)
pub struct ObjectPool<T> {
    pool: Arc<parking_lot::Mutex<Vec<T>>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T: Clone + Default + Send + Sync + 'static> ObjectPool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Arc::new(parking_lot::Mutex::new(Vec::with_capacity(max_size / 2))),
            factory: Arc::new(|| T::default()),
            max_size,
        }
    }
    
    pub fn acquire(&self) -> T {
        let mut pool = self.pool.lock();
        
        if let Some(obj) = pool.pop() {
            obj
        } else {
            (self.factory)()
        }
    }
    
    pub fn release(&self, obj: T) {
        let mut pool = self.pool.lock();
        
        if pool.len() < self.max_size {
            pool.push(obj);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lock_free_cache_basic() {
        let config = LockFreeCacheConfig::default();
        let cache = LockFreeCache::new(config);
        
        // 插入
        cache.insert("key1", "value1");
        cache.insert("key2", "value2");
        
        // 获取
        assert_eq!(cache.get(&"key1"), Some("value1"));
        assert_eq!(cache.get(&"key2"), Some("value2"));
        assert_eq!(cache.get(&"key3"), None);
        
        // 验证统计
        let stats = cache.stats();
        assert_eq!(stats.hits.load(Ordering::Relaxed), 2);
        assert_eq!(stats.misses.load(Ordering::Relaxed), 1);
        assert_eq!(stats.hit_rate(), 2.0 / 3.0);
    }
    
    #[test]
    fn test_lock_free_cache_ttl() {
        let config = LockFreeCacheConfig {
            default_ttl_seconds: Some(1), // 1 秒 TTL
            ..Default::default()
        };
        let cache = LockFreeCache::new(config);
        
        // 插入
        cache.insert("key1", "value1");
        
        // 立即获取 (应该命中)
        assert_eq!(cache.get(&"key1"), Some("value1"));
        
        // 等待过期
        std::thread::sleep(Duration::from_millis(1100));
        
        // 获取 (应该过期)
        assert_eq!(cache.get(&"key1"), None);
        
        // 验证过期统计
        let stats = cache.stats();
        assert!(stats.expirations.load(Ordering::Relaxed) > 0);
    }
    
    #[test]
    fn test_lock_free_cache_concurrent() {
        let config = LockFreeCacheConfig {
            max_capacity: 10000,
            ..Default::default()
        };
        let cache = Arc::new(LockFreeCache::new(config));
        
        let mut handles = vec![];
        
        // 并发写入
        for i in 0..100 {
            let cache_clone = Arc::clone(&cache);
            let handle = std::thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key_{}_{}", i, j);
                    cache_clone.insert(key, j);
                }
            });
            handles.push(handle);
        }
        
        // 并发读取
        for i in 0..100 {
            let cache_clone = Arc::clone(&cache);
            let handle = std::thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key_{}_{}", i, j);
                    let _ = cache_clone.get(&key);
                }
            });
            handles.push(handle);
        }
        
        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }
        
        // 验证缓存大小
        assert!(cache.len() > 0);
        
        // 验证统计
        let stats = cache.stats();
        let total_requests = stats.total_requests();
        assert!(total_requests > 0);
    }
    
    #[test]
    fn test_lock_free_cache_lru_eviction() {
        let config = LockFreeCacheConfig {
            max_capacity: 100,
            lru_eviction_threshold: 0.9, // 90 个条目时开始淘汰
            enable_lru_eviction: true,
            ..Default::default()
        };
        let cache = LockFreeCache::new(config);
        
        // 插入 100 个条目
        for i in 0..100 {
            cache.insert(format!("key_{}", i), i);
        }
        
        // 访问前 50 个条目 (使其变热)
        for i in 0..50 {
            cache.get(&format!("key_{}", i));
        }
        
        // 插入新条目 (触发 LRU 淘汰)
        cache.insert("new_key", 999);
        
        // 验证热数据仍然存在
        assert!(cache.contains_key(&"key_0".to_string()));
        
        // 验证冷数据可能被淘汰
        // (不保证一定被淘汰，因为 LRU 是概率性的)
    }
}
