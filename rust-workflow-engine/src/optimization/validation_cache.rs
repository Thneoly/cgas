//! 校验缓存
//! 
//! 实现校验结果缓存，减少重复验证计算
//! Phase 2 Week 4 性能优化关键组件

use std::collections::HashMap;
use std::time::{Duration, Instant};
use log::{info, debug};

/// 校验缓存配置
#[derive(Debug, Clone)]
pub struct ValidationCacheConfig {
    /// 缓存容量 (最大条目数)
    pub capacity: usize,
    /// 缓存过期时间 (秒)
    pub ttl_secs: u64,
    /// 自动清理间隔 (秒)
    pub cleanup_interval_secs: u64,
}

impl Default for ValidationCacheConfig {
    fn default() -> Self {
        Self {
            capacity: 10000,
            ttl_secs: 300, // 5 分钟
            cleanup_interval_secs: 60, // 1 分钟
        }
    }
}

/// 校验缓存
pub struct ValidationCache {
    /// 缓存数据
    cache: HashMap<String, CacheEntry>,
    /// 配置
    config: ValidationCacheConfig,
    /// 缓存命中统计
    hits: u64,
    /// 缓存未命中统计
    misses: u64,
    /// 最后清理时间
    last_cleanup: Instant,
}

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 校验结果
    result: bool,
    /// 创建时间
    created_at: Instant,
    /// 访问次数
    access_count: u64,
}

/// 校验缓存键
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ValidationCacheKey {
    /// 请求哈希
    pub request_hash: String,
    /// 验证器版本
    pub verifier_version: String,
}

impl ValidationCache {
    /// 创建新的校验缓存
    pub fn new(config: ValidationCacheConfig) -> Self {
        Self {
            cache: HashMap::with_capacity(config.capacity / 2),
            config,
            hits: 0,
            misses: 0,
            last_cleanup: Instant::now(),
        }
    }

    /// 获取校验结果
    pub fn get(&mut self, key: &ValidationCacheKey) -> Option<bool> {
        // 检查是否需要清理
        self.maybe_cleanup();
        
        let cache_key = self.compute_cache_key(key);
        
        if let Some(entry) = self.cache.get_mut(&cache_key) {
            // 检查是否过期
            if entry.created_at.elapsed() > Duration::from_secs(self.config.ttl_secs) {
                self.cache.remove(&cache_key);
                self.misses += 1;
                return None;
            }
            
            // 缓存命中
            entry.access_count += 1;
            self.hits += 1;
            debug!("Validation cache hit: key={}, access_count={}", cache_key, entry.access_count);
            return Some(entry.result);
        }
        
        // 缓存未命中
        self.misses += 1;
        debug!("Validation cache miss: key={}", cache_key);
        None
    }

    /// 设置校验结果
    pub fn set(&mut self, key: &ValidationCacheKey, result: bool) {
        // 检查是否需要清理
        self.maybe_cleanup();
        
        let cache_key = self.compute_cache_key(key);
        
        // 检查缓存是否已满
        if self.cache.len() >= self.config.capacity {
            // 清理最少使用的条目
            self.evict_least_used();
        }
        
        let entry = CacheEntry {
            result,
            created_at: Instant::now(),
            access_count: 1,
        };
        
        self.cache.insert(cache_key, entry);
        debug!("Validation cache set: key={}, result={}", cache_key, result);
    }

    /// 计算缓存键
    fn compute_cache_key(&self, key: &ValidationCacheKey) -> String {
        format!("{}:{}", key.request_hash, key.verifier_version)
    }

    /// 清理过期缓存 (如果需要)
    fn maybe_cleanup(&mut self) {
        if self.last_cleanup.elapsed() > Duration::from_secs(self.config.cleanup_interval_secs) {
            self.cleanup();
            self.last_cleanup = Instant::now();
        }
    }

    /// 清理过期缓存
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        let ttl = Duration::from_secs(self.config.ttl_secs);
        
        let expired_keys: Vec<String> = self.cache
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.created_at) > ttl)
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in expired_keys {
            self.cache.remove(&key);
        }
        
        info!(
            "Validation cache cleanup: removed {} expired entries, hit_rate={:.2}%",
            expired_keys.len(),
            self.get_hit_rate()
        );
    }

    /// 淘汰最少使用的条目
    fn evict_least_used(&mut self) {
        if let Some(least_used_key) = self.cache
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(key, _)| key.clone())
        {
            self.cache.remove(&least_used_key);
            debug!("Evicted least used cache entry: key={}", least_used_key);
        }
    }

    /// 获取缓存命中率
    pub fn get_hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> ValidationCacheStats {
        ValidationCacheStats {
            capacity: self.config.capacity,
            current_size: self.cache.len(),
            hits: self.hits,
            misses: self.misses,
            hit_rate: self.get_hit_rate(),
        }
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
        info!("Validation cache cleared");
    }
}

/// 校验缓存统计信息
#[derive(Debug, Clone)]
pub struct ValidationCacheStats {
    /// 缓存容量
    pub capacity: usize,
    /// 当前大小
    pub current_size: usize,
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 命中率
    pub hit_rate: f64,
}

impl Default for ValidationCache {
    fn default() -> Self {
        Self::new(ValidationCacheConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_cache_creation() {
        let cache = ValidationCache::new(ValidationCacheConfig::default());
        assert_eq!(cache.hits, 0);
        assert_eq!(cache.misses, 0);
        assert_eq!(cache.get_hit_rate(), 0.0);
    }

    #[test]
    fn test_validation_cache_set_get() {
        let mut cache = ValidationCache::new(ValidationCacheConfig {
            capacity: 100,
            ttl_secs: 300,
            cleanup_interval_secs: 60,
        });
        
        let key = ValidationCacheKey {
            request_hash: "hash_1".to_string(),
            verifier_version: "v1".to_string(),
        };
        
        // 首次获取应该未命中
        assert_eq!(cache.get(&key), None);
        
        // 设置结果
        cache.set(&key, true);
        
        // 再次获取应该命中
        assert_eq!(cache.get(&key), Some(true));
        assert_eq!(cache.get_hit_rate(), 50.0); // 1 hit, 1 miss
    }

    #[test]
    fn test_validation_cache_stats() {
        let mut cache = ValidationCache::new(ValidationCacheConfig::default());
        
        let key = ValidationCacheKey {
            request_hash: "hash_1".to_string(),
            verifier_version: "v1".to_string(),
        };
        
        cache.set(&key, true);
        cache.get(&key);
        cache.get(&key);
        
        let stats = cache.get_stats();
        assert!(stats.current_size >= 1);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!(stats.hit_rate > 60.0);
    }
}
