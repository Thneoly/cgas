use crate::error::EngineError;
use crate::model::Role;
use serde_json::{json, Value};
use std::env;
use std::process::Command;

pub trait OpenClawExecutor {
    fn execute(&self, role: &Role, prompt: &str, release_id: &str) -> Result<Value, EngineError>;
}

pub struct CliOpenClawExecutor {
    pub bin: String,
}

impl CliOpenClawExecutor {
    pub fn new(bin: impl Into<String>) -> Self {
        Self { bin: bin.into() }
    }
}

impl OpenClawExecutor for CliOpenClawExecutor {
    fn execute(&self, role: &Role, prompt: &str, release_id: &str) -> Result<Value, EngineError> {
        let role_identity = phase01_role_identity_prompt(role);
        let message = format!(
            "{}\n\nrelease_id={}\n任务={}\n\n输出要求：\n1) 仅输出一个 JSON 对象，不允许 markdown 或额外说明。\n2) 字段必须包含：role, release_id, decision, summary, next_role, evidence, skills_applied, risk_controls, evidence_refs。\n3) decision 仅允许 approved/rejected。\n4) evidence 必须包含 metric_value, window, sample_size, source。\n5) skills_applied 必须为非空字符串数组，列出本次执行使用的技能。\n6) risk_controls 必须为非空字符串数组，列出本次风险控制措施。\n7) evidence_refs 必须为对象，至少包含 skill_evidence 与 risk_evidence 字段。\n8) next_role 允许为 null。\n9) 必须提供 deliverables 数组，每项包含 path 与 content 字段，用于落地交付物文件。\n10) path 必须是相对路径（例如 phase1_week1_adr_v1.md），content 必须是完整 markdown 文本。",
            role_identity, release_id, prompt
        );
        let role_agent_key = format!("OPENCLAW_AGENT_{}", role.as_key().to_ascii_uppercase());
        let agent_id = env::var(&role_agent_key)
            .or_else(|_| env::var("OPENCLAW_AGENT_ID"))
            .unwrap_or_else(|_| "main".to_string());

        let output = Command::new(&self.bin)
            .arg("--no-color")
            .arg("agent")
            .arg("--local")
            .arg("--agent")
            .arg(agent_id)
            .arg("--message")
            .arg(message)
            .arg("--json")
            .output()
            .map_err(|e| EngineError::ExecutorInvoke(format!("spawn failed: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(EngineError::ExecutorInvoke(format!(
                "openclaw exit={:?}, stderr={} ",
                output.status.code(),
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let outer = parse_json_object(&stdout)
            .map_err(|e| EngineError::ArtifactParse(format!("stdout is not valid json: {e}")))?;

        if outer.get("role").is_some() && outer.get("decision").is_some() {
            return Ok(outer);
        }

        if let Some(text) = outer
            .get("response")
            .and_then(|v| v.get("content"))
            .and_then(Value::as_str)
        {
            return serde_json::from_str::<Value>(text).map_err(|e| {
                EngineError::ArtifactParse(format!("response.content is not artifact json: {e}"))
            });
        }

        if let Some(text) = outer.get("output").and_then(Value::as_str) {
            return serde_json::from_str::<Value>(text).map_err(|e| {
                EngineError::ArtifactParse(format!("output is not artifact json: {e}"))
            });
        }

        if let Some(text) = outer
            .get("payloads")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("text"))
            .and_then(Value::as_str)
        {
            return parse_json_object(text).map_err(|e| {
                EngineError::ArtifactParse(format!("payloads[0].text is not artifact json: {e}"))
            });
        }

        Err(EngineError::ArtifactParse(
            "cannot find artifact json in openclaw output".to_string(),
        ))
    }
}

fn phase01_role_identity_prompt(role: &Role) -> &'static str {
    match role {
        Role::PM => "你是 Phase01 产品经理（PM）。具备全角色共享的 PMP 基础项目管理技能，并具备更高阶 PfMP+PgMP 治理技能与 Gate 指标化治理能力。聚焦范围冻结、非目标、Gate 指标映射、跨项目依赖统筹与风险收敛。",
        Role::Dev => {
            "你是 Phase01 架构/开发角色。具备 PMP 基础项目管理技能、Rust 核心工程能力（所有权/借用、Result、并发异步、serde 契约）和 Platform 工程能力（TypeScript/Node.js、gRPC/REST 契约联调）。聚焦 ADR、接口契约、失败路径与回滚路径，不越权提交。"
        }
        Role::QA => {
            "你是 Phase01 QA。具备 PMP 基础项目管理技能、回放一致性与契约测试能力（ExecuteRequest/ExecutionResult/VerifyResult），以及性能韧性测试能力（k6/Locust、Chaos 注入与恢复验证）。聚焦测试矩阵、样本策略与指标证据四元组，样本不足不可给通过结论。"
        }
        Role::Security => "你是 Phase01 安全工程师。具备 PMP 基础项目管理技能、身份与授权能力（OIDC/OAuth2、RBAC+ABAC）和运行/供应链安全能力（seccomp/apparmor、Vault/KMS、Manifest 签名校验、SAST/SCA）。聚焦提交闸门红线、非确定性风险与阻断覆盖结论。",
        Role::SRE => {
            "你是 Phase01 SRE/项目经理执行角色。具备 PMP 基础项目管理技能，并具备平台与运维工程能力（Linux、Docker、Kubernetes、CI/CD/GitOps、Prometheus+Grafana 可观测、告警与故障响应）及 Rust 服务运维诊断能力（发布健康检查、panic/backtrace 排障、性能与资源异常定位）。聚焦执行看板、关键路径、阻塞项与开工条件。"
        }
        Role::Blackboard => {
            "你是 Blackboard Agent，仅处理信息汇总与事件中继，不输出业务结论。"
        }
    }
}

fn parse_json_object(raw: &str) -> Result<Value, serde_json::Error> {
    if let Ok(value) = serde_json::from_str::<Value>(raw) {
        return Ok(value);
    }

    if let Some(candidate) = extract_first_json_object(raw) {
        return serde_json::from_str::<Value>(&candidate);
    }

    serde_json::from_str::<Value>(raw)
}

fn extract_first_json_object(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    let mut start: Option<usize> = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, byte) in bytes.iter().enumerate() {
        let ch = *byte as char;

        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            continue;
        }

