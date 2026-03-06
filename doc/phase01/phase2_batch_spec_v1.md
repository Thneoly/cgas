# Batch 指令规范 v1.0（Phase 2）

**版本**: v1.0  
**日期**: 2026-04-08  
**责任人**: PM + Dev  
**状态**: 📋 草案评审中  
**release_id**: release-2026-04-08-phase2_week02  
**参与角色**: PM, Dev, QA, SRE, Security

---

## 1. 目标

定义 Phase 2 Batch 批量指令的语义规范，确保：
- 批量执行确定性
- 原子性保证可验证
- 执行结果可重放
- 安全闸门可集成

---

## 2. 设计原则

| 原则 | 描述 | 验证方法 |
|---|---|---|
| **确定性** | 相同输入产生相同输出 | 重放一致性≥99.95% |
| **原子性** | 全部成功或全部失败 (可配置) | 原子性违反检测 |
| **可追溯** | 每条子指令独立 trace_id | 审计日志完整 |
| **可验证** | Verifier 支持 Batch 重放 | SG-1~SG-4 验证 |
| **向后兼容** | 保持 Phase 1 接口契约 | 接口兼容性测试 |

---

## 3. Batch 指令定义

### 3.1 指令分类

| 类别 | 指令名 | 说明 |
|---|---|---|
| **Batch 控制类** | `BATCH_EXECUTE` | 执行批量指令 |
| **子指令** | 继承 Phase 1 最小指令集 | READ/COMPUTE/WRITE/CONTROL |

### 3.2 指令语义

**BATCH_EXECUTE**:
- **功能**: 批量执行多条子指令
- **输入**: BatchExecuteRequest
- **输出**: BatchExecuteResult
- **副作用**: 仅产出 state_diff，不直接提交

---

## 4. 输入输出契约

### 4.1 输入契约

```rust
/// Batch 执行请求
pub struct BatchExecuteRequest {
    /// Batch 级 trace ID (格式：UUID v4)
    pub trace_id: String,
    
    /// Batch 唯一标识 (格式：UUID v4)
    pub batch_id: String,
    
    /// 子指令列表 (1-100 条)
    pub instructions: Vec<ExecuteRequest>,
    
    /// 原子性保证 (true=全部成功或全部失败)
    pub atomic: bool,
    
    /// 请求时间戳 (RFC3339 格式)
    pub timestamp: String,
}
```

**约束条件**:
| 字段 | 约束 | 验证规则 |
|---|---|---|
| trace_id | 非空，UUID v4 | `^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$` |
| batch_id | 非空，UUID v4 | 同上 |
| instructions | 1-100 条 | `1 <= len <= 100` |
| atomic | 布尔值 | `true` 或 `false` |
| timestamp | RFC3339 | `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d+Z$` |

### 4.2 输出契约

```rust
/// Batch 执行结果
pub struct BatchExecuteResult {
    /// Batch 级 trace ID (与请求一致)
    pub trace_id: String,
    
    /// Batch 唯一标识 (与请求一致)
    pub batch_id: String,
    
    /// Batch 状态
    pub status: BatchStatus,
    
    /// 子指令结果列表 (与请求指令数一致)
    pub results: Vec<ExecutionResult>,
    
    /// Batch 级哈希 (SHA256)
    pub batch_hash: String,
    
    /// 结果时间戳 (RFC3339 格式)
    pub timestamp: String,
}

/// Batch 状态
pub enum BatchStatus {
    Success,         // 全部成功
    PartialFailure,  // 部分失败 (atomic=false)
    Failed,          // 全部失败
}
```

**状态转换规则**:

| 输入条件 | 执行结果 | 输出状态 |
|---|---|---|
| atomic=true, 全部成功 | 无失败 | Success |
| atomic=true, 有失败 | 回滚已执行 | Failed |
| atomic=false, 全部成功 | 无失败 | Success |
| atomic=false, 部分失败 | 记录失败，继续执行 | PartialFailure |
| atomic=false, 全部失败 | 全部失败 | Failed |

---

## 5. Batch 哈希链规范

### 5.1 哈希计算算法

