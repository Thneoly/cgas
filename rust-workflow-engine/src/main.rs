use rust_workflow_engine::engine::{EngineConfig, WorkflowEngine};
use rust_workflow_engine::executor::{CliOpenClawExecutor, MockOpenClawExecutor, OpenClawExecutor};
use rust_workflow_engine::gates::{
    gate_artifact_skill_execution, gate_dispatch_audit_structured, gate_phase0_ci_integration,
    gate_phase0_contract_defined, gate_phase0_contract_frozen, gate_phase0_gate_report_ready,
    gate_phase0_replay_set_ready, gate_phase0_sample_minimum, gate_phase0_stakeholders_approved,
    gate_phase0_validator_ready, gate_phase2_readiness, gate_pm_dev_qa_approved,
    gate_security_or_exception,
};
use rust_workflow_engine::model::{Role, RoleState, WorkflowContext};
use rust_workflow_engine::workflow_plan::{WorkflowPlan, WorkflowStage};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor_mode = env::var("OPENCLAW_EXECUTOR_MODE").unwrap_or_else(|_| "mock".to_string());
    let executor: Box<dyn OpenClawExecutor> = if executor_mode.eq_ignore_ascii_case("cli") {
        let bin = env::var("OPENCLAW_BIN").unwrap_or_else(|_| "openclaw".to_string());
        println!("executor_mode=cli, bin={bin}");
        Box::new(CliOpenClawExecutor::new(bin))
    } else {
        println!("executor_mode=mock");
        Box::new(MockOpenClawExecutor)
    };

    if executor_mode.eq_ignore_ascii_case("cli") && env_bool("OPENCLAW_CLEAR_AGENT_HISTORY") {
        clear_openclaw_agent_histories()?;
        println!("openclaw agent histories cleared before this run");
    }

    let (plan, plan_path) = WorkflowPlan::load_from_env()?;
    println!("workflow_plan={} stages={}", plan_path, plan.stages.len());

    for (index, stage) in plan.stages.iter().enumerate() {
        let week_hint = stage.week_hint.unwrap_or(index + 1);
        println!("== {} orchestration start ==", stage.id);
        env::set_var("OPENCLAW_CURRENT_WEEK", week_hint.to_string());
        env::set_var("OPENCLAW_CURRENT_STAGE", &stage.id);
        set_stage_deliverable_env(stage, week_hint);

        let mut ctx = WorkflowContext::new(format!("{}-{}", plan.release_prefix, stage.id));
        ctx.role_states.insert(Role::PM, RoleState::Idle);
        ctx.role_states.insert(Role::Dev, RoleState::Idle);
        ctx.role_states.insert(Role::QA, RoleState::Idle);
        ctx.role_states.insert(Role::SRE, RoleState::Idle);
        ctx.role_states.insert(Role::Security, RoleState::Idle);
        ctx.role_states.insert(Role::Blackboard, RoleState::Idle);

        let mut engine = WorkflowEngine::new(ctx, EngineConfig::mvp_default());
        register_stage_gates(&mut engine, stage)?;

        let prompts = build_stage_prompts(stage, &plan, week_hint)?;
        let initial_roles = resolve_roles(&stage.initial_roles)?;
        let max_steps = stage.max_steps.unwrap_or(30);
        engine.run_orchestration(executor.as_ref(), &prompts, initial_roles, max_steps)?;

        println!("== {} after orchestration ==", stage.id);
        engine.print_state();

        let artifact_dir = stage.artifact_output_dir.clone().unwrap_or_else(|| {
            format!(
                "{}/{}",
                plan.runtime_artifacts_root.trim_end_matches('/'),
                stage.id
            )
        });
        engine.export_artifacts(&artifact_dir)?;
        println!("artifacts exported to {}", artifact_dir);

        let deliverables_root = stage
            .deliverables_root
            .as_deref()
            .unwrap_or(&plan.deliverables_root);
        engine.materialize_deliverables(deliverables_root, week_hint)?;
        println!(
            "deliverables materialized for stage={} to {}",
            stage.id, deliverables_root
        );

        engine.execute_gates()?;
        println!("== {} gates pass ==", stage.id);
        engine.print_state();
    }

    Ok(())
}

