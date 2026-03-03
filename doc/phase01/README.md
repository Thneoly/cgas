# Phase01 多角色协作提示词包

本目录用于把执行蓝图落地为“可直接投喂给 Agent/角色成员”的提示词资产，支撑 `Phase 0 ~ Phase 5` 交付。

## 目录说明
- `role_prompt_playbook.md`：角色矩阵 + 角色提示词模板（含执行准确性守卫）
- `collaboration_runbook.md`：多角色协作机制、交接模板、关卡决策模板（含 Pre-Gate 清单）
- `phase1_execution_prompt_pack.md`：Phase 1 即用执行包（按周任务拆解 + 责任人 + DoD + Gate 映射 + 角色周度 Prompt）
- `phase1_min_instruction_set_spec_v1.md`：最小指令集规范 v1.0
- `phase1_submission_gate_rules_v1.md`：提交闸门规则 v1.0
- `phase1_review_updates_2026-03-03.md`：Phase 1 回顾与补充更新记录

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

## Phase 1 Week 1 已开工包
- `phase1_week1_execution_board.md`（执行看板与角色启动指令）
- `phase1_week1_prd_v1.md`（PM 交付）
- `phase1_week1_adr_v1.md`（架构师交付）
- `phase1_week1_test_matrix_v1.md`（QA 交付）
- `phase1_week1_risk_register_v1.md`（项目经理交付）

## Phase 1 Week 2 开发接力包
- `phase1_week2_execution_board.md`（Week 2 执行看板）
- `phase1_week2_dev_delivery.md`（开发交付计划）
- `phase1_week2_qa_plan.md`（QA 计划）
- `phase1_week2_observability_plan.md`（观测计划）
- `phase1_week2_sre_plan.md`（SRE 计划）

## Phase 1 Week 3 重放一致性专项包
- `phase1_week3_execution_board.md`（Week 3 执行看板）
- `phase1_week3_dev_replay_plan.md`（开发重放计划）
- `phase1_week3_qa_consistency_plan.md`（QA 一致性计划）
- `phase1_week3_observability_plan.md`（观测证据计划）
- `phase1_week3_security_review.md`（安全审查）
- `phase1_week3_gate_precheck.md`（门禁预审模板）

## Phase 1 Week 4 阻断与扫描专项包
- `phase1_week4_execution_board.md`（Week 4 执行看板）
- `phase1_week4_commit_blocking_plan.md`（提交硬阻断计划）
- `phase1_week4_nondeterminism_scanner_plan.md`（非确定性扫描阻断计划）
- `phase1_week4_qa_adversarial_plan.md`（QA 对抗测试计划）
- `phase1_week4_sre_readiness.md`（SRE 就绪评估）
- `phase1_week4_gate_precheck.md`（门禁预审模板）

## Phase 1 Week 5 集成回归与灰度准备包
- `phase1_week5_execution_board.md`（Week 5 执行看板）
- `phase1_week5_dev_stabilization.md`（开发收敛计划）
- `phase1_week5_qa_e2e_plan.md`（QA E2E 回归计划）
- `phase1_week5_sre_gray_readiness.md`（SRE 灰度准备）
- `phase1_week5_observability_gate_rehearsal.md`（观测 Gate 预演）
- `phase1_week5_pmo_gate_pack.md`（项目经理材料收敛）
- `phase1_week5_gate_precheck.md`（门禁预审模板）

## Phase 1 Week 6 正式关卡放行包
- `phase1_week6_execution_board.md`（Week 6 执行看板）
- `phase1_week6_gate_material_checklist.md`（材料终检清单）
- `phase1_week6_gate_final_review.md`（门禁官终审模板）
- `phase1_week6_metrics_evidence_pack.md`（指标证据终稿）
- `phase1_week6_security_final_opinion.md`（安全终审意见）
- `phase1_week6_exception_approval_template.md`（例外审批模板）
- `phase1_week6_closeout_report.md`（Phase 1 收官报告）

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
