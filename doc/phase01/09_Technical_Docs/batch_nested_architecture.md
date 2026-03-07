# Batch 嵌套指令架构设计

**版本**: v1.0  
**日期**: 2026-05-12  
**责任人**: Architect-Agent  
**状态**: 📋 草案  
**release_id**: release-2026-05-12-phase3_week01  
**关联 ADR**: ADR-007 (Phase 3 ADR v5)

---

## 1. 设计目标

### 1.1 Phase 2 vs Phase 3 Batch 对比

| 特性 | Phase 2 (单层) | Phase 3 (嵌套) | 改进 |
|---|---|---|---|
| Batch 结构 | 扁平指令列表 | 树形结构 (最多 2 层) | 支持复杂业务场景 |
| 执行语义 | 串行执行 | 外层串行，内层可并行 | 性能优化 |
| 哈希链 | 单层 batch_hash | 双层哈希 (outer + inner) | 增强完整性验证 |
| 最大指令数 | 100 条 | 外层 100 × 内层 100 = 10,000 条 | 扩展性提升 |
| 性能开销 | 基准 | <25% (相比单层) | 可接受的性能折损 |

### 1.2 使用场景

| 场景 | Phase 2 限制 | Phase 3 解决方案 |
|---|---|---|
| 批量用户批量操作 | 需要多次 Batch 调用 | 单次嵌套 Batch |
| 分层数据更新 | 无法表达层级关系 | 嵌套 Batch 天然支持 |
| 条件批量操作 | 需要客户端协调 | 内层 Batch 原子性保证 |
| 跨服务批量调用 | 多次网络往返 | 单次嵌套 Batch |

---

## 2. 数据结构设计

### 2.1 核心数据结构

```rust
/// Phase 3: Batch 指令 (支持嵌套)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchInstruction {
    /// 单条指令
    Single(ExecuteRequest),
    /// 嵌套 Batch (Phase 3 新增)
    Nested(Box<BatchExecuteRequest>),
}

/// Phase 3: Batch 执行请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecuteRequest {
    /// 追踪 ID
    pub trace_id: String,
    /// Batch 唯一标识
    pub batch_id: String,
    /// 指令列表 (支持嵌套)
    pub instructions: Vec<BatchInstruction>,
    /// 原子性保证
    pub atomic: bool,
    /// Phase 3 新增：隔离级别
    pub isolation_level: BatchIsolationLevel,
    /// Phase 3 新增：最大嵌套深度 (默认=2)
    pub max_depth: u8,
    /// 时间戳
    pub timestamp: String,
}

/// Phase 3: Batch 隔离级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchIsolationLevel {
    /// 串行执行 (默认，兼容 Phase 2)
    Sequential,
    /// 外层串行，内层并行 (Phase 3 新增)
    ParallelInner,
}

impl Default for BatchIsolationLevel {
    fn default() -> Self {
        Self::Sequential
    }
}

/// Phase 3: Batch 上下文 (管理嵌套状态)
pub struct BatchContext {
    /// Batch ID
    pub batch_id: String,
    /// 父 Batch ID (Phase 3 新增)
    pub parent_batch_id: Option<String>,
    /// 嵌套深度 (Phase 3 新增，0=外层，1=内层)
    pub depth: u8,
    /// 隔离级别
    pub isolation_level: BatchIsolationLevel,
    /// 开始时间
    pub start_time: u64,
    /// Phase 3 新增：状态快照 (用于回滚)
    pub state_snapshot: Option<StateSnapshot>,
    /// 子 Batch 上下文 (Phase 3 新增)
    pub child_contexts: Vec<BatchContext>,
}

impl BatchContext {
    /// 创建外层 Batch 上下文
    pub fn new_outer(batch_id: String, isolation_level: BatchIsolationLevel) -> Self {
        Self {
            batch_id,
            parent_batch_id: None,
            depth: 0,
            isolation_level,
            start_time: get_current_timestamp(),
            state_snapshot: None,
            child_contexts: Vec::new(),
        }
    }
    
    /// 创建内层 Batch 上下文 (Phase 3 新增)
    pub fn new_inner(
        batch_id: String,
        parent_batch_id: String,
        isolation_level: BatchIsolationLevel,
    ) -> Result<Self> {
        // 检查嵌套深度
        if parent_depth >= 2 {
            return Err(BatchError::MaxDepthExceeded);
        }
        
        Ok(Self {
            batch_id,
            parent_batch_id: Some(parent_batch_id),
            depth: parent_depth + 1,
            isolation_level,
            start_time: get_current_timestamp(),
            state_snapshot: None,
            child_contexts: Vec::new(),
        })
    }
    
    /// 检查是否可以嵌套 (Phase 3 新增)
    pub fn can_nested(&self) -> bool {
        self.depth < 2  // 最大深度=2
    }
}

/// Phase 3: Batch 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecuteResult {
    /// 追踪 ID
    pub trace_id: String,
    /// Batch ID
    pub batch_id: String,
    /// Batch 状态
    pub status: BatchStatus,
    /// 指令结果 (支持嵌套)
    pub results: Vec<BatchInstructionResult>,
    /// Phase 3 新增：外层哈希
    pub batch_hash: String,
    /// Phase 3 新增：内层哈希 (如果有嵌套)
    pub inner_hash: Option<String>,
    /// Phase 3 新增：嵌套深度
    pub depth: u8,
    /// 时间戳
    pub timestamp: String,
}

/// Phase 3: Batch 指令结果 (支持嵌套)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchInstructionResult {
    /// 单条指令结果
    Single(ExecutionResult),
    /// 嵌套 Batch 结果 (Phase 3 新增)
    Nested(Box<BatchExecuteResult>),
}

/// Phase 3: Batch 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchStatus {
    /// 成功
    Success,
    /// 部分失败 (atomic=false 时)
    PartialFailure,
    /// 全部失败
    Failed,
    /// 嵌套失败 (Phase 3 新增)
    NestedFailure {
        outer_batch_id: String,
        inner_batch_id: String,
        error: String,
    },
}
```

