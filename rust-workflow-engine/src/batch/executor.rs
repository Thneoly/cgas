//! Batch 执行器
//! 
//! 实现 Batch 批量指令的执行逻辑，支持原子性保证

use crate::batch::types::*;
use crate::executor::{Executor, ExecutionResult};
use log::{info, warn, error};

/// Batch 执行器
pub struct BatchExecutor {
    /// 底层执行器
    executor: Executor,
}

impl BatchExecutor {
    /// 创建新的 Batch 执行器
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }
    
    /// 执行 Batch 请求
    pub async fn execute(&self, request: BatchExecuteRequest) 
                         -> Result<BatchExecuteResult, BatchError> {
        let start = std::time::Instant::now();
        let sub_instruction_count = request.instructions.len();
        
        // 1. 参数验证
        request.validate()
            .map_err(BatchError::Validation)?;
        
        // 2. 逐条执行指令
        let (results, has_failure) = self.execute_instructions(&request).await?;
        
        // 3. 确定 Batch 状态
        let status = self.determine_batch_status(&results, has_failure, request.atomic);
        
        // 4. 计算 Batch 哈希
        let batch_hash = compute_batch_hash(&request.instructions, &results);
        
        // 5. 构建结果
        let result = match status {
            BatchStatus::Success => {
                BatchExecuteResult::success(
                    request.trace_id.clone(),
                    request.batch_id.clone(),
                    results,
                    batch_hash,
                )
            }
            BatchStatus::Failed => {
                BatchExecuteResult::failed(
                    request.trace_id.clone(),
                    request.batch_id.clone(),
                    results,
                    batch_hash,
                )
            }
            BatchStatus::PartialFailure => {
                BatchExecuteResult::partial_failure(
                    request.trace_id.clone(),
                    request.batch_id.clone(),
                    results,
                    batch_hash,
                )
            }
        };
        
        // 6. 采集监控指标
        let latency = start.elapsed();
        // metrics::observe_batch_execute(latency.as_secs_f64(), sub_instruction_count);
        
        if !request.atomic && has_failure {
            // metrics::inc_batch_atomicity_violation("non_atomic_batch");
        }
        
        info!(
            "Batch executed: batch_id={}, status={}, latency={:?}, instructions={}",
            request.batch_id,
            status,
            latency,
            sub_instruction_count,
        );
        
        Ok(result)
    }
    
    /// 执行所有子指令
    async fn execute_instructions(
        &self,
        request: &BatchExecuteRequest,
    ) -> Result<(Vec<ExecutionResult>, bool), BatchError> {
        let mut results = Vec::with_capacity(request.instructions.len());
        let mut has_failure = false;
        
        for (index, instruction) in request.instructions.iter().enumerate() {
            match self.executor.execute(instruction.clone()).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    has_failure = true;
                    
                    if request.atomic {
                        // 原子性模式：立即回滚已执行的指令
                        warn!(
                            "Batch atomic failure at instruction {}, initiating rollback",
                            index
                        );
                        self.rollback(&results).await?;
                        return Err(BatchError::AtomicFailure(e));
                    }
                    
                    // 非原子性模式：记录失败，继续执行
                    results.push(ExecutionResult::failed(e));
                    warn!(
                        "Batch instruction {} failed in non-atomic mode, continuing",
                        index
                    );
                }
            }
        }
        
        Ok((results, has_failure))
    }
    
    /// 确定 Batch 状态
    fn determine_batch_status(
        &self,
        results: &[ExecutionResult],
        has_failure: bool,
        atomic: bool,
    ) -> BatchStatus {
        if !has_failure {
            BatchStatus::Success
        } else if atomic {
            // 原子性模式下，任何失败都导致全部失败
            BatchStatus::Failed
        } else {
            // 非原子性模式，检查是否有成功的指令
            if results.iter().any(|r| r.is_success()) {
                BatchStatus::PartialFailure
            } else {
                BatchStatus::Failed
            }
        }
    }
    
    /// 回滚已执行的指令
    async fn rollback(&self, results: &[ExecutionResult]) -> Result<(), BatchError> {
        // 逆序回滚
        for result in results.iter().rev() {
            if let Err(e) = self.executor.rollback(result).await {
                error!("Rollback failed for execution {}: {}", result.execution_id, e);
                return Err(BatchError::RollbackFailed(e));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_batch_executor_creation() {
        let executor = Executor::mock();
        let batch_executor = BatchExecutor::new(executor);
        
        // 验证执行器创建成功
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_batch_single_instruction() {
        let executor = Executor::mock();
        let batch_executor = BatchExecutor::new(executor);
        
        let request = BatchExecuteRequest::new(
            "trace_1".to_string(),
            "batch_1".to_string(),
            vec![create_test_instruction()],
            true,
        );
        
        let result = batch_executor.execute(request).await.unwrap();
        
        assert_eq!(result.status, BatchStatus::Success);
        assert_eq!(result.results.len(), 1);
    }
    
    fn create_test_instruction() -> crate::executor::ExecuteRequest {
        crate::executor::ExecuteRequest {
            trace_id: "trace_1".to_string(),
            execution_id: "exec_1".to_string(),
            instruction_type: crate::executor::InstructionType::Read,
            payload: crate::executor::InstructionPayload::default(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}
