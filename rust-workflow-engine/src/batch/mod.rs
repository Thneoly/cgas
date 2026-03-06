//! Batch 指令模块
//! 
//! 实现 Batch 批量指令的完整功能，包括：
//! - 批量执行多条指令
//! - 原子性保证 (全部成功或全部失败)
//! - Batch 哈希链验证

pub mod types;
pub mod executor;
pub mod hash;

pub use types::*;
pub use executor::BatchExecutor;
pub use hash::{compute_batch_hash, verify_batch_hash};
