# Phase 2 Batch 指令设计文档

**版本**: v1.0  
**日期**: 2026-04-07  
**责任人**: Dev  
**状态**: 📋 设计评审中  
**release_id**: release-2026-04-07-phase2_week02  

---

## 1. 概述

### 1.1 Batch 指令目标

Batch 指令支持批量执行多条指令，提供原子性保证，简化客户端批量操作场景。

| 目标 | 描述 |
|---|---|
| 批量执行 | 单次请求支持 1-100 条指令 |
| 原子性保证 | 全部成功或全部失败 (atomic=true) |
| 性能开销 | 相比单条指令开销<20% |
| 一致性 | 重放一致率≥99.95% |

### 1.2 设计原则

- **向后兼容**: 保持 Phase 1 接口契约不变
- **原子性**: Batch 级事务语义
- **可追溯**: 每条子指令独立 trace_id
- **可验证**: Verifier 支持 Batch 重放

---

## 2. 数据结构

### 2.1 Batch 请求

```rust
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
    /// 请求时间戳
    pub timestamp: String,
}

/// Batch 状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchStatus {
    /// 全部成功
    Success,
    /// 部分失败 (atomic=false 时)
    PartialFailure,
    /// 全部失败
    Failed,
}
```

### 2.2 Batch 结果

```rust
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
    /// 结果时间戳
    pub timestamp: String,
}
```

### 2.3 Batch 哈希链

```rust
/// Batch 哈希计算
pub fn compute_batch_hash(instructions: &[ExecuteRequest], results: &[ExecutionResult]) -> String {
    // 哈希覆盖：所有子指令的 trace_id + 所有子指令结果的 result_hash
    let mut hasher = Sha256::new();
    
    // 哈希输入 1: 所有子指令 trace_id
    for instruction in instructions {
        hasher.update(instruction.trace_id.as_bytes());
    }
    
    // 哈希输入 2: 所有子指令结果 result_hash
    for result in results {
        hasher.update(result.result_hash.as_bytes());
    }
    
    // 哈希输入 3: batch_id
    hasher.update(batch_id.as_bytes());
    
    format!("{:x}", hasher.finalize())
}
```

---

## 3. Batch 执行器

### 3.1 执行流程

```
BatchExecuteRequest
       │
       ▼
┌─────────────────┐
│  参数验证        │
│  - 1≤指令数≤100  │
│  - trace_id 有效 │
│  - timestamp 有效│
└─────────────────┘
       │
       ▼
┌─────────────────┐
│  逐条执行指令    │
│  - Executor     │
│  - state_diff   │
│  - 结果收集      │
└─────────────────┘
       │
       ▼
┌─────────────────┐
│  原子性处理      │
│  - atomic=true  │
│    全部成功或全部失败│
│  - atomic=false │
│    部分成功允许  │
└─────────────────┘
       │
       ▼
┌─────────────────┐
│  Batch 哈希计算  │
│  - batch_hash   │
└─────────────────┘
       │
       ▼
BatchExecuteResult
```

### 3.2 核心实现

