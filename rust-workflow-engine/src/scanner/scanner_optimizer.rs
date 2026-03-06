//! 扫描器优化
//! 
//! 实现非确定性扫描器的误报率优化
//! Phase 2 Week 4 扫描器优化关键组件

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{info, debug, warn};

/// 扫描器优化配置
#[derive(Debug, Clone)]
pub struct ScannerOptimizerConfig {
    /// 锁检测灵敏度 (0.0-1.0)
    pub lock_detection_sensitivity: f64,
    /// 时间窗口 (毫秒)
    pub time_window_ms: u64,
    /// 误报率目标
    pub false_positive_target: f64,
    /// 是否启用自适应优化
    pub adaptive_optimization: bool,
}

impl Default for ScannerOptimizerConfig {
    fn default() -> Self {
        Self {
            lock_detection_sensitivity: 0.7,
            time_window_ms: 50, // Phase 2 优化：从 1ms 提升到 50ms
            false_positive_target: 0.02, // 2%
            adaptive_optimization: true,
        }
    }
}

/// 扫描器优化器
pub struct ScannerOptimizer {
    /// 配置
    config: ScannerOptimizerConfig,
    /// 误报统计
    false_positives: u64,
    /// 真阳性统计
    true_positives: u64,
    /// 总检测数
    total_detections: u64,
    /// 路径类型缓存
    path_type_cache: HashMap<String, PathType>,
}

/// 路径类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PathType {
    /// 确定性路径
    Deterministic,
    /// 非确定性路径 - 并发竞争
    NonDeterministicConcurrency,
    /// 非确定性路径 - 时序依赖
    NonDeterministicTiming,
    /// 非确定性路径 - 外部依赖
    NonDeterministicExternal,
    /// 非确定性路径 - 资源竞争
    NonDeterministicResource,
}

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// 路径 ID
    pub path_id: String,
    /// 路径类型
    pub path_type: PathType,
    /// 是否确定性
    pub is_deterministic: bool,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 误报风险
    pub false_positive_risk: f64,
    /// 优化建议
    pub optimization_suggestions: Vec<String>,
}

/// 锁信息
#[derive(Debug, Clone)]
pub struct LockInfo {
    /// 锁 ID
    pub lock_id: String,
    /// 锁类型
    pub lock_type: LockType,
    /// 锁粒度
    pub granularity: LockGranularity,
    /// 保护的资源
    pub protected_resources: Vec<String>,
}

/// 锁类型
#[derive(Debug, Clone, PartialEq)]
pub enum LockType {
    /// 互斥锁
    Mutex,
    /// 读写锁
    RwLock,
    /// 自旋锁
    SpinLock,
    /// 无锁
    LockFree,
}

/// 锁粒度
#[derive(Debug, Clone, PartialEq)]
pub enum LockGranularity {
    /// 粗粒度
    Coarse,
    /// 中等粒度
    Medium,
    /// 细粒度
    Fine,
}

impl ScannerOptimizer {
    /// 创建新的扫描器优化器
    pub fn new(config: ScannerOptimizerConfig) -> Self {
        info!("ScannerOptimizer created: target_fp_rate={}", config.false_positive_target);
        
        Self {
            config,
            false_positives: 0,
            true_positives: 0,
            total_detections: 0,
            path_type_cache: HashMap::new(),
        }
    }

    /// 扫描路径并判断是否为非确定性
    pub fn scan_path(&mut self, path: &ExecutionPath) -> ScanResult {
        self.total_detections += 1;
        
        // 1. 检查缓存
        if let Some(cached_type) = self.path_type_cache.get(&path.id) {
            return self.create_scan_result(path, cached_type.clone(), true);
        }
        
        // 2. 分析路径
        let path_type = self.analyze_path(path);
        
        // 3. 缓存结果
        self.path_type_cache.insert(path.id.clone(), path_type.clone());
        
        // 4. 创建扫描结果
        self.create_scan_result(path, path_type, false)
    }

