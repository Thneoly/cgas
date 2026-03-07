//! 性能优化模块
//! 
//! Phase 3 性能优化实施：
//! - 异步执行池
//! - 对象池复用
//! - 验证缓存
//! - 增量回放优化
//! - 工作窃取执行器 (Week 3)
//! - 并行验证器 (Week 3)
//! - 无锁缓存 (Week 3)
//! - 验证器流水线优化 (Week 4)
//! - 吞吐量优化 (Week 4)
//! - 边界场景修复 (Week 4)
//! - P99 巩固优化 (Week 5)

pub mod async_pool;
pub mod object_pool;
pub mod validation_cache;
pub mod incremental_replay;
pub mod work_stealing_executor;
pub mod parallel_verifier;
pub mod lockfree_cache;
pub mod verifier_pipeline_optimization;
pub mod throughput_optimization;
pub mod boundary_fixes_batch2;
pub mod p99_consolidation_optimization;

pub use p99_consolidation_optimization::{
    P99ConsolidationOptimizer, P99ConsolidationConfig, HotspotAnalyzer, HotspotPathInfo,
    MemoryAccessOptimizer, LockContentionOptimizer, LockContentionStats,
    BranchPredictionOptimizer, BranchStats, OptimizationEffect, MemoryStats,
    PerformanceGuard, profile_path,
};
pub use async_pool::{AsyncExecutionPool, PoolConfig, ExecutionTask};
pub use object_pool::{ObjectPool, Poolable, PoolStats};
pub use validation_cache::{ValidationCache, CacheEntry, CacheConfig};
pub use incremental_replay::{IncrementalReplayer, ReplayStrategy, ReplayResult};
pub use work_stealing_executor::{
    WorkStealingExecutor, WorkStealingExecutorConfig, Task, WorkerStats, ExecutorStats,
};
pub use parallel_verifier::{
    ParallelVerifier, ParallelVerifierConfig, BatchVerifyRequest, BatchVerifyResponse,
    InstructionToVerify, VerifyResult, VerifierStats, SimdAcceleration,
};
pub use lockfree_cache::{
    LockFreeCache, LockFreeCacheConfig, CacheEntry, CacheStats,
};
pub use verifier_pipeline_optimization::{
    VerifierPipeline, VerifierPipelineConfig, PipelineStats, PipelineInstruction, PipelineStage,
};
pub use throughput_optimization::{
    ThroughputOptimizer, ThroughputOptimizerConfig, ThroughputStats,
    OptimizedConnectionPool, ConnectionPoolConfig, ConnectionPoolStats,
    AdaptiveBatchProcessor, BatchProcessingConfig, BatchProcessorStats,
};
pub use boundary_fixes_batch2::*;
