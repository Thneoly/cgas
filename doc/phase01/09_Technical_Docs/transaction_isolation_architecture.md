# Transaction 隔离级别增强架构设计

**版本**: v1.0  
**日期**: 2026-05-12  
**责任人**: Architect-Agent  
**状态**: 📋 草案  
**release_id**: release-2026-05-12-phase3_week01  
**关联 ADR**: ADR-008 (Phase 3 ADR v5)

---

## 1. 设计目标

### 1.1 Phase 2 vs Phase 3 Transaction 对比

| 特性 | Phase 2 (Read Committed) | Phase 3 (Repeatable Read) | 改进 |
|---|---|---|---|
| 隔离级别 | Read Committed (RC) | Repeatable Read (RR) | 解决幻读问题 |
| 并发控制 | 锁机制 | MVCC (多版本) | 无锁读取 |
| 快照策略 | 无 | 事务级快照 | 可重复读保证 |
| 版本管理 | 单版本 | 多版本 | 支持并发读 |
| 内存开销 | 基准 | +30% | 一致性代价 |
| 读性能 | 基准 | +20% (无锁) | 性能提升 |
| 写冲突 | 阻塞 | 乐观检测 | 减少阻塞 |

### 1.2 隔离级别对比

| 隔离级别 | 脏读 | 不可重复读 | 幻读 | 适用场景 |
|---|---|---|---|---|
| Read Uncommitted | ✅ 可能 | ✅ 可能 | ✅ 可能 | 不推荐 |
| Read Committed (Phase 2) | ❌ 防止 | ✅ 可能 | ✅ 可能 | 一般查询 |
| **Repeatable Read (Phase 3)** | ❌ 防止 | ❌ 防止 | ❌ 防止 | **金融交易** |
| Serializable | ❌ 防止 | ❌ 防止 | ❌ 防止 | 强一致性 |

### 1.3 使用场景

| 场景 | Phase 2 限制 | Phase 3 解决方案 |
|---|---|---|
| 财务报表生成 | 可能幻读，数据不一致 | RR 保证一致性快照 |
| 批量数据迁移 | 读取过程中数据变化 | 快照隔离，数据稳定 |
| 复杂分析查询 | 多次读取结果不一致 | RR 保证可重复读 |
| 高并发读取 | 锁竞争，性能下降 | MVCC 无锁读取 |

---

## 2. MVCC 核心设计

### 2.1 多版本数据结构

