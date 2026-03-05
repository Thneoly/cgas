# Phase 2 Week 1 执行看板

**版本**: v1.0 (Phase 2 Kickoff)  
**日期**: 2026-03-31  
**责任人**: PM + 各角色负责人  
**状态**: 🟡 Phase 2 启动中  
**release_id**: release-2026-03-31-phase2_week01  
**参与角色**: PM, Dev, QA, SRE, Security, 观测工程师

---

## 1. Phase 2 启动确认

### 1.1 Phase 1→Phase 2 交接状态

| 交接项 | Phase 1 状态 | Phase 2 继承 | 确认方 |
|---|---|---|---|
| Phase 1 Exit Gate | ✅ 正式关闭 | - | 四方联签 |
| 重放一致率 | 99.94% | ≥99.95% 目标 | Dev+QA |
| E2E 回归通过率 | 98.7% | ≥99% 目标 | QA |
| P99 执行时延 | 423ms | <300ms 目标 | SRE+Dev |
| P99 验证时延 | 467ms | <300ms 目标 | SRE+Dev |
| 监控指标 | 15 个 | 25 个目标 | SRE |
| 扫描器误报率 | 3.2% | <2% 目标 | Dev+Security |
| 风险台账 | 15 项关闭 11 项 | 继承 4 项 + 新增 | PM+Security |

### 1.2 Phase 2 范围冻结 (Week 1 核心任务)

| 任务 ID | 任务描述 | 责任人 | 优先级 | 预计完成 | 状态 |
|---|---|---|---|---|---|
| W2-T1 | Phase 2 PRD v2 编写 | PM | P0 | Week 1-T3 | 📋 待开始 |
| W2-T2 | Phase 2 ADR v4 架构设计 | Dev+ 架构师 | P0 | Week 1-T4 | 📋 待开始 |
| W2-T3 | Phase 2 风险台账 v1 | PM+Security | P1 | Week 1-T3 | 📋 待开始 |
| W2-T4 | Phase 2 测试矩阵 v1 | QA | P1 | Week 1-T4 | 📋 待开始 |
| W2-T5 | Phase 2 运维规划 v1 | SRE | P1 | Week 1-T4 | 📋 待开始 |
| W2-T6 | Phase 2 安全规划 v1 | Security | P1 | Week 1-T4 | 📋 待开始 |
| W2-T7 | Phase 2 Week 1 Entry Gate 评审 | 门禁官 | P0 | Week 1-T5 | 📋 待开始 |

---

## 2. Phase 2 核心目标 (6 周周期)

### 2.1 Phase 2 周度规划

| 周次 | 主题 | 关键产出 | Gate 映射 |
|---|---|---|---|
| **Week 1** | Phase 2 范围冻结与设计 | PRD v2, ADR v4, 风险台账 v1 | Phase 2 Entry Gate |
| **Week 2-3** | 功能扩展开发 | Batch/Transaction 指令支持 | 功能评审 |
| **Week 4** | 性能优化 | P99<300ms 性能报告 | 性能 Gate |
| **Week 5** | 集成回归与验证 | E2E 报告 v2 (≥99%) | 回归 Gate |
| **Week 6** | Phase 2 Exit Gate | GATE-REPORT v2, 生产发布决策 | Phase 2 Exit |

### 2.2 Phase 2 Exit Gate 目标 (15 项)

| 指标类别 | 指标名称 | Phase 1 实际 | Phase 2 目标 | 状态 |
|---|---|---|---|---|
| 一致性 | 重放一致率 | 99.94% | ≥99.95% | 📋 规划中 |
| 回归测试 | E2E 回归通过率 | 98.7% | ≥99% | 📋 规划中 |
| 性能 | P99 执行时延 | 423ms | **<300ms** | 📋 规划中 |
| 性能 | P99 验证时延 | 467ms | **<300ms** | 📋 规划中 |
| 运维 | 回滚演练耗时 | 2 分 58 秒 | <5 分钟 (保持) | 📋 规划中 |
| 安全 | 未验证提交率 | 0% | 0% (红线保持) | 📋 规划中 |
| 安全 | SG-1~SG-4 验证 | 100% | 100% (保持) | 📋 规划中 |
| 安全 | 扫描器误报率 | 3.2% | **<2%** | 📋 规划中 |
| 数据质量 | gate-report schema | 47 字段 | 扩展至 60+ 字段 | 📋 规划中 |
| 稳定性 | 72 小时连续运行 | 零故障 | 零故障 (保持) | 📋 规划中 |
| 监控 | 核心指标接入 | 15 个 | **25 个** | 📋 规划中 |
| 功能 | Batch 指令支持 | 无 | 100% 完成 | 📋 规划中 |
| 功能 | Transaction 指令支持 | 无 | 100% 完成 | 📋 规划中 |
| 修复 | 32 个边界场景 | 未修复 | 100% 修复 | 📋 规划中 |
| 风险 | 风险收敛率 | 73.3% | ≥75% | 📋 规划中 |

