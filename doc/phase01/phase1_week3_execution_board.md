# Phase 1 - Week 3 执行看板（重放一致性专项）

## 本周目标
- 完成 Verifier 独立重放链路并与执行链路隔离
- 建立 mismatch 归因机制并形成修复闭环
- 产出一致性报告 v1，支撑 Phase 1 指标推进
- 形成 Week 4 提交阻断与非确定性扫描的输入条件

## 周内里程碑
- D1：重放链路接口冻结（开发/架构）
- D2：Verifier 重放主流程可运行
- D3：mismatch 分类报告首版（开发+QA）
- D4：一致性回放报告 v1（QA+观测）
- D5：周评审 + Week 4 阻断改造开工条件确认

## 任务表（责任人 + DoD + Gate 映射）

| 任务ID | 角色 | 任务 | 交付物 | DoD | Gate 映射 |
|---|---|---|---|---|---|
| W3-T1 | 开发（Core） | Verifier 独立重放链路 | `phase1_week3_dev_replay_plan.md` | 重放进程与执行进程隔离 | 重放一致率 |
| W3-T2 | 开发+QA | mismatch 归因机制 | `phase1_week3_qa_consistency_plan.md` | 可定位到规则/输入/依赖 | 一致率修复闭环 |
| W3-T3 | QA | 一致性样本批跑 | `phase1_week3_qa_consistency_plan.md` | 样本批跑报告可追溯 | 重放一致率 `>=99.9%` 路径 |
| W3-T4 | 观测工程师 | 一致性看板与证据包 | `phase1_week3_observability_plan.md` | 指标四元组可审计 | Gate 证据准备 |
| W3-T5 | 安全工程师 | 旁路与污染风险审查 | `phase1_week3_security_review.md` | 出具 Week4 阻断输入清单 | 非确定性防护前置 |
| W3-T6 | 门禁官（预审） | Week3 门禁预审 | `phase1_week3_gate_precheck.md` | 识别红线风险与缺口 | Week6 放行前闭环 |

## 周末验收清单
- [ ] Verifier 重放链路与执行链路隔离已验证
- [ ] mismatch 分类与修复责任分配完成
- [ ] 一致性报告 v1 形成并可复核
- [ ] 指标证据四元组具备 Gate 预审质量
- [ ] Week 4 阻断开发任务已就绪
