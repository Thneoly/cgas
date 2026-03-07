# Phase 2 架构决策记录 (ADR v4)

**版本**: v4.0 (Draft - Phase 2 Kickoff)  
**日期**: 2026-03-31  
**责任人**: Dev + 架构师  
**状态**: 📋 草案评审中  
**release_id**: release-2026-03-31-phase2_week01  
**参与角色**: PM, Dev, QA, SRE, Security, 观测工程师

---

## 1. 架构概述

### 1.1 Phase 2 架构目标

在 Phase 1 架构基础上进行扩展，保持向后兼容，新增功能与性能优化：

| 目标 | Phase 1 状态 | Phase 2 策略 |
|---|---|---|
| 接口契约 | 冻结 (Execute/Verify/Commit) | 向后兼容，新增 Batch/Transaction |
| 架构模式 | 执行器 + 验证器 + 阻断中间件 | 保持，新增 Batch/Transaction 服务 |
| 性能基线 | P99 423ms/467ms | 优化至<300ms |
| 可观测性 | 15 个监控指标 | 扩展至 25 个 |

### 1.2 Phase 2 架构演进

```
Phase 1 架构 (保持):
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │───▶│   Blocking  │───▶│  Verifier   │───▶│   State     │
│  (请求)     │    │  Middleware │    │  (验证器)   │    │   Commit    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                          │                    │
                          ▼                    ▼
                   ┌─────────────┐    ┌─────────────┐
                   │  Scanner    │    │  Monitoring │
                   │ (非确定性)  │    │  (15 指标)   │
                   └─────────────┘    └─────────────┘

Phase 2 架构扩展 (新增):
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │───▶│  Batch      │───▶│  Executor   │
│  (Batch)    │    │  Service    │    │  (扩展)     │
└─────────────┘    └─────────────┘    └─────────────┘

┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │───▶│ Transaction │───▶│  Executor   │
│  (Trans)    │    │  Service    │    │  (扩展)     │
└─────────────┘    └─────────────┘    └─────────────┘

                          │
                          ▼
                   ┌─────────────┐    ┌─────────────┐
                   │  Monitoring │    │ Distributed │
                   │  (25 指标)   │    │  Tracing    │
                   └─────────────┘    └─────────────┘
```

---

## 2. 架构决策

### 2.1 ADR-001: Batch 指令架构

| 决策项 | 决策 | 理由 |
|---|---|---|
| Batch 执行模式 | 串行执行，原子性保证 | 简化实现，保证一致性 |
| Batch 大小限制 | 1-100 条指令 | 平衡性能与功能 |
| Batch 哈希链 | 独立 batch_hash，覆盖所有子指令 | 保证 Batch 完整性 |
| Batch 重放 | 整体重放，子指令独立 trace_id | 可追溯性 + 一致性 |
| 性能开销目标 | <20% (相比单条指令) | 用户体验 |

**接口设计**:
```rust
pub struct BatchExecuteRequest {
    pub trace_id: String,        // Batch 级 trace
    pub batch_id: String,        // Batch 唯一标识
    pub instructions: Vec<ExecuteRequest>,  // 子指令列表
    pub atomic: bool,            // 原子性保证
    pub timestamp: String,
}

pub struct BatchExecuteResult {
    pub trace_id: String,
    pub batch_id: String,
    pub status: BatchStatus,     // Success/PartialFailure/Failed
    pub results: Vec<ExecutionResult>,  // 子指令结果
    pub batch_hash: String,      // Batch 级哈希
    pub timestamp: String,
}
```

**状态**: 📋 待评审

---

### 2.2 ADR-002: Transaction 指令架构

| 决策项 | 决策 | 理由 |
|---|---|---|
| 事务语义 | BEGIN/COMMIT/ROLLBACK | 标准事务模型 |
| 隔离级别 | Read Committed (RC) | 平衡性能与一致性 |
| 事务超时 | 可配置，默认 5000ms | 防止长事务 |
| 事务重放 | 事务级重放，保持隔离性 | 一致性保证 |
| 回滚策略 | 自动回滚 + 审计日志 | 安全性 |

