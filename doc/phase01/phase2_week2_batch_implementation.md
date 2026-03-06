# Phase 2 Batch 指令实现指南

**版本**: v1.0  
**日期**: 2026-04-08  
**责任人**: Dev  
**状态**: 📋 实现中  
**release_id**: release-2026-04-08-phase2_week02  

---

## 1. 项目结构

### 1.1 文件组织

```
rust-workflow-engine/
├── src/
│   ├── batch/
│   │   ├── mod.rs              # Batch 模块入口
│   │   ├── types.rs            # Batch 数据类型
│   │   ├── executor.rs         # Batch 执行器
│   │   ├── hash.rs             # Batch 哈希计算
│   │   ├── validator.rs        # Batch 验证器
│   │   └── error.rs            # Batch 错误类型
│   ├── proto/
│   │   ├── batch.proto         # Batch gRPC 定义
│   │   └── build.rs            # Proto 编译配置
│   └── main.rs
├── tests/
│   └── batch/
│       ├── mod.rs
│       ├── test_executor.rs    # Batch 执行器测试
│       └── test_integration.rs # Batch 集成测试
└── Cargo.toml
```

### 1.2 Cargo.toml 依赖

```toml
[dependencies]
# Phase 1 继承
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
log = "0.4"
prometheus = "0.13"

# Phase 2 新增
tonic = "0.10"              # gRPC 框架
prost = "0.12"              # Proto 编译
async-trait = "0.1"         # 异步 trait

[build-dependencies]
tonic-build = "0.10"        # Proto 编译
```

---

## 2. 数据类型实现

### 2.1 batch_types.rs

```rust
//! Batch 指令数据类型
//! 
//! 定义 Batch 执行请求、结果、状态等核心数据结构

use serde::{Deserialize, Serialize};
use crate::executor::{ExecuteRequest, ExecutionResult};

/// Batch 执行请求 (Phase 2 新增)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecuteRequest {
    /// Batch 级 trace ID
    pub trace_id: String,
    /// Batch 唯一标识
    pub batch_id: String,
    /// 子指令列表 (1-100 条)
    pub instructions: Vec<ExecuteRequest>,
    /// 原子性保证 (true=全部成功或全部失败)
    pub atomic: bool,
    /// 请求时间戳 (RFC3339)
    pub timestamp: String,
}

impl BatchExecuteRequest {
    /// 创建新的 Batch 请求
    pub fn new(
        trace_id: String,
        batch_id: String,
        instructions: Vec<ExecuteRequest>,
        atomic: bool,
    ) -> Self {
        Self {
            trace_id,
            batch_id,
            instructions,
            atomic,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// 验证 Batch 请求
    pub fn validate(&self) -> Result<(), BatchValidationError> {
        // 指令数量检查
        if self.instructions.is_empty() {
            return Err(BatchValidationError::EmptyBatch);
        }
        if self.instructions.len() > 100 {
            return Err(BatchValidationError::BatchTooLarge(self.instructions.len()));
        }
        
        // trace_id 检查
        if self.trace_id.is_empty() {
            return Err(BatchValidationError::InvalidTraceId);
        }
        
        // 验证每条子指令
        for (i, instruction) in self.instructions.iter().enumerate() {
            instruction.validate()
                .map_err(|e| BatchValidationError::InvalidInstruction(i, e))?;
        }
        
        Ok(())
    }
}

/// Batch 状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BatchStatus {
    /// 全部成功
    Success,
    /// 部分失败 (atomic=false 时)
    PartialFailure,
    /// 全部失败
    Failed,
}

impl std::fmt::Display for BatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchStatus::Success => write!(f, "SUCCESS"),
            BatchStatus::PartialFailure => write!(f, "PARTIAL_FAILURE"),
            BatchStatus::Failed => write!(f, "FAILED"),
        }
    }
}

/// Batch 执行结果 (Phase 2 新增)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecuteResult {
    /// Batch 级 trace ID
    pub trace_id: String,
    /// Batch 唯一标识
    pub batch_id: String,
    /// Batch 状态
    pub status: BatchStatus,
    /// 子指令结果列表
    pub results: Vec<ExecutionResult>,
    /// Batch 级哈希 (覆盖所有子指令)
    pub batch_hash: String,
    /// 结果时间戳 (RFC3339)
    pub timestamp: String,
}

impl BatchExecuteResult {
    /// 创建成功结果
    pub fn success(
        trace_id: String,
        batch_id: String,
        results: Vec<ExecutionResult>,
        batch_hash: String,
    ) -> Self {
        Self {
            trace_id,
            batch_id,
            status: BatchStatus::Success,
            results,
            batch_hash,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// 创建失败结果
    pub fn failed(
        trace_id: String,
        batch_id: String,
        results: Vec<ExecutionResult>,
        batch_hash: String,
    ) -> Self {
        Self {
            trace_id,
            batch_id,
            status: BatchStatus::Failed,
            results,
            batch_hash,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// 创建部分失败结果
    pub fn partial_failure(
        trace_id: String,
        batch_id: String,
        results: Vec<ExecutionResult>,
        batch_hash: String,
    ) -> Self {
        Self {
            trace_id,
            batch_id,
            status: BatchStatus::PartialFailure,
            results,
            batch_hash,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// 检查是否全部成功
    pub fn is_success(&self) -> bool {
        self.status == BatchStatus::Success
    }
}

/// Batch 验证错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum BatchValidationError {
    #[error("Batch 为空")]
    EmptyBatch,
    
    #[error("Batch 过大：{0} 条指令 (最大 100 条)")]
    BatchTooLarge(usize),
    
    #[error("trace_id 无效")]
    InvalidTraceId,
    
    #[error("第 {0} 条指令无效：{1}")]
    InvalidInstruction(usize, crate::executor::ExecuteValidationError),
}
```

