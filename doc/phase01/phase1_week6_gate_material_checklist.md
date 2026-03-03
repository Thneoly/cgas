# Week 6 材料终检清单（PMO）

## 必备附件
- [ ] PRD（冻结版）
- [ ] ADR（冻结版）
- [ ] TEST-MATRIX（含 E2E 与一致性结果）
- [ ] DEPLOY-RUNBOOK（灰度与回滚）
- [ ] GATE-REPORT（json+md）

## 强校验项
- [ ] `gate-report.json` 已通过 schema 校验
- [ ] 红线指标证据四元组齐全
- [ ] 高危样本量达标或有批准例外
- [ ] 回滚演练记录有效
- [ ] 风险台账包含责任人和 ETA

## 输出
- 材料终检结论：`通过 / 条件通过 / 不通过`
- 不通过项及修复负责人：
  1. {{item}} -> {{owner}} / {{eta}}
  2. {{item}} -> {{owner}} / {{eta}}
