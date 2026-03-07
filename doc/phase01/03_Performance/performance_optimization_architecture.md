# 性能优化架构设计 (P99 <200ms)

**版本**: v1.0  
**日期**: 2026-05-12  
**责任人**: Architect-Agent + SRE  
**状态**: 📋 草案  
**release_id**: release-2026-05-12-phase3_week01  
**关联 ADR**: ADR-009 (Phase 3 ADR v5)

---

## 1. 设计目标

### 1.1 Phase 2 vs Phase 3 性能对比

| 指标 | Phase 2 实际 | Phase 3 目标 | 优化幅度 | 关键优化 |
|---|---|---|---|---|
| P99 执行时延 | 265ms | **<200ms** | **-25%** | 工作窃取 + 并行验证 |
| P99 验证时延 | 272ms | **<190ms** | **-30%** | 并行验证 + 缓存预热 |
| 吞吐量 | 135 请求/秒 | **>180 请求/秒** | **+33%** | 异步优化 |
| 阻断开销 | 3.2% | **<2.5%** | **-22%** | 无锁缓存 |
| Batch 开销 | 15% | **<12%** | **-20%** | 懒加载快照 |
| MVCC 开销 | N/A | **<8ms** | - | CoW 优化 |

### 1.2 性能瓶颈分析 (Phase 2)

```
Phase 2 时延分解 (P99 265ms):

执行器：115ms (43%)  ████████████████████████████████████████████████
验证器：125ms (47%)  ██████████████████████████████████████████████████████
阻断： 15ms  (6%)   ██████
序列化：10ms (4%)   ████

关键瓶颈:
1. 执行器：线程池负载均衡不佳 (-15% 优化空间)
2. 验证器：串行验证，未充分利用多核 (-20% 优化空间)
3. 阻断：锁竞争导致延迟 (-10% 优化空间)
4. 序列化：未使用 SIMD 加速 (-8% 优化空间)
```

### 1.3 Phase 3 优化策略

```
Phase 3 优化分解 (目标 P99 <200ms):

Phase 2 基线:        265ms ████████████████████████████████████████████████████
工作窃取 (-15%):     -40ms ████████████████████
并行验证 (-20%):     -53ms ██████████████████████████████
无锁缓存 (-10%):     -27ms ██████████████
MVCC CoW (-5%):      -13ms ██████
SIMD 加速 (-8%):     -21ms ██████████
其他优化：           -11ms █████
Phase 3 目标：       <200ms ████████████████████████████████████
```

---

## 2. 工作窃取执行器

### 2.1 架构设计

