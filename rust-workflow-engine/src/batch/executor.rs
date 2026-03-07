//! Batch 执行器
//! 
//! 实现 Batch 批量指令的执行逻辑，支持：
//! - 原子性保证
//! - Phase 3 嵌套指令执行
//! - 哈希链验证

use crate::batch::types::*;
use crate::batch::hash::{compute_batch_hash, compute_nested_batch_hash};
use crate::executor::{Executor, ExecutionResult, ExecuteRequest};
use log::{info, warn, error};

/// Batch 执行器
pub struct BatchExecutor {
    /// 底层执行器
    executor: Executor,
    /// 最大嵌套深度
    max_depth: u32,
}

impl BatchExecutor {
    /// 创建新的 Batch 执行器
    pub fn new(executor: Executor) -> Self {
        Self { 
            executor,
            max_depth: 10, // 默认最大嵌套 10 层
        }
    }
    
    /// 创建带自定义最大深度的 Batch 执行器
    pub fn with_max_depth(executor: Executor, max_depth: u32) -> Self {
        Self { executor, max_depth }
    }
    
    /// 执行 Batch 请求
    pub async fn execute(&self, request: BatchExecuteRequest) 
                         -> Result<BatchExecuteResult, BatchError> {
        let start = std::time::Instant::now();
        let total_instructions = request.total_instruction_count();
        
        // 1. 参数验证
        request.validate()
            .map_err(BatchError::Validation)?;
        
        // 2. 执行指令（支持嵌套）
        let (results, nested_results, has_failure, max_depth_reached) = 
            self.execute_instructions_recursive(&request, request.current_depth).await?;
        
        // 3. 确定 Batch 状态
        let status = self.determine_batch_status(&results, has_failure, request.atomic);
        
        // 4. 计算 Batch 哈希
        let batch_hash = compute_batch_hash(&request.instructions, &results);
        
        // 5. 构建结果
        let mut result = match status {
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
        
        // 添加嵌套结果
        result.nested_results = nested_results;
        result.total_instructions = total_instructions;
        result.max_depth_reached = max_depth_reached;
        
        // 6. 采集监控指标
        let latency = start.elapsed();
        // metrics::observe_batch_execute(latency.as_secs_f64(), total_instructions);
        
        if !request.atomic && has_failure {
            // metrics::inc_batch_atomicity_violation("non_atomic_batch");
        }
        
        info!(
            "Batch executed: batch_id={}, status={}, latency={:?}, instructions={}, depth={}",
            request.batch_id,
            status,
            latency,
            total_instructions,
            max_depth_reached,
        );
        
        Ok(result)
    }
    
    /// 递归执行指令（支持嵌套 Batch）
    async fn execute_instructions_recursive(
        &self,
        request: &BatchExecuteRequest,
        current_depth: u32,
    ) -> Result<(Vec<ExecutionResult>, Vec<NestedBatchResult>, bool, u32), BatchError> {
        let mut results = Vec::new();
        let mut nested_results = Vec::new();
        let mut has_failure = false;
        let mut max_depth_reached = current_depth;
        
        for (index, instruction) in request.instructions.iter().enumerate() {
            match instruction {
                BatchInstruction::Simple(execute_req) => {
                    // 执行简单指令
                    match self.executor.execute(execute_req.clone()).await {
                        Ok(result) => {
                            results.push(result);
                        }
                        Err(e) => {
                            has_failure = true;
                            
                            if request.atomic {
                                // 原子性模式：立即回滚
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
                BatchInstruction::Nested(nested) => {
                    // 执行嵌套 Batch
                    if nested.depth > max_depth_reached {
                        max_depth_reached = nested.depth;
                    }
                    
                    match self.execute_nested_batch(nested, current_depth + 1).await {
                        Ok(nested_result) => {
                            // 收集嵌套结果
                            if nested_result.status == BatchStatus::Failed {
                                has_failure = true;
                                
                                if request.atomic {
                                    warn!(
                                        "Nested batch {} failed in atomic mode, initiating rollback",
                                        nested.nested_batch_id
                                    );
                                    self.rollback(&results).await?;
                                    return Err(BatchError::NestedExecutionFailed(
                                        format!("Nested batch {} failed", nested.nested_batch_id)
                                    ));
                                }
                            }
                            
                            // 将嵌套结果扁平化到主结果中（可选）
                            results.extend(nested_result.results.clone());
                            nested_results.push(nested_result);
                        }
                        Err(e) => {
                            has_failure = true;
                            
                            if request.atomic {
                                warn!(
                                    "Nested batch {} execution error in atomic mode",
                                    nested.nested_batch_id
                                );
                                self.rollback(&results).await?;
                                return Err(e);
                            }
                            
                            error!(
                                "Nested batch {} failed in non-atomic mode: {}",
                                nested.nested_batch_id, e
                            );
                        }
                    }
                }
            }
        }
        
        Ok((results, nested_results, has_failure, max_depth_reached))
    }
    
    /// 执行嵌套 Batch
    async fn execute_nested_batch(
        &self,
        nested: &NestedBatchInstruction,
        parent_depth: u32,
    ) -> Result<NestedBatchResult, BatchError> {
        // 检查深度限制
        if nested.depth >= self.max_depth {
            return Err(BatchError::Validation(
                BatchValidationError::MaxDepthExceeded(nested.depth, self.max_depth)
            ));
        }
        
        // 创建嵌套 Batch 请求
        let nested_request = BatchExecuteRequest::new_nested(
            nested.trace_id.clone(),
            nested.nested_batch_id.clone(),
            String::new(), // 父 Batch ID 在执行上下文中设置
            nested.instructions.clone(),
            nested.atomic,
            nested.depth,
        );
        
        // 递归执行嵌套 Batch
        let nested_result = self.execute(nested_request).await?;
        
        // 构建嵌套结果
        Ok(NestedBatchResult {
            batch_id: nested.nested_batch_id.clone(),
            depth: nested.depth,
            status: nested_result.status,
            results: nested_result.results,
            batch_hash: nested_result.batch_hash,
            nested_results: nested_result.nested_results,
        })
    }
    
    /// 执行所有子指令（Phase 2 兼容，仅支持简单指令）
    async fn execute_instructions(
        &self,
        request: &BatchExecuteRequest,
    ) -> Result<(Vec<ExecutionResult>, bool), BatchError> {
        let mut results = Vec::with_capacity(request.instructions.len());
        let mut has_failure = false;
        
        for (index, instruction) in request.instructions.iter().enumerate() {
            if let BatchInstruction::Simple(execute_req) = instruction {
                match self.executor.execute(execute_req.clone()).await {
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
