use crate::blackboard_agent::BlackboardAgent;
use crate::error::EngineError;
use crate::executor::OpenClawExecutor;
use crate::model::{Artifact, Role, RoleState, WorkflowContext, WorkflowSnapshot};
use jsonschema::validator_for;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub type GateFn = fn(&WorkflowContext) -> Result<(), EngineError>;

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub transitions: HashMap<Role, Vec<(RoleState, RoleState)>>,
}

impl EngineConfig {
    pub fn mvp_default() -> Self {
        let allowed = vec![
            (RoleState::Idle, RoleState::InProgress),
            (RoleState::InProgress, RoleState::Submitted),
            (RoleState::Submitted, RoleState::Approved),
            (RoleState::Submitted, RoleState::Rejected),
            (RoleState::Rejected, RoleState::InProgress),
            (RoleState::Approved, RoleState::InProgress),
        ];

        let mut transitions = HashMap::new();
        transitions.insert(Role::PM, allowed.clone());
        transitions.insert(Role::Dev, allowed.clone());
        transitions.insert(Role::QA, allowed.clone());
        transitions.insert(Role::SRE, allowed.clone());
        transitions.insert(Role::Security, allowed);
        transitions.insert(Role::Blackboard, vec![]);

        Self { transitions }
    }
}

pub struct WorkflowEngine {
    pub context: WorkflowContext,
    config: EngineConfig,
    gates: Vec<GateFn>,
    history: Vec<WorkflowSnapshot>,
}

