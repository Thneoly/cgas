//! Batch 哈希链验证
//! 
//! 实现 Batch 指令的哈希链计算与验证，确保：
//! - 指令执行顺序可验证
//! - 嵌套 Batch 哈希链完整性
//! - 结果不可篡改

use crate::batch::types::*;
use crate::executor::{ExecuteRequest, ExecutionResult};
use sha2::{Sha256, Digest};
use serde_json;

/// 计算单个指令的哈希
pub fn compute_instruction_hash(instruction: &BatchInstruction) -> String {
    let data = match instruction {
        BatchInstruction::Simple(req) => {
            serde_json::to_vec(&req).unwrap_or_default()
        }
        BatchInstruction::Nested(nested) => {
            // 嵌套指令：计算嵌套 Batch 的哈希
            let mut hasher = Sha256::new();
            hasher.update(nested.nested_batch_id.as_bytes());
            hasher.update(nested.trace_id.as_bytes());
            hasher.update(&nested.depth.to_ne_bytes());
            
            // 递归计算子指令哈希
            for sub_instruction in &nested.instructions {
                let sub_hash = compute_instruction_hash(sub_instruction);
                hasher.update(sub_hash.as_bytes());
            }
            
            return format!("{:x}", hasher.finalize());
        }
    };
    
    let mut hasher = Sha256::new();
    hasher.update(&data);
    format!("{:x}", hasher.finalize())
}

/// 计算指令列表的哈希链
pub fn compute_instructions_hash(instructions: &[BatchInstruction]) -> String {
    let mut hasher = Sha256::new();
    
    // 初始哈希
    let mut current_hash = [0u8; 32];
    
    for (index, instruction) in instructions.iter().enumerate() {
        let instruction_hash = compute_instruction_hash(instruction);
        
        // 哈希链：H(current, index, instruction_hash)
        hasher.update(&current_hash);
        hasher.update(&index.to_ne_bytes());
        hasher.update(instruction_hash.as_bytes());
        
        current_hash = hasher.finalize_reset().into();
    }
    
    format!("{:x}", current_hash)
}

/// 计算执行结果的哈希
pub fn compute_results_hash(results: &[ExecutionResult]) -> String {
    let mut hasher = Sha256::new();
    
    for (index, result) in results.iter().enumerate() {
        let result_data = serde_json::to_vec(&result).unwrap_or_default();
        hasher.update(&index.to_ne_bytes());
        hasher.update(&result_data);
    }
    
    format!("{:x}", hasher.finalize())
}

/// 计算 Batch 哈希（指令 + 结果）
pub fn compute_batch_hash(
    instructions: &[BatchInstruction],
    results: &[ExecutionResult],
) -> String {
    let mut hasher = Sha256::new();
    
    // 1. 计算指令哈希
    let instructions_hash = compute_instructions_hash(instructions);
    hasher.update(instructions_hash.as_bytes());
    
    // 2. 计算结果哈希
    let results_hash = compute_results_hash(results);
    hasher.update(results_hash.as_bytes());
    
    // 3. 组合哈希
    format!("{:x}", hasher.finalize())
}

/// 计算嵌套 Batch 哈希
pub fn compute_nested_batch_hash(
    parent_hash: &str,
    nested_batch_id: &str,
    nested_hash: &str,
    depth: u32,
) -> String {
    let mut hasher = Sha256::new();
    
    hasher.update(parent_hash.as_bytes());
    hasher.update(nested_batch_id.as_bytes());
    hasher.update(nested_hash.as_bytes());
    hasher.update(&depth.to_ne_bytes());
    
    format!("{:x}", hasher.finalize())
}

/// 验证 Batch 哈希
pub fn verify_batch_hash(
    instructions: &[BatchInstruction],
    results: &[ExecutionResult],
    expected_hash: &str,
) -> Result<bool, BatchError> {
    let computed_hash = compute_batch_hash(instructions, results);
    
    if computed_hash == expected_hash {
        Ok(true)
    } else {
        Err(BatchError::HashVerificationFailed {
            expected: expected_hash.to_string(),
            actual: computed_hash,
        })
    }
}

/// 验证嵌套 Batch 哈希链
pub fn verify_nested_batch_hash(
    nested_result: &NestedBatchResult,
    parent_hash: &str,
) -> Result<bool, BatchError> {
    // 验证当前层级的哈希
    let computed_hash = compute_batch_hash_for_nested(nested_result);
    
    if computed_hash != nested_result.batch_hash {
        return Err(BatchError::HashVerificationFailed {
            expected: nested_result.batch_hash.clone(),
            actual: computed_hash,
        });
    }
    
    // 递归验证子嵌套
    for child_nested in &nested_result.nested_results {
        verify_nested_batch_hash(child_nested, &nested_result.batch_hash)?;
    }
    
    Ok(true)
}

/// 计算嵌套 Batch 结果的哈希
fn compute_batch_hash_for_nested(nested_result: &NestedBatchResult) -> String {
    let mut hasher = Sha256::new();
    
    hasher.update(nested_result.batch_id.as_bytes());
    hasher.update(&nested_result.depth.to_ne_bytes());
    
    // 计算结果哈希
    let results_hash = compute_results_hash(&nested_result.results);
    hasher.update(results_hash.as_bytes());
    
    // 计算子嵌套哈希
    for child in &nested_result.nested_results {
        let child_hash = compute_batch_hash_for_nested(child);
        hasher.update(child_hash.as_bytes());
    }
    
    format!("{:x}", hasher.finalize())
}

