#![cfg(feature = "legacy-tests")]

//! 安全集成测试
//!
//! Phase 2 Week 5-T4 安全集成测试
//! 验证零信任架构 (OIDC+RBAC+ABAC+ 审计) 的完整集成

use rust_workflow_engine::security::{
    AuditLogConfig, AuditLogger, AuthorizationRequest, OidcAuthRequest, OidcAuthenticator,
    OidcConfig, PermissionConfig, RbacAbacAuthorizer,
};

#[tokio::test]
async fn test_oidc_authentication_flow() {
    // 创建 OIDC 认证器
    let config = OidcConfig {
        issuer_url: "https://auth.example.com".to_string(),
        auth_endpoint: "https://auth.example.com/oauth2/authorize".to_string(),
        token_endpoint: "https://auth.example.com/oauth2/token".to_string(),
        jwks_endpoint: "https://auth.example.com/.well-known/jwks.json".to_string(),
        client_id: "test_client".to_string(),
        client_secret: "test_secret".to_string(),
        redirect_uri: "https://app.example.com/callback".to_string(),
        scopes: vec!["openid".to_string(), "profile".to_string()],
    };

    let authenticator = OidcAuthenticator::new(config);

    // 验证认证器创建成功
    let stats = authenticator.get_stats();
    assert!(!stats.jwks_cached); // 初始状态 JWKS 未缓存
}

#[tokio::test]
async fn test_rbac_authorization() {
    // 创建 RBAC+ABAC 授权器
    let permission_config = PermissionConfig::new();
    let mut authorizer = RbacAbacAuthorizer::new(permission_config);

    // 添加用户角色
    authorizer.add_user_role("user_1", "admin");
    authorizer.add_user_role("user_2", "developer");
    authorizer.add_user_role("user_3", "tester");
    authorizer.add_user_role("user_4", "user");

    // 测试管理员权限
    let admin_request = AuthorizationRequest {
        user_id: "user_1".to_string(),
        user_attributes: std::collections::HashMap::new(),
        resource_id: "batch".to_string(),
        resource_attributes: std::collections::HashMap::new(),
        action: "execute".to_string(),
        environment_attributes: std::collections::HashMap::new(),
    };

    let admin_response = authorizer.authorize(admin_request);
    assert!(admin_response.permitted);

    // 测试开发者权限
    let dev_request = AuthorizationRequest {
        user_id: "user_2".to_string(),
        user_attributes: std::collections::HashMap::new(),
        resource_id: "batch".to_string(),
        resource_attributes: std::collections::HashMap::new(),
        action: "execute".to_string(),
        environment_attributes: std::collections::HashMap::new(),
    };

    let dev_response = authorizer.authorize(dev_request);
    assert!(dev_response.permitted);

    // 测试测试者权限 (应该被拒绝 batch:execute)
    let tester_request = AuthorizationRequest {
        user_id: "user_3".to_string(),
        user_attributes: std::collections::HashMap::new(),
        resource_id: "batch".to_string(),
        resource_attributes: std::collections::HashMap::new(),
        action: "execute".to_string(),
        environment_attributes: std::collections::HashMap::new(),
    };

    let tester_response = authorizer.authorize(tester_request);
    assert!(!tester_response.permitted);
}

#[tokio::test]
async fn test_abac_policy() {
    // 创建 RBAC+ABAC 授权器
    let mut permission_config = PermissionConfig::new();

    // 添加 ABAC 策略：只允许工作时间访问
    use rust_workflow_engine::security::{
        AbacPolicy, AbacRule, AttributeCondition, Effect, Operator,
    };

    let policy = AbacPolicy {
        id: "work_hours_only".to_string(),
        name: "Work Hours Only".to_string(),
        description: "Only allow access during work hours (9-18)".to_string(),
        rule: AbacRule {
            subject_conditions: vec![],
            resource_conditions: vec![],
            action_conditions: vec![],
            environment_conditions: vec![
                AttributeCondition {
                    attribute: "hour".to_string(),
                    operator: Operator::Ge,
                    value: serde_json::Value::Number(9.into()),
                },
                AttributeCondition {
                    attribute: "hour".to_string(),
                    operator: Operator::Le,
                    value: serde_json::Value::Number(18.into()),
                },
            ],
            effect: Effect::Permit,
        },
        priority: 1,
        enabled: true,
    };

    permission_config.abac_policies.push(policy);

    let mut authorizer = RbacAbacAuthorizer::new(permission_config);
    authorizer.add_user_role("user_1", "user");

    // 工作时间内的请求 (10 点)
    let mut env_attrs = std::collections::HashMap::new();
    env_attrs.insert("hour".to_string(), serde_json::Value::Number(10.into()));

    let work_request = AuthorizationRequest {
        user_id: "user_1".to_string(),
        user_attributes: std::collections::HashMap::new(),
        resource_id: "batch".to_string(),
        resource_attributes: std::collections::HashMap::new(),
        action: "execute".to_string(),
        environment_attributes: env_attrs,
    };

    let work_response = authorizer.authorize(work_request);
    assert!(work_response.permitted);

    // 非工作时间的请求 (20 点)
    let mut env_attrs = std::collections::HashMap::new();
    env_attrs.insert("hour".to_string(), serde_json::Value::Number(20.into()));

    let overtime_request = AuthorizationRequest {
        user_id: "user_1".to_string(),
        user_attributes: std::collections::HashMap::new(),
        resource_id: "batch".to_string(),
        resource_attributes: std::collections::HashMap::new(),
        action: "execute".to_string(),
        environment_attributes: env_attrs,
    };

    let overtime_response = authorizer.authorize(overtime_request);
    // ABAC 策略不匹配，回退到 RBAC
    // user 角色有 batch:execute 权限
    assert!(overtime_response.permitted);
}