### 2.2 哈希链设计

```rust
/// Phase 3: 双层哈希链计算
pub struct BatchHashChain;

impl BatchHashChain {
    /// 计算内层 Batch 哈希 (Phase 3 新增)
    pub fn compute_inner_hash(instructions: &[BatchInstruction]) -> String {
        let mut hasher = Sha256::new();
        
        for (index, instruction) in instructions.iter().enumerate() {
            // 索引 + 指令哈希
            hasher.update(index.to_le_bytes());
            
            let instruction_hash = match instruction {
                BatchInstruction::Single(req) => {
                    Self::compute_single_hash(req)
                }
                BatchInstruction::Nested(req) => {
                    // 递归计算嵌套 Batch 哈希
                    Self::compute_batch_hash(req)
                }
            };
            
            hasher.update(instruction_hash.as_bytes());
        }
        
        hex_encode(hasher.finalize())
    }
    
    /// 计算外层 Batch 哈希 (增强 Phase 2)
    pub fn compute_outer_hash(
        batch_id: &str,
        inner_hash: Option<&str>,
        instructions: &[BatchInstruction],
    ) -> String {
        let mut hasher = Sha256::new();
        
        // Batch ID
        hasher.update(batch_id.as_bytes());
        
        // Phase 3: 如果有内层哈希，加入计算
        if let Some(hash) = inner_hash {
            hasher.update(hash.as_bytes());
        }
        
        // 外层指令哈希
        for instruction in instructions {
            if let BatchInstruction::Single(req) = instruction {
                hasher.update(Self::compute_single_hash(req).as_bytes());
            }
        }
        
        hex_encode(hasher.finalize())
    }
    
    /// 计算完整 Batch 哈希 (Phase 3 新增)
    pub fn compute_batch_hash(request: &BatchExecuteRequest) -> String {
        // 1. 计算内层哈希 (如果有嵌套)
        let inner_hash = Self::compute_inner_hash(&request.instructions);
        
        // 2. 计算外层哈希
        let outer_hash = Self::compute_outer_hash(
            &request.batch_id,
            Some(&inner_hash),
            &request.instructions,
        );
        
        // 3. 返回双层哈希
        outer_hash
    }
}
```

---

## 3. 执行器设计

### 3.1 嵌套 Batch 执行器架构

