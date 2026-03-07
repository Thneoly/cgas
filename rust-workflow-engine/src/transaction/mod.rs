//! Transaction 指令模块
//! 
//! 实现 Transaction 事务指令的完整功能，包括：
//! - BEGIN/COMMIT/ROLLBACK 事务控制
//! - Read Committed / Repeatable Read 隔离级别
//! - 超时自动回滚
//! - 事务哈希链验证
//! - Phase 3: MVCC 多版本并发控制
//! - Phase 3 Week 2: 快照隔离

pub mod types;
pub mod executor;
pub mod hash;
pub mod repeatable_read;
pub mod mvcc;
pub mod snapshot;

pub use types::*;
pub use executor::{TransactionExecutor, TransactionContextManager, TransactionError};
pub use hash::{
    compute_transaction_hash,
    compute_diff_hash,
    compute_commit_hash,
    verify_transaction_hash,
    verify_commit_hash,
};
pub use repeatable_read::{
    RepeatableReadContext,
    RepeatableReadExecutor,
    RepeatableReadError,
    CommitResult,
};
pub use mvcc::{
    MvccManager,
    MvccError,
    StateVersion,
    TransactionSnapshot,
    MvccStats,
};
pub use snapshot::{
    SnapshotManager,
    SnapshotError,
    DataSnapshot,
    SnapshotStats,
    WriteOperation,
    WriteOpType,
};
