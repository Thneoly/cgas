# Phase 3 多 Agent 启动文档

**版本**: v1.0  
**日期**: 2026-05-12  
**责任人**: PM  
**状态**: ✅ 已启动  
**release_id**: release-2026-05-12-phase3_kickoff  

---

## 1. Phase 3 主题

**Phase 3: 功能扩展与性能深化**

| 类别 | 主题 | 优先级 |
|---|---|---|
| **功能扩展** | 复杂指令支持 (Batch 嵌套/Transaction 隔离增强) | P0 |
| **性能优化** | P99 时延进一步优化 (<200ms) | P0 |
| **可观测性** | 分布式追踪全链路覆盖 (25→50 指标) | P1 |
| **安全增强** | 零信任策略完善/威胁检测 | P1 |
| **质量提升** | E2E 回归通过率≥99.5% | P1 |

---

## 2. 多 Agent 角色与技能

根据 `/home/cc/Desktop/code/AIPro/cgas/doc/agent_prompts/` 定义，Phase 3 激活以下 Agent：

### 2.1 Agent 角色矩阵

| Agent | 核心技能 | Phase 3 职责 |
|---|---|---|
| **PM-Agent** | PMP + PfMP + PgMP | 范围冻结、优先级治理、Gate 指标化决策 |
| **Architect-Agent** | 架构权衡、确定性控制、边界与契约设计 | ADR v5、接口契约扩展、隔离级别设计 |
| **Dev-Agent** | Rust 核心工程、Platform 工程、可验证实现 | 复杂指令实现、性能优化代码 |
| **QA-Agent** | 测试设计、统计抽样、证据链构建、压测与故障演练 | 测试矩阵 v3、E2E≥99.5% 验证 |
| **SRE-Agent** | 平台运维、可观测体系、GitOps、容量规划 | 50 指标扩展、性能基线 v5、监控仪表盘 v6 |
| **Security-Agent** | 威胁建模、权限模型、执行沙箱、密钥治理 | 零信任策略完善、威胁检测 |
| **Observability-Agent** | 指标口径治理、链路追踪、报告自动化 | 分布式追踪全链路、gate-report 自动化 |

### 2.2 专业技能选择依据

根据 `agent_identity_prompts` 定义：

| Agent | 技能文件 | 核心能力 |
|---|---|---|
| PM | `pm.md` | PfMP + PgMP 组合与项目群管理、Gate 指标化治理 |
| Dev | `dev.md` | Rust 核心工程 (所有权/借用/并发/异步/serde)、Platform 工程 (TypeScript/gRPC/REST) |
| QA | `qa.md` | 回放一致性验证、压测与故障演练 (k6/Locust/Chaos) |
| Security | `security.md` | OIDC/OAuth2、RBAC+ABAC、seccomp/apparmor、Vault/KMS |
| SRE | `sre.md` | Linux/Docker/Kubernetes、Prometheus+Grafana、Rust 服务运维诊断 |

---

## 3. Phase 3 时间规划

| 周次 | 主题 | 关键产出 | Gate 映射 |
|---|---|---|---|
| Week 1 | Phase 3 范围冻结与设计 | PRD v3, ADR v5 | Phase 3 Entry Gate |
| Week 2-3 | 功能扩展开发 | Batch 嵌套/Transaction 隔离增强 | 功能评审 |
| Week 4 | 性能优化实施 | P99<200ms | 性能 Gate |
| Week 5 | 可观测性 + 安全增强 | 50 指标/威胁检测 | 可观测性/安全 Gate |
| Week 6 | Phase 3 Exit Gate | GATE-REPORT v3 | Phase 3 Exit |

**启动时间**: 2026-05-12  
**预计完成**: 2026-06-20

---

## 4. 多 Agent 任务分解

### 4.1 Week 1 任务 (范围冻结与设计)