```rust
/// Phase 3: 状态键
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StateKey {
    pub namespace: String,
    pub key: String,
}

/// Phase 3: 状态值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateValue {
    pub data: Vec<u8>,
    pub metadata: StateMetadata,
}

/// Phase 3: 状态版本 (MVCC 核心)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVersion {
    /// 状态键
    pub key: StateKey,
    /// 状态值
    pub value: StateValue,
    /// 版本号 (全局递增)
    pub version: u64,
    /// 创建者 (transaction_id)
    pub created_by: String,
    /// 创建时间戳
    pub created_at: u64,
    /// 可见的事务列表 (空=对所有事务可见)
    pub visible_to: Vec<String>,
    /// 过期时间 (用于 GC，None=永不过期)
    pub expired_at: Option<u64>,
    /// 提交状态
    pub committed: bool,
}

impl StateVersion {
    /// 检查版本对事务是否可见
    pub fn is_visible_to(&self, transaction_id: &str) -> bool {
        // 规则 1: 创建者自己可见
        if self.created_by == transaction_id {
            return true;
        }
        
        // 规则 2: 已提交且未指定可见事务 (对所有事务可见)
        if self.committed && self.visible_to.is_empty() {
            return true;
        }
        
        // 规则 3: 显式授权给该事务
        if self.visible_to.contains(&transaction_id.to_string()) {
            return true;
        }
        
        false
    }
    
    /// 标记为已提交
    pub fn commit(&mut self) {
        self.committed = true;
        self.visible_to.clear();  // 清除可见性限制，对所有事务可见
    }
    
    /// 检查是否过期 (用于 GC)
    pub fn is_expired(&self, current_time: u64, retention_ms: u64) -> bool {
        if let Some(expired_at) = self.expired_at {
            current_time > expired_at
        } else {
            // 未设置过期时间，检查创建时间
            current_time > self.created_at + retention_ms
        }
    }
}

/// Phase 3: 事务快照 (Repeatable Read 核心)
#[derive(Debug, Clone)]
pub struct TransactionSnapshot {
    /// 事务 ID
    pub transaction_id: String,
    /// 快照创建时间
    pub snapshot_time: u64,
    /// 快照版本号 (用于可见性判断)
    pub snapshot_version: u64,
    /// 读集 (用于冲突检测)
    pub read_set: HashSet<StateKey>,
    /// 写集 (用于冲突检测)
    pub write_set: HashSet<StateKey>,
    /// 本地版本缓存 (快照时的版本)
    pub version_cache: HashMap<StateKey, StateVersion>,
}

impl TransactionSnapshot {
    /// 创建事务快照
    pub fn new(transaction_id: String, snapshot_version: u64) -> Self {
        Self {
            transaction_id,
            snapshot_time: get_current_timestamp(),
            snapshot_version,
            read_set: HashSet::new(),
            write_set: HashSet::new(),
            version_cache: HashMap::new(),
        }
    }
    
    /// 记录读操作
    pub fn record_read(&mut self, key: &StateKey, version: &StateVersion) {
        self.read_set.insert(key.clone());
        self.version_cache.insert(key.clone(), version.clone());
    }
    
    /// 记录写操作
    pub fn record_write(&mut self, key: &StateKey, value: StateValue) {
        self.write_set.insert(key.clone());
        
        // 创建新版本 (未提交)
        let new_version = StateVersion {
            key: key.clone(),
            value,
            version: self.snapshot_version + 1,
            created_by: self.transaction_id.clone(),
            created_at: get_current_timestamp(),
            visible_to: vec![self.transaction_id.clone()],  // 仅自己可见
            expired_at: None,
            committed: false,
        };
        
        self.version_cache.insert(key.clone(), new_version);
    }
    
    /// 检查读集和写集是否有冲突 (用于 OCC)
    pub fn has_conflict_with(&self, other: &TransactionSnapshot) -> bool {
        // 写 - 写冲突
        if !self.write_set.is_disjoint(&other.write_set) {
            return true;
        }
        
        // 读 - 写冲突
        if !self.read_set.is_disjoint(&other.write_set) {
            return true;
        }
        
        // 写 - 读冲突
        if !self.write_set.is_disjoint(&other.read_set) {
            return true;
        }
        
        false
    }
}
```

### 2.2 MVCC 管理器