fn build_stage_prompts(
    stage: &WorkflowStage,
    plan: &WorkflowPlan,
    week_hint: usize,
) -> Result<HashMap<Role, String>, Box<dyn std::error::Error>> {
    let board_path = stage
        .board_path
        .clone()
        .unwrap_or_else(|| format!("../doc/phase01/phase1_week{}_execution_board.md", week_hint));
    let board = match fs::read_to_string(&board_path) {
        Ok(content) => content,
        Err(_) => {
            let fallback = &plan.prompt_pack_fallback_path;
            println!(
                "stage={} execution board missing at {}, fallback to {}",
                stage.id, board_path, fallback
            );
            fs::read_to_string(fallback)?
        }
    };
    let gate_rules_path = stage
        .gate_rules_path
        .as_deref()
        .unwrap_or(&plan.gate_rules_path);
    let gate_rules = fs::read_to_string(gate_rules_path)?;

    let board_excerpt = extract_markdown_sections(&board, &plan.board_headings, 5000);
    let gate_excerpt = extract_markdown_sections(&gate_rules, &plan.gate_headings, 2600);

    let fixed_context = format!(
        "上下文说明：\n- 当前阶段：{}。\n- 你不需要自行在文件系统查找文档。\n- 以下已提供本地文档摘录，直接基于摘录执行。\n- role 字段必须使用 PM/Dev/QA/Security/SRE 之一。\n- 你必须输出 deliverables: [{{path, content}}]，path 必须使用给定目标路径。\n",
        stage.context_label()
    );

    let mut prompts = HashMap::new();
    prompts.insert(
        Role::PM,
        format!(
            "{}\n任务：以 PM 身份输出本周执行结论、风险与下周准入条件。\n目标交付路径：{}\n\n[execution_board 摘录]\n{}",
            fixed_context,
            stage_role_deliverable_path(stage, week_hint, &Role::PM),
            board_excerpt
        ),
    );
    prompts.insert(
        Role::Dev,
        format!(
            "{}\n任务：以架构/开发身份输出本周技术方案、接口契约、失败与回滚路径。\n目标交付路径：{}\n\n[execution_board 摘录]\n{}",
            fixed_context,
            stage_role_deliverable_path(stage, week_hint, &Role::Dev),
            board_excerpt
        ),
    );
    prompts.insert(
        Role::QA,
        format!(
            "{}\n任务：以 QA 身份输出本周测试策略、样本门禁和证据四元组。\n目标交付路径：{}\n\n[execution_board 摘录]\n{}",
            fixed_context,
            stage_role_deliverable_path(stage, week_hint, &Role::QA),
            board_excerpt
        ),
    );
    prompts.insert(
        Role::Security,
        format!(
            "{}\n任务：以 Security 身份输出本周安全审查结论、红线风险与阻断覆盖。\n目标交付路径：{}\n\n[phase1_submission_gate_rules 摘录]\n{}",
            fixed_context,
            stage_role_deliverable_path(stage, week_hint, &Role::Security),
            gate_excerpt
        ),
    );
    prompts.insert(
        Role::SRE,
        format!(
            "{}\n任务：以 SRE/执行负责人身份输出本周执行看板、风险台账、阻塞项与下一周开工条件。\n目标交付路径：{}\n\n[execution_board 摘录]\n{}",
            fixed_context,
            stage_role_deliverable_path(stage, week_hint, &Role::SRE),
            board_excerpt
        ),
    );

    Ok(prompts)
}

fn stage_role_deliverable_path(stage: &WorkflowStage, week_hint: usize, role: &Role) -> String {
    if let Some(path) = stage
        .deliverable_paths
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(role.as_key()))
        .map(|(_, v)| v.clone())
    {
        return path;
    }

    week_role_deliverable_path(week_hint, role).to_string()
}