```rust
/// Phase 3: 嵌套 Batch 执行器
pub struct NestedBatchExecutor {
    /// 执行器池
    executor_pool: Arc<ExecutorPool>,
    /// 验证器
    verifier: Arc<Verifier>,
    /// 提交器
    committer: Arc<Committer>,
    /// 批上下文管理器
    context_manager: Arc<BatchContextManager>,
    /// 监控指标
    metrics: Arc<BatchMetrics>,
}

impl NestedBatchExecutor {
    /// 执行嵌套 Batch (Phase 3 新增)
    pub async fn execute_nested_batch(
        &self,
        request: BatchExecuteRequest,
    ) -> Result<BatchExecuteResult> {
        let start_time = Instant::now();
        
        // 1. 创建 Batch 上下文
        let mut context = BatchContext::new_outer(
            request.batch_id.clone(),
            request.isolation_level,
        );
        
        // 2. 验证嵌套深度
        self.validate_nested_depth(&request.instructions)?;
        
        // 3. 执行 Batch
        let results = match request.isolation_level {
            BatchIsolationLevel::Sequential => {
                self.execute_sequential(&mut context, &request.instructions).await?
            }
            BatchIsolationLevel::ParallelInner => {
                self.execute_parallel_inner(&mut context, &request.instructions).await?
            }
        };
        
        // 4. 计算双层哈希
        let inner_hash = Some(BatchHashChain::compute_inner_hash(&request.instructions));
        let batch_hash = BatchHashChain::compute_outer_hash(
            &request.batch_id,
            inner_hash.as_deref(),
            &request.instructions,
        );
        
        // 5. 确定 Batch 状态
        let status = self.determine_batch_status(&results, request.atomic);
        
        // 6. 记录监控指标
        self.metrics.record_nested_batch_execution(
            context.depth,
            start_time.elapsed().as_millis() as u64,
            &status,
        );
        
        Ok(BatchExecuteResult {
            trace_id: request.trace_id,
            batch_id: request.batch_id,
            status,
            results,
            batch_hash,
            inner_hash,
            depth: context.depth,
            timestamp: get_current_timestamp(),
        })
    }
    
    /// 串行执行 (兼容 Phase 2)
    async fn execute_sequential(
        &self,
        context: &mut BatchContext,
        instructions: &[BatchInstruction],
    ) -> Result<Vec<BatchInstructionResult>> {
        let mut results = Vec::new();
        
        for instruction in instructions {
            let result = match instruction {
                BatchInstruction::Single(req) => {
                    // 执行单条指令
                    let exec_result = self.executor_pool.execute(req).await?;
                    BatchInstructionResult::Single(exec_result)
                }
                BatchInstruction::Nested(req) => {
                    // 递归执行嵌套 Batch
                    if !context.can_nested() {
                        return Err(BatchError::MaxDepthExceeded);
                    }
                    
                    // 创建内层上下文
                    let mut inner_context = BatchContext::new_inner(
                        req.batch_id.clone(),
                        context.batch_id.clone(),
                        req.isolation_level,
                    )?;
                    
                    // 递归执行
                    let inner_result = self.execute_nested_batch(*req.clone()).await?;
                    
                    // 记录子上下文
                    context.child_contexts.push(inner_context);
                    
                    BatchInstructionResult::Nested(Box::new(inner_result))
                }
            };
            
            // 原子性检查
            if context.atomic && result.is_failed() {
                // 回滚已执行的指令
                self.rollback_batch(context).await?;
                return Err(BatchError::AtomicityViolation);
            }
            
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// 并行执行内层 Batch (Phase 3 新增)
    async fn execute_parallel_inner(
        &self,
        context: &mut BatchContext,
        instructions: &[BatchInstruction],
    ) -> Result<Vec<BatchInstructionResult>> {
        // 将指令分为单条指令和嵌套 Batch
        let singles: Vec<_> = instructions.iter()
            .filter_map(|i| match i {
                BatchInstruction::Single(req) => Some(req),
                _ => None,
            })
            .collect();
        
        let nested: Vec<_> = instructions.iter()
            .filter_map(|i| match i {
                BatchInstruction::Nested(req) => Some(req),
                _ => None,
            })
            .collect();
        
        // 并行执行单条指令和嵌套 Batch
        let single_futures = singles.iter()
            .map(|req| async {
                let exec_result = self.executor_pool.execute(req).await?;
                Ok::<_, Error>(BatchInstructionResult::Single(exec_result))
            });
        
        let nested_futures = nested.iter()
            .map(|req| async {
                if !context.can_nested() {
                    return Err(BatchError::MaxDepthExceeded);
                }
                
                let inner_result = self.execute_nested_batch(*req.clone()).await?;
                Ok::<_, Error>(BatchInstructionResult::Nested(Box::new(inner_result)))
            });
        
        // 合并所有 futures 并行执行
        let all_futures = single_futures.chain(nested_futures);
        let results = try_join_all(all_futures).await?;
        
        Ok(results)
    }
    
    /// 验证嵌套深度 (Phase 3 新增)
    fn validate_nested_depth(&self, instructions: &[BatchInstruction]) -> Result<()> {
        for instruction in instructions {
            if let BatchInstruction::Nested(req) = instruction {
                // 检查是否超过最大深度
                if req.max_depth > 2 {
                    return Err(BatchError::MaxDepthExceeded);
                }
                
                // 递归检查内层
                self.validate_nested_depth(&req.instructions)?;
            }
        }
        
        Ok(())
    }
    
    /// 回滚 Batch (增强 Phase 2)
    async fn rollback_batch(&self, context: &BatchContext) -> Result<()> {
        // 1. 回滚子 Batch (Phase 3 新增)
        for child_context in &context.child_contexts {
            self.rollback_batch(child_context).await?;
        }
        
        // 2. 回滚当前 Batch
        if let Some(snapshot) = &context.state_snapshot {
            self.committer.rollback(snapshot).await?;
        }
        
        // 3. 记录审计日志
        self.audit_log.batch_rollback(context).await;
        
        Ok(())
    }
    
    /// 确定 Batch 状态
    fn determine_batch_status(
        &self,
        results: &[BatchInstructionResult],
        atomic: bool,
    ) -> BatchStatus {
        let has_failure = results.iter().any(|r| r.is_failed());
        let has_success = results.iter().any(|r| r.is_success());
        
        if !has_failure {
            BatchStatus::Success
        } else if atomic {
            BatchStatus::Failed
        } else if has_success {
            BatchStatus::PartialFailure
        } else {
            BatchStatus::Failed
        }
    }
}
```

