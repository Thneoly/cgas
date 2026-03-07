# Phase 2 Week 3 Transaction 指令设计文档

**版本**: v1.0  
**日期**: 2026-04-14  
**责任人**: Dev  
**状态**: 📋 设计评审中  
**release_id**: release-2026-04-14-phase2_week03  

---

## 1. 概述

### 1.1 Transaction 指令目标

Transaction 指令支持事务语义，提供 ACID 保证，实现 Read Committed 隔离级别。

| 目标 | 描述 |
|---|---|
| 事务控制 | BEGIN/COMMIT/ROLLBACK 完整生命周期 |
| 隔离级别 | Read Committed (RC) |
| 超时控制 | 可配置超时，自动回滚 |
| 可追溯性 | 完整事务链路追踪 |

### 1.2 设计范围

- Transaction 数据结构设计
- Transaction 执行器实现方案
- Transaction gRPC 服务定义
- Transaction 哈希链设计
- 与 Phase 1 架构集成

---

## 2. 数据结构设计

### 2.1 Transaction 状态

```rust
/// Transaction 状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    /// 事务已创建
    Created,
    /// 事务活跃中
    Active,
    /// 事务执行中
    Executing,
    /// 事务已提交
    Committed,
    /// 事务已回滚
    RolledBack,
}
```

**状态转换**:
```
Created → Active (BEGIN)
Active → Executing (EXECUTE)
Executing → Active (EXECUTE 完成)
Active → Committed (COMMIT)
Active → RolledBack (ROLLBACK 或超时)
Executing → RolledBack (超时)
```

### 2.2 Transaction 上下文

```rust
/// Transaction 上下文
pub struct TransactionContext {
    /// 事务 ID
    pub transaction_id: String,
    /// 事务 trace ID
    pub trace_id: String,
    /// 事务状态
    pub status: TransactionStatus,
    /// 隔离级别
    pub isolation_level: IsolationLevel,
    /// 超时时间 (毫秒)
    pub timeout_ms: i64,
    /// 创建时间
    pub created_at: i64,
    /// 最后活动时间
    pub last_activity_at: i64,
    /// 累积的 state_diff
    pub accumulated_diff: Vec<StateDiffOperation>,
    /// 执行的指令列表
    pub executed_instructions: Vec<ExecuteRequest>,
    /// 指令执行结果
    pub execution_results: Vec<ExecutionResult>,
}
```

### 2.3 隔离级别

```rust
/// 隔离级别 (Phase 2 仅支持 RC)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Read Committed (Phase 2)
    ReadCommitted,
    // RepeatableRead,  // Phase 3 扩展
    // Serializable,    // Phase 3 扩展
}
```

---

## 3. Transaction 执行器设计

### 3.1 执行流程

```
BEGIN_TRANSACTION
       │
       ▼
┌─────────────────┐
│  创建事务上下文  │
│  - 分配事务 ID   │
│  - 设置状态     │
│  - 记录创建时间 │
└─────────────────┘
       │
       ▼
┌─────────────────┐
│  执行事务内指令  │
│  - 累积 state_diff│
│  - 记录执行结果 │
│  - 检查超时     │
└─────────────────┘
       │
       ▼
    ┌──┴──┐
    │     │
COMMIT  ROLLBACK
    │     │
    ▼     ▼
┌─────┐ ┌──────┐
│提交 │ │回滚  │
│diff │ │diff  │
└─────┘ └──────┘
```

### 3.2 核心接口

```rust
/// Transaction 执行器
pub struct TransactionExecutor {
    /// 底层执行器
    executor: Executor,
    /// 事务上下文管理器
    context_manager: TransactionContextManager,
    /// 超时检查器
    timeout_checker: TimeoutChecker,
}

impl TransactionExecutor {
    /// 开始事务
    pub fn begin(&self, request: BeginTransactionRequest) 
                 -> Result<BeginTransactionResponse, TransactionError>;
    
    /// 执行事务内指令
    pub fn execute(&self, request: TransactionExecuteRequest) 
                   -> Result<TransactionExecuteResult, TransactionError>;
    
    /// 提交事务
    pub fn commit(&self, request: CommitTransactionRequest) 
                  -> Result<CommitTransactionResponse, TransactionError>;
    
    /// 回滚事务
    pub fn rollback(&self, request: RollbackTransactionRequest) 
                    -> Result<RollbackTransactionResponse, TransactionError>;
}
```

---

## 4. 超时机制设计

### 4.1 超时配置

| 配置项 | 默认值 | 可配置范围 |
|---|---|---|
| 事务超时 | 5000ms | 1000ms - 60000ms |
| 超时检查间隔 | 100ms | 50ms - 1000ms |

### 4.2 超时检查实现

