# Phase 2 SRE 规划 v1

**版本**: v1.0 (Phase 2 Kickoff)  
**日期**: 2026-03-31  
**责任人**: SRE  
**状态**: 📋 草案评审中  
**release_id**: release-2026-03-31-phase2_week01  
**参与角色**: PM, Dev, QA, SRE, Security

---

## 1. SRE 目标概述

### 1.1 Phase 2 SRE 核心目标

| 目标 | Phase 1 基线 | Phase 2 目标 | 提升 |
|---|---|---|---|
| P99 执行时延 | 423ms | **<300ms** | -29% |
| P99 验证时延 | 467ms | **<300ms** | -36% |
| 监控指标 | 15 个 | **25 个** | +10 个 |
| 阻断性能开销 | 8.5% | **<5%** | -41% |
| 回滚演练 | 2 分 58 秒 | <5 分钟 (保持) | - |
| 72h 稳定性 | 零故障 | 零故障 (保持) | - |

### 1.2 Phase 2 SRE 关键任务

| 任务类别 | 任务 | 优先级 | 计划周次 |
|---|---|---|---|
| 性能优化 | P99 时延优化至<300ms | P0 | Week 2-4 |
| 监控扩展 | 15 个→25 个指标 | P1 | Week 2-3 |
| 分布式追踪 | 全链路覆盖 | P1 | Week 2-3 |
| 生产影子验证 | 监控支持 | P0 | Week 1-2 |
| 性能压测 | 周度压测 | P0 | Week 2-5 |
| 稳定性测试 | 72 小时连续运行 | P0 | Week 5 |

---

## 2. 性能优化方案

### 2.1 性能瓶颈分析 (Phase 1)

| 组件 | P99 时延 | 占比 | 瓶颈分析 | 优化空间 |
|---|---|---|---|---|
| 执行器 | 187ms | 44% | 单线程处理 | 异步并发 +20% |
| 验证器 | 203ms | 48% | 全量重放 | 增量重放 +15% |
| 阻断中间件 | 23ms | 5% | 全量校验 | 缓存优化 +10% |
| 序列化 | 10ms | 3% | JSON 序列化 | 零拷贝 +5% |
| **总计** | **423ms** | **100%** | | **-29%** |

### 2.2 性能优化措施

| 优化项 | 技术方案 | 预期提升 | 实施周次 | 责任人 |
|---|---|---|---|---|
| 异步并发池 | tokio 运行时 + 任务队列 | -20% | Week 2 | Dev |
| 增量重放 | 仅重放变化 state_diff | -15% | Week 3 | Dev |
| 校验缓存 | 热点数据缓存 | -10% | Week 3 | Dev |
| 对象池复用 | Vec<StateDiffOperation> 池化 | -5% | Week 4 | Dev |
| 序列化优化 | serde + 零拷贝 | -5% | Week 4 | Dev |

### 2.3 性能压测计划

| 周次 | 压测类型 | 样本量 | 目标 | 输出 |
|---|---|---|---|---|
| Week 2 | 基准压测 | 100,000+ | 建立 Phase 2 基线 | 性能报告 v1 |
| Week 3 | 优化后压测 | 500,000+ | 验证异步并发效果 | 性能报告 v2 |
| Week 4 | 全量压测 | 1,000,000+ | 验证 P99<300ms | 性能报告 v3 |
| Week 5 | 稳定性压测 | 72 小时 | 验证稳定性 | 稳定性报告 |

---

## 3. 监控指标扩展

### 3.1 Phase 1 监控指标 (15 个，继承)

| 指标名 | 类型 | P0 告警阈值 | 状态 |
|---|---|---|---|
| gray_release_consistency_rate | Gauge | <99.9% | ✅ 继承 |
| gray_release_unverified_submit_rate | Gauge | >0 | ✅ 继承 |
| execution_latency_p99 | Histogram | >500ms | ✅ 继承 |
| verification_latency_p99 | Histogram | >500ms | ✅ 继承 |
| blocking_middleware_overhead | Histogram | >100ms | ✅ 继承 |
| ... (共 15 个) | | | ✅ 继承 |

### 3.2 Phase 2 新增监控指标 (10 个)