### 3.2 执行流程图

```
Phase 3 嵌套 Batch 执行流程:

Client Request (Nested Batch)
         │
         ▼
┌─────────────────┐
│  Gateway        │ 解析嵌套 Batch 请求
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  BatchContext   │ 创建外层上下文 (depth=0)
│  Manager        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Depth          │ 验证嵌套深度 ≤2
│  Validator      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Isolation      │ 选择执行策略
│  Checker        │ ┌──────────────┐
└────────┬────────┘ │ Sequential   │
         │          │ ParallelInner│
         │          └──────┬───────┘
         │                 │
         ▼                 ▼
┌──────────────────────────────────┐
│      Executor Pool               │
│  ┌────────────┐  ┌────────────┐  │
│  │ Single     │  │ Nested     │  │
│  │ Execute    │  │ Execute    │  │
│  │ (depth=0)  │  │ (depth=1)  │  │
│  └────────────┘  └────────────┘  │
└─────────────────┬────────────────┘
                  │
                  ▼
         ┌─────────────────┐
         │  Verifier       │ 并行验证
         │  (Incremental)  │
         └────────┬────────┘
                  │
                  ▼
         ┌─────────────────┐
         │  Committer      │ 原子提交
         └────────┬────────┘
                  │
                  ▼
         ┌─────────────────┐
         │  Hash Chain     │ 计算双层哈希
         │  (outer+inner)  │
         └────────┬────────┘
                  │
                  ▼
         ┌─────────────────┐
         │  Monitoring     │ 记录 50 指标
         │  (50 metrics)   │
         └────────┬────────┘
                  │
                  ▼
         Client Response (Nested Result)
```

---

## 4. 性能优化

### 4.1 性能开销分析

| 优化项 | Phase 2 单层 | Phase 3 嵌套 | 开销 |
|---|---|---|---|
| 哈希计算 | 1 次 (batch_hash) | 2 次 (outer+inner) | +10-15ms |
| 上下文管理 | 简单 | 树形结构 | +5-8ms |
| 递归执行 | 无 | 递归调用 | +8-12ms |
| 状态快照 | 可选 | 必需 (用于回滚) | +5-10ms |
| **总计** | **基准** | **嵌套** | **+28-45ms** |

### 4.2 优化策略