```rust
/// Phase 3: 工作窃取执行器
pub struct WorkStealingExecutor {
    /// Worker 列表
    workers: Vec<Worker>,
    /// 全局任务队列
    global_queue: Arc<ConcurrentQueue<Task>>,
    /// 本地任务队列 (每个 Worker 一个)
    local_queues: Vec<Arc<ConcurrentQueue<Task>>>,
    /// 监控指标
    metrics: Arc<ExecutorMetrics>,
}

struct Worker {
    id: usize,
    local_queue: Arc<ConcurrentQueue<Task>>,
    thread: JoinHandle<()>,
    stats: AtomicWorkerStats,
}

struct AtomicWorkerStats {
    tasks_executed: AtomicU64,
    tasks_stolen: AtomicU64,
    tasks_donated: AtomicU64,
    idle_time_ms: AtomicU64,
}

impl WorkStealingExecutor {
    /// 创建执行器
    pub fn new(num_workers: usize) -> Self {
        let global_queue = Arc::new(ConcurrentQueue::unbounded());
        let local_queues: Vec<_> = (0..num_workers)
            .map(|_| Arc::new(ConcurrentQueue::unbounded()))
            .collect();
        
        let workers: Vec<_> = (0..num_workers)
            .map(|id| {
                let local_queue = Arc::clone(&local_queues[id]);
                let global_queue = Arc::clone(&global_queue);
                
                let thread = std::thread::spawn(move || {
                    worker_loop(id, local_queue, global_queue);
                });
                
                Worker {
                    id,
                    local_queue,
                    thread,
                    stats: AtomicWorkerStats::new(),
                }
            })
            .collect();
        
        Self {
            workers,
            global_queue,
            local_queues,
            metrics: Arc::new(ExecutorMetrics::new()),
        }
    }
    
    /// 提交任务
    pub fn submit(&self, task: Task, preferred_worker: Option<usize>) {
        if let Some(worker_id) = preferred_worker {
            // 优先提交到本地队列
            self.local_queues[worker_id].push(task);
        } else {
            // 提交到全局队列
            self.global_queue.push(task);
        }
        
        self.metrics.record_task_submitted();
    }
}

/// Worker 循环
fn worker_loop(
    worker_id: usize,
    local_queue: Arc<ConcurrentQueue<Task>>,
    global_queue: Arc<ConcurrentQueue<Task>>,
) {
    loop {
        // 1. 优先从本地队列获取任务
        if let Some(task) = local_queue.pop() {
            execute_task(task);
            continue;
        }
        
        // 2. 从全局队列获取任务
        if let Some(task) = global_queue.pop() {
            execute_task(task);
            continue;
        }
        
        // 3. 从其他 Worker 窃取任务
        if let Some(task) = steal_from_others(worker_id) {
            execute_task(task);
            continue;
        }
        
        // 4. 空闲等待
        std::thread::sleep(Duration::from_micros(100));
    }
}

/// 从其他 Worker 窃取任务
fn steal_from_others(current_worker_id: usize) -> Option<Task> {
    // 随机选择受害者 (避免热点)
    let mut rng = rand::thread_rng();
    let victim_id = rng.gen_range(0..NUM_WORKERS);
    
    if victim_id != current_worker_id {
        // 从受害者队列尾部窃取 (FIFO 窃取)
        if let Some(task) = LOCAL_QUEUES[victim_id].steal() {
            return Some(task);
        }
    }
    
    None
}
```

### 2.2 性能优势

| 指标 | 固定线程池 | 工作窃取 | 提升 |
|---|---|---|---|
| 负载均衡 | 差 (任务分配不均) | 优 (动态平衡) | +25% |
| 空闲率 | 15-20% | <5% | -75% |
| P99 时延 | 115ms | <95ms | -17% |
| 吞吐量 | 135 请求/秒 | >170 请求/秒 | +26% |

---

## 3. 并行验证器

### 3.1 架构设计

