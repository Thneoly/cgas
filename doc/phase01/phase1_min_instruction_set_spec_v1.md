# 最小指令集规范 v1.0（Phase 1）

## 1. 目标
定义 Phase 1 最小可执行指令语义，确保可验证、可重放、可审计。

## 2. 设计原则
- 指令语义确定性：同输入得到同输出。
- 执行阶段仅产出 `state_diff`。
- 指令执行不直接触发最终提交。
- 所有异常路径必须返回可判定错误码。

## 3. 指令分类（最小集）

| 类别 | 示例 | 说明 |
|---|---|---|
| 读类 | `READ_STATE` | 仅读取状态，不产生副作用 |
| 算类 | `COMPUTE` | 纯函数计算，输出中间结果 |
| 写意图类 | `WRITE_DIFF` | 仅写入 `state_diff` 草稿 |
| 控制类 | `ASSERT`/`RETURN` | 断言与返回控制 |

> 注：指令名为规范占位，具体实现可按运行时命名，但语义必须等价。

## 4. 输入输出契约
- 输入：`program`, `input`, `state_root`, `env_fingerprint`
- 输出：`ExecutionResult(status, state_diff, trace_hash, result_hash, state_diff_hash, error_code)`

## 5. 错误与 REVERT 语义
- 任一关键断言失败或验证失败，执行状态进入 `REVERT`。
- `REVERT` 仅回退未提交状态，不影响已确认提交。

## 6. 可测试性要求
- 每条指令至少 1 条成功路径 + 1 条失败路径。
- 异常分支必须可在自动化测试中复现。
