# Phase 2 产品需求文档 (PRD v2 草案)

**版本**: v2.0 (Draft - Phase 2 Kickoff)  
**日期**: 2026-03-31  
**责任人**: PM  
**状态**: 📋 草案评审中  
**release_id**: release-2026-03-31-phase2_week01  
**参与角色**: PM, Dev, QA, SRE, Security, 观测工程师

---

## 1. 执行摘要

### 1.1 Phase 2 目标

Phase 2 在 Phase 1 基础上进行功能扩展与性能优化，核心目标：

| 目标类别 | Phase 1 基线 | Phase 2 目标 | 提升幅度 |
|---|---|---|---|
| **功能扩展** | 基础指令集 | Batch + Transaction 指令 | +2 类指令 |
| **性能优化** | P99 423ms/467ms | **P99 <300ms** | ~30% 提升 |
| **可观测性** | 15 个监控指标 | **25 个监控指标** | +10 个指标 |
| **安全增强** | 扫描器误报率 3.2% | **<2%** | ~40% 降低 |
| **质量提升** | E2E 98.7% | **≥99%** | +0.3% 提升 |
| **一致性** | 重放一致率 99.94% | **≥99.95%** | +0.01% 提升 |

### 1.2 Phase 2 范围

**纳入范围**:
- ✅ Batch 批量指令支持
- ✅ Transaction 事务指令支持
- ✅ P99 时延优化至<300ms
- ✅ 扫描器误报率优化至<2%
- ✅ 32 个 Phase 1 边界场景修复
- ✅ 监控指标扩展至 25 个
- ✅ 分布式追踪全链路覆盖
- ✅ 零信任架构接入

**排除范围**:
- ❌ 新指令类型扩展 (Batch/Transaction 除外)
- ❌ 架构重构 (保持 Phase 1 架构向后兼容)
- ❌ 生产全量发布 (Phase 2 仅影子验证/只读模式)

---

## 2. Phase 2 功能需求

### 2.1 Batch 批量指令 (P0)

| 需求 ID | 需求描述 | 优先级 | 验收标准 |
|---|---|---|---|
| F-BATCH-001 | 支持批量提交多个指令 | P0 | 单次请求支持 1-100 条指令 |
| F-BATCH-002 | Batch 指令原子性保证 | P0 | 全部成功或全部失败 |
| F-BATCH-003 | Batch 指令重放一致性 | P0 | 重放一致率≥99.95% |
| F-BATCH-004 | Batch 指令性能开销 | P1 | 相比单条指令开销<20% |
| F-BATCH-005 | Batch 指令审计日志 | P1 | 每条子指令独立 trace_id |

### 2.2 Transaction 事务指令 (P0)

| 需求 ID | 需求描述 | 优先级 | 验收标准 |
|---|---|---|---|
| F-TRANS-001 | 支持事务语义 (BEGIN/COMMIT/ROLLBACK) | P0 | 完整事务生命周期 |
| F-TRANS-002 | 事务隔离级别 (Read Committed) | P0 | 符合 RC 隔离级别 |
| F-TRANS-003 | 事务重放一致性 | P0 | 重放一致率≥99.95% |
| F-TRANS-004 | 事务超时处理 | P1 | 超时自动回滚 |
| F-TRANS-005 | 事务审计日志 | P1 | 完整事务链路追踪 |

### 2.3 性能优化 (P0)

| 需求 ID | 需求描述 | 优先级 | 验收标准 |
|---|---|---|---|
| F-PERF-001 | P99 执行时延<300ms | P0 | 压测验证通过 |
| F-PERF-002 | P99 验证时延<300ms | P0 | 压测验证通过 |
| F-PERF-003 | 阻断性能开销<5% | P1 | 性能基线报告验证 |
| F-PERF-004 | 内存使用优化 | P2 | 无内存泄漏，使用率<60% |

### 2.4 安全增强 (P1)

| 需求 ID | 需求描述 | 优先级 | 验收标准 |
|---|---|---|---|
| F-SEC-001 | 扫描器误报率<2% | P1 | 对抗测试验证 |
| F-SEC-002 | 零信任架构接入 | P1 | 身份验证 + 授权完整 |
| F-SEC-003 | 未验证提交率=0 | P0 | 红线保持 |
| F-SEC-004 | SG-1~SG-4 验证通过率 100% | P0 | 红线保持 |

### 2.5 可观测性增强 (P1)

| 需求 ID | 需求描述 | 优先级 | 验收标准 |
|---|---|---|---|
| F-OBS-001 | 监控指标扩展至 25 个 | P1 | Prometheus 配置完成 |
| F-OBS-002 | 分布式追踪全链路覆盖 | P1 | Trace 覆盖率 100% |
| F-OBS-003 | gate-report schema 扩展至 60+ 字段 | P1 | schema 校验 100% 通过 |
| F-OBS-004 | 日志采集完整性 | P1 | 审计日志 100% 完整 |

### 2.6 质量提升 (P1)