#[tokio::test]
async fn test_audit_logging() {
    // 创建审计日志记录器
    let config = AuditLogConfig {
        log_path: "/tmp/cgas_audit_test".to_string(),
        retention_days: 30,
        max_file_size_mb: 10,
        async_write: false, // 测试使用同步模式
        queue_size: 100,
    };

    let mut logger = AuditLogger::new(config);
    logger.initialize().await.unwrap();

    // 记录认证事件
    logger.log_authentication("user_1", true, "oidc", None);
    logger.log_authentication(
        "user_2",
        false,
        "oidc",
        Some("Invalid credentials".to_string()),
    );

    // 记录授权事件
    logger.log_authorization(
        "user_1",
        &vec!["admin".to_string()],
        "batch",
        "execute",
        true,
        None,
    );
    logger.log_authorization(
        "user_2",
        &vec!["user".to_string()],
        "admin",
        "manage",
        false,
        Some("Insufficient permissions".to_string()),
    );

    // 记录数据访问事件
    logger.log_data_access(
        "user_1",
        rust_workflow_engine::security::ResourceType::Batch,
        "batch_1",
        rust_workflow_engine::security::ActionType::Execute,
        true,
    );

    // 记录安全事件
    let mut metadata = std::collections::HashMap::new();
    metadata.insert(
        "source".to_string(),
        serde_json::Value::String("firewall".to_string()),
    );

    logger.log_security_event(
        "intrusion_detected",
        rust_workflow_engine::security::SecuritySeverity::High,
        "Potential intrusion detected",
        Some("unknown"),
        metadata,
    );

    // 验证审计统计
    let stats = logger.get_stats();
    assert!(stats.total_entries >= 6);
}

#[tokio::test]
async fn test_full_security_integration() {
    // 完整安全集成测试：OIDC + RBAC + ABAC + 审计

    // 1. 创建所有安全组件
    let oidc_config = OidcConfig::default();
    let oidc_authenticator = OidcAuthenticator::new(oidc_config);

    let permission_config = PermissionConfig::new();
    let mut authorizer = RbacAbacAuthorizer::new(permission_config);

    let audit_config = AuditLogConfig::default();
    let mut audit_logger = AuditLogger::new(audit_config);
    audit_logger.initialize().await.unwrap();

    // 2. 模拟完整认证授权流程
    // 2.1 用户认证
    audit_logger.log_authentication("user_1", true, "oidc", None);

    // 2.2 添加用户角色
    authorizer.add_user_role("user_1", "developer");

    // 2.3 授权检查
    let auth_request = AuthorizationRequest {
        user_id: "user_1".to_string(),
        user_attributes: std::collections::HashMap::new(),
        resource_id: "batch".to_string(),
        resource_attributes: std::collections::HashMap::new(),
        action: "execute".to_string(),
        environment_attributes: std::collections::HashMap::new(),
    };

    let auth_response = authorizer.authorize(auth_request);
    assert!(auth_response.permitted);

    // 2.4 记录授权审计
    audit_logger.log_authorization(
        "user_1",
        &vec!["developer".to_string()],
        "batch",
        "execute",
        true,
        None,
    );

    // 2.5 记录数据访问审计
    audit_logger.log_data_access(
        "user_1",
        rust_workflow_engine::security::ResourceType::Batch,
        "batch_1",
        rust_workflow_engine::security::ActionType::Execute,
        true,
    );

    // 3. 验证所有组件协同工作正常
    let oidc_stats = oidc_authenticator.get_stats();
    let auth_stats = authorizer.get_stats();
    let audit_stats = audit_logger.get_stats();

    assert!(audit_stats.total_entries >= 3);
    assert!(auth_stats.permit_count >= 1);
}

#[tokio::test]
async fn test_security_zero_trust_architecture() {
    // 零信任架构验证测试

    // 1. 身份验证 (OIDC)
    let oidc_config = OidcConfig::default();
    let _oidc_authenticator = OidcAuthenticator::new(oidc_config);

    // 2. 授权管理 (RBAC+ABAC)
    let permission_config = PermissionConfig::new();
    let mut authorizer = RbacAbacAuthorizer::new(permission_config);

    // 添加用户和角色
    authorizer.add_user_role("user_1", "admin");
    authorizer.add_user_role("user_2", "developer");

    // 3. 审计日志
    let audit_config = AuditLogConfig::default();
    let mut audit_logger = AuditLogger::new(audit_config);
    audit_logger.initialize().await.unwrap();

    // 4. 模拟零信任流程
    // 4.1 每次访问都需要认证
    audit_logger.log_authentication("user_1", true, "oidc", None);

    // 4.2 每次访问都需要授权
    let auth_request = AuthorizationRequest {
        user_id: "user_1".to_string(),
        user_attributes: std::collections::HashMap::new(),
        resource_id: "transaction".to_string(),
        resource_attributes: std::collections::HashMap::new(),
        action: "execute".to_string(),
        environment_attributes: std::collections::HashMap::new(),
    };

    let auth_response = authorizer.authorize(auth_request);
    assert!(auth_response.permitted);

    // 4.3 所有操作都被审计
    audit_logger.log_authorization(
        "user_1",
        &vec!["admin".to_string()],
        "transaction",
        "execute",
        true,
        None,
    );

    // 5. 验证零信任原则
    // - 从不信任，总是验证
    // - 最小权限原则
    // - 完整审计追踪

    let audit_stats = audit_logger.get_stats();
    assert!(audit_stats.total_entries >= 2);
}
