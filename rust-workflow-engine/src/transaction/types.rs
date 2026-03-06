//! Transaction 指令数据类型
//! 
//! 定义 Transaction 事务的状态、上下文、请求/结果等核心数据结构

use serde::{Deserialize, Serialize};
use crate::executor::{ExecuteRequest, ExecutionResult, StateDiffOperation};

/// Transaction 状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    /// 事务已创建
    Created,
    /// 事务活跃中
    Active,
    /// 事务执行中
    Executing,
    /// 事务已提交
    Committed,
    /// 事务已回滚
    RolledBack,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Created => write!(f, "CREATED"),
            TransactionStatus::Active => write!(f, "ACTIVE"),
            TransactionStatus::Executing => write!(f, "EXECUTING"),
            TransactionStatus::Committed => write!(f, "COMMITTED"),
            TransactionStatus::RolledBack => write!(f, "ROLLEDBACK"),
        }
    }
}

/// 隔离级别 (Phase 2 仅支持 RC)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Read Committed (Phase 2)
    ReadCommitted,
    // RepeatableRead,  // Phase 3 扩展
    // Serializable,    // Phase 3 扩展
}

impl std::fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsolationLevel::ReadCommitted => write!(f, "READ_COMMITTED"),
        }
    }
}

/// Transaction 上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionContext {
    /// 事务 ID
    pub transaction_id: String,
    /// 事务 trace ID
    pub trace_id: String,
    /// 事务状态
    pub status: TransactionStatus,
    /// 隔离级别
    pub isolation_level: IsolationLevel,
    /// 超时时间 (毫秒)
    pub timeout_ms: i64,
    /// 创建时间 (Unix 时间戳毫秒)
    pub created_at: i64,
    /// 最后活动时间 (Unix 时间戳毫秒)
    pub last_activity_at: i64,
    /// 累积的 state_diff
    pub accumulated_diff: Vec<StateDiffOperation>,
    /// 执行的指令列表
    pub executed_instructions: Vec<ExecuteRequest>,
    /// 指令执行结果
    pub execution_results: Vec<ExecutionResult>,
}

impl TransactionContext {
    /// 创建新的事务上下文
    pub fn new(
        transaction_id: String,
        trace_id: String,
        isolation_level: IsolationLevel,
        timeout_ms: i64,
    ) -> Self {
        let now = current_timestamp_ms();
        Self {
            transaction_id,
            trace_id,
            status: TransactionStatus::Created,
            isolation_level,
            timeout_ms,
            created_at: now,
            last_activity_at: now,
            accumulated_diff: Vec::new(),
            executed_instructions: Vec::new(),
            execution_results: Vec::new(),
        }
    }
    
    /// 检查事务是否超时
    pub fn is_timeout(&self) -> bool {
        let now = current_timestamp_ms();
        let elapsed = now - self.created_at;
        elapsed > self.timeout_ms
    }
    
    /// 检查最后活动时间是否超时
    pub fn is_activity_timeout(&self) -> bool {
        let now = current_timestamp_ms();
        let elapsed = now - self.last_activity_at;
        elapsed > self.timeout_ms
    }
    
    /// 更新最后活动时间
    pub fn update_activity(&mut self) {
        self.last_activity_at = current_timestamp_ms();
    }
    
    /// 添加执行的指令
    pub fn add_instruction(&mut self, instruction: ExecuteRequest, result: ExecutionResult) {
        self.executed_instructions.push(instruction);
        self.execution_results.push(result);
        self.update_activity();
    }
    
    /// 累积 state_diff
    pub fn accumulate_diff(&mut self, diff: Vec<StateDiffOperation>) {
        self.accumulated_diff.extend(diff);
        self.update_activity();
    }
}

/// 事务开始请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeginTransactionRequest {
    /// 事务 trace ID
    pub trace_id: String,
    /// 事务唯一标识
    pub transaction_id: String,
    /// 隔离级别
    pub isolation_level: IsolationLevel,
    /// 超时时间 (毫秒)
    pub timeout_ms: i64,
    /// 请求时间戳
    pub timestamp: String,
}

impl BeginTransactionRequest {
    /// 创建新的事务开始请求
    pub fn new(
        trace_id: String,
        transaction_id: String,
        isolation_level: IsolationLevel,
        timeout_ms: i64,
    ) -> Self {
        Self {
            trace_id,
            transaction_id,
            isolation_level,
            timeout_ms,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// 验证请求
    pub fn validate(&self) -> Result<(), TransactionValidationError> {
        if self.trace_id.is_empty() {
            return Err(TransactionValidationError::InvalidTraceId);
        }
        if self.transaction_id.is_empty() {
            return Err(TransactionValidationError::InvalidTransactionId);
        }
        if self.timeout_ms < 1000 || self.timeout_ms > 60000 {
            return Err(TransactionValidationError::InvalidTimeout(
                self.timeout_ms,
            ));
        }
        Ok(())
    }
}

/// 事务开始响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeginTransactionResponse {
    /// 事务 trace ID
    pub trace_id: String,
    /// 事务唯一标识
    pub transaction_id: String,
    /// 事务状态
    pub status: TransactionStatus,
    /// 响应时间戳
    pub timestamp: String,
}

/// 事务执行请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionExecuteRequest {
    /// 事务 trace ID
    pub trace_id: String,
    /// 事务唯一标识
    pub transaction_id: String,
    /// 要执行的指令列表
    pub instructions: Vec<ExecuteRequest>,
    /// 请求时间戳
    pub timestamp: String,
}

