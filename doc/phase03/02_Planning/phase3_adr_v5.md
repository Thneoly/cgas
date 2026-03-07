# Phase 3 架构决策记录 (ADR v5)

**版本**: v5.0 (Phase 3 Kickoff)  
**日期**: 2026-05-12  
**责任人**: Architect-Agent  
**状态**: 📋 草案评审中  
**release_id**: release-2026-05-12-phase3_week01  
**参与角色**: PM, Architect, Dev, QA, SRE, Security, Observability

---

## 1. 架构概述

### 1.1 Phase 3 架构目标

在 Phase 2 架构基础上进行深度优化，聚焦功能扩展与性能深化，保持向后兼容：

| 目标 | Phase 2 状态 | Phase 3 策略 | Phase 3 目标 |
|---|---|---|---|
| Batch 指令 | 单层 (1-100 条) | 嵌套 (2 层) | 支持 Batch 内嵌 Batch |
| Transaction 隔离 | Read Committed | Repeatable Read + MVCC | 支持可重复读 |
| P99 时延 | 265ms/272ms | 异步优化 + 缓存 | **<200ms** |
| 监控指标 | 25 个 | 扩展至 50 个 | 全链路可观测 |
| 分布式追踪 | 部分覆盖 | 全链路覆盖 | Trace 100% 关联 |

### 1.2 Phase 3 架构演进

```
Phase 2 架构 (保持):
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │───▶│   Batch     │───▶│  Executor   │───▶│  Verifier   │
│  (Batch)    │    │  Service    │    │  (Pool)     │    │  (Incremental)│
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                          │                    │                    │
                          ▼                    ▼                    ▼
                   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
                   │ Transaction │───▶│  Executor   │───▶│  Monitoring │
                   │  Service    │    │  (Pool)     │    │  (25 指标)   │
                   └─────────────┘    └─────────────┘    └─────────────┘

Phase 3 架构扩展 (新增):
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │───▶│  Batch      │───▶│  Batch      │
│  (Nested)   │    │  Context    │    │  Executor   │
│             │    │  (嵌套管理)  │    │  (2 层)     │
└─────────────┘    └─────────────┘    └─────────────┘

┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │───▶│  MVCC       │───▶│  Snapshot   │
│  (Trans)    │    │  Manager    │    │  Isolation  │
│             │    │  (多版本)   │    │  (RR)       │
└─────────────┘    └─────────────┘    └─────────────┘

                          │
                          ▼
                   ┌─────────────┐    ┌─────────────┐
                   │  Monitoring │    │ Distributed │
                   │  (50 指标)   │    │  Tracing    │
                   └─────────────┘    └─────────────┘
```

### 1.3 五层分层与双平面边界

| 层级 | Phase 3 组件 | 平面 | 职责 |
|---|---|---|---|
| 接入层 | Gateway, Batch Context, MVCC Manager | 控制平面 | 请求路由、上下文管理 |
| 调度层 | Batch Executor, Transaction Scheduler | 控制平面 | 任务调度、并发控制 |
| 执行层 | Executor Pool, Snapshot Isolation | 数据平面 | 指令执行、状态读取 |
| 验证层 | Verifier (Incremental), MVCC Validator | 数据平面 | 结果验证、隔离检查 |
| 治理层 | Monitoring (50 指标), Distributed Tracing | 控制平面 | 可观测性、审计 |

---

## 2. 架构决策

### 2.1 ADR-007: Batch 嵌套指令架构

| 决策项 | 决策 | 理由 | 权衡 |
|---|---|---|---|
| 嵌套层数 | 2 层 (Batch 内嵌 Batch) | 平衡复杂度与灵活性 | 3 层+ 会显著增加验证复杂度 |
| 执行语义 | 外层串行，内层可并行 | 保证原子性 + 性能优化 | 需要 BatchContext 管理 |
| 哈希链 | 双层哈希 (outer_hash + inner_hash) | 保证嵌套完整性 | 增加 10-15ms 计算开销 |
| 重放策略 | 分层重放 (外层→内层) | 保持确定性 | 重放时延 +8% |
| 性能开销 | <25% (相比单层 Batch) | 用户体验 | 可接受的性能折损 |

