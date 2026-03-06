//! 对象池复用
//! 
//! 实现对象池机制，复用频繁创建/销毁的对象，减少内存分配开销
//! Phase 2 Week 4 性能优化关键组件

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use log::{info, debug};

/// 对象池配置
#[derive(Debug, Clone)]
pub struct ObjectPoolConfig {
    /// 初始容量
    pub initial_capacity: usize,
    /// 最大容量
    pub max_capacity: usize,
    /// 对象存活时间 (秒)
    pub ttl_secs: u64,
}

impl Default for ObjectPoolConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 100,
            max_capacity: 1000,
            ttl_secs: 300, // 5 分钟
        }
    }
}

/// 对象池
pub struct ObjectPool<T> {
    /// 池化对象队列
    pool: Arc<Mutex<VecDeque<PoolObject<T>>>>,
    /// 配置
    config: ObjectPoolConfig,
    /// 创建计数
    create_count: u64,
    /// 复用计数
    reuse_count: u64,
}

/// 池化对象
struct PoolObject<T> {
    /// 对象数据
    data: T,
    /// 创建时间
    created_at: std::time::Instant,
    /// 最后使用时间
    last_used_at: std::time::Instant,
}

impl<T> ObjectPool<T>
where
    T: Default + Clone + 'static,
{
    /// 创建新的对象池
    pub fn new(config: ObjectPoolConfig) -> Self {
        let mut pool = VecDeque::with_capacity(config.initial_capacity);
        
        // 预创建初始对象
        for _ in 0..config.initial_capacity {
            pool.push_back(PoolObject {
                data: T::default(),
                created_at: std::time::Instant::now(),
                last_used_at: std::time::Instant::now(),
            });
        }
        
        info!(
            "ObjectPool created: initial_capacity={}, max_capacity={}",
            config.initial_capacity, config.max_capacity
        );
        
        Self {
            pool: Arc::new(Mutex::new(pool)),
            config,
            create_count: config.initial_capacity as u64,
            reuse_count: 0,
        }
    }

    /// 获取对象
    pub fn get(&self) -> T {
        let mut pool_guard = self.pool.lock().unwrap();
        
        // 尝试从池中获取未过期的对象
        while let Some(pool_obj) = pool_guard.pop_front() {
            // 检查是否过期
            if pool_obj.last_used_at.elapsed().as_secs() < self.config.ttl_secs {
                self.reuse_count += 1;
                debug!("ObjectPool: reused object, reuse_count={}", self.reuse_count);
                return pool_obj.data;
            }
            // 过期对象丢弃，继续获取下一个
        }
        
        // 池为空或所有对象都过期，创建新对象
        self.create_count += 1;
        debug!("ObjectPool: created new object, create_count={}", self.create_count);
        T::default()
    }

    /// 归还对象到池中
    pub fn return_object(&self, obj: T) {
        let mut pool_guard = self.pool.lock().unwrap();
        
        // 检查池是否已满
        if pool_guard.len() >= self.config.max_capacity {
            // 池已满，丢弃对象
            debug!("ObjectPool: pool full, discarding object");
            return;
        }
        
        pool_guard.push_back(PoolObject {
            data: obj,
            created_at: std::time::Instant::now(),
            last_used_at: std::time::Instant::now(),
        });
        
        debug!("ObjectPool: returned object, pool_size={}", pool_guard.len());
    }

    /// 获取池统计信息
    pub fn get_stats(&self) -> ObjectPoolStats {
        let pool_guard = self.pool.lock().unwrap();
        let pool_size = pool_guard.len();
        
        let total_requests = self.create_count + self.reuse_count;
        let reuse_rate = if total_requests > 0 {
            (self.reuse_count as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        ObjectPoolStats {
            pool_size,
            max_capacity: self.config.max_capacity,
            create_count: self.create_count,
            reuse_count: self.reuse_count,
            reuse_rate,
        }
    }

    /// 清理过期对象
    pub fn cleanup(&self) -> usize {
        let mut pool_guard = self.pool.lock().unwrap();
        let now = std::time::Instant::now();
        let ttl = std::time::Duration::from_secs(self.config.ttl_secs);
        
        let initial_size = pool_guard.len();
        
        // 保留未过期的对象
        pool_guard.retain(|pool_obj| {
            now.duration_since(pool_obj.last_used_at) < ttl
        });
        
        let removed_count = initial_size - pool_guard.len();
        
        if removed_count > 0 {
            info!("ObjectPool cleanup: removed {} expired objects", removed_count);
        }
        
        removed_count
    }

    /// 清空池
    pub fn clear(&self) {
        let mut pool_guard = self.pool.lock().unwrap();
        pool_guard.clear();
        info!("ObjectPool cleared");
    }
}

/// 对象池统计信息
#[derive(Debug, Clone)]
pub struct ObjectPoolStats {
    /// 当前池大小
    pub pool_size: usize,
    /// 最大容量
    pub max_capacity: usize,
    /// 创建次数
    pub create_count: u64,
    /// 复用次数
    pub reuse_count: u64,
    /// 复用率
    pub reuse_rate: f64,
}

/// StateDiff 操作对象池
pub type StateDiffObjectPool = ObjectPool<Vec<crate::executor::StateDiffOperation>>;

/// ExecutionResult 对象池
pub type ExecutionResultObjectPool = ObjectPool<crate::executor::ExecutionResult>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool_creation() {
        let config = ObjectPoolConfig {
            initial_capacity: 10,
            max_capacity: 100,
            ttl_secs: 300,
        };
        
        let pool = ObjectPool::<i32>::new(config);
        let stats = pool.get_stats();
        
        assert_eq!(stats.pool_size, 10);
        assert_eq!(stats.max_capacity, 100);
        assert_eq!(stats.create_count, 10);
        assert_eq!(stats.reuse_count, 0);
    }

    #[test]
    fn test_object_pool_get_return() {
        let config = ObjectPoolConfig::default();
        let pool = ObjectPool::<i32>::new(config);
        
        // 获取对象
        let obj = pool.get();
        assert_eq!(obj, 0); // i32 默认值为 0
        
        // 归还对象
        pool.return_object(42);
        
        // 再次获取应该复用
        let obj = pool.get();
        assert_eq!(obj, 42);
        
        let stats = pool.get_stats();
        assert_eq!(stats.reuse_count, 1);
        assert!(stats.reuse_rate > 0.0);
    }

    #[test]
    fn test_object_pool_cleanup() {
        let config = ObjectPoolConfig {
            initial_capacity: 10,
            max_capacity: 100,
            ttl_secs: 0, // 立即过期
        };
        
        let pool = ObjectPool::<i32>::new(config);
        
        // 等待过期
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // 清理过期对象
        let removed = pool.cleanup();
        assert_eq!(removed, 10);
        
        let stats = pool.get_stats();
        assert_eq!(stats.pool_size, 0);
    }

    #[test]
    fn test_object_pool_max_capacity() {
        let config = ObjectPoolConfig {
            initial_capacity: 5,
            max_capacity: 5,
            ttl_secs: 300,
        };
        
        let pool = ObjectPool::<i32>::new(config);
        
        // 获取所有对象
        for _ in 0..5 {
            let _ = pool.get();
        }
        
        // 归还超过容量的对象
        for i in 0..10 {
            pool.return_object(i);
        }
        
        let stats = pool.get_stats();
        assert_eq!(stats.pool_size, 5); // 不超过最大容量
    }

    #[test]
    fn test_object_pool_reuse_rate() {
        let config = ObjectPoolConfig {
            initial_capacity: 10,
            max_capacity: 100,
            ttl_secs: 300,
        };
        
        let pool = ObjectPool::<i32>::new(config);
        
        // 获取并归还多次
        for i in 0..100 {
            let obj = pool.get();
            pool.return_object(obj + 1);
        }
        
        let stats = pool.get_stats();
        assert!(stats.reuse_rate > 90.0); // 复用率应该很高
    }
}
