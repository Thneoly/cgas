//! Transaction 哈希链
//! 
//! 实现 Transaction 事务的哈希计算和验证

use sha2::{Sha256, Digest};
use crate::transaction::types::*;
use crate::executor::{ExecuteRequest, ExecutionResult, StateDiffOperation};

/// 计算 Transaction 哈希
/// 
/// 哈希覆盖：
/// 1. transaction_id
/// 2. isolation_level
/// 3. 指令数量
/// 4. 所有子指令的 trace_id (按顺序)
/// 5. 所有子指令结果的 result_hash (按顺序)
/// 6. accumulated_diff_hash
pub fn compute_transaction_hash(
    transaction_id: &str,
    isolation_level: &IsolationLevel,
    instructions: &[ExecuteRequest],
    results: &[ExecutionResult],
    accumulated_diff: &[StateDiffOperation],
) -> String {
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
    
    // 哈希输入 4: 所有子指令 trace_id (按顺序)
    for instruction in instructions {
        hasher.update(instruction.trace_id.as_bytes());
        hasher.update(b"\x00");
    }
    
    // 哈希输入 5: 所有子指令结果 result_hash (按顺序)
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
    let mut hasher = Sha256::new();
    
    hasher.update(transaction_id.as_bytes());
    hasher.update(b"\x00");
    hasher.update(transaction_hash.as_bytes());
    
    format!("{:x}", hasher.finalize())
}

/// 验证 Transaction 哈希
pub fn verify_transaction_hash(
    transaction_id: &str,
    isolation_level: &IsolationLevel,
    instructions: &[ExecuteRequest],
    results: &[ExecutionResult],
    accumulated_diff: &[StateDiffOperation],
    expected_hash: &str,
) -> bool {
    let computed_hash = compute_transaction_hash(
        transaction_id,
        isolation_level,
        instructions,
        results,
        accumulated_diff,
    );
    computed_hash == expected_hash
}

/// 验证提交哈希
pub fn verify_commit_hash(transaction_id: &str, transaction_hash: &str, expected_commit_hash: &str) -> bool {
    let computed_commit_hash = compute_commit_hash(transaction_id, transaction_hash);
    computed_commit_hash == expected_commit_hash
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transaction_hash_deterministic() {
        // 相同输入产生相同哈希
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
    
    #[test]
    fn test_transaction_hash_unique() {
        // 不同输入产生不同哈希
        let instructions1 = vec![];
        let instructions2 = vec![create_test_instruction("trace_1")];
        let results = vec![];
        let diff = vec![];
        
        let hash1 = compute_transaction_hash(
            "txn_1",
            &IsolationLevel::ReadCommitted,
            &instructions1,
            &results,
            &diff,
        );
        
        let hash2 = compute_transaction_hash(
            "txn_1",
            &IsolationLevel::ReadCommitted,
            &instructions2,
            &results,
            &diff,
        );
        
        assert_ne!(hash1, hash2);
    }
    
    #[test]
    fn test_transaction_hash_verify() {
        let instructions = vec![
            create_test_instruction("trace_1"),
            create_test_instruction("trace_2"),
        ];
        let results = vec![
            create_test_result("trace_1", "hash_1"),
            create_test_result("trace_2", "hash_2"),
        ];
        let diff = vec![];
        
        let hash = compute_transaction_hash(
            "txn_1",
            &IsolationLevel::ReadCommitted,
            &instructions,
            &results,
            &diff,
        );
        
        // 验证应该通过
        assert!(verify_transaction_hash(
            "txn_1",
            &IsolationLevel::ReadCommitted,
            &instructions,
            &results,
            &diff,
            &hash,
        ));
        
        // 错误哈希验证应该失败
        assert!(!verify_transaction_hash(
            "txn_1",
            &IsolationLevel::ReadCommitted,
            &instructions,
            &results,
            &diff,
            "wrong_hash",
        ));
    }
    
    #[test]
    fn test_commit_hash_verification() {
        let transaction_id = "txn_1";
        let transaction_hash = "abc123";
        
        let commit_hash = compute_commit_hash(transaction_id, transaction_hash);
        
        // 验证应该通过
        assert!(verify_commit_hash(transaction_id, transaction_hash, &commit_hash));
        
        // 错误哈希验证应该失败
        assert!(!verify_commit_hash(transaction_id, "wrong_hash", &commit_hash));
    }
    
    #[test]
    fn test_diff_hash_computation() {
        let diff1 = vec![];
        let diff2 = vec![create_test_diff_op()];
        
        let hash1 = compute_diff_hash(&diff1);
        let hash2 = compute_diff_hash(&diff2);
        
        // 空 diff 和非空 diff 哈希应该不同
        assert_ne!(hash1, hash2);
    }
    
    fn create_test_instruction(trace_id: &str) -> ExecuteRequest {
        ExecuteRequest {
            trace_id: trace_id.to_string(),
            execution_id: format!("exec_{}", trace_id),
            instruction_type: crate::executor::InstructionType::Read,
            payload: crate::executor::InstructionPayload::default(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    fn create_test_result(trace_id: &str, result_hash: &str) -> ExecutionResult {
        ExecutionResult {
            trace_id: trace_id.to_string(),
            execution_id: format!("exec_{}", trace_id),
            status: crate::executor::ExecutionStatus::Success,
            state_diff: vec![],
            result_hash: result_hash.to_string(),
            state_diff_hash: String::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    fn create_test_diff_op() -> StateDiffOperation {
        StateDiffOperation {
            op_type: crate::executor::DiffOpType::Insert,
            key: "test_key".to_string(),
            value: Some("test_value".to_string()),
        }
    }
}