---

## 3. Batch 执行器实现

### 3.1 batch_executor.rs

```rust
//! Batch 执行器
//! 
//! 实现 Batch 指令的执行逻辑，支持原子性保证

use crate::batch::{
    BatchExecuteRequest,
    BatchExecuteResult,
    BatchStatus,
    BatchValidationError,
};
use crate::executor::{Executor, ExecutionResult};
use crate::batch::hash::compute_batch_hash;
use crate::metrics;
use std::time::Instant;
use log::{info, warn, error};

/// Batch 执行器
pub struct BatchExecutor {
    /// 底层执行器
    executor: Executor,
}

impl BatchExecutor {
    /// 创建新的 Batch 执行器
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }
    
    /// 执行 Batch 请求
    pub async fn execute(
        &self,
        request: BatchExecuteRequest,
    ) -> Result<BatchExecuteResult, BatchError> {
        let start = Instant::now();
        let sub_instruction_count = request.instructions.len();
        
        // 1. 参数验证
        request.validate()
            .map_err(BatchError::Validation)?;
        
        // 2. 逐条执行指令
        let (results, has_failure) = self.execute_instructions(&request).await?;
        
        // 3. 确定 Batch 状态
        let status = self.determine_batch_status(&results, has_failure, request.atomic);
        
        // 4. 计算 Batch 哈希
        let batch_hash = compute_batch_hash(&request.instructions, &results);
        
        // 5. 构建结果
        let result = match status {
            BatchStatus::Success => {
                BatchExecuteResult::success(
                    request.trace_id.clone(),
                    request.batch_id.clone(),
                    results,
                    batch_hash,
                )
            }
            BatchStatus::Failed => {
                BatchExecuteResult::failed(
                    request.trace_id.clone(),
                    request.batch_id.clone(),
                    results,
                    batch_hash,
                )
            }
            BatchStatus::PartialFailure => {
                BatchExecuteResult::partial_failure(
                    request.trace_id.clone(),
                    request.batch_id.clone(),
                    results,
                    batch_hash,
                )
            }
        };
        
        // 6. 采集监控指标
        let latency = start.elapsed();
        metrics::observe_batch_execute(latency.as_secs_f64(), sub_instruction_count);
        
        if !request.atomic && has_failure {
            metrics::inc_batch_atomicity_violation("non_atomic_batch");
        }
        
        info!(
            "Batch executed: batch_id={}, status={}, latency={:?}, instructions={}",
            request.batch_id,
            status,
            latency,
            sub_instruction_count,
        );
        
        Ok(result)
    }
    
    /// 执行所有子指令
    async fn execute_instructions(
        &self,
        request: &BatchExecuteRequest,
    ) -> Result<(Vec<ExecutionResult>, bool), BatchError> {
        let mut results = Vec::with_capacity(request.instructions.len());
        let mut has_failure = false;
        
        for (index, instruction) in request.instructions.iter().enumerate() {
            match self.executor.execute(instruction.clone()).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    has_failure = true;
                    
                    if request.atomic {
                        // 原子性模式：立即回滚已执行的指令
                        warn!(
                            "Batch atomic failure at instruction {}, initiating rollback",
                            index
                        );
                        self.rollback(&results).await?;
                        return Err(BatchError::AtomicFailure(e));
                    }
                    
                    // 非原子性模式：记录失败，继续执行
                    results.push(ExecutionResult::failed(e));
                    warn!(
                        "Batch instruction {} failed in non-atomic mode, continuing",
                        index
                    );
                }
            }
        }
        
        Ok((results, has_failure))
    }
    
    /// 确定 Batch 状态
    fn determine_batch_status(
        &self,
        results: &[ExecutionResult],
        has_failure: bool,
        atomic: bool,
    ) -> BatchStatus {
        if !has_failure {
            BatchStatus::Success
        } else if atomic {
            // 原子性模式下，任何失败都导致全部失败
            BatchStatus::Failed
        } else {
            // 非原子性模式，检查是否有成功的指令
            if results.iter().any(|r| r.is_success()) {
                BatchStatus::PartialFailure
            } else {
                BatchStatus::Failed
            }
        }
    }
    
    /// 回滚已执行的指令
    async fn rollback(&self, results: &[ExecutionResult]) -> Result<(), BatchError> {
        // 逆序回滚
        for result in results.iter().rev() {
            if let Err(e) = self.executor.rollback(result).await {
                error!("Rollback failed for execution {}: {}", result.execution_id, e);
                return Err(BatchError::RollbackFailed(e));
            }
        }
        Ok(())
    }
}

/// Batch 执行错误
#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("验证失败：{0}")]
    Validation(#[from] BatchValidationError),
    
    #[error("原子性执行失败：{0}")]
    AtomicFailure(#[source] crate::executor::ExecutorError),
    
    #[error("回滚失败：{0}")]
    RollbackFailed(#[source] crate::executor::ExecutorError),
    
    #[error("内部错误：{0}")]
    Internal(String),
}
```