**接口设计**:
```rust
/// Phase 3: 支持嵌套的 Batch 指令
pub struct BatchExecuteRequest {
    pub trace_id: String,
    pub batch_id: String,
    pub instructions: Vec<BatchInstruction>,  // Phase 3: 支持嵌套
    pub atomic: bool,
    pub isolation_level: BatchIsolationLevel,  // Phase 3 新增
    pub timestamp: String,
}

/// Phase 3: Batch 指令可以是单条指令或嵌套 Batch
pub enum BatchInstruction {
    Single(ExecuteRequest),
    Nested(BatchExecuteRequest),  // Phase 3 新增
}

/// Phase 3: Batch 隔离级别
pub enum BatchIsolationLevel {
    Sequential,      // 串行执行 (默认)
    ParallelInner,   // 外层串行，内层并行
}

/// Phase 3: Batch 上下文管理
pub struct BatchContext {
    pub batch_id: String,
    pub parent_batch_id: Option<String>,  // Phase 3 新增：父 Batch
    pub depth: u8,                        // Phase 3 新增：嵌套深度 (max=2)
    pub isolation_level: BatchIsolationLevel,
    pub start_time: u64,
    pub state_snapshot: Option<StateSnapshot>,  // Phase 3 新增：状态快照
}

pub struct BatchExecuteResult {
    pub trace_id: String,
    pub batch_id: String,
    pub status: BatchStatus,
    pub results: Vec<BatchInstructionResult>,  // Phase 3: 支持嵌套结果
    pub batch_hash: String,
    pub inner_hash: Option<String>,  // Phase 3 新增：内层哈希
    pub depth: u8,
    pub timestamp: String,
}
```

**状态**: 📋 待评审

---

### 2.2 ADR-008: Transaction 隔离级别增强架构

| 决策项 | 决策 | 理由 | 权衡 |
|---|---|---|---|
| 隔离级别 | Repeatable Read (RR) | 解决幻读问题，增强一致性 | 需要 MVCC，内存开销 +30% |
| MVCC 实现 | 多版本状态快照 | 支持并发读，无锁读取 | 增加 GC 压力 |
| 快照策略 | 事务开始时创建快照 | 保证可重复读 | 快照创建时延 +5ms |
| 版本保留 | 事务期间保留所有版本 | 保证一致性读 | 内存占用 +25% |
| 写冲突检测 | 乐观并发控制 (OCC) | 低冲突场景性能优 | 高冲突场景需重试 |

**接口设计**:
```rust
/// Phase 3: 增强的隔离级别
pub enum IsolationLevel {
    ReadCommitted,      // Phase 2 (保持兼容)
    RepeatableRead,     // Phase 3 新增
    Serializable,       // 预留 (Phase 4)
}

/// Phase 3: MVCC 状态快照
pub struct StateSnapshot {
    pub transaction_id: String,
    pub snapshot_time: u64,
    pub state_versions: HashMap<StateKey, StateVersion>,
    pub read_set: HashSet<StateKey>,  // 读集 (用于冲突检测)
    pub write_set: HashSet<StateKey>, // 写集 (用于冲突检测)
}

/// Phase 3: 多版本状态
pub struct StateVersion {
    pub key: StateKey,
    pub value: StateValue,
    pub version: u64,
    pub created_by: String,  // transaction_id
    pub visible_to: Vec<String>,  // 可见的事务列表
    pub expired_at: Option<u64>,  // 过期时间 (用于 GC)
}

/// Phase 3: MVCC 管理器
pub struct MvccManager {
    versions: DashMap<StateKey, Vec<StateVersion>>,
    active_transactions: DashMap<String, StateSnapshot>,
    gc_interval_ms: u64,
}

impl MvccManager {
    /// 创建事务快照
    pub fn create_snapshot(&self, transaction_id: &str) -> Result<StateSnapshot>;
    
    /// 读取版本 (可见性检查)
    pub fn read_version(&self, key: &StateKey, transaction_id: &str) -> Result<Option<StateVersion>>;
    
    /// 写入新版本
    pub fn write_version(&self, key: &StateKey, value: StateValue, transaction_id: &str) -> Result<()>;
    
    /// 提交事务 (使版本可见)
    pub fn commit(&self, transaction_id: &str) -> Result<()>;
    
    /// 回滚事务 (清理未提交版本)
    pub fn rollback(&self, transaction_id: &str) -> Result<()>;
    
    /// GC 过期版本
    pub fn gc(&self, before_version: u64) -> Result<usize>;
}

pub struct TransactionExecuteRequest {
    pub trace_id: String,
    pub transaction_id: String,
    pub isolation_level: IsolationLevel,
    pub instructions: Vec<ExecuteRequest>,
    pub timeout_ms: i64,
    pub use_mvcc: bool,  // Phase 3 新增：是否启用 MVCC
    pub timestamp: String,
}
```