---

## 3. Phase 2 优化 Backlog (来自 Phase 1)

### 3.1 Phase 1 遗留优化项

| 优化项 ID | 优化项描述 | 来源 | 优先级 | 责任人 | 计划周次 |
|---|---|---|---|---|---|
| OPT-001 | 32 个非核心失败用例修复 | E2E 回归 | P1 | Dev+QA | Week 2-3 |
| OPT-002 | 阻断性能优化 (<5%) | 性能基线 | P1 | SRE+Dev | Week 4 |
| OPT-003 | 扫描器误报率优化 (<2%) | 扫描器报告 | P2 | Dev+Security | Week 4 |
| OPT-004 | P99 时延优化 (<300ms) | 性能基线 | P0 | SRE+Dev | Week 4 |
| OPT-005 | 生产影子验证完善 | 放行决策 | P1 | SRE+Security | Week 5 |
| OPT-006 | 分布式追踪全链路覆盖 | 可观测性 | P1 | SRE | Week 2-3 |
| OPT-007 | 零信任架构接入 | 安全增强 | P1 | Security | Week 2-3 |

### 3.2 Phase 2 新增功能

| 功能 ID | 功能描述 | 优先级 | 责任人 | 计划周次 |
|---|---|---|---|---|
| FUNC-001 | Batch 批量指令支持 | P0 | Dev | Week 2-3 |
| FUNC-002 | Transaction 事务指令支持 | P0 | Dev | Week 2-3 |
| FUNC-003 | 监控指标扩展 (15→25) | P1 | SRE | Week 2-3 |
| FUNC-004 | gate-report schema 扩展 | P1 | 观测工程师 | Week 4 |

---

## 4. Phase 2 风险台账 v1 (初始)

### 4.1 Phase 1 继承风险 (4 项)

| 风险 ID | 风险描述 | 影响等级 | Phase 2 缓解计划 | 责任人 |
|---|---|---|---|---|
| R-PH2-001 | Phase 2 功能范围蔓延 | 中 | 范围冻结 + Gate 评审 | PM |
| R-PH2-002 | 生产发布窗口冲突 | 中 | 提前协调 + 备用窗口 | SRE |
| R-PH2-003 | 性能基线漂移 | 中 | 持续监控 + 自动告警 | SRE |
| R-PH2-004 | 32 个边界场景修复复杂度 | 低 | 分批次修复 + 回归验证 | QA+Dev |

### 4.2 Phase 2 新增风险 (预估)

| 风险 ID | 风险描述 | 影响等级 | 缓解计划 | 责任人 |
|---|---|---|---|---|
| R-P2-W1-001 | Batch/Transaction 语义定义分歧 | 中 | 早期对齐 + 示例用例 | PM+Dev |
| R-P2-W1-002 | 性能优化技术难度 | 中 | 技术预研 + PoC 验证 | Dev+SRE |
| R-P2-W1-003 | 零信任架构集成复杂度 | 中 | 分阶段接入 + 灰度验证 | Security |
| R-P2-W1-004 | 监控指标扩展工作量 | 低 | 优先级排序 + 迭代接入 | SRE |

### 4.3 风险统计 (Phase 2 初始)

```
风险总数：8 项 (继承 4 项 + 新增 4 项)
高风险：0
中风险：5 项
低风险：3 项
风险收敛目标：≥75% (Phase 2 Exit)
```

---

## 5. Phase 2 Week 1 详细任务

### 5.1 PM 任务

