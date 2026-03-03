# Phase 1 - Week 2 执行看板（开发接力）

## 本周目标
- 完成最小指令集执行语义实现与单测基线
- 落地 state_diff-only 语义并移除直接提交路径
- 补齐 trace/result/state_diff 哈希链与关键日志字段
- 形成可联调的 QA/观测/SRE 接口与报告输入

## 周内里程碑
- D1：开发任务拆分与接口冻结
- D2：执行器核心能力提交（指令语义 + state_diff）
- D3：哈希链与日志字段完成，QA 首轮回归
- D4：联调修复与观测校验
- D5：Week 2 评审与 Week 3 重放开发开工条件确认

## 任务表（责任人 + DoD + Gate 映射）

| 任务ID | 角色 | 任务 | 交付物 | DoD | Gate 映射 |
|---|---|---|---|---|---|
| W2-T1 | 开发（Core） | 最小指令语义实现 | `phase1_week2_dev_delivery.md` | 核心指令单测通过 | Phase1 Exit: 回归基础 |
| W2-T2 | 开发（Core） | state_diff-only 语义落地 | `phase1_week2_dev_delivery.md` | 无直接提交路径 | 红线前提: 未验证提交率=0 |
| W2-T3 | QA | Week 2 回归与边界验证 | `phase1_week2_qa_plan.md` | 首轮回归报告完成 | Exit: 回归通过率 |
| W2-T4 | 观测工程师 | 哈希覆盖与指标口径校验 | `phase1_week2_observability_plan.md` | 字段覆盖与数据质量达标 | Exit: result_hash 覆盖率 |
| W2-T5 | SRE | 联调环境与发布预案预热 | `phase1_week2_sre_plan.md` | staging 联调路径稳定 | Week5 发布准备前置 |

## Week 2 周末验收清单
- [ ] 核心指令语义实现并通过单测
- [ ] 执行阶段仅产出 state_diff，直提路径被关闭
- [ ] trace/result/state_diff 哈希链可追踪
- [ ] QA 首轮回归与缺陷分级完成
- [ ] 观测指标可用于 Week 3 一致性开发输入