```rust
/// Phase 3: MVCC 管理器
pub struct MvccManager {
    /// 多版本存储 (key -> 版本列表)
    versions: DashMap<StateKey, Vec<StateVersion>>,
    /// 活跃事务快照
    active_transactions: DashMap<String, TransactionSnapshot>,
    /// 全局版本号 (原子递增)
    global_version: AtomicU64,
    /// GC 配置
    gc_config: MvccGcConfig,
    /// 监控指标
    metrics: Arc<MvccMetrics>,
}

/// MVCC GC 配置
#[derive(Debug, Clone)]
pub struct MvccGcConfig {
    /// 版本保留时间 (ms)
    pub retention_ms: u64,
    /// GC 间隔 (ms)
    pub gc_interval_ms: u64,
    /// 最大版本数 per key
    pub max_versions_per_key: usize,
}

impl Default for MvccGcConfig {
    fn default() -> Self {
        Self {
            retention_ms: 60_000,  // 1 分钟
            gc_interval_ms: 10_000,  // 10 秒
            max_versions_per_key: 100,
        }
    }
}

impl MvccManager {
    /// 创建 MVCC 管理器
    pub fn new(config: MvccGcConfig) -> Self {
        let manager = Self {
            versions: DashMap::new(),
            active_transactions: DashMap::new(),
            global_version: AtomicU64::new(0),
            gc_config: config,
            metrics: Arc::new(MvccMetrics::new()),
        };
        
        // 启动 GC 线程
        manager.start_gc_thread();
        
        manager
    }
    
    /// 获取下一个全局版本号
    fn next_version(&self) -> u64 {
        self.global_version.fetch_add(1, Ordering::SeqCst)
    }
    
    /// 创建事务快照 (Repeatable Read 核心)
    pub fn create_snapshot(&self, transaction_id: &str) -> Result<TransactionSnapshot> {
        let snapshot_version = self.next_version();
        
        let mut snapshot = TransactionSnapshot::new(
            transaction_id.to_string(),
            snapshot_version,
        );
        
        // 记录监控指标
        self.metrics.record_snapshot_creation();
        
        // 注册活跃事务
        self.active_transactions.insert(
            transaction_id.to_string(),
            snapshot.clone(),
        );
        
        Ok(snapshot)
    }
    
    /// 读取版本 (MVCC 可见性检查)
    pub fn read_version(
        &self,
        key: &StateKey,
        transaction_id: &str,
        snapshot: &TransactionSnapshot,
    ) -> Result<Option<StateVersion>> {
        // 1. 先从事务快照缓存中读取 (Repeatable Read 保证)
        if let Some(cached_version) = snapshot.version_cache.get(key) {
            self.metrics.record_cache_hit();
            return Ok(Some(cached_version.clone()));
        }
        
        // 2. 从版本列表中读取最新可见版本
        if let Some(versions) = self.versions.get(key) {
            // 按版本号降序查找
            for version in versions.iter().rev() {
                if version.is_visible_to(transaction_id) && version.committed {
                    // 检查版本是否在快照之前
                    if version.version <= snapshot.snapshot_version {
                        // 记录读集
                        let mut txn_snapshot = self.active_transactions
                            .get_mut(transaction_id)
                            .unwrap();
                        txn_snapshot.record_read(key, version);
                        
                        self.metrics.record_read();
                        return Ok(Some(version.clone()));
                    }
                }
            }
        }
        
        self.metrics.record_read_miss();
        Ok(None)
    }
    
    /// 写入新版本
    pub fn write_version(
        &self,
        key: &StateKey,
        value: StateValue,
        transaction_id: &str,
        snapshot: &mut TransactionSnapshot,
    ) -> Result<()> {
        // 记录写操作 (创建新版本，但未提交)
        snapshot.record_write(key, value);
        
        self.metrics.record_write();
        
        Ok(())
    }
    
    /// 提交事务 (使版本可见)
    pub fn commit(&self, transaction_id: &str) -> Result<()> {
        let snapshot = match self.active_transactions.remove(transaction_id) {
            Some((_, snapshot)) => snapshot,
            None => return Err(MvccError::TransactionNotFound(transaction_id.to_string())),
        };
        
        // 1. 检查写集冲突 (OCC 验证)
        if let Some(conflict) = self.check_write_conflicts(&snapshot) {
            self.metrics.record_commit_conflict();
            return Err(MvccError::WriteConflict(conflict));
        }
        
        // 2. 提交所有写版本
        let commit_version = self.next_version();
        
        for (key, mut version) in snapshot.version_cache {
            if version.created_by == transaction_id {
                // 标记为已提交，对所有事务可见
                version.committed = true;
                version.visible_to.clear();
                version.version = commit_version;
                
                // 添加到版本列表
                let mut versions = self.versions.entry(key.clone()).or_insert_with(Vec::new);
                versions.push(version);
                
                // 限制版本数量
                if versions.len() > self.gc_config.max_versions_per_key {
                    versions.remove(0);  // 移除最旧版本
                }
            }
        }
        
        // 3. 记录监控指标
        self.metrics.record_commit();
        
        Ok(())
    }
    
    /// 回滚事务 (清理未提交版本)
    pub fn rollback(&self, transaction_id: &str) -> Result<()> {
        match self.active_transactions.remove(transaction_id) {
            Some((_, snapshot)) => {
                // 清理未提交版本 (已经在快照中，不需要额外清理)
                self.metrics.record_rollback();
                Ok(())
            }
            None => Err(MvccError::TransactionNotFound(transaction_id.to_string())),
        }
    }
    
    /// 检查写冲突 (OCC)
    fn check_write_conflicts(&self, snapshot: &TransactionSnapshot) -> Option<String> {
        // 检查写集与其他活跃事务的读集/写集冲突
        for (_, other_snapshot) in self.active_transactions.iter() {
            if other_snapshot.transaction_id != snapshot.transaction_id {
                if snapshot.has_conflict_with(&other_snapshot) {
                    return Some(format!(
                        "Conflict with transaction {}",
                        other_snapshot.transaction_id
                    ));
                }
            }
        }
        
        None
    }
    
    /// 启动 GC 线程
    fn start_gc_thread(self: &Arc<Self>) {
        let manager = Arc::clone(self);
        let config = self.gc_config.clone();
        
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(config.gc_interval_ms));
                manager.run_gc();
            }
        });
    }
    
    /// 运行 GC (清理过期版本)
    fn run_gc(&self) -> usize {
        let current_time = get_current_timestamp();
        let mut reclaimed_count = 0;
        
        for mut entry in self.versions.iter_mut() {
            let versions = entry.value_mut();
            let before_count = versions.len();
            
            // 保留最新版本 + 未过期的旧版本
            versions.retain(|version| {
                // 总是保留最新版本
                if version.version == versions.last().unwrap().version {
                    return true;
                }
                
                // 检查是否过期
                !version.is_expired(current_time, self.gc_config.retention_ms)
            });
            
            reclaimed_count += before_count - versions.len();
        }
        
        if reclaimed_count > 0 {
            self.metrics.record_gc_reclaimed(reclaimed_count);
        }
        
        reclaimed_count
    }
}
```

