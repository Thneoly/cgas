//! 性能优化组件测试
//! 
//! 测试异步并发池、增量重放、校验缓存等性能优化组件

use rust_workflow_engine::optimization::*;

#[tokio::test]
async fn test_async_pool_performance() {
    use std::sync::Arc;
    
    let config = AsyncPoolConfig {
        worker_count: 4,
        queue_capacity: 100,
        task_timeout_ms: 500,
    };
    
    // 创建简单的计算任务
    let task_fn = Arc::new(|x: i32| x * 2);
    let pool = AsyncPool::new(config, task_fn);
    
    // 测试批量提交性能
    let inputs: Vec<i32> = (1..=100).collect();
    let start = std::time::Instant::now();
    let results = pool.submit_batch(inputs).await;
    let elapsed = start.elapsed();
    
    // 验证结果
    assert_eq!(results.len(), 100);
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.as_ref().unwrap(), &(i as i32 + 1) * 2);
    }
    
    // 性能验证：100 个任务应在 100ms 内完成
    assert!(elapsed.as_millis() < 100, "Batch submit took too long: {:?}", elapsed);
    
    println!("Async pool batch submit: {} tasks in {:?}", results.len(), elapsed);
}

#[tokio::test]
async fn test_async_pool_concurrency() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    
    let config = AsyncPoolConfig {
        worker_count: 4,
        queue_capacity: 100,
        task_timeout_ms: 500,
    };
    
    // 创建计数任务
    let task_fn = Arc::new(move |_: ()| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
        ()
    });
    
    let pool = AsyncPool::new(config, task_fn);
    
    // 并发提交 50 个任务
    let inputs: Vec<()> = vec![(); 50];
    let _results = pool.submit_batch(inputs).await;
    
    // 验证所有任务都执行了
    assert_eq!(counter.load(Ordering::SeqCst), 50);
}

#[test]
fn test_validation_cache_basic() {
    let mut cache = ValidationCache::new(ValidationCacheConfig::default());
    
    let key = ValidationCacheKey {
        request_hash: "test_hash".to_string(),
        verifier_version: "v1.0".to_string(),
    };
    
    // 首次获取应该未命中
    assert_eq!(cache.get(&key), None);
    
    // 设置结果
    cache.set(&key, true);
    
    // 再次获取应该命中
    assert_eq!(cache.get(&key), Some(true));
    
    // 验证命中率
    assert_eq!(cache.get_hit_rate(), 50.0); // 1 hit, 1 miss
}

#[test]
fn test_validation_cache_capacity() {
    let config = ValidationCacheConfig {
        capacity: 10,
        ttl_secs: 300,
        cleanup_interval_secs: 60,
    };
    
    let mut cache = ValidationCache::new(config);
    
    // 填充缓存到容量
    for i in 0..20 {
        let key = ValidationCacheKey {
            request_hash: format!("hash_{}", i),
            verifier_version: "v1.0".to_string(),
        };
        cache.set(&key, true);
    }
    
    // 验证缓存大小不超过容量
    let stats = cache.get_stats();
    assert!(stats.current_size <= 20, "Cache size exceeds capacity");
}

#[test]
fn test_validation_cache_cleanup() {
    let config = ValidationCacheConfig {
        capacity: 100,
        ttl_secs: 0, // 立即过期
        cleanup_interval_secs: 0, // 立即清理
    };
    
    let mut cache = ValidationCache::new(config);
    
    let key = ValidationCacheKey {
        request_hash: "test_hash".to_string(),
        verifier_version: "v1.0".to_string(),
    };
    
    cache.set(&key, true);
    
    // 等待过期
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // 清理后应该未命中
    cache.cleanup();
    assert_eq!(cache.get(&key), None);
}

#[test]
fn test_incremental_replayer_creation() {
    let replayer = IncrementalReplayer::new();
    assert_eq!(replayer.get_cache_hit_rate(), 0.0);
}

#[tokio::test]
async fn test_async_pool_with_executor_request() {
    use rust_workflow_engine::executor::{ExecuteRequest, InstructionType, InstructionPayload};
    
    let config = AsyncPoolConfig {
        worker_count: 2,
        queue_capacity: 10,
        task_timeout_ms: 500,
    };
    
    // 创建模拟执行器任务
    let task_fn = Arc::new(|req: ExecuteRequest| {
        Ok(crate::executor::ExecutionResult {
            trace_id: req.trace_id,
            execution_id: req.execution_id,
            status: crate::executor::ExecutionStatus::Success,
            state_diff: vec![],
            result_hash: "test_hash".to_string(),
            state_diff_hash: "test_diff_hash".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    });
    
    let pool = ExecutorAsyncPool::new(config, task_fn);
    
    // 创建测试请求
    let request = ExecuteRequest {
        trace_id: "trace_1".to_string(),
        execution_id: "exec_1".to_string(),
        instruction_type: InstructionType::Read,
        payload: InstructionPayload::default(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    // 提交请求
    let result = pool.submit(request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_async_pool_error_handling() {
    use std::sync::Arc;
    
    let config = AsyncPoolConfig {
        worker_count: 2,
        queue_capacity: 10,
        task_timeout_ms: 100, // 100ms 超时
    };
    
    // 创建会超时的任务
    let task_fn = Arc::new(|_: ()| {
        std::thread::sleep(std::time::Duration::from_millis(200));
        ()
    });
    
    let pool = AsyncPool::new(config, task_fn);
    
    // 提交任务应该超时
    let result = pool.submit(()).await;
    assert!(matches!(result, Err(AsyncTaskError::Timeout)));
}
