//! Transaction 执行器
//! 
//! 实现 Transaction 事务的执行逻辑，支持 BEGIN/COMMIT/ROLLBACK 语义

use crate::transaction::types::*;
use crate::executor::{Executor, ExecutionResult, StateDiffOperation};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::{info, warn, error};

/// Transaction 执行器
pub struct TransactionExecutor {
    /// 底层执行器
    executor: Executor,
    /// 事务上下文管理器
    context_manager: Arc<Mutex<TransactionContextManager>>,
}

impl TransactionExecutor {
    /// 创建新的 Transaction 执行器
    pub fn new(executor: Executor) -> Self {
        Self {
            executor,
            context_manager: Arc::new(Mutex::new(TransactionContextManager::new())),
        }
    }
    
    /// 开始事务
    pub fn begin(&self, request: BeginTransactionRequest) 
                 -> Result<BeginTransactionResponse, TransactionError> {
        // 1. 验证请求
        request.validate()
            .map_err(TransactionError::Validation)?;
        
        // 2. 检查事务 ID 是否已存在
        let manager = self.context_manager.lock().unwrap();
        if manager.contexts.contains_key(&request.transaction_id) {
            return Err(TransactionError::TransactionAlreadyExists(
                request.transaction_id,
            ));
        }
        drop(manager);
        
        // 3. 创建事务上下文
        let mut context = TransactionContext::new(
            request.transaction_id.clone(),
            request.trace_id.clone(),
            request.isolation_level,
            request.timeout_ms,
        );
        context.status = TransactionStatus::Active;
        
        // 4. 保存上下文
        let mut manager = self.context_manager.lock().unwrap();
        manager.contexts.insert(request.transaction_id.clone(), context);
        drop(manager);
        
        info!("Transaction started: transaction_id={}, trace_id={}", 
              request.transaction_id, request.trace_id);
        
        // 5. 返回响应
        Ok(BeginTransactionResponse {
            trace_id: request.trace_id,
            transaction_id: request.transaction_id,
            status: TransactionStatus::Active,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// 执行事务内指令
    pub fn execute(&self, request: TransactionExecuteRequest) 
                   -> Result<TransactionExecuteResult, TransactionError> {
        // 1. 获取事务上下文
        let mut manager = self.context_manager.lock().unwrap();
        let context = manager.contexts.get_mut(&request.transaction_id)
            .ok_or_else(|| TransactionError::TransactionNotFound(
                request.transaction_id.clone(),
            ))?;
        
        // 2. 检查事务状态
        if context.status != TransactionStatus::Active {
            return Err(TransactionError::InvalidStatus {
                expected: "ACTIVE".to_string(),
                actual: context.status.to_string(),
            });
        }
        
        // 3. 检查超时
        if context.is_timeout() {
            // 自动回滚
            context.status = TransactionStatus::RolledBack;
            return Err(TransactionError::TransactionTimeout);
        }
        
        // 4. 更新事务状态为 Executing
        let old_status = context.status.clone();
        context.status = TransactionStatus::Executing;
        
        // 5. 执行所有指令
        let mut results = Vec::with_capacity(request.instructions.len());
        let mut has_failure = false;
        
        for instruction in &request.instructions {
            match self.executor.execute(instruction.clone()) {
                Ok(result) => {
                    // 累积 state_diff
                    context.accumulate_diff(result.state_diff.clone());
                    context.add_instruction(instruction.clone(), result.clone());
                    results.push(result);
                }
                Err(e) => {
                    has_failure = true;
                    error!("Transaction instruction execution failed: {}", e);
                    // 事务模式下，任何失败都导致事务失败
                    context.status = TransactionStatus::RolledBack;
                    return Err(TransactionError::ExecutorError(e));
                }
            }
        }
        
        // 6. 恢复事务状态为 Active
        if !has_failure {
            context.status = TransactionStatus::Active;
        }
        
        info!("Transaction executed: transaction_id={}, instructions={}, has_failure={}", 
              request.transaction_id, request.instructions.len(), has_failure);
        
        // 7. 返回结果
        Ok(TransactionExecuteResult {
            trace_id: request.transaction_id.clone(),
            transaction_id: request.transaction_id,
            status: if has_failure { ExecutionStatus::Failed } else { ExecutionStatus::Success },
            results,
            accumulated_diff: context.accumulated_diff.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// 提交事务
    pub fn commit(&self, request: CommitTransactionRequest) 
                  -> Result<CommitTransactionResponse, TransactionError> {
        // 1. 获取事务上下文
        let mut manager = self.context_manager.lock().unwrap();
        let context = manager.contexts.get_mut(&request.transaction_id)
            .ok_or_else(|| TransactionError::TransactionNotFound(
                request.transaction_id.clone(),
            ))?;
        
        // 2. 检查事务状态
        if context.status != TransactionStatus::Active {
            return Err(TransactionError::InvalidStatus {
                expected: "ACTIVE".to_string(),
                actual: context.status.to_string(),
            });
        }
        
        // 3. 验证事务哈希
        let computed_hash = compute_transaction_hash(
            &context.transaction_id,
            &context.isolation_level,
            &context.executed_instructions,
            &context.execution_results,
            &context.accumulated_diff,
        );
        
        if computed_hash != request.transaction_hash {
            return Err(TransactionError::HashMismatch {
                expected: request.transaction_hash,
                computed: computed_hash,
            });
        }
        
        // 4. 提交 state_diff 到状态存储
        // TODO: 实际提交到状态存储
        for diff in &context.accumulated_diff {
            // self.state_store.apply(diff)?;
        }
        
        // 5. 更新事务状态
        context.status = TransactionStatus::Committed;
        
        // 6. 计算提交确认哈希
        let commit_hash = compute_commit_hash(&request.transaction_id, &computed_hash);
        
        info!("Transaction committed: transaction_id={}, hash={}", 
              request.transaction_id, commit_hash);
        
        // 7. 清理事务上下文
        manager.contexts.remove(&request.transaction_id);
        drop(manager);
        
        // 8. 返回响应
        Ok(CommitTransactionResponse {
            trace_id: request.transaction_id.clone(),
            transaction_id: request.transaction_id,
            status: CommitStatus::Success,
            timestamp: chrono::Utc::now().to_rfc3339(),
            commit_hash,
        })
    }
    
    /// 回滚事务
    pub fn rollback(&self, request: RollbackTransactionRequest) 
                    -> Result<RollbackTransactionResponse, TransactionError> {
        // 1. 获取事务上下文
        let mut manager = self.context_manager.lock().unwrap();
        let context = manager.contexts.get_mut(&request.transaction_id)
            .ok_or_else(|| TransactionError::TransactionNotFound(
                request.transaction_id.clone(),
            ))?;
        
        // 2. 检查事务状态 (Active 或 Executing 都可以回滚)
        if context.status != TransactionStatus::Active && 
           context.status != TransactionStatus::Executing {
            return Err(TransactionError::InvalidStatus {
                expected: "ACTIVE or EXECUTING".to_string(),
                actual: context.status.to_string(),
            });
        }
        
        info!("Transaction rollback initiated: transaction_id={}, reason={}", 
              request.transaction_id, request.reason);
        
        // 3. 回滚 state_diff
        // TODO: 实际回滚操作
        // 由于 state_diff 尚未提交，只需丢弃即可
        
        // 4. 更新事务状态
        context.status = TransactionStatus::RolledBack;
        
        // 5. 清理事务上下文
        manager.contexts.remove(&request.transaction_id);
        drop(manager);
        
        info!("Transaction rolled back: transaction_id={}", request.transaction_id);
        
        // 6. 返回响应
        Ok(RollbackTransactionResponse {
            trace_id: request.transaction_id.clone(),
            transaction_id: request.transaction_id,
            status: RollbackStatus::Success,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// 检查事务状态
    pub fn get_transaction_status(&self, transaction_id: &str) 
                                  -> Result<TransactionStatus, TransactionError> {
        let manager = self.context_manager.lock().unwrap();
        let context = manager.contexts.get(transaction_id)
            .ok_or_else(|| TransactionError::TransactionNotFound(
                transaction_id.to_string(),
            ))?;
        
        Ok(context.status.clone())
    }
}

/// 事务上下文管理器
pub struct TransactionContextManager {
    /// 事务上下文映射表
    pub contexts: HashMap<String, TransactionContext>,
}

impl TransactionContextManager {
    /// 创建新的事务上下文管理器
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }
    
    /// 获取事务数量
    pub fn count(&self) -> usize {
        self.contexts.len()
    }
    
    /// 清理超时事务
    pub fn cleanup_timeout_transactions(&mut self) -> Vec<String> {
        let mut removed = Vec::new();
        
        for (id, context) in &self.contexts {
            if context.is_timeout() {
                removed.push(id.clone());
            }
        }
        
        for id in &removed {
            self.contexts.remove(id);
            info!("Timeout transaction cleaned up: transaction_id={}", id);
        }
        
        removed
    }
}

impl Default for TransactionContextManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Transaction 错误
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("验证失败：{0}")]
    Validation(#[from] TransactionValidationError),
    
    #[error("事务不存在：{0}")]
    TransactionNotFound(String),
    
    #[error("事务已存在：{0}")]
    TransactionAlreadyExists(String),
    
    #[error("事务状态错误：期望 {expected}, 实际 {actual}")]
    InvalidStatus { expected: String, actual: String },
    
    #[error("事务超时")]
    TransactionTimeout,
    
    #[error("执行器错误：{0}")]
    ExecutorError(#[source] crate::executor::ExecutorError),
    
    #[error("哈希不匹配：期望 {expected}, 计算 {computed}")]
    HashMismatch { expected: String, computed: String },
    
    #[error("内部错误：{0}")]
    Internal(String),
}

/// 计算 Transaction 哈希
pub fn compute_transaction_hash(
    transaction_id: &str,
    isolation_level: &IsolationLevel,
    instructions: &[ExecuteRequest],
    results: &[ExecutionResult],
    accumulated_diff: &[StateDiffOperation],
) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    
    // 哈希输入 1: transaction_id
    hasher.update(transaction_id.as_bytes());
    hasher.update(b"\x00");
    
    // 哈希输入 2: isolation_level
    let isolation_str = match isolation_level {
        IsolationLevel::ReadCommitted => "RC",
    };
    hasher.update(isolation_str.as_bytes());
    hasher.update(b"\x00");
    
    // 哈希输入 3: 指令数量
    hasher.update((instructions.len() as u64).to_be_bytes());
    hasher.update(b"\x00");
    
    // 哈希输入 4: 所有子指令 trace_id
    for instruction in instructions {
        hasher.update(instruction.trace_id.as_bytes());
        hasher.update(b"\x00");
    }
    
    // 哈希输入 5: 所有子指令结果 result_hash
    for result in results {
        hasher.update(result.result_hash.as_bytes());
        hasher.update(b"\x00");
    }
    
    // 哈希输入 6: accumulated_diff_hash
    let diff_hash = compute_diff_hash(accumulated_diff);
    hasher.update(diff_hash.as_bytes());
    
    format!("{:x}", hasher.finalize())
}

/// 计算 state_diff 哈希
pub fn compute_diff_hash(diff: &[StateDiffOperation]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    
    for op in diff {
        // 序列化操作并哈希
        let serialized = serde_json::to_string(op).unwrap_or_default();
        hasher.update(serialized.as_bytes());
        hasher.update(b"\x00");
    }
    
    format!("{:x}", hasher.finalize())
}

/// 计算提交确认哈希
pub fn compute_commit_hash(transaction_id: &str, transaction_hash: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    
    hasher.update(transaction_id.as_bytes());
    hasher.update(b"\x00");
    hasher.update(transaction_hash.as_bytes());
    
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transaction_executor_creation() {
        let executor = Executor::mock();
        let tx_executor = TransactionExecutor::new(executor);
        
        let manager = tx_executor.context_manager.lock().unwrap();
        assert_eq!(manager.count(), 0);
    }
    
    #[test]
    fn test_transaction_hash_computation() {
        // 测试哈希计算确定性
        let instructions = vec![];
        let results = vec![];
        let diff = vec![];
        
        let hash1 = compute_transaction_hash(
            "txn_1",
            &IsolationLevel::ReadCommitted,
            &instructions,
            &results,
            &diff,
        );
        
        let hash2 = compute_transaction_hash(
            "txn_1",
            &IsolationLevel::ReadCommitted,
            &instructions,
            &results,
            &diff,
        );
        
        assert_eq!(hash1, hash2);
    }
}
