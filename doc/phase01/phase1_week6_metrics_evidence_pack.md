# Week 6 指标证据终稿（QA / 观测）

## 证据四元组总表

| 指标 | 指标值 | 时间窗口 | 样本量 | 数据来源 | 判定 |
|---|---:|---|---:|---|---|
| 未验证提交率 | {{value}} | 14天 | {{n}} | 审计聚合 | PASS/FAIL |
| 重放一致率 | {{value}} | 7/14天 | {{n}} | 回放报告 | PASS/FAIL |
| 核心回归通过率 | {{value}} | Week5/Week6 | {{n}} | E2E 报告 | PASS/FAIL |
| `result_hash` 覆盖率 | {{value}} | Week6 | {{n}} | 字段完整率报告 | PASS/FAIL |

## 统计与口径声明
- 指标分母定义与窗口：对齐 `doc/review_framework_one_pager.md` 第 8.1/8.1.1
- 样本量不足策略：仅允许修复发布，不判定功能扩展通过

## 附件索引
- 回放一致性报告：{{link}}
- E2E 报告：{{link}}
- 审计聚合报表：{{link}}
- schema 校验报告：{{link}}
