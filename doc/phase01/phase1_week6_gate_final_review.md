# Week 6 正式 Gate 终审模板（门禁官）

## 1. 终审输入
1) PRD
2) ADR
3) TEST-MATRIX
4) DEPLOY-RUNBOOK
5) GATE-REPORT（json+md）

## 2. 红线终审

| 红线项 | 判定条件 | 当前结果 | 结论 |
|---|---|---|---|
| 未验证提交率 | `= 0` 且窗口/关卡达标 | {{value}} | PASS/FAIL |
| 高危误放行率 | 阈值与 `u95` 同时达标 | {{value}} | PASS/FAIL |
| 样本量门禁 | 核心/高危样本量达标 | {{value}} | PASS/FAIL |
| 治理闭环（如适用） | 提案-测试-灰度-回滚完整 | {{value}} | PASS/FAIL |

## 3. 放行结论
- 结论：`Go / Conditional Go / No-Go`
- 关键依据（3~5 条）：
  1. {{evidence}}
  2. {{evidence}}
  3. {{evidence}}

## 4. 未达标项闭环计划

| 问题 | 责任人 | ETA | 补偿控制 | 复验标准 |
|---|---|---|---|---|
| {{issue}} | {{owner}} | {{eta}} | {{control}} | {{verify}} |