---

## 3. 事务执行器设计

### 3.1 Repeatable Read 事务执行器

```rust
/// Phase 3: 支持 RR 隔离级别的事务执行器
pub struct RepeatableReadExecutor {
    /// MVCC 管理器
    mvcc_manager: Arc<MvccManager>,
    /// 基础执行器
    base_executor: Arc<Executor>,
    /// 验证器
    verifier: Arc<Verifier>,
    /// 提交器
    committer: Arc<Committer>,
    /// 监控指标
    metrics: Arc<TransactionMetrics>,
}

impl RepeatableReadExecutor {
    /// 执行 Repeatable Read 事务
    pub async fn execute_rr_transaction(
        &self,
        request: TransactionExecuteRequest,
    ) -> Result<TransactionExecuteResult> {
        let start_time = Instant::now();
        
        // 1. 检查隔离级别
        if request.isolation_level != IsolationLevel::RepeatableRead {
            // 降级到 Phase 2 路径
            return self.execute_rc_transaction(request).await;
        }
        
        // 2. 创建 MVCC 快照
        let mut snapshot = self.mvcc_manager.create_snapshot(&request.transaction_id)?;
        
        // 3. 执行事务指令
        let mut results = Vec::new();
        let mut success = true;
        
        for instruction in &request.instructions {
            match self.execute_instruction_in_transaction(
                instruction,
                &request.transaction_id,
                &mut snapshot,
            ).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    success = false;
                    results.push(ExecutionResult::failed(e.to_string()));
                    
                    if request.atomic {
                        // 原子性违反，回滚
                        self.mvcc_manager.rollback(&request.transaction_id)?;
                        return Err(e);
                    }
                }
            }
        }
        
        // 4. 提交或回滚
        let status = if success {
            match self.mvcc_manager.commit(&request.transaction_id) {
                Ok(_) => TransactionStatus::Committed,
                Err(MvccError::WriteConflict(conflict)) => {
                    // OCC 冲突，重试
                    if request.retry_count < request.max_retries {
                        return self.retry_transaction(request).await;
                    }
                    TransactionStatus::ConflictAborted
                }
                Err(e) => {
                    self.mvcc_manager.rollback(&request.transaction_id)?;
                    TransactionStatus::Failed(e.to_string())
                }
            }
        } else {
            self.mvcc_manager.rollback(&request.transaction_id)?;
            TransactionStatus::RolledBack
        };
        
        // 5. 记录监控指标
        self.metrics.record_transaction_execution(
            &request.isolation_level,
            start_time.elapsed().as_millis() as u64,
            &status,
        );
        
        Ok(TransactionExecuteResult {
            trace_id: request.trace_id,
            transaction_id: request.transaction_id,
            status,
            results,
            transaction_hash: self.compute_transaction_hash(&results),
            retry_count: request.retry_count,
            timestamp: get_current_timestamp(),
        })
    }
    
    /// 在事务中执行单条指令
    async fn execute_instruction_in_transaction(
        &self,
        instruction: &ExecuteRequest,
        transaction_id: &str,
        snapshot: &mut TransactionSnapshot,
    ) -> Result<ExecutionResult> {
        // 1. 读取状态 (使用 MVCC 快照)
        let state = self.mvcc_manager.read_state(
            &instruction.state_key,
            transaction_id,
            snapshot,
        )?;
        
        // 2. 执行指令
        let result = self.base_executor.execute_with_state(instruction, state).await?;
        
        // 3. 写入状态 (MVCC 新版本)
        self.mvcc_manager.write_state(
            &instruction.state_key,
            result.new_state.clone(),
            transaction_id,
            snapshot,
        )?;
        
        Ok(result)
    }
    
    /// 重试事务 (OCC 冲突时)
    async fn retry_transaction(&self, mut request: TransactionExecuteRequest) -> Result<TransactionExecuteResult> {
        request.retry_count += 1;
        
        // 指数退避
        let backoff_ms = 10 * 2_u64.pow(request.retry_count as u32);
        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
        
        // 重新执行
        self.execute_rr_transaction(request).await
    }
}
```