/// 构建 Batch 哈希链（用于审计）
pub struct BatchHashChain {
    /// 层级深度
    pub depth: u32,
    /// 当前层级哈希
    pub hash: String,
    /// 父层级哈希（顶层为 None）
    pub parent_hash: Option<String>,
    /// 子层级哈希列表
    pub child_hashes: Vec<String>,
}

impl BatchHashChain {
    /// 从 Batch 结果构建哈希链
    pub fn from_batch_result(result: &BatchExecuteResult) -> Self {
        Self {
            depth: 0,
            hash: result.batch_hash.clone(),
            parent_hash: None,
            child_hashes: result.nested_results
                .iter()
                .map(|nested| nested.batch_hash.clone())
                .collect(),
        }
    }
    
    /// 从嵌套结果构建哈希链
    pub fn from_nested_result(
        nested_result: &NestedBatchResult,
        parent_hash: Option<String>,
    ) -> Self {
        Self {
            depth: nested_result.depth,
            hash: nested_result.batch_hash.clone(),
            parent_hash,
            child_hashes: nested_result.nested_results
                .iter()
                .map(|child| child.batch_hash.clone())
                .collect(),
        }
    }
    
    /// 验证哈希链完整性
    pub fn verify_chain(&self) -> bool {
        // 简单验证：确保所有哈希都是有效的 SHA256
        self.hash.len() == 64 && self.hash.chars().all(|c| c.is_ascii_hexdigit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::InstructionType;
    
    #[test]
    fn test_instruction_hash() {
        let instruction = BatchInstruction::Simple(ExecuteRequest {
            trace_id: "trace_1".to_string(),
            execution_id: "exec_1".to_string(),
            instruction_type: InstructionType::Read,
            payload: crate::executor::InstructionPayload::default(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
        
        let hash1 = compute_instruction_hash(&instruction);
        let hash2 = compute_instruction_hash(&instruction);
        
        // 相同指令应产生相同哈希
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 十六进制长度
    }
    
    #[test]
    fn test_instructions_hash_chain() {
        let instructions = vec![
            BatchInstruction::Simple(ExecuteRequest {
                trace_id: "trace_1".to_string(),
                execution_id: "exec_1".to_string(),
                instruction_type: InstructionType::Read,
                payload: crate::executor::InstructionPayload::default(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
            BatchInstruction::Simple(ExecuteRequest {
                trace_id: "trace_1".to_string(),
                execution_id: "exec_2".to_string(),
                instruction_type: InstructionType::Write,
                payload: crate::executor::InstructionPayload::default(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
        ];
        
        let hash = compute_instructions_hash(&instructions);
        assert_eq!(hash.len(), 64);
        
        // 顺序不同应产生不同哈希
        let reversed = vec![instructions[1].clone(), instructions[0].clone()];
        let reversed_hash = compute_instructions_hash(&reversed);
        assert_ne!(hash, reversed_hash);
    }
    
    #[test]
    fn test_nested_instruction_hash() {
        let nested = NestedBatchInstruction {
            nested_batch_id: "nested_1".to_string(),
            trace_id: "trace_1".to_string(),
            depth: 1,
            atomic: true,
            instructions: vec![
                BatchInstruction::Simple(ExecuteRequest {
                    trace_id: "trace_1".to_string(),
                    execution_id: "exec_1".to_string(),
                    instruction_type: InstructionType::Read,
                    payload: crate::executor::InstructionPayload::default(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }),
            ],
        };
        
        let hash = compute_instruction_hash(&BatchInstruction::Nested(nested));
        assert_eq!(hash.len(), 64);
    }
    
    #[test]
    fn test_batch_hash_verification() {
        let instructions = vec![
            BatchInstruction::Simple(ExecuteRequest {
                trace_id: "trace_1".to_string(),
                execution_id: "exec_1".to_string(),
                instruction_type: InstructionType::Read,
                payload: crate::executor::InstructionPayload::default(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
        ];
        
        let results = vec![ExecutionResult::success(
            "exec_1".to_string(),
            serde_json::json!({"result": "success"}),
        )];
        
        let hash = compute_batch_hash(&instructions, &results);
        
        // 验证应成功
        assert!(verify_batch_hash(&instructions, &results, &hash).is_ok());
        
        // 篡改结果后验证应失败
        let mut tampered_results = results.clone();
        tampered_results[0] = ExecutionResult::success(
            "exec_1".to_string(),
            serde_json::json!({"result": "tampered"}),
        );
        
        assert!(verify_batch_hash(&instructions, &tampered_results, &hash).is_err());
    }
    
    #[test]
    fn test_hash_chain_building() {
        let result = BatchExecuteResult::success(
            "trace_1".to_string(),
            "batch_1".to_string(),
            vec![],
            "test_hash".to_string(),
        );
        
        let chain = BatchHashChain::from_batch_result(&result);
        
        assert!(chain.verify_chain());
        assert_eq!(chain.depth, 0);
        assert!(chain.parent_hash.is_none());
    }
}
