# Transaction 指令规范 v1.0（Phase 2）

**版本**: v1.0  
**日期**: 2026-04-08  
**责任人**: PM + Dev  
**状态**: 📋 草案评审中  
**release_id**: release-2026-04-08-phase2_week02  
**参与角色**: PM, Dev, QA, SRE, Security

---

## 1. 目标

定义 Phase 2 Transaction 事务指令的语义规范，确保：
- 事务语义正确性 (ACID)
- 隔离级别可验证
- 超时自动回滚
- 执行结果可重放

---

## 2. 设计原则

| 原则 | 描述 | 验证方法 |
|---|---|---|
| **ACID 语义** | 原子性、一致性、隔离性、持久性 | 事务测试 |
| **隔离级别** | Read Committed (RC) | 隔离违反检测 |
| **超时控制** | 可配置超时，自动回滚 | 超时测试 |
| **可追溯** | 完整事务链路追踪 | 审计日志 |
| **可验证** | Verifier 支持事务重放 | SG-1~SG-4 验证 |

---

## 3. Transaction 指令定义

### 3.1 指令分类

| 类别 | 指令名 | 说明 |
|---|---|---|
| **事务控制类** | `BEGIN_TRANSACTION` | 开始事务 |
| **事务控制类** | `COMMIT_TRANSACTION` | 提交事务 |
| **事务控制类** | `ROLLBACK_TRANSACTION` | 回滚事务 |
| **事务执行类** | `TRANSACTION_EXECUTE` | 执行事务内指令 |
| **子指令** | 继承 Phase 1 最小指令集 | READ/COMPUTE/WRITE/CONTROL |

### 3.2 指令语义

**BEGIN_TRANSACTION**:
- **功能**: 开始一个新事务
- **输入**: BeginTransactionRequest
- **输出**: BeginTransactionResponse
- **副作用**: 创建事务上下文

**COMMIT_TRANSACTION**:
- **功能**: 提交事务
- **输入**: CommitTransactionRequest
- **输出**: CommitTransactionResponse
- **副作用**: 提交所有 state_diff

**ROLLBACK_TRANSACTION**:
- **功能**: 回滚事务
- **输入**: RollbackTransactionRequest
- **输出**: RollbackTransactionResponse
- **副作用**: 撤销所有 state_diff

**TRANSACTION_EXECUTE**:
- **功能**: 在事务上下文中执行指令
- **输入**: TransactionExecuteRequest
- **输出**: TransactionExecuteResult
- **副作用**: 累积 state_diff (未提交)

---

## 4. 输入输出契约

### 4.1 事务开始契约

```rust
/// 事务开始请求
pub struct BeginTransactionRequest {
    /// 事务 trace ID (格式：UUID v4)
    pub trace_id: String,
    
    /// 事务唯一标识 (格式：UUID v4)
    pub transaction_id: String,
    
    /// 隔离级别 (Phase 2 仅支持 RC)
    pub isolation_level: IsolationLevel,
    
    /// 超时时间 (毫秒，默认 5000ms)
    pub timeout_ms: i64,
    
    /// 请求时间戳 (RFC3339 格式)
    pub timestamp: String,
}

/// 隔离级别
pub enum IsolationLevel {
    ReadCommitted,  // Phase 2 支持
    // RepeatableRead,  // Phase 3 扩展
    // Serializable,    // Phase 3 扩展
}

/// 事务开始响应
pub struct BeginTransactionResponse {
    /// 事务 trace ID
    pub trace_id: String,
    
    /// 事务唯一标识
    pub transaction_id: String,
    
    /// 事务状态
    pub status: TransactionStatus,
    
    /// 响应时间戳
    pub timestamp: String,
}
```

### 4.2 事务执行契约

