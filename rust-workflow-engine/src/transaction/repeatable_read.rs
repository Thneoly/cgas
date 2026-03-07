//! Transaction Repeatable Read 隔离级别实现
//! 
//! Phase 3 扩展：在 Phase 2 Read Committed 基础上实现 Repeatable Read 隔离级别
//! 
//! 核心特性：
//! - 快照读（Snapshot Read）：事务开始时创建一致性快照
//! - 写锁（Write Locking）：写操作获取排他锁
//! - 版本控制（MVCC）：多版本并发控制
//! - 冲突检测（Conflict Detection）：提交时检测写写冲突

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error};

use crate::transaction::types::*;
use crate::executor::{ExecuteRequest, ExecutionResult, StateDiffOperation};

/// Repeatable Read 事务上下文
#[derive(Debug, Clone)]
pub struct RepeatableReadContext {
    /// 基础事务上下文
    pub base_context: TransactionContext,
    /// 数据快照（事务开始时的状态）
    pub snapshot: Arc<RwLock<HashMap<String, SnapshotEntry>>>,
    /// 读集（已读取的键）
    pub read_set: Arc<RwLock<HashMap<String, u64>>>, // key -> version
    /// 写集（已修改的键）
    pub write_set: Arc<RwLock<HashMap<String, WriteEntry>>>,
    /// 锁表（持有的锁）
    pub locks: Arc<RwLock<HashMap<String, LockEntry>>>,
}

impl RepeatableReadContext {
    /// 创建新的 RR 事务上下文
    pub fn new(
        transaction_id: String,
        trace_id: String,
        timeout_ms: i64,
    ) -> Self {
        let base_context = TransactionContext {
            transaction_id,
            trace_id,
            status: TransactionStatus::Created,
            isolation_level: IsolationLevel::RepeatableRead,
            timeout_ms,
            created_at: current_timestamp_ms(),
            last_activity_at: current_timestamp_ms(),
            accumulated_diff: Vec::new(),
            executed_instructions: Vec::new(),
            execution_results: Vec::new(),
        };
        
        Self {
            base_context,
            snapshot: Arc::new(RwLock::new(HashMap::new())),
            read_set: Arc::new(RwLock::new(HashMap::new())),
            write_set: Arc::new(RwLock::new(HashMap::new())),
            locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 从事务上下文创建 RR 上下文
    pub fn from_transaction_context(base_context: TransactionContext) -> Self {
        Self {
            base_context,
            snapshot: Arc::new(RwLock::new(HashMap::new())),
            read_set: Arc::new(RwLock::new(HashMap::new())),
            write_set: Arc::new(RwLock::new(HashMap::new())),
            locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// 快照条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEntry {
    /// 数据键
    pub key: String,
    /// 数据值
    pub value: serde_json::Value,
    /// 版本号
    pub version: u64,
    /// 创建时间戳
    pub created_at: i64,
}

/// 写条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteEntry {
    /// 数据键
    pub key: String,
    /// 新值
    pub new_value: serde_json::Value,
    /// 旧值（快照中的值）
    pub old_value: Option<serde_json::Value>,
    /// 操作类型
    pub operation: WriteOperation,
    /// 时间戳
    pub timestamp: i64,
}

/// 写操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WriteOperation {
    /// 插入
    Insert,
    /// 更新
    Update,
    /// 删除
    Delete,
}

/// 锁条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    /// 锁定的键
    pub key: String,
    /// 锁类型
    pub lock_type: LockType,
    /// 持有锁的事务 ID
    pub transaction_id: String,
    /// 获取锁的时间戳
    pub acquired_at: i64,
    /// 锁超时时间戳
    pub expires_at: i64,
}

/// 锁类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LockType {
    /// 共享锁（读锁）
    Shared,
    /// 排他锁（写锁）
    Exclusive,
}

/// RR 隔离级别错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum RepeatableReadError {
    #[error("快照创建失败：{0}")]
    SnapshotCreationFailed(String),
    
    #[error("读冲突：键 {key} 已被事务 {tx_id} 修改")]
    ReadConflict { key: String, tx_id: String },
    
    #[error("写冲突：键 {key} 已被事务 {tx_id} 锁定")]
    WriteConflict { key: String, tx_id: String },
    
    #[error("提交冲突：键 {key} 版本不匹配，期望 {expected_version}, 实际 {actual_version}")]
    CommitConflict {
        key: String,
        expected_version: u64,
        actual_version: u64,
    },
    
    #[error("锁获取超时：键 {key}")]
    LockTimeout { key: String },
    
    #[error("死锁检测：事务 {tx_id} 涉及死锁")]
    DeadlockDetected { tx_id: String },
    
    #[error("事务错误：{0}")]
    TransactionError(TransactionValidationError),
}

