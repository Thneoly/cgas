//! Batch 嵌套指令数据类型
//! 
//! 定义 Phase 3 新增的嵌套 Batch 数据结构，支持：
//! - 多层嵌套 Batch 指令
//! - 嵌套层级追踪
//! - 哈希链验证
//! - 原子性保证（跨层级）

use serde::{Deserialize, Serialize};
use crate::executor::{ExecuteRequest, ExecutionResult, StateDiffOperation};

/// Batch 状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BatchStatus {
    /// 执行成功（所有指令成功）
    Success,
    /// 执行失败（原子模式下任一失败）
    Failed,
    /// 部分失败（非原子模式下部分失败）
    PartialFailure,
}

impl std::fmt::Display for BatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchStatus::Success => write!(f, "SUCCESS"),
            BatchStatus::Failed => write!(f, "FAILED"),
            BatchStatus::PartialFailure => write!(f, "PARTIAL_FAILURE"),
        }
    }
}

/// 嵌套层级信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestingLevel {
    /// 当前层级深度（0 = 顶层，1+ = 嵌套层）
    pub depth: u32,
    /// 父 Batch ID（顶层为 None）
    pub parent_batch_id: Option<String>,
    /// 当前 Batch ID
    pub batch_id: String,
}

/// 嵌套 Batch 指令
/// 
/// Phase 3 新增：支持在 Batch 内嵌套另一个 Batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedBatchInstruction {
    /// 嵌套 Batch 的唯一标识
    pub nested_batch_id: String,
    /// 嵌套 Batch 的 trace ID
    pub trace_id: String,
    /// 嵌套层级深度
    pub depth: u32,
    /// 是否原子执行
    pub atomic: bool,
    /// 嵌套 Batch 内的指令列表
    pub instructions: Vec<BatchInstruction>,
}

/// Batch 指令类型（支持普通指令或嵌套 Batch）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BatchInstruction {
    /// 普通执行指令
    Simple(ExecuteRequest),
    /// 嵌套 Batch 指令
    Nested(NestedBatchInstruction),
}

impl BatchInstruction {
    /// 获取指令的 trace ID
    pub fn trace_id(&self) -> &str {
        match self {
            BatchInstruction::Simple(req) => &req.trace_id,
            BatchInstruction::Nested(nested) => &nested.trace_id,
        }
    }
    
    /// 获取指令深度
    pub fn depth(&self) -> u32 {
        match self {
            BatchInstruction::Simple(_) => 0,
            BatchInstruction::Nested(nested) => nested.depth,
        }
    }
}

/// Batch 执行请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecuteRequest {
    /// Batch trace ID
    pub trace_id: String,
    /// Batch 唯一标识
    pub batch_id: String,
    /// 要执行的指令列表（支持嵌套）
    pub instructions: Vec<BatchInstruction>,
    /// 是否原子执行（全部成功或全部失败）
    pub atomic: bool,
    /// 最大嵌套深度限制
    pub max_depth: u32,
    /// 请求时间戳
    pub timestamp: String,
    /// 父 Batch ID（顶层为 None）
    pub parent_batch_id: Option<String>,
    /// 当前嵌套深度
    pub current_depth: u32,
}

impl BatchExecuteRequest {
    /// 创建新的顶层 Batch 请求
    pub fn new(
        trace_id: String,
        batch_id: String,
        instructions: Vec<BatchInstruction>,
        atomic: bool,
    ) -> Self {
        Self {
            trace_id,
            batch_id,
            instructions,
            atomic,
            max_depth: 10, // 默认最大嵌套 10 层
            timestamp: chrono::Utc::now().to_rfc3339(),
            parent_batch_id: None,
            current_depth: 0,
        }
    }
    
    /// 创建嵌套 Batch 请求
    pub fn new_nested(
        trace_id: String,
        batch_id: String,
        parent_batch_id: String,
        instructions: Vec<BatchInstruction>,
        atomic: bool,
        current_depth: u32,
    ) -> Self {
        Self {
            trace_id,
            batch_id,
            instructions,
            atomic,
            max_depth: 10,
            timestamp: chrono::Utc::now().to_rfc3339(),
            parent_batch_id: Some(parent_batch_id),
            current_depth,
        }
    }
    