```rust
/// 事务执行请求
pub struct TransactionExecuteRequest {
    /// 事务 trace ID
    pub trace_id: String,
    
    /// 事务唯一标识
    pub transaction_id: String,
    
    /// 要执行的指令列表
    pub instructions: Vec<ExecuteRequest>,
    
    /// 请求时间戳
    pub timestamp: String,
}

/// 事务执行结果
pub struct TransactionExecuteResult {
    /// 事务 trace ID
    pub trace_id: String,
    
    /// 事务唯一标识
    pub transaction_id: String,
    
    /// 执行状态
    pub status: ExecutionStatus,
    
    /// 指令执行结果
    pub results: Vec<ExecutionResult>,
    
    /// 累积的 state_diff (未提交)
    pub accumulated_diff: Vec<StateDiffOperation>,
    
    /// 响应时间戳
    pub timestamp: String,
}
```

### 4.3 事务提交契约

```rust
/// 事务提交请求
pub struct CommitTransactionRequest {
    /// 事务 trace ID
    pub trace_id: String,
    
    /// 事务唯一标识
    pub transaction_id: String,
    
    /// 事务哈希 (覆盖所有指令)
    pub transaction_hash: String,
    
    /// 请求时间戳
    pub timestamp: String,
}

/// 事务提交响应
pub struct CommitTransactionResponse {
    /// 事务 trace ID
    pub trace_id: String,
    
    /// 事务唯一标识
    pub transaction_id: String,
    
    /// 提交状态
    pub status: CommitStatus,
    
    /// 提交时间戳
    pub timestamp: String,
    
    /// 提交确认哈希
    pub commit_hash: String,
}
```

### 4.4 事务回滚契约

```rust
/// 事务回滚请求
pub struct RollbackTransactionRequest {
    /// 事务 trace ID
    pub trace_id: String,
    
    /// 事务唯一标识
    pub transaction_id: String,
    
    /// 回滚原因
    pub reason: String,
    
    /// 请求时间戳
    pub timestamp: String,
}

/// 事务回滚响应
pub struct RollbackTransactionResponse {
    /// 事务 trace ID
    pub trace_id: String,
    
    /// 事务唯一标识
    pub transaction_id: String,
    
    /// 回滚状态
    pub status: RollbackStatus,
    
    /// 回滚时间戳
    pub timestamp: String,
}
```

### 4.5 事务状态机

```
                    ┌─────────────┐
                    │   Created   │
                    └──────┬──────┘
                           │ BEGIN
                           ▼
                    ┌─────────────┐
              ┌────▶│   Active    │◀─────┐
              │     └──────┬──────┘      │
              │            │ EXECUTE     │
              │            ▼             │
              │     ┌─────────────┐      │
              │     │  Executing  │──────┘
              │     └──────┬──────┘
              │            │
         ROLLBACK          │ COMMIT
              │            │
              ▼            ▼
       ┌─────────────┐ ┌─────────────┐
       │  RolledBack │ │  Committed  │
       └─────────────┘ └─────────────┘
```

**状态转换规则**:

| 当前状态 | 动作 | 下一状态 | 条件 |
|---|---|---|---|
| Created | BEGIN | Active | 参数验证通过 |
| Active | EXECUTE | Executing | 事务未超时 |
| Executing | EXECUTE 完成 | Active | 指令执行完成 |
| Active | COMMIT | Committed | 所有指令成功 |
| Active | ROLLBACK | RolledBack | 任意原因 |
| Active | 超时 | RolledBack | 自动回滚 |
| Executing | 超时 | RolledBack | 自动回滚 |

---

## 5. 事务哈希链规范

### 5.1 哈希计算算法

```
transaction_hash = SHA256(
    transaction_id || "\x00" ||
    isolation_level || "\x00" ||
    instruction_count || "\x00" ||
    trace_id_1 || "\x00" ||
    trace_id_2 || "\x00" ||
    ... ||
    trace_id_n || "\x00" ||
    result_hash_1 || "\x00" ||
    result_hash_2 || "\x00" ||
    ... ||
    result_hash_n || "\x00" ||
    accumulated_diff_hash
)
```

**哈希覆盖**:
1. transaction_id
2. isolation_level
3. 指令数量
4. 所有子指令的 trace_id (按顺序)
5. 所有子指令结果的 result_hash (按顺序)
6. 累积 state_diff 的哈希

### 5.2 哈希验证规则

