# Rust Workflow Engine (MVP)

最小可运行的 Workflow Engine 框架，用于充当 OpenClaw 的中心决策调度器。

当前实现遵循：

- Rust Workflow Engine 做总控（编排、状态机、Schema、Gate、回滚）
- OpenClaw 只做执行器（按角色接收 prompt，返回 JSON artifact）
- 所有 artifact JSON 化
- 黑板驱动协作：角色写黑板，Engine 监听黑板事件后调度下一角色

## MVP 能力

- 中心决策调度：`WorkflowEngine` 统一驱动状态、门禁与回滚。
- 多角色状态流转：内置 `PM/Dev/QA/SRE/Security` 与状态机约束。
- 敏捷协作路由：不强制固定角色顺序，支持 artifact 返回 `next_role` 动态决定下一个角色。
- Schema 校验：交付物基于 JSON Schema 进行结构校验。
- 门禁逻辑：支持注册多个门禁函数并按顺序执行。
- 回滚：支持快照与最近一次状态回滚。

## 目录

- `src/model.rs`：角色、状态、上下文、交付物、快照模型。
- `src/engine.rs`：引擎主逻辑（流转、校验、门禁、回滚）。
- `src/gates.rs`：MVP 门禁样例。
- `src/executor.rs`：OpenClaw 执行器抽象（CLI + Mock）。
- `src/main.rs`：可直接运行的端到端示例（Scrum 式动态路由）。

## 运行

```bash
cargo run
```

## 测试（库化后的分层策略）

- 默认主线测试（CI 建议口径）：

```bash
cargo test --tests
```

- Legacy 测试（历史模块兼容验证，按需启用）：

```bash
cargo test --tests --features legacy-tests
```

说明：当前仓库已完成 `src/lib.rs` 库化基础，但历史 `batch/transaction/security/optimization` 套件与现行执行器契约存在接口漂移。`legacy-tests` 用于隔离这部分迁移工作，避免影响主线交付。

## 真实 OpenClaw 对接（现在可用）

1) 切换执行器模式为 CLI：

```bash
export OPENCLAW_EXECUTOR_MODE=cli
export OPENCLAW_BIN=openclaw
# 默认即分会话（即使不设也会使用 pm/dev/qa/security/sre）
# 可选：按角色覆盖 agent
export OPENCLAW_AGENT_PM=pm
export OPENCLAW_AGENT_DEV=dev
export OPENCLAW_AGENT_QA=qa
export OPENCLAW_AGENT_SECURITY=security
export OPENCLAW_AGENT_SRE=sre
# 可选：全局兜底（不建议在正式分会话模式使用）
# export OPENCLAW_AGENT_ID=main
# 开发模式兜底（默认关闭）
# export OPENCLAW_DEV_FALLBACK_ENABLED=true
# export OPENCLAW_DEV_FALLBACK_MAX_RETRY=2
# export OPENCLAW_DEV_FALLBACK_ALLOW_CONDITIONAL_CONTINUE=true
# 多轮协作（默认 2 轮）
# export OPENCLAW_COLLAB_ROUNDS=2
cargo run
```

2) Rust 总控会按以下命令调用 OpenClaw（每个角色一轮）：

```bash
openclaw agent --local --agent <AGENT_ID> --message "<PROMPT>" --json
```

> agent 选择规则：
> 1. 优先读取 `OPENCLAW_AGENT_<ROLE>`（如 `OPENCLAW_AGENT_PM`）
> 2. 未设置时使用 `OPENCLAW_AGENT_ID`（全局兜底）
> 3. 仍未设置时默认按角色分会话：`pm/dev/qa/security/sre`

开发模式兜底策略（默认关闭）：

- `OPENCLAW_DEV_FALLBACK_ENABLED=true`：启用兜底。
- `OPENCLAW_DEV_FALLBACK_MAX_RETRY=<n>`：当 `decision=rejected` 时，同角色自动重试次数。
- `OPENCLAW_DEV_FALLBACK_ALLOW_CONDITIONAL_CONTINUE=true`：重试后仍拒绝时，按 Conditional Continue 继续后续流转（仅建议开发环境使用）。

多轮协作与反馈通道：

- 引擎会把上轮各角色的 `decision + summary` 注入下一角色 prompt，形成跨角色反馈闭环。
- `OPENCLAW_COLLAB_ROUNDS` 控制协作轮次（默认 2），避免“一轮就结束”。
- 导出工件包含每轮工件（如 `pm_r1_artifact.json`）、`collaboration_log.json`、`blackboard_state.json`。

黑板驱动 MVP 要点：

- Blackboard Agent（特殊角色）负责信息面：汇总角色 artifact、标准化字段、广播反馈、维护版本与审计事件。
- Agent 写黑板：每次角色产出 artifact 后由 Blackboard Agent 写入 `blackboard.slots[role]` 并记录 `artifact_published` 事件。
- Engine 监听黑板：监听黑板事件并记录 `dispatch_decided` 事件，再决定下一角色执行（Engine 只做调度决策）。
- 黑板快照导出：每轮运行结束导出 `blackboard_state.json`（含版本号、slots、events）。

3) OpenClaw 必须输出单个 JSON artifact 到 stdout，最小字段契约：

```json
{
	"role": "PM",
	"release_id": "release-2026-03-03",
	"decision": "approved",
	"summary": "...",
	"next_role": "Dev",
	"evidence": {
		"metric_value": 0.98,
		"window": "14d",
		"sample_size": 120,
		"source": "..."
	}
}
```

4) `decision` 仅允许 `approved|rejected`；`next_role` 可为空（null）表示流程结束。

运行输出会展示：

1. 从初始角色开始（示例为 PM）
2. 每轮调用执行器获取 JSON artifact
3. 对 artifact 做 schema 校验并更新状态
4. 根据 `next_role` 动态推进协作链路
5. 最终执行 Gate 判定

> 说明：默认示例使用 `MockOpenClawExecutor` 便于本地直接运行；
> 切换到真实 OpenClaw CLI 时可改为 `CliOpenClawExecutor`。

## 与 OpenClaw 对接建议（下一步）

- 将 `Role` / `RoleState` 抽象为可配置枚举（YAML/TOML）。
- 将 `GateFn` 扩展为 trait，对接外部策略服务与审计日志。
- 在 `snapshot` 持久化到存储（PostgreSQL/Redis）以支持跨进程恢复。
- 为每次流转附加 `trace_id`、操作者、时间戳用于可观测与审计。

## Phase01 角色身份提示词（已生成）

- 目录：`doc/agent_prompts/agent_identity_prompts`
- 文件：`pm.md`、`dev.md`、`qa.md`、`security.md`、`sre.md`
- 用途：约束各角色 agent 稳定输出单个 JSON artifact，减少联调中的格式漂移。