| 指标名 | 类型 | P0 告警阈值 | 来源 | 计划周次 |
|---|---|---|---|---|
| batch_execute_latency_p99 | Histogram | >400ms | Batch 服务 | Week 2 |
| batch_atomicity_violation_count | Counter | >0 | Batch 服务 | Week 2 |
| batch_sub_instruction_count | Histogram | - | Batch 服务 | Week 2 |
| transaction_commit_latency_p99 | Histogram | >400ms | Transaction 服务 | Week 2 |
| transaction_rollback_count | Counter | >10/h | Transaction 服务 | Week 2 |
| transaction_timeout_count | Counter | >5/h | Transaction 服务 | Week 2 |
| zero_trust_auth_failure_count | Counter | >10/h | 零信任模块 | Week 3 |
| zero_trust_policy_violation_count | Counter | >5/h | 零信任模块 | Week 3 |
| instruction_type_distribution | Histogram | - | 执行器 | Week 2 |
| distributed_trace_coverage | Gauge | <95% | 追踪系统 | Week 3 |

### 3.3 监控指标接入计划

| 周次 | 接入指标 | 数量 | 责任人 | 状态 |
|---|---|---|---|---|
| Week 2 | Batch 相关 (3 个) + 业务 (2 个) | 5 个 | SRE | 📋 待开始 |
| Week 3 | Transaction 相关 (3 个) + 零信任 (2 个) | 5 个 | SRE | 📋 待开始 |
| Week 4 | 指标验证 + 告警配置 | 25 个 | SRE | 📋 待开始 |

---

## 4. 分布式追踪方案

### 4.1 追踪架构

```
Phase 2 分布式追踪架构:
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │───▶│   Gateway   │───▶│   Service   │
│  (请求)     │    │ (Trace 生成)│    │  (Span)     │
└─────────────┘    └─────────────┘    └─────────────┘
                        │
                        ▼
                 ┌─────────────┐
                 │  Jaeger/    │
                 │   Tempo     │
                 └─────────────┘
```

### 4.2 Trace 层级设计

```
Trace (trace_id)
├── Span: BatchExecute/TransactionExecute (service: gateway)
│   ├── Span: Executor.execute (service: executor)
│   │   ├── Span: Verifier.verify (service: verifier)
│   │   └── Span: Commit.commit (service: commit)
│   ├── Span: Executor.execute (service: executor)
│   │   └── ...
│   └── ...
```

### 4.3 追踪实施计划

| 周次 | 任务 | 输出 | 状态 |
|---|---|---|---|
| Week 1 | OpenTelemetry SDK 集成设计 | 设计文档 | 📋 待开始 |
| Week 2 | SDK 集成 + Trace 生成 | Trace 覆盖率 50% | 📋 待开始 |
| Week 3 | 全链路覆盖 + Jaeger 部署 | Trace 覆盖率 100% | 📋 待开始 |
| Week 4 | 追踪数据分析 | 追踪报告 | 📋 待开始 |

---

## 5. 生产影子验证支持

### 5.1 影子验证监控

| 监控项 | 指标 | 告警阈值 | 响应 |
|---|---|---|---|
| 影子流量比例 | 流量占比 | <90% | 告警 |
| 影子验证一致率 | 一致率 | <99.9% | P1 告警 |
| 影子验证延迟 | 延迟 | >1s | P2 告警 |
| 只读模式错误 | 错误数 | >0 | P0 告警 |

### 5.2 影子验证 SRE 任务

| 任务 | 优先级 | 计划周次 | 责任人 | 状态 |
|---|---|---|---|---|
| 影子验证监控配置 | P0 | Week 1-T3 | SRE | 📋 待开始 |
| 只读模式部署支持 | P0 | Week 1-T4 | SRE+Dev | 📋 待开始 |
| 影子验证日报 | P1 | Week 2 起 | SRE | 📋 待开始 |

---

## 6. 灰度发布与回滚

### 6.1 Phase 2 灰度阶段

| 阶段 | 环境 | 放量比例 | 放行条件 | 状态 |
|---|---|---|---|---|
| Stage 1 | staging | 10% | 核心指标达标 | ✅ Phase 1 完成 |
| Stage 2 | staging | 50% | 性能稳定 | ✅ Phase 1 完成 |
| Stage 3 | staging | 100% | 全量验证 24h | ✅ Phase 1 完成 |
| Stage 4 | pre-prod | 100% | 影子验证通过 | ✅ Phase 1 批准 |
| Stage 5 | prod | 影子验证/只读 | 生产验证 | 🟡 Phase 2 进行中 |
| Stage 6 | prod | 读写模式 | Phase 2 Exit Gate | 📋 Phase 2 目标 |

### 6.2 Phase 2 回滚触发条件