**MVCC 可见性规则**:
```rust
/// Phase 3: MVCC 可见性检查
pub fn is_visible(version: &StateVersion, transaction_id: &str) -> bool {
    // 规则 1: 创建者自己可见
    if version.created_by == transaction_id {
        return true;
    }
    
    // 规则 2: 已提交且版本在事务开始前
    if version.visible_to.contains(&"committed".to_string()) {
        return true;
    }
    
    // 规则 3: 显式授权给该事务
    if version.visible_to.contains(&transaction_id.to_string()) {
        return true;
    }
    
    false
}
```

**状态**: 📋 待评审

---

### 2.3 ADR-009: 性能优化架构 (P99 <200ms)

| 优化领域 | Phase 2 状态 | Phase 3 优化策略 | 目标提升 | 风险 |
|---|---|---|---|---|
| 执行器 | 异步并发池 (8 线程) | 工作窃取 + 协程 | -15% | 调度复杂度 |
| 验证器 | 增量重放 | 并行验证 + 缓存预热 | -20% | 缓存一致性 |
| 阻断中间件 | 校验缓存 | 无锁缓存 + 批量校验 | -10% | 并发冲突 |
| MVCC 快照 | N/A | 写时复制 (CoW) | -5% | 内存开销 |
| 序列化 | serde + 零拷贝 | SIMD 加速 + 对象池 | -8% | 平台依赖 |
| **总计** | **265ms/272ms** | **综合优化** | **<200ms** | **可控** |

**优化技术栈**:
```rust
// Phase 3: 工作窃取执行器
pub struct WorkStealingExecutor {
    workers: Vec<Worker>,
    global_queue: Arc<ConcurrentQueue<Task>>,
    local_queues: Vec<Arc<ConcurrentQueue<Task>>>,
}

struct Worker {
    id: usize,
    local_queue: Arc<ConcurrentQueue<Task>>,
    thread: JoinHandle<()>,
}

impl Worker {
    fn run(&self) {
        loop {
            // 优先从本地队列获取任务
            if let Some(task) = self.local_queue.pop() {
                self.execute(task);
            }
            // 本地队列为空时，从全局队列或其他 worker 窃取
            else if let Some(task) = self.steal() {
                self.execute(task);
            }
        }
    }
    
    fn steal(&self) -> Option<Task> {
        // 工作窃取算法
        // 1. 尝试从全局队列获取
        // 2. 尝试从其他 worker 队列窃取
    }
}

// Phase 3: 并行验证器
pub struct ParallelVerifier {
    worker_pool: ThreadPool,
    cache: Arc<DashMap<Hash, VerifyResult>>,
}

impl ParallelVerifier {
    pub async fn parallel_verify(&self, request: &BatchVerifyRequest) -> Result<BatchVerifyResponse> {
        // 将 Batch 拆分为多个子任务并行验证
        let futures: Vec<_> = request.instructions
            .par_chunks(10)  // 每 10 条指令一组
            .map(|chunk| self.verify_chunk(chunk))
            .collect();
        
        let results = join_all(futures).await;
        Ok(self.merge_results(results))
    }
}

// Phase 3: 无锁缓存
pub struct LockFreeCache<K, V> {
    cache: DashMap<K, V>,
    bloom_filter: Arc<RwLock<BloomFilter>>,
}

impl<K, V> LockFreeCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn get(&self, key: &K) -> Option<V> {
        // 先用 Bloom Filter 快速检查
        if !self.bloom_filter.read().contains(key) {
            return None;
        }
        
        // 再从缓存获取
        self.cache.get(key).map(|v| v.clone())
    }
    
    pub fn insert(&self, key: K, value: V) {
        self.cache.insert(key.clone(), value);
        self.bloom_filter.write().insert(key);
    }
}

// Phase 3: SIMD 加速序列化
#[target_feature(enable = "avx2")]
unsafe fn simd_serialize(data: &[u8]) -> Vec<u8> {
    // 使用 AVX2 指令集加速序列化
    // 性能提升：30-50%
}
```