```
batch_hash = SHA256(
    trace_id_1 || "\x00" ||
    trace_id_2 || "\x00" ||
    ... ||
    trace_id_n || "\x00" ||
    result_hash_1 || "\x00" ||
    result_hash_2 || "\x00" ||
    ... ||
    result_hash_n || "\x00" ||
    batch_id
)
```

**哈希覆盖**:
1. 所有子指令的 trace_id (按顺序)
2. 所有子指令结果的 result_hash (按顺序)
3. batch_id

### 5.2 哈希验证规则

| 验证项 | 规则 | 失败动作 |
|---|---|---|
| 哈希长度 | 64 字符 (SHA256 hex) | 拒绝提交 |
| 哈希一致性 | 重算哈希与 batch_hash 一致 | SG-3 阻断 |
| 子指令哈希 | 每个 result_hash 有效 | 拒绝提交 |

---

## 6. 原子性语义

### 6.1 原子性模式

**atomic=true (原子性模式)**:
- 任何子指令失败 → 回滚所有已执行指令
- 最终状态：全部成功 或 全部失败
- 无中间状态暴露

**atomic=false (非原子性模式)**:
- 子指令失败 → 记录失败，继续执行后续指令
- 最终状态：可能部分成功
- 允许 PartialFailure 状态

### 6.2 回滚语义

**回滚触发条件**:
- atomic=true 且任意子指令执行失败

**回滚顺序**:
- 逆序回滚 (LIFO)
- 回滚失败 → 记录审计日志，告警

**回滚保证**:
- 回滚操作本身必须可重放
- 回滚失败必须告警 (P1 告警)

---

## 7. 错误与 REVERT 语义

### 7.1 错误分类

| 错误类型 | 错误码 | 触发条件 | 处理动作 |
|---|---|---|---|
| ValidationError | 1001 | 参数验证失败 | 立即拒绝 |
| EmptyBatch | 1002 | 指令数为 0 | 立即拒绝 |
| BatchTooLarge | 1003 | 指令数>100 | 立即拒绝 |
| InvalidTraceId | 1004 | trace_id 格式错误 | 立即拒绝 |
| ExecutorError | 2001-2999 | 子指令执行失败 | 根据 atomic 处理 |
| RollbackFailed | 3001 | 回滚操作失败 | 告警 + 审计 |
| HashMismatch | 4001 | batch_hash 验证失败 | SG-3 阻断 |

### 7.2 REVERT 语义

**REVERT 触发条件**:
- atomic=true 且任意子指令失败
- batch_hash 验证失败

**REVERT 动作**:
1. 回滚所有已执行的子指令
2. 记录审计日志
3. 返回 Failed 状态

**REVERT 保证**:
- REVERT 操作本身必须可重放
- REVERT 不影响已提交状态

---

## 8. 安全闸门集成

### 8.1 SG-1: Batch 路径验证

**验证规则**:
- Batch 提交必须经过 Batch Verifier
- batch_hash 必须存在

**阻断动作**:
- 未验证 Batch → 拒绝提交
- 缺失 batch_hash → 拒绝提交

### 8.2 SG-2: Batch 隔离验证

**验证规则**:
- Batch 执行器与验证器隔离
- 原子性回滚机制就绪

**阻断动作**:
- 隔离失效 → 拒绝执行
- 回滚机制未就绪 → 拒绝执行 (atomic=true)

### 8.3 SG-3: Batch 哈希验证

**验证规则**:
- 重新计算 batch_hash 与提交 hash 一致
- 每个子指令 result_hash 有效

**阻断动作**:
- 哈希不一致 → 拒绝提交
- 子指令 hash 无效 → 拒绝提交

### 8.4 SG-4: Batch 权限 + 重放检查

**验证规则**:
- 用户有 Batch 提交权限
- batch_id 未重复使用
- batch_hash 未重复使用

**阻断动作**:
- 权限不足 → 拒绝提交
- 重放检测 → 拒绝提交，告警

---

## 9. 性能规范

### 9.1 性能指标

| 指标 | 目标值 | 测量方法 |
|---|---|---|
| P99 执行时延 (1 条) | <450ms | 压测 |
| P99 执行时延 (10 条) | <500ms | 压测 |
| P99 执行时延 (100 条) | <700ms | 压测 |
| Batch 开销 (相比单条) | <20% | 对比测试 |
| 吞吐量 | ≥100 请求/秒 | 压测 |