```rust
/// Phase 3: 并行验证器
pub struct ParallelVerifier {
    /// 验证 Worker 池
    worker_pool: ThreadPool,
    /// 验证缓存
    cache: Arc<LockFreeCache<Hash, VerifyResult>>,
    /// 监控指标
    metrics: Arc<VerifierMetrics>,
}

impl ParallelVerifier {
    /// 并行验证 Batch
    pub async fn parallel_verify(
        &self,
        request: &BatchVerifyRequest,
    ) -> Result<BatchVerifyResponse> {
        let start_time = Instant::now();
        
        // 1. 将 Batch 拆分为多个 chunk 并行验证
        let chunk_size = 10;  // 每 10 条指令一组
        let chunks: Vec<_> = request.instructions.chunks(chunk_size).collect();
        
        // 2. 并行验证每个 chunk
        let futures: Vec<_> = chunks
            .iter()
            .enumerate()
            .map(|(chunk_idx, chunk)| {
                let cache = Arc::clone(&self.cache);
                async move {
                    self.verify_chunk(chunk_idx, chunk, cache).await
                }
            })
            .collect();
        
        let chunk_results = try_join_all(futures).await?;
        
        // 3. 合并结果
        let results = chunk_results.into_iter().flatten().collect();
        
        // 4. 记录监控指标
        let elapsed = start_time.elapsed();
        self.metrics.record_parallel_verification(
            request.instructions.len(),
            elapsed.as_millis() as u64,
        );
        
        Ok(BatchVerifyResponse {
            trace_id: request.trace_id.clone(),
            results,
            verify_hash: self.compute_verify_hash(&chunk_results),
            timestamp: get_current_timestamp(),
        })
    }
    
    /// 验证单个 chunk
    async fn verify_chunk(
        &self,
        chunk_idx: usize,
        instructions: &[Instruction],
        cache: Arc<LockFreeCache<Hash, VerifyResult>>,
    ) -> Result<Vec<VerifyResult>> {
        let mut results = Vec::new();
        
        for instruction in instructions {
            // 1. 检查缓存
            let instruction_hash = compute_instruction_hash(instruction);
            
            if let Some(cached_result) = cache.get(&instruction_hash) {
                // 缓存命中
                self.metrics.record_cache_hit();
                results.push(cached_result);
                continue;
            }
            
            // 2. 缓存未命中，执行验证
            let result = self.verify_single_instruction(instruction).await?;
            
            // 3. 写入缓存
            cache.insert(instruction_hash, result.clone());
            
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// 验证单条指令
    async fn verify_single_instruction(
        &self,
        instruction: &Instruction,
    ) -> Result<VerifyResult> {
        // 增量重放验证
        let recomputed_hash = self.replay_and_hash(instruction).await?;
        
        Ok(VerifyResult {
            instruction_id: instruction.id.clone(),
            verified: recomputed_hash == instruction.expected_hash,
            recomputed_hash,
            timestamp: get_current_timestamp(),
        })
    }
}
```

### 3.2 无锁缓存

```rust
/// Phase 3: 无锁缓存 (基于 DashMap + Bloom Filter)
pub struct LockFreeCache<K, V> {
    /// 主缓存
    cache: DashMap<K, V>,
    /// Bloom Filter (快速检查)
    bloom_filter: Arc<RwLock<BloomFilter>>,
    /// 缓存统计
    stats: CacheStats,
}

struct CacheStats {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

impl<K, V> LockFreeCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            bloom_filter: Arc::new(RwLock::new(BloomFilter::new(10000, 0.01))),
            stats: CacheStats::new(),
        }
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        // 1. 先用 Bloom Filter 快速检查
        {
            let bf = self.bloom_filter.read();
            if !bf.contains(key) {
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
                return None;
            }
        }
        
        // 2. 从缓存获取
        match self.cache.get(key) {
            Some(value) => {
                self.stats.hits.fetch_add(1, Ordering::Relaxed);
                Some(value.clone())
            }
            None => {
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }
    
    pub fn insert(&self, key: K, value: V) {
        self.cache.insert(key.clone(), value);
        
        // 更新 Bloom Filter
        {
            let mut bf = self.bloom_filter.write();
            bf.insert(&key);
        }
    }
    
    pub fn hit_rate(&self) -> f64 {
        let hits = self.stats.hits.load(Ordering::Relaxed) as f64;
        let misses = self.stats.misses.load(Ordering::Relaxed) as f64;
        
        if hits + misses > 0.0 {
            hits / (hits + misses)
        } else {
            0.0
        }
    }
}
```

### 3.3 性能优势

| 指标 | 串行验证 | 并行验证 | 提升 |
|---|---|---|---|
| P99 验证时延 | 125ms | <95ms | -24% |
| 缓存命中率 | 60% | 85% | +42% |
| 吞吐量 | 135 请求/秒 | >180 请求/秒 | +33% |
| CPU 利用率 | 45% | 85% | +89% |

---

## 4. SIMD 加速序列化

### 4.1 架构设计

