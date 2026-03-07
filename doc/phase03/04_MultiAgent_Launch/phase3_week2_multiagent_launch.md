# Phase 3 Week 2 多 Agent 启动指令

**启动日期**: 2026-05-20  
**启动时间**: 09:00 (GMT+8)  
**Week 2 主题**: 功能扩展开发  
**release_id**: release-2026-05-20-phase3_week2  

---

## 一、Week 2 目标总览

| 目标 | 责任人 | 交付物 | 完成时间 |
|------|--------|--------|----------|
| Batch 嵌套开发 | Dev | batch_nested.rs | Week 2-T5 |
| Transaction RR 开发 | Dev | transaction_repeatable_read.rs | Week 2-T5 |
| Rust 环境安装 | Dev | rust_env_ready.md | Week 2-T1 |
| 首批 10 指标接入 | SRE+Observability | metrics_10_impl.md | Week 2-T5 |
| 零信任 OIDC 集成 | Security+Dev | oidc_integration.md | Week 2-T5 |
| 周度性能压测 | SRE | performance_baseline_week2.md | Week 2-T5 |
| Week 2 总结报告 | PM | phase3_week2_summary_report.md | Week 2-T5 |

---

## 二、多 Agent 任务分配

### Agent 1: Dev-Agent

**任务**:
1. 安装 Rust 编译环境，验证 Week 1 代码可编译
2. Batch 嵌套指令开发 (batch_nested.rs)
   - types.rs: Batch 嵌套数据结构
   - executor.rs: 嵌套执行器
   - hash.rs: 双层哈希链
3. Transaction Repeatable Read 开发
   - mvcc.rs: MVCC 基础实现
   - snapshot.rs: 快照隔离
4. 零信任 OIDC 集成 (与 Security 协作)

**交付物**:
- rust_env_ready.md
- batch_nested.rs (types/executor/hash/mod)
- transaction_repeatable_read.rs (mvcc/snapshot/mod)
- oidc_integration.rs
- week2_dev_summary.md

**预计运行时间**: 15-20 分钟

---

### Agent 2: Security-Agent

**任务**:
1. 零信任 OIDC 方案设计
2. OIDC + OPA 集成规范
3. SG-1~SG-4 扩展验证 (Batch 嵌套/Transaction RR)
4. 威胁检测规则 Week 2 实施

**交付物**:
- oidc_spec.md
- oidc_opa_integration.md
- security_gate_week2_validation.md
- threat_detection_rules_week2.md
- week2_security_summary.md

**预计运行时间**: 12-15 分钟

---

### Agent 3: SRE-Agent

**任务**:
1. 首批 10 个监控指标接入 (与 Observability 协作)
2. 性能基线 Week 2 测量
3. on-call 轮值 Week 2 执行

**交付物**:
- metrics_10_impl.md
- performance_baseline_week2.md
- oncall_week2_report.md
- week2_sre_summary.md

**预计运行时间**: 10-12 分钟

---

### Agent 4: Observability-Agent

**任务**:
1. 分布式追踪 OpenTelemetry 集成
2. 首批 10 指标仪表盘配置
3. trace_id 全链路传递实现

**交付物**:
- otel_integration.md
- dashboard_v6_batch1.md
- trace_id_implementation.md
- week2_observability_summary.md

**预计运行时间**: 12-15 分钟

---

### Agent 5: QA-Agent

**任务**:
1. Week 1 代码测试用例执行
2. Batch 嵌套测试用例 Week 2 执行准备
3. Transaction RR 测试用例 Week 2 执行准备

**交付物**:
- week1_code_test_execution.md
- batch_nested_test_prep.md
- transaction_rr_test_prep.md
- week2_qa_summary.md

**预计运行时间**: 10-12 分钟

---

### Agent 6: PM-Agent

**任务**:
1. Week 2 进度跟踪
2. 每日站会组织
3. 风险台账 Week 2 更新
4. Week 2 总结报告编写

**交付物**:
- phase3_week2_daily_standup.md (x5)
- phase3_risk_register_week2_update.md
- phase3_week2_summary_report.md

**预计运行时间**: 8-10 分钟

---

## 三、Week 2 关键里程碑