| 任务 ID | 任务描述 | 交付物 | 优先级 | 状态 |
|---|---|---|---|---|
| W1-T-PM-01 | Phase 2 范围定义 | PRD v2 初稿 | P0 | 📋 待开始 |
| W1-T-PM-02 | 利益相关者对齐 | 会议纪要 | P0 | 📋 待开始 |
| W1-T-PM-03 | Phase 2 风险台账 v1 | 风险台账文档 | P1 | 📋 待开始 |
| W1-T-PM-04 | Week 1 Entry Gate 准备 | Gate 材料包 | P0 | 📋 待开始 |

### 5.2 Dev 任务

| 任务 ID | 任务描述 | 交付物 | 优先级 | 状态 |
|---|---|---|---|---|
| W1-T-DEV-01 | Batch 指令语义设计 | 设计文档 | P0 | 📋 待开始 |
| W1-T-DEV-02 | Transaction 指令语义设计 | 设计文档 | P0 | 📋 待开始 |
| W1-T-DEV-03 | ADR v4 架构设计 | ADR v4 文档 | P0 | 📋 待开始 |
| W1-T-DEV-04 | 性能优化技术方案 | 技术方案文档 | P1 | 📋 待开始 |

### 5.3 QA 任务

| 任务 ID | 任务描述 | 交付物 | 优先级 | 状态 |
|---|---|---|---|---|
| W1-T-QA-01 | Phase 2 测试策略 | 测试策略文档 | P1 | 📋 待开始 |
| W1-T-QA-02 | Phase 2 测试矩阵 v1 | TEST-MATRIX v1 | P1 | 📋 待开始 |
| W1-T-QA-03 | 32 个边界场景分析 | 场景清单 | P2 | 📋 待开始 |

### 5.4 SRE 任务

| 任务 ID | 任务描述 | 交付物 | 优先级 | 状态 |
|---|---|---|---|---|
| W1-T-SRE-01 | Phase 2 监控指标规划 | 监控指标清单 | P1 | 📋 待开始 |
| W1-T-SRE-02 | 性能基线优化方案 | 性能优化方案 | P1 | 📋 待开始 |
| W1-T-SRE-03 | prod 影子验证监控 | 监控配置 | P0 | 🟡 进行中 |

### 5.5 Security 任务

| 任务 ID | 任务描述 | 交付物 | 优先级 | 状态 |
|---|---|---|---|---|
| W1-T-SEC-01 | 零信任架构方案 | 安全方案文档 | P1 | 📋 待开始 |
| W1-T-SEC-02 | Phase 2 安全规划 v1 | 安全规划文档 | P1 | 📋 待开始 |
| W1-T-SEC-03 | 扫描器规则优化方案 | 优化方案文档 | P2 | 📋 待开始 |

### 5.6 观测工程师任务

| 任务 ID | 任务描述 | 交付物 | 优先级 | 状态 |
|---|---|---|---|---|
| W1-T-OBS-01 | gate-report schema v2 设计 | schema 设计文档 | P1 | 📋 待开始 |
| W1-T-OBS-02 | Phase 2 证据包规划 | 证据包清单 | P1 | 📋 待开始 |

---

## 6. Phase 2 Week 1 Entry Gate 检查清单

### 6.1 Entry Gate 条件

| 条件 ID | 检查项 | 目标 | 状态 | 验证方 |
|---|---|---|---|---|
| EG-001 | PRD v2 初稿完成 | 完成 | 📋 待验证 | PM |
| EG-002 | ADR v4 初稿完成 | 完成 | 📋 待验证 | Dev |
| EG-003 | 风险台账 v1 完成 | 完成 | 📋 待验证 | PM+Security |
| EG-004 | 测试矩阵 v1 完成 | 完成 | 📋 待验证 | QA |
| EG-005 | 运维规划 v1 完成 | 完成 | 📋 待验证 | SRE |
| EG-006 | 安全规划 v1 完成 | 完成 | 📋 待验证 | Security |
| EG-007 | 四方联签确认 | 完成 | 📋 待验证 | PM/Dev/QA/SRE/Security |

### 6.2 Entry Gate 时间安排

| 时间点 | 事件 | 责任人 |
|---|---|---|
| Week 1-T3 | PRD v2 初稿完成 | PM |
| Week 1-T4 | ADR v4 初稿完成 | Dev |
| Week 1-T4 | 所有规划文档完成 | 各角色 |
| Week 1-T5 | Entry Gate 评审会议 | 门禁官 + 四方 |

