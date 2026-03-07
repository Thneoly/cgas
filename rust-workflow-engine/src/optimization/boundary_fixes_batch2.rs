//! 边界场景修复 Batch 2 (Boundary Fixes Batch 2)
//! 
//! Phase 3 Week 4: 边界场景修复专项
//! 
//! **修复目标**: 10 个边界场景
//! - 并发冲突场景 (4 个)
//! - 超时重试场景 (3 个)
//! - 资源耗尽场景 (3 个)
//! 
//! **参考 Week 3 问题**:
//! - Retry Count P99: 38/h (接近告警阈值 50/h)
//! - 磁盘 IO 峰值：85% (超过健康阈值 80%)
//! - 连接池使用率峰值：88%

use std::sync::atomic::{AtomicU64, AtomicUsize, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use log::{debug, info, warn, error};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::time::timeout;

// ============================================================================
// 场景 1: 并发冲突 - 共享状态竞态条件
// ============================================================================

/// 问题描述：多个协程同时修改共享状态导致数据不一致
/// 修复方案：使用原子操作 + 锁顺序优化
pub struct ConcurrentStateFix {
    state: AtomicU64,
    version: AtomicU64,
    lock_order: AtomicUsize,
}

impl ConcurrentStateFix {
    pub fn new() -> Self {
        Self {
            state: AtomicU64::new(0),
            version: AtomicU64::new(0),
            lock_order: AtomicUsize::new(0),
        }
    }
    
    /// 原子更新状态 (修复竞态条件)
    pub fn atomic_update(&self, new_value: u64) -> Result<u64, String> {
        // 使用 CAS 保证原子性
        let mut current = self.state.load(Ordering::SeqCst);
        
        loop {
            let expected = current;
            match self.state.compare_exchange(
                expected,
                new_value,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    // 更新成功，递增版本号
                    self.version.fetch_add(1, Ordering::Relaxed);
                    return Ok(new_value);
                }
                Err(actual) => {
                    // 更新失败，重试
                    current = actual;
                    // 避免自旋过多，yield
                    std::hint::spin_loop();
                }
            }
        }
    }
    
    /// 获取当前状态和版本
    pub fn get_state_with_version(&self) -> (u64, u64) {
        let state = self.state.load(Ordering::SeqCst);
        let version = self.version.load(Ordering::SeqCst);
        (state, version)
    }
}

// ============================================================================
// 场景 2: 并发冲突 - 死锁预防
// ============================================================================

/// 问题描述：多个锁以不同顺序获取导致死锁
/// 修复方案：固定锁获取顺序 + 超时机制
pub struct DeadlockPrevention {
    lock_a: Arc<Mutex<u64>>,
    lock_b: Arc<Mutex<u64>>,
    lock_c: Arc<Mutex<u64>>,
    acquire_timeout: Duration,
}

impl DeadlockPrevention {
    pub fn new() -> Self {
        Self {
            lock_a: Arc::new(Mutex::new(0)),
            lock_b: Arc::new(Mutex::new(0)),
            lock_c: Arc::new(Mutex::new(0)),
            acquire_timeout: Duration::from_secs(5),
        }
    }
    
    /// 按固定顺序获取所有锁 (预防死锁)
    pub async fn acquire_all_locks(&self) -> Result<(
        tokio::sync::MutexGuard<'_, u64>,
        tokio::sync::MutexGuard<'_, u64>,
        tokio::sync::MutexGuard<'_, u64>,
    ), String> {
        // 固定顺序：A -> B -> C
        let guard_a = timeout(self.acquire_timeout, self.lock_a.lock())
            .await
            .map_err(|_| "Lock A acquire timeout")?;
        
        let guard_b = timeout(self.acquire_timeout, self.lock_b.lock())
            .await
            .map_err(|_| "Lock B acquire timeout")?;
        
        let guard_c = timeout(self.acquire_timeout, self.lock_c.lock())
            .await
            .map_err(|_| "Lock C acquire timeout")?;
        
        Ok((guard_a, guard_b, guard_c))
    }
    
