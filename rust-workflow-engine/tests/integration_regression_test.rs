#![cfg(feature = "legacy-tests")]

//! 集成回归测试
//! 
//! Phase 2 Week 5 集成回归测试
//! 验证 Batch + Transaction + 性能优化 + 零信任架构的完整集成

use rust_workflow_engine::batch::{BatchExecutor, BatchExecuteRequest, BatchStatus};
use rust_workflow_engine::transaction::{TransactionExecutor, BeginTransactionRequest, IsolationLevel};
use rust_workflow_engine::executor::{Executor, ExecuteRequest, InstructionType};
use rust_workflow_engine::security::{OidcAuthenticator, RbacAbacAuthorizer, AuditLogger};

#[tokio::test]
async fn test_batch_transaction_integration() {
    // 创建执行器
    let executor = Executor::mock();
    
    // 创建 Batch 执行器
    let batch_executor = BatchExecutor::new(executor.clone());
    
    // 创建 Transaction 执行器
    let tx_executor = TransactionExecutor::new(executor.clone());
    
    // 1. 执行 Batch 指令
    let batch_request = BatchExecuteRequest::new(
        "batch_trace_1".to_string(),
        "batch_1".to_string(),
        vec![
            create_test_instruction("trace_1"),
            create_test_instruction("trace_2"),
        ],
        true,
    );
    
    let batch_result = batch_executor.execute(batch_request).await.unwrap();
    assert_eq!(batch_result.status, BatchStatus::Success);
    assert_eq!(batch_result.results.len(), 2);
    
    // 2. 执行 Transaction 指令
    let begin_request = BeginTransactionRequest::new(
        "tx_trace_1".to_string(),
        "tx_1".to_string(),
        IsolationLevel::ReadCommitted,
        5000,
    );
    
    let begin_response = tx_executor.begin(begin_request).unwrap();
    assert_eq!(begin_response.status, rust_workflow_engine::transaction::TransactionStatus::Active);
    
    // 3. 验证 Batch 和 Transaction 可以并行执行
    // (实际场景需要更复杂的并发测试)
}

#[tokio::test]
async fn test_security_integration() {
    // 创建 OIDC 认证器
    let oidc_config = rust_workflow_engine::security::OidcConfig::default();
    let oidc_authenticator = OidcAuthenticator::new(oidc_config);
    
    // 创建 RBAC+ABAC 授权器
    let permission_config = rust_workflow_engine::security::PermissionConfig::new();
    let mut authorizer = RbacAbacAuthorizer::new(permission_config);
    
    // 创建审计日志记录器
    let audit_config = rust_workflow_engine::security::AuditLogConfig::default();
    let mut audit_logger = AuditLogger::new(audit_config);
    
    // 1. 测试认证
    // (实际场景需要完整的 OIDC flow)
    
    // 2. 测试授权
    authorizer.add_user_role("user_1", "admin");
    
    let auth_request = rust_workflow_engine::security::AuthorizationRequest {
        user_id: "user_1".to_string(),
        user_attributes: std::collections::HashMap::new(),
        resource_id: "batch".to_string(),
        resource_attributes: std::collections::HashMap::new(),
        action: "execute".to_string(),
        environment_attributes: std::collections::HashMap::new(),
    };
    
    let auth_response = authorizer.authorize(auth_request);
    assert!(auth_response.permitted);
    
    // 3. 测试审计日志
    audit_logger.initialize().await.unwrap();
    audit_logger.log_authentication("user_1", true, "oidc", None);
    audit_logger.log_authorization("user_1", &vec!["admin".to_string()], "batch", "execute", true, None);
}

#[tokio::test]
async fn test_performance_optimization_integration() {
    use rust_workflow_engine::optimization::{AsyncPool, AsyncPoolConfig, ValidationCache};
    use std::sync::Arc;
    
    // 1. 测试异步并发池
    let config = AsyncPoolConfig {
        worker_count: 4,
        queue_capacity: 100,
        task_timeout_ms: 500,
    };
    
    let task_fn = Arc::new(|x: i32| x * 2);
    let pool = AsyncPool::new(config, task_fn);
    
    let result = pool.submit(5).await.unwrap();
    assert_eq!(result, 10);
    
    // 2. 测试校验缓存
    let mut cache = ValidationCache::new(rust_workflow_engine::optimization::ValidationCacheConfig::default());
    
    let key = rust_workflow_engine::optimization::ValidationCacheKey {
        request_hash: "test_hash".to_string(),
        verifier_version: "v1.0".to_string(),
    };
    
    cache.set(&key, true);
    assert_eq!(cache.get(&key), Some(true));
}