| 触发条件 | 阈值 | 动作 | 目标耗时 |
|---|---|---|---|
| 一致率 | <99.9% | 立即回滚 | <5 分钟 |
| 未验证提交率 | >0 | 立即回滚 | <5 分钟 |
| P99 时延 | >500ms 持续 5 分钟 | 告警，>30% 回滚 | <5 分钟 |
| 错误率 | >1% 持续 5 分钟 | 告警，>5% 回滚 | <5 分钟 |
| Batch 原子性违反 | >0 | 立即回滚 | <5 分钟 |
| 事务隔离违反 | >0 | 立即回滚 | <5 分钟 |

---

## 7. on-call 与告警

### 7.1 Phase 2 on-call 轮值表

| 周次 | 主班 (P0) | 副班 (P1/P2) | 备份 |
|---|---|---|---|
| Week 1 | SRE-A | SRE-B | SRE-C |
| Week 2 | SRE-B | SRE-C | SRE-A |
| Week 3 | SRE-C | SRE-A | SRE-B |
| Week 4 | SRE-A | SRE-B | SRE-C |
| Week 5 | SRE-B | SRE-C | SRE-A |
| Week 6 | SRE-C | SRE-A | SRE-B |

### 7.2 Phase 2 告警分级

| 级别 | 响应时间 | 升级路径 | 示例 |
|---|---|---|---|
| P0 (严重) | <5 分钟 | SRE→Dev→Security→PM | 未验证提交>0, 一致率<99.9% |
| P1 (高) | <15 分钟 | SRE→Dev | P99>500ms, 错误率>1% |
| P2 (中) | <1 小时 | SRE | 资源使用率>80% |

---

## 8. SRE 周度任务计划

### 8.1 Week 1 任务

| 任务 ID | 任务描述 | 交付物 | 优先级 | 状态 |
|---|---|---|---|---|
| W1-T-SRE-01 | Phase 2 监控指标规划 | 监控指标清单 | P1 | ✅ 完成 |
| W1-T-SRE-02 | 性能优化方案设计 | 性能优化方案 | P1 | ✅ 完成 |
| W1-T-SRE-03 | prod 影子验证监控配置 | 监控配置 | P0 | 📋 待开始 |
| W1-T-SRE-04 | SRE 规划 v1 编写 | 本文档 | P1 | ✅ 完成 |

### 8.2 Week 2 任务

| 任务 ID | 任务描述 | 交付物 | 优先级 | 状态 |
|---|---|---|---|---|
| W2-T-SRE-01 | Batch 监控指标接入 | 5 个指标配置 | P1 | 📋 待开始 |
| W2-T-SRE-02 | OpenTelemetry SDK 集成 | Trace 覆盖率 50% | P1 | 📋 待开始 |
| W2-T-SRE-03 | 周度性能压测 | 性能报告 v1 | P0 | 📋 待开始 |

### 8.3 Week 3-6 任务概览

| 周次 | 重点任务 | 关键产出 |
|---|---|---|
| Week 3 | Transaction 监控 + 分布式追踪 | 10 个新指标，Trace 100% |
| Week 4 | 性能优化验证 | 性能报告 v3 (P99<300ms) |
| Week 5 | E2E 回归 + 稳定性测试 | E2E 报告 v2, 稳定性报告 |
| Week 6 | Exit Gate 证据包 | GATE-REPORT v2 |

---

## 9. 附录

### 9.1 Phase 1 SRE 交付物参考

| 文档 | 版本 | 路径 |
|---|---|---|
| 性能基线报告 v3 | v3 | phase1_week5_*.md |
| DEPLOY-RUNBOOK v1 | v1 | phase1_week5_*.md |
| 稳定性测试报告 | v1 | phase1_week5_*.md |
| 灰度方案 v1 | v1 | phase1_week5_*.md |
| 监控指标配置清单 | v1 | phase1_week6_*.md |

### 9.2 SRE 工具链

| 工具 | 用途 | Phase 2 状态 |
|---|---|---|
| Prometheus | 指标采集 | ✅ 继承 + 扩展 |
| Grafana | 仪表盘 | ✅ 继承 + 扩展 |
| Alertmanager | 告警路由 | ✅ 继承 |
| Jaeger/Tempo | 分布式追踪 | 📋 Week 2 新增 |
| k6/vegeta | 性能压测 | ✅ 继承 |

---

**文档状态**: 📋 草案评审中  
**下次更新**: Week 1-T5 (Entry Gate 评审后)  
**责任人**: SRE  
**评审计划**: Week 1-T5 Entry Gate 评审
