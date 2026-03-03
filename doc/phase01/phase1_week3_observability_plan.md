# Week 3 观测计划（重放一致性证据）

## 1. 目标
- 建立重放一致性指标看板与证据输出。
- 保证 QA 与门禁官共享统一指标口径。

## 2. 本周任务

| 任务ID | 任务 | 输出 | DoD |
|---|---|---|---|
| OBS-W3-1 | 一致性指标口径确认 | 指标字典更新 | QA/门禁官双签 |
| OBS-W3-2 | replay 指标看板 | 看板说明 | 可按失败类型过滤 |
| OBS-W3-3 | 证据包导出 | 周证据包 | 满足四元组要求 |
| OBS-W3-4 | gate 输入预校验 | 预校验报告 | schema 通过率 100% |

## 3. 指标建议
- replay_consistency_rate
- mismatch_attribution_rate
- replay_trace_link_coverage

## 4. DoD
- 周末产出可直接用于门禁预审的证据包。
- 指标与日志口径无冲突。