**接口设计**:
```rust
pub enum IsolationLevel {
    ReadCommitted,
    // Phase 2 仅支持 RC，Phase 3 可扩展
}

pub struct TransactionExecuteRequest {
    pub trace_id: String,
    pub transaction_id: String,
    pub isolation_level: IsolationLevel,
    pub instructions: Vec<ExecuteRequest>,
    pub timeout_ms: i64,
    pub timestamp: String,
}

pub struct TransactionExecuteResult {
    pub trace_id: String,
    pub transaction_id: String,
    pub status: TransactionStatus,  // Committed/RolledBack/Timeout
    pub results: Vec<ExecutionResult>,
    pub transaction_hash: String,
    pub timestamp: String,
}
```

**状态**: 📋 待评审

---

### 2.3 ADR-003: 性能优化架构

| 优化领域 | Phase 1 状态 | Phase 2 优化策略 | 目标提升 |
|---|---|---|---|
| 执行器 | 单线程处理 | 异步并发池 | +20% |
| 验证器 | 独立重放 | 增量重放优化 | +15% |
| 阻断中间件 | 全量校验 | 缓存 + 批量校验 | +10% |
| 序列化 | serde JSON | serde + 零拷贝优化 | +5% |
| 内存管理 | 标准分配 | 对象池复用 | +5% |

**优化技术栈**:
```rust
// 异步并发池
pub struct ExecutorPool {
    workers: Vec<ExecutorWorker>,
    task_queue: async_channel::Sender<Task>,
}

// 增量重放优化
impl Verifier {
    pub async fn incremental_replay(&self, request: &VerifyRequest) -> Result<VerifyResponse> {
        // 仅重放变化的 state_diff，跳过未变化部分
    }
}

// 对象池复用
pub struct StateDiffPool {
    pool: object_pool::Pool<Vec<StateDiffOperation>>,
}
```

**状态**: 📋 待评审

---

### 2.4 ADR-004: 监控指标扩展架构

| 指标类别 | Phase 1 (15 个) | Phase 2 新增 (10 个) |
|---|---|---|
| 性能指标 | 5 个 | +3 个 (Batch/Transaction 性能) |
| 一致性指标 | 3 个 | +1 个 (事务一致性) |
| 安全指标 | 4 个 | +2 个 (零信任相关) |
| 业务指标 | 3 个 | +4 个 (Batch/Transaction 业务指标) |

**新增监控指标**:
```rust
// Batch 相关
batch_execute_latency_p99
batch_atomicity_violation_count
batch_sub_instruction_count

// Transaction 相关
transaction_commit_latency_p99
transaction_rollback_count
transaction_timeout_count
transaction_isolation_violation_count

// 零信任相关
zero_trust_auth_failure_count
zero_trust_policy_violation_count

// 业务相关
instruction_type_distribution
client_version_distribution
```

**状态**: 📋 待评审

---

### 2.5 ADR-005: 分布式追踪架构

| 决策项 | 决策 | 理由 |
|---|---|---|
| 追踪标准 | OpenTelemetry | 行业标准，生态完善 |
| Trace 传播 | W3C Trace Context | 跨服务兼容 |
| Span 层级 | Request → Batch/Trans → Instruction | 完整链路 |
| 采样策略 | 头部 100% + 自适应采样 | 保证关键链路完整 |
| 存储后端 | Jaeger/Tempo | 开源，易集成 |

**Trace 层级设计**:
```
Trace (trace_id)
├── Span: BatchExecute/TransactionExecute
│   ├── Span: Executor.execute (instruction_1)
│   │   ├── Span: Verifier.verify
│   │   └── Span: Commit.commit
│   ├── Span: Executor.execute (instruction_2)
│   │   ├── Span: Verifier.verify
│   │   └── Span: Commit.commit
│   └── ...
```

**状态**: 📋 待评审

---

### 2.6 ADR-006: 零信任架构接入

| 决策项 | 决策 | 理由 |
|---|---|---|
| 身份验证 | OIDC/OAuth2 | 标准协议，易集成 |
| 授权模型 | RBAC + ABAC | 细粒度控制 |
| 策略引擎 | OPA (Open Policy Agent) | 灵活，可审计 |
| 密钥管理 | Vault/KMS | 安全存储 |
| 运行时保护 | seccomp/apparmor | 容器安全 |