```rust
/// Phase 3: SIMD 加速序列化
pub struct SimdSerializer;

impl SimdSerializer {
    /// 使用 AVX2 加速序列化
    #[target_feature(enable = "avx2")]
    pub unsafe fn serialize_avx2(data: &[u8]) -> Vec<u8> {
        use std::arch::x86_64::*;
        
        let mut result = Vec::with_capacity(data.len() * 2);
        
        // 每次处理 32 字节 (256 位 AVX2)
        let chunks = data.chunks_exact(32);
        let remainder = chunks.remainder();
        
        for chunk in chunks {
            // 加载 32 字节到 AVX2 寄存器
            let v = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            
            // SIMD 处理 (示例：十六进制编码)
            let hex = simd_hex_encode(v);
            
            // 存储结果
            _mm256_storeu_si256(result.as_mut_ptr() as *mut __m256i, hex);
            result.set_len(result.len() + 32);
        }
        
        // 处理剩余字节
        for &byte in remainder {
            result.extend_from_slice(&hex_encode_byte(byte));
        }
        
        result
    }
    
    /// SIMD 十六进制编码
    #[target_feature(enable = "avx2")]
    unsafe fn simd_hex_encode(input: __m256i) -> __m256i {
        // 使用 AVX2 指令并行编码 32 字节
        // 实现细节省略...
        input
    }
}

/// 运行时检测 CPU 特性并选择最优实现
pub fn serialize(data: &[u8]) -> Vec<u8> {
    if is_x86_feature_detected!("avx2") {
        unsafe { SimdSerializer::serialize_avx2(data) }
    } else {
        // 降级到普通实现
        serde_json::to_vec(data).unwrap()
    }
}
```

### 4.2 性能对比

| 序列化方法 | 时延 | 吞吐量 | 相对提升 |
|---|---|---|---|
| serde_json (基准) | 10ms | 100MB/s | - |
| SIMD AVX2 | 6ms | 167MB/s | +67% |
| SIMD SSE4.2 | 7.5ms | 133MB/s | +33% |

---

## 5. MVCC 写时复制优化

### 5.1 CoW 快照实现

```rust
/// Phase 3: 写时复制 (CoW) 事务快照
pub struct CowTransactionSnapshot {
    /// 基础快照 (只读，可共享)
    base_snapshot: Arc<TransactionSnapshot>,
    /// 本地修改 (写时复制)
    local_changes: DashMap<StateKey, Cow<StateVersion>>,
    /// 修改计数
    modification_count: AtomicUsize,
}

impl CowTransactionSnapshot {
    pub fn new(base: TransactionSnapshot) -> Self {
        Self {
            base_snapshot: Arc::new(base),
            local_changes: DashMap::new(),
            modification_count: AtomicUsize::new(0),
        }
    }
    
    /// 读取状态
    pub fn read(&self, key: &StateKey) -> Option<&StateVersion> {
        // 1. 先读本地修改
        if let Some(version) = self.local_changes.get(key) {
            return Some(version);
        }
        
        // 2. 读基础快照
        self.base_snapshot.version_cache.get(key)
    }
    
    /// 写入状态 (写时复制)
    pub fn write(&mut self, key: StateKey, value: StateValue) {
        // 创建新版本 (只复制修改的部分)
        let base_version = self.read(&key).cloned().unwrap_or_default();
        
        let new_version = StateVersion {
            value,  // 只复制值
            version: base_version.version + 1,
            created_by: self.base_snapshot.transaction_id.clone(),
            ..base_version  // 复用其他字段
        };
        
        self.local_changes.insert(key, Cow::Owned(new_version));
        self.modification_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 获取修改计数 (用于优化决策)
    pub fn modification_count(&self) -> usize {
        self.modification_count.load(Ordering::Relaxed)
    }
    
    /// 合并到基础快照 (提交时)
    pub fn merge_into_base(self) -> TransactionSnapshot {
        let mut base = Arc::try_unwrap(self.base_snapshot)
            .unwrap_or_else(|arc| (*arc).clone());
        
        // 合并本地修改
        for (key, version) in self.local_changes {
            base.version_cache.insert(key, version.into_owned());
        }
        
        base
    }
}
```

### 5.2 性能优势