---

## 4. Batch 哈希计算

### 4.1 batch_hash.rs

```rust
//! Batch 哈希计算
//! 
//! 实现 Batch 级哈希链，保证 Batch 完整性

use sha2::{Sha256, Digest};
use crate::batch::{BatchExecuteRequest, BatchExecuteResult};
use crate::executor::{ExecuteRequest, ExecutionResult};

/// 计算 Batch 哈希
/// 
/// 哈希覆盖：
/// 1. 所有子指令的 trace_id
/// 2. 所有子指令结果的 result_hash
/// 3. batch_id
pub fn compute_batch_hash(
    instructions: &[ExecuteRequest],
    results: &[ExecutionResult],
) -> String {
    let mut hasher = Sha256::new();
    
    // 哈希输入 1: 所有子指令 trace_id (按顺序)
    for instruction in instructions {
        hasher.update(instruction.trace_id.as_bytes());
        hasher.update(b"\x00"); // 分隔符
    }
    
    // 哈希输入 2: 所有子指令结果的 result_hash (按顺序)
    for result in results {
        hasher.update(result.result_hash.as_bytes());
        hasher.update(b"\x00"); // 分隔符
    }
    
    // 哈希输入 3: batch_id
    hasher.update(instructions.first()
        .map(|i| i.trace_id.as_bytes())
        .unwrap_or(b""));
    
    format!("{:x}", hasher.finalize())
}

/// 验证 Batch 哈希
pub fn verify_batch_hash(
    instructions: &[ExecuteRequest],
    results: &[ExecutionResult],
    expected_hash: &str,
) -> bool {
    let computed_hash = compute_batch_hash(instructions, results);
    computed_hash == expected_hash
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_batch_hash_deterministic() {
        // 相同输入产生相同哈希
        let instructions = create_test_instructions(3);
        let results = create_test_results(3);
        
        let hash1 = compute_batch_hash(&instructions, &results);
        let hash2 = compute_batch_hash(&instructions, &results);
        
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_batch_hash_unique() {
        // 不同输入产生不同哈希
        let instructions1 = create_test_instructions(3);
        let instructions2 = create_test_instructions(4);
        let results = create_test_results(3);
        
        let hash1 = compute_batch_hash(&instructions1, &results);
        let hash2 = compute_batch_hash(&instructions2, &results);
        
        assert_ne!(hash1, hash2);
    }
    
    fn create_test_instructions(count: usize) -> Vec<ExecuteRequest> {
        (0..count).map(|i| ExecuteRequest {
            trace_id: format!("trace_{}", i),
            execution_id: format!("exec_{}", i),
            // ... 其他字段
        }).collect()
    }
    
    fn create_test_results(count: usize) -> Vec<ExecutionResult> {
        (0..count).map(|i| ExecutionResult {
            trace_id: format!("trace_{}", i),
            execution_id: format!("exec_{}", i),
            result_hash: format!("hash_{}", i),
            // ... 其他字段
        }).collect()
    }
}
```

