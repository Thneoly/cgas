mod blackboard_agent;
mod engine;
mod error;
mod executor;
mod gates;
mod model;

use crate::engine::{EngineConfig, WorkflowEngine};
use crate::executor::{CliOpenClawExecutor, MockOpenClawExecutor, OpenClawExecutor};
use crate::gates::{
    gate_dispatch_audit_structured, gate_phase2_readiness, gate_pm_dev_qa_approved,
    gate_security_or_exception,
};
use crate::model::{Role, RoleState, WorkflowContext};
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

    for week in 1..=6 {
        println!("== week{:02} orchestration start ==", week);
        env::set_var("OPENCLAW_CURRENT_WEEK", week.to_string());

        let mut ctx = WorkflowContext::new(format!("release-2026-03-03-w{:02}", week));
        ctx.role_states.insert(Role::PM, RoleState::Idle);
        ctx.role_states.insert(Role::Dev, RoleState::Idle);
        ctx.role_states.insert(Role::QA, RoleState::Idle);
        ctx.role_states.insert(Role::SRE, RoleState::Idle);
        ctx.role_states.insert(Role::Security, RoleState::Idle);
        ctx.role_states.insert(Role::Blackboard, RoleState::Idle);

        let mut engine = WorkflowEngine::new(ctx, EngineConfig::mvp_default());
        engine.register_gate(gate_pm_dev_qa_approved);
        engine.register_gate(gate_security_or_exception);
        engine.register_gate(gate_dispatch_audit_structured);
        if week == 6 {
            engine.register_gate(gate_phase2_readiness);
        }

        let prompts = build_phase_week_prompts(week)?;
        engine.run_orchestration(executor.as_ref(), &prompts, vec![Role::PM], 30)?;

        println!("== week{:02} after orchestration ==", week);
        engine.print_state();

        let week_artifacts_dir = format!("../doc/phase01/runtime_artifacts/week{}", week);
        engine.export_artifacts(&week_artifacts_dir)?;
        println!("artifacts exported to {}", week_artifacts_dir);

        engine.materialize_deliverables("../doc/phase01", week)?;
        println!("deliverables materialized for week{:02} to ../doc/phase01", week);

        engine.execute_gates()?;
        println!("== week{:02} gates pass ==", week);
        engine.print_state();
    }

    Ok(())
}

fn build_phase_week_prompts(week: usize) -> Result<HashMap<Role, String>, Box<dyn std::error::Error>> {
    let board_path = format!("../doc/phase01/phase1_week{}_execution_board.md", week);
    let board = fs::read_to_string(&board_path)?;
    let gate_rules = fs::read_to_string("../doc/phase01/phase1_submission_gate_rules_v1.md")?;

    let board_excerpt = extract_markdown_sections(
        &board,
        &[
            "## 本周目标",
            "## 任务表",
            "## 角色启动指令",
            "## 周末验收清单",
        ],
        5000,
    );
    let gate_excerpt = extract_markdown_sections(
        &gate_rules,
        &["## 2. 核心规则", "## 3. 判定口径", "## 4. 例外策略", "## 5. 审计要求"],
        2600,
    );

    let fixed_context = format!(
        "上下文说明：\n- 当前阶段：Phase1 Week{}。\n- 你不需要自行在文件系统查找文档。\n- 以下已提供本地 Phase1 文档摘录，直接基于摘录执行。\n- role 字段必须使用 PM/Dev/QA/Security/SRE 之一。\n- 你必须输出 deliverables: [{{path, content}}]，path 必须使用给定目标路径。\n",
        week
    );

    let mut prompts = HashMap::new();
    prompts.insert(
        Role::PM,
        format!(
            "{}\n任务：以 PM 身份输出本周执行结论、风险与下周准入条件。\n目标交付路径：{}\n\n[execution_board 摘录]\n{}",
            fixed_context,
            week_role_deliverable_path(week, &Role::PM),
            board_excerpt
        ),
    );
    prompts.insert(
        Role::Dev,
        format!(
            "{}\n任务：以架构/开发身份输出本周技术方案、接口契约、失败与回滚路径。\n目标交付路径：{}\n\n[execution_board 摘录]\n{}",
            fixed_context,
            week_role_deliverable_path(week, &Role::Dev),
            board_excerpt
        ),
    );
    prompts.insert(
        Role::QA,
        format!(
            "{}\n任务：以 QA 身份输出本周测试策略、样本门禁和证据四元组。\n目标交付路径：{}\n\n[execution_board 摘录]\n{}",
            fixed_context,
            week_role_deliverable_path(week, &Role::QA),
            board_excerpt
        ),
    );
    prompts.insert(
        Role::Security,
        format!(
            "{}\n任务：以 Security 身份输出本周安全审查结论、红线风险与阻断覆盖。\n目标交付路径：{}\n\n[phase1_submission_gate_rules 摘录]\n{}",
            fixed_context,
            week_role_deliverable_path(week, &Role::Security),
            gate_excerpt
        ),
    );
    prompts.insert(
        Role::SRE,
        format!(
            "{}\n任务：以 SRE/执行负责人身份输出本周执行看板、风险台账、阻塞项与下一周开工条件。\n目标交付路径：{}\n\n[execution_board 摘录]\n{}",
            fixed_context,
            week_role_deliverable_path(week, &Role::SRE),
            board_excerpt
        ),
    );

    Ok(prompts)
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

fn extract_markdown_sections(doc: &str, headings: &[&str], max_chars: usize) -> String {
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
            .find(|(heading, _)| heading.contains(target))
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

fn env_bool(key: &str) -> bool {
    match env::var(key) {
        Ok(v) => matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
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
                if name == "sessions.json" || name.ends_with(".jsonl") || name.contains(".jsonl.reset") {
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

    if ids.is_empty() {
        ids.push("main".to_string());
    }

    ids
}
