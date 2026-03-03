# Week 3 开发计划（Verifier 重放专项）

## 1. 本周范围
1. 实现 Verifier 独立重放路径。
2. 建立 `result_hash` 重算与比对流程。
3. 对 mismatch 产出可分类的原因码。

## 2. 任务拆解

| 模块 | 任务 | 输入 | 输出 | DoD |
|---|---|---|---|---|
| Verifier Runtime | 独立重放主流程 | ExecutionResult + 请求上下文 | VerifyResult | 不共享执行进程状态 |
| Hash Compare | 哈希重算与比对 | result_hash/recomputed_result_hash | mismatch 判定 | 比对逻辑可回放 |
| Mismatch Classifier | 原因分类 | 失败样本 | mismatch_reason 分类 | 至少覆盖规则/输入/依赖三类 |
| Audit Trace | 重放审计日志 | trace_id/request_id | 重放日志流 | 可与执行日志关联 |

## 3. DoD
- 可对同一输入批量执行重放。
- mismatch 原因可回溯到最小定位单元。
- 为 QA 提供批跑接口和失败样本导出。
- 为观测提供一致性指标原始数据。

## 4. Week 4 交接项
- 提供被识别的非确定性热点清单。
- 提供提交阻断中间件所需旁路清单。
