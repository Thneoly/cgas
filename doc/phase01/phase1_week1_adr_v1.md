# ADR v1（Phase 1 / Week 1）

## ADR-1：执行-验证-提交硬闸门
- 决策：`Executor -> Verifier -> Committer` 强依赖，验证失败必须 `REVERT`。
- 原因：确保未验证提交率可实现为 `0`。
- 影响：提交链路需中间件阻断；开发与测试需覆盖绕过路径。

## ADR-2：执行阶段仅产出 state_diff
- 决策：执行阶段禁止直接写入最终状态。
- 原因：降低越权风险，支持独立复验。
- 影响：需显式定义 `state_diff` 数据模型与哈希字段。

## ADR-3：哈希契约标准化
- 决策：统一输出 `trace_hash`、`result_hash`、`state_diff_hash`。
- 原因：支持回放一致性比对和审计追踪。
- 影响：观测链路需补齐字段与校验。

## ADR-4：非确定性扫描阻断模式
- 决策：默认阻断隐式时间/随机/未声明外部依赖。
- 原因：避免“同输入不同输出”破坏可信性。
- 影响：引入扫描器与例外审批流程。

## 接口契约边界（本周冻结）
- `ExecuteRequest`: `program_id`, `program_hash`, `input`, `state_root`, `gas_limit`, `policy_context`
- `ExecutionResult`: `status`, `state_diff`, `trace_hash`, `result_hash`, `gas_used`, `error_code`
- `VerifyResult`: `verified`, `recomputed_result_hash`, `mismatch_reason`

## Week 2 实施约束
1. Core Runtime 变更优先落在执行/验证核心，不引入生态扩展。
2. 控制平面只做必要编排改造，不改变治理流程层级。
3. 任何新增路径必须可审计、可回滚。