impl WorkflowEngine {
    pub fn new(context: WorkflowContext, config: EngineConfig) -> Self {
        Self {
            context,
            config,
            gates: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn register_gate(&mut self, gate: GateFn) {
        self.gates.push(gate);
    }

    pub fn add_artifact(&mut self, artifact: Artifact) {
        self.context
            .artifacts
            .insert(artifact.name.clone(), artifact);
    }

    pub fn transition_role(
        &mut self,
        role: Role,
        target: RoleState,
        reason: &str,
    ) -> Result<(), EngineError> {
        let current = self
            .context
            .role_states
            .get(&role)
            .cloned()
            .unwrap_or(RoleState::Idle);

        let Some(transitions) = self.config.transitions.get(&role) else {
            return Err(EngineError::UnknownRole(role.as_key().to_string()));
        };

        let valid = transitions
            .iter()
            .any(|(from, to)| *from == current && *to == target);
        if !valid {
            return Err(EngineError::InvalidTransition {
                role: role.as_key().to_string(),
                from: current.as_key().to_string(),
                to: target.as_key().to_string(),
            });
        }

        self.snapshot(reason);
        self.context.role_states.insert(role, target);
        Ok(())
    }

    pub fn validate_artifacts(&self) -> Result<(), EngineError> {
        for artifact in self.context.artifacts.values() {
            let compiled = validator_for(&artifact.schema).map_err(|e| {
                EngineError::SchemaValidationFailed {
                    artifact_name: artifact.name.clone(),
                    reason: format!("schema compile failed: {e}"),
                }
            })?;

            if !compiled.is_valid(&artifact.payload) {
                return Err(EngineError::SchemaValidationFailed {
                    artifact_name: artifact.name.clone(),
                    reason: "payload does not satisfy schema".to_string(),
                });
            }
        }
        Ok(())
    }

    pub fn execute_gates(&self) -> Result<(), EngineError> {
        for gate in &self.gates {
            gate(&self.context)?;
        }
        Ok(())
    }

    pub fn checkpoint(&mut self, reason: &str) {
        self.snapshot(reason);
    }

    pub fn rollback_last(&mut self) -> Result<(), EngineError> {
        let Some(snapshot) = self.history.pop() else {
            return Err(EngineError::RollbackUnavailable(
                "no snapshot found".to_string(),
            ));
        };
        self.context = snapshot.context;
        Ok(())
    }

    fn snapshot(&mut self, reason: &str) {
        self.history
            .push(WorkflowSnapshot::new(self.context.clone(), reason));
    }

    pub fn print_state(&self) {
        println!("release_id={}", self.context.release_id);
        for (role, state) in &self.context.role_states {
            println!("  role={} state={}", role.as_key(), state.as_key());
        }
        if !self.history.is_empty() {
            println!("  snapshots={}", self.history.len());
            if let Some(last) = self.history.last() {
                println!("  last_snapshot_reason={}", last.reason);
            }
        }
        println!("  blackboard_version={}", self.context.blackboard.version);
        println!(
            "  blackboard_events={}",
            self.context.blackboard.events.len()
        );
    }

    pub fn run_orchestration(
        &mut self,
        executor: &dyn OpenClawExecutor,
        prompts: &HashMap<Role, String>,
        initial_roles: Vec<Role>,
        max_steps: usize,
    ) -> Result<(), EngineError> {
        let dev_fallback = DevFallbackConfig::from_env();
        let collab_rounds = env_usize("OPENCLAW_COLLAB_ROUNDS", 2).max(1);
        let all_roles = vec![Role::PM, Role::Dev, Role::QA, Role::Security, Role::SRE];
        let mut retries: HashMap<Role, usize> = HashMap::new();
        let mut steps = 0usize;

        for round in 1..=collab_rounds {
            let initial = if round == 1 {
                initial_roles.clone()
            } else {
                all_roles.clone()
            };
            let mut queue: VecDeque<Role> = VecDeque::from(initial.clone());
            let mut queued: HashSet<Role> = initial.into_iter().collect();

            while let Some(role) = queue.pop_front() {
                queued.remove(&role);
                steps += 1;
                if steps > max_steps {
                    return Err(EngineError::GateFailed(format!(
                        "orchestration exceeded max_steps={max_steps}"
                    )));
                }

                let base_prompt = prompts.get(&role).ok_or_else(|| {
                    EngineError::ExecutorInvoke(format!(
                        "missing prompt for role={}",
                        role.as_key()
                    ))
                })?;
                let slots_snapshot = self.context.blackboard.slots.clone();
                let prompt =
                    augment_prompt_with_collaboration(base_prompt, round, &role, &slots_snapshot);

                self.transition_role(
                    role.clone(),
                    RoleState::InProgress,
                    &format!("{} starts", role.as_key()),
                )?;

                let raw_artifact = executor.execute(&role, &prompt, &self.context.release_id)?;
                let ingest = BlackboardAgent::ingest(
                    &mut self.context.blackboard,
                    &self.context.release_id,
                    round,
                    &role,
                    raw_artifact,
                );

                let mut artifact_json = ingest.normalized_artifact;
                artifact_json["round"] = Value::Number((round as u64).into());

                let schema = role_artifact_schema();
                let round_artifact_name =
                    format!("{}_r{}_artifact", role.as_key().to_lowercase(), round);
                self.add_artifact(Artifact::new(
                    round_artifact_name,
                    artifact_json.clone(),
                    schema.clone(),
                ));

                let latest_artifact_name = format!("{}_artifact", role.as_key().to_lowercase());
                self.add_artifact(Artifact::new(
                    latest_artifact_name,
                    artifact_json.clone(),
                    schema,
                ));

                let blackboard_artifact_name = format!(
                    "blackboard_r{}_from_{}_artifact",
                    round,
                    role.as_key().to_lowercase()
                );
                self.add_artifact(Artifact::new(
                    blackboard_artifact_name,
                    ingest.blackboard_artifact,
                    blackboard_artifact_schema(),
                ));
                self.validate_artifacts()?;

                self.transition_role(
                    role.clone(),
                    RoleState::Submitted,
                    &format!("{} submits artifact", role.as_key()),
                )?;

                let decision = artifact_json
                    .get("decision")
                    .and_then(Value::as_str)
                    .unwrap_or("rejected")
                    .to_string();

                let summary_value = artifact_json
                    .get("summary")
                    .cloned()
                    .unwrap_or(Value::String(String::new()));

                if decision.eq_ignore_ascii_case("approved") {
                    self.transition_role(
                        role.clone(),
                        RoleState::Approved,
                        &format!("{} approved", role.as_key()),
                    )?;
                } else {
                    self.transition_role(
                        role.clone(),
                        RoleState::Rejected,
                        &format!("{} rejected", role.as_key()),
                    )?;

                    let retry_count = retries.entry(role.clone()).or_insert(0);
                    if dev_fallback.enabled && *retry_count < dev_fallback.max_retry {
                        *retry_count += 1;
                        println!(
                            "dev-fallback retry role={} attempt={}/{}",
                            role.as_key(),
                            *retry_count,
                            dev_fallback.max_retry
                        );
                        queue.push_front(role.clone());
                        queued.insert(role.clone());
                        continue;
                    }

                    if dev_fallback.enabled && dev_fallback.allow_conditional_continue {
                        println!(
                            "dev-fallback conditional continue role={} (decision={})",
                            role.as_key(),
                            decision
                        );
                        self.transition_role(
                            role.clone(),
                            RoleState::InProgress,
                            &format!("{} fallback restart", role.as_key()),
                        )?;
                        self.transition_role(
                            role.clone(),
                            RoleState::Submitted,
                            &format!("{} fallback submit", role.as_key()),
                        )?;
                        self.transition_role(
                            role.clone(),
                            RoleState::Approved,
                            &format!("{} conditional approved", role.as_key()),
                        )?;
                    } else {
                        return Err(EngineError::GateFailed(format!(
                            "role={} rejected by artifact decision",
                            role.as_key()
                        )));
                    }
                }

                let routed = ingest.next_role;
                let routed_key = routed.as_ref().map(|r| r.as_key().to_string());

                let candidate = routed.filter(|next| next != &role);
                let mut next_role = candidate;
                let mut route_source = if routed_key.is_some() {
                    "artifact_next_role"
                } else {
                    "none"
                };
                let mut blocked_reason: Option<String> = None;

                if let Some(next) = &next_role {
                    let state = self
                        .context
                        .role_states
                        .get(next)
                        .cloned()
                        .unwrap_or(RoleState::Idle);
                    if !(state == RoleState::Idle || state == RoleState::Rejected) {
                        next_role = None;
                        blocked_reason = Some(format!(
                            "candidate state is {} (must be Idle/Rejected)",
                            state.as_key()
                        ));
                    }
                }

                let used_fallback = next_role.is_none();
                let next_role = next_role.or_else(|| fallback_next_role(&role));
                if used_fallback && next_role.is_some() {
                    route_source = "fallback_next_role";
                }
                if let Some(next_role) = next_role {
                    let state = self
                        .context
                        .role_states
                        .get(&next_role)
                        .cloned()
                        .unwrap_or(RoleState::Idle);
                    if (state == RoleState::Idle || state == RoleState::Rejected)
                        && !queued.contains(&next_role)
                    {
                        queue.push_back(next_role.clone());
                        queued.insert(next_role.clone());
                    }
                    let evidence_event_id = self.context.blackboard.version;
                    BlackboardAgent::record_dispatch(
                        &mut self.context.blackboard,
                        round,
                        &next_role,
                        &format!("Engine scheduled next role {}", next_role.as_key()),
                        build_dispatch_audit(
                            &self.context.release_id,
                            round,
                            &role,
                            &decision,
                            routed_key.as_deref(),
                            route_source,
                            blocked_reason.as_deref(),
                            &next_role,
                            state,
                            evidence_event_id,
                        ),
                    );

                    self.context.collaboration_log.push(json!({
                        "round": round,
                        "from": role.as_key(),
                        "to": next_role.as_key(),
                        "decision": decision,
                        "summary": summary_value
                    }));
                }
            }
        }
        Ok(())
    }

    pub fn export_artifacts<P: AsRef<Path>>(&self, dir: P) -> Result<(), EngineError> {
        fs::create_dir_all(&dir)?;
        for (name, artifact) in &self.context.artifacts {
            let file_path = dir.as_ref().join(format!("{name}.json"));
            let content = serde_json::to_string_pretty(&artifact.payload)?;
            fs::write(file_path, content)?;
        }
        let log_file = dir.as_ref().join("collaboration_log.json");
        let log_content = serde_json::to_string_pretty(&self.context.collaboration_log)?;
        fs::write(log_file, log_content)?;

        let blackboard_file = dir.as_ref().join("blackboard_state.json");
        let blackboard_content = serde_json::to_string_pretty(&self.context.blackboard)?;
        fs::write(blackboard_file, blackboard_content)?;
        Ok(())
    }

    pub fn materialize_deliverables<P: AsRef<Path>>(
        &self,
        base_dir: P,
        week: usize,
    ) -> Result<(), EngineError> {
        let base_dir = base_dir.as_ref();
        fs::create_dir_all(base_dir)?;

        let roles = [Role::PM, Role::Dev, Role::QA, Role::Security, Role::SRE];
        for role in roles {
            let artifact_name = format!("{}_artifact", role.as_key().to_lowercase());
            let Some(artifact) = self.context.artifacts.get(&artifact_name) else {
                continue;
            };

            let deliverables = artifact
                .payload
                .get("deliverables")
                .and_then(Value::as_array);
            let mut wrote_any = false;

            if let Some(deliverables) = deliverables {
                for item in deliverables {
                    let path = item.get("path").and_then(Value::as_str);
                    let content = item.get("content").and_then(Value::as_str);
                    let (Some(path), Some(content)) = (path, content) else {
                        continue;
                    };

                    let Some(file_path) = safe_join(base_dir, path) else {
                        continue;
                    };
                    if let Some(parent) = file_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(file_path, content)?;
                    wrote_any = true;
                }
            }

            if !wrote_any {
                let default_rel = role_default_deliverable_path(&role, week);
                let Some(file_path) = safe_join(base_dir, default_rel) else {
                    continue;
                };
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let summary = artifact
                    .payload
                    .get("summary")
                    .and_then(Value::as_str)
                    .unwrap_or("no summary provided");
                let fallback_content = format!(
                    "# {} auto-generated deliverable\n\n- release_id: {}\n- role: {}\n- source: workflow artifact fallback\n\n## summary\n{}\n",
                    role.as_key(),
                    self.context.release_id,
                    role.as_key(),
                    summary
                );
                fs::write(file_path, fallback_content)?;
            }
        }

        Ok(())
    }
}

fn role_default_deliverable_path(role: &Role, week: usize) -> &'static str {
    match week {
        1 => match role {
            Role::PM => "phase1_week1_prd_v1.md",
            Role::Dev => "phase1_week1_adr_v1.md",
            Role::QA => "phase1_week1_test_matrix_v1.md",
            Role::Security => "phase1_week3_security_review.md",
            Role::SRE => "phase1_week1_risk_register_v1.md",
            Role::Blackboard => "runtime_artifacts/blackboard_fallback.md",
        },
        2 => match role {
            Role::PM => "phase1_week2_execution_board.md",
            Role::Dev => "phase1_week2_dev_delivery.md",
            Role::QA => "phase1_week2_qa_plan.md",
            Role::Security => "phase1_week2_observability_plan.md",
            Role::SRE => "phase1_week2_sre_plan.md",
            Role::Blackboard => "runtime_artifacts/blackboard_fallback.md",
        },
        3 => match role {
            Role::PM => "phase1_week3_execution_board.md",
            Role::Dev => "phase1_week3_dev_replay_plan.md",
            Role::QA => "phase1_week3_qa_consistency_plan.md",
            Role::Security => "phase1_week3_security_review.md",
            Role::SRE => "phase1_week3_observability_plan.md",
            Role::Blackboard => "runtime_artifacts/blackboard_fallback.md",
        },
        4 => match role {
            Role::PM => "phase1_week4_execution_board.md",
            Role::Dev => "phase1_week4_commit_blocking_plan.md",
            Role::QA => "phase1_week4_qa_adversarial_plan.md",
            Role::Security => "phase1_week4_nondeterminism_scanner_plan.md",
            Role::SRE => "phase1_week4_sre_readiness.md",
            Role::Blackboard => "runtime_artifacts/blackboard_fallback.md",
        },
        5 => match role {
            Role::PM => "phase1_week5_execution_board.md",
            Role::Dev => "phase1_week5_dev_stabilization.md",
            Role::QA => "phase1_week5_qa_e2e_plan.md",
            Role::Security => "phase1_week5_pmo_gate_pack.md",
            Role::SRE => "phase1_week5_sre_gray_readiness.md",
            Role::Blackboard => "runtime_artifacts/blackboard_fallback.md",
        },
        6 => match role {
            Role::PM => "phase1_week6_closeout_report.md",
            Role::Dev => "phase1_week6_gate_final_review.md",
            Role::QA => "phase1_week6_metrics_evidence_pack.md",
            Role::Security => "phase1_week6_security_final_opinion.md",
            Role::SRE => "phase1_week6_gate_material_checklist.md",
            Role::Blackboard => "runtime_artifacts/blackboard_fallback.md",
        },
        _ => match role {
            Role::PM => "phase1_week1_prd_v1.md",
            Role::Dev => "phase1_week1_adr_v1.md",
            Role::QA => "phase1_week1_test_matrix_v1.md",
            Role::Security => "phase1_week3_security_review.md",
            Role::SRE => "phase1_week1_risk_register_v1.md",
            Role::Blackboard => "runtime_artifacts/blackboard_fallback.md",
        },
    }
}

fn safe_join(base: &Path, rel: &str) -> Option<PathBuf> {
    let normalized = rel.trim_start_matches('/');
    let candidate = Path::new(normalized);
    if candidate
        .components()
        .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_)))
    {
        return None;
    }
    Some(base.join(candidate))
}

