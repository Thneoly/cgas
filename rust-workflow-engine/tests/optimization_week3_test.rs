//! 性能优化模块测试
//! 
//! Phase 3 Week 3 性能优化专项测试

#[cfg(test)]
mod optimization_tests {
    use crate::optimization::{
        WorkStealingExecutor, WorkStealingExecutorConfig,
        ParallelVerifier, ParallelVerifierConfig, BatchVerifyRequest, InstructionToVerify,
        LockFreeCache, LockFreeCacheConfig,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    /// 测试任务
    struct TestTask {
        id: usize,
        counter: Arc<AtomicUsize>,
    }

    impl crate::optimization::Task for TestTask {
        fn execute(self: Box<Self>) {
            self.counter.fetch_add(1, Ordering::Relaxed);
        }

        fn priority(&self) -> u8 {
            5
        }
    }

    #[test]
    fn test_work_stealing_executor_basic() {
        println!("Testing Work-Stealing Executor...");

        let config = WorkStealingExecutorConfig {
            num_workers: 4,
            ..Default::default()
        };

        let executor = WorkStealingExecutor::new(config);
        let counter = Arc::new(AtomicUsize::new(0));

        // 提交 100 个任务
        for i in 0..100 {
            let task = Box::new(TestTask {
                id: i,
                counter: Arc::clone(&counter),
            });
            executor.submit(task).unwrap();
        }

        // 等待任务执行
        thread::sleep(Duration::from_millis(500));

        // 验证所有任务都执行了
        assert_eq!(counter.load(Ordering::Relaxed), 100);

        // 验证统计
        let stats = executor.stats();
        assert_eq!(stats.tasks_submitted.load(Ordering::Relaxed), 100);

        println!("✅ Work-Stealing Executor test passed");
    }

    #[tokio::test]
    async fn test_parallel_verifier_basic() {
        println!("Testing Parallel Verifier...");

        let config = ParallelVerifierConfig::default();
        let verifier = ParallelVerifier::new(config);

        // 创建测试指令
        let instructions = (0..50)
            .map(|i| InstructionToVerify {
                id: format!("instr_{}", i),
                instruction_bytes: format!("data_{}", i).into_bytes(),
                expected_hash: format!("hash_{}", i),
            })
            .collect();

        let request = BatchVerifyRequest {
            trace_id: "trace_1".to_string(),
            batch_id: "batch_1".to_string(),
            instructions,
            parallel: true,
            chunk_size: 10,
        };

        let response = verifier.parallel_verify(request).await.unwrap();

        assert_eq!(response.results.len(), 50);
        assert!(response.chunk_count > 0);
        assert!(response.total_duration_ms > 0);

        println!("✅ Parallel Verifier test passed");
    }

    #[test]
    fn test_lock_free_cache_basic() {
        println!("Testing Lock-Free Cache...");

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
        assert!(stats.hit_rate() > 0.6);

        println!("✅ Lock-Free Cache test passed");
    }

    #[test]
    fn test_lock_free_cache_concurrent() {
        println!("Testing Lock-Free Cache (Concurrent)...");

        let config = LockFreeCacheConfig {
            max_capacity: 10000,
            ..Default::default()
        };
        let cache = Arc::new(LockFreeCache::new(config));

        let mut handles = vec![];

        // 并发写入
        for i in 0..100 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
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
            let handle = thread::spawn(move || {
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

        println!("✅ Lock-Free Cache (Concurrent) test passed");
    }

    #[test]
    fn test_all_optimizations_integration() {
        println!("Testing All Optimizations Integration...");

        // 1. 创建工作窃取执行器
        let executor_config = WorkStealingExecutorConfig {
            num_workers: 4,
            ..Default::default()
        };
        let executor = WorkStealingExecutor::new(executor_config);

        // 2. 创建并行验证器
        let verifier_config = ParallelVerifierConfig::default();
        let verifier = ParallelVerifier::new(verifier_config);

        // 3. 创建无锁缓存
        let cache_config = LockFreeCacheConfig::default();
        let cache = LockFreeCache::new(cache_config);

        // 验证所有组件都创建成功
        assert_eq!(executor.num_workers(), 4);
        assert!(verifier.stats().instructions_verified.load(Ordering::Relaxed) == 0);
        assert!(cache.is_empty());

        println!("✅ All Optimizations Integration test passed");
    }
}

fn main() {
    println!("Phase 3 Week 3 Performance Optimization Tests");
    println!("=============================================\n");

    // 运行测试 (需要在 test 模式下)
    // cargo test --release optimization_tests
}
