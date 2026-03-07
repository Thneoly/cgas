//! MVCC (Multi-Version Concurrency Control) 基础实现
//! 
//! Phase 3 Week 2: 实现多版本并发控制，支持 Repeatable Read 隔离级别
//! 
//! 核心特性:
//! - 多版本状态存储
//! - 版本可见性检查
//! - 事务快照管理
//! - 乐观并发控制 (OCC)
//! - 版本 GC

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, debug, warn, error};

use crate::transaction::types::IsolationLevel;

/// MVCC 管理器
/// 
/// 负责管理多版本数据、事务快照和版本可见性
pub struct MvccManager {
    /// 数据版本存储：key -> 版本历史
    versions: Arc<RwLock<HashMap<String, Vec<StateVersion>>>>,
    /// 活跃事务快照：transaction_id -> 快照
    active_transactions: Arc<RwLock<HashMap<String, TransactionSnapshot>>>,
    /// 版本计数器
    version_counter: Arc<RwLock<u64>>,
    /// GC 配置
    gc_config: GcConfig,
}

/// 状态版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVersion {
    /// 数据键
    pub key: String,
    /// 数据值
    pub value: serde_json::Value,
    /// 版本号 (全局递增)
    pub version: u64,
    /// 创建事务 ID
    pub created_by: String,
    /// 可见事务列表 (空表示对所有事务可见)
    pub visible_to: Vec<String>,
    /// 过期时间 (用于 GC, None 表示永不过期)
    pub expired_at: Option<i64>,
    /// 创建时间戳
    pub created_at: i64,
}

/// 事务快照
#[derive(Debug, Clone)]
pub struct TransactionSnapshot {
    /// 事务 ID
    pub transaction_id: String,
    /// 快照创建时间
    pub snapshot_time: i64,
    /// 快照时的版本映射：key -> version
    pub state_versions: HashMap<String, u64>,
    /// 读集：key -> 读取的版本号
    pub read_set: HashMap<String, u64>,
    /// 写集：key -> 新值
    pub write_set: HashMap<String, serde_json::Value>,
    /// 隔离级别
    pub isolation_level: IsolationLevel,
}

impl TransactionSnapshot {
    /// 创建新的事务快照
    pub fn new(
        transaction_id: String,
        isolation_level: IsolationLevel,
    ) -> Self {
        let now = current_timestamp_ms();
        Self {
            transaction_id,
            snapshot_time: now,
            state_versions: HashMap::new(),
            read_set: HashMap::new(),
            write_set: HashMap::new(),
            isolation_level,
        }
    }
    
    /// 记录读操作
    pub fn record_read(&mut self, key: &str, version: u64) {
        self.read_set.insert(key.to_string(), version);
    }
    
    /// 记录写操作
    pub fn record_write(&mut self, key: &str, value: serde_json::Value) {
        self.write_set.insert(key.to_string(), value);
    }
}

/// GC 配置
#[derive(Debug, Clone)]
pub struct GcConfig {
    /// GC 间隔 (毫秒)
    pub gc_interval_ms: u64,
    /// 版本保留时间 (毫秒)
    pub retention_ms: u64,
    /// 最小保留版本数
    pub min_versions_to_keep: usize,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            gc_interval_ms: 60000, // 1 分钟
            retention_ms: 300000,  // 5 分钟
            min_versions_to_keep: 2,
        }
    }
}

/// MVCC 错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum MvccError {
    #[error("事务不存在：{0}")]
    TransactionNotFound(String),
    
    #[error("版本不可见：key={key}, version={version}, transaction={tx_id}")]
    VersionNotVisible {
        key: String,
        version: u64,
        tx_id: String,
    },
    
    #[error("写冲突：key={key} 已被其他事务修改")]
    WriteConflict { key: String },
    
    #[error("快照创建失败：{0}")]
    SnapshotCreationFailed(String),
    
    #[error("提交失败：{0}")]
    CommitFailed(String),
    
    #[error("GC 失败：{0}")]
    GcFailed(String),
}

impl MvccManager {
    /// 创建新的 MVCC 管理器
    pub fn new() -> Self {
        Self {
            versions: Arc::new(RwLock::new(HashMap::new())),
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            version_counter: Arc::new(RwLock::new(0)),
            gc_config: GcConfig::default(),
        }
    }
    
    /// 创建带自定义 GC 配置的 MVCC 管理器
    pub fn with_gc_config(gc_config: GcConfig) -> Self {
        Self {
            versions: Arc::new(RwLock::new(HashMap::new())),
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            version_counter: Arc::new(RwLock::new(0)),
            gc_config,
        }
    }
    
