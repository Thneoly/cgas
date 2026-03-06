//! 性能优化模块
//! 
//! 实现 Phase 2 性能优化的关键组件：
//! - 异步并发池：提升执行器和验证器并发性能
//! - 增量重放：仅重放变化的 state_diff，提升验证性能
//! - 校验缓存：缓存热点数据，减少重复计算
//! - 对象池：复用对象，减少内存分配

pub mod async_pool;
pub mod incremental_replay;
pub mod validation_cache;
pub mod object_pool;

pub use async_pool::{AsyncPool, AsyncPoolConfig, ExecutorAsyncPool, VerifierAsyncPool};
pub use incremental_replay::IncrementalReplayer;
pub use validation_cache::{ValidationCache, ValidationCacheConfig, ValidationCacheKey};
pub use object_pool::{ObjectPool, ObjectPoolConfig, StateDiffObjectPool, ExecutionResultObjectPool};