| 验证项 | 规则 | 失败动作 |
|---|---|---|
| 哈希长度 | 64 字符 (SHA256 hex) | 拒绝提交 |
| 哈希一致性 | 重算哈希与 transaction_hash 一致 | SG-3 阻断 |
| 状态一致性 | 事务状态=Committed | 拒绝查询 |

---

## 6. 隔离级别语义 (Read Committed)

### 6.1 RC 隔离保证

**读已提交**:
- 事务只能读取已提交的数据
- 不可重复读：允许 (同一事务内多次读取可能不同)
- 脏读：禁止
- 幻读：允许

### 6.2 隔离违反检测

**脏读检测**:
- 读取未提交数据 → 隔离违反
- 动作：拒绝读取，告警 (P1)

**写偏斜检测** (Phase 3 扩展):
- 并发事务写同一数据 → 检测写偏斜
- 动作：记录审计日志

---

## 7. 超时语义

### 7.1 超时配置

| 配置项 | 默认值 | 可配置范围 |
|---|---|---|
| 事务超时 | 5000ms | 1000ms - 60000ms |
| 超时检查间隔 | 100ms | 50ms - 1000ms |

### 7.2 超时处理

**超时触发条件**:
- 事务执行时间 > timeout_ms

**超时动作**:
1. 中断当前执行
2. 自动回滚事务
3. 记录审计日志
4. 返回 Timeout 错误

**超时保证**:
- 超时检查误差不超过 ±100ms
- 超时回滚必须成功

---

## 8. 错误与 REVERT 语义

### 8.1 错误分类

| 错误类型 | 错误码 | 触发条件 | 处理动作 |
|---|---|---|---|
| TransactionNotFound | 5001 | 事务 ID 不存在 | 返回错误 |
| InvalidIsolationLevel | 5002 | 不支持的隔离级别 | 拒绝开始 |
| TransactionTimeout | 5003 | 事务超时 | 自动回滚 |
| TransactionAlreadyCommitted | 5004 | 重复提交 | 返回错误 |
| TransactionAlreadyRolledBack | 5005 | 重复回滚 | 返回错误 |
| IsolationViolation | 5006 | 隔离级别违反 | 拒绝操作 |
| HashMismatch | 5007 | transaction_hash 验证失败 | SG-3 阻断 |

### 8.2 REVERT 语义

**REVERT 触发条件**:
- 事务超时
- 事务回滚请求
- transaction_hash 验证失败

**REVERT 动作**:
1. 撤销所有累积的 state_diff
2. 释放事务资源
3. 记录审计日志
4. 返回 RolledBack 状态

**REVERT 保证**:
- REVERT 操作本身必须可重放
- REVERT 不影响其他事务

---

## 9. 安全闸门集成

### 9.1 SG-1: Transaction 路径验证

**验证规则**:
- Transaction 提交必须经过 Transaction Verifier
- transaction_hash 必须存在

**阻断动作**:
- 未验证 Transaction → 拒绝提交
- 缺失 transaction_hash → 拒绝提交

### 9.2 SG-2: Transaction 隔离验证

**验证规则**:
- Transaction 执行器与验证器隔离
- 隔离级别正确实施

**阻断动作**:
- 隔离失效 → 拒绝执行
- 隔离违反 → 拒绝操作

### 9.3 SG-3: Transaction 哈希验证

**验证规则**:
- 重新计算 transaction_hash 与提交 hash 一致
- 每个子指令 result_hash 有效

**阻断动作**:
- 哈希不一致 → 拒绝提交
- 子指令 hash 无效 → 拒绝提交

### 9.4 SG-4: Transaction 权限 + 重放检查

**验证规则**:
- 用户有 Transaction 提交权限
- transaction_id 未重复使用
- transaction_hash 未重复使用

**阻断动作**:
- 权限不足 → 拒绝提交
- 重放检测 → 拒绝提交，告警

---

## 10. 性能规范

### 10.1 性能指标

| 指标 | 目标值 | 测量方法 |
|---|---|---|
| P99 事务提交时延 | <400ms | 压测 |
| P99 事务回滚时延 | <200ms | 压测 |
| 事务超时准确率 | ≥99% | 超时测试 |
| 并发事务数 | ≥100 | 并发测试 |