**性能目标分解**:
```
Phase 2 基线: 265ms ████████████████████████████████████████████████████
工作窃取 (-15%):  -40ms ████████████████████
并行验证 (-20%):  -53ms ██████████████████████████
无锁缓存 (-10%):  -27ms ██████████████
MVCC CoW (-5%):   -13ms ██████
SIMD 加速 (-8%):   -21ms ██████████
其他优化：        -11ms █████
Phase 3 目标：    <200ms ████████████████████████████████████
```

**状态**: 📋 待评审

---

### 2.4 ADR-010: 50 指标可观测性架构

| 指标类别 | Phase 2 (25 个) | Phase 3 新增 (25 个) | 总计 |
|---|---|---|---|
| 性能指标 | 8 个 | +8 个 (嵌套 Batch/MVCC 性能) | 16 个 |
| 一致性指标 | 4 个 | +4 个 (MVCC 一致性) | 8 个 |
| 安全指标 | 6 个 | +5 个 (威胁检测) | 11 个 |
| 业务指标 | 7 个 | +5 个 (嵌套/隔离业务指标) | 12 个 |
| 追踪指标 | 0 个 | +3 个 (分布式追踪) | 3 个 |
| **总计** | **25 个** | **25 个** | **50 个** |

**新增监控指标**:
```rust
// Batch 嵌套相关 (8 个)
batch_nested_depth_histogram       // 嵌套深度分布
batch_nested_inner_latency_p99     // 内层 Batch 时延
batch_nested_outer_latency_p99     // 外层 Batch 时延
batch_nested_overhead_percent      // 嵌套开销百分比
batch_nested_parallel_ratio        // 并行执行比例
batch_nested_conflict_count        // 嵌套冲突次数
batch_nested_retry_count           // 嵌套重试次数
batch_nested_success_rate          // 嵌套成功率

// MVCC 相关 (4 个)
mvcc_snapshot_creation_latency     // 快照创建时延
mvcc_version_count                 // 版本数量
mvcc_gc_reclaimed_versions         // GC 回收版本数
mvcc_conflict_detection_count      // 冲突检测次数

// 威胁检测相关 (5 个)
threat_detection_alert_count       // 威胁告警数
threat_detection_false_positive_rate  // 误报率
threat_detection_response_time     // 响应时间
threat_detection_policy_violation  // 策略违反数
threat_detection_anomaly_score     // 异常评分

// 业务指标 (5 个)
nested_batch_usage_ratio           // 嵌套 Batch 使用率
repeatable_read_transaction_ratio  // RR 事务占比
mvcc_read_amplification            // MVCC 读放大
transaction_retry_rate             // 事务重试率
isolation_level_distribution       // 隔离级别分布

// 分布式追踪相关 (3 个)
trace_completion_rate              // Trace 完成率
span_duration_p99                  // Span 时延
trace_correlation_accuracy         // Trace 关联准确率
```