```rust
/// 超时检查器
pub struct TimeoutChecker {
    /// 超时时间 (毫秒)
    timeout_ms: i64,
    /// 检查间隔 (毫秒)
    check_interval_ms: i64,
}

impl TimeoutChecker {
    /// 检查事务是否超时
    pub fn is_timeout(&self, context: &TransactionContext) -> bool {
        let now = current_timestamp_ms();
        let elapsed = now - context.created_at;
        elapsed > self.timeout_ms
    }
    
    /// 检查最后活动时间
    pub fn is_activity_timeout(&self, context: &TransactionContext) -> bool {
        let now = current_timestamp_ms();
        let elapsed = now - context.last_activity_at;
        elapsed > self.timeout_ms
    }
}
```

### 4.3 超时处理流程

```
事务执行中
       │
       ▼
┌─────────────────┐
│  定期检查超时    │
│  (每 100ms)      │
└─────────────────┘
       │
       ▼
    超时？
    ┌──┴──┐
   否     是
    │     │
    │     ▼
    │ ┌─────────────────┐
    │ │  自动回滚       │
    │ │  - 撤销 diff    │
    │ │  - 释放资源     │
    │ │  - 记录日志     │
    │ └─────────────────┘
    │     │
    │     ▼
    │ ┌─────────────────┐
    │ │  返回 Timeout    │
    │ │  错误           │
    │ └─────────────────┘
    │
    ▼
继续执行
```

---

## 5. 事务哈希链设计

### 5.1 transaction_hash 计算

```rust
/// 计算 Transaction 哈希
pub fn compute_transaction_hash(
    transaction_id: &str,
    isolation_level: &IsolationLevel,
    instructions: &[ExecuteRequest],
    results: &[ExecutionResult],
    accumulated_diff: &[StateDiffOperation],
) -> String {
    let mut hasher = Sha256::new();
    
    // 哈希输入 1: transaction_id
    hasher.update(transaction_id.as_bytes());
    hasher.update(b"\x00");
    
    // 哈希输入 2: isolation_level
    let isolation_str = match isolation_level {
        IsolationLevel::ReadCommitted => "RC",
    };
    hasher.update(isolation_str.as_bytes());
    hasher.update(b"\x00");
    
    // 哈希输入 3: 指令数量
    hasher.update((instructions.len() as u64).to_be_bytes());
    hasher.update(b"\x00");
    
    // 哈希输入 4: 所有子指令 trace_id
    for instruction in instructions {
        hasher.update(instruction.trace_id.as_bytes());
        hasher.update(b"\x00");
    }
    
    // 哈希输入 5: 所有子指令结果 result_hash
    for result in results {
        hasher.update(result.result_hash.as_bytes());
        hasher.update(b"\x00");
    }
    
    // 哈希输入 6: accumulated_diff_hash
    let diff_hash = compute_diff_hash(accumulated_diff);
    hasher.update(diff_hash.as_bytes());
    
    format!("{:x}", hasher.finalize())
}
```

### 5.2 哈希验证规则

| 验证项 | 规则 | 失败动作 |
|---|---|---|
| 哈希长度 | 64 字符 (SHA256 hex) | 拒绝提交 |
| 哈希一致性 | 重算 hash 与 transaction_hash 一致 | SG-3 阻断 |
| 状态一致性 | 事务状态=Committed | 拒绝查询 |

---

## 6. gRPC 服务定义

### 6.1 Transaction 服务

```protobuf
// transaction.proto (Phase 2 新增)
syntax = "proto3";

package cgas.transaction;

import "executor.proto";

// 隔离级别
enum IsolationLevel {
  ISOLATION_LEVEL_UNSPECIFIED = 0;
  ISOLATION_LEVEL_READ_COMMITTED = 1;
}

// 事务状态
enum TransactionStatus {
  TRANSACTION_STATUS_UNSPECIFIED = 0;
  TRANSACTION_STATUS_CREATED = 1;
  TRANSACTION_STATUS_ACTIVE = 2;
  TRANSACTION_STATUS_EXECUTING = 3;
  TRANSACTION_STATUS_COMMITTED = 4;
  TRANSACTION_STATUS_ROLLEDBACK = 5;
}

// 事务开始请求
message BeginTransactionRequest {
  string trace_id = 1;
  string transaction_id = 2;
  IsolationLevel isolation_level = 3;
  int64 timeout_ms = 4;
  string timestamp = 5;
}

// 事务开始响应
message BeginTransactionResponse {
  string trace_id = 1;
  string transaction_id = 2;
  TransactionStatus status = 3;
  string timestamp = 6;
}

// 事务执行请求
message TransactionExecuteRequest {
  string trace_id = 1;
  string transaction_id = 2;
  repeated ExecuteRequest instructions = 3;
  string timestamp = 4;
}

// 事务执行结果
message TransactionExecuteResult {
  string trace_id = 1;
  string transaction_id = 2;
  ExecutionStatus status = 3;
  repeated ExecutionResult results = 4;
  repeated StateDiffOperation accumulated_diff = 5;
  string timestamp = 6;
}

// 事务提交请求
message CommitTransactionRequest {
  string trace_id = 1;
  string transaction_id = 2;
  string transaction_hash = 3;
  string timestamp = 4;
}

// 事务提交响应
message CommitTransactionResponse {
  string trace_id = 1;
  string transaction_id = 2;
  CommitStatus status = 3;
  string timestamp = 4;
  string commit_hash = 5;
}

// 事务回滚请求
message RollbackTransactionRequest {
  string trace_id = 1;
  string transaction_id = 2;
  string reason = 3;
  string timestamp = 4;
}

// 事务回滚响应
message RollbackTransactionResponse {
  string trace_id = 1;
  string transaction_id = 2;
  RollbackStatus status = 3;
  string timestamp = 4;
}

// Transaction 服务
service TransactionService {
  rpc BeginTransaction(BeginTransactionRequest) returns (BeginTransactionResponse);
  rpc TransactionExecute(TransactionExecuteRequest) returns (TransactionExecuteResult);
  rpc CommitTransaction(CommitTransactionRequest) returns (CommitTransactionResponse);
  rpc RollbackTransaction(RollbackTransactionRequest) returns (RollbackTransactionResponse);
}
```