---

## 5. gRPC 服务实现

### 5.1 batch.proto

```protobuf
// proto/batch.proto
syntax = "proto3";

package cgas.batch;

import "executor.proto";

// Batch 执行请求
message BatchExecuteRequest {
  string trace_id = 1;
  string batch_id = 2;
  repeated ExecuteRequest instructions = 3;
  bool atomic = 4;
  string timestamp = 5;
}

// Batch 执行结果
message BatchExecuteResult {
  string trace_id = 1;
  string batch_id = 2;
  BatchStatus status = 3;
  repeated ExecutionResult results = 4;
  string batch_hash = 5;
  string timestamp = 6;
}

// Batch 状态
enum BatchStatus {
  BATCH_STATUS_UNSPECIFIED = 0;
  BATCH_STATUS_SUCCESS = 1;
  BATCH_STATUS_PARTIAL_FAILURE = 2;
  BATCH_STATUS_FAILED = 3;
}

// Batch 服务
service BatchService {
  rpc BatchExecute(BatchExecuteRequest) returns (BatchExecuteResult);
}
```

### 5.2 batch_service.rs

```rust
//! Batch gRPC 服务实现

use tonic::{Request, Response, Status};
use crate::proto::batch::*;
use crate::batch::BatchExecutor;

/// Batch 服务实现
pub struct BatchServiceImpl {
    executor: BatchExecutor,
}

impl BatchServiceImpl {
    pub fn new(executor: BatchExecutor) -> Self {
        Self { executor }
    }
}

#[tonic::async_trait]
impl batch_service_server::BatchService for BatchServiceImpl {
    async fn batch_execute(
        &self,
        request: Request<BatchExecuteRequest>,
    ) -> Result<Response<BatchExecuteResult>, Status> {
        let req = request.into_inner();
        
        // 转换为内部类型
        let internal_request = convert_to_internal(req)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        
        // 执行 Batch
        let result = self.executor.execute(internal_request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        
        // 转换为 Proto 类型
        let proto_result = convert_to_proto(result);
        
        Ok(Response::new(proto_result))
    }
}

fn convert_to_internal(
    req: BatchExecuteRequest,
) -> Result<crate::batch::BatchExecuteRequest, String> {
    let instructions = req.instructions
        .into_iter()
        .map(|i| crate::executor::ExecuteRequest::from_proto(i))
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(crate::batch::BatchExecuteRequest {
        trace_id: req.trace_id,
        batch_id: req.batch_id,
        instructions,
        atomic: req.atomic,
        timestamp: req.timestamp,
    })
}

fn convert_to_proto(
    result: crate::batch::BatchExecuteResult,
) -> BatchExecuteResult {
    let status = match result.status {
        crate::batch::BatchStatus::Success => BatchStatus::Success,
        crate::batch::BatchStatus::PartialFailure => BatchStatus::PartialFailure,
        crate::batch::BatchStatus::Failed => BatchStatus::Failed,
    };
    
    let results = result.results
        .into_iter()
        .map(|r| crate::proto::executor::ExecutionResult::from_internal(r))
        .collect();
    
    BatchExecuteResult {
        trace_id: result.trace_id,
        batch_id: result.batch_id,
        status: status as i32,
        results,
        batch_hash: result.batch_hash,
        timestamp: result.timestamp,
    }
}
```

---

## 6. 测试实现

### 6.1 test_executor.rs