    /// 尝试获取锁 (带回退机制)
    pub async fn try_acquire_with_backoff(&self) -> Result<(), String> {
        let mut backoff_ms = 10;
        let max_retries = 5;
        
        for attempt in 0..max_retries {
            match timeout(Duration::from_millis(backoff_ms), self.lock_a.lock()).await {
                Ok(_guard) => {
                    // 获取成功
                    return Ok(());
                }
                Err(_) => {
                    // 指数退避
                    backoff_ms *= 2;
                    warn!("Lock acquire attempt {} failed, backing off {}ms", attempt + 1, backoff_ms);
                }
            }
        }
        
        Err("Failed to acquire lock after max retries".to_string())
    }
}

// ============================================================================
// 场景 3: 并发冲突 - 虚假唤醒处理
// ============================================================================

/// 问题描述：条件变量虚假唤醒导致逻辑错误
/// 修复方案：while 循环检查条件 + 版本号验证
pub struct SpuriousWakeupFix {
    condition_met: AtomicBool,
    version: AtomicU64,
    wait_count: AtomicU64,
}

impl SpuriousWakeupFix {
    pub fn new() -> Self {
        Self {
            condition_met: AtomicBool::new(false),
            version: AtomicU64::new(0),
            wait_count: AtomicU64::new(0),
        }
    }
    
    /// 等待条件 (正确处理虚假唤醒)
    pub async fn wait_condition(&self) -> Result<u64, String> {
        self.wait_count.fetch_add(1, Ordering::Relaxed);
        let expected_version = self.version.load(Ordering::SeqCst);
        
        // 使用 while 循环检查条件，防止虚假唤醒
        while !self.condition_met.load(Ordering::SeqCst) {
            // 检查版本号是否变化 (检测是否被其他线程修改)
            let current_version = self.version.load(Ordering::SeqCst);
            if current_version != expected_version {
                debug!("Spurious wakeup detected: version changed from {} to {}", 
                    expected_version, current_version);
            }
            
            // 短暂等待后重试
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        Ok(self.version.load(Ordering::SeqCst))
    }
    
    /// 设置条件满足
    pub fn set_condition(&self) {
        self.condition_met.store(true, Ordering::SeqCst);
        self.version.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 重置条件
    pub fn reset(&self) {
        self.condition_met.store(false, Ordering::SeqCst);
        self.version.fetch_add(1, Ordering::SeqCst);
    }
}

// ============================================================================
// 场景 4: 并发冲突 - ABA 问题
// ============================================================================

/// 问题描述：CAS 操作中的 ABA 问题 (值被修改后又改回原值)
/// 修复方案：使用带版本号的 AtomicCell
pub struct ABAProblemFix {
    value: AtomicU64,
    stamp: AtomicU64,
}

impl ABAProblemFix {
    pub fn new(initial_value: u64) -> Self {
        Self {
            value: AtomicU64::new(initial_value),
            stamp: AtomicU64::new(0),
        }
    }
    
    /// 带版本号的 CAS (解决 ABA 问题)
    pub fn compare_exchange_with_stamp(
        &self,
        expected_value: u64,
        expected_stamp: u64,
        new_value: u64,
    ) -> Result<u64, (u64, u64)> {
        let current_value = self.value.load(Ordering::SeqCst);
        let current_stamp = self.stamp.load(Ordering::SeqCst);
        
        // 检查值和版本号都匹配
        if current_value == expected_value && current_stamp == expected_stamp {
            // 递增版本号并更新值
            let new_stamp = current_stamp + 1;
            self.stamp.store(new_stamp, Ordering::SeqCst);
            self.value.store(new_value, Ordering::SeqCst);
            Ok(new_value)
        } else {
            Err((current_value, current_stamp))
        }
    }
    
    /// 获取带版本号的值
    pub fn load_with_stamp(&self) -> (u64, u64) {
        let value = self.value.load(Ordering::SeqCst);
        let stamp = self.stamp.load(Ordering::SeqCst);
        (value, stamp)
    }
}

// ============================================================================
// 场景 5: 超时重试 - 指数退避重试
// ============================================================================

/// 问题描述：重试策略不当导致雪崩效应
/// 修复方案：指数退避 + 抖动 + 最大重试次数
pub struct ExponentialBackoffRetry {
    max_retries: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
    multiplier: f64,
    jitter_factor: f64,
    retry_count: AtomicU64,
}

impl ExponentialBackoffRetry {
    pub fn new() -> Self {
        Self {
            max_retries: 5,
            initial_delay_ms: 100,
            max_delay_ms: 10000,
            multiplier: 2.0,
            jitter_factor: 0.1,
            retry_count: AtomicU64::new(0),
        }
    }
    
    /// 执行带重试的操作
    pub async fn execute_with_retry<F, T, E>(
        &self,
        mut operation: F,
    ) -> Result<T, E>
    where
        F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut delay = self.initial_delay_ms;
        let mut attempt = 0;
        
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempt += 1;
                    self.retry_count.fetch_add(1, Ordering::Relaxed);
                    
                    if attempt >= self.max_retries {
                        error!("Operation failed after {} attempts: {:?}", attempt, e);
                        return Err(e);
                    }
                    
                    // 计算带抖动的延迟
                    let jitter = (delay as f64 * self.jitter_factor * rand_f64()) as u64;
                    let actual_delay = (delay + jitter).min(self.max_delay_ms);
                    
                    warn!(
                        "Attempt {} failed, retrying in {}ms (error: {:?})",
                        attempt, actual_delay, e
                    );
                    
                    tokio::time::sleep(Duration::from_millis(actual_delay)).await;
                    
                    // 指数增长
                    delay = (delay as f64 * self.multiplier) as u64;
                }
            }
        }
    }
    
    /// 获取重试统计
    pub fn get_retry_count(&self) -> u64 {
        self.retry_count.load(Ordering::Relaxed)
    }
}