| 任务 | 负责 Agent | 交付物 | 优先级 |
|---|---|---|---|
| Phase 3 PRD v3 编写 | PM | phase3_prd_v3.md | P0 |
| Phase 3 ADR v5 架构设计 | Architect | phase3_adr_v5.md | P0 |
| Phase 3 测试矩阵 v3 | QA | phase3_test_matrix_v3.md | P0 |
| Phase 3 性能基线 v5 规划 | SRE | phase3_performance_plan.md | P0 |
| Phase 3 安全策略规划 | Security | phase3_security_plan.md | P0 |
| Phase 3 可观测性规划 | Observability | phase3_observability_plan.md | P0 |

### 4.2 Week 2-3 任务 (功能扩展开发)

| 任务 | 负责 Agent | 交付物 | 优先级 |
|---|---|---|---|
| Batch 嵌套指令实现 | Dev | batch_nested.rs | P0 |
| Transaction 隔离级别增强 | Dev | transaction_isolation.rs | P0 |
| Batch 嵌套测试 | QA | batch_nested_test.rs | P0 |
| Transaction 隔离测试 | QA | transaction_isolation_test.rs | P0 |

### 4.3 Week 4 任务 (性能优化)

| 任务 | 负责 Agent | 交付物 | 优先级 |
|---|---|---|---|
| P99 性能分析 | SRE | performance_analysis_p99.md | P0 |
| 性能优化实施 | Dev | performance_optimization.rs | P0 |
| 性能基线 v5 测量 | SRE | performance_baseline_v5.md | P0 |

### 4.4 Week 5 任务 (可观测性 + 安全)

| 任务 | 负责 Agent | 交付物 | 优先级 |
|---|---|---|---|
| 50 指标扩展 | SRE+Observability | monitoring_50_metrics.md | P1 |
| 分布式追踪全链路 | Observability | distributed_tracing.md | P1 |
| 零信任策略完善 | Security | zero_trust_enhancement.md | P1 |
| 威胁检测实施 | Security | threat_detection.md | P1 |

### 4.5 Week 6 任务 (Exit Gate)

| 任务 | 负责 Agent | 交付物 | 优先级 |
|---|---|---|---|
| Exit Gate 检查清单 | PM | phase3_exit_gate_checklist.md | P0 |
| GATE-REPORT v3 | PM | phase3_gate_report_v3.md | P0 |
| Phase 3 关闭报告 | PM | phase3_close_report.md | P0 |

---

## 5. Phase 3 Exit Gate 指标

| # | 指标 | Phase 2 实际 | Phase 3 目标 | 验证方 |
|---|---|---|---|---|
| 1 | 重放一致率 | 99.96% | ≥99.97% | Dev+QA+SRE+Security |
| 2 | 未验证提交率 | 0% | =0 | Security |
| 3 | E2E 回归通过率 | 100% | ≥99.5% | QA |
| 4 | P99 执行时延 | 265ms | **<200ms** | SRE |
| 5 | P99 验证时延 | 272ms | **<200ms** | SRE |
| 6 | 回滚演练耗时 | 2 分 58 秒 | <5 分钟 | SRE |
| 7 | gate-report schema | 100% (60 字段) | 100% (80 字段) | Observability |
| 8 | SG-1~SG-4 验证 | 100% | 100% | Security |
| 9 | 72h 稳定性 | 零故障 | 零故障 | SRE+Dev |
| 10 | 监控指标接入 | 25 个 | **50 个** | SRE+Observability |
| 11 | 扫描器误报率 | 1.8% | <1.5% | Security |
| 12 | Batch 嵌套指令 | N/A | 100% | Dev+QA |
| 13 | Transaction 隔离增强 | N/A | 100% | Dev+QA |
| 14 | 边界场景修复 | 100% (32/32) | 100% (新增 20 个) | Dev+QA |
| 15 | 风险收敛率 | 80% | ≥85% | PM+Security |

---

## 6. 多 Agent 协作机制

### 6.1 沟通机制