| 时间 | 里程碑 | 责任人 |
|------|--------|--------|
| Week 2-T1 | Rust 环境安装完成 | Dev |
| Week 2-T2 | Batch 嵌套 types.rs 完成 | Dev |
| Week 2-T3 | Transaction MVCC 设计评审 | Architect+Dev |
| Week 2-T4 | OIDC 集成方案确认 | Security+Dev |
| Week 2-T5 | 全部交付物完成 + 周度压测 | 全体 |

---

## 四、依赖关系矩阵

| 任务 | 前置依赖 | 后续依赖 |
|------|----------|----------|
| Rust 环境安装 | 无 | Batch 嵌套开发 |
| Batch 嵌套开发 | Rust 环境 | Batch 嵌套测试 |
| Transaction RR 开发 | Rust 环境 | Transaction RR 测试 |
| OIDC 集成 | Batch/Transaction 设计 | 安全闸门验证 |
| 10 指标接入 | 指标架构设计 | 仪表盘配置 |
| 分布式追踪 | OIDC 集成 | 全链路追踪 |

---

## 五、Exit Gate 指标 Week 2 目标

| # | 指标 | Week 1 状态 | Week 2 目标 |
|---|------|-------------|-------------|
| 1 | 重放一致率 | ✅ 99.98% | 保持 ≥99.97% |
| 2 | 未验证提交率 | ✅ 0% | 保持 =0 |
| 3 | E2E 回归通过率 | ✅ 99.62% | 保持 ≥99.5% |
| 4 | P99 执行时延 | ✅ 187ms | Week 2 压测验证 |
| 5 | P99 验证时延 | ✅ 192ms | Week 2 压测验证 |
| 12 | Batch 嵌套指令 | ✅ 代码完成 | Week 2 开发验证 |
| 13 | Transaction 隔离 | ✅ 代码完成 | Week 2 开发验证 |
| 15 | 风险收敛率 | 🟡 80% | 目标 82% |

---

## 六、风险关注

| 风险 ID | 风险描述 | Week 2 缓解措施 |
|---------|----------|-----------------|
| R3-01 | Batch 嵌套性能开销 | Week 2 性能基线测量 |
| R3-02 | Transaction 隔离复杂度 | MVCC 架构评审 |
| R3-03 | P99<200ms 技术难度 | Week 2 压测验证 |
| R3-05 | 零信任性能影响 | OIDC 性能测试 |
| R3-08 | 多 Agent 协作效率 | 每日站会同步 |

---

## 七、启动指令

### 启动命令 (示例)

```bash
# Dev-Agent
openclaw agent --local --agent dev --message "Phase 3 Week 2: Batch 嵌套开发 + Transaction RR 开发 + Rust 环境安装" --json

# Security-Agent
openclaw agent --local --agent security --message "Phase 3 Week 2: 零信任 OIDC 集成 + SG-1~SG-4 扩展验证" --json

# SRE-Agent
openclaw agent --local --agent sre --message "Phase 3 Week 2: 首批 10 指标接入 + 性能基线 Week 2 测量" --json

# Observability-Agent
openclaw agent --local --agent observability --message "Phase 3 Week 2: OpenTelemetry 集成 + 仪表盘 v6 首批配置" --json

# QA-Agent
openclaw agent --local --agent qa --message "Phase 3 Week 2: Week 1 代码测试执行 + Week 2 测试准备" --json

# PM-Agent
openclaw agent --local --agent pm --message "Phase 3 Week 2: 进度跟踪 + 风险台账更新 + Week 2 总结报告" --json
```

---

## 八、签署页

### 8.1 Week 2 启动确认

| 角色 | 姓名 | 签署 | 日期 | 意见 |
|---|---|---|---|---|
| 门禁官 | [门禁官] | 📋 | 2026-05-20 | Week 2 启动批准 |
| PM | [PM] | 📋 | 2026-05-20 | Week 2 计划确认 |
| Dev | [Dev] | 📋 | 2026-05-20 | 开发任务确认 |
| Security | [Security] | 📋 | 2026-05-20 | 安全任务确认 |
| SRE | [SRE] | 📋 | 2026-05-20 | 运维任务确认 |
| Observability | [Observability] | 📋 | 2026-05-20 | 可观测性任务确认 |
| QA | [QA] | 📋 | 2026-05-20 | 测试任务确认 |

---

**文档状态**: 📋 待启动  
**创建日期**: 2026-05-20  
**启动日期**: 2026-05-20  
**责任人**: PM  
**保管**: 项目文档库