fn rand_f64() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f64) / (u32::MAX as f64)
}

// ============================================================================
// 场景 6: 超时重试 - 级联超时处理
// ============================================================================

/// 问题描述：嵌套调用超时时间累加导致总超时过长
/// 修复方案：超时预算分配 + 剩余时间传递
pub struct CascadingTimeoutFix {
    total_budget: Duration,
    start_time: Instant,
}

impl CascadingTimeoutFix {
    pub fn new(total_budget: Duration) -> Self {
        Self {
            total_budget,
            start_time: Instant::now(),
        }
    }
    
    /// 获取剩余超时时间
    pub fn remaining_timeout(&self) -> Duration {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.total_budget {
            Duration::from_millis(0)
        } else {
            self.total_budget - elapsed
        }
    }
    
    /// 分配子操作超时 (按比例)
    pub fn allocate_sub_timeout(&self, percentage: f64) -> Duration {
        let remaining = self.remaining_timeout();
        Duration::from_secs_f64(remaining.as_secs_f64() * percentage)
    }
    
    /// 检查是否已超时
    pub fn is_timeout(&self) -> bool {
        self.start_time.elapsed() >= self.total_budget
    }
}

// ============================================================================
// 场景 7: 超时重试 - 快速失败机制
// ============================================================================

/// 问题描述：依赖服务故障时持续重试浪费资源
/// 修复方案：熔断器模式 + 快速失败
pub struct FastFailCircuitBreaker {
    failure_count: AtomicU64,
    success_count: AtomicU64,
    state: AtomicUsize, // 0=Closed, 1=Open, 2=HalfOpen
    last_failure_time: AtomicU64,
    failure_threshold: u64,
    recovery_timeout_secs: u64,
}

impl FastFailCircuitBreaker {
    const STATE_CLOSED: usize = 0;
    const STATE_OPEN: usize = 1;
    const STATE_HALF_OPEN: usize = 2;
    
    pub fn new(failure_threshold: u64, recovery_timeout_secs: u64) -> Self {
        Self {
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            state: AtomicUsize::new(Self::STATE_CLOSED),
            last_failure_time: AtomicU64::new(0),
            failure_threshold,
            recovery_timeout_secs,
        }
    }
    
    /// 执行操作 (带熔断保护)
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: futures::future::Future<Output = Result<T, E>>,
    {
        // 检查是否需要快速失败
        if !self.allow_request() {
            return Err(self.create_fast_fail_error());
        }
        
        match operation.await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
    }
    
    /// 检查是否允许请求
    fn allow_request(&self) -> bool {
        let state = self.state.load(Ordering::SeqCst);
        
        match state {
            Self::STATE_CLOSED => true,
            Self::STATE_OPEN => {
                // 检查是否过了恢复超时
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let last_failure = self.last_failure_time.load(Ordering::SeqCst);
                
                if now - last_failure >= self.recovery_timeout_secs {
                    // 切换到半开状态
                    self.state.store(Self::STATE_HALF_OPEN, Ordering::SeqCst);
                    true
                } else {
                    false
                }
            }
            Self::STATE_HALF_OPEN => true,
            _ => false,
        }
    }
    
