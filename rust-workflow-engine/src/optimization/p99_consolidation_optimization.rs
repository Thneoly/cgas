//! P99 巩固优化 (P99 Consolidation Optimization)
//! 
//! Phase 3 Week 5: P99 巩固优化专项
//! 
//! **优化目标**: P99 <160ms (Week 4 基线：165ms)
//! 
//! **优化策略**:
//! 1. 热点路径分析 - 识别并优化 Top 5 热点
//! 2. 内存访问模式优化 - 减少 cache miss
//! 3. 锁竞争优化 - 减少临界区时间
//! 4. 分支预测优化 - 减少 branch misprediction
//! 
//! **预期收益**: -5ms P99 (165ms → 160ms)

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use log::{debug, info, warn};

/// P99 优化配置
#[derive(Debug, Clone)]
pub struct P99ConsolidationConfig {
    /// 是否启用热点路径分析
    pub enable_hotspot_analysis: bool,
    /// 是否启用内存访问优化
    pub enable_memory_optimization: bool,
    /// 是否启用锁竞争优化
    pub enable_lock_optimization: bool,
    /// 是否启用分支预测优化
    pub enable_branch_optimization: bool,
    /// 热点路径采样率 (0.0-1.0)
    pub hotspot_sampling_rate: f64,
    /// 内存对齐大小
    pub memory_alignment: usize,
}

impl Default for P99ConsolidationConfig {
    fn default() -> Self {
        Self {
            enable_hotspot_analysis: true,
            enable_memory_optimization: true,
            enable_lock_optimization: true,
            enable_branch_optimization: false,
            hotspot_sampling_rate: 0.1,
            memory_alignment: 64, // Cache line size
        }
    }
}

/// 热点路径分析器
pub struct HotspotAnalyzer {
    /// 调用计数
    call_counts: Arc<Vec<AtomicU64>>,
    /// 累计耗时 (微秒)
    total_durations_us: Arc<Vec<AtomicU64>>,
    /// 路径名称
    path_names: Vec<String>,
    /// 采样计数器
    sample_counter: AtomicU64,
    /// 采样率
    sampling_rate: f64,
}

impl HotspotAnalyzer {
    /// 创建分析器
    pub fn new(path_names: Vec<String>, sampling_rate: f64) -> Self {
        let count = path_names.len();
        Self {
            call_counts: Arc::new((0..count).map(|_| AtomicU64::new(0)).collect()),
            total_durations_us: Arc::new((0..count).map(|_| AtomicU64::new(0)).collect()),
            path_names,
            sample_counter: AtomicU64::new(0),
            sampling_rate,
        }
    }
    
    /// 记录路径调用
    pub fn record_path(&self, path_id: usize, duration_us: u64) {
        // 采样检查
        if self.sampling_rate < 1.0 {
            let sample = self.sample_counter.fetch_add(1, Ordering::Relaxed);
            if (sample as f64 * self.sampling_rate) % 1.0 != 0.0 {
                return;
            }
        }
        
        if path_id < self.call_counts.len() {
            self.call_counts[path_id].fetch_add(1, Ordering::Relaxed);
            self.total_durations_us[path_id].fetch_add(duration_us, Ordering::Relaxed);
        }
    }
    
    /// 获取热点路径报告
    pub fn get_hotspot_report(&self) -> Vec<HotspotPathInfo> {
        let mut report = Vec::new();
        
        for (i, name) in self.path_names.iter().enumerate() {
            let calls = self.call_counts[i].load(Ordering::Relaxed);
            let total_us = self.total_durations_us[i].load(Ordering::Relaxed);
            
            if calls > 0 {
                let avg_us = total_us as f64 / calls as f64;
                report.push(HotspotPathInfo {
                    path_id: i,
                    name: name.clone(),
                    call_count: calls,
                    total_duration_us: total_us,
                    avg_duration_us: avg_us,
                });
            }
        }
        
        // 按总耗时排序
        report.sort_by(|a, b| {
            b.total_duration_us.cmp(&a.total_duration_us)
        });
        
        report
    }
    
    /// 获取 Top N 热点路径
    pub fn get_top_hotspots(&self, n: usize) -> Vec<HotspotPathInfo> {
        let report = self.get_hotspot_report();
        report.into_iter().take(n).collect()
    }
}

/// 热点路径信息
#[derive(Debug, Clone)]
pub struct HotspotPathInfo {
    pub path_id: usize,
    pub name: String,
    pub call_count: u64,
    pub total_duration_us: u64,
    pub avg_duration_us: f64,
}

/// 内存访问优化器
pub struct MemoryAccessOptimizer {
    /// 预取距离 (元素数)
    prefetch_distance: usize,
    /// 数据对齐大小
    alignment: usize,
    /// cache miss 计数
    cache_miss_count: AtomicU64,
    /// 访问总数
    access_count: AtomicU64,
}