---

## 7. 与 Phase 1 架构集成

### 7.1 集成点

| 集成点 | Phase 1 组件 | Transaction 集成方式 |
|---|---|---|
| 执行器 | Executor | 复用底层执行器 |
| 验证器 | Verifier | 新增 Transaction 重放支持 |
| 阻断中间件 | CommitBlockingMiddleware | 新增 Transaction 验证 |
| 状态存储 | State Store | 支持事务性写入 |

### 7.2 集成架构

```
Phase 1 架构:
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │───▶│   Blocking  │───▶│  Verifier   │
│  (请求)     │    │  Middleware │    │  (验证器)   │
└─────────────┘    └─────────────┘    └─────────────┘

Transaction 集成:
┌─────────────┐    ┌─────────────┐
│   Client    │───▶│ Transaction │
│  (事务)     │    │  Executor   │
└─────────────┘    └─────────────┘
                          │
                          ▼
                   ┌─────────────┐
                   │  Executor   │
                   │  (Phase 1)  │
                   └─────────────┘
```

---

## 8. 测试策略

### 8.1 单元测试用例

| 用例 ID | 用例描述 | 类型 | 优先级 |
|---|---|---|---|
| UT-TRANS-001 | 事务开始 | 功能 | P0 |
| UT-TRANS-002 | 事务执行 (单指令) | 功能 | P0 |
| UT-TRANS-003 | 事务执行 (多指令) | 功能 | P0 |
| UT-TRANS-004 | 事务提交 | 功能 | P0 |
| UT-TRANS-005 | 事务回滚 | 功能 | P0 |
| UT-TRANS-006 | 事务超时自动回滚 | 超时 | P0 |
| UT-TRANS-007 | 事务哈希计算 | 安全 | P0 |
| UT-TRANS-008 | 事务重放一致性 | 一致性 | P0 |

### 8.2 集成测试用例

| 用例 ID | 用例描述 | 类型 | 优先级 |
|---|---|---|---|
| IT-TRANS-001 | Transaction 端到端执行 | E2E | P0 |
| IT-TRANS-002 | Transaction + Verifier 集成 | 集成 | P0 |
| IT-TRANS-003 | Transaction + SG-1~SG-4 集成 | 安全 | P0 |
| IT-TRANS-004 | 脏读检测 | 隔离级别 | P0 |

---

## 9. 待决策项

| 待决策 ID | 决策描述 | 选项 | 建议 | 责任人 |
|---|---|---|---|---|
| TBD-TRANS-001 | 事务超时默认值 | 5000ms / 10000ms | 5000ms | PM+Dev |
| TBD-TRANS-002 | 超时检查间隔 | 100ms / 500ms | 100ms | Dev |
| TBD-TRANS-003 | 事务并发数限制 | 无限制 / 100 | 100 | Dev+SRE |

---

## 10. 附录

### 10.1 术语表

| 术语 | 定义 |
|---|---|
| Transaction | 事务，ACID 语义保证 |
| ACID | 原子性、一致性、隔离性、持久性 |
| Read Committed | 读已提交隔离级别 |
| Dirty Read | 脏读，读取未提交数据 |
| Timeout | 超时，自动回滚机制 |

### 10.2 参考文档

- Phase 2 Transaction 规范 v1
- Phase 2 ADR v4
- Phase 2 Batch 设计文档
- Phase 1 执行器设计文档

---

**文档状态**: 📋 设计评审中  
**评审计划**: Week 3-T2 Transaction 设计评审会议  
**责任人**: Dev  
**保管**: 项目文档库