**指标采集架构**:
```rust
// Phase 3: 统一指标采集
pub struct MetricsCollector {
    registry: Arc<Registry>,
    exporters: Vec<Box<dyn Exporter>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let registry = Arc::new(Registry::new());
        
        // 注册 50 个指标
        Self::register_performance_metrics(&registry);
        Self::register_consistency_metrics(&registry);
        Self::register_security_metrics(&registry);
        Self::register_business_metrics(&registry);
        Self::register_tracing_metrics(&registry);
        
        Self {
            registry,
            exporters: vec![
                Box::new(PrometheusExporter::new()),
                Box::new(GrafanaExporter::new()),
                Box::new(LogExporter::new()),
            ],
        }
    }
    
    pub fn record_batch_nested_depth(&self, depth: u8) {
        self.registry
            .get_or_create::<Histogram>(
                "batch_nested_depth_histogram",
                "Batch 嵌套深度分布"
            )
            .observe(depth as f64);
    }
    
    pub fn record_mvcc_snapshot_latency(&self, latency_ms: f64) {
        self.registry
            .get_or_create::<Histogram>(
                "mvcc_snapshot_creation_latency",
                "MVCC 快照创建时延"
            )
            .observe(latency_ms);
    }
}
```

**状态**: 📋 待评审

---

### 2.5 ADR-011: 分布式追踪全链路架构

| 决策项 | 决策 | 理由 | 权衡 |
|---|---|---|---|
| 追踪标准 | OpenTelemetry 1.0+ | 行业标准，生态完善 | 学习曲线 |
| Trace 传播 | W3C Trace Context | 跨服务兼容 | 增加请求头开销 |
| Span 层级 | 5 层 (Request→Batch/Trans→Executor→Verifier→Commit) | 完整链路 | Span 数量增加 |
| 采样策略 | 自适应采样 (错误 100% + 正常 10%) | 保证关键链路 | 存储成本 |
| 存储后端 | Tempo + S3 | 开源，成本低 | 运维复杂度 |

**Trace 层级设计**:
```
Trace (trace_id: "abc123...")
├── Span: Gateway.receive (span_id: "span_001")
│   ├── Span: BatchExecute (span_id: "span_002")
│   │   ├── Span: BatchContext.create (span_id: "span_003")
│   │   ├── Span: Executor.execute (instruction_1) (span_id: "span_004")
│   │   │   ├── Span: Verifier.verify (span_id: "span_005")
│   │   │   │   └── Span: IncrementalReplay.replay (span_id: "span_006")
│   │   │   └── Span: Commit.commit (span_id: "span_007")
│   │   ├── Span: Executor.execute (instruction_2) (span_id: "span_008")
│   │   │   ├── Span: Verifier.verify (span_id: "span_009")
│   │   │   └── Span: Commit.commit (span_id: "span_010")
│   │   └── Span: BatchContext.close (span_id: "span_011")
│   └── Span: Monitoring.record (span_id: "span_012")
│       └── Span: MetricsCollector.collect (span_id: "span_013")
```

**状态**: 📋 待评审

---

## 3. 接口契约扩展

### 3.1 Phase 1 & Phase 2 接口 (保持向后兼容)

| 接口 | 状态 | Phase 3 策略 |
|---|---|---|
| ExecuteRequest | 冻结 | 完全兼容 |
| BatchExecuteRequest (v1) | 冻结 | 完全兼容 |
| TransactionExecuteRequest (v1) | 冻结 | 完全兼容 |

### 3.2 Phase 3 新增接口

```rust
// Batch 嵌套接口
pub enum BatchInstruction {
    Single(ExecuteRequest),
    Nested(BatchExecuteRequest),
}

pub struct BatchExecuteRequestV2 {
    // 继承 v1 字段
    pub trace_id: String,
    pub batch_id: String,
    pub instructions: Vec<BatchInstruction>,  // Phase 3: 支持嵌套
    pub atomic: bool,
    
    // Phase 3 新增
    pub isolation_level: BatchIsolationLevel,
    pub max_depth: u8,  // 默认=2
}

// MVCC 相关接口
pub struct CreateSnapshotRequest {
    pub transaction_id: String,
    pub isolation_level: IsolationLevel,
    pub timestamp: String,
}

pub struct CreateSnapshotResponse {
    pub snapshot_id: String,
    pub snapshot_time: u64,
    pub version_count: u64,
}

pub struct ReadVersionRequest {
    pub transaction_id: String,
    pub key: StateKey,
    pub snapshot_id: String,
}

pub struct ReadVersionResponse {
    pub value: Option<StateValue>,
    pub version: u64,
    pub visible: bool,
}
```

