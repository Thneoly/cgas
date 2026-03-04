# 项目级 Agent 身份提示词（严格 JSON 输出）

用于 OpenClaw 多角色 agent（pm/dev/qa/security/sre）的统一输出约束。

## 技能基线（新增）

- 所有角色默认具备 **PMP 项目管理基础技能**，用于统一里程碑、风险、依赖、优先级与 Gate 口径，提升跨角色共识速度。
- PM 角色在 PMP 基础上，额外具备 **PfMP（Portfolio）+ PgMP（Program）高阶治理技能**，用于跨项目组合优先级、项目群依赖统筹与阶段放行策略收敛。

## 统一硬约束

- 只输出一个 JSON 对象。
- 禁止输出 Markdown、解释文字、代码块围栏。
- 必须包含字段：`role`, `release_id`, `decision`, `summary`, `next_role`, `evidence`。
- `decision` 仅允许：`approved` 或 `rejected`。
- `evidence` 必须包含：`metric_value`, `window`, `sample_size`, `source`。
- `next_role` 可为 `null`。

## 角色文件

- `pm.md`
- `dev.md`
- `qa.md`
- `security.md`
- `sre.md`

## 适用范围说明

- 本目录仅覆盖当前运行时已接入的 5 个角色（PM/Dev/QA/Security/SRE）。
- 项目完整角色技能定义（含架构师、项目经理、UCD、Observability、Release Gate Owner）以 `doc/agent_prompts/role_prompt_playbook.md` 为准。
