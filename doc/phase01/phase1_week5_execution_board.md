# Phase 1 - Week 5 执行看板（集成回归 + 灰度准备）

## 本周目标
- 完成 E2E 全链路回归并收敛高优缺陷
- 完成 staging/pre-prod 灰度前置检查与回滚演练
- 完成 gate-report 预演（json/schema/md）
- 形成 Week 6 正式关卡材料初版

## 周内里程碑
- D1：回归范围冻结与环境基线确认
- D2：E2E 首轮回归 + 缺陷分级
- D3：修复回归 + 性能稳定性验证
- D4：Gate 报告预演 + 预审结论
- D5：Week 6 放行会议材料冻结

## 任务表（责任人 + DoD + Gate 映射）

| 任务ID | 角色 | 任务 | 交付物 | DoD | Gate 映射 |
|---|---|---|---|---|---|
| W5-T1 | 开发（Core/Platform） | 高优缺陷修复与收敛 | `phase1_week5_dev_stabilization.md` | P0/P1 闭环并可复验 | 回归通过率 |
| W5-T2 | QA | E2E 全链路回归 | `phase1_week5_qa_e2e_plan.md` | 核心场景通过率 `>=98%` 路径明确 | Phase1 Exit |
| W5-T3 | SRE | 灰度与回滚准备 | `phase1_week5_sre_gray_readiness.md` | 灰度步骤/回滚步骤可执行 | 可发布性 |
| W5-T4 | 观测工程师 | gate-report 预演 | `phase1_week5_observability_gate_rehearsal.md` | schema 通过率 100% | Gate 材料有效性 |
| W5-T5 | 项目经理 | 材料收敛与评审组织 | `phase1_week5_pmo_gate_pack.md` | Week6 资料齐全可审 | Gate 前置条件 |
| W5-T6 | 门禁官（预审） | Week5 预审结论 | `phase1_week5_gate_precheck.md` | 红线缺口可量化闭环 | Week6 正式判定准备 |

## 周末验收清单
- [ ] E2E 报告完成并标注缺陷优先级
- [ ] P0/P1 缺陷闭环或有批准例外
- [ ] 灰度方案与回滚预案完成演练
- [ ] `gate-report.json` 预演通过 schema
- [ ] Week 6 关卡材料冻结
