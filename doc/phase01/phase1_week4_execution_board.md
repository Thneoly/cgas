# Phase 1 - Week 4 执行看板（提交硬阻断 + 非确定性扫描）

## 本周目标
- 未验证提交阻断中间件上线并覆盖全部提交路径
- 非确定性扫描器进入阻断模式（时间/随机/未声明外部依赖）
- 完成对抗注入测试并输出风险闭环
- 为 Week 5 集成回归与灰度准备提供稳定输入

## 周内里程碑
- D1：提交链路与旁路清单冻结（开发+安全）
- D2：阻断中间件主路径完成（开发）
- D3：非确定性扫描规则上线（开发+安全）
- D4：QA 对抗测试与缺陷修复（QA+开发）
- D5：Week 4 关卡预审与 Week 5 开工确认（门禁官+项目经理）

## 任务表（责任人 + DoD + Gate 映射）

| 任务ID | 角色 | 任务 | 交付物 | DoD | Gate 映射 |
|---|---|---|---|---|---|
| W4-T1 | 开发（Platform/Core） | 提交硬阻断中间件 | `phase1_week4_commit_blocking_plan.md` | 未验证路径全部拒绝 | `gate_trust_unverified_zero` |
| W4-T2 | 开发+安全 | 非确定性扫描阻断 | `phase1_week4_nondeterminism_scanner_plan.md` | 关键风险类别可识别可阻断 | 重放一致率保障 |
| W4-T3 | QA+安全 | 对抗注入测试 | `phase1_week4_qa_adversarial_plan.md` | 注入样本拦截率达标 | 红线风险前置验证 |
| W4-T4 | SRE | 联调与发布准备评估 | `phase1_week4_sre_readiness.md` | staging 稳定、回滚可演练 | Week 5 灰度准备 |
| W4-T5 | 门禁官（预审） | Week 4 预审结论 | `phase1_week4_gate_precheck.md` | 红线缺口清晰可闭环 | Week 6 正式放行前置 |

## 周末验收清单
- [ ] 未验证提交路径无旁路
- [ ] 非确定性扫描阻断模式生效
- [ ] 对抗测试报告完成并有修复计划
- [ ] staging 联调稳定，可进入 Week 5 回归
- [ ] 预审结论给出“可继续/条件继续/暂停扩展”