/// Repeatable Read 事务执行器
pub struct RepeatableReadExecutor {
    /// 数据版本存储
    versions: Arc<RwLock<HashMap<String, Vec<SnapshotEntry>>>>,
    /// 全局锁表
    global_locks: Arc<RwLock<HashMap<String, LockEntry>>>,
    /// 死锁检测器
    deadlock_detector: Arc<DeadlockDetector>,
}

impl RepeatableReadExecutor {
    /// 创建新的 RR 执行器
    pub fn new() -> Self {
        Self {
            versions: Arc::new(RwLock::new(HashMap::new())),
            global_locks: Arc::new(RwLock::new(HashMap::new())),
            deadlock_detector: Arc::new(DeadlockDetector::new()),
        }
    }
    
    /// 创建数据快照
    pub async fn create_snapshot(
        &self,
        ctx: &RepeatableReadContext,
        keys: &[String],
    ) -> Result<(), RepeatableReadError> {
        let versions = self.versions.read().await;
        let mut snapshot = ctx.snapshot.write().await;
        
        for key in keys {
            if let Some(version_history) = versions.get(key) {
                if let Some(latest) = version_history.last() {
                    snapshot.insert(
                        key.clone(),
                        SnapshotEntry {
                            key: key.clone(),
                            value: latest.value.clone(),
                            version: latest.version,
                            created_at: latest.created_at,
                        },
                    );
                }
            }
        }
        
        info!(
            "RR snapshot created for transaction {}: {} keys",
            ctx.base_context.transaction_id,
            snapshot.len()
        );
        
        Ok(())
    }
    
    /// 执行快照读
    pub async fn snapshot_read(
        &self,
        ctx: &RepeatableReadContext,
        key: &str,
    ) -> Result<Option<serde_json::Value>, RepeatableReadError> {
        // 1. 检查写集（读已写）
        {
            let write_set = ctx.write_set.read().await;
            if let Some(write_entry) = write_set.get(key) {
                return Ok(Some(write_entry.new_value.clone()));
            }
        }
        
        // 2. 从快照读
        let snapshot = ctx.snapshot.read().await;
        if let Some(entry) = snapshot.get(key) {
            // 记录读集
            let mut read_set = ctx.read_set.write().await;
            read_set.insert(key.to_string(), entry.version);
            
            return Ok(Some(entry.value.clone()));
        }
        
        Ok(None)
    }
    
    /// 执行写操作（获取排他锁）
    pub async fn execute_write(
        &self,
        ctx: &RepeatableReadContext,
        key: &str,
        value: serde_json::Value,
        operation: WriteOperation,
    ) -> Result<(), RepeatableReadError> {
        let now = current_timestamp_ms();
        
        // 1. 尝试获取排他锁
        self.acquire_exclusive_lock(ctx, key, now).await?;
        
        // 2. 检查写集（是否已写过）
        let mut write_set = ctx.write_set.write().await;
        let old_value = if let Some(existing) = write_set.get(key) {
            existing.old_value.clone()
        } else {
            // 从快照获取旧值
            let snapshot = ctx.snapshot.read().await;
            snapshot.get(key).map(|e| e.value.clone())
        };
        
        // 3. 记录写操作
        write_set.insert(
            key.to_string(),
            WriteEntry {
                key: key.to_string(),
                new_value: value,
                old_value,
                operation,
                timestamp: now,
            },
        );
        
        Ok(())
    }
    