| 需求 ID | 需求描述 | 优先级 | 验收标准 |
|---|---|---|---|
| F-QUAL-001 | E2E 回归通过率≥99% | P1 | E2E 报告验证 |
| F-QUAL-002 | 32 个边界场景 100% 修复 | P1 | 回归验证通过 |
| F-QUAL-003 | 重放一致率≥99.95% | P0 | 一致性报告验证 |
| F-QUAL-004 | 单测覆盖率≥97% | P2 | 覆盖率报告验证 |

---

## 3. Phase 2 非功能需求

### 3.1 性能需求

| 指标 | Phase 1 基线 | Phase 2 目标 | 测量方法 |
|---|---|---|---|
| P99 执行时延 | 423ms | **<300ms** | Prometheus + 压测 |
| P99 验证时延 | 467ms | **<300ms** | Prometheus + 压测 |
| 吞吐量 | 基准值 | ≥基准值 120% | 压测 |
| 阻断开销 | 8.5% | **<5%** | 对比测试 |

### 3.2 可靠性需求

| 指标 | Phase 1 基线 | Phase 2 目标 | 测量方法 |
|---|---|---|---|
| 重放一致率 | 99.94% | ≥99.95% | Verifier 比对 |
| 72h 稳定性 | 零故障 | 零故障 | 连续运行测试 |
| 回滚成功率 | 100% | 100% | 回滚演练 |
| 回滚耗时 | 2 分 58 秒 | <5 分钟 | 回滚演练 |

### 3.3 安全需求

| 指标 | Phase 1 基线 | Phase 2 目标 | 测量方法 |
|---|---|---|---|
| 未验证提交率 | 0% | 0% (红线) | Security 审计 |
| SG-1~SG-4 通过率 | 100% | 100% | Security 验证 |
| 扫描器误报率 | 3.2% | **<2%** | 对抗测试 |
| 高风险项 | 0 | 0 | 风险台账 |

### 3.4 可观测性需求

| 指标 | Phase 1 基线 | Phase 2 目标 | 测量方法 |
|---|---|---|---|
| 监控指标数量 | 15 个 | **25 个** | Prometheus 配置 |
| Trace 覆盖率 | 部分 | **100%** | 分布式追踪系统 |
| 日志完整性 | 100% | 100% | 审计日志检查 |
| gate-report 字段 | 47 个 | **60+ 个** | schema 校验 |

---

## 4. Phase 2 接口契约

### 4.1 新增接口 (Phase 2)

```rust
/// Batch 执行请求 (Phase 2 新增)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecuteRequest {
    pub trace_id: String,
    pub batch_id: String,
    pub instructions: Vec<ExecuteRequest>,
    pub atomic: bool,  // 原子性保证
    pub timestamp: String,
}

/// Batch 执行结果 (Phase 2 新增)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecuteResult {
    pub trace_id: String,
    pub batch_id: String,
    pub status: BatchStatus,
    pub results: Vec<ExecutionResult>,
    pub batch_hash: String,
    pub timestamp: String,
}

/// Transaction 执行请求 (Phase 2 新增)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionExecuteRequest {
    pub trace_id: String,
    pub transaction_id: String,
    pub isolation_level: IsolationLevel,
    pub instructions: Vec<ExecuteRequest>,
    pub timeout_ms: i64,
    pub timestamp: String,
}

/// Transaction 执行结果 (Phase 2 新增)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionExecuteResult {
    pub trace_id: String,
    pub transaction_id: String,
    pub status: TransactionStatus,
    pub results: Vec<ExecutionResult>,
    pub transaction_hash: String,
    pub timestamp: String,
}
```

### 4.2 gRPC 服务扩展 (Phase 2)

```protobuf
// batch.proto (Phase 2 新增)
service BatchService {
  rpc BatchExecute(BatchExecuteRequest) returns (BatchExecuteResult);
}

// transaction.proto (Phase 2 新增)
service TransactionService {
  rpc TransactionExecute(TransactionExecuteRequest) returns (TransactionExecuteResult);
  rpc BeginTransaction(BeginTransactionRequest) returns (BeginTransactionResponse);
  rpc CommitTransaction(CommitTransactionRequest) returns (CommitTransactionResponse);
  rpc RollbackTransaction(RollbackTransactionRequest) returns (RollbackTransactionResponse);
}
```

### 4.3 向后兼容性保证

| 接口 | Phase 1 状态 | Phase 2 策略 |
|---|---|---|
| ExecuteRequest/Result | 冻结 | 完全向后兼容 |
| VerifyRequest/Response | 冻结 | 完全向后兼容 |
| CommitRequest/Response | 冻结 | 完全向后兼容 |
| gRPC ExecutorService | 冻结 | 完全向后兼容 |
| gRPC VerifierService | 冻结 | 完全向后兼容 |

---

## 5. Phase 2 验收标准

### 5.1 Phase 2 Exit Gate (15 项)