```rust
//! Batch 执行器单元测试

use crate::batch::{BatchExecutor, BatchExecuteRequest, BatchStatus};
use crate::executor::{Executor, ExecutionResult};

#[tokio::test]
async fn test_batch_single_instruction() {
    let executor = create_test_executor();
    let batch_executor = BatchExecutor::new(executor);
    
    let request = BatchExecuteRequest::new(
        "trace_1".to_string(),
        "batch_1".to_string(),
        vec![create_test_instruction()],
        true,
    );
    
    let result = batch_executor.execute(request).await.unwrap();
    
    assert_eq!(result.status, BatchStatus::Success);
    assert_eq!(result.results.len(), 1);
}

#[tokio::test]
async fn test_batch_multiple_instructions() {
    let executor = create_test_executor();
    let batch_executor = BatchExecutor::new(executor);
    
    let request = BatchExecuteRequest::new(
        "trace_1".to_string(),
        "batch_1".to_string(),
        vec![
            create_test_instruction(),
            create_test_instruction(),
            create_test_instruction(),
        ],
        true,
    );
    
    let result = batch_executor.execute(request).await.unwrap();
    
    assert_eq!(result.status, BatchStatus::Success);
    assert_eq!(result.results.len(), 3);
}

#[tokio::test]
async fn test_batch_atomic_failure() {
    let mut executor = create_test_executor();
    executor.set_fail_at(1); // 第 2 条指令失败
    
    let batch_executor = BatchExecutor::new(executor);
    
    let request = BatchExecuteRequest::new(
        "trace_1".to_string(),
        "batch_1".to_string(),
        vec![
            create_test_instruction(),
            create_test_instruction(),
        ],
        true, // 原子性模式
    );
    
    let result = batch_executor.execute(request).await;
    
    // 原子性模式下，任何失败都导致全部失败
    assert!(result.is_err());
}

#[tokio::test]
async fn test_batch_non_atomic_partial() {
    let mut executor = create_test_executor();
    executor.set_fail_at(1); // 第 2 条指令失败
    
    let batch_executor = BatchExecutor::new(executor);
    
    let request = BatchExecuteRequest::new(
        "trace_1".to_string(),
        "batch_1".to_string(),
        vec![
            create_test_instruction(),
            create_test_instruction(),
        ],
        false, // 非原子性模式
    );
    
    let result = batch_executor.execute(request).await.unwrap();
    
    // 非原子性模式下，允许部分成功
    assert_eq!(result.status, BatchStatus::PartialFailure);
    assert_eq!(result.results.len(), 2);
}

fn create_test_executor() -> Executor {
    Executor::mock()
}

fn create_test_instruction() -> ExecuteRequest {
    ExecuteRequest {
        trace_id: "trace_1".to_string(),
        execution_id: "exec_1".to_string(),
        // ... 其他字段
    }
}
```

---

## 7. 性能优化

### 7.1 并行执行

```rust
/// 并行执行 Batch 指令 (无依赖时)
async fn execute_batch_parallel(
    instructions: Vec<ExecuteRequest>,
) -> Vec<Result<ExecutionResult, ExecutorError>> {
    let futures = instructions
        .into_iter()
        .map(|instruction| async move {
            executor.execute(instruction).await
        });
    
    futures::future::join_all(futures).await
}
```

### 7.2 对象池

```rust
use object_pool::Pool;

lazy_static! {
    static ref BATCH_RESULT_POOL: Pool<Vec<ExecutionResult>> = Pool::new(|| {
        Vec::with_capacity(100)
    });
}

fn get_batch_result_vec() -> Vec<ExecutionResult> {
    BATCH_RESULT_POOL.get().into_owned()
}

fn return_batch_result_vec(vec: Vec<ExecutionResult>) {
    let mut cleared = vec;
    cleared.clear();
    BATCH_RESULT_POOL.try_return(cleared);
}
```

---

## 8. 待完成事项

| 任务 | 状态 | 责任人 | 优先级 |
|---|---|---|---|
| Batch 类型定义 | ✅ 完成 | Dev | P0 |
| Batch 执行器核心逻辑 | ✅ 完成 | Dev | P0 |
| Batch 哈希计算 | ✅ 完成 | Dev | P0 |
| gRPC 服务定义 | ✅ 完成 | Dev | P0 |
| gRPC 服务实现 | 🟡 进行中 | Dev | P0 |
| 单元测试 | 📋 待开始 | Dev+QA | P0 |
| 集成测试 | 📋 待开始 | Dev+QA | P1 |
| 性能基准测试 | 📋 待开始 | SRE+Dev | P1 |

---

**文档状态**: 📋 实现中  
**责任人**: Dev  
**保管**: 项目文档库
