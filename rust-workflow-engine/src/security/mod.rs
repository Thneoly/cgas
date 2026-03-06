//! 安全模块
//! 
//! 实现 Phase 2 安全相关功能：
//! - OIDC 身份验证：零信任架构基础
//! - 授权管理：RBAC+ABAC
//! - 审计日志：完整审计追踪

pub mod oidc;

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
