//! Transaction 指令模块
//! 
//! 实现 Transaction 事务指令的完整功能，包括：
//! - BEGIN/COMMIT/ROLLBACK 事务控制
//! - Read Committed 隔离级别
//! - 超时自动回滚
//! - 事务哈希链验证

pub mod types;
pub mod executor;
pub mod hash;

pub use types::*;
pub use executor::{TransactionExecutor, TransactionContextManager, TransactionError};
pub use hash::{
    compute_transaction_hash,
    compute_diff_hash,
    compute_commit_hash,
    verify_transaction_hash,
    verify_commit_hash,
};