```rust
// Phase 3: 懒加载状态快照
impl BatchContext {
    pub fn create_snapshot_lazy(&mut self) -> Result<()> {
        // 仅在 atomic=true 时创建快照
        if self.atomic {
            self.state_snapshot = Some(self.capture_snapshot()?);
        }
        Ok(())
    }
}

// Phase 3: 增量哈希计算
pub struct IncrementalBatchHash {
    hasher: Sha256,
    instruction_count: usize,
}

impl IncrementalBatchHash {
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
            instruction_count: 0,
        }
    }
    
    pub fn add_instruction(&mut self, hash: &str) {
        self.hasher.update(self.instruction_count.to_le_bytes());
        self.hasher.update(hash.as_bytes());
        self.instruction_count += 1;
    }
    
    pub fn finalize(self) -> String {
        hex_encode(self.hasher.finalize())
    }
}

// Phase 3: 对象池复用 BatchContext
pub struct BatchContextPool {
    pool: ObjectPool<BatchContext>,
}

impl BatchContextPool {
    pub fn acquire(&self) -> BatchContext {
        self.pool.acquire().unwrap_or_else(|| {
            BatchContext::new_outer("".to_string(), BatchIsolationLevel::Sequential)
        })
    }
    
    pub fn release(&self, context: BatchContext) {
        // 重置上下文状态
        let mut reset_context = context;
        reset_context.child_contexts.clear();
        reset_context.state_snapshot = None;
        
        self.pool.release(reset_context);
    }
}
```

### 4.3 性能基线目标

| 指标 | Phase 2 单层 | Phase 3 嵌套目标 | 测量方法 |
|---|---|---|---|
| P99 时延 (100 条) | 265ms | <330ms (+25%) | k6 压测 |
| P99 时延 (10 层嵌套) | N/A | <350ms | k6 压测 |
| 吞吐量 | 135 请求/秒 | >100 请求/秒 | k6 压测 |
| 内存占用 | 基准 | +30% | Prometheus |

---

## 5. 测试策略

### 5.1 测试用例分类

| 类别 | 用例数 | 覆盖场景 |
|---|---|---|
| 单元测试 | 20 | 数据结构、哈希计算、上下文管理 |
| 集成测试 | 15 | 嵌套执行、回滚、原子性 |
| 性能测试 | 5 | 时延、吞吐量、内存 |
| 边界测试 | 10 | 深度超限、空 Batch、大 Batch |
| **总计** | **50** | **全场景覆盖** |

### 5.2 关键测试用例

```rust
#[cfg(test)]
mod nested_batch_tests {
    use super::*;
    
    /// 测试：单层嵌套 (Phase 3 新增)
    #[tokio::test]
    async fn test_single_level_nested_batch() {
        let executor = create_test_executor();
        
        let request = BatchExecuteRequest {
            trace_id: "trace_001".to_string(),
            batch_id: "batch_001".to_string(),
            instructions: vec![
                BatchInstruction::Single(create_test_instruction()),
                BatchInstruction::Nested(Box::new(BatchExecuteRequest {
                    trace_id: "trace_002".to_string(),
                    batch_id: "batch_002".to_string(),
                    instructions: vec![
                        BatchInstruction::Single(create_test_instruction()),
                        BatchInstruction::Single(create_test_instruction()),
                    ],
                    atomic: true,
                    isolation_level: BatchIsolationLevel::Sequential,
                    max_depth: 1,
                    timestamp: get_current_timestamp(),
                })),
            ],
            atomic: true,
            isolation_level: BatchIsolationLevel::Sequential,
            max_depth: 2,
            timestamp: get_current_timestamp(),
        };
        
        let result = executor.execute_nested_batch(request).await.unwrap();
        
        assert_eq!(result.status, BatchStatus::Success);
        assert_eq!(result.depth, 0);
        assert!(result.inner_hash.is_some());
    }
    
    /// 测试：嵌套深度超限 (Phase 3 新增)
    #[tokio::test]
    async fn test_max_depth_exceeded() {
        let executor = create_test_executor();
        
        // 创建 3 层嵌套 (超过最大深度 2)
        let request = create_triple_nested_batch();
        
        let result = executor.execute_nested_batch(request).await;
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BatchError::MaxDepthExceeded);
    }
    
    /// 测试：原子性保证 (嵌套回滚)
    #[tokio::test]
    async fn test_nested_batch_atomicity() {
        let executor = create_test_executor();
        
        let request = BatchExecuteRequest {
            trace_id: "trace_001".to_string(),
            batch_id: "batch_001".to_string(),
            instructions: vec![
                BatchInstruction::Single(create_success_instruction()),
                BatchInstruction::Nested(Box::new(BatchExecuteRequest {
                    trace_id: "trace_002".to_string(),
                    batch_id: "batch_002".to_string(),
                    instructions: vec![
                        BatchInstruction::Single(create_failure_instruction()),
                    ],
                    atomic: true,
                    isolation_level: BatchIsolationLevel::Sequential,
                    max_depth: 1,
                    timestamp: get_current_timestamp(),
                })),
            ],
            atomic: true,
            isolation_level: BatchIsolationLevel::Sequential,
            max_depth: 2,
            timestamp: get_current_timestamp(),
        };
        
        let result = executor.execute_nested_batch(request).await;
        
        // 原子性违反，全部回滚
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BatchError::AtomicityViolation);
    }
    
    /// 测试：并行执行内层 Batch (Phase 3 新增)
    #[tokio::test]
    async fn test_parallel_inner_execution() {
        let executor = create_test_executor();
        
        let request = BatchExecuteRequest {
            trace_id: "trace_001".to_string(),
            batch_id: "batch_001".to_string(),
            instructions: vec![
                BatchInstruction::Nested(Box::new(create_test_batch())),
                BatchInstruction::Nested(Box::new(create_test_batch())),
                BatchInstruction::Nested(Box::new(create_test_batch())),
            ],
            atomic: false,
            isolation_level: BatchIsolationLevel::ParallelInner,
            max_depth: 2,
            timestamp: get_current_timestamp(),
        };
        
        let start = Instant::now();
        let result = executor.execute_nested_batch(request).await.unwrap();
        let elapsed = start.elapsed();
        
        // 并行执行应该比串行快
        assert!(elapsed.as_millis() < 300);  // 假设串行需要 300ms+
        assert_eq!(result.status, BatchStatus::Success);
    }
}
```