### 3.3 gRPC 服务定义 (Phase 3 扩展)

```protobuf
// Phase 1 & 2 服务 (保持)
service ExecutorService {
  rpc Execute(ExecuteRequest) returns (ExecutionResult);
}

service BatchService {
  rpc BatchExecute(BatchExecuteRequest) returns (BatchExecuteResult);
}

service TransactionService {
  rpc BeginTransaction(BeginTransactionRequest) returns (BeginTransactionResponse);
  rpc CommitTransaction(CommitTransactionRequest) returns (CommitTransactionResponse);
  rpc RollbackTransaction(RollbackTransactionRequest) returns (RollbackTransactionResponse);
}

// Phase 3 新增服务
service BatchServiceV2 {
  // 支持嵌套 Batch
  rpc BatchExecuteV2(BatchExecuteRequestV2) returns (BatchExecuteResultV2);
}

service MvccService {
  // MVCC 快照管理
  rpc CreateSnapshot(CreateSnapshotRequest) returns (CreateSnapshotResponse);
  rpc ReadVersion(ReadVersionRequest) returns (ReadVersionResponse);
  rpc CommitSnapshot(CommitSnapshotRequest) returns (CommitSnapshotResponse);
  rpc RollbackSnapshot(RollbackSnapshotRequest) returns (RollbackSnapshotResponse);
}

service ObservabilityService {
  // 50 指标查询
  rpc GetMetrics(GetMetricsRequest) returns (GetMetricsResponse);
  rpc GetTrace(GetTraceRequest) returns (GetTraceResponse);
  rpc SearchTraces(SearchTracesRequest) returns (SearchTracesResponse);
}
```

---

## 4. 性能优化方案

### 4.1 性能瓶颈分析 (Phase 2)

| 组件 | P99 时延 | 占比 | Phase 3 优化空间 |
|---|---|---|---|
| 执行器 | 115ms | 43% | 工作窃取 (-15%) |
| 验证器 | 125ms | 47% | 并行验证 (-20%) |
| 阻断中间件 | 15ms | 6% | 无锁缓存 (-10%) |
| 序列化 | 10ms | 4% | SIMD 加速 (-8%) |

### 4.2 优化目标分解

| 组件 | Phase 2 P99 | Phase 3 目标 | 优化幅度 | 关键技术 |
|---|---|---|---|---|
| 执行器 | 115ms | <95ms | -17% | 工作窃取 + 协程 |
| 验证器 | 125ms | <95ms | -24% | 并行验证 + 缓存 |
| 阻断中间件 | 15ms | <13ms | -13% | 无锁缓存 |
| 序列化 | 10ms | <9ms | -10% | SIMD 加速 |
| MVCC 快照 | N/A | <8ms | - | CoW 优化 |
| **总计** | **265ms** | **<200ms** | **-25%** | **综合优化** |

### 4.3 优化实施计划

| 周次 | 优化项 | 责任人 | 验证方法 | 预期提升 |
|---|---|---|---|---|
| Week 2 | 工作窃取执行器 | Dev | 压测 | -15% |
| Week 2 | 并行验证器 | Dev | 压测 | -20% |
| Week 3 | 无锁缓存 | Dev+SRE | 压测 | -10% |
| Week 3 | MVCC CoW 优化 | Dev | 压测 | -5% |
| Week 4 | SIMD 加速 | Dev | 压测 | -8% |
| Week 4 | 性能基线 v5 | SRE | 性能报告 | 验证<200ms |

---

## 5. 安全架构

### 5.1 Phase 2 安全机制 (保持)

| 机制 | Phase 2 状态 | Phase 3 策略 |
|---|---|---|
| SG-1~SG-4 闸门 | 硬阻断 | 保持 + 增强 |
| 未验证提交阻断 | 100% | 保持 |
| 哈希链验证 | 双层哈希 | 扩展至三层 (嵌套) |
| 零信任架构 | OIDC+RBAC+ABAC | 保持 + 威胁检测 |

### 5.2 Phase 3 安全增强

