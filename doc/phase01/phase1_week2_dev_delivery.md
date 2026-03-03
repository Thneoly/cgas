# Week 2 开发交付计划（开发工程师）

## 1. 本周开发范围
1. 最小指令语义：执行成功、失败、异常分支统一行为。
2. state_diff-only：执行器只输出差异，不执行最终提交。
3. 哈希与日志：输出 trace_hash、result_hash、state_diff_hash，并补齐关键字段。

## 2. 模块任务拆解

| 模块 | 任务 | 输入 | 输出 | DoD |
|---|---|---|---|---|
| Executor Core | 最小指令语义实现 | ADR v1、PRD v1 | 指令处理逻辑 | 单测通过，异常分支可复现 |
| Execution Result | state_diff 结构化输出 | 现有结果模型 | ExecutionResult 增量 | 无直接提交动作 |
| Hash Pipeline | 三类哈希生成 | 执行上下文、结果对象 | 哈希字段产出 | 字段完整率达标 |
| Audit Logging | 关键日志补齐 | 字段字典 | 统一日志格式 | trace_id/request_id 串联完整 |

## 3. 开发完成定义（DoD）
- 至少完成核心指令单测与一条集成路径验证。
- 任何执行路径均不可绕过 Verifier 进入 Committer。
- 对 QA 暴露稳定接口：ExecuteRequest/ExecutionResult。
- 对观测暴露稳定字段：trace_hash/result_hash/state_diff_hash。

## 4. 交接给 QA
- 可执行测试构建版本
- 变更清单（功能点 -> 模块 -> 接口）
- 失败码与异常映射表

## 5. 交接给观测工程师
- 指标字段定义更新
- 日志样例（成功/失败/阻断）
- 哈希字段完整性检查脚本说明