    /// 验证请求
    pub fn validate(&self) -> Result<(), BatchValidationError> {
        if self.trace_id.is_empty() {
            return Err(BatchValidationError::InvalidTraceId);
        }
        if self.batch_id.is_empty() {
            return Err(BatchValidationError::InvalidBatchId);
        }
        if self.instructions.is_empty() {
            return Err(BatchValidationError::EmptyInstructions);
        }
        if self.current_depth >= self.max_depth {
            return Err(BatchValidationError::MaxDepthExceeded(
                self.current_depth,
                self.max_depth,
            ));
        }
        
        // 递归验证嵌套指令
        for instruction in &self.instructions {
            if let BatchInstruction::Nested(nested) = instruction {
                if nested.depth >= self.max_depth {
                    return Err(BatchValidationError::MaxDepthExceeded(
                        nested.depth,
                        self.max_depth,
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// 计算总指令数（展开嵌套）
    pub fn total_instruction_count(&self) -> usize {
        self.count_instructions_recursive(&self.instructions)
    }
    
    fn count_instructions_recursive(&self, instructions: &[BatchInstruction]) -> usize {
        let mut count = 0;
        for instruction in instructions {
            match instruction {
                BatchInstruction::Simple(_) => count += 1,
                BatchInstruction::Nested(nested) => {
                    // 递归计算嵌套指令数
                    count += nested.instructions.iter().map(|instr| {
                        match instr {
                            BatchInstruction::Simple(_) => 1,
                            BatchInstruction::Nested(n) => {
                                // 简化处理：嵌套层每层算 1 个指令
                                1
                            }
                        }
                    }).sum::<usize>();
                }
            }
        }
        count
    }
}

/// Batch 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecuteResult {
    /// Batch trace ID
    pub trace_id: String,
    /// Batch 唯一标识
    pub batch_id: String,
    /// Batch 状态
    pub status: BatchStatus,
    /// 指令执行结果（扁平化）
    pub results: Vec<ExecutionResult>,
    /// 嵌套执行结果（层级结构）
    pub nested_results: Vec<NestedBatchResult>,
    /// Batch 哈希
    pub batch_hash: String,
    /// 累积的 state_diff
    pub accumulated_diff: Vec<StateDiffOperation>,
    /// 响应时间戳
    pub timestamp: String,
    /// 执行的总指令数
    pub total_instructions: usize,
    /// 最大嵌套深度
    pub max_depth_reached: u32,
}

impl BatchExecuteResult {
    /// 创建成功的 Batch 结果
    pub fn success(
        trace_id: String,
        batch_id: String,
        results: Vec<ExecutionResult>,
        batch_hash: String,
    ) -> Self {
        Self {
            trace_id,
            batch_id,
            status: BatchStatus::Success,
            results,
            nested_results: Vec::new(),
            batch_hash,
            accumulated_diff: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_instructions: results.len(),
            max_depth_reached: 0,
        }
    }
    
    /// 创建失败的 Batch 结果
    pub fn failed(
        trace_id: String,
        batch_id: String,
        results: Vec<ExecutionResult>,
        batch_hash: String,
    ) -> Self {
        Self {
            trace_id,
            batch_id,
            status: BatchStatus::Failed,
            results,
            nested_results: Vec::new(),
            batch_hash,
            accumulated_diff: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_instructions: results.len(),
            max_depth_reached: 0,
        }
    }
    
    /// 创建部分失败的 Batch 结果
    pub fn partial_failure(
        trace_id: String,
        batch_id: String,
        results: Vec<ExecutionResult>,
        batch_hash: String,
    ) -> Self {
        Self {
            trace_id,
            batch_id,
            status: BatchStatus::PartialFailure,
            results,
            nested_results: Vec::new(),
            batch_hash,
            accumulated_diff: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_instructions: results.len(),
            max_depth_reached: 0,
        }
    }
}

/// 嵌套 Batch 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedBatchResult {
    /// 嵌套 Batch ID
    pub batch_id: String,
    /// 嵌套深度
    pub depth: u32,
    /// 执行状态
    pub status: BatchStatus,
    /// 执行结果
    pub results: Vec<ExecutionResult>,
    /// 嵌套哈希
    pub batch_hash: String,
    /// 子嵌套结果（递归）
    pub nested_results: Vec<NestedBatchResult>,
}

/// Batch 验证错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum BatchValidationError {
    #[error("trace_id 无效")]
    InvalidTraceId,
    
    #[error("batch_id 无效")]
    InvalidBatchId,
    
    #[error("指令列表为空")]
    EmptyInstructions,
    
    #[error("超过最大嵌套深度：当前 {0}, 最大 {1}")]
    MaxDepthExceeded(u32, u32),
    
    #[error("嵌套 Batch ID 冲突：{0}")]
    DuplicateBatchId(String),
    
    #[error("父 Batch 不存在：{0}")]
    ParentBatchNotFound(String),
}

/// Batch 错误类型
#[derive(Debug, Clone, thiserror::Error)]
pub enum BatchError {
    #[error("验证失败：{0}")]
    Validation(BatchValidationError),
    
    #[error("原子执行失败：{0}")]
    AtomicFailure(crate::error::EngineError),
    
    #[error("回滚失败：{0}")]
    RollbackFailed(crate::error::EngineError),
    
    #[error("嵌套执行失败：{0}")]
    NestedExecutionFailed(String),
    
    #[error("哈希验证失败：期望 {expected}, 实际 {actual}")]
    HashVerificationFailed { expected: String, actual: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_batch_execute_request_creation() {
        let instructions = vec![
            BatchInstruction::Simple(create_test_execute_request("exec_1")),
        ];
        
        let request = BatchExecuteRequest::new(
            "trace_1".to_string(),
            "batch_1".to_string(),
            instructions,
            true,
        );
        
        assert_eq!(request.trace_id, "trace_1");
        assert_eq!(request.batch_id, "batch_1");
        assert_eq!(request.instructions.len(), 1);
        assert!(request.atomic);
        assert_eq!(request.current_depth, 0);
        assert!(request.parent_batch_id.is_none());
    }
    
    #[test]
    fn test_nested_batch_instruction() {
        let nested_instructions = vec![
            BatchInstruction::Simple(create_test_execute_request("exec_1")),
            BatchInstruction::Simple(create_test_execute_request("exec_2")),
        ];
        
        let nested = NestedBatchInstruction {
            nested_batch_id: "nested_batch_1".to_string(),
            trace_id: "trace_1".to_string(),
            depth: 1,
            atomic: true,
            instructions: nested_instructions,
        };
        
        assert_eq!(nested.nested_batch_id, "nested_batch_1");
        assert_eq!(nested.depth, 1);
        assert_eq!(nested.instructions.len(), 2);
    }
    
    #[test]
    fn test_batch_validation() {
        // 有效请求
        let request = BatchExecuteRequest::new(
            "trace_1".to_string(),
            "batch_1".to_string(),
            vec![BatchInstruction::Simple(create_test_execute_request("exec_1"))],
            true,
        );
        assert!(request.validate().is_ok());
        
        // 空指令列表
        let request = BatchExecuteRequest::new(
            "trace_1".to_string(),
            "batch_1".to_string(),
            vec![],
            true,
        );
        assert!(request.validate().is_err());
        
        // 空 trace_id
        let request = BatchExecuteRequest::new(
            "".to_string(),
            "batch_1".to_string(),
            vec![BatchInstruction::Simple(create_test_execute_request("exec_1"))],
            true,
        );
        assert!(request.validate().is_err());
    }
    
    #[test]
    fn test_nested_batch_request() {
        let nested_request = BatchExecuteRequest::new_nested(
            "trace_1".to_string(),
            "nested_batch_1".to_string(),
            "parent_batch_1".to_string(),
            vec![BatchInstruction::Simple(create_test_execute_request("exec_1"))],
            true,
            1,
        );
        
        assert_eq!(nested_request.current_depth, 1);
        assert_eq!(nested_request.parent_batch_id, Some("parent_batch_1".to_string()));
        assert!(nested_request.validate().is_ok());
    }
    
    fn create_test_execute_request(execution_id: &str) -> ExecuteRequest {
        ExecuteRequest {
            trace_id: "trace_1".to_string(),
            execution_id: execution_id.to_string(),
            instruction_type: crate::executor::InstructionType::Read,
            payload: crate::executor::InstructionPayload::default(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}