---

## 7. Phase 2 接口契约扩展规划

### 7.1 Phase 1 冻结接口 (向后兼容)

| 接口 | Phase 1 状态 | Phase 2 策略 |
|---|---|---|
| ExecuteRequest/Result | Week 2 冻结 | 向后兼容，扩展 Batch/Transaction |
| VerifyRequest/Response | Week 3 冻结 | 向后兼容 |
| CommitRequest/Response | Week 4 冻结 | 向后兼容 |
| gRPC 服务定义 | Week 4 冻结 | 向后兼容，新增 BatchVerify |

### 7.2 Phase 2 新增接口

| 接口类型 | 接口名称 | 描述 | 计划周次 |
|---|---|---|---|
| Instruction | BatchExecute | 批量指令执行 | Week 2 |
| Instruction | TransactionExecute | 事务指令执行 | Week 2 |
| gRPC | BatchService | 批量服务 | Week 3 |
| gRPC | TransactionService | 事务服务 | Week 3 |

---

## 8. Phase 2 沟通与协作

### 8.1 站会安排

| 会议 | 频率 | 时间 | 参与方 |
|---|---|---|---|
| Phase 2 每日站会 | 每日 | 09:30 | 全体角色 |
| Phase 2 周度评审 | 每周五 | 15:00 | 全体角色 + 门禁官 |
| Phase 2 风险评审 | 双周 | 周四 | PM+Security+ 各角色 |

### 8.2 沟通渠道

| 渠道 | 用途 | 参与方 |
|---|---|---|
| Phase 2 项目群 | 日常沟通 | 全体角色 |
| Phase 2 文档库 | 文档协作 | 全体角色 |
| Phase 2 问题跟踪 | 问题管理 | 全体角色 |

---

## 9. 附录：Phase 2 参考文档

### 9.1 Phase 1 交付物 (继承)

| 文档 | 版本 | 路径 |
|---|---|---|
| PRD v1 | v1 | phase1_week1_prd_v1.md |
| ADR v1-v3 | v1-v3 | phase1_week*_adr_v*.md |
| 风险台账 v5 | v5 | phase1_week1_risk_register_v1.md + 周度更新 |
| 测试矩阵 v1 | v1 | phase1_week1_test_matrix_v1.md |
| GATE-REPORT v1 | v1 | runtime_artifacts/phase1_week06/ |
| Phase 1 关闭报告 | v1.1 | phase1_week6_closeout_report.md |

### 9.2 Phase 2 新文档 (待创建)

| 文档 | 版本 | 计划完成 | 责任人 |
|---|---|---|---|
| PRD v2 | v2 | Week 1-T3 | PM |
| ADR v4 | v4 | Week 1-T4 | Dev |
| 风险台账 v1 | v1 | Week 1-T3 | PM+Security |
| 测试矩阵 v1 | v1 | Week 1-T4 | QA |
| SRE 规划 v1 | v1 | Week 1-T4 | SRE |
| 安全规划 v1 | v1 | Week 1-T4 | Security |

---

## 10. Phase 2 启动确认

### 10.1 启动签字

| 角色 | 姓名 | 签署 | 日期 |
|---|---|---|---|
| PM | [PM 姓名] | 📋 待签署 | 2026-03-31 |
| Dev | [Dev 姓名] | 📋 待签署 | 2026-03-31 |
| QA | [QA 姓名] | 📋 待签署 | 2026-03-31 |
| SRE | [SRE 姓名] | 📋 待签署 | 2026-03-31 |
| Security | [Security 姓名] | 📋 待签署 | 2026-03-31 |

### 10.2 Phase 2 状态

| 状态项 | 值 |
|---|---|
| Phase 2 启动日期 | 2026-03-31 |
| Phase 2 预计完成 | 2026-05-12 (6 周) |
| 当前周次 | Week 1 |
| 当前状态 | 🟡 启动中 |
| Entry Gate 状态 | 📋 待评审 |

---

*本文档由 PM 角色生成，作为 Phase 2 Week 1 执行看板。Phase 2 正式启动，Week 1 Entry Gate 评审待安排。*