| 操作 | 深拷贝 | CoW | 提升 |
|---|---|---|---|
| 快照创建 | 50ms | 5ms | -90% |
| 快照内存 | 10MB | 2MB | -80% |
| 读操作 | 基准 | 基准 | 0% |
| 写操作 | 基准 | +10% | -10% |

---

## 6. 对象池复用

### 6.1 对象池设计

```rust
/// Phase 3: 通用对象池
pub struct ObjectPool<T> {
    pool: Arc<Mutex<Vec<T>>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
    current_size: AtomicUsize,
}

impl<T> ObjectPool<T>
where
    T: Send + Sync + 'static,
{
    pub fn new<F>(max_size: usize, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size / 2))),
            factory: Arc::new(factory),
            max_size,
            current_size: AtomicUsize::new(0),
        }
    }
    
    pub fn acquire(&self) -> Option<T> {
        // 1. 尝试从池中获取
        let mut pool = self.pool.lock().unwrap();
        
        if let Some(obj) = pool.pop() {
            return Some(obj);
        }
        
        drop(pool);
        
        // 2. 池为空，创建新对象 (如果未超限)
        if self.current_size.load(Ordering::Relaxed) < self.max_size {
            self.current_size.fetch_add(1, Ordering::Relaxed);
            return Some((self.factory)());
        }
        
        None
    }
    
    pub fn release(&self, mut obj: T) {
        // 重置对象状态
        self.reset(&mut obj);
        
        // 返回池中
        let mut pool = self.pool.lock().unwrap();
        
        if pool.len() < self.max_size {
            pool.push(obj);
        }
    }
    
    fn reset(&self, _obj: &mut T) {
        // 子类实现具体重置逻辑
    }
}

/// Phase 3: StateVersion 对象池
pub struct StateVersionPool {
    pool: ObjectPool<StateVersion>,
}

impl StateVersionPool {
    pub fn new() -> Self {
        Self {
            pool: ObjectPool::new(1000, || StateVersion::default()),
        }
    }
    
    pub fn acquire(&self, key: StateKey, value: StateValue) -> StateVersion {
        match self.pool.acquire() {
            Some(mut version) => {
                // 复用并重置
                version.key = key;
                version.value = value;
                version.version = 0;
                version.committed = false;
                version
            }
            None => StateVersion::new(key, value),
        }
    }
    
    pub fn release(&self, version: StateVersion) {
        self.pool.release(version);
    }
}
```

### 6.2 性能优势

| 指标 | 无对象池 | 对象池 | 提升 |
|---|---|---|---|
| 内存分配次数 | 1000 次/秒 | 100 次/秒 | -90% |
| GC 时间 | 15ms | 6ms | -60% |
| P99 时延 | 265ms | 250ms | -6% |

---

## 7. 监控指标

### 7.1 性能优化指标 (8 个)

```rust
// Phase 3: 性能优化监控指标
pub struct PerformanceOptimizationMetrics {
    /// 工作窃取统计
    pub work_steal_count: Counter,
    /// 并行验证 chunk 数
    pub parallel_verification_chunks: Histogram,
    /// 缓存命中率
    pub cache_hit_rate: Gauge,
    /// SIMD 加速使用率
    pub simd_acceleration_ratio: Gauge,
    /// CoW 快照修改率
    pub cow_modification_ratio: Gauge,
    /// 对象池复用率
    pub object_pool_reuse_rate: Gauge,
    /// 执行器 P99 时延
    pub executor_p99_latency: Histogram,
    /// 验证器 P99 时延
    pub verifier_p99_latency: Histogram,
}

impl PerformanceOptimizationMetrics {
    pub fn record_work_steal(&self) {
        self.work_steal_count.inc();
    }
    
    pub fn record_parallel_verification(&self, chunk_count: usize) {
        self.parallel_verification_chunks.observe(chunk_count as f64);
    }
    
    pub fn update_cache_hit_rate(&self, hit_rate: f64) {
        self.cache_hit_rate.set(hit_rate);
    }
    
    pub fn update_simd_ratio(&self, ratio: f64) {
        self.simd_acceleration_ratio.set(ratio);
    }
    
    pub fn update_cow_modification_ratio(&self, ratio: f64) {
        self.cow_modification_ratio.set(ratio);
    }
    
    pub fn update_object_pool_reuse_rate(&self, rate: f64) {
        self.object_pool_reuse_rate.set(rate);
    }
    
    pub fn record_executor_latency(&self, latency_ms: u64) {
        self.executor_p99_latency.observe(latency_ms as f64);
    }
    
    pub fn record_verifier_latency(&self, latency_ms: u64) {
        self.verifier_p99_latency.observe(latency_ms as f64);
    }
}
```