    /// 分析路径
    fn analyze_path(&self, path: &ExecutionPath) -> PathType {
        // 1. 检查锁保护
        if let Some(lock_info) = &path.lock_info {
            if self.is_lock_protected(lock_info, path) {
                return PathType::Deterministic;
            }
        }
        
        // 2. 检查时序依赖
        if self.has_timing_dependency(path) {
            // 优化：检查时间窗口
            if !self.is_within_time_window(path) {
                return PathType::Deterministic;
            }
            return PathType::NonDeterministicTiming;
        }
        
        // 3. 检查并发竞争
        if self.has_concurrency_race(path) {
            return PathType::NonDeterministicConcurrency;
        }
        
        // 4. 检查外部依赖
        if self.has_external_dependency(path) {
            return PathType::NonDeterministicExternal;
        }
        
        // 5. 检查资源竞争
        if self.has_resource_contention(path) {
            return PathType::NonDeterministicResource;
        }
        
        // 默认为确定性路径
        PathType::Deterministic
    }

    /// 检查锁保护
    fn is_lock_protected(&self, lock_info: &LockInfo, path: &ExecutionPath) -> bool {
        // 检查锁粒度是否匹配操作
        match lock_info.granularity {
            LockGranularity::Fine => {
                // 细粒度锁，检查是否覆盖当前操作
                lock_info.protected_resources.iter().any(|r| {
                    path.operations.iter().any(|op| op.resource == *r)
                })
            }
            LockGranularity::Medium => {
                // 中等粒度锁，检查资源类型
                lock_info.protected_resources.iter().any(|r| {
                    path.operations.iter().any(|op| {
                        self.get_resource_type(&op.resource) == self.get_resource_type(r)
                    })
                })
            }
            LockGranularity::Coarse => {
                // 粗粒度锁，默认保护
                true
            }
        }
    }

    /// 检查时序依赖
    fn has_timing_dependency(&self, path: &ExecutionPath) -> bool {
        // 检查操作之间是否有时间敏感的依赖
        path.operations.windows(2).any(|ops| {
            ops[0].timestamp + self.config.time_window_ms >= ops[1].timestamp
        })
    }

    /// 检查是否在时间窗口内
    fn is_within_time_window(&self, path: &ExecutionPath) -> bool {
        if path.operations.len() < 2 {
            return false;
        }
        
        let first_time = path.operations[0].timestamp;
        let last_time = path.operations[path.operations.len() - 1].timestamp;
        
        (last_time - first_time) <= self.config.time_window_ms
    }

    /// 检查并发竞争
    fn has_concurrency_race(&self, path: &ExecutionPath) -> bool {
        // 检查是否有未保护的共享资源访问
        path.operations.iter().any(|op| {
            op.is_shared && op.lock_id.is_none()
        })
    }

    /// 检查外部依赖
    fn has_external_dependency(&self, path: &ExecutionPath) -> bool {
        // 检查是否有外部系统调用
        path.operations.iter().any(|op| {
            op.operation_type == OperationType::ExternalCall
        })
    }

    /// 检查资源竞争
    fn has_resource_contention(&self, path: &ExecutionPath) -> bool {
        // 检查是否有多个操作竞争同一资源
        let mut resource_access: HashMap<String, usize> = HashMap::new();
        
        for op in &path.operations {
            *resource_access.entry(op.resource.clone()).or_insert(0) += 1;
        }
        
        resource_access.values().any(|&count| count > 1)
    }

    /// 创建扫描结果
    fn create_scan_result(&self, path: &ExecutionPath, path_type: PathType, from_cache: bool) -> ScanResult {
        let is_deterministic = path_type == PathType::Deterministic;
        
        // 计算置信度
        let confidence = if from_cache {
            0.95 // 缓存结果置信度高
        } else {
            self.calculate_confidence(&path_type, path)
        };
        
        // 计算误报风险
        let false_positive_risk = self.calculate_false_positive_risk(&path_type);
        
        // 生成优化建议
        let suggestions = self.generate_optimization_suggestions(&path_type, path);
        
        // 更新统计
        if !is_deterministic {
            if false_positive_risk < self.config.false_positive_target {
                self.true_positives += 1;
            } else {
                self.false_positives += 1;
            }
        }
        
        ScanResult {
            path_id: path.id.clone(),
            path_type,
            is_deterministic,
            confidence,
            false_positive_risk,
            optimization_suggestions: suggestions,
        }
    }