fn week_role_deliverable_path(week: usize, role: &Role) -> &'static str {
    match week {
        1 => match role {
            Role::PM => "phase1_week1_prd_v1.md",
            Role::Dev => "phase1_week1_adr_v1.md",
            Role::QA => "phase1_week1_test_matrix_v1.md",
            Role::Security => "phase1_week3_security_review.md",
            Role::SRE => "phase1_week1_risk_register_v1.md",
            Role::Blackboard => "runtime_artifacts/blackboard.md",
        },
        2 => match role {
            Role::PM => "phase1_week2_execution_board.md",
            Role::Dev => "phase1_week2_dev_delivery.md",
            Role::QA => "phase1_week2_qa_plan.md",
            Role::Security => "phase1_week2_observability_plan.md",
            Role::SRE => "phase1_week2_sre_plan.md",
            Role::Blackboard => "runtime_artifacts/blackboard.md",
        },
        3 => match role {
            Role::PM => "phase1_week3_execution_board.md",
            Role::Dev => "phase1_week3_dev_replay_plan.md",
            Role::QA => "phase1_week3_qa_consistency_plan.md",
            Role::Security => "phase1_week3_security_review.md",
            Role::SRE => "phase1_week3_observability_plan.md",
            Role::Blackboard => "runtime_artifacts/blackboard.md",
        },
        4 => match role {
            Role::PM => "phase1_week4_execution_board.md",
            Role::Dev => "phase1_week4_commit_blocking_plan.md",
            Role::QA => "phase1_week4_qa_adversarial_plan.md",
            Role::Security => "phase1_week4_nondeterminism_scanner_plan.md",
            Role::SRE => "phase1_week4_sre_readiness.md",
            Role::Blackboard => "runtime_artifacts/blackboard.md",
        },
        5 => match role {
            Role::PM => "phase1_week5_execution_board.md",
            Role::Dev => "phase1_week5_dev_stabilization.md",
            Role::QA => "phase1_week5_qa_e2e_plan.md",
            Role::Security => "phase1_week5_pmo_gate_pack.md",
            Role::SRE => "phase1_week5_sre_gray_readiness.md",
            Role::Blackboard => "runtime_artifacts/blackboard.md",
        },
        6 => match role {
            Role::PM => "phase1_week6_closeout_report.md",
            Role::Dev => "phase1_week6_gate_final_review.md",
            Role::QA => "phase1_week6_metrics_evidence_pack.md",
            Role::Security => "phase1_week6_security_final_opinion.md",
            Role::SRE => "phase1_week6_gate_material_checklist.md",
            Role::Blackboard => "runtime_artifacts/blackboard.md",
        },
        _ => match role {
            Role::PM => "phase1_week1_prd_v1.md",
            Role::Dev => "phase1_week1_adr_v1.md",
            Role::QA => "phase1_week1_test_matrix_v1.md",
            Role::Security => "phase1_week3_security_review.md",
            Role::SRE => "phase1_week1_risk_register_v1.md",
            Role::Blackboard => "runtime_artifacts/blackboard.md",
        },
    }
}