    /// 创建事务快照
    pub async fn create_snapshot(
        &self,
        transaction_id: &str,
        isolation_level: IsolationLevel,
    ) -> Result<TransactionSnapshot, MvccError> {
        debug!("Creating MVCC snapshot for transaction: {}", transaction_id);
        
        let mut snapshot = TransactionSnapshot::new(
            transaction_id.to_string(),
            isolation_level,
        );
        
        // 捕获当前所有键的最新版本
        let versions = self.versions.read().await;
        for (key, version_history) in versions.iter() {
            if let Some(latest) = version_history.last() {
                snapshot.state_versions.insert(key.clone(), latest.version);
            }
        }
        drop(versions);
        
        // 注册活跃事务
        {
            let mut active_txs = self.active_transactions.write().await;
            active_txs.insert(transaction_id.to_string(), snapshot.clone());
        }
        
        info!(
            "MVCC snapshot created: tx={}, keys={}",
            transaction_id,
            snapshot.state_versions.len()
        );
        
        Ok(snapshot)
    }
    
    /// 读取版本 (带可见性检查)
    pub async fn read_version(
        &self,
        key: &str,
        transaction_id: &str,
    ) -> Result<Option<StateVersion>, MvccError> {
        debug!("Reading version: key={}, tx={}", key, transaction_id);
        
        let versions = self.versions.read().await;
        
        if let Some(version_history) = versions.get(key) {
            // 查找对事务可见的最新版本
            for version in version_history.iter().rev() {
                if self.is_visible(version, transaction_id).await {
                    // 记录读集
                    self.record_read(transaction_id, key, version.version).await;
                    return Ok(Some(version.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    /// 检查版本可见性
    /// 
    /// 可见性规则:
    /// 1. 创建者自己可见
    /// 2. 已提交版本对所有事务可见 (visible_to 包含 "committed")
    /// 3. 显式授权给特定事务可见
    async fn is_visible(&self, version: &StateVersion, transaction_id: &str) -> bool {
        // 规则 1: 创建者自己可见
        if version.created_by == transaction_id {
            return true;
        }
        
        // 规则 2: 已提交版本
        if version.visible_to.contains(&"committed".to_string()) {
            return true;
        }
        
        // 规则 3: 显式授权
        if version.visible_to.contains(&transaction_id.to_string()) {
            return true;
        }
        
        false
    }
    
    /// 记录读操作到事务快照
    async fn record_read(&self, transaction_id: &str, key: &str, version: u64) {
        let mut active_txs = self.active_transactions.write().await;
        if let Some(snapshot) = active_txs.get_mut(transaction_id) {
            snapshot.record_read(key, version);
        }
    }
    
    /// 写入新版本 (未提交)
    pub async fn write_version(
        &self,
        key: &str,
        value: serde_json::Value,
        transaction_id: &str,
    ) -> Result<u64, MvccError> {
        debug!("Writing new version: key={}, tx={}", key, transaction_id);
        
        // 获取新版本号
        let mut counter = self.version_counter.write().await;
        let new_version = *counter;
        *counter += 1;
        drop(counter);
        
        let now = current_timestamp_ms();
        
        // 创建新版本 (仅对创建者可见)
        let state_version = StateVersion {
            key: key.to_string(),
            value,
            version: new_version,
            created_by: transaction_id.to_string(),
            visible_to: vec![transaction_id.to_string()], // 仅创建者可见
            expired_at: None,
            created_at: now,
        };
        
        // 写入版本历史
        let mut versions = self.versions.write().await;
        let version_history = versions.entry(key.to_string()).or_insert_with(Vec::new);
        version_history.push(state_version);
        drop(versions);
        
        // 记录写集
        self.record_write(transaction_id, key, new_version).await;
        
        debug!("New version created: key={}, version={}, tx={}", key, new_version, transaction_id);
        
        Ok(new_version)
    }
    
    /// 记录写操作到事务快照
    async fn record_write(&self, transaction_id: &str, key: &str, version: u64) {
        let mut active_txs = self.active_transactions.write().await;
        if let Some(snapshot) = active_txs.get_mut(transaction_id) {
            snapshot.write_set.insert(key.to_string(), serde_json::Value::Number(version.into()));
        }
    }
    
    /// 提交事务 (使版本对所有事务可见)
    pub async fn commit(&self, transaction_id: &str) -> Result<usize, MvccError> {
        debug!("Committing transaction: {}", transaction_id);
        
        // 获取事务快照
        let snapshot = {
            let active_txs = self.active_transactions.read().await;
            active_txs.get(transaction_id).cloned()
                .ok_or_else(|| MvccError::TransactionNotFound(transaction_id.to_string()))?
        };
        
        // 检查写写冲突 (乐观并发控制)
        for (key, read_version) in &snapshot.read_set {
            let versions = self.versions.read().await;
            if let Some(version_history) = versions.get(key) {
                if let Some(latest) = version_history.last() {
                    if latest.version != *read_version && !latest.created_by.eq(transaction_id) {
                        // 检测到冲突：读取的版本已被其他事务修改
                        warn!(
                            "Write-write conflict detected: key={}, tx={}, read_version={}, latest_version={}",
                            key, transaction_id, read_version, latest.version
                        );
                        return Err(MvccError::WriteConflict { key: key.clone() });
                    }
                }
            }
        }
        
        // 使事务创建的所有版本对所有事务可见
        let mut versions = self.versions.write().await;
        let mut committed_count = 0;
        
        for (key, version_history) in versions.iter_mut() {
            for version in version_history.iter_mut() {
                if version.created_by == transaction_id && !version.visible_to.contains(&"committed".to_string()) {
                    version.visible_to.push("committed".to_string());
                    committed_count += 1;
                }
            }
        }
        
        // 清理事务快照
        {
            let mut active_txs = self.active_transactions.write().await;
            active_txs.remove(transaction_id);
        }
        
        info!("Transaction committed: tx={}, versions={}", transaction_id, committed_count);
        
        Ok(committed_count)
    }
    
    /// 回滚事务 (清理未提交版本)
    pub async fn rollback(&self, transaction_id: &str) -> Result<usize, MvccError> {
        debug!("Rolling back transaction: {}", transaction_id);
        
        // 清理未提交版本
        let mut versions = self.versions.write().await;
        let mut removed_count = 0;
        
        for (_, version_history) in versions.iter_mut() {
            version_history.retain(|version| {
                if version.created_by == transaction_id && !version.visible_to.contains(&"committed".to_string()) {
                    removed_count += 1;
                    false
                } else {
                    true
                }
            });
        }
        
        // 清理事务快照
        {
            let mut active_txs = self.active_transactions.write().await;
            active_txs.remove(transaction_id);
        }
        
        info!("Transaction rolled back: tx={}, removed_versions={}", transaction_id, removed_count);
        
        Ok(removed_count)
    }
    
    /// GC 过期版本
    pub async fn gc(&self) -> Result<usize, MvccError> {
        debug!("Running MVCC GC");
        
        let now = current_timestamp_ms();
        let retention_cutoff = now - self.gc_config.retention_ms as i64;
        
        let mut versions = self.versions.write().await;
        let mut reclaimed_count = 0;
        
        for (_, version_history) in versions.iter_mut() {
            // 保留最小版本数
            if version_history.len() <= self.gc_config.min_versions_to_keep {
                continue;
            }
            
            // 移除过期版本
            let original_len = version_history.len();
            version_history.retain(|version| {
                // 保留已提交的版本
                if version.visible_to.contains(&"committed".to_string()) {
                    return true;
                }
                
                // 保留未过期的版本
                if let Some(expired_at) = version.expired_at {
                    return expired_at > now;
                }
                
                // 保留最近的版本
                let age = now - version.created_at;
                age < self.gc_config.retention_ms as i64
            });
            
            reclaimed_count += original_len - version_history.len();
        }
        
        info!("MVCC GC completed: reclaimed {} versions", reclaimed_count);
        
        Ok(reclaimed_count)
    }
    
    /// 获取统计信息
    pub async fn get_stats(&self) -> MvccStats {
        let versions = self.versions.read().await;
        let active_txs = self.active_transactions.read().await;
        let version_counter = self.version_counter.read().await;
        
        let total_versions: usize = versions.values().map(|v| v.len()).sum();
        let total_keys = versions.len();
        
        MvccStats {
            total_keys,
            total_versions,
            active_transactions: active_txs.len(),
            current_version: *version_counter,
        }
    }
}

impl Default for MvccManager {
    fn default() -> Self {
        Self::new()
    }
}

/// MVCC 统计信息
#[derive(Debug, Clone)]
pub struct MvccStats {
    /// 总键数
    pub total_keys: usize,
    /// 总版本数
    pub total_versions: usize,
    /// 活跃事务数
    pub active_transactions: usize,
    /// 当前版本号
    pub current_version: u64,
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
    async fn test_mvcc_create_snapshot() {
        let mvcc = MvccManager::new();
        
        // 创建一些初始数据
        mvcc.write_version("key1", serde_json::json!("value1"), "tx_init").await.unwrap();
        mvcc.commit("tx_init").await.unwrap();
        
        // 创建事务快照
        let snapshot = mvcc.create_snapshot("tx1", IsolationLevel::RepeatableRead).await.unwrap();
        
        assert_eq!(snapshot.transaction_id, "tx1");
        assert_eq!(snapshot.isolation_level, IsolationLevel::RepeatableRead);
        assert!(snapshot.state_versions.contains_key("key1"));
    }
    
    #[tokio::test]
    async fn test_mvcc_read_write() {
        let mvcc = MvccManager::new();
        
        // 写入初始数据
        mvcc.write_version("key1", serde_json::json!("value1"), "tx1").await.unwrap();
        mvcc.commit("tx1").await.unwrap();
        
        // 新事务读取
        mvcc.create_snapshot("tx2", IsolationLevel::RepeatableRead).await.unwrap();
        let version = mvcc.read_version("key1", "tx2").await.unwrap();
        
        assert!(version.is_some());
        assert_eq!(version.unwrap().value, serde_json::json!("value1"));
        
        // 新事务写入
        mvcc.write_version("key1", serde_json::json!("value2"), "tx2").await.unwrap();
        
        // 提交前，tx2 应该能看到自己的写入
        let version = mvcc.read_version("key1", "tx2").await.unwrap();
        assert_eq!(version.unwrap().value, serde_json::json!("value2"));
    }
    
    #[tokio::test]
    async fn test_mvcc_commit_rollback() {
        let mvcc = MvccManager::new();
        
        // 初始数据
        mvcc.write_version("key1", serde_json::json!("value1"), "tx1").await.unwrap();
        mvcc.commit("tx1").await.unwrap();
        
        // 事务 2 修改
        mvcc.create_snapshot("tx2", IsolationLevel::RepeatableRead).await.unwrap();
        mvcc.write_version("key1", serde_json::json!("value2"), "tx2").await.unwrap();
        
        // 事务 3 也修改 (会冲突)
        mvcc.create_snapshot("tx3", IsolationLevel::RepeatableRead).await.unwrap();
        mvcc.write_version("key1", serde_json::json!("value3"), "tx3").await.unwrap();
        
        // tx2 提交
        let result = mvcc.commit("tx2").await;
        assert!(result.is_ok());
        
        // tx3 提交应该失败 (写写冲突)
        let result = mvcc.commit("tx3").await;
        assert!(result.is_err());
        
        // 回滚 tx3
        let removed = mvcc.rollback("tx3").await.unwrap();
        assert!(removed > 0);
    }
    
    #[tokio::test]
    async fn test_mvcc_visibility() {
        let mvcc = MvccManager::new();
        
        // tx1 写入但未提交
        mvcc.write_version("key1", serde_json::json!("value1"), "tx1").await.unwrap();
        
        // tx2 不应该看到 tx1 的未提交写入
        let version = mvcc.read_version("key1", "tx2").await.unwrap();
        assert!(version.is_none());
        
        // tx1 应该能看到自己的写入
        let version = mvcc.read_version("key1", "tx1").await.unwrap();
        assert!(version.is_some());
        
        // tx1 提交
        mvcc.commit("tx1").await.unwrap();
        
        // tx2 现在应该能看到
        let version = mvcc.read_version("key1", "tx2").await.unwrap();
        assert!(version.is_some());
    }
    
    #[tokio::test]
    async fn test_mvcc_stats() {
        let mvcc = MvccManager::new();
        
        // 初始统计
        let stats = mvcc.get_stats().await;
        assert_eq!(stats.total_keys, 0);
        assert_eq!(stats.total_versions, 0);
        
        // 写入数据
        mvcc.write_version("key1", serde_json::json!("value1"), "tx1").await.unwrap();
        mvcc.write_version("key2", serde_json::json!("value2"), "tx1").await.unwrap();
        mvcc.commit("tx1").await.unwrap();
        
        // 更新统计
        let stats = mvcc.get_stats().await;
        assert_eq!(stats.total_keys, 2);
        assert!(stats.total_versions >= 2);
    }
}