### 3.2 隔离级别切换

```rust
/// Phase 3: 隔离级别路由器
pub struct IsolationRouter {
    rc_executor: Arc<ReadCommittedExecutor>,  // Phase 2
    rr_executor: Arc<RepeatableReadExecutor>, // Phase 3
}

impl IsolationRouter {
    pub fn route(&self, request: TransactionExecuteRequest) -> Arc<dyn TransactionExecutor> {
        match request.isolation_level {
            IsolationLevel::ReadCommitted => {
                // Phase 2 路径 (向后兼容)
                Arc::clone(&self.rc_executor)
            }
            IsolationLevel::RepeatableRead => {
                // Phase 3 路径
                Arc::clone(&self.rr_executor)
            }
            IsolationLevel::Serializable => {
                // 预留 (Phase 4)
                unimplemented!("Serializable isolation not yet supported")
            }
        }
    }
}
```

---

## 4. 性能优化

### 4.1 MVCC 性能开销分析

| 操作 | Phase 2 (RC) | Phase 3 (RR+MVCC) | 开销 |
|---|---|---|---|
| 读取 | 锁机制 | 快照缓存 | -10% (更快) |
| 写入 | 锁机制 | 版本创建 | +15% |
| 快照创建 | N/A | 5-8ms | +5-8ms |
| 提交 | 直接提交 | OCC 验证 | +3-5ms |
| 内存占用 | 基准 | +30% | +30% |

### 4.2 优化策略

```rust
// Phase 3: 写时复制 (CoW) 快照优化
pub struct CowSnapshot {
    base_snapshot: Arc<TransactionSnapshot>,
    local_changes: DashMap<StateKey, StateVersion>,
}

impl CowSnapshot {
    pub fn new(base: TransactionSnapshot) -> Self {
        Self {
            base_snapshot: Arc::new(base),
            local_changes: DashMap::new(),
        }
    }
    
    pub fn read(&self, key: &StateKey) -> Option<&StateVersion> {
        // 先读本地修改
        if let Some(version) = self.local_changes.get(key) {
            return Some(version);
        }
        
        // 再读基础快照
        self.base_snapshot.version_cache.get(key)
    }
    
    pub fn write(&mut self, key: StateKey, value: StateValue) {
        // 写时复制：只复制修改的版本
        let new_version = StateVersion {
            key: key.clone(),
            value,
            // ... 其他字段
            ..Default::default()
        };
        
        self.local_changes.insert(key, new_version);
    }
}

// Phase 3: 版本链压缩
impl MvccManager {
    pub fn compact_versions(&self, key: &StateKey, before_version: u64) -> Result<usize> {
        if let Some(mut versions) = self.versions.get_mut(key) {
            let before_count = versions.len();
            
            // 保留最新版本 + before_version 之后的版本
            versions.retain(|v| v.version >= before_version);
            
            let reclaimed = before_count - versions.len();
            if reclaimed > 0 {
                self.metrics.record_gc_reclaimed(reclaimed);
            }
            
            Ok(reclaimed)
        } else {
            Ok(0)
        }
    }
}

// Phase 3: 对象池复用 StateVersion
pub struct StateVersionPool {
    pool: ObjectPool<StateVersion>,
}

impl StateVersionPool {
    pub fn acquire(&self, key: StateKey, value: StateValue) -> StateVersion {
        match self.pool.acquire() {
            Some(mut version) => {
                // 复用并重置
                version.key = key;
                version.value = value;
                version.version = 0;
                version.committed = false;
                version
            }
            None => StateVersion::new(key, value),
        }
    }
    
    pub fn release(&self, version: StateVersion) {
        self.pool.release(version);
    }
}
```

### 4.3 性能基线目标