---

## 6. 监控指标

### 6.1 Batch 嵌套相关指标 (8 个)

```rust
// Phase 3: Batch 嵌套监控指标
pub struct BatchNestedMetrics {
    /// 嵌套深度分布
    pub depth_histogram: Histogram,
    /// 内层 Batch 时延
    pub inner_latency: Histogram,
    /// 外层 Batch 时延
    pub outer_latency: Histogram,
    /// 嵌套开销百分比
    pub overhead_percent: Gauge,
    /// 并行执行比例
    pub parallel_ratio: Gauge,
    /// 嵌套冲突次数
    pub conflict_count: Counter,
    /// 嵌套重试次数
    pub retry_count: Counter,
    /// 嵌套成功率
    pub success_rate: Gauge,
}

impl BatchNestedMetrics {
    pub fn record_nested_batch_execution(
        &self,
        depth: u8,
        latency_ms: u64,
        status: &BatchStatus,
    ) {
        // 记录深度分布
        self.depth_histogram.observe(depth as f64);
        
        // 记录时延
        if depth == 0 {
            self.outer_latency.observe(latency_ms as f64);
        } else {
            self.inner_latency.observe(latency_ms as f64);
        }
        
        // 记录成功率
        if matches!(status, BatchStatus::Success) {
            self.success_rate.inc();
        }
    }
    
    pub fn record_overhead(&self, overhead_percent: f64) {
        self.overhead_percent.set(overhead_percent);
    }
    
    pub fn record_parallel_execution(&self, is_parallel: bool) {
        if is_parallel {
            self.parallel_ratio.inc();
        }
    }
    
    pub fn record_conflict(&self) {
        self.conflict_count.inc();
    }
    
    pub fn record_retry(&self) {
        self.retry_count.inc();
    }
}
```

---

## 7. 失败处理

### 7.1 错误类型定义

```rust
/// Phase 3: Batch 嵌套错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchError {
    /// 嵌套深度超限
    MaxDepthExceeded,
    /// 原子性违反
    AtomicityViolation,
    /// 内层 Batch 失败
    InnerBatchFailed {
        inner_batch_id: String,
        error: String,
    },
    /// 哈希验证失败
    HashVerificationFailed {
        expected: String,
        actual: String,
    },
    /// 回滚失败
    RollbackFailed {
        batch_id: String,
        error: String,
    },
    /// 超时
    Timeout {
        batch_id: String,
        timeout_ms: u64,
    },
}

impl std::fmt::Display for BatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchError::MaxDepthExceeded => write!(f, "嵌套深度超限 (最大 2 层)"),
            BatchError::AtomicityViolation => write!(f, "原子性违反"),
            BatchError::InnerBatchFailed { inner_batch_id, error } => {
                write!(f, "内层 Batch {} 失败：{}", inner_batch_id, error)
            }
            BatchError::HashVerificationFailed { expected, actual } => {
                write!(f, "哈希验证失败：期望 {}, 实际 {}", expected, actual)
            }
            BatchError::RollbackFailed { batch_id, error } => {
                write!(f, "Batch {} 回滚失败：{}", batch_id, error)
            }
            BatchError::Timeout { batch_id, timeout_ms } => {
                write!(f, "Batch {} 超时 ({}ms)", batch_id, timeout_ms)
            }
        }
    }
}
```