---

## 8. 性能测试

### 8.1 压测配置

```yaml
# Phase 3 性能压测配置 (k6)
# 文件：scripts/performance_test_phase3.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// 自定义指标
const p99Latency = new Rate('p99_latency');
const throughput = new Rate('throughput');

export const options = {
    stages: [
        { duration: '2m', target: 100 },  // 热身
        { duration: '5m', target: 200 },  // 负载
        { duration: '2m', target: 0 },    // 冷却
    ],
    thresholds: {
        'http_req_duration': ['p(99)<200'],  // P99 <200ms
        'p99_latency': ['rate>0.95'],        // 95% 请求达标
        'throughput': ['rate>180'],          // 吞吐量>180 请求/秒
    },
};

export default function () {
    const payload = JSON.stringify({
        instruction: 'UPDATE balance SET +100',
        state_key: 'account_1',
        // ...
    });
    
    const params = {
        headers: { 'Content-Type': 'application/json' },
    };
    
    const res = http.post('http://localhost:8080/execute', payload, params);
    
    check(res, {
        'status is 200': (r) => r.status === 200,
        'latency < 200ms': (r) => r.timings.duration < 200,
    });
    
    sleep(1);
}
```

### 8.2 性能基线目标

| 测试场景 | Phase 2 | Phase 3 目标 | 验证方法 |
|---|---|---|---|
| 单指令 P99 | 265ms | <200ms | k6 压测 |
| Batch P99 (100 条) | 330ms | <250ms | k6 压测 |
| Transaction P99 | 272ms | <200ms | k6 压测 |
| 吞吐量 | 135 请求/秒 | >180 请求/秒 | k6 压测 |
| 72h 稳定性 | 零故障 | 零故障 | 长期压测 |

---

## 9. 性能优化检查清单

| 优化项 | 状态 | 预期提升 | 验证方法 |
|---|---|---|---|
| 工作窃取执行器 | 📋 待实施 | -15% | 压测对比 |
| 并行验证器 | 📋 待实施 | -20% | 压测对比 |
| 无锁缓存 | 📋 待实施 | -10% | 压测对比 |
| SIMD 加速 | 📋 待实施 | -8% | 单元测试 |
| MVCC CoW | 📋 待实施 | -5% | 压测对比 |
| 对象池复用 | 📋 待实施 | -3% | 内存分析 |

---

## 10. 附录

### 10.1 性能分析工具

| 工具 | 用途 | 配置 |
|---|---|---|
| k6 | 负载测试 | 200 并发，10 分钟 |
| Prometheus | 指标采集 | 1s 采集间隔 |
| Grafana | 可视化 | 性能仪表盘 |
| perf | CPU 分析 | 采样频率 99Hz |
| heaptrack | 内存分析 | 全程跟踪 |

### 10.2 性能优化最佳实践

1. **先测量，后优化**: 使用 perf 定位瓶颈
2. **A/B 测试**: 优化前后对比验证
3. **渐进式优化**: 每次只优化一个点
4. **回归测试**: 确保优化不引入新问题
5. **文档化**: 记录优化效果和权衡

---

**文档状态**: 📋 草案  
**责任人**: Architect-Agent + SRE  
**保管**: 项目文档库