### 9.2 性能优化要求

| 优化项 | 要求 | 验证方法 |
|---|---|---|
| 并行执行 | 无依赖子指令可并行 | 代码审查 |
| 对象池 | Vec<ExecutionResult> 池化 | 代码审查 |
| 批量哈希 | 一次性计算所有子指令哈希 | 代码审查 |

---

## 10. 可测试性要求

### 10.1 测试覆盖要求

| 测试类型 | 覆盖率要求 | 验证方法 |
|---|---|---|
| 单元测试 | ≥97% | 覆盖率工具 |
| 集成测试 | 100% 关键路径 | 测试报告 |
| E2E 测试 | 100% 端到端场景 | 测试报告 |

### 10.2 必测场景

| 场景 ID | 场景描述 | 优先级 |
|---|---|---|
| SPEC-BATCH-001 | 单条指令 Batch 执行 | P0 |
| SPEC-BATCH-002 | 多条指令 Batch 执行 (10 条) | P0 |
| SPEC-BATCH-003 | 最大指令数 Batch 执行 (100 条) | P0 |
| SPEC-BATCH-004 | 原子性成功 | P0 |
| SPEC-BATCH-005 | 原子性失败回滚 | P0 |
| SPEC-BATCH-006 | 非原子性部分成功 | P1 |
| SPEC-BATCH-007 | 空 Batch 验证 | P0 |
| SPEC-BATCH-008 | 超大批 Batch 验证 | P0 |
| SPEC-BATCH-009 | Batch 哈希计算正确性 | P0 |
| SPEC-BATCH-010 | Batch 重放一致性 | P0 |

---

## 11. 监控与告警

### 11.1 必采指标

| 指标名 | 类型 | 采集频率 | 告警阈值 |
|---|---|---|---|
| batch_execute_latency_p99 | Histogram | 实时 | >400ms |
| batch_atomicity_violation_count | Counter | 实时 | >0 |
| batch_sub_instruction_count | Histogram | 实时 | - |
| batch_success_rate | Gauge | 实时 | <99.9% |

### 11.2 告警分级

| 告警级别 | 触发条件 | 响应时间 | 升级路径 |
|---|---|---|---|
| P0 | batch_hash 验证失败 | <5 分钟 | Security→Dev→PM |
| P1 | 原子性违反 | <15 分钟 | Dev→PM |
| P2 | P99 时延>500ms | <1 小时 | SRE |

---

## 12. Spec 符合性验证

### 12.1 验证清单

| 验证项 | 验证方法 | 责任人 | 状态 |
|---|---|---|---|
| 输入契约符合性 | 代码审查 + 测试 | Dev+QA | 📋 待验证 |
| 输出契约符合性 | 代码审查 + 测试 | Dev+QA | 📋 待验证 |
| 哈希链符合性 | 代码审查 + 测试 | Dev+Security | 📋 待验证 |
| 原子性符合性 | 代码审查 + 测试 | Dev+QA | 📋 待验证 |
| 安全闸门符合性 | 代码审查 + 测试 | Security+QA | 📋 待验证 |
| 性能符合性 | 压测 | SRE | 📋 待验证 |

### 12.2 Spec 版本控制

| 版本 | 变更日期 | 变更内容 | 批准人 |
|---|---|---|---|
| v1.0 | 2026-04-08 | 初始版本 | 📋 待批准 |

---

## 13. 附录

### 13.1 术语表

| 术语 | 定义 |
|---|---|
| Batch | 批量指令执行，原子性保证 |
| Atomic | 原子性，全部成功或全部失败 |
| PartialFailure | 部分失败，部分指令成功 |
| batch_hash | Batch 级哈希，覆盖所有子指令 |
| REVERT | 回滚操作，撤销已执行指令 |

### 13.2 参考文档

- Phase 1 最小指令集规范 v1
- Phase 2 PRD v2
- Phase 2 ADR v4
- Phase 2 Batch 设计文档

---

**文档状态**: 📋 草案评审中  
**评审计划**: Week 2-T3 Spec 评审会议  
**责任人**: PM + Dev  
**保管**: 项目文档库
