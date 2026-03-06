//! Transaction 集成测试
//! 
//! 测试 Transaction 事务的完整功能，包括 BEGIN/COMMIT/ROLLBACK 流程

use rust_workflow_engine::transaction::*;
use rust_workflow_engine::executor::Executor;

#[tokio::test]
async fn test_transaction_begin_commit() {
    // 创建执行器
    let executor = Executor::mock();
    let tx_executor = TransactionExecutor::new(executor);
    
    // 1. 开始事务
    let begin_request = BeginTransactionRequest::new(
        "trace_1".to_string(),
        "txn_1".to_string(),
        IsolationLevel::ReadCommitted,
        5000,
    );
    
    let begin_response = tx_executor.begin(begin_request).unwrap();
    assert_eq!(begin_response.status, TransactionStatus::Active);
    
    // 2. 执行事务内指令
    let execute_request = TransactionExecuteRequest {
        trace_id: "trace_1".to_string(),
        transaction_id: "txn_1".to_string(),
        instructions: vec![],
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let execute_response = tx_executor.execute(execute_request).unwrap();
    assert_eq!(execute_response.status, ExecutionStatus::Success);
    
    // 3. 提交事务
    let commit_request = CommitTransactionRequest {
        trace_id: "trace_1".to_string(),
        transaction_id: "txn_1".to_string(),
        transaction_hash: "test_hash".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let commit_response = tx_executor.commit(commit_request).unwrap();
    assert_eq!(commit_response.status, CommitStatus::Success);
}

#[tokio::test]
async fn test_transaction_rollback() {
    // 创建执行器
    let executor = Executor::mock();
    let tx_executor = TransactionExecutor::new(executor);
    
    // 1. 开始事务
    let begin_request = BeginTransactionRequest::new(
        "trace_1".to_string(),
        "txn_1".to_string(),
        IsolationLevel::ReadCommitted,
        5000,
    );
    
    let begin_response = tx_executor.begin(begin_request).unwrap();
    assert_eq!(begin_response.status, TransactionStatus::Active);
    
    // 2. 回滚事务
    let rollback_request = RollbackTransactionRequest {
        trace_id: "trace_1".to_string(),
        transaction_id: "txn_1".to_string(),
        reason: "test_rollback".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let rollback_response = tx_executor.rollback(rollback_request).unwrap();
    assert_eq!(rollback_response.status, RollbackStatus::Success);
}

#[tokio::test]
async fn test_transaction_timeout() {
    // 创建执行器
    let executor = Executor::mock();
    let tx_executor = TransactionExecutor::new(executor);
    
    // 1. 开始事务 (超时 100ms)
    let begin_request = BeginTransactionRequest::new(
        "trace_1".to_string(),
        "txn_1".to_string(),
        IsolationLevel::ReadCommitted,
        100, // 100ms 超时
    );
    
    let begin_response = tx_executor.begin(begin_request).unwrap();
    assert_eq!(begin_response.status, TransactionStatus::Active);
    
    // 2. 等待超时
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    
    // 3. 执行事务内指令 (应该超时)
    let execute_request = TransactionExecuteRequest {
        trace_id: "trace_1".to_string(),
        transaction_id: "txn_1".to_string(),
        instructions: vec![],
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let execute_response = tx_executor.execute(execute_request);
    assert!(execute_response.is_err());
    
    // 验证错误类型是超时
    match execute_response.unwrap_err() {
        TransactionError::TransactionTimeout => {
            // 预期错误
        }
        e => panic!("Expected TransactionTimeout, got {:?}", e),
    }
}

#[tokio::test]
async fn test_transaction_multiple_instructions() {
    // 创建执行器
    let executor = Executor::mock();
    let tx_executor = TransactionExecutor::new(executor);
    
    // 1. 开始事务
    let begin_request = BeginTransactionRequest::new(
        "trace_1".to_string(),
        "txn_1".to_string(),
        IsolationLevel::ReadCommitted,
        5000,
    );
    
    let begin_response = tx_executor.begin(begin_request).unwrap();
    assert_eq!(begin_response.status, TransactionStatus::Active);
    
    // 2. 执行多条指令
    let instructions = vec![
        create_test_instruction("trace_1"),
        create_test_instruction("trace_2"),
        create_test_instruction("trace_3"),
    ];
    
    let execute_request = TransactionExecuteRequest {
        trace_id: "trace_1".to_string(),
        transaction_id: "txn_1".to_string(),
        instructions,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let execute_response = tx_executor.execute(execute_request).unwrap();
    assert_eq!(execute_response.status, ExecutionStatus::Success);
    assert_eq!(execute_response.results.len(), 3);
    
    // 3. 提交事务
    let commit_request = CommitTransactionRequest {
        trace_id: "trace_1".to_string(),
        transaction_id: "txn_1".to_string(),
        transaction_hash: "test_hash".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let commit_response = tx_executor.commit(commit_request).unwrap();
    assert_eq!(commit_response.status, CommitStatus::Success);
}

fn create_test_instruction(trace_id: &str) -> crate::executor::ExecuteRequest {
    crate::executor::ExecuteRequest {
        trace_id: trace_id.to_string(),
        execution_id: format!("exec_{}", trace_id),
        instruction_type: crate::executor::InstructionType::Read,
        payload: crate::executor::InstructionPayload::default(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}