| 增强项 | 实现方案 | 优先级 | 安全收益 |
|---|---|---|---|
| 嵌套 Batch 验证 | 三层哈希链 | P0 | 防止嵌套篡改 |
| MVCC 隔离检查 | 版本可见性验证 | P0 | 防止隔离违反 |
| 威胁检测 | 异常行为分析 | P1 | 主动防御 |
| 审计日志增强 | 嵌套/隔离审计 | P1 | 可追溯性 |

---

## 6. 可观测性架构

### 6.1 监控指标 (50 个)

| 类别 | Phase 2 | Phase 3 新增 | 总计 |
|---|---|---|---|
| 性能 | 8 | 8 | 16 |
| 一致性 | 4 | 4 | 8 |
| 安全 | 6 | 5 | 11 |
| 业务 | 7 | 5 | 12 |
| 追踪 | 0 | 3 | 3 |
| **总计** | **25** | **25** | **50** |

### 6.2 日志规范

| 日志类型 | Phase 2 | Phase 3 扩展 |
|---|---|---|
| 审计日志 | trace_id/batch_id/transaction_id | + nested_depth/isolation_level |
| 阻断日志 | 完整记录 | + mvcc_conflict_info |
| 性能日志 | 分指令类型统计 | + 嵌套/隔离维度 |

### 6.3 分布式追踪

| 层级 | Trace 覆盖 | Span 层级 | 采样率 |
|---|---|---|---|
| Request 级 | 100% | 1 | 100% |
| Batch/Trans 级 | 100% | 1 | 100% |
| Executor 级 | 100% | N | 100% |
| Verifier 级 | 100% | N | 100% |
| Commit 级 | 100% | N | 100% |
| **Trace 完成率** | **≥99%** | **5 层** | **自适应** |

---

## 7. 失败路径与回滚

### 7.1 Batch 嵌套失败处理

| 失败场景 | 处理策略 | 回滚机制 | 影响范围 |
|---|---|---|---|
| 内层指令失败 (atomic=true) | 内层回滚，外层继续 | 自动 REVERT | 内层 |
| 内层指令失败 (atomic=false) | 记录失败，继续执行 | 部分提交 | 单指令 |
| 外层指令失败 | 全部回滚 | 自动 REVERT | 整个嵌套 Batch |
| 嵌套深度超限 | 阻断执行 | 错误返回 | 请求级 |
| 嵌套哈希验证失败 | 阻断提交 | 审计日志 | 整个嵌套 Batch |

### 7.2 MVCC 失败处理

| 失败场景 | 处理策略 | 回滚机制 | 影响范围 |
|---|---|---|---|
| 快照创建失败 | 降级为 RC | 使用 Phase 2 路径 | 事务级 |
| 版本冲突检测 | 事务重试 | 自动回滚 + 重试 | 事务级 |
| MVCC 内存超限 | 触发 GC | 强制回收旧版本 | 系统级 |
| 可见性检查失败 | 阻断读取 | 错误返回 | 读操作 |
| 提交冲突 | OCC 重试 | 自动重试 (max=3) | 事务级 |

### 7.3 回滚策略

```rust
// Phase 3: 嵌套 Batch 回滚
impl BatchExecutor {
    pub async fn rollback_nested_batch(&self, context: &BatchContext) -> Result<()> {
        // 1. 回滚内层 Batch (如果有)
        for instruction in &context.instructions {
            if let BatchInstruction::Nested(nested_batch) = instruction {
                self.rollback_batch(&nested_batch.batch_id).await?;
            }
        }
        
        // 2. 回滚外层 Batch
        self.rollback_batch(&context.batch_id).await?;
        
        // 3. 记录审计日志
        self.audit_log.rollback_nested_batch(context).await;
        
        Ok(())
    }
}

// Phase 3: MVCC 回滚
impl MvccManager {
    pub fn rollback_transaction(&self, transaction_id: &str) -> Result<()> {
        // 1. 清理未提交版本
        self.cleanup_uncommitted_versions(transaction_id)?;
        
        // 2. 释放快照
        self.active_transactions.remove(transaction_id);
        
        // 3. 记录审计日志
        self.audit_log.mvcc_rollback(transaction_id).await;
        
        Ok(())
    }
}
```