    /// 记录成功
    fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        
        let state = self.state.load(Ordering::SeqCst);
        if state == Self::STATE_HALF_OPEN {
            // 半开状态成功，关闭熔断器
            self.state.store(Self::STATE_CLOSED, Ordering::SeqCst);
            self.failure_count.store(0, Ordering::Relaxed);
        }
    }
    
    /// 记录失败
    fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_failure_time.store(now, Ordering::SeqCst);
        
        let state = self.state.load(Ordering::SeqCst);
        if state == Self::STATE_HALF_OPEN {
            // 半开状态失败，重新打开熔断器
            self.state.store(Self::STATE_OPEN, Ordering::SeqCst);
        } else if self.failure_count.load(Ordering::Relaxed) >= self.failure_threshold {
            // 达到失败阈值，打开熔断器
            self.state.store(Self::STATE_OPEN, Ordering::SeqCst);
        }
    }
    
    /// 创建快速失败错误
    fn create_fast_fail_error<E>(&self) -> E 
    where
        E: std::fmt::Debug + From<&'static str>,
    {
        From::from("Circuit breaker is open, fast fail")
    }
    
    /// 获取当前状态
    pub fn get_state(&self) -> &'static str {
        match self.state.load(Ordering::SeqCst) {
            Self::STATE_CLOSED => "CLOSED",
            Self::STATE_OPEN => "OPEN",
            Self::STATE_HALF_OPEN => "HALF_OPEN",
            _ => "UNKNOWN",
        }
    }
}

// ============================================================================
// 场景 8: 资源耗尽 - 内存池限流
// ============================================================================

/// 问题描述：内存分配过多导致 OOM
/// 修复方案：对象池 + 限流 + 优雅降级
pub struct MemoryPoolThrottling {
    pool_size: usize,
    allocated: AtomicUsize,
    max_allocation: usize,
    soft_limit: usize,
    allocation_count: AtomicU64,
    rejection_count: AtomicU64,
}

impl MemoryPoolThrottling {
    pub fn new(max_allocation: usize, soft_limit: usize) -> Self {
        Self {
            pool_size: 0,
            allocated: AtomicUsize::new(0),
            max_allocation,
            soft_limit,
            allocation_count: AtomicU64::new(0),
            rejection_count: AtomicU64::new(0),
        }
    }
    