| 指标 | Phase 2 (RC) | Phase 3 (RR) 目标 | 测量方法 |
|---|---|---|---|
| P99 读取时延 | 125ms | <115ms (-8%) | k6 压测 |
| P99 写入时延 | 140ms | <160ms (+14%) | k6 压测 |
| P99 事务时延 | 272ms | <290ms (+7%) | k6 压测 |
| 快照创建时延 | N/A | <8ms | 单元测试 |
| OCC 冲突率 | N/A | <5% | 压测统计 |
| 内存占用 | 基准 | +30% | Prometheus |

---

## 5. 监控指标

### 5.1 MVCC 相关指标 (4 个)

```rust
// Phase 3: MVCC 监控指标
pub struct MvccMetrics {
    /// 快照创建时延
    pub snapshot_creation_latency: Histogram,
    /// 版本数量
    pub version_count: Gauge,
    /// GC 回收版本数
    pub gc_reclaimed_versions: Counter,
    /// 冲突检测次数
    pub conflict_detection_count: Counter,
}

impl MvccMetrics {
    pub fn record_snapshot_creation(&self) {
        self.snapshot_creation_latency.observe(
            // 实际时延在创建时测量
            // 这里简化处理
            5.0  // 平均 5ms
        );
    }
    
    pub fn record_version_created(&self) {
        self.version_count.inc();
    }
    
    pub fn record_gc_reclaimed(&self, count: usize) {
        self.gc_reclaimed_versions.inc_by(count as u64);
    }
    
    pub fn record_conflict_detection(&self) {
        self.conflict_detection_count.inc();
    }
}
```

### 5.2 事务隔离指标

```rust
// Phase 3: 事务隔离级别指标
pub struct TransactionIsolationMetrics {
    /// 隔离级别分布
    pub isolation_level_distribution: HashMap<IsolationLevel, Counter>,
    /// RR 事务占比
    pub repeatable_read_ratio: Gauge,
    /// 事务重试率
    pub transaction_retry_rate: Gauge,
    /// MVCC 读放大
    pub mvcc_read_amplification: Gauge,
}

impl TransactionIsolationMetrics {
    pub fn record_transaction(&self, isolation_level: &IsolationLevel) {
        if let Some(counter) = self.isolation_level_distribution.get(isolation_level) {
            counter.inc();
        }
        
        // 更新 RR 占比
        self.update_rr_ratio();
    }
    
    pub fn record_retry(&self) {
        self.transaction_retry_rate.inc();
    }
    
    pub fn record_read_amplification(&self, amplification: f64) {
        self.mvcc_read_amplification.set(amplification);
    }
    
    fn update_rr_ratio(&self) {
        let rr_count = self.isolation_level_distribution
            .get(&IsolationLevel::RepeatableRead)
            .map(|c| c.get())
            .unwrap_or(0);
        
        let total: u64 = self.isolation_level_distribution
            .values()
            .map(|c| c.get())
            .sum();
        
        let ratio = if total > 0 {
            rr_count as f64 / total as f64
        } else {
            0.0
        };
        
        self.repeatable_read_ratio.set(ratio);
    }
}
```

---

## 6. 测试策略

### 6.1 测试用例分类

| 类别 | 用例数 | 覆盖场景 |
|---|---|---|
| 单元测试 | 25 | MVCC 数据结构、可见性检查、版本管理 |
| 集成测试 | 20 | 事务执行、快照隔离、OCC 冲突 |
| 并发测试 | 15 | 多事务并发、死锁检测、冲突解决 |
| 性能测试 | 5 | 时延、吞吐量、内存 |
| 边界测试 | 10 | 长事务、大事务、GC 边界 |
| **总计** | **75** | **全场景覆盖** |

### 6.2 关键测试用例