---

## 8. 架构评审状态

### 8.1 待评审决策

| ADR ID | 决策主题 | 状态 | 责任人 | 优先级 |
|---|---|---|---|---|
| ADR-007 | Batch 嵌套指令架构 | 📋 待评审 | Architect | P0 |
| ADR-008 | Transaction 隔离增强 | 📋 待评审 | Architect | P0 |
| ADR-009 | 性能优化架构 | 📋 待评审 | Dev+SRE | P0 |
| ADR-010 | 50 指标可观测性 | 📋 待评审 | SRE+Observability | P1 |
| ADR-011 | 分布式追踪全链路 | 📋 待评审 | Observability | P1 |

### 8.2 评审时间安排

| 时间点 | 事件 | 参与方 | 产出 |
|---|---|---|---|
| Week 1-T3 | ADR v5 初稿完成 | Architect | phase3_adr_v5.md |
| Week 1-T4 | 架构评审会议 | Architect+Dev+ 各角色 | 评审意见 |
| Week 1-T5 | Entry Gate 评审 | 门禁官 + 四方 | Go/No-Go 决策 |
| Week 2-T1 | ADR v5 定稿 | Architect | phase3_adr_v5_final.md |

---

## 9. 架构权衡分析

### 9.1 Batch 嵌套权衡

| 方案 | 优点 | 缺点 | 选择理由 |
|---|---|---|---|
| 2 层嵌套 | 复杂度可控，性能可接受 | 灵活性受限 | ✅ 平衡点 |
| 3 层+ 嵌套 | 灵活性高 | 验证复杂度指数增长 | ❌ 不选 |
| 无限嵌套 | 最灵活 | 无法保证确定性 | ❌ 不选 |

### 9.2 MVCC 权衡

| 方案 | 优点 | 缺点 | 选择理由 |
|---|---|---|---|
| MVCC + RR | 无锁读，一致性好 | 内存开销 +30% | ✅ 一致性优先 |
| 锁机制 + RR | 内存开销小 | 读写冲突，性能差 | ❌ 性能影响大 |
| 保持 RC | 实现简单 | 存在幻读问题 | ❌ 不满足需求 |

### 9.3 性能优化权衡

| 方案 | 优点 | 缺点 | 选择理由 |
|---|---|---|---|
| 工作窃取 | 负载均衡好 | 调度复杂度 | ✅ 适合多核 |
| 固定线程池 | 实现简单 | 负载不均 | ❌ 扩展性差 |
| 协程 | 轻量级并发 | 运行时依赖 | ✅ 结合使用 |

---

## 10. 附录

### 10.1 Phase 1 & 2 ADR 参考

| ADR | 主题 | 状态 |
|---|---|---|
| ADR v1 | Phase 1 基础架构 | ✅ 已实施 |
| ADR v2 | 验证器重放链路 | ✅ 已实施 |
| ADR v3 | 阻断中间件部署 | ✅ 已实施 |
| ADR v4 | Phase 2 架构扩展 | ✅ 已实施 |

### 10.2 术语表

| 术语 | 定义 |
|---|---|
| Batch 嵌套 | Batch 内嵌 Batch，最多 2 层 |
| MVCC | Multi-Version Concurrency Control，多版本并发控制 |
| Repeatable Read | 可重复读隔离级别，解决幻读 |
| 工作窃取 | Work Stealing，负载均衡调度算法 |
| CoW | Copy-on-Write，写时复制优化 |

### 10.3 关键设计模式

| 模式 | 应用场景 | 实现位置 |
|---|---|---|
| Context 模式 | Batch 上下文管理 | `batch_context.rs` |
| MVCC 模式 | 多版本状态管理 | `mvcc_manager.rs` |
| 工作窃取模式 | 执行器调度 | `work_stealing_executor.rs` |
| 观察者模式 | 指标采集 | `metrics_collector.rs` |

---

**文档状态**: 📋 草案评审中  
**下次更新**: Week 1-T4 (架构评审后)  
**责任人**: Architect-Agent  
**保管**: 项目文档库
