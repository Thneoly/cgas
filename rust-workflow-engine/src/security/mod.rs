//! 安全模块
//! 
//! 实现 Phase 2 安全相关功能：
//! - OIDC 身份验证：零信任架构基础
//! - 授权管理：RBAC+ABAC
//! - 审计日志：完整审计追踪
//!
//! Phase 3 Week 3 新增功能：
//! - OIDC 多 Provider 支持 (≥3 Provider)
//! - OPA 策略引擎 (RBAC+ABAC 联合策略)
//! - 安全闸门扩展 (Batch 嵌套 + Transaction 隔离)
//! - 威胁检测引擎 (25 类规则)

pub mod oidc;
pub mod rbac;
pub mod audit_log;
pub mod oidc_provider_impl;
pub mod opa_policy_engine;
pub mod security_gates_week3_impl;
pub mod threat_detection_impl;

pub use oidc::{
    OidcAuthenticator,
    OidcConfig,
    OidcToken,
    OidcUserInfo,
    OidcAuthRequest,
    OidcAuthResponse,
    OidcStats,
    OidcError,
};

pub use rbac::{
    RbacAbacAuthorizer,
    PermissionConfig,
    Role,
    Permission,
    AbacPolicy,
    AbacRule,
    AttributeCondition,
    Operator,
    Effect,
    AuthorizationRequest,
    AuthorizationResponse,
    AuthorizationStats,
};

pub use audit_log::{
    AuditLogger,
    AuditLogConfig,
    AuditLogEntry,
    AuditLogQuery,
    AuditLogStats,
    EventType,
    Subject,
    SubjectType,
    Action,
    ActionType,
    Resource,
    ResourceType,
    OperationResult,
    Environment,
    SecuritySeverity,
    AuditLogError,
};

// Phase 3 Week 3 新增导出
pub use oidc_provider_impl::{
    OidcMultiProviderManager,
    ProviderConfig,
    ProviderHealth,
    HealthCheckConfig,
    TokenCacheEntry,
    TokenClaims,
    TokenValidationResult,
    ValidationSource,
    CacheStats,
    CacheConfig,
    OidcProviderError,
};

pub use opa_policy_engine::{
    OpaPolicyEngine,
    OpaInput,
    OpaDecision,
    UserContext,
    UserAttributes,
    Action,
    Resource,
    Context,
    GeoLocation,
    DeviceTrust,
    PolicyBundle,
    BundleInfo,
    CacheConfig as OpaCacheConfig,
    CacheStats as OpaCacheStats,
    OpaError,
};

pub use security_gates_week3_impl::{
    SG1BatchNestedValidator,
    SG2PermissionValidator,
    SG3DataIntegrityValidator,
    SG4AuditValidator,
    OptimizedGateValidator,
    BatchCommand,
    Command,
    ExecuteCommand,
    TransactionCommand,
    IsolationLevel,
    BatchResult,
    CommandResult,
    ValidationResult,
    GateValidationResult,
    GateValidationRequest,
    PermissionRequest,
    AuditConfig,
};

pub use threat_detection_impl::{
    ThreatDetectionEngine,
    ThreatEvent,
    DetectionRule,
    ThreatCategory,
    Severity,
    RuleCondition,
    Alert,
    AlertAction,
    DetectionStats,
    ThreatDetectionError,
};
