//! 性能优化实施主模块
//! 
//! Phase 3 性能优化综合实现，包括：
//! - 异步执行优化（tokio 任务调度）
//! - 多级缓存策略
//! - 对象池复用
//! - 批量操作优化
//! - 内存管理优化

use std::sync::Arc;
use std::time::Duration;
use log::{info, warn, error};

use crate::optimization::async_pool::{AsyncExecutionPool, PoolConfig};
use crate::optimization::object_pool::ObjectPool;
use crate::optimization::validation_cache::{ValidationCache, CacheConfig};

/// 性能优化器配置
#[derive(Debug, Clone)]
pub struct PerformanceOptimizerConfig {
    /// 异步执行池大小
    pub pool_size: usize,
    /// 对象池容量
    pub object_pool_capacity: usize,
    /// 缓存大小
    pub cache_size: usize,
    /// 缓存 TTL（秒）
    pub cache_ttl_secs: u64,
    /// 批量操作大小
    pub batch_size: usize,
    /// 是否启用预取
    pub enable_prefetch: bool,
    /// 是否启用压缩
    pub enable_compression: bool,
}

impl Default for PerformanceOptimizerConfig {
    fn default() -> Self {
        Self {
            pool_size: num_cpus::get() * 2,
            object_pool_capacity: 1000,
            cache_size: 10000,
            cache_ttl_secs: 300, // 5 分钟
            batch_size: 100,
            enable_prefetch: true,
            enable_compression: false,
        }
    }
}

/// 性能优化器
pub struct PerformanceOptimizer {
    /// 配置
    config: PerformanceOptimizerConfig,
    /// 异步执行池
    execution_pool: Arc<AsyncExecutionPool>,
    /// 对象池
    object_pool: Arc<ObjectPool<ExecutionBuffer>>,
    /// 验证缓存
    cache: Arc<ValidationCache>,
}

impl PerformanceOptimizer {
    /// 创建新的性能优化器
    pub fn new(config: PerformanceOptimizerConfig) -> Self {
        let pool_config = PoolConfig {
            max_workers: config.pool_size,
            queue_size: config.batch_size * 10,
            task_timeout: Duration::from_secs(60),
        };
        
        let execution_pool = Arc::new(AsyncExecutionPool::new(pool_config));
        
        let object_pool = Arc::new(ObjectPool::new(config.object_pool_capacity));
        
        let cache_config = CacheConfig {
            max_size: config.cache_size,
            ttl: Duration::from_secs(config.cache_ttl_secs),
            eviction_policy: crate::optimization::validation_cache::EvictionPolicy::LRU,
        };
        
        let cache = Arc::new(ValidationCache::new(cache_config));
        
        info!(
            "Performance optimizer created: pool_size={}, cache_size={}, batch_size={}",
            config.pool_size,
            config.cache_size,
            config.batch_size
        );
        
        Self {
            config,
            execution_pool,
            object_pool,
            cache,
        }
    }
    
    /// 获取异步执行池
    pub fn execution_pool(&self) -> Arc<AsyncExecutionPool> {
        self.execution_pool.clone()
    }
    
    /// 获取对象池
    pub fn object_pool(&self) -> Arc<ObjectPool<ExecutionBuffer>> {
        self.object_pool.clone()
    }
    
    /// 获取缓存
    pub fn cache(&self) -> Arc<ValidationCache> {
        self.cache.clone()
    }
    