    /// 计算置信度
    fn calculate_confidence(&self, path_type: &PathType, path: &ExecutionPath) -> f64 {
        match path_type {
            PathType::Deterministic => 0.99,
            PathType::NonDeterministicConcurrency => {
                // 并发竞争的置信度取决于锁保护情况
                if path.lock_info.is_some() {
                    0.8
                } else {
                    0.95
                }
            }
            PathType::NonDeterministicTiming => {
                // 时序依赖的置信度取决于时间窗口
                let time_span = path.operations.last().unwrap().timestamp - path.operations.first().unwrap().timestamp;
                if time_span < self.config.time_window_ms / 2 {
                    0.9
                } else {
                    0.7
                }
            }
            PathType::NonDeterministicExternal => 0.85,
            PathType::NonDeterministicResource => 0.8,
        }
    }

    /// 计算误报风险
    fn calculate_false_positive_risk(&self, path_type: &PathType) -> f64 {
        match path_type {
            PathType::Deterministic => 0.0,
            PathType::NonDeterministicConcurrency => 0.15,
            PathType::NonDeterministicTiming => 0.25, // 时序依赖误报风险较高
            PathType::NonDeterministicExternal => 0.1,
            PathType::NonDeterministicResource => 0.2,
        }
    }

    /// 生成优化建议
    fn generate_optimization_suggestions(&self, path_type: &PathType, path: &ExecutionPath) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        match path_type {
            PathType::NonDeterministicConcurrency => {
                suggestions.push("Consider adding lock protection for shared resource access".to_string());
                suggestions.push("Use atomic operations if applicable".to_string());
            }
            PathType::NonDeterministicTiming => {
                suggestions.push(format!("Increase time window from {}ms to {}ms", 
                    self.config.time_window_ms, 
                    self.config.time_window_ms * 2));
                suggestions.push("Add explicit synchronization if ordering is required".to_string());
            }
            PathType::NonDeterministicExternal => {
                suggestions.push("Add retry logic for external calls".to_string());
                suggestions.push("Implement circuit breaker pattern".to_string());
            }
            PathType::NonDeterministicResource => {
                suggestions.push("Use resource pooling to reduce contention".to_string());
                suggestions.push("Consider using read-write locks".to_string());
            }
            _ => {}
        }
        
        suggestions
    }

    /// 获取资源类型
    fn get_resource_type(&self, resource: &str) -> String {
        // 简化实现：提取资源类型前缀
        resource.split(':').next().unwrap_or(resource).to_string()
    }

    /// 获取当前误报率
    pub fn get_false_positive_rate(&self) -> f64 {
        if self.total_detections == 0 {
            0.0
        } else {
            let non_deterministic = self.false_positives + self.true_positives;
            if non_deterministic == 0 {
                0.0
            } else {
                self.false_positives as f64 / non_deterministic as f64
            }
        }
    }

    /// 获取优化器统计信息
    pub fn get_stats(&self) -> ScannerOptimizerStats {
        ScannerOptimizerStats {
            total_detections: self.total_detections,
            true_positives: self.true_positives,
            false_positives: self.false_positives,
            false_positive_rate: self.get_false_positive_rate(),
            cache_size: self.path_type_cache.len(),
            target_false_positive_rate: self.config.false_positive_target,
        }
    }

    /// 自适应优化配置
    pub fn adaptive_optimize(&mut self) {
        if !self.config.adaptive_optimization {
            return;
        }
        
        let current_fp_rate = self.get_false_positive_rate();
        
        if current_fp_rate > self.config.false_positive_target {
            // 误报率过高，调整配置
            warn!("False positive rate {} exceeds target {}, adjusting config...", 
                current_fp_rate, self.config.false_positive_target);
            
            // 增加时间窗口
            self.config.time_window_ms = (self.config.time_window_ms as f64 * 1.2) as u64;
            
            // 降低锁检测灵敏度
            self.config.lock_detection_sensitivity *= 0.9;
            
            info!("Adjusted config: time_window={}ms, sensitivity={}", 
                self.config.time_window_ms, self.config.lock_detection_sensitivity);
        }
    }

    /// 清除缓存
    pub fn clear_cache(&mut self) {
        self.path_type_cache.clear();
        info!("Scanner optimizer cache cleared");
    }
}

