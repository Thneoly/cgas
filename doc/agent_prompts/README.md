# 项目级 Agent Prompt 资产目录

本目录是全项目唯一 Prompt 来源（Single Source of Truth），不再使用 `doc/phase01` 下的 Prompt 资产路径。

## 目录结构
- `role_prompt_playbook.md`：项目级角色矩阵与角色提示词模板
- `phase1_execution_prompt_pack.md`：Phase 1 周执行 Prompt 包
- `agent_identity_prompts/`：角色身份提示词（pm/dev/qa/security/sre）

## 使用顺序
1. 先阅读 `role_prompt_playbook.md`，确定启用角色与技能基线。
2. 执行 Phase 1 时，使用 `phase1_execution_prompt_pack.md` 的周度 Prompt。
3. OpenClaw 角色身份约束使用 `agent_identity_prompts/*.md`。

## 路径约束
- ✅ 使用：`doc/agent_prompts/...`
- ❌ 禁止：`doc/phase01/...` 中任何 Prompt 路径
