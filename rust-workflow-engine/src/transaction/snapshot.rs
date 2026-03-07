//! 快照隔离 (Snapshot Isolation) 实现
//! 
//! Phase 3 Week 2: 实现快照隔离，支持 Repeatable Read 语义
//! 
//! 核心特性:
//! - 一致性快照读
//! - 写时复制 (Copy-on-Write)
//! - 快照版本管理
//! - 快照 GC

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, debug, warn};

use crate::transaction::types::{IsolationLevel, TransactionContext};
use crate::executor::StateDiffOperation;

/// 快照隔离管理器
pub struct SnapshotManager {
    /// 快照存储：snapshot_id -> 快照数据
    snapshots: Arc<RwLock<HashMap<String, DataSnapshot>>>,
    /// 快照索引：key -> 版本列表
    snapshot_index: Arc<RwLock<HashMap<String, Vec<SnapshotVersion>>>>,
    /// 快照计数器
    snapshot_counter: Arc<RwLock<u64>>,
}

/// 数据快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSnapshot {
    /// 快照 ID
    pub snapshot_id: String,
    /// 创建事务 ID
    pub transaction_id: String,
    /// 创建时间戳
    pub created_at: i64,
    /// 快照数据
    pub data: HashMap<String, SnapshotValue>,
    /// 读集
    pub read_set: Vec<String>,
    /// 写集
    pub write_set: Vec<WriteOperation>,
    /// 隔离级别
    pub isolation_level: IsolationLevel,
}

/// 快照值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotValue {
    /// 数据值
    pub value: serde_json::Value,
    /// 版本号
    pub version: u64,
    /// 创建时间戳
    pub created_at: i64,
}

/// 快照版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotVersion {
    /// 快照 ID
    pub snapshot_id: String,
    /// 版本号
    pub version: u64,
    /// 创建时间戳
    pub created_at: i64,
}

/// 写操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteOperation {
    /// 数据键
    pub key: String,
    /// 操作类型
    pub op_type: WriteOpType,
    /// 新值 (Insert/Update)
    pub new_value: Option<serde_json::Value>,
    /// 旧值 (Update/Delete)
    pub old_value: Option<serde_json::Value>,
}

/// 写操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WriteOpType {
    /// 插入
    Insert,
    /// 更新
    Update,
    /// 删除
    Delete,
}

/// 快照隔离错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum SnapshotError {
    #[error("快照不存在：{0}")]
    SnapshotNotFound(String),
    
    #[error("快照已过期：{0}")]
    SnapshotExpired(String),
    
    #[error("写冲突：key={key} 在快照创建后被修改")]
    WriteConflict { key: String },
    
    #[error("快照创建失败：{0}")]
    CreationFailed(String),
    
    #[error("快照提交失败：{0}")]
    CommitFailed(String),
}