/// 事务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionExecuteResult {
    /// 事务 trace ID
    pub trace_id: String,
    /// 事务唯一标识
    pub transaction_id: String,
    /// 执行状态
    pub status: ExecutionStatus,
    /// 指令执行结果
    pub results: Vec<ExecutionResult>,
    /// 累积的 state_diff (未提交)
    pub accumulated_diff: Vec<StateDiffOperation>,
    /// 响应时间戳
    pub timestamp: String,
}

/// 事务提交请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitTransactionRequest {
    /// 事务 trace ID
    pub trace_id: String,
    /// 事务唯一标识
    pub transaction_id: String,
    /// 事务哈希
    pub transaction_hash: String,
    /// 请求时间戳
    pub timestamp: String,
}

/// 事务提交响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitTransactionResponse {
    /// 事务 trace ID
    pub trace_id: String,
    /// 事务唯一标识
    pub transaction_id: String,
    /// 提交状态
    pub status: CommitStatus,
    /// 响应时间戳
    pub timestamp: String,
    /// 提交确认哈希
    pub commit_hash: String,
}

/// 提交状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommitStatus {
    /// 提交成功
    Success,
    /// 提交失败
    Failed,
}

/// 事务回滚请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackTransactionRequest {
    /// 事务 trace ID
    pub trace_id: String,
    /// 事务唯一标识
    pub transaction_id: String,
    /// 回滚原因
    pub reason: String,
    /// 请求时间戳
    pub timestamp: String,
}

/// 事务回滚响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackTransactionResponse {
    /// 事务 trace ID
    pub trace_id: String,
    /// 事务唯一标识
    pub transaction_id: String,
    /// 回滚状态
    pub status: RollbackStatus,
    /// 响应时间戳
    pub timestamp: String,
}

/// 回滚状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RollbackStatus {
    /// 回滚成功
    Success,
    /// 回滚失败
    Failed,
}

/// 执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// 执行成功
    Success,
    /// 执行失败
    Failed,
    /// 执行中
    Executing,
}

/// Transaction 验证错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum TransactionValidationError {
    #[error("trace_id 无效")]
    InvalidTraceId,
    
    #[error("transaction_id 无效")]
    InvalidTransactionId,
    
    #[error("超时时间无效：{0}ms (有效范围：1000-60000ms)")]
    InvalidTimeout(i64),
    
    #[error("事务不存在：{0}")]
    TransactionNotFound(String),
    
    #[error("事务状态错误：期望 {expected}, 实际 {actual}")]
    InvalidStatus { expected: String, actual: String },
}

/// 获取当前 Unix 时间戳 (毫秒)
fn current_timestamp_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transaction_context_creation() {
        let ctx = TransactionContext::new(
            "txn_1".to_string(),
            "trace_1".to_string(),
            IsolationLevel::ReadCommitted,
            5000,
        );
        
        assert_eq!(ctx.transaction_id, "txn_1");
        assert_eq!(ctx.trace_id, "trace_1");
        assert_eq!(ctx.status, TransactionStatus::Created);
        assert_eq!(ctx.isolation_level, IsolationLevel::ReadCommitted);
        assert_eq!(ctx.timeout_ms, 5000);
        assert!(ctx.accumulated_diff.is_empty());
        assert!(ctx.executed_instructions.is_empty());
        assert!(ctx.execution_results.is_empty());
    }
    
    #[test]
    fn test_transaction_timeout() {
        let mut ctx = TransactionContext::new(
            "txn_1".to_string(),
            "trace_1".to_string(),
            IsolationLevel::ReadCommitted,
            100, // 100ms 超时
        );
        
        // 初始不应超时
        assert!(!ctx.is_timeout());
        
        // 等待超时
        std::thread::sleep(std::time::Duration::from_millis(150));
        
        // 应该超时
        assert!(ctx.is_timeout());
    }
    
    #[test]
    fn test_begin_transaction_request_validation() {
        // 有效请求
        let req = BeginTransactionRequest::new(
            "trace_1".to_string(),
            "txn_1".to_string(),
            IsolationLevel::ReadCommitted,
            5000,
        );
        assert!(req.validate().is_ok());
        
        // 无效 trace_id
        let req = BeginTransactionRequest::new(
            "".to_string(),
            "txn_1".to_string(),
            IsolationLevel::ReadCommitted,
            5000,
        );
        assert!(req.validate().is_err());
        
        // 无效 timeout (太短)
        let req = BeginTransactionRequest::new(
            "trace_1".to_string(),
            "txn_1".to_string(),
            IsolationLevel::ReadCommitted,
            500,
        );
        assert!(req.validate().is_err());
        
        // 无效 timeout (太长)
        let req = BeginTransactionRequest::new(
            "trace_1".to_string(),
            "txn_1".to_string(),
            IsolationLevel::ReadCommitted,
            70000,
        );
        assert!(req.validate().is_err());
    }
}