### 7.2 回滚策略

```rust
// Phase 3: 嵌套 Batch 回滚策略
impl NestedBatchExecutor {
    async fn rollback_nested_batch(&self, context: &BatchContext) -> Result<()> {
        // 1. 从最内层开始回滚 (深度优先)
        for child_context in context.child_contexts.iter().rev() {
            self.rollback_nested_batch(child_context).await?;
        }
        
        // 2. 回滚当前层
        if let Some(snapshot) = &context.state_snapshot {
            self.committer.rollback(snapshot).await.map_err(|e| {
                BatchError::RollbackFailed {
                    batch_id: context.batch_id.clone(),
                    error: e.to_string(),
                }
            })?;
        }
        
        // 3. 记录审计日志
        self.audit_log
            .log_batch_rollback(&context.batch_id, context.depth)
            .await;
        
        Ok(())
    }
}
```

---

## 8. 附录

### 8.1 使用示例

```rust
// Phase 3: 嵌套 Batch 使用示例
#[tokio::main]
async fn main() -> Result<()> {
    let executor = NestedBatchExecutor::new();
    
    // 创建外层 Batch (批量用户操作)
    let outer_batch = BatchExecuteRequest {
        trace_id: "trace_001".to_string(),
        batch_id: "batch_outer".to_string(),
        instructions: vec![
            // 用户 1 的批量操作 (嵌套 Batch)
            BatchInstruction::Nested(Box::new(BatchExecuteRequest {
                trace_id: "trace_002".to_string(),
                batch_id: "batch_user1".to_string(),
                instructions: vec![
                    BatchInstruction::Single(ExecuteRequest {
                        instruction: "UPDATE balance SET +100".to_string(),
                        ..Default::default()
                    }),
                    BatchInstruction::Single(ExecuteRequest {
                        instruction: "UPDATE points SET +50".to_string(),
                        ..Default::default()
                    }),
                ],
                atomic: true,
                isolation_level: BatchIsolationLevel::Sequential,
                max_depth: 1,
                timestamp: get_current_timestamp(),
            })),
            // 用户 2 的批量操作 (嵌套 Batch)
            BatchInstruction::Nested(Box::new(BatchExecuteRequest {
                trace_id: "trace_003".to_string(),
                batch_id: "batch_user2".to_string(),
                instructions: vec![
                    BatchInstruction::Single(ExecuteRequest {
                        instruction: "UPDATE balance SET +200".to_string(),
                        ..Default::default()
                    }),
                    BatchInstruction::Single(ExecuteRequest {
                        instruction: "UPDATE points SET +100".to_string(),
                        ..Default::default()
                    }),
                ],
                atomic: true,
                isolation_level: BatchIsolationLevel::Sequential,
                max_depth: 1,
                timestamp: get_current_timestamp(),
            })),
        ],
        atomic: false,  // 外层不要求原子性
        isolation_level: BatchIsolationLevel::ParallelInner,  // 并行执行内层
        max_depth: 2,
        timestamp: get_current_timestamp(),
    };
    
    // 执行嵌套 Batch
    let result = executor.execute_nested_batch(outer_batch).await?;
    
    println!("Batch 执行完成:");
    println!("  状态：{:?}", result.status);
    println!("  深度：{}", result.depth);
    println!("  外层哈希：{}", result.batch_hash);
    println!("  内层哈希：{:?}", result.inner_hash);
    
    Ok(())
}
```

### 8.2 性能优化检查清单

| 优化项 | 状态 | 验证方法 |
|---|---|---|
| 懒加载快照 | 📋 待实施 | 压测对比 |
| 增量哈希 | 📋 待实施 | 单元测试 |
| 对象池复用 | 📋 待实施 | 内存分析 |
| 并行执行 | 📋 待实施 | 压测对比 |

---

**文档状态**: 📋 草案  
**责任人**: Architect-Agent  
**保管**: 项目文档库
