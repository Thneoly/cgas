use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("unknown role: {0}")]
    UnknownRole(String),

    #[error("invalid state transition for role={role}, from={from}, to={to}")]
    InvalidTransition {
        role: String,
        from: String,
        to: String,
    },

    #[error("schema invalid for artifact={artifact_name}: {reason}")]
    SchemaValidationFailed {
        artifact_name: String,
        reason: String,
    },

    #[error("gate failed: {0}")]
    GateFailed(String),

    #[error("rollback unavailable: {0}")]
    RollbackUnavailable(String),

    #[error("executor invocation failed: {0}")]
    ExecutorInvoke(String),

    #[error("artifact parse failed: {0}")]
    ArtifactParse(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
