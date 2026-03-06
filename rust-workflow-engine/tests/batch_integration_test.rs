//! Batch 集成测试
//! 
//! 测试 Batch 批量指令的完整功能，包括原子性保证

use rust_workflow_engine::batch::*;
use rust_workflow_engine::executor::Executor;

#[tokio::test]
async fn test_batch_single_instruction() {
    // 创建执行器
    let executor = Executor::mock();
    let batch_executor = BatchExecutor::new(executor);
    
    // 创建 Batch 请求 (单条指令)
    let request = BatchExecuteRequest::new(
        "trace_1".to_string(),
        "batch_1".to_string(),
        vec![create_test_instruction("trace_1")],
        true, // atomic=true
    );
    
    // 执行 Batch
    let result = batch_executor.execute(request).await.unwrap();
    
    // 验证结果
    assert_eq!(result.status, BatchStatus::Success);
    assert_eq!(result.results.len(), 1);
    assert!(!result.batch_hash.is_empty());
}

#[tokio::test]
async fn test_batch_multiple_instructions() {
    // 创建执行器
    let executor = Executor::mock();
    let batch_executor = BatchExecutor::new(executor);
    
    // 创建 Batch 请求 (10 条指令)
    let instructions = (1..=10)
        .map(|i| create_test_instruction(&format!("trace_{}", i)))
        .collect();
    
    let request = BatchExecuteRequest::new(
        "trace_1".to_string(),
        "batch_1".to_string(),
        instructions,
        true, // atomic=true
    );
    
    // 执行 Batch
    let result = batch_executor.execute(request).await.unwrap();
    
    // 验证结果
    assert_eq!(result.status, BatchStatus::Success);
    assert_eq!(result.results.len(), 10);
}

#[tokio::test]
async fn test_batch_atomic_failure() {
    // 创建执行器 (配置为在第 2 条指令失败)
    let mut executor = Executor::mock();
    executor.set_fail_at(2); // 第 2 条指令失败
    
    let batch_executor = BatchExecutor::new(executor);
    
    // 创建 Batch 请求 (atomic=true)
    let instructions = vec![
        create_test_instruction("trace_1"),
        create_test_instruction("trace_2"),
        create_test_instruction("trace_3"),
    ];
    
    let request = BatchExecuteRequest::new(
        "trace_1".to_string(),
        "batch_1".to_string(),
        instructions,
        true, // atomic=true
    );
    
    // 执行 Batch (应该失败并回滚)
    let result = batch_executor.execute(request).await;
    
    // 验证结果
    assert!(result.is_err());
    match result.unwrap_err() {
        BatchError::AtomicFailure(_) => {
            // 预期错误
        }
        e => panic!("Expected AtomicFailure, got {:?}", e),
    }
}

#[tokio::test]
async fn test_batch_non_atomic_partial_failure() {
    // 创建执行器 (配置为在第 2 条指令失败)
    let mut executor = Executor::mock();
    executor.set_fail_at(2); // 第 2 条指令失败
    
    let batch_executor = BatchExecutor::new(executor);
    
    // 创建 Batch 请求 (atomic=false)
    let instructions = vec![
        create_test_instruction("trace_1"),
        create_test_instruction("trace_2"),
        create_test_instruction("trace_3"),
    ];
    
    let request = BatchExecuteRequest::new(
        "trace_1".to_string(),
        "batch_1".to_string(),
        instructions,
        false, // atomic=false
    );
    
    // 执行 Batch (应该部分成功)
    let result = batch_executor.execute(request).await.unwrap();
    
    // 验证结果
    assert_eq!(result.status, BatchStatus::PartialFailure);
    assert_eq!(result.results.len(), 3);
}

#[tokio::test]
async fn test_batch_hash_verification() {
    // 创建执行器
    let executor = Executor::mock();
    let batch_executor = BatchExecutor::new(executor);
    
    // 创建 Batch 请求
    let instructions = vec![
        create_test_instruction("trace_1"),
        create_test_instruction("trace_2"),
    ];
    
    let request = BatchExecuteRequest::new(
        "trace_1".to_string(),
        "batch_1".to_string(),
        instructions.clone(),
        true,
    );
    
    // 执行 Batch
    let result = batch_executor.execute(request).await.unwrap();
    
    // 验证哈希
    let verified = verify_batch_hash(&instructions, &result.results, &result.batch_hash);
    assert!(verified);
}

#[tokio::test]
async fn test_batch_empty_batch() {
    // 创建执行器
    let executor = Executor::mock();
    let batch_executor = BatchExecutor::new(executor);
    
    // 创建空 Batch 请求
    let request = BatchExecuteRequest::new(
        "trace_1".to_string(),
        "batch_1".to_string(),
        vec![], // 空指令列表
        true,
    );
    
    // 执行 Batch (应该失败)
    let result = batch_executor.execute(request).await;
    
    // 验证结果
    assert!(result.is_err());
    match result.unwrap_err() {
        BatchError::Validation(BatchValidationError::EmptyBatch) => {
            // 预期错误
        }
        e => panic!("Expected EmptyBatch validation error, got {:?}", e),
    }
}

#[tokio::test]
async fn test_batch_too_large() {
    // 创建执行器
    let executor = Executor::mock();
    let batch_executor = BatchExecutor::new(executor);
    
    // 创建超大 Batch 请求 (101 条指令)
    let instructions = (1..=101)
        .map(|i| create_test_instruction(&format!("trace_{}", i)))
        .collect();
    
    let request = BatchExecuteRequest::new(
        "trace_1".to_string(),
        "batch_1".to_string(),
        instructions,
        true,
    );
    
    // 执行 Batch (应该失败)
    let result = batch_executor.execute(request).await;
    
    // 验证结果
    assert!(result.is_err());
    match result.unwrap_err() {
        BatchError::Validation(BatchValidationError::BatchTooLarge(101)) => {
            // 预期错误
        }
        e => panic!("Expected BatchTooLarge validation error, got {:?}", e),
    }
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