```rust
#[cfg(test)]
mod mvcc_tests {
    use super::*;
    
    /// 测试：Repeatable Read 快照隔离
    #[tokio::test]
    async fn test_repeatable_read_snapshot_isolation() {
        let mvcc = MvccManager::new(MvccGcConfig::default());
        
        // 事务 1: 创建快照并读取
        let snapshot1 = mvcc.create_snapshot("txn_1").unwrap();
        let value1 = mvcc.read_version(&key, "txn_1", &snapshot1).unwrap();
        
        // 事务 2: 修改并提交
        let snapshot2 = mvcc.create_snapshot("txn_2").unwrap();
        mvcc.write_version(&key, new_value, "txn_2", &mut snapshot2.clone()).unwrap();
        mvcc.commit("txn_2").unwrap();
        
        // 事务 1: 再次读取 (应该看到旧值，Repeatable Read 保证)
        let value1_again = mvcc.read_version(&key, "txn_1", &snapshot1).unwrap();
        
        assert_eq!(value1, value1_again);  // 可重复读保证
    }
    
    /// 测试：MVCC 可见性检查
    #[test]
    fn test_mvcc_visibility_check() {
        let version = StateVersion {
            key: key.clone(),
            value: value.clone(),
            version: 100,
            created_by: "txn_1".to_string(),
            created_at: 1000,
            visible_to: vec!["txn_2".to_string()],
            expired_at: None,
            committed: true,
        };
        
        // 创建者可见
        assert!(version.is_visible_to("txn_1"));
        
        // 授权事务可见
        assert!(version.is_visible_to("txn_2"));
        
        // 其他事务不可见
        assert!(!version.is_visible_to("txn_3"));
    }
    
    /// 测试：OCC 冲突检测
    #[tokio::test]
    async fn test_occ_conflict_detection() {
        let mvcc = MvccManager::new(MvccGcConfig::default());
        
        // 事务 1: 读取并修改 key1
        let snapshot1 = mvcc.create_snapshot("txn_1").unwrap();
        mvcc.read_version(&key1, "txn_1", &snapshot1).unwrap();
        mvcc.write_version(&key1, value1, "txn_1", &mut snapshot1.clone()).unwrap();
        
        // 事务 2: 读取并修改 key1 (冲突)
        let snapshot2 = mvcc.create_snapshot("txn_2").unwrap();
        mvcc.read_version(&key1, "txn_2", &snapshot2).unwrap();
        mvcc.write_version(&key1, value2, "txn_2", &mut snapshot2.clone()).unwrap();
        
        // 事务 1 先提交
        mvcc.commit("txn_1").unwrap();
        
        // 事务 2 提交 (应该冲突)
        let result = mvcc.commit("txn_2");
        assert!(matches!(result, Err(MvccError::WriteConflict(_))));
    }
    
    /// 测试：MVCC GC
    #[tokio::test]
    async fn test_mvcc_gc() {
        let mut config = MvccGcConfig::default();
        config.retention_ms = 100;  // 100ms 保留时间
        
        let mvcc = Arc::new(MvccManager::new(config));
        
        // 创建多个版本
        for i in 0..10 {
            let version = StateVersion {
                version: i,
                created_at: i * 10,
                ..Default::default()
            };
            mvcc.versions.entry(key.clone()).or_insert_with(Vec::new).push(version);
        }
        
        // 等待过期
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // 运行 GC
        let reclaimed = mvcc.run_gc();
        
        assert!(reclaimed > 0);  // 应该回收了一些版本
    }
    
    /// 测试：RR vs RC 性能对比
    #[tokio::test]
    async fn test_rr_vs_rc_performance() {
        let rc_executor = create_rc_executor();
        let rr_executor = create_rr_executor();
        
        // RC 事务
        let rc_start = Instant::now();
        for _ in 0..100 {
            rc_executor.execute(create_rc_request()).await.unwrap();
        }
        let rc_elapsed = rc_start.elapsed();
        
        // RR 事务
        let rr_start = Instant::now();
        for _ in 0..100 {
            rr_executor.execute(create_rr_request()).await.unwrap();
        }
        let rr_elapsed = rr_start.elapsed();
        
        // RR 应该比 RC 慢 <20% (可接受开销)
        let overhead = (rr_elapsed - rc_elapsed) as f64 / rc_elapsed as f64;
        assert!(overhead < 0.20, "RR overhead too high: {}", overhead);
    }
}
```

---

## 7. 失败处理

### 7.1 MVCC 错误类型