        if ch == '{' {
            if start.is_none() {
                start = Some(index);
            }
            depth += 1;
            continue;
        }

        if ch == '}' && depth > 0 {
            depth -= 1;
            if depth == 0 {
                if let Some(begin) = start {
                    return Some(raw[begin..=index].to_string());
                }
            }
        }
    }

    None
}

pub struct MockOpenClawExecutor;

impl OpenClawExecutor for MockOpenClawExecutor {
    fn execute(&self, role: &Role, prompt: &str, release_id: &str) -> Result<Value, EngineError> {
        let next_role = match role {
            Role::PM => Some("Dev"),
            Role::Dev => Some("QA"),
            Role::QA => Some("Security"),
            Role::Security => Some("SRE"),
            Role::SRE => None,
            Role::Blackboard => None,
        };

        let week = env::var("OPENCLAW_CURRENT_WEEK")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1);
        let deliverable_path = default_deliverable_path(week, role);

        Ok(json!({
            "role": role.as_key(),
            "release_id": release_id,
            "decision": "approved",
            "summary": format!("{} executed prompt", role.as_key()),
            "prompt_hash": format!("len:{}", prompt.len()),
            "next_role": next_role,
            "skills_applied": [
                "pmp_baseline",
                format!("{}_role_skill", role.as_key().to_lowercase())
            ],
            "risk_controls": [
                "gate_alignment",
                "evidence_traceability"
            ],
            "evidence_refs": {
                "skill_evidence": "prompt_contract_v2",
                "risk_evidence": "gate_rules_v1",
                "artifact_version": "v2"
            },
            "deliverables": [
                {
                    "path": deliverable_path,
                    "content": format!(
                        "# {} deliverable\n\n- release_id: {}\n- generated_by: mock-openclaw\n\n{}",
                        role.as_key(),
                        release_id,
                        prompt
                    )
                }
            ],
            "evidence": {
                "metric_value": 0.98,
                "window": "14d",
                "sample_size": 120,
                "source": "mock-openclaw"
            }
        }))
    }
}

fn default_deliverable_path(week: usize, role: &Role) -> &'static str {
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