#[tokio::test]
async fn test_scanner_optimization_integration() {
    use rust_workflow_engine::scanner::{ScannerOptimizer, ScannerOptimizerConfig, ExecutionPath, Operation, OperationType};
    
    // 创建扫描器优化器
    let config = ScannerOptimizerConfig {
        lock_detection_sensitivity: 0.7,
        time_window_ms: 50,
        false_positive_target: 0.02,
        adaptive_optimization: true,
    };
    
    let mut optimizer = ScannerOptimizer::new(config);
    
    // 1. 测试确定性路径检测
    let deterministic_path = ExecutionPath {
        id: "path_1".to_string(),
        operations: vec![
            Operation {
                id: "op_1".to_string(),
                operation_type: OperationType::Read,
                resource: "resource:1".to_string(),
                timestamp: 0,
                is_shared: false,
                lock_id: None,
            },
        ],
        lock_info: None,
    };
    
    let result = optimizer.scan_path(&deterministic_path);
    assert!(result.is_deterministic);
    
    // 2. 测试误报率统计
    let stats = optimizer.get_stats();
    assert!(stats.false_positive_rate <= 0.02);
}

#[tokio::test]
async fn test_full_integration() {
    // 完整集成测试：Batch + Transaction + Security + Performance + Scanner
    
    // 1. 创建所有组件
    let executor = Executor::mock();
    let batch_executor = BatchExecutor::new(executor.clone());
    let tx_executor = TransactionExecutor::new(executor.clone());
    
    let permission_config = rust_workflow_engine::security::PermissionConfig::new();
    let mut authorizer = RbacAbacAuthorizer::new(permission_config);
    
    let audit_config = rust_workflow_engine::security::AuditLogConfig::default();
    let mut audit_logger = AuditLogger::new(audit_config);
    audit_logger.initialize().await.unwrap();
    
    let scanner_config = rust_workflow_engine::scanner::ScannerOptimizerConfig::default();
    let mut scanner = rust_workflow_engine::scanner::ScannerOptimizer::new(scanner_config);
    
    // 2. 执行完整流程
    // 2.1 认证
    audit_logger.log_authentication("user_1", true, "oidc", None);
    
    // 2.2 授权
    authorizer.add_user_role("user_1", "developer");
    
    let auth_request = rust_workflow_engine::security::AuthorizationRequest {
        user_id: "user_1".to_string(),
        user_attributes: std::collections::HashMap::new(),
        resource_id: "batch".to_string(),
        resource_attributes: std::collections::HashMap::new(),
        action: "execute".to_string(),
        environment_attributes: std::collections::HashMap::new(),
    };
    
    let auth_response = authorizer.authorize(auth_request);
    assert!(auth_response.permitted);
    
    // 2.3 审计
    audit_logger.log_authorization("user_1", &vec!["developer".to_string()], "batch", "execute", true, None);
    
    // 2.4 执行 Batch
    let batch_request = BatchExecuteRequest::new(
        "batch_trace_1".to_string(),
        "batch_1".to_string(),
        vec![create_test_instruction("trace_1")],
        true,
    );
    
    let batch_result = batch_executor.execute(batch_request).await.unwrap();
    assert_eq!(batch_result.status, BatchStatus::Success);
    
    // 2.5 扫描器检测
    let path = ExecutionPath {
        id: "batch_path_1".to_string(),
        operations: vec![
            Operation {
                id: "op_1".to_string(),
                operation_type: OperationType::Read,
                resource: "resource:1".to_string(),
                timestamp: 0,
                is_shared: false,
                lock_id: None,
            },
        ],
        lock_info: None,
    };
    
    let scan_result = scanner.scan_path(&path);
    assert!(scan_result.is_deterministic);
    
    // 3. 验证所有组件协同工作正常
    let audit_stats = audit_logger.get_stats();
    assert!(audit_stats.total_entries >= 3);
}

fn create_test_instruction(trace_id: &str) -> ExecuteRequest {
    ExecuteRequest {
        trace_id: trace_id.to_string(),
        execution_id: format!("exec_{}", trace_id),
        instruction_type: InstructionType::Read,
        payload: rust_workflow_engine::executor::InstructionPayload::default(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}