fn augment_prompt_with_collaboration(
    base_prompt: &str,
    round: usize,
    role: &Role,
    latest_by_role: &HashMap<Role, Value>,
) -> String {
    let mut feedback_lines = Vec::new();
    for (from_role, artifact) in latest_by_role {
        if from_role == role {
            continue;
        }
        let decision = artifact
            .get("decision")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let summary = artifact
            .get("summary")
            .and_then(Value::as_str)
            .unwrap_or("");
        feedback_lines.push(format!(
            "- 来自{}: decision={}, summary={} ",
            from_role.as_key(),
            decision,
            summary
        ));
    }

    let feedback = if feedback_lines.is_empty() {
        "- 暂无上轮反馈，你需要先输出可交接结果。".to_string()
    } else {
        feedback_lines.join("\n")
    };

    format!(
        "{}\n\n[协作上下文]\n当前轮次: {}\n你必须吸收以下跨角色反馈并在本轮输出里体现闭环：\n{}",
        base_prompt, round, feedback
    )
}

struct DevFallbackConfig {
    enabled: bool,
    max_retry: usize,
    allow_conditional_continue: bool,
}

impl DevFallbackConfig {
    fn from_env() -> Self {
        Self {
            enabled: env_bool("OPENCLAW_DEV_FALLBACK_ENABLED"),
            max_retry: env_usize("OPENCLAW_DEV_FALLBACK_MAX_RETRY", 0),
            allow_conditional_continue: env_bool(
                "OPENCLAW_DEV_FALLBACK_ALLOW_CONDITIONAL_CONTINUE",
            ),
        }
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

fn env_usize(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(default)
}

fn fallback_next_role(current: &Role) -> Option<Role> {
    match current {
        Role::PM => Some(Role::Dev),
        Role::Dev => Some(Role::QA),
        Role::QA => Some(Role::Security),
        Role::Security => Some(Role::SRE),
        Role::SRE => None,
        Role::Blackboard => None,
    }
}

fn build_dispatch_audit(
    release_id: &str,
    round: usize,
    from_role: &Role,
    decision: &str,
    routed_from_artifact: Option<&str>,
    route_source: &str,
    blocked_reason: Option<&str>,
    selected_role: &Role,
    selected_state_before_enqueue: RoleState,
    evidence_event_id: u64,
) -> Value {
    json!({
        "rule_id": "engine.dispatch.route.v1",
        "trigger": "dispatch_decided",
        "release_id": release_id,
        "round": round,
        "from_role": from_role.as_key(),
        "artifact_decision": decision,
        "routed_from_artifact": routed_from_artifact,
        "route_source": route_source,
        "blocked_reason": blocked_reason,
        "selected_role": selected_role.as_key(),
        "selected_state_before_enqueue": selected_state_before_enqueue.as_key(),
        "candidates": {
            "artifact_next_role": routed_from_artifact,
            "fallback_next_role": fallback_next_role(from_role).map(|r| r.as_key().to_string())
        },
        "evidence_refs": {
            "blackboard_event_id": evidence_event_id,
            "artifact_type": "artifact_published"
        }
    })
}

fn role_artifact_schema() -> Value {
    json!({
        "type": "object",
        "required": [
            "role",
            "release_id",
            "decision",
            "summary",
            "evidence",
            "skills_applied",
            "risk_controls",
            "evidence_refs"
        ],
        "properties": {
            "role": { "type": "string" },
            "release_id": { "type": "string" },
            "decision": { "type": "string", "enum": ["approved", "rejected"] },
            "summary": { "type": "string" },
            "prompt_hash": { "type": "string" },
            "next_role": { "type": ["string", "null"] },
            "skills_applied": {
                "type": "array",
                "minItems": 1,
                "items": { "type": "string", "minLength": 1 }
            },
            "risk_controls": {
                "type": "array",
                "minItems": 1,
                "items": { "type": "string", "minLength": 1 }
            },
            "evidence_refs": {
                "type": "object",
                "required": ["skill_evidence", "risk_evidence"],
                "properties": {
                    "skill_evidence": { "type": "string", "minLength": 1 },
                    "risk_evidence": { "type": "string", "minLength": 1 },
                    "artifact_version": { "type": ["string", "number"] }
                },
                "additionalProperties": true
            },
            "deliverables": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["path", "content"],
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "additionalProperties": true
                }
            },
            "evidence": {
                "type": "object",
                "required": ["metric_value", "window", "sample_size", "source"],
                "properties": {
                    "metric_value": { "type": ["number", "string"] },
                    "window": { "type": "string" },
                    "sample_size": { "type": "number" },
                    "source": { "type": "string" }
                },
                "additionalProperties": true
            }
        },
        "additionalProperties": true
    })
}

fn blackboard_artifact_schema() -> Value {
    json!({
        "type": "object",
        "required": [
            "role",
            "release_id",
            "decision",
            "summary",
            "source_role",
            "blackboard_version",
            "event_type",
            "normalized",
            "evidence"
        ],
        "properties": {
            "role": { "type": "string", "enum": ["Blackboard"] },
            "release_id": { "type": "string" },
            "decision": { "type": "string", "enum": ["approved"] },
            "summary": { "type": "string" },
            "next_role": { "type": ["string", "null"] },
            "source_role": { "type": "string" },
            "blackboard_version": { "type": "number" },
            "event_type": { "type": "string", "enum": ["artifact_published"] },
            "normalized": { "type": "object" },
            "evidence": {
                "type": "object",
                "required": ["metric_value", "window", "sample_size", "source"],
                "properties": {
                    "metric_value": { "type": ["number", "string"] },
                    "window": { "type": "string" },
                    "sample_size": { "type": "number" },
                    "source": { "type": "string" }
                },
                "additionalProperties": true
            }
        },
        "additionalProperties": true
    })
}
