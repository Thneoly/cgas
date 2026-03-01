# Phase01 多角色协作提示词包

本目录用于把执行蓝图落地为“可直接投喂给 Agent/角色成员”的提示词资产，支撑 `Phase 0 ~ Phase 5` 交付。

## 目录说明
- `role_prompt_playbook.md`：角色矩阵 + 角色提示词模板（含执行准确性守卫）
- `collaboration_runbook.md`：多角色协作机制、交接模板、关卡决策模板（含 Pre-Gate 清单）
- `phase1_execution_prompt_pack.md`：Phase 1 即用执行包（按周任务拆解 + 责任人 + DoD + Gate 映射 + 角色周度 Prompt）

## 使用顺序（建议）
1. 先在 `role_prompt_playbook.md` 选择本阶段启用角色。
2. 将“通用系统提示词” + “角色专属提示词”组合后投喂给各角色。
3. 按 `collaboration_runbook.md` 的节奏执行周评审、双周关卡、月度复盘。
4. 每阶段输出附件：`PRD`、`ADR`、`TEST-MATRIX`、`DEPLOY-RUNBOOK`、`GATE-REPORT`。

## Phase 1 启动方式（即用）
1. 打开 `phase1_execution_prompt_pack.md`，选择当前周（Week1~Week6）。
2. 按角色复制对应“周度 Prompt”，结合 `role_prompt_playbook.md` 第2节通用提示词投喂执行。
3. 按 `collaboration_runbook.md` 执行周评审与双周关卡。
4. Week 6 使用门禁官 Prompt 产出 Go/Conditional Go/No-Go 结论。

## 本次升级重点（执行准确性）
- 新增强制对齐文档：`technical_architecture_implementation_plan.md`、`executive_summary.md`
- 强化“指标证据四元组”：`指标值 / 时间窗口 / 样本量 / 数据来源`
- 强化架构边界检查：五层分层 + 双平面 + `Executor -> Verifier -> Committer`
- 增加 Pre-Gate 清单，减少“材料齐但不可放行”的评审返工

## 与现有文档对齐
- 上位目标与门禁口径：`doc/review_framework_one_pager.md`
- 分阶段任务蓝图：`doc/phase_detailed_execution_blueprint.md`
- Phase 0 首周计划：`doc/phase0_week1_kickoff_plan.md`
- 技术架构实现方案：`doc/technical_architecture_implementation_plan.md`
- 管理层执行摘要：`doc/executive_summary.md`