| 会议 | 时间 | 参与方 | 议程 |
|---|---|---|---|
| 每日站会 | 每日 09:30 | 全体 Agent | 进度同步、问题暴露 |
| 周度评审 | 每周五 15:00 | 全体 Agent | 周度总结、下周计划 |
| 技术评审 | Week 2-T3 | Architect+Dev | ADR v5 评审 |
| Exit Gate 评审 | Week 6-T5 | 全体 + 门禁官 | Phase 3 Exit Gate 决策 |

### 6.2 交付物管理

| 类别 | 存储路径 | 命名规范 |
|---|---|---|
| 代码 | `rust-workflow-engine/src/` | `phase3_*.rs` |
| 测试 | `rust-workflow-engine/tests/` | `phase3_*_test.rs` |
| 文档 | `doc/phase01/` | `phase3_*.md` |
| 配置 | `rust-workflow-engine/config/` | `phase3_*.toml` |

### 6.3 问题升级路径

| 问题等级 | 响应时间 | 升级路径 |
|---|---|---|
| P0 (阻塞) | 立即 | Agent → PM → 门禁官 |
| P1 (高) | <1h | Agent → PM |
| P2 (中) | <4h | Agent 自行解决 |

---

## 7. Agent 启动确认

| Agent | 会话 ID | 任务 | 状态 |
|---|---|---|---|
| PM-Agent | 待创建 | Phase 3 协调、PRD v3、Exit Gate | 📋 待启动 |
| Architect-Agent | 待创建 | ADR v5 架构设计 | 📋 待启动 |
| Dev-Agent | 待创建 | 功能扩展开发、性能优化 | 📋 待启动 |
| QA-Agent | 待创建 | 测试矩阵 v3、E2E≥99.5% | 📋 待启动 |
| SRE-Agent | 待创建 | 50 指标扩展、性能基线 v5 | 📋 待启动 |
| Security-Agent | 待创建 | 零信任策略、威胁检测 | 📋 待启动 |
| Observability-Agent | 待创建 | 分布式追踪、gate-report 自动化 | 📋 待启动 |

---

## 8. Phase 3 启动签署

| 角色 | 姓名 | 签署 | 日期 | 意见 |
|---|---|---|---|---|
| 门禁官 | _________ | 📋 | 2026-05-12 | Phase 3 启动批准 |
| PM | [PM] | 📋 | 2026-05-12 | Phase 3 协调确认 |
| Architect | [Architect] | 📋 | 2026-05-12 | 架构设计确认 |
| Dev | [Dev] | 📋 | 2026-05-12 | 功能开发确认 |
| QA | [QA] | 📋 | 2026-05-12 | 测试验证确认 |
| SRE | [SRE] | 📋 | 2026-05-12 | 运维支持确认 |
| Security | [Security] | 📋 | 2026-05-12 | 安全验证确认 |
| Observability | [Observability] | 📋 | 2026-05-12 | 可观测性确认 |

---

## 9. 附录

### 9.1 参考文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Agent 身份提示词 | `agent_prompts/agent_identity_prompts/` | Agent 角色定义 |
| 角色提示词手册 | `agent_prompts/role_prompt_playbook.md` | 角色技能矩阵 |
| Phase 2 关闭报告 | `phase2_close_report.md` | Phase 2 总结 |
| Phase 2 经验教训 | `phase2_lessons_learned.md` | Phase 3 改进依据 |

### 9.2 联系方式

| Agent | 职责 | 会话 ID |
|---|---|---|
| PM-Agent | 总体协调 | 待创建 |
| Architect-Agent | 架构设计 | 待创建 |
| Dev-Agent | 功能开发 | 待创建 |
| QA-Agent | 测试验证 | 待创建 |
| SRE-Agent | 运维支持 | 待创建 |
| Security-Agent | 安全验证 | 待创建 |
| Observability-Agent | 可观测性 | 待创建 |

---

**文档状态**: ✅ 已启动  
**启动时间**: 2026-05-12  
**责任人**: PM  
**保管**: 项目文档库
