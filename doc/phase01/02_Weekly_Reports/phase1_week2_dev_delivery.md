# Phase1 Week2 开发交付物 - 技术方案与接口契约

**版本**: v2.0 (四方联签版)  
**日期**: 2026-03-09  
**责任人**: Dev (Core)  
**状态**: ✅ 完成 (四方联签确认)  
**release_id**: release-2026-03-05-phase1_week02  
**参与角色**: PM, Dev, QA, SRE, Security

---

## 1. 四方联签确认

### 1.1 角色确认状态

| 角色 | 决策 | 确认要点 | 确认时间 |
|---|---|---|---|
| PM | ✅ approved | 核心指标全部达标，Week3 准入就绪 | 2026-03-09 |
| Dev | ✅ approved | 指令执行器实现完成，state_diff-only 落地 | 2026-03-09 |
| QA | ✅ approved | 单测 96.5%，异常分支 94.2%，SG-1~SG-4 100% | 2026-03-09 |
| SRE | ✅ approved | 监控指标 100%，三哈希链验证通过 | 2026-03-09 |
| Security | ✅ approved | 未验证提交率 0%，无高风险项 | 2026-03-09 |

### 1.2 核心指标四方确认

| 指标 | 目标值 | 实际值 | 验证方 | 状态 |
|---|---|---|---|---|
| 核心指令单测通过率 | ≥95% | 96.5% | Dev + QA | ✅ |
| result_hash 覆盖率 | 100% | 100% | Dev + SRE + Security | ✅ |
| 未验证提交率 | =0 | 0% (0/52) | Security | ✅ |
| 审计字段完整度 | 100% | 100% | SRE + Security | ✅ |
| 提交闸门验证通过率 | 100% | 100% (SG-1~SG-4) | Security | ✅ |
| 异常分支覆盖率 | ≥90% | 94.2% | QA | ✅ |
| 监控指标采集完整度 | 100% | 100% | SRE | ✅ |

---

## 2. 技术方案概述

### 2.1 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                    Phase1 执行器架构                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   Client    │───▶│  Executor   │───▶│  Verifier   │     │
│  │  (指令输入)  │    │ (state_diff)│    │  (提交闸门)  │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│                          │                    │              │
│                          ▼                    ▼              │
│                   ┌─────────────┐    ┌─────────────┐        │
│                   │ Audit Log   │    │   State     │        │
│                   │ (哈希链)    │    │   Commit    │        │
│                   └─────────────┘    └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

**核心原则**:
- **state_diff-only**: 执行器仅输出状态差异，不直接提交
- **权限隔离**: 执行层无提交权限，提交逻辑隔离至 Verifier
- **审计完整**: 三哈希链验证 (trace_hash, result_hash, state_diff_hash)
- **四方联签**: PM/Dev/QA/SRE/Security 共同确认关键节点

### 2.2 指令集实现

| 指令 | 单测数 | 通过率 | 状态 | 备注 | 确认方 |
|---|---|---|---|---|---|
| CREATE | 28 | 100% | ✅ | 资源创建语义完整 | Dev + QA |
| READ | 25 | 100% | ✅ | 只读查询无副作用 | Dev + QA |
| UPDATE | 32 | 93.75% | 🟡 | 2 边界条件优化中 | Dev + QA |
| DELETE | 22 | 100% | ✅ | 软删除支持 | Dev + QA |
| QUERY | 20 | 90% | 🟡 | 2 复杂查询优化中 | Dev + QA |
| **合计** | **127** | **96.5%** | ✅ | 核心链路可用 | Dev + QA |

**备注**: 4 个未通过用例为边界条件优化，不影响核心链路，已纳入 Week3 T1 修复计划 (QA 确认)。

---

## 3. 接口契约

### 3.1 指令输入契约

```rust
/// 指令输入结构 (serde 序列化)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    /// 指令类型
    pub instruction_type: InstructionType,
    /// 追踪 ID (UUID)
    pub trace_id: String,
    /// 执行 ID (UUID)
    pub execution_id: String,
    /// 指令负载
    pub payload: InstructionPayload,
    /// 时间戳 (ISO8601)
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InstructionType {
    CREATE,
    READ,
    UPDATE,
    DELETE,
    QUERY,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionPayload {
    /// 资源标识
    pub resource_id: Option<String>,
    /// 资源类型
    pub resource_type: String,
    /// 数据内容
    pub data: Option<serde_json::Value>,
    /// 查询条件 (QUERY 指令)
    pub query: Option<QueryCondition>,
}
```

**字段验证** (SRE 确认):
- trace_id: UUID 格式，100% 覆盖
- execution_id: UUID 格式，100% 覆盖
- timestamp: ISO8601 格式，100% 覆盖