fn extract_markdown_sections(doc: &str, headings: &[String], max_chars: usize) -> String {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current_heading = String::new();
    let mut current_body = String::new();

    for line in doc.lines() {
        if line.trim_start().starts_with('#') {
            if !current_heading.is_empty() {
                sections.push((current_heading.clone(), current_body.clone()));
            }
            current_heading = line.trim().to_string();
            current_body.clear();
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }

    if !current_heading.is_empty() {
        sections.push((current_heading, current_body));
    }

    let mut merged = String::new();
    for target in headings {
        if let Some((heading, body)) = sections
            .iter()
            .find(|(heading, _)| heading.contains(target.as_str()))
        {
            merged.push_str(heading);
            merged.push('\n');
            merged.push_str(body);
            merged.push('\n');
        }
    }

    if merged.trim().is_empty() {
        merged = doc.chars().take(max_chars).collect();
    }

    merged.chars().take(max_chars).collect()
}

fn resolve_roles(raw_roles: &[String]) -> Result<Vec<Role>, Box<dyn std::error::Error>> {
    let mut roles = Vec::new();
    for raw in raw_roles {
        let role = Role::from_key(raw).ok_or_else(|| {
            format!(
                "unknown role '{}' in workflow stage initial_roles; expected PM/Dev/QA/Security/SRE",
                raw
            )
        })?;
        roles.push(role);
    }

    if roles.is_empty() {
        roles.push(Role::PM);
    }

    Ok(roles)
}

fn register_stage_gates(
    engine: &mut WorkflowEngine,
    stage: &WorkflowStage,
) -> Result<(), Box<dyn std::error::Error>> {
    for gate_name in &stage.gates {
        match gate_name.as_str() {
            // Phase 1 gates
            "pm_dev_qa_approved" => engine.register_gate(gate_pm_dev_qa_approved),
            "security_or_exception" => engine.register_gate(gate_security_or_exception),
            "dispatch_audit_structured" => engine.register_gate(gate_dispatch_audit_structured),
            "artifact_skill_execution" => engine.register_gate(gate_artifact_skill_execution),
            "phase2_readiness" => engine.register_gate(gate_phase2_readiness),

            // Phase 0 gates
            "phase0_contract_defined" => engine.register_gate(gate_phase0_contract_defined),
            "phase0_stakeholders_approved" => {
                engine.register_gate(gate_phase0_stakeholders_approved)
            }
            "phase0_contract_frozen" => engine.register_gate(gate_phase0_contract_frozen),
            "phase0_validator_ready" => engine.register_gate(gate_phase0_validator_ready),
            "phase0_replay_set_ready" => engine.register_gate(gate_phase0_replay_set_ready),
            "phase0_sample_minimum" => engine.register_gate(gate_phase0_sample_minimum),
            "phase0_gate_report_ready" => engine.register_gate(gate_phase0_gate_report_ready),
            "phase0_ci_integration" => engine.register_gate(gate_phase0_ci_integration),

            other => {
                return Err(format!("unknown gate in workflow plan: {}", other).into());
            }
        }
    }

    Ok(())
}

fn set_stage_deliverable_env(stage: &WorkflowStage, week_hint: usize) {
    for role in [Role::PM, Role::Dev, Role::QA, Role::Security, Role::SRE] {
        let key = format!(
            "OPENCLAW_DEFAULT_DELIVERABLE_{}",
            role.as_key().to_ascii_uppercase()
        );
        let value = stage_role_deliverable_path(stage, week_hint, &role);
        env::set_var(key, value);
    }
}

fn env_bool(key: &str) -> bool {
    match env::var(key) {
        Ok(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

fn clear_openclaw_agent_histories() -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    let base = PathBuf::from(home).join(".openclaw").join("agents");
    let agent_ids = resolve_openclaw_agent_ids();

    for agent_id in agent_ids {
        let sessions_dir = base.join(&agent_id).join("sessions");
        if !sessions_dir.exists() {
            continue;
        }

        for entry in fs::read_dir(&sessions_dir)? {
            let path = entry?.path();
            if path.is_file() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                if name == "sessions.json"
                    || name.ends_with(".jsonl")
                    || name.contains(".jsonl.reset")
                {
                    fs::remove_file(&path)?;
                }
            }
        }

        fs::write(sessions_dir.join("sessions.json"), "{}")?;
    }

    Ok(())
}

fn resolve_openclaw_agent_ids() -> Vec<String> {
    let mut ids = Vec::new();
    let vars = [
        "OPENCLAW_AGENT_PM",
        "OPENCLAW_AGENT_DEV",
        "OPENCLAW_AGENT_QA",
        "OPENCLAW_AGENT_SECURITY",
        "OPENCLAW_AGENT_SRE",
        "OPENCLAW_AGENT_ID",
    ];

    for key in vars {
        if let Ok(v) = env::var(key) {
            let value = v.trim();
            if !value.is_empty() && !ids.iter().any(|x| x == value) {
                ids.push(value.to_string());
            }
        }
    }

    let default_role_ids = ["pm", "dev", "qa", "security", "sre"];
    for value in default_role_ids {
        if !ids.iter().any(|x| x == value) {
            ids.push(value.to_string());
        }
    }

    if ids.is_empty() {
        ids.push("main".to_string());
    }

    ids
}