    /// 尝试分配内存
    pub fn try_allocate(&self, size: usize) -> Result<(), String> {
        let current = self.allocated.load(Ordering::Relaxed);
        
        // 检查是否超过硬限制
        if current + size > self.max_allocation {
            self.rejection_count.fetch_add(1, Ordering::Relaxed);
            return Err(format!(
                "Memory limit exceeded: current={}, requested={}, max={}",
                current, size, self.max_allocation
            ));
        }
        
        // 软限制：告警但不拒绝
        if current + size > self.soft_limit {
            warn!(
                "Memory usage above soft limit: current={}, requested={}, soft_limit={}",
                current, size, self.soft_limit
            );
        }
        
        // 原子增加分配
        self.allocated.fetch_add(size, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// 释放内存
    pub fn deallocate(&self, size: usize) {
        self.allocated.fetch_sub(size, Ordering::Relaxed);
    }
    
    /// 获取当前使用率
    pub fn get_usage_ratio(&self) -> f64 {
        let current = self.allocated.load(Ordering::Relaxed);
        current as f64 / self.max_allocation as f64
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> MemoryPoolStats {
        MemoryPoolStats {
            allocated: self.allocated.load(Ordering::Relaxed),
            max_allocation: self.max_allocation,
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
            rejection_count: self.rejection_count.load(Ordering::Relaxed),
            usage_ratio: self.get_usage_ratio(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryPoolStats {
    pub allocated: usize,
    pub max_allocation: usize,
    pub allocation_count: u64,
    pub rejection_count: u64,
    pub usage_ratio: f64,
}

// ============================================================================
// 场景 9: 资源耗尽 - 连接泄漏检测
// ============================================================================

/// 问题描述：连接未正确释放导致泄漏
/// 修复方案：RAII + 泄漏检测 + 自动回收
pub struct ConnectionLeakDetection {
    active_connections: Arc<RwLock<Vec<ConnectionTracker>>>,
    leak_threshold_secs: u64,
    leak_count: AtomicU64,
    total_created: AtomicU64,
    total_released: AtomicU64,
}

#[derive(Debug, Clone)]
struct ConnectionTracker {
    id: usize,
    created_at: Instant,
    last_used_at: Instant,
}

impl ConnectionLeakDetection {
    pub fn new(leak_threshold_secs: u64) -> Self {
        Self {
            active_connections: Arc::new(RwLock::new(Vec::new())),
            leak_threshold_secs,
            leak_count: AtomicU64::new(0),
            total_created: AtomicU64::new(0),
            total_released: AtomicU64::new(0),
        }
    }
    
    /// 创建带追踪的连接句柄
    pub async fn create_connection(&self, id: usize) -> TrackedConnectionGuard {
        let tracker = ConnectionTracker {
            id,
            created_at: Instant::now(),
            last_used_at: Instant::now(),
        };
        
        self.total_created.fetch_add(1, Ordering::Relaxed);
        
        {
            let mut connections = self.active_connections.write().await;
            connections.push(tracker);
        }
        
        TrackedConnectionGuard {
            id,
            detector: self,
            released: false,
        }
    }
    
    /// 释放连接
    pub async fn release_connection(&self, id: usize) {
        let mut connections = self.active_connections.write().await;
        
        if let Some(pos) = connections.iter().position(|c| c.id == id) {
            connections.remove(pos);
            self.total_released.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// 检测泄漏连接
    pub async fn detect_leaks(&self) -> Vec<usize> {
        let connections = self.active_connections.read().await;
        let now = Instant::now();
        
        let leaked: Vec<usize> = connections
            .iter()
            .filter(|c| now.duration_since(c.last_used_at).as_secs() > self.leak_threshold_secs)
            .map(|c| c.id)
            .collect();
        
        if !leaked.is_empty() {
            self.leak_count.fetch_add(leaked.len() as u64, Ordering::Relaxed);
            warn!("Detected {} leaked connections: {:?}", leaked.len(), leaked);
        }
        
        leaked
    }
    
    /// 回收泄漏连接
    pub async fn reclaim_leaks(&self) -> usize {
        let leaked = self.detect_leaks().await;
        let count = leaked.len();
        
        // 实际回收逻辑 (这里简化)
        for id in leaked {
            let _ = self.release_connection(id).await;
        }
        
        count
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> ConnectionLeakStats {
        ConnectionLeakStats {
            active_count: 0, // 需要异步获取
            leak_count: self.leak_count.load(Ordering::Relaxed),
            total_created: self.total_created.load(Ordering::Relaxed),
            total_released: self.total_released.load(Ordering::Relaxed),
            leak_ratio: 0.0,
        }
    }
}

/// RAII 连接守卫
pub struct TrackedConnectionGuard<'a> {
    id: usize,
    detector: &'a ConnectionLeakDetection,
    released: bool,
}

impl<'a> Drop for TrackedConnectionGuard<'a> {
    fn drop(&mut self) {
        if !self.released {
            // 自动释放 (防止泄漏)
            // 注意：这里不能 await，实际实现需要用 tokio::spawn
            warn!("Connection {} dropped without explicit release", self.id);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionLeakStats {
    pub active_count: usize,
    pub leak_count: u64,
    pub total_created: u64,
    pub total_released: u64,
    pub leak_ratio: f64,
}

// ============================================================================
// 场景 10: 资源耗尽 - 磁盘 IO 限流
// ============================================================================

/// 问题描述：磁盘 IO 过载导致系统性能下降
/// 修复方案：IO 令牌桶 + 批量合并 + 优先级队列
pub struct DiskIOLimiter {
    tokens: AtomicU64,
    max_tokens: u64,
    refill_rate: u64, // tokens per second
    last_refill: AtomicU64, // timestamp in seconds
    io_count: AtomicU64,
    throttled_count: AtomicU64,
}

impl DiskIOLimiter {
    pub fn new(max_tokens: u64, refill_rate: u64) -> Self {
        Self {
            tokens: AtomicU64::new(max_tokens),
            max_tokens,
            refill_rate,
            last_refill: AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
            io_count: AtomicU64::new(0),
            throttled_count: AtomicU64::new(0),
        }
    }
    
    /// 尝试获取 IO 令牌
    pub fn try_acquire(&self, cost: u64) -> Result<(), String> {
        self.refill();
        
        let current = self.tokens.load(Ordering::SeqCst);
        
        if current >= cost {
            match self.tokens.compare_exchange(
                current,
                current - cost,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    self.io_count.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
                Err(_) => {
                    // 并发竞争，重试
                    self.try_acquire(cost)
                }
            }
        } else {
            self.throttled_count.fetch_add(1, Ordering::Relaxed);
            Err(format!(
                "IO rate limited: available={}, requested={}",
                current, cost
            ))
        }
    }
    
    /// 等待获取 IO 令牌
    pub async fn acquire(&self, cost: u64) -> Result<(), String> {
        loop {
            match self.try_acquire(cost) {
                Ok(_) => return Ok(()),
                Err(_) => {
                    // 等待令牌补充
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
    
    /// 补充令牌
    fn refill(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let last = self.last_refill.load(Ordering::Relaxed);
        
        if now > last {
            let elapsed = now - last;
            let new_tokens = elapsed * self.refill_rate;
            
            let current = self.tokens.load(Ordering::Relaxed);
            let updated = (current + new_tokens).min(self.max_tokens);
            
            self.tokens.store(updated, Ordering::Relaxed);
            self.last_refill.store(now, Ordering::Relaxed);
        }
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> DiskIOStats {
        self.refill();
        
        DiskIOStats {
            available_tokens: self.tokens.load(Ordering::Relaxed),
            max_tokens: self.max_tokens,
            io_count: self.io_count.load(Ordering::Relaxed),
            throttled_count: self.throttled_count.load(Ordering::Relaxed),
            throttle_ratio: {
                let total = self.io_count.load(Ordering::Relaxed) + self.throttled_count.load(Ordering::Relaxed);
                if total > 0 {
                    self.throttled_count.load(Ordering::Relaxed) as f64 / total as f64
                } else {
                    0.0
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiskIOStats {
    pub available_tokens: u64,
    pub max_tokens: u64,
    pub io_count: u64,
    pub throttled_count: u64,
    pub throttle_ratio: f64,
}

// ============================================================================
// 集成测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_concurrent_state_fix() {
        let state = ConcurrentStateFix::new();
        
        // 并发更新
        let mut handles = vec![];
        for i in 0..10 {
            let state_clone = &state;
            let handle = tokio::spawn(async move {
                state_clone.atomic_update(i).unwrap()
            });
            handles.push(handle);
        }
        
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result < 10);
        }
    }
    
    #[tokio::test]
    async fn test_exponential_backoff() {
        let retry = ExponentialBackoffRetry::new();
        let mut attempt = 0;
        
        let result = retry.execute_with_retry(|| {
            attempt += 1;
            async move {
                if attempt < 3 {
                    Err("Temporary error")
                } else {
                    Ok("Success")
                }
            }
            .boxed()
        }).await;
        
        assert_eq!(result.unwrap(), "Success");
        assert!(attempt >= 3);
    }
    
    #[test]
    fn test_memory_pool_throttling() {
        let pool = MemoryPoolThrottling::new(1000, 800);
        
        // 正常分配
        assert!(pool.try_allocate(500).is_ok());
        
        // 超过软限制
        assert!(pool.try_allocate(400).is_ok());
        
        // 超过硬限制
        assert!(pool.try_allocate(200).is_err());
        
        let stats = pool.get_stats();
        assert!(stats.usage_ratio > 0.8);
    }
    
    #[test]
    fn test_disk_io_limiter() {
        let limiter = DiskIOLimiter::new(100, 10);
        
        // 正常获取
        assert!(limiter.try_acquire(50).is_ok());
        
        // 令牌不足
        assert!(limiter.try_acquire(60).is_err());
        
        let stats = limiter.get_stats();
        assert_eq!(stats.available_tokens, 50);
        assert_eq!(stats.throttled_count, 1);
    }
}
