use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    PM,
    Dev,
    QA,
    SRE,
    Security,
    Blackboard,
}

impl Role {
    pub fn as_key(&self) -> &'static str {
        match self {
            Role::PM => "PM",
            Role::Dev => "Dev",
            Role::QA => "QA",
            Role::SRE => "SRE",
            Role::Security => "Security",
            Role::Blackboard => "Blackboard",
        }
    }

    pub fn from_key(value: &str) -> Option<Self> {
        match value {
            "PM" => Some(Role::PM),
            "Dev" => Some(Role::Dev),
            "QA" => Some(Role::QA),
            "SRE" => Some(Role::SRE),
            "Security" => Some(Role::Security),
            "Blackboard" => Some(Role::Blackboard),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoleState {
    Idle,
    InProgress,
    Submitted,
    Approved,
    Rejected,
}

impl RoleState {
    pub fn as_key(&self) -> &'static str {
        match self {
            RoleState::Idle => "Idle",
            RoleState::InProgress => "InProgress",
            RoleState::Submitted => "Submitted",
            RoleState::Approved => "Approved",
            RoleState::Rejected => "Rejected",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Artifact {
    pub name: String,
    pub payload: Value,
    pub schema: Value,
}

impl Artifact {
    pub fn new(name: impl Into<String>, payload: Value, schema: Value) -> Self {
        Self {
            name: name.into(),
            payload,
            schema,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowContext {
    pub release_id: String,
    pub role_states: HashMap<Role, RoleState>,
    pub artifacts: HashMap<String, Artifact>,
    pub collaboration_log: Vec<Value>,
    pub blackboard: BlackboardState,
}

impl WorkflowContext {
    pub fn new(release_id: impl Into<String>) -> Self {
        Self {
            release_id: release_id.into(),
            role_states: HashMap::new(),
            artifacts: HashMap::new(),
            collaboration_log: Vec::new(),
            blackboard: BlackboardState::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackboardEvent {
    pub id: u64,
    pub round: usize,
    pub from: String,
    pub event_type: String,
    pub decision: String,
    pub summary: String,
    pub next_role: Option<String>,
    #[serde(default)]
    pub audit: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlackboardState {
    pub version: u64,
    pub slots: HashMap<Role, Value>,
    pub events: Vec<BlackboardEvent>,
}

#[derive(Debug, Clone)]
pub struct WorkflowSnapshot {
    pub context: WorkflowContext,
    pub reason: String,
}

impl WorkflowSnapshot {
    pub fn new(context: WorkflowContext, reason: impl Into<String>) -> Self {
        Self {
            context,
            reason: reason.into(),
        }
    }
}