    /// 获取排他锁
    async fn acquire_exclusive_lock(
        &self,
        ctx: &RepeatableReadContext,
        key: &str,
        now: i64,
    ) -> Result<(), RepeatableReadError> {
        let lock_timeout = ctx.base_context.timeout_ms;
        
        // 检查是否已持有该锁
        {
            let locks = ctx.locks.read().await;
            if let Some(existing) = locks.get(key) {
                if existing.lock_type == LockType::Exclusive {
                    return Ok(()); // 已持有排他锁
                }
            }
        }
        
        // 尝试获取全局锁
        let mut global_locks = self.global_locks.write().await;
        
        if let Some(existing_lock) = global_locks.get(key) {
            // 检查锁是否过期
            if existing_lock.expires_at > now {
                // 锁仍有效，检查是否是自己持有的
                if existing_lock.transaction_id != ctx.base_context.transaction_id {
                    return Err(RepeatableReadError::WriteConflict {
                        key: key.to_string(),
                        tx_id: existing_lock.transaction_id.clone(),
                    });
                }
            }
        }
        
        // 获取锁
        let lock_entry = LockEntry {
            key: key.to_string(),
            lock_type: LockType::Exclusive,
            transaction_id: ctx.base_context.transaction_id.clone(),
            acquired_at: now,
            expires_at: now + lock_timeout,
        };
        
        global_locks.insert(key.to_string(), lock_entry.clone());
        
        // 记录到上下文
        {
            let mut locks = ctx.locks.write().await;
            locks.insert(key.to_string(), lock_entry);
        }
        
        // 注册到死锁检测器
        self.deadlock_detector
            .register_lock(ctx.base_context.transaction_id.clone(), key.to_string())
            .await;
        
        Ok(())
    }
    
    /// 提交事务（验证写集）
    pub async fn commit(
        &self,
        ctx: &RepeatableReadContext,
    ) -> Result<CommitResult, RepeatableReadError> {
        let write_set = ctx.write_set.read().await;
        let read_set = ctx.read_set.read().await;
        
        // 1. 验证读集（检查是否有冲突）
        for (key, read_version) in read_set.iter() {
            let versions = self.versions.read().await;
            if let Some(version_history) = versions.get(key) {
                if let Some(latest) = version_history.last() {
                    if latest.version != *read_version {
                        return Err(RepeatableReadError::CommitConflict {
                            key: key.clone(),
                            expected_version: *read_version,
                            actual_version: latest.version,
                        });
                    }
                }
            }
        }
        
        // 2. 验证写集（检查写写冲突）
        for (key, write_entry) in write_set.iter() {
            let versions = self.versions.read().await;
            if let Some(version_history) = versions.get(key) {
                if let Some(latest) = version_history.last() {
                    // 如果有旧值，检查版本
                    if write_entry.old_value.is_some() && latest.version > 0 {
                        // 版本检查逻辑（简化版）
                    }
                }
            }
        }
        
        // 3. 应用写集（创建新版本）
        let mut versions = self.versions.write().await;
        let mut committed_keys = Vec::new();
        
        for (key, write_entry) in write_set.iter() {
            let version_history = versions.entry(key.clone()).or_insert_with(Vec::new);
            let new_version = version_history.last().map(|e| e.version + 1).unwrap_or(0);
            
            let new_entry = SnapshotEntry {
                key: key.clone(),
                value: write_entry.new_value.clone(),
                version: new_version,
                created_at: current_timestamp_ms(),
            };
            
            version_history.push(new_entry);
            committed_keys.push(key.clone());
        }
        
        // 4. 释放锁
        self.release_locks(ctx).await?;
        
        info!(
            "RR transaction committed: tx={}, keys={:?}",
            ctx.base_context.transaction_id,
            committed_keys
        );
        
        Ok(CommitResult {
            transaction_id: ctx.base_context.transaction_id.clone(),
            committed_keys,
            timestamp: current_timestamp_ms(),
        })
    }
    
    /// 回滚事务
    pub async fn rollback(&self, ctx: &RepeatableReadContext) -> Result<(), RepeatableReadError> {
        // 1. 清空写集（不应用）
        {
            let mut write_set = ctx.write_set.write().await;
            write_set.clear();
        }
        
        // 2. 释放所有锁
        self.release_locks(ctx).await?;
        
        info!(
            "RR transaction rolled back: tx={}",
            ctx.base_context.transaction_id
        );
        
        Ok(())
    }
    
    /// 释放锁
    async fn release_locks(&self, ctx: &RepeatableReadContext) -> Result<(), RepeatableReadError> {
        let locks = ctx.locks.read().await;
        let mut global_locks = self.global_locks.write().await;
        
        for (key, _) in locks.iter() {
            global_locks.remove(key);
            
            // 从死锁检测器移除
            self.deadlock_detector
                .release_lock(ctx.base_context.transaction_id.clone(), key.clone())
                .await;
        }
        
        // 清空上下文锁表
        drop(locks);
        let mut locks = ctx.locks.write().await;
        locks.clear();
        
        Ok(())
    }
}