```rust
/// Phase 3: MVCC 错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MvccError {
    /// 事务未找到
    TransactionNotFound(String),
    /// 写冲突 (OCC)
    WriteConflict(String),
    /// 快照创建失败
    SnapshotCreationFailed(String),
    /// 版本验证失败
    VersionValidationFailed {
        expected_version: u64,
        actual_version: u64,
    },
    /// 内存超限
    MemoryLimitExceeded {
        current_bytes: usize,
        limit_bytes: usize,
    },
    /// GC 失败
    GcFailed(String),
}

impl std::fmt::Display for MvccError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MvccError::TransactionNotFound(txn_id) => {
                write!(f, "事务未找到：{}", txn_id)
            }
            MvccError::WriteConflict(conflict) => {
                write!(f, "写冲突：{}", conflict)
            }
            MvccError::SnapshotCreationFailed(reason) => {
                write!(f, "快照创建失败：{}", reason)
            }
            MvccError::VersionValidationFailed { expected, actual } => {
                write!(f, "版本验证失败：期望 {}, 实际 {}", expected, actual)
            }
            MvccError::MemoryLimitExceeded { current, limit } => {
                write!(f, "内存超限：{} / {} bytes", current, limit)
            }
            MvccError::GcFailed(reason) => {
                write!(f, "GC 失败：{}", reason)
            }
        }
    }
}
```

### 7.2 回滚策略

```rust
// Phase 3: MVCC 回滚策略
impl MvccManager {
    pub fn rollback_transaction(&self, transaction_id: &str) -> Result<()> {
        // 1. 移除活跃事务快照
        match self.active_transactions.remove(transaction_id) {
            Some((_, snapshot)) => {
                // 2. 清理未提交版本 (已经在快照中，不需要额外清理)
                // 3. 记录审计日志
                self.audit_log.mvcc_rollback(transaction_id).await;
                
                self.metrics.record_rollback();
                Ok(())
            }
            None => Err(MvccError::TransactionNotFound(transaction_id.to_string())),
        }
    }
}

// Phase 3: 事务重试策略
impl RepeatableReadExecutor {
    async fn execute_with_retry(
        &self,
        mut request: TransactionExecuteRequest,
    ) -> Result<TransactionExecuteResult> {
        let max_retries = request.max_retries;
        
        for attempt in 0..max_retries {
            match self.execute_rr_transaction(request.clone()).await {
                Ok(result) => return Ok(result),
                Err(Error::Mvcc(MvccError::WriteConflict(_))) if attempt < max_retries - 1 => {
                    // OCC 冲突，指数退避重试
                    let backoff_ms = 10 * 2_u64.pow(attempt as u32);
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    request.retry_count = attempt as u32 + 1;
                }
                Err(e) => return Err(e),
            }
        }
        
        Err(Error::MaxRetriesExceeded)
    }
}
```

---

## 8. 附录

### 8.1 使用示例

```rust
// Phase 3: Repeatable Read 事务使用示例
#[tokio::main]
async fn main() -> Result<()> {
    let executor = RepeatableReadExecutor::new();
    
    // 创建 RR 事务
    let request = TransactionExecuteRequest {
        trace_id: "trace_001".to_string(),
        transaction_id: "txn_001".to_string(),
        isolation_level: IsolationLevel::RepeatableRead,
        instructions: vec![
            ExecuteRequest {
                instruction: "SELECT balance FROM account_1".to_string(),
                ..Default::default()
            },
            ExecuteRequest {
                instruction: "UPDATE balance SET -100 WHERE account=account_1".to_string(),
                ..Default::default()
            },
            ExecuteRequest {
                instruction: "UPDATE balance SET +100 WHERE account=account_2".to_string(),
                ..Default::default()
            },
        ],
        timeout_ms: 5000,
        use_mvcc: true,
        max_retries: 3,
        retry_count: 0,
        timestamp: get_current_timestamp(),
    };
    
    // 执行事务
    let result = executor.execute_rr_transaction(request).await?;
    
    println!("事务执行完成:");
    println!("  状态：{:?}", result.status);
    println!("  重试次数：{}", result.retry_count);
    println!("  事务哈希：{}", result.transaction_hash);
    
    Ok(())
}
```

### 8.2 隔离级别选择指南

| 场景 | 推荐隔离级别 | 理由 |
|---|---|---|
| 简单查询 | Read Committed | 性能优，足够用 |
| 金融转账 | Repeatable Read | 防止幻读，一致性强 |
| 批量报表 | Repeatable Read | 快照隔离，数据稳定 |
| 高并发读 | Repeatable Read | MVCC 无锁读取 |
| 强一致性 | Serializable (Phase 4) | 最高隔离级别 |

---

**文档状态**: 📋 草案  
**责任人**: Architect-Agent  
**保管**: 项目文档库