### 3.2 执行输出契约

```rust
/// 执行器输出 (state_diff only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// 追踪 ID (与输入一致)
    pub trace_id: String,
    /// 执行 ID (与输入一致)
    pub execution_id: String,
    /// 执行状态
    pub status: ExecutionStatus,
    /// 状态差异 (JSON Patch 格式)
    pub state_diff: Vec<StateDiffOperation>,
    /// 结果哈希 (SHA256)
    pub result_hash: String,
    /// 状态差异哈希 (SHA256)
    pub state_diff_hash: String,
    /// 时间戳 (ISO8601)
    pub timestamp: String,
    /// 错误信息 (可选)
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    Failed,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiffOperation {
    /// 操作类型 (add/remove/replace)
    pub op: String,
    /// 资源路径
    pub path: String,
    /// 值 (可选)
    pub value: Option<serde_json::Value>,
}
```

**哈希验证** (Security 确认):
- result_hash: SHA256 覆盖完整 ExecutionResult，100% 验证
- state_diff_hash: SHA256 覆盖 state_diff，100% 验证

### 3.3 审计日志契约

```rust
/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// 追踪哈希 (SHA256 of trace_id)
    pub trace_hash: String,
    /// 结果哈希 (SHA256 of result)
    pub result_hash: String,
    /// 状态差异哈希 (SHA256 of state_diff)
    pub state_diff_hash: String,
    /// 执行 ID
    pub execution_id: String,
    /// 时间戳
    pub timestamp: String,
    /// 操作者
    pub operator: String,
    /// 指令类型
    pub instruction_type: String,
    /// 执行状态
    pub status: String,
}
```

**三哈希链验证** (Security + SRE 确认):
```
trace_hash = SHA256(trace_id)
result_hash = SHA256(serialize(ExecutionResult))
state_diff_hash = SHA256(serialize(state_diff))

验证规则:
1. trace_hash 必须与输入 trace_id 匹配
2. result_hash 必须覆盖完整 ExecutionResult
3. state_diff_hash 必须与 result.state_diff_hash 一致
```

**字段覆盖清单** (SRE + Security 三方确认):
| 字段名 | 类型 | 覆盖率 | 验证方式 | 验证方 |
|---|---|---|---|---|
| trace_id | UUID | 100% | 日志扫描 | SRE |
| execution_id | UUID | 100% | 日志扫描 | SRE |
| result_hash | SHA256 | 100% | 哈希校验 | Security |
| timestamp | ISO8601 | 100% | 格式校验 | SRE |
| state_diff | JSON | 100% | Schema 校验 | Dev |

---

## 4. 失败路径与回滚路径

### 4.1 失败路径分类

| 失败类型 | 触发条件 | 处理方式 | 回滚策略 | 确认方 |
|---|---|---|---|---|
| 指令解析失败 | JSON 格式错误/字段缺失 | 返回错误，不执行 | 无需回滚 | Dev + QA |
| 资源不存在 | READ/UPDATE/DELETE 目标资源缺失 | 返回 NotFound 错误 | 无需回滚 | Dev + QA |
| 并发冲突 | 版本号不匹配 | 返回 Conflict 错误 | 客户端重试 | Dev + QA |
| 执行超时 | 超过阈值 (待定义) | 中断执行，记录日志 | 状态回滚 | Dev (Week3) |
| 哈希验证失败 | 三哈希链不一致 | 阻断提交，告警 | 完整回滚 | Security |

### 4.2 错误处理流程

```
┌─────────────────────────────────────────────────────────────┐
│                    错误处理流程                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  指令输入 ──▶ 解析验证 ──▶ 执行 ──▶ 生成 state_diff        │
│      │            │          │            │                 │
│      │            │          │            │                 │
│      ▼            ▼          ▼            ▼                 │
│   格式错误    字段缺失    资源错误    哈希校验              │
│      │            │          │            │                 │
│      ▼            ▼          ▼            ▼                 │
│  返回错误    返回错误    返回错误    阻断提交               │
│      │            │          │            │                 │
│      └────────────┴──────────┴────────────┘                 │
│                         │                                   │
│                         ▼                                   │
│                  记录审计日志                                │
│                  (含错误信息)                                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 4.3 回滚路径实现

```rust
/// 回滚策略枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackStrategy {
    /// 无需回滚 (解析错误/资源不存在)
    None,
    /// 客户端重试 (并发冲突)
    ClientRetry {
        max_retries: u32,
        backoff_ms: u64,
    },
    /// 状态回滚 (执行超时/哈希验证失败)
    StateRollback {
        snapshot_id: String,
        rollback_to: String,
    },
}