    /// 执行批量优化任务
    pub async fn execute_batch<T, F, R>(&self, items: Vec<T>, processor: F) -> Vec<R>
    where
        T: Send + 'static,
        F: Fn(T) -> R + Send + Sync + 'static,
        R: Send + 'static,
    {
        let processor = Arc::new(processor);
        let mut handles = Vec::with_capacity(items.len());
        
        for item in items {
            let processor = processor.clone();
            let handle = self.execution_pool.spawn(async move {
                processor(item)
            });
            handles.push(handle);
        }
        
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Batch task failed: {}", e);
                }
            }
        }
        
        results
    }
    
    /// 从缓存获取或计算
    pub async fn get_or_compute<F, R>(&self, key: String, compute_fn: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Clone + Send + 'static,
    {
        // 尝试从缓存获取
        if let Some(cached) = self.cache.get(&key) {
            info!("Cache hit for key: {}", key);
            return cached;
        }
        
        // 计算并缓存
        info!("Cache miss for key: {}, computing...", key);
        let result = compute_fn();
        
        self.cache.insert(key, result.clone());
        result
    }
    
    /// 预取数据到缓存
    pub async fn prefetch<F, R>(&self, keys: Vec<String>, fetch_fn: F)
    where
        F: Fn(String) -> R + Send + Sync + 'static,
        R: Clone + Send + 'static,
    {
        if !self.config.enable_prefetch {
            return;
        }
        
        let fetch_fn = Arc::new(fetch_fn);
        let mut handles = Vec::with_capacity(keys.len());
        
        for key in keys {
            let fetch_fn = fetch_fn.clone();
            let cache = self.cache.clone();
            
            let handle = self.execution_pool.spawn(async move {
                if cache.contains(&key) {
                    return;
                }
                
                let value = fetch_fn(key.clone());
                cache.insert(key, value);
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            let _ = handle.await;
        }
    }
    
    /// 获取优化器统计信息
    pub fn get_stats(&self) -> OptimizerStats {
        OptimizerStats {
            pool_stats: self.execution_pool.get_stats(),
            object_pool_stats: self.object_pool.get_stats(),
            cache_stats: self.cache.get_stats(),
            config: self.config.clone(),
        }
    }
    
    /// 清空缓存
    pub fn clear_cache(&self) {
        self.cache.clear();
        info!("Cache cleared");
    }
    
    /// 预热缓存（加载常用数据）
    pub fn warmup_cache(&self, hot_keys: Vec<String>) {
        if !self.config.enable_prefetch {
            return;
        }
        
        info!("Warming up cache with {} hot keys", hot_keys.len());
        // 实际实现中会异步加载这些键
    }
}

/// 优化器统计信息
#[derive(Debug, Clone)]
pub struct OptimizerStats {
    /// 执行池统计
    pub pool_stats: crate::optimization::async_pool::PoolStats,
    /// 对象池统计
    pub object_pool_stats: crate::optimization::object_pool::PoolStats,
    /// 缓存统计
    pub cache_stats: crate::optimization::validation_cache::CacheStats,
    /// 配置
    pub config: PerformanceOptimizerConfig,
}

impl std::fmt::Display for OptimizerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Performance Optimizer Stats:")?;
        writeln!(f, "  Pool: active={}, queued={}", 
            self.pool_stats.active_tasks, 
            self.pool_stats.queued_tasks)?;
        writeln!(f, "  Object Pool: used={}, available={}", 
            self.object_pool_stats.used, 
            self.object_pool_stats.available)?;
        writeln!(f, "  Cache: size={}, hits={}, misses={}, hit_rate={:.2}%", 
            self.cache_stats.size,
            self.cache_stats.hits,
            self.cache_stats.misses,
            if self.cache_stats.hits + self.cache_stats.misses > 0 {
                (self.cache_stats.hits as f64 / (self.cache_stats.hits + self.cache_stats.misses) as f64) * 100.0
            } else {
                0.0
            })?;
        Ok(())
    }
}

/// 执行缓冲区（对象池复用）
#[derive(Debug, Clone)]
pub struct ExecutionBuffer {
    /// 缓冲区数据
    pub data: Vec<u8>,
    /// 使用计数
    pub use_count: u32,
}

impl ExecutionBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            use_count: 0,
        }
    }
}

impl crate::optimization::object_pool::Poolable for ExecutionBuffer {
    fn reset(&mut self) {
        self.data.clear();
        self.use_count = 0;
    }
    
    fn is_expired(&self) -> bool {
        self.use_count > 100 // 使用 100 次后过期
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_optimizer_creation() {
        let config = PerformanceOptimizerConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.config.pool_size, num_cpus::get() * 2);
        assert_eq!(stats.config.cache_size, 10000);
    }
    
    #[tokio::test]
    async fn test_batch_execution() {
        let config = PerformanceOptimizerConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        let items = vec![1, 2, 3, 4, 5];
        let results = optimizer.execute_batch(items, |x| x * 2).await;
        
        assert_eq!(results.len(), 5);
        assert!(results.contains(&2));
        assert!(results.contains(&10));
    }
    
    #[tokio::test]
    async fn test_cache_get_or_compute() {
        let config = PerformanceOptimizerConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        let result = optimizer.get_or_compute(
            "test_key".to_string(),
            || 42,
        ).await;
        
        assert_eq!(result, 42);
        
        // 第二次应该从缓存获取
        let result2 = optimizer.get_or_compute(
            "test_key".to_string(),
            || 99, // 这个不会被调用
        ).await;
        
        assert_eq!(result2, 42);
    }
    
    #[test]
    fn test_execution_buffer_poolable() {
        let mut buffer = ExecutionBuffer::new(1024);
        buffer.data.extend_from_slice(&[1, 2, 3]);
        buffer.use_count = 5;
        
        buffer.reset();
        
        assert!(buffer.data.is_empty());
        assert_eq!(buffer.use_count, 0);
    }
}
