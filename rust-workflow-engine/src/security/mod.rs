//! 安全模块
//! 
//! 实现 Phase 2 安全相关功能：
//! - OIDC 身份验证：零信任架构基础
//! - 授权管理：RBAC+ABAC
//! - 审计日志：完整审计追踪

pub mod oidc;
pub mod rbac;
pub mod audit_log;

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
