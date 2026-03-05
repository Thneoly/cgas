use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowPlan {
    #[serde(default = "default_release_prefix")]
    pub release_prefix: String,
    #[serde(default = "default_deliverables_root")]
    pub deliverables_root: String,
    #[serde(default = "default_runtime_artifacts_root")]
    pub runtime_artifacts_root: String,
    #[serde(default = "default_prompt_pack_fallback_path")]
    pub prompt_pack_fallback_path: String,
    #[serde(default = "default_gate_rules_path")]
    pub gate_rules_path: String,
    #[serde(default = "default_board_headings")]
    pub board_headings: Vec<String>,
    #[serde(default = "default_gate_headings")]
    pub gate_headings: Vec<String>,
    pub stages: Vec<WorkflowStage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowStage {
    pub id: String,
    pub context_label: Option<String>,
    pub board_path: Option<String>,
    pub gate_rules_path: Option<String>,
    pub deliverables_root: Option<String>,
    pub artifact_output_dir: Option<String>,
    pub week_hint: Option<usize>,
    #[serde(default)]
    pub deliverable_paths: HashMap<String, String>,
    #[serde(default = "default_stage_gates")]
    pub gates: Vec<String>,
    #[serde(default = "default_initial_roles")]
    pub initial_roles: Vec<String>,
    pub max_steps: Option<usize>,
}

impl WorkflowPlan {
    pub fn load_from_env() -> Result<(Self, String), Box<dyn std::error::Error>> {
        let path = env::var("OPENCLAW_WORKFLOW_PLAN")
            .unwrap_or_else(|_| "../doc/agent_prompts/workflows/phase1.yaml".to_string());
        let content = fs::read_to_string(&path)?;
        let plan: WorkflowPlan = serde_yaml::from_str(&content)?;
        if plan.stages.is_empty() {
            return Err("workflow plan has no stages".into());
        }
        Ok((plan, path))
    }
}

impl WorkflowStage {
    pub fn context_label(&self) -> &str {
        self.context_label.as_deref().unwrap_or(&self.id)
    }
}

fn default_release_prefix() -> String {
    "release-2026-03-03".to_string()
}

fn default_deliverables_root() -> String {
    "../doc/phase01".to_string()
}

fn default_runtime_artifacts_root() -> String {
    "../doc/phase01/runtime_artifacts".to_string()
}

fn default_prompt_pack_fallback_path() -> String {
    "../doc/agent_prompts/phase1_execution_prompt_pack.md".to_string()
}

fn default_gate_rules_path() -> String {
    "../doc/phase01/phase1_submission_gate_rules_v1.md".to_string()
}

fn default_board_headings() -> Vec<String> {
    vec![
        "## 本周目标".to_string(),
        "## 任务表".to_string(),
        "## 角色启动指令".to_string(),
        "## 周末验收清单".to_string(),
    ]
}

fn default_gate_headings() -> Vec<String> {
    vec![
        "## 2. 核心规则".to_string(),
        "## 3. 判定口径".to_string(),
        "## 4. 例外策略".to_string(),
        "## 5. 审计要求".to_string(),
    ]
}

fn default_stage_gates() -> Vec<String> {
    vec![
        "pm_dev_qa_approved".to_string(),
        "security_or_exception".to_string(),
        "dispatch_audit_structured".to_string(),
        "artifact_skill_execution".to_string(),
    ]
}

fn default_initial_roles() -> Vec<String> {
    vec!["PM".to_string()]
}