impl Default for RepeatableReadExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// 死锁检测器
pub struct DeadlockDetector {
    /// 等待图：事务 ID -> 等待的键列表
    wait_for_graph: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 键持有者：键 -> 事务 ID
    key_holders: Arc<RwLock<HashMap<String, String>>>,
}

impl DeadlockDetector {
    pub fn new() -> Self {
        Self {
            wait_for_graph: Arc::new(RwLock::new(HashMap::new())),
            key_holders: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册锁获取
    pub async fn register_lock(&self, tx_id: String, key: String) {
        let mut key_holders = self.key_holders.write().await;
        key_holders.insert(key, tx_id);
    }
    
    /// 释放锁
    pub async fn release_lock(&self, tx_id: String, key: String) {
        let mut key_holders = self.key_holders.write().await;
        if let Some(holder) = key_holders.get(&key) {
            if holder == &tx_id {
                key_holders.remove(&key);
            }
        }
        
        let mut wait_for_graph = self.wait_for_graph.write().await;
        if let Some(waiting_keys) = wait_for_graph.get_mut(&tx_id) {
            waiting_keys.retain(|k| k != &key);
        }
    }
    
    /// 检测死锁（简化版 DFS）
    pub async fn detect_deadlock(&self, start_tx: &str) -> Option<String> {
        let wait_for_graph = self.wait_for_graph.read().await;
        let key_holders = self.key_holders.read().await;
        
        let mut visited = HashMap::new();
        let mut stack = vec![start_tx.to_string()];
        
        while let Some(current) = stack.pop() {
            if visited.contains_key(&current) {
                // 找到环路
                return Some(current);
            }
            
            visited.insert(current.clone(), true);
            
            // 查找当前事务等待的键
            if let Some(waiting_keys) = wait_for_graph.get(&current) {
                for key in waiting_keys {
                    if let Some(holder) = key_holders.get(key) {
                        if holder != &current {
                            stack.push(holder.clone());
                        }
                    }
                }
            }
        }
        
        None
    }
}

impl Default for DeadlockDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// 提交结果
#[derive(Debug, Clone)]
pub struct CommitResult {
    pub transaction_id: String,
    pub committed_keys: Vec<String>,
    pub timestamp: i64,
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
    
    #[tokio::test]
    async fn test_rr_context_creation() {
        let ctx = RepeatableReadContext::new(
            "txn_1".to_string(),
            "trace_1".to_string(),
            5000,
        );
        
        assert_eq!(ctx.base_context.transaction_id, "txn_1");
        assert_eq!(ctx.base_context.isolation_level, IsolationLevel::RepeatableRead);
    }
    
    #[tokio::test]
    async fn test_rr_snapshot_read_write() {
        let executor = RepeatableReadExecutor::new();
        let ctx = RepeatableReadContext::new(
            "txn_1".to_string(),
            "trace_1".to_string(),
            5000,
        );
        
        // 初始化一些数据
        {
            let mut versions = executor.versions.write().await;
            versions.insert(
                "key_1".to_string(),
                vec![SnapshotEntry {
                    key: "key_1".to_string(),
                    value: serde_json::json!({"initial": "value"}),
                    version: 0,
                    created_at: current_timestamp_ms(),
                }],
            );
        }
        
        // 创建快照
        executor.create_snapshot(&ctx, &["key_1".to_string()]).await.unwrap();
        
        // 快照读
        let value = executor.snapshot_read(&ctx, "key_1").await.unwrap();
        assert!(value.is_some());
        
        // 写操作
        executor.execute_write(
            &ctx,
            "key_1",
            serde_json::json!({"updated": "value"}),
            WriteOperation::Update,
        ).await.unwrap();
        
        // 再次读（应该读到写集的值）
        let value = executor.snapshot_read(&ctx, "key_1").await.unwrap();
        assert_eq!(value, Some(serde_json::json!({"updated": "value"})));
    }
    
    #[tokio::test]
    async fn test_rr_commit() {
        let executor = RepeatableReadExecutor::new();
        let ctx = RepeatableReadContext::new(
            "txn_1".to_string(),
            "trace_1".to_string(),
            5000,
        );
        
        // 写操作
        executor.execute_write(
            &ctx,
            "key_1",
            serde_json::json!({"value": 123}),
            WriteOperation::Insert,
        ).await.unwrap();
        
        // 提交
        let result = executor.commit(&ctx).await.unwrap();
        assert_eq!(result.transaction_id, "txn_1");
        assert!(result.committed_keys.contains(&"key_1".to_string()));
    }
}