/// 执行路径
#[derive(Debug, Clone)]
pub struct ExecutionPath {
    /// 路径 ID
    pub id: String,
    /// 操作列表
    pub operations: Vec<Operation>,
    /// 锁信息
    pub lock_info: Option<LockInfo>,
}

/// 操作
#[derive(Debug, Clone)]
pub struct Operation {
    /// 操作 ID
    pub id: String,
    /// 操作类型
    pub operation_type: OperationType,
    /// 资源
    pub resource: String,
    /// 时间戳 (毫秒)
    pub timestamp: u64,
    /// 是否共享资源
    pub is_shared: bool,
    /// 锁 ID
    pub lock_id: Option<String>,
}

/// 操作类型
#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    /// 读取
    Read,
    /// 写入
    Write,
    /// 外部调用
    ExternalCall,
    /// 其他
    Other,
}

/// 扫描器优化器统计信息
#[derive(Debug, Clone)]
pub struct ScannerOptimizerStats {
    /// 总检测数
    pub total_detections: u64,
    /// 真阳性数
    pub true_positives: u64,
    /// 误报数
    pub false_positives: u64,
    /// 误报率
    pub false_positive_rate: f64,
    /// 缓存大小
    pub cache_size: usize,
    /// 目标误报率
    pub target_false_positive_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_optimizer_creation() {
        let config = ScannerOptimizerConfig::default();
        let optimizer = ScannerOptimizer::new(config);
        
        assert_eq!(optimizer.false_positives, 0);
        assert_eq!(optimizer.true_positives, 0);
        assert_eq!(optimizer.total_detections, 0);
    }

    #[test]
    fn test_deterministic_path_detection() {
        let config = ScannerOptimizerConfig::default();
        let mut optimizer = ScannerOptimizer::new(config);
        
        let path = ExecutionPath {
            id: "path_1".to_string(),
            operations: vec![
                Operation {
                    id: "op_1".to_string(),
                    operation_type: OperationType::Read,
                    resource: "resource:1".to_string(),
                    timestamp: 0,
                    is_shared: false,
                    lock_id: None,
                },
            ],
            lock_info: None,
        };
        
        let result = optimizer.scan_path(&path);
        
        assert!(result.is_deterministic);
        assert_eq!(result.path_type, PathType::Deterministic);
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn test_timing_dependency_detection() {
        let config = ScannerOptimizerConfig {
            time_window_ms: 50,
            ..Default::default()
        };
        let mut optimizer = ScannerOptimizer::new(config);
        
        let path = ExecutionPath {
            id: "path_1".to_string(),
            operations: vec![
                Operation {
                    id: "op_1".to_string(),
                    operation_type: OperationType::Read,
                    resource: "resource:1".to_string(),
                    timestamp: 0,
                    is_shared: true,
                    lock_id: None,
                },
                Operation {
                    id: "op_2".to_string(),
                    operation_type: OperationType::Write,
                    resource: "resource:1".to_string(),
                    timestamp: 10, // 在时间窗口内
                    is_shared: true,
                    lock_id: None,
                },
            ],
            lock_info: None,
        };
        
        let result = optimizer.scan_path(&path);
        
        assert!(!result.is_deterministic);
        assert_eq!(result.path_type, PathType::NonDeterministicTiming);
    }

    #[test]
    fn test_false_positive_rate() {
        let config = ScannerOptimizerConfig::default();
        let mut optimizer = ScannerOptimizer::new(config);
        
        // 模拟一些检测
        for i in 0..100 {
            let path = ExecutionPath {
                id: format!("path_{}", i),
                operations: vec![],
                lock_info: None,
            };
            optimizer.scan_path(&path);
        }
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.total_detections, 100);
    }

    #[test]
    fn test_adaptive_optimization() {
        let config = ScannerOptimizerConfig {
            false_positive_target: 0.02,
            adaptive_optimization: true,
            ..Default::default()
        };
        let mut optimizer = ScannerOptimizer::new(config);
        
        // 模拟高误报率
        optimizer.false_positives = 50;
        optimizer.true_positives = 50;
        optimizer.total_detections = 100;
        
        let initial_time_window = optimizer.config.time_window_ms;
        
        // 执行自适应优化
        optimizer.adaptive_optimize();
        
        // 时间窗口应该增加
        assert!(optimizer.config.time_window_ms > initial_time_window);
    }
}