**状态**: 📋 待评审

---

## 3. 接口契约扩展

### 3.1 Phase 1 接口 (保持向后兼容)

| 接口 | Phase 1 状态 | Phase 2 策略 |
|---|---|---|
| ExecuteRequest | 冻结 | 完全兼容 |
| ExecutionResult | 冻结 | 完全兼容 |
| VerifyRequest | 冻结 | 完全兼容 |
| VerifyResponse | 冻结 | 完全兼容 |
| CommitRequest | 冻结 | 完全兼容 |
| CommitResponse | 冻结 | 完全兼容 |

### 3.2 Phase 2 新增接口

```rust
// Batch 服务
pub struct BatchExecuteRequest { /* ... */ }
pub struct BatchExecuteResult { /* ... */ }

// Transaction 服务
pub struct BeginTransactionRequest { /* ... */ }
pub struct BeginTransactionResponse { /* ... */ }
pub struct CommitTransactionRequest { /* ... */ }
pub struct CommitTransactionResponse { /* ... */ }
pub struct RollbackTransactionRequest { /* ... */ }
pub struct RollbackTransactionResponse { /* ... */ }
```

### 3.3 gRPC 服务定义

```protobuf
// Phase 1 服务 (保持)
service ExecutorService {
  rpc Execute(ExecuteRequest) returns (ExecutionResult);
}

service VerifierService {
  rpc Verify(VerifyRequest) returns (VerifyResponse);
  rpc BatchVerify(BatchVerifyRequest) returns (BatchVerifyResponse);
}

service CommitService {
  rpc Commit(CommitRequest) returns (CommitResponse);
}

// Phase 2 新增服务
service BatchService {
  rpc BatchExecute(BatchExecuteRequest) returns (BatchExecuteResult);
}

service TransactionService {
  rpc BeginTransaction(BeginTransactionRequest) returns (BeginTransactionResponse);
  rpc CommitTransaction(CommitTransactionRequest) returns (CommitTransactionResponse);
  rpc RollbackTransaction(RollbackTransactionRequest) returns (RollbackTransactionResponse);
}
```

---

## 4. 性能优化方案

### 4.1 性能瓶颈分析 (Phase 1)

| 组件 | P99 时延 | 占比 | 优化空间 |
|---|---|---|---|
| 执行器 | 187ms | 44% | 异步并发 |
| 验证器 | 203ms | 48% | 增量重放 |
| 阻断中间件 | 23ms | 5% | 缓存优化 |
| 序列化 | 10ms | 3% | 零拷贝 |

### 4.2 优化目标分解

| 组件 | Phase 1 P99 | Phase 2 目标 | 优化幅度 |
|---|---|---|---|
| 执行器 | 187ms | <120ms | -36% |
| 验证器 | 203ms | <150ms | -26% |
| 阻断中间件 | 23ms | <15ms | -35% |
| 序列化 | 10ms | <8ms | -20% |
| **总计** | **423ms** | **<300ms** | **-29%** |

### 4.3 优化实施计划

| 周次 | 优化项 | 责任人 | 验证方法 |
|---|---|---|---|
| Week 2 | 异步并发池 | Dev | 压测 |
| Week 3 | 增量重放优化 | Dev | 压测 |
| Week 4 | 缓存 + 对象池 | Dev+SRE | 压测 |
| Week 4 | 性能基线报告 | SRE | 性能报告 |

---

## 5. 安全架构

### 5.1 Phase 1 安全机制 (保持)

| 机制 | Phase 1 状态 | Phase 2 策略 |
|---|---|---|
| SG-1~SG-4 闸门 | 硬阻断 | 保持 + 增强 |
| 未验证提交阻断 | 100% | 保持 |
| 哈希链验证 | 三哈希 | 保持 |
| 非确定性扫描 | 100% 识别 | 误报率优化 |

### 5.2 Phase 2 安全增强