| 指标 ID | 指标名称 | 目标值 | 验证方法 | 责任人 |
|---|---|---|---|---|
| M-01 | 重放一致率 | ≥99.95% | Verifier 比对 | Dev+QA |
| M-02 | 未验证提交率 | =0 | Security 审计 | Security |
| M-03 | E2E 回归通过率 | ≥99% | E2E 测试 | QA |
| M-04 | P99 执行时延 | <300ms | 性能压测 | SRE |
| M-05 | P99 验证时延 | <300ms | 性能压测 | SRE |
| M-06 | 回滚演练耗时 | <5 分钟 | 回滚演练 | SRE |
| M-07 | gate-report schema | 100% (60+ 字段) | schema 校验 | 观测工程师 |
| M-08 | SG-1~SG-4 验证 | 100% | Security 验证 | Security |
| M-09 | 72h 稳定性 | 零故障 | 连续运行 | SRE+Dev |
| M-10 | 监控指标接入 | 25 个 (100%) | Prometheus 配置 | SRE |
| M-11 | 扫描器误报率 | <2% | 对抗测试 | Dev+Security |
| M-12 | Batch 指令支持 | 100% 完成 | 功能测试 | Dev+QA |
| M-13 | Transaction 指令支持 | 100% 完成 | 功能测试 | Dev+QA |
| M-14 | 32 个边界场景修复 | 100% 修复 | 回归测试 | QA |
| M-15 | 风险收敛率 | ≥75% | 风险台账 | PM+Security |

### 5.2 放行决策 (Phase 2 Exit)

| 决策条件 | 目标 | 决策类型 |
|---|---|---|
| 15 项 Exit Gate 指标 | 全部达标 | Go |
| 15 项 Exit Gate 指标 | 13-14 项达标 | Conditional Go |
| 15 项 Exit Gate 指标 | ≤12 项达标 | No-Go |

---

## 6. Phase 2 时间规划

### 6.1 周度里程碑

| 周次 | 主题 | 关键里程碑 | 交付物 |
|---|---|---|---|
| Week 1 | 范围冻结与设计 | Entry Gate 评审 | PRD v2, ADR v4, 风险台账 v1 |
| Week 2 | Batch 指令开发 | Batch 指令完成 | Batch 指令代码 + 单测 |
| Week 3 | Transaction 指令开发 | Transaction 指令完成 | Transaction 指令代码 + 单测 |
| Week 4 | 性能优化 | P99<300ms 验证 | 性能优化报告 |
| Week 5 | 集成回归 | E2E≥99% 验证 | E2E 报告 v2 |
| Week 6 | Exit Gate | Phase 2 放行决策 | GATE-REPORT v2 |

### 6.2 关键路径

```
Week 1: PRD/ADR 评审 → Week 2-3: 功能开发 → Week 4: 性能优化 → Week 5: 回归测试 → Week 6: Exit Gate
```

---

## 7. Phase 2 风险台账

### 7.1 风险清单 (初始 8 项)

| 风险 ID | 风险描述 | 影响等级 | 缓解计划 | 责任人 |
|---|---|---|---|---|
| R-PH2-001 | Phase 2 功能范围蔓延 | 中 | 范围冻结 + Gate 评审 | PM |
| R-PH2-002 | 生产发布窗口冲突 | 中 | 提前协调 + 备用窗口 | SRE |
| R-PH2-003 | 性能基线漂移 | 中 | 持续监控 + 自动告警 | SRE |
| R-PH2-004 | 32 个边界场景修复复杂度 | 低 | 分批次修复 + 回归验证 | QA+Dev |
| R-P2-W1-001 | Batch/Transaction 语义定义分歧 | 中 | 早期对齐 + 示例用例 | PM+Dev |
| R-P2-W1-002 | 性能优化技术难度 | 中 | 技术预研 + PoC 验证 | Dev+SRE |
| R-P2-W1-003 | 零信任架构集成复杂度 | 中 | 分阶段接入 + 灰度验证 | Security |
| R-P2-W1-004 | 监控指标扩展工作量 | 低 | 优先级排序 + 迭代接入 | SRE |

### 7.2 风险统计

```
风险总数：8 项
高风险：0
中风险：5 项
低风险：3 项
风险收敛目标：≥75% (Phase 2 Exit)
```

---

## 8. 附录

### 8.1 术语表

| 术语 | 定义 |
|---|---|
| Batch 指令 | 批量执行多条指令，原子性保证 |
| Transaction 指令 | 事务语义指令，支持 BEGIN/COMMIT/ROLLBACK |
| 零信任架构 | Never Trust, Always Verify 安全架构 |
| 分布式追踪 | 全链路请求追踪 (Trace/Span) |

### 8.2 参考文档

- Phase 1 PRD v1
- Phase 1 ADR v1-v3
- Phase 1 关闭报告
- Phase 2 Execution Board v1

---

**文档状态**: 📋 草案评审中  
**下次更新**: Week 1-T3 (PRD v2 定稿)  
**责任人**: PM