impl MemoryAccessOptimizer {
    /// 创建优化器
    pub fn new(prefetch_distance: usize, alignment: usize) -> Self {
        Self {
            prefetch_distance,
            alignment,
            cache_miss_count: AtomicU64::new(0),
            access_count: AtomicU64::new(0),
        }
    }
    
    /// 对齐内存分配
    pub fn aligned_allocate<T>(&self, size: usize) -> Vec<T> 
    where
        T: Default + Clone,
    {
        // 计算对齐后的大小
        let aligned_size = ((size + self.alignment - 1) / self.alignment) * self.alignment;
        
        // 分配对齐的内存
        let mut vec = Vec::with_capacity(aligned_size);
        for _ in 0..aligned_size {
            vec.push(T::default());
        }
        
        vec
    }
    
    /// 预取数据 (提示 CPU 预加载)
    pub fn prefetch<T>(&self, data: &[T], index: usize) {
        if index + self.prefetch_distance < data.len() {
            // Rust 没有直接的预取指令，这里通过访问来触发
            // 实际实现可以使用 core::arch 的 prefetch 指令
            let _ = &data[index + self.prefetch_distance];
        }
    }
    
    /// 记录 cache miss (估算)
    pub fn record_cache_miss(&self) {
        self.cache_miss_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录内存访问
    pub fn record_access(&self) {
        self.access_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 获取 cache miss 率
    pub fn get_cache_miss_rate(&self) -> f64 {
        let misses = self.cache_miss_count.load(Ordering::Relaxed);
        let accesses = self.access_count.load(Ordering::Relaxed);
        
        if accesses > 0 {
            misses as f64 / accesses as f64
        } else {
            0.0
        }
    }
}

/// 锁竞争优化器
pub struct LockContentionOptimizer {
    /// 锁获取成功次数
    lock_success_count: AtomicU64,
    /// 锁等待次数
    lock_wait_count: AtomicU64,
    /// 总等待时间 (微秒)
    total_wait_time_us: AtomicU64,
    /// 自旋次数限制
    spin_limit: usize,
}

impl LockContentionOptimizer {
    /// 创建优化器
    pub fn new(spin_limit: usize) -> Self {
        Self {
            lock_success_count: AtomicU64::new(0),
            lock_wait_count: AtomicU64::new(0),
            total_wait_time_us: AtomicU64::new(0),
            spin_limit,
        }
    }
    
    /// 尝试获取锁 (带自旋优化)
    pub fn try_acquire_lock<F, R>(&self, try_lock: F) -> Option<R>
    where
        F: Fn() -> Option<R>,
    {
        let start = Instant::now();
        
        // 自旋尝试
        for _ in 0..self.spin_limit {
            if let Some(result) = try_lock() {
                self.lock_success_count.fetch_add(1, Ordering::Relaxed);
                let wait_time = start.elapsed().as_micros() as u64;
                self.total_wait_time_us.fetch_add(wait_time, Ordering::Relaxed);
                return Some(result);
            }
            
            // 短暂自旋
            std::hint::spin_loop();
        }
        
        // 自旋失败，记录等待
        self.lock_wait_count.fetch_add(1, Ordering::Relaxed);
        None
    }
    
    /// 记录锁获取成功
    pub fn record_lock_success(&self, wait_time_us: u64) {
        self.lock_success_count.fetch_add(1, Ordering::Relaxed);
        self.total_wait_time_us.fetch_add(wait_time_us, Ordering::Relaxed);
    }
    
    /// 记录锁等待
    pub fn record_lock_wait(&self) {
        self.lock_wait_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 获取锁竞争统计
    pub fn get_contention_stats(&self) -> LockContentionStats {
        let success = self.lock_success_count.load(Ordering::Relaxed);
        let waits = self.lock_wait_count.load(Ordering::Relaxed);
        let total_wait = self.total_wait_time_us.load(Ordering::Relaxed);
        
        LockContentionStats {
            success_count: success,
            wait_count: waits,
            total_wait_time_us: total_wait,
            avg_wait_time_us: if success > 0 { total_wait as f64 / success as f64 } else { 0.0 },
            contention_rate: if success + waits > 0 {
                waits as f64 / (success + waits) as f64
            } else {
                0.0
            },
        }
    }
}

/// 锁竞争统计
#[derive(Debug, Clone)]
pub struct LockContentionStats {
    pub success_count: u64,
    pub wait_count: u64,
    pub total_wait_time_us: u64,
    pub avg_wait_time_us: f64,
    pub contention_rate: f64,
}

/// 分支预测优化器
pub struct BranchPredictionOptimizer {
    /// 分支预测错误计数
    mispredict_count: AtomicU64,
    /// 总分支数
    total_branches: AtomicU64,
    /// 热点分支阈值
    hot_branch_threshold: u64,
}

impl BranchPredictionOptimizer {
    /// 创建优化器
    pub fn new(hot_branch_threshold: u64) -> Self {
        Self {
            mispredict_count: AtomicU64::new(0),
            total_branches: AtomicU64::new(0),
            hot_branch_threshold,
        }
    }
    
    /// 记录分支执行
    pub fn record_branch(&self, predicted: bool, actual: bool) {
        self.total_branches.fetch_add(1, Ordering::Relaxed);
        
        if predicted != actual {
            self.mispredict_count.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// 优化热点分支 (使用查找表替代条件判断)
    pub fn optimize_hot_branch<T, F>(&self, branch_id: u64, values: &[T], selector: F) -> &T
    where
        F: Fn() -> usize,
    {
        // 对于热点分支，使用查找表替代 if/else
        // 这可以减少分支预测错误
        let index = selector();
        &values[index.min(values.len() - 1)]
    }
    
    /// 获取分支预测统计
    pub fn get_branch_stats(&self) -> BranchStats {
        let total = self.total_branches.load(Ordering::Relaxed);
        let mispredicts = self.mispredict_count.load(Ordering::Relaxed);
        
        BranchStats {
            total_branches: total,
            mispredict_count: mispredicts,
            mispredict_rate: if total > 0 {
                mispredicts as f64 / total as f64
            } else {
                0.0
            },
        }
    }
}

/// 分支统计
#[derive(Debug, Clone)]
pub struct BranchStats {
    pub total_branches: u64,
    pub mispredict_count: u64,
    pub mispredict_rate: f64,
}

/// P99 巩固优化器
pub struct P99ConsolidationOptimizer {
    config: P99ConsolidationConfig,
    hotspot_analyzer: Option<HotspotAnalyzer>,
    memory_optimizer: Option<MemoryAccessOptimizer>,
    lock_optimizer: Option<LockContentionOptimizer>,
    branch_optimizer: Option<BranchPredictionOptimizer>,
    /// 优化前后对比统计
    before_p99_ms: AtomicU64,
    after_p99_ms: AtomicU64,
}

impl P99ConsolidationOptimizer {
    /// 创建优化器
    pub fn new(config: P99ConsolidationConfig) -> Self {
        let hotspot_analyzer = if config.enable_hotspot_analysis {
            Some(HotspotAnalyzer::new(
                vec![
                    "instruction_execute".to_string(),
                    "verification_check".to_string(),
                    "cache_lookup".to_string(),
                    "lock_acquire".to_string(),
                    "memory_allocate".to_string(),
                ],
                config.hotspot_sampling_rate,
            ))
        } else {
            None
        };
        
        let memory_optimizer = if config.enable_memory_optimization {
            Some(MemoryAccessOptimizer::new(8, config.memory_alignment))
        } else {
            None
        };
        
        let lock_optimizer = if config.enable_lock_optimization {
            Some(LockContentionOptimizer::new(100))
        } else {
            None
        };
        
        let branch_optimizer = if config.enable_branch_optimization {
            Some(BranchPredictionOptimizer::new(1000))
        } else {
            None
        };
        
        Self {
            config,
            hotspot_analyzer,
            memory_optimizer,
            lock_optimizer,
            branch_optimizer,
            before_p99_ms: AtomicU64::new(0),
            after_p99_ms: AtomicU64::new(0),
        }
    }
    
    /// 记录优化前 P99
    pub fn record_before_p99(&self, p99_ms: u64) {
        self.before_p99_ms.store(p99_ms, Ordering::Relaxed);
    }
    
    /// 记录优化后 P99
    pub fn record_after_p99(&self, p99_ms: u64) {
        self.after_p99_ms.store(p99_ms, Ordering::Relaxed);
    }
    
    /// 获取优化效果
    pub fn get_optimization_effect(&self) -> OptimizationEffect {
        let before = self.before_p99_ms.load(Ordering::Relaxed);
        let after = self.after_p99_ms.load(Ordering::Relaxed);
        
        let improvement_ms = if before > after { before - after } else { 0 };
        let improvement_percent = if before > 0 {
            (improvement_ms as f64 / before as f64) * 100.0
        } else {
            0.0
        };
        
        OptimizationEffect {
            before_p99_ms: before,
            after_p99_ms: after,
            improvement_ms,
            improvement_percent,
        }
    }
    
    /// 获取热点路径报告
    pub fn get_hotspot_report(&self) -> Option<Vec<HotspotPathInfo>> {
        self.hotspot_analyzer.as_ref().map(|a| a.get_top_hotspots(5))
    }
    
    /// 获取内存优化统计
    pub fn get_memory_stats(&self) -> Option<MemoryStats> {
        self.memory_optimizer.as_ref().map(|m| MemoryStats {
            cache_miss_rate: m.get_cache_miss_rate(),
            alignment: m.alignment,
            prefetch_distance: m.prefetch_distance,
        })
    }
    
    /// 获取锁竞争统计
    pub fn get_lock_stats(&self) -> Option<LockContentionStats> {
        self.lock_optimizer.as_ref().map(|l| l.get_contention_stats())
    }
    
    /// 获取分支预测统计
    pub fn get_branch_stats(&self) -> Option<BranchStats> {
        self.branch_optimizer.as_ref().map(|b| b.get_branch_stats())
    }
}

/// 优化效果
#[derive(Debug, Clone)]
pub struct OptimizationEffect {
    pub before_p99_ms: u64,
    pub after_p99_ms: u64,
    pub improvement_ms: u64,
    pub improvement_percent: f64,
}

/// 内存统计
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub cache_miss_rate: f64,
    pub alignment: usize,
    pub prefetch_distance: usize,
}

/// 性能分析守卫
pub struct PerformanceGuard<'a> {
    analyzer: Option<&'a HotspotAnalyzer>,
    path_id: usize,
    start: Instant,
}

impl<'a> PerformanceGuard<'a> {
    /// 创建守卫
    pub fn new(analyzer: Option<&'a HotspotAnalyzer>, path_id: usize) -> Self {
        Self {
            analyzer,
            path_id,
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for PerformanceGuard<'a> {
    fn drop(&mut self) {
        if let Some(analyzer) = self.analyzer {
            let duration_us = self.start.elapsed().as_micros() as u64;
            analyzer.record_path(self.path_id, duration_us);
        }
    }
}

/// 创建性能分析守卫
pub fn profile_path(analyzer: &HotspotAnalyzer, path_id: usize) -> PerformanceGuard {
    PerformanceGuard::new(Some(analyzer), path_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hotspot_analyzer() {
        let paths = vec![
            "path1".to_string(),
            "path2".to_string(),
            "path3".to_string(),
        ];
        
        let analyzer = HotspotAnalyzer::new(paths, 1.0);
        
        // 记录调用
        analyzer.record_path(0, 1000);
        analyzer.record_path(0, 1200);
        analyzer.record_path(1, 500);
        analyzer.record_path(2, 2000);
        
        let report = analyzer.get_hotspot_report();
        assert_eq!(report.len(), 3);
        
        // path2 应该是热点 (总耗时最长)
        assert_eq!(report[0].name, "path3");
    }
    
    #[test]
    fn test_memory_optimizer() {
        let optimizer = MemoryAccessOptimizer::new(8, 64);
        
        // 测试对齐分配
        let data: Vec<u8> = optimizer.aligned_allocate(100);
        assert!(data.len() >= 100);
        assert!(data.len() % 64 == 0);
        
        // 测试预取
        optimizer.prefetch(&data, 0);
        
        // 测试 cache miss 率
        optimizer.record_access();
        optimizer.record_access();
        optimizer.record_cache_miss();
        
        let miss_rate = optimizer.get_cache_miss_rate();
        assert!((miss_rate - 0.5).abs() < 0.01);
    }
    
    #[test]
    fn test_lock_optimizer() {
        let optimizer = LockContentionOptimizer::new(10);
        
        let mut lock_acquired = false;
        let result = optimizer.try_acquire_lock(|| {
            if !lock_acquired {
                lock_acquired = true;
                Some(42)
            } else {
                None
            }
        });
        
        assert_eq!(result, Some(42));
        
        let stats = optimizer.get_contention_stats();
        assert_eq!(stats.success_count, 1);
    }
    
    #[test]
    fn test_branch_optimizer() {
        let optimizer = BranchPredictionOptimizer::new(100);
        
        // 记录分支预测
        optimizer.record_branch(true, true); // 正确预测
        optimizer.record_branch(true, false); // 预测错误
        optimizer.record_branch(false, false); // 正确预测
        
        let stats = optimizer.get_branch_stats();
        assert_eq!(stats.total_branches, 3);
        assert_eq!(stats.mispredict_count, 1);
        assert!((stats.mispredict_rate - 0.333).abs() < 0.01);
    }
    
    #[test]
    fn test_p99_consolidation_optimizer() {
        let config = P99ConsolidationConfig::default();
        let optimizer = P99ConsolidationOptimizer::new(config);
        
        // 记录优化前后 P99
        optimizer.record_before_p99(165);
        optimizer.record_after_p99(160);
        
        let effect = optimizer.get_optimization_effect();
        assert_eq!(effect.before_p99_ms, 165);
        assert_eq!(effect.after_p99_ms, 160);
        assert_eq!(effect.improvement_ms, 5);
        assert!((effect.improvement_percent - 3.03).abs() < 0.01);
    }
}