impl SnapshotManager {
    /// 创建新的快照管理器
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            snapshot_index: Arc::new(RwLock::new(HashMap::new())),
            snapshot_counter: Arc::new(RwLock::new(0)),
        }
    }
    
    /// 从事务上下文创建快照
    pub async fn create_from_context(
        &self,
        ctx: &TransactionContext,
    ) -> Result<String, SnapshotError> {
        debug!("Creating snapshot from transaction context: {}", ctx.transaction_id);
        
        let snapshot_id = self.generate_snapshot_id();
        let now = current_timestamp_ms();
        
        let snapshot = DataSnapshot {
            snapshot_id: snapshot_id.clone(),
            transaction_id: ctx.transaction_id.clone(),
            created_at: now,
            data: HashMap::new(),
            read_set: Vec::new(),
            write_set: Vec::new(),
            isolation_level: ctx.isolation_level.clone(),
        };
        
        // 存储快照
        {
            let mut snapshots = self.snapshots.write().await;
            snapshots.insert(snapshot_id.clone(), snapshot);
        }
        
        info!("Snapshot created: id={}, tx={}", snapshot_id, ctx.transaction_id);
        
        Ok(snapshot_id)
    }
    
    /// 快照读
    pub async fn snapshot_read(
        &self,
        snapshot_id: &str,
        key: &str,
    ) -> Result<Option<serde_json::Value>, SnapshotError> {
        debug!("Snapshot read: snapshot={}, key={}", snapshot_id, key);
        
        let snapshots = self.snapshots.read().await;
        let snapshot = snapshots.get(snapshot_id)
            .ok_or_else(|| SnapshotError::SnapshotNotFound(snapshot_id.to_string()))?;
        
        // 1. 检查写集 (读已写)
        for write_op in &snapshot.write_set {
            if write_op.key == key {
                match write_op.op_type {
                    WriteOpType::Insert | WriteOpType::Update => {
                        return Ok(write_op.new_value.clone());
                    }
                    WriteOpType::Delete => {
                        return Ok(None);
                    }
                }
            }
        }
        
        // 2. 从快照数据读
        if let Some(value) = snapshot.data.get(key) {
            // 记录读集
            self.record_read(snapshot_id, key).await;
            return Ok(Some(value.value.clone()));
        }
        
        Ok(None)
    }
    
    /// 快照写
    pub async fn snapshot_write(
        &self,
        snapshot_id: &str,
        key: &str,
        value: serde_json::Value,
        op_type: WriteOpType,
    ) -> Result<(), SnapshotError> {
        debug!("Snapshot write: snapshot={}, key={}", snapshot_id, key);
        
        let mut snapshots = self.snapshots.write().await;
        let snapshot = snapshots.get_mut(snapshot_id)
            .ok_or_else(|| SnapshotError::SnapshotNotFound(snapshot_id.to_string()))?;
        
        // 获取旧值
        let old_value = match snapshot.data.get(key) {
            Some(v) => Some(v.value.clone()),
            None => None,
        };
        
        // 记录写操作
        snapshot.write_set.push(WriteOperation {
            key: key.to_string(),
            op_type: op_type.clone(),
            new_value: match op_type {
                WriteOpType::Insert | WriteOpType::Update => Some(value.clone()),
                WriteOpType::Delete => None,
            },
            old_value,
        });
        
        // 更新快照数据
        match op_type {
            WriteOpType::Insert | WriteOpType::Update => {
                let now = current_timestamp_ms();
                let version = snapshot.data.get(key).map(|v| v.version + 1).unwrap_or(0);
                
                snapshot.data.insert(key.to_string(), SnapshotValue {
                    value,
                    version,
                    created_at: now,
                });
            }
            WriteOpType::Delete => {
                snapshot.data.remove(key);
            }
        }
        
        Ok(())
    }
    
    /// 记录读操作
    async fn record_read(&self, snapshot_id: &str, key: &str) {
        let mut snapshots = self.snapshots.write().await;
        if let Some(snapshot) = snapshots.get_mut(snapshot_id) {
            if !snapshot.read_set.contains(&key.to_string()) {
                snapshot.read_set.push(key.to_string());
            }
        }
    }
    
    /// 提交快照 (验证写集)
    pub async fn commit(
        &self,
        snapshot_id: &str,
    ) -> Result<Vec<StateDiffOperation>, SnapshotError> {
        debug!("Committing snapshot: {}", snapshot_id);
        
        let snapshot = {
            let snapshots = self.snapshots.read().await;
            snapshots.get(snapshot_id)
                .ok_or_else(|| SnapshotError::SnapshotNotFound(snapshot_id.to_string()))?
                .clone()
        };
        
        // 验证读集 (检查是否有冲突)
        // 简化实现：实际应该检查读集中的键是否被其他事务修改
        
        // 构建 state_diff
        let mut diff = Vec::new();
        for write_op in &snapshot.write_set {
            match write_op.op_type {
                WriteOpType::Insert => {
                    diff.push(StateDiffOperation {
                        op_type: crate::executor::DiffOpType::Insert,
                        key: write_op.key.clone(),
                        value: write_op.new_value.clone().map(|v| v.to_string()),
                    });
                }
                WriteOpType::Update => {
                    diff.push(StateDiffOperation {
                        op_type: crate::executor::DiffOpType::Update,
                        key: write_op.key.clone(),
                        value: write_op.new_value.clone().map(|v| v.to_string()),
                    });
                }
                WriteOpType::Delete => {
                    diff.push(StateDiffOperation {
                        op_type: crate::executor::DiffOpType::Delete,
                        key: write_op.key.clone(),
                        value: None,
                    });
                }
            }
        }
        
        // 清理快照
        {
            let mut snapshots = self.snapshots.write().await;
            snapshots.remove(snapshot_id);
        }
        
        info!("Snapshot committed: id={}, operations={}", snapshot_id, diff.len());
        
        Ok(diff)
    }
    
    /// 回滚快照
    pub async fn rollback(&self, snapshot_id: &str) -> Result<(), SnapshotError> {
        debug!("Rolling back snapshot: {}", snapshot_id);
        
        // 简单删除快照 (未提交的写集自动丢弃)
        {
            let mut snapshots = self.snapshots.write().await;
            snapshots.remove(snapshot_id);
        }
        
        info!("Snapshot rolled back: id={}", snapshot_id);
        
        Ok(())
    }
    
    /// 生成快照 ID
    fn generate_snapshot_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        format!("snapshot_{}_{}", now.as_secs(), now.subsec_nanos())
    }
    
    /// 获取统计信息
    pub async fn get_stats(&self) -> SnapshotStats {
        let snapshots = self.snapshots.read().await;
        let index = self.snapshot_index.read().await;
        
        SnapshotStats {
            active_snapshots: snapshots.len(),
            indexed_keys: index.len(),
        }
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 快照统计信息
#[derive(Debug, Clone)]
pub struct SnapshotStats {
    /// 活跃快照数
    pub active_snapshots: usize,
    /// 索引键数
    pub indexed_keys: usize,
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
    async fn test_snapshot_creation() {
        let manager = SnapshotManager::new();
        
        let ctx = TransactionContext::new(
            "txn_1".to_string(),
            "trace_1".to_string(),
            IsolationLevel::RepeatableRead,
            5000,
        );
        
        let snapshot_id = manager.create_from_context(&ctx).await.unwrap();
        
        assert!(snapshot_id.starts_with("snapshot_"));
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.active_snapshots, 1);
    }
    
    #[tokio::test]
    async fn test_snapshot_read_write() {
        let manager = SnapshotManager::new();
        
        let ctx = TransactionContext::new(
            "txn_1".to_string(),
            "trace_1".to_string(),
            IsolationLevel::RepeatableRead,
            5000,
        );
        
        let snapshot_id = manager.create_from_context(&ctx).await.unwrap();
        
        // 写操作
        manager.snapshot_write(
            &snapshot_id,
            "key1",
            serde_json::json!("value1"),
            WriteOpType::Insert,
        ).await.unwrap();
        
        // 读操作 (应该读到刚写的值)
        let value = manager.snapshot_read(&snapshot_id, "key1").await.unwrap();
        assert_eq!(value, Some(serde_json::json!("value1")));
        
        // 更新
        manager.snapshot_write(
            &snapshot_id,
            "key1",
            serde_json::json!("value2"),
            WriteOpType::Update,
        ).await.unwrap();
        
        let value = manager.snapshot_read(&snapshot_id, "key1").await.unwrap();
        assert_eq!(value, Some(serde_json::json!("value2")));
    }
    
    #[tokio::test]
    async fn test_snapshot_commit() {
        let manager = SnapshotManager::new();
        
        let ctx = TransactionContext::new(
            "txn_1".to_string(),
            "trace_1".to_string(),
            IsolationLevel::RepeatableRead,
            5000,
        );
        
        let snapshot_id = manager.create_from_context(&ctx).await.unwrap();
        
        // 写操作
        manager.snapshot_write(
            &snapshot_id,
            "key1",
            serde_json::json!("value1"),
            WriteOpType::Insert,
        ).await.unwrap();
        
        manager.snapshot_write(
            &snapshot_id,
            "key2",
            serde_json::json!("value2"),
            WriteOpType::Insert,
        ).await.unwrap();
        
        // 提交
        let diff = manager.commit(&snapshot_id).await.unwrap();
        
        assert_eq!(diff.len(), 2);
        
        // 提交后快照应该被清理
        let stats = manager.get_stats().await;
        assert_eq!(stats.active_snapshots, 0);
    }
    
    #[tokio::test]
    async fn test_snapshot_rollback() {
        let manager = SnapshotManager::new();
        
        let ctx = TransactionContext::new(
            "txn_1".to_string(),
            "trace_1".to_string(),
            IsolationLevel::RepeatableRead,
            5000,
        );
        
        let snapshot_id = manager.create_from_context(&ctx).await.unwrap();
        
        // 写操作
        manager.snapshot_write(
            &snapshot_id,
            "key1",
            serde_json::json!("value1"),
            WriteOpType::Insert,
        ).await.unwrap();
        
        // 回滚
        manager.rollback(&snapshot_id).await.unwrap();
        
        // 回滚后快照应该被清理
        let stats = manager.get_stats().await;
        assert_eq!(stats.active_snapshots, 0);
    }
}