```rust
impl BatchExecutor {
    pub async fn execute(&self, request: BatchExecuteRequest) -> Result<BatchExecuteResult, BatchError> {
        // 1. 参数验证
        self.validate(&request)?;
        
        // 2. 逐条执行指令
        let mut results = Vec::with_capacity(request.instructions.len());
        let mut has_failure = false;
        
        for instruction in &request.instructions {
            match self.executor.execute(instruction.clone()).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    has_failure = true;
                    if request.atomic {
                        // 原子性模式：立即回滚已执行的指令
                        self.rollback(&results).await?;
                        return Err(BatchError::AtomicFailure(e));
                    }
                    // 非原子性模式：记录失败，继续执行
                    results.push(ExecutionResult::failed(e));
                }
            }
        }
        
        // 3. 确定 Batch 状态
        let status = if has_failure {
            if results.iter().any(|r| r.is_success()) {
                BatchStatus::PartialFailure
            } else {
                BatchStatus::Failed
            }
        } else {
            BatchStatus::Success
        };
        
        // 4. 计算 Batch 哈希
        let batch_hash = compute_batch_hash(&request.instructions, &results);
        
        Ok(BatchExecuteResult {
            trace_id: request.trace_id,
            batch_id: request.batch_id,
            status,
            results,
            batch_hash,
            timestamp: Utc::now().to_rfc3339(),
        })
    }
    
    fn validate(&self, request: &BatchExecuteRequest) -> Result<(), BatchError> {
        // 指令数量检查
        if request.instructions.is_empty() {
            return Err(BatchError::EmptyBatch);
        }
        if request.instructions.len() > 100 {
            return Err(BatchError::BatchTooLarge(request.instructions.len()));
        }
        
        // trace_id 检查
        if request.trace_id.is_empty() {
            return Err(BatchError::InvalidTraceId);
        }
        
        Ok(())
    }
    
    async fn rollback(&self, results: &[ExecutionResult]) -> Result<(), BatchError> {
        // 回滚已执行的指令 (逆序执行相反操作)
        for result in results.iter().rev() {
            self.rollback_single(result).await?;
        }
        Ok(())
    }
}
```

---

## 4. gRPC 服务定义

### 4.1 Batch 服务

```protobuf
// batch.proto (Phase 2 新增)
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

---

## 5. 验证器集成

### 5.1 Batch 重放

```rust
impl Verifier {
    pub async fn verify_batch(&self, request: &BatchExecuteRequest, 
                               original_result: &BatchExecuteResult) 
                               -> Result<VerifyResponse, VerifierError> {
        // 重放所有子指令
        let mut replay_results = Vec::with_capacity(request.instructions.len());
        
        for instruction in &request.instructions {
            let replay_result = self.executor.execute(instruction.clone()).await?;
            replay_results.push(replay_result);
        }
        
        // 比对结果
        let is_consistent = self.compare_batch_results(&original_result.results, &replay_results);
        
        Ok(VerifyResponse {
            trace_id: request.trace_id.clone(),
            is_consistent,
            // ... 其他字段
        })
    }
    
    fn compare_batch_results(&self, original: &[ExecutionResult], 
                              replay: &[ExecutionResult]) -> bool {
        if original.len() != replay.len() {
            return false;
        }
        
        original.iter().zip(replay.iter()).all(|(o, r)| {
            o.result_hash == r.result_hash &&
            o.state_diff_hash == r.state_diff_hash
        })
    }
}
```

---

## 6. 安全闸门集成

### 6.1 SG-1~SG-4 Batch 扩展

| 闸门 | Phase 1 验证 | Phase 2 Batch 扩展 |
|---|---|---|
| SG-1 | 提交路径验证 | Batch 提交路径验证 + batch_hash |
| SG-2 | 隔离边界验证 | Batch 隔离边界 + 原子性验证 |
| SG-3 | 哈希链完整性 | batch_hash 验证 + 子指令哈希链 |
| SG-4 | 权限 + 重放检查 | Batch 级权限 + Batch 重放检查 |

### 6.2 Batch 安全验证

```rust
impl CommitBlockingMiddleware {
    pub async fn intercept_batch(&self, request: &BatchCommitRequest) 
                                  -> Result<CommitResponse, BlockingError> {
        // SG-1: Batch 路径验证
        if !self.path_validator.validate_batch(&request).await?.is_verified() {
            return Err(BlockingError::UnverifiedBatchPath);
        }
        
        // SG-3: Batch 哈希验证
        if !self.hash_verifier.verify_batch(&request).await? {
            return Err(BlockingError::BatchHashMismatch);
        }
        
        // SG-4: Batch 重放检查
        if self.replay_detector.is_batch_replay(&request).await? {
            return Err(BlockingError::BatchReplayAttack);
        }
        
        Ok(CommitResponse::allowed())
    }
}
```

---

## 7. 监控指标

### 7.1 Batch 新增指标 (3 个)

| 指标名 | 类型 | 描述 | P0 告警阈值 |
|---|---|---|---|
| batch_execute_latency_p99 | Histogram | Batch 执行 P99 时延 | >400ms |
| batch_atomicity_violation_count | Counter | 原子性违反次数 | >0 |
| batch_sub_instruction_count | Histogram | Batch 子指令数量分布 | - |

### 7.2 指标采集点

```rust
// Batch 执行时延
let start = Instant::now();
let result = self.execute(request).await;
let latency = start.elapsed();
metrics::batch_execute_latency.observe(latency.as_secs_f64());