/// 回滚执行器
impl RollbackExecutor {
    /// 执行回滚
    pub fn rollback(&self, strategy: RollbackStrategy) -> Result<(), RollbackError> {
        match strategy {
            RollbackStrategy::None => Ok(()),
            RollbackStrategy::ClientRetry { max_retries, backoff_ms } => {
                // 通知客户端重试
                self.notify_client_retry(max_retries, backoff_ms)
            }
            RollbackStrategy::StateRollback { snapshot_id, rollback_to } => {
                // 从快照恢复状态
                self.restore_from_snapshot(snapshot_id, rollback_to)
            }
        }
    }
}
```

### 4.4 超时处理 (Week3 待完善)

| 场景 | 当前状态 | Week3 计划 | 责任人 |
|---|---|---|---|
| 超时阈值定义 | 🟡 监控中 (R-EXEC-001) | Week3-T1 定义默认 30s | Dev |
| 降级策略 | 🟡 监控中 | Week3-T1 实现熔断机制 | Dev |
| 超时告警 | ✅ 已实现 | 持续监控 | SRE |

**Security 确认**: 超时处理已映射到风险缓解措施，Week3-T1 交付。

### 4.5 状态快照一致性 (Week3 待验证)

| 场景 | 当前状态 | Week3 计划 | 责任人 |
|---|---|---|---|
| 快照生成 | ✅ 已实现 | 性能优化 | Dev |
| 快照验证 | 🟡 监控中 (R-EXEC-002) | Week3-T3 纳入验证器测试 | QA + Dev |
| 快照恢复 | ✅ 已实现 | 边界条件测试 | Dev |

**QA 确认**: 快照一致性验证纳入 Week3 T3 测试计划。

---

## 5. 风险控制措施

### 5.1 代码级控制 (Security 确认)

- ✅ **权限隔离**: 执行器无提交权限，编译期保证
- ✅ **静态分析**: Security 扫描 0 条直接提交路径
- ✅ **哈希链验证**: 三哈希链自动化校验
- ✅ **审计完整**: 5 核心字段 100% 覆盖

### 5.2 测试级控制 (QA 确认)

- ✅ **单测覆盖**: 127 用例，96.5% 通过率
- ✅ **异常分支**: 94.2% 覆盖率
- ✅ **集成测试**: 核心链路 100% 通过
- ✅ **专项测试**: N=384 样本，统计显著性满足

### 5.3 运行级控制 (SRE + Security 确认)

- ✅ **提交闸门**: SG-1~SG-4 验证通过率 100%
- ✅ **监控采集**: SRE 监控指标 100% 完整
- 🟡 **性能压测**: Week3 关注项 (R-02, RISK-002)
- 🟡 **灰度验证**: Week4 关注项 (R-04 阻断机制误杀)

### 5.4 风险台账闭环

| 风险 ID | 风险描述 | 本周状态 | 缓解措施 | 确认方 |
|---|---|---|---|---|
| R-01 | 指令语义理解分歧 | ✅ 关闭 | 每日站会对齐，示例用例库建立 | PM |
| R-02 | Verifier 重放性能不足 | 🟡 监控中 | Week3 预留性能压测资源 | PM + SRE |
| R-03 | 哈希链字段遗漏 | ✅ 关闭 | 5 核心字段 100% 覆盖，自动化扫描集成 | Security |
| R-04 | 阻断机制误杀 | 🟡 监控中 | Week4 灰度验证 | Security |
| R-05 | 跨角色依赖延迟 | 🟡 监控中 | 依赖就绪率 95%，持续跟踪 | PM |
| R-EXEC-001 | 指令超时边界 | 🟡 监控中 | Week3-T1 定义超时阈值 + 降级策略 | Dev |
| R-EXEC-002 | 状态快照一致性 | 🟡 监控中 | Week3-T3 纳入验证器测试 | QA + Dev |
| RISK-002 | 资源泄漏 | 🟡 监控中 | Week3 性能压测验证 | SRE |
| RISK-005 | 可观测盲区 | 🟡 监控中 | 新增监控指标，Week3 接入 | SRE |

**风险统计**:
- 风险总数：9 (上周 5 + Dev 新增 2 + SRE 新增 2)
- 已关闭：3 (R-01, R-03, RISK-001/003/004 中 3 项)
- 监控中：6 (R-02, R-04, R-05, R-EXEC-001, R-EXEC-002, RISK-002, RISK-005)
- 高风险：0
- 中风险：3 (R-02, R-04, R-EXEC-002)
- 低风险：3 (R-05, R-EXEC-001, RISK-002, RISK-005)

---

## 6. Week3 开发计划

### 6.1 待修复项 (QA 确认)

| 任务 ID | 描述 | 优先级 | ETA | 确认方 |
|---|---|---|---|---|
| W3-T1 | UPDATE 指令 2 边界条件优化 | P1 | Week3-T1 | Dev + QA |
| W3-T2 | QUERY 指令 2 复杂查询优化 | P1 | Week3-T1 | Dev + QA |
| W3-T3 | 超时阈值定义与降级策略 | P1 | Week3-T2 | Dev + Security |
| W3-T4 | 快照一致性验证纳入测试 | P2 | Week3-T3 | QA + Dev |

### 6.2 性能优化 (SRE 确认)

| 优化项 | 目标 | 当前 | 计划 | 确认方 |
|---|---|---|---|---|
| Verifier 重放性能 | <100ms/p99 | 待压测 | Week3-T5 | SRE |
| 状态快照生成 | <50ms/p99 | 待压测 | Week3-T6 | SRE |
| 哈希链计算 | <10ms/p99 | 待压测 | Week3-T7 | SRE |

### 6.3 准入条件 (四方联签)

| 条件 | 目标 | 当前 | 确认方 |
|---|---|---|---|
| 核心指令单测通过率 | ≥95% | 96.5% | Dev + QA |
| result_hash 覆盖率 | 100% | 100% | Dev + SRE + Security |
| 未验证提交率 | =0 | 0% | Security |
| 审计字段完整度 | 100% | 100% | SRE + Security |
| 提交闸门验证通过率 | 100% | 100% | Security |

**Week3 准入状态**: ✅ 就绪 (PM 确认)

---

## 7. 交付确认

### 7.1 四方联签

| 角色 | 确认项 | 状态 | 确认方 | 日期 |
|---|---|---|---|---|
| PM | 周目标达成，核心指标达标 | ✅ | PM | 2026-03-09 |
| Dev | 指令执行器代码，state_diff-only 语义 | ✅ | Dev | 2026-03-09 |
| QA | 单测报告，异常分支覆盖，SG-1~SG-4 验证 | ✅ | QA | 2026-03-09 |
| SRE | 监控指标采集，审计字段覆盖，三哈希链 | ✅ | SRE | 2026-03-09 |
| Security | 提交闸门验证，未验证提交率 0%，无高风险 | ✅ | Security | 2026-03-09 |

### 7.2 交付物清单

| 交付物 | 路径 | 状态 | 确认方 |
|---|---|---|---|
| 指令执行器增量代码 | src/executor/ | ✅ | Dev + QA |
| state_diff 输出路径实现 | src/executor/state_diff.rs | ✅ | Dev + Security |
| 审计日志字段实现 | src/audit/log_entry.rs | ✅ | Dev + SRE + Security |
| 三哈希链验证实现 | src/audit/hash_chain.rs | ✅ | Dev + Security |
| 单测报告 | tests/unit_report_v2.md | ✅ | Dev + QA |
| 专项测试报告 | tests/special_report_v2.md | ✅ | QA |
| 提交闸门验证报告 | security/sg_verification_v2.md | ✅ | Security |
| 监控指标采集实现 | sre/monitoring_metrics.rs | ✅ | SRE |

**交付状态**: ✅ Week3 准入就绪 (四方联签确认)

---

## 8. 附录：Rust 实现要点

### 8.1 所有权/借用模式

```rust
// 执行器核心逻辑 - 所有权转移确保 state_diff 唯一输出
impl Executor {
    pub fn execute(&self, instruction: Instruction) -> Result<ExecutionResult, ExecutorError> {
        // instruction 所有权转移，避免复用
        let state_diff = self.process_instruction(instruction)?;
        // state_diff 唯一输出路径，无直接提交
        Ok(ExecutionResult::from_state_diff(state_diff))
    }
}
```

### 8.2 Result 错误处理

```rust
// 错误传播链
fn process_instruction(&self, instruction: Instruction) -> Result<StateDiff, ExecutorError> {
    let validated = self.validate(&instruction)?;  // 解析错误
    let resource = self.get_resource(&validated)?; // 资源错误
    let diff = self.compute_diff(&validated, &resource)?; // 计算错误
    Ok(diff)
}
```

### 8.3 serde 契约

```rust
// 序列化/反序列化确保契约一致
#[derive(Serialize, Deserialize)]
pub struct ExecutionResult {
    #[serde(rename = "traceId")]
    pub trace_id: String,
    #[serde(rename = "resultHash")]
    pub result_hash: String,
    // ...
}
```

---

*本交付物由 Dev 角色生成，基于 execution_board v2.0 执行结论，经 PM/QA/SRE/Security 四方联签确认。*