### 10.2 性能优化要求

| 优化项 | 要求 | 验证方法 |
|---|---|---|
| 事务上下文池 | 事务上下文对象池化 | 代码审查 |
| 批量提交 | 多事务批量提交支持 | 代码审查 |
| 超时检查优化 | 定时器轮询优化 | 代码审查 |

---

## 11. 可测试性要求

### 11.1 测试覆盖要求

| 测试类型 | 覆盖率要求 | 验证方法 |
|---|---|---|
| 单元测试 | ≥97% | 覆盖率工具 |
| 集成测试 | 100% 关键路径 | 测试报告 |
| E2E 测试 | 100% 端到端场景 | 测试报告 |

### 11.2 必测场景

| 场景 ID | 场景描述 | 优先级 |
|---|---|---|
| SPEC-TRANS-001 | 事务开始 | P0 |
| SPEC-TRANS-002 | 事务执行 (单指令) | P0 |
| SPEC-TRANS-003 | 事务执行 (多指令) | P0 |
| SPEC-TRANS-004 | 事务提交 | P0 |
| SPEC-TRANS-005 | 事务回滚 | P0 |
| SPEC-TRANS-006 | 事务超时自动回滚 | P0 |
| SPEC-TRANS-007 | 脏读检测 | P0 |
| SPEC-TRANS-008 | 重复提交检测 | P0 |
| SPEC-TRANS-009 | 事务哈希验证 | P0 |
| SPEC-TRANS-010 | 事务重放一致性 | P0 |

---

## 12. 监控与告警

### 12.1 必采指标

| 指标名 | 类型 | 采集频率 | 告警阈值 |
|---|---|---|---|
| transaction_commit_latency_p99 | Histogram | 实时 | >400ms |
| transaction_rollback_count | Counter | 实时 | >10/h |
| transaction_timeout_count | Counter | 实时 | >5/h |
| transaction_success_rate | Gauge | 实时 | <99% |

### 12.2 告警分级

| 告警级别 | 触发条件 | 响应时间 | 升级路径 |
|---|---|---|---|
| P0 | transaction_hash 验证失败 | <5 分钟 | Security→Dev→PM |
| P1 | 隔离违反 | <15 分钟 | Dev→PM |
| P2 | 事务超时率>5% | <1 小时 | SRE |

---

## 13. Spec 符合性验证

### 13.1 验证清单

| 验证项 | 验证方法 | 责任人 | 状态 |
|---|---|---|---|
| 输入契约符合性 | 代码审查 + 测试 | Dev+QA | 📋 待验证 |
| 输出契约符合性 | 代码审查 + 测试 | Dev+QA | 📋 待验证 |
| 哈希链符合性 | 代码审查 + 测试 | Dev+Security | 📋 待验证 |
| 隔离级别符合性 | 代码审查 + 测试 | Dev+QA | 📋 待验证 |
| 超时语义符合性 | 代码审查 + 测试 | Dev+SRE | 📋 待验证 |
| 安全闸门符合性 | 代码审查 + 测试 | Security+QA | 📋 待验证 |
| 性能符合性 | 压测 | SRE | 📋 待验证 |

### 13.2 Spec 版本控制

| 版本 | 变更日期 | 变更内容 | 批准人 |
|---|---|---|---|
| v1.0 | 2026-04-08 | 初始版本 | 📋 待批准 |

---

## 14. 附录

### 14.1 术语表

| 术语 | 定义 |
|---|---|
| Transaction | 事务，ACID 语义保证 |
| ACID | 原子性、一致性、隔离性、持久性 |
| Read Committed | 读已提交隔离级别 |
| Dirty Read | 脏读，读取未提交数据 |
| Non-Repeatable Read | 不可重复读 |
| Phantom Read | 幻读 |

### 14.2 参考文档

- Phase 1 最小指令集规范 v1
- Phase 2 PRD v2
- Phase 2 ADR v4
- Phase 2 Batch 规范 v1

---

**文档状态**: 📋 草案评审中  
**评审计划**: Week 3-T1 Spec 评审会议  
**责任人**: PM + Dev  
**保管**: 项目文档库