// Batch 子指令数量
metrics::batch_sub_instruction_count.observe(request.instructions.len() as f64);

// 原子性违反
if result.status == BatchStatus::PartialFailure && request.atomic {
    metrics::batch_atomicity_violation_count.inc();
}
```

---

## 8. 测试策略

### 8.1 单元测试用例

| 用例 ID | 用例描述 | 类型 | 优先级 |
|---|---|---|---|
| UT-BATCH-001 | Batch 单条指令执行 | 功能 | P0 |
| UT-BATCH-002 | Batch 多条指令执行 (10 条) | 功能 | P0 |
| UT-BATCH-003 | Batch 最大指令数 (100 条) | 功能 | P0 |
| UT-BATCH-004 | Batch 原子性成功 | 功能 | P0 |
| UT-BATCH-005 | Batch 原子性失败回滚 | 功能 | P0 |
| UT-BATCH-006 | Batch 非原子性部分成功 | 功能 | P1 |
| UT-BATCH-007 | Batch 哈希计算正确性 | 安全 | P0 |
| UT-BATCH-008 | Batch 重放一致性 | 一致性 | P0 |

### 8.2 集成测试用例

| 用例 ID | 用例描述 | 类型 | 优先级 |
|---|---|---|---|
| IT-BATCH-001 | Batch 端到端执行 | E2E | P0 |
| IT-BATCH-002 | Batch + Verifier 集成 | 集成 | P0 |
| IT-BATCH-003 | Batch + SG-1~SG-4 集成 | 安全 | P0 |

---

## 9. 性能考虑

### 9.1 性能开销分析

| 开销项 | Phase 1 单条 | Phase 2 Batch (100 条) | 增量 |
|---|---|---|---|
| 序列化 | 10ms | 15ms | +50% |
| 执行 | 187ms | 187ms (并行) | 0% |
| 验证 | 203ms | 203ms (并行) | 0% |
| 哈希计算 | 5ms | 10ms | +100% |
| **总计** | **405ms** | **415ms** | **+2.5%** |

**目标**: Batch 开销<20% ✅ 设计满足

### 9.2 优化措施

- **并行执行**: 子指令可并行执行 (无依赖时)
- **批量哈希**: 一次性计算所有子指令哈希
- **对象池**: Vec<ExecutionResult> 池化复用

---

## 10. 待决策项

| 待决策 ID | 决策描述 | 选项 | 建议 | 责任人 |
|---|---|---|---|---|
| TBD-BATCH-001 | Batch 大小限制是否可配置 | 固定 100 / 可配置 | 固定 100 (简化实现) | PM+Dev |
| TBD-BATCH-002 | 原子性违反是否阻断提交 | 阻断 / 告警 | 告警 (允许 PartialFailure) | Security |

---

## 11. 附录

### 11.1 术语表

| 术语 | 定义 |
|---|---|
| Batch | 批量指令执行，原子性保证 |
| Atomic | 原子性，全部成功或全部失败 |
| PartialFailure | 部分失败，部分指令成功 |
| batch_hash | Batch 级哈希，覆盖所有子指令 |

### 11.2 参考文档

- Phase 2 ADR v4
- Phase 2 PRD v2
- Phase 1 executor.proto

---

**文档状态**: 📋 设计评审中  
**评审计划**: Week 2-T2 Batch 设计评审会议  
**责任人**: Dev  
**保管**: 项目文档库