| 增强项 | 实现方案 | 优先级 |
|---|---|---|
| 零信任身份验证 | OIDC/OAuth2 | P1 |
| 细粒度授权 | RBAC+ABAC+OPA | P1 |
| 密钥轮换自动化 | Vault/KMS | P1 |
| 审计日志增强 | 完整链路追踪 | P1 |

---

## 6. 可观测性架构

### 6.1 监控指标 (25 个)

| 类别 | Phase 1 | Phase 2 新增 | 总计 |
|---|---|---|---|
| 性能 | 5 | 3 | 8 |
| 一致性 | 3 | 1 | 4 |
| 安全 | 4 | 2 | 6 |
| 业务 | 3 | 4 | 7 |
| **总计** | **15** | **10** | **25** |

### 6.2 日志规范

| 日志类型 | Phase 1 | Phase 2 扩展 |
|---|---|---|
| 审计日志 | trace_id/execution_id | + batch_id/transaction_id |
| 阻断日志 | 完整记录 | 保持 |
| 性能日志 | P99 采集 | + 分指令类型统计 |

### 6.3 分布式追踪

| 层级 | Trace 覆盖 | Span 层级 |
|---|---|---|
| Request 级 | 100% | 1 |
| Batch/Trans 级 | 100% | 1 |
| Instruction 级 | 100% | N |
| Verifier 级 | 100% | N |
| Commit 级 | 100% | N |

---

## 7. 失败路径与回滚

### 7.1 Batch 失败处理

| 失败场景 | 处理策略 | 回滚机制 |
|---|---|---|
| 单条指令失败 (atomic=true) | 全部回滚 | 自动 REVERT |
| 单条指令失败 (atomic=false) | 继续执行，记录失败 | 部分提交 |
| Batch 超时 | 中断执行 | 自动回滚 |
| Batch 哈希验证失败 | 阻断提交 | 审计日志 |

### 7.2 Transaction 失败处理

| 失败场景 | 处理策略 | 回滚机制 |
|---|---|---|
| 事务超时 | 自动 ROLLBACK | 自动回滚 |
| 隔离级别违反 | 中断 + 错误 | 自动回滚 |
| 事务哈希验证失败 | 阻断提交 | 审计日志 |
| 死锁检测 | 选择牺牲者回滚 | 自动重试 |

---

## 8. 架构评审状态

### 8.1 待评审决策

| ADR ID | 决策主题 | 状态 | 责任人 |
|---|---|---|---|
| ADR-001 | Batch 指令架构 | 📋 待评审 | Dev |
| ADR-002 | Transaction 指令架构 | 📋 待评审 | Dev |
| ADR-003 | 性能优化架构 | 📋 待评审 | Dev+SRE |
| ADR-004 | 监控指标扩展 | 📋 待评审 | SRE |
| ADR-005 | 分布式追踪 | 📋 待评审 | SRE |
| ADR-006 | 零信任架构 | 📋 待评审 | Security |

### 8.2 评审时间安排

| 时间点 | 事件 | 参与方 |
|---|---|---|
| Week 1-T3 | ADR v4 初稿完成 | Dev |
| Week 1-T4 | 架构评审会议 | Dev+ 架构师 + 各角色 |
| Week 1-T5 | Entry Gate 评审 | 门禁官 + 四方 |

---

## 9. 附录

### 9.1 Phase 1 ADR 参考

| ADR | 主题 | 状态 |
|---|---|---|
| ADR v1 | Phase 1 基础架构 | ✅ 已实施 |
| ADR v2 | 验证器重放链路 | ✅ 已实施 |
| ADR v3 | 阻断中间件部署 | ✅ 已实施 |

### 9.2 术语表

| 术语 | 定义 |
|---|---|
| Batch | 批量指令执行，原子性保证 |
| Transaction | 事务语义，支持 BEGIN/COMMIT/ROLLBACK |
| Read Committed | 读已提交隔离级别 |
| OpenTelemetry | 分布式追踪标准 |
| OPA | Open Policy Agent，策略引擎 |

---

**文档状态**: 📋 草案评审中  
**下次更新**: Week 1-T4 (架构评审后)  
**责任人**: Dev + 架构师
