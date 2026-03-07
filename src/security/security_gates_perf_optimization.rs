// security_gates_perf_optimization.rs
// Phase 3 Week 4 Security - 安全闸门性能优化
// SG-1~SG-4 并行验证 + 策略缓存优化

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;
use anyhow::{Result, anyhow, bail};

// ============================================================================
// 安全闸门定义
// ============================================================================

/// 安全闸门类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SecurityGateType {
    SG1_Authentication,  // 身份认证闸门
    SG2_Authorization,   // 权限验证闸门
    SG3_RateLimiting,    // 限流闸门
    SG4_AuditLogging,    // 审计日志闸门
}

/// 闸门验证结果
#[derive(Debug, Clone)]
pub struct GateVerificationResult {
    pub gate_type: SecurityGateType,
    pub passed: bool,
    pub latency_ms: u64,
    pub error_message: Option<String>,
}

/// 闸门验证请求上下文
#[derive(Debug, Clone)]
pub struct GateContext {
    pub request_id: String,
    pub user_id: String,
    pub user_roles: Vec<String>,
    pub resource_id: String,
    pub action: String,
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// 并行安全闸门验证器
// ============================================================================

/// 并行安全闸门验证器 - SG-1~SG-4 并行执行
pub struct ParallelSecurityGateVerifier {
    // 闸门配置
    gates: Arc<RwLock<HashMap<SecurityGateType, GateConfig>>>,
    // 验证超时
    verification_timeout: Duration,
    // 并发度控制
    max_concurrent_verifications: usize,
    // 统计信息
    stats: Arc<RwLock<GateStats>>,
}

#[derive(Debug, Clone)]
struct GateConfig {
    enabled: bool,
    timeout_ms: u64,
    retry_count: u32,
    failure_mode: FailureMode,  // 失败时的行为：FailOpen 或 FailClosed
}

#[derive(Debug, Clone, PartialEq)]
enum FailureMode {
    FailOpen,   // 失败时放行 (适用于非关键闸门)
    FailClosed, // 失败时拒绝 (适用于关键闸门)
}

#[derive(Debug, Default)]
struct GateStats {
    total_verifications: u64,
    successful_verifications: u64,
    failed_verifications: u64,
    timeout_verifications: u64,
    p50_latency_ms: f64,
    p95_latency_ms: f64,
    p99_latency_ms: f64,
}

impl ParallelSecurityGateVerifier {
    pub fn new(verification_timeout_ms: u64, max_concurrent: usize) -> Self {
        Self {
            gates: Arc::new(RwLock::new(Self::default_gate_configs())),
            verification_timeout: Duration::from_millis(verification_timeout_ms),
            max_concurrent_verifications: max_concurrent,
            stats: Arc::new(RwLock::new(GateStats::default())),
        }
    }

    /// 默认闸门配置
    fn default_gate_configs() -> HashMap<SecurityGateType, GateConfig> {
        let mut configs = HashMap::new();
        
        // SG-1: 身份认证 - 关键闸门，失败时拒绝
        configs.insert(SecurityGateType::SG1_Authentication, GateConfig {
            enabled: true,
            timeout_ms: 100,
            retry_count: 2,
            failure_mode: FailureMode::FailClosed,
        });

        // SG-2: 权限验证 - 关键闸门，失败时拒绝
        configs.insert(SecurityGateType::SG2_Authorization, GateConfig {
            enabled: true,
            timeout_ms: 150,
            retry_count: 2,
            failure_mode: FailureMode::FailClosed,
        });

        // SG-3: 限流 - 非关键闸门，失败时放行
        configs.insert(SecurityGateType::SG3_RateLimiting, GateConfig {
            enabled: true,
            timeout_ms: 50,
            retry_count: 1,
            failure_mode: FailureMode::FailOpen,
        });

        // SG-4: 审计日志 - 非关键闸门，失败时放行
        configs.insert(SecurityGateType::SG4_AuditLogging, GateConfig {
            enabled: true,
            timeout_ms: 200,
            retry_count: 1,
            failure_mode: FailureMode::FailOpen,
        });

        configs
    }

    /// 并行验证所有闸门
    pub async fn verify_all_gates(&self, ctx: &GateContext) -> Result<VerificationSummary> {
        let start = Instant::now();
        
        // 获取启用的闸门
        let enabled_gates = {
            let gates = self.gates.read().await;
            gates.iter()
                .filter(|(_, config)| config.enabled)
                .map(|(gate_type, _)| gate_type.clone())
                .collect::<Vec<_>>()
        };

        // 创建信号量控制并发度
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_verifications));
        
        // 并行执行所有闸门验证
        let mut tasks = Vec::new();
        for gate_type in enabled_gates {
            let semaphore = semaphore.clone();
            let ctx = ctx.clone();
            let gate_config = self.get_gate_config(&gate_type).await?;
            
            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                Self::verify_single_gate(gate_type, gate_config, ctx).await
            });
            
            tasks.push(task);
        }

        // 收集所有结果
        let mut results = Vec::new();
        for task in tasks {
            match timeout(Duration::from_secs(5), task).await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    log::error!("Gate verification task failed: {}", e);
                    results.push(GateVerificationResult {
                        gate_type: SecurityGateType::SG1_Authentication, // Placeholder
                        passed: false,
                        latency_ms: 0,
                        error_message: Some(e.to_string()),
                    });
                }
                Err(_) => {
                    log::error!("Gate verification task timeout");
                    results.push(GateVerificationResult {
                        gate_type: SecurityGateType::SG1_Authentication, // Placeholder
                        passed: false,
                        latency_ms: 5000,
                        error_message: Some("Timeout".to_string()),
                    });
                }
            }
        }

        // 更新统计
        self.update_stats(&results, start.elapsed()).await;

        // 生成汇总报告
        let summary = VerificationSummary {
            request_id: ctx.request_id.clone(),
            total_gates: results.len(),
            passed_gates: results.iter().filter(|r| r.passed).count(),
            failed_gates: results.iter().filter(|r| !r.passed).count(),
            total_latency_ms: start.elapsed().as_millis() as u64,
            results,
            all_passed: results.iter().all(|r| r.passed),
        };

        Ok(summary)
    }

    /// 验证单个闸门
    async fn verify_single_gate(
        gate_type: SecurityGateType,
        config: GateConfig,
        ctx: GateContext,
    ) -> GateVerificationResult {
        let start = Instant::now();
        
        // 带重试的验证
        let mut last_error = None;
        for attempt in 0..=config.retry_count {
            match timeout(
                Duration::from_millis(config.timeout_ms),
                Self::execute_gate_verification(&gate_type, &ctx)
            ).await {
                Ok(Ok(passed)) => {
                    return GateVerificationResult {
                        gate_type,
                        passed,
                        latency_ms: start.elapsed().as_millis() as u64,
                        error_message: None,
                    };
                }
                Ok(Err(e)) => {
                    last_error = Some(e.to_string());
                    if attempt < config.retry_count {
                        tokio::time::sleep(Duration::from_millis(10 * (attempt as u64 + 1))).await;
                    }
                }
                Err(_) => {
                    last_error = Some(format!("Timeout after {}ms", config.timeout_ms));
                }
            }
        }

        // 所有重试失败，根据 failure_mode 决定结果
        let passed = match config.failure_mode {
            FailureMode::FailOpen => true,
            FailureMode::FailClosed => false,
        };

        GateVerificationResult {
            gate_type,
            passed,
            latency_ms: start.elapsed().as_millis() as u64,
            error_message: last_error,
        }
    }

    /// 执行具体的闸门验证逻辑
    async fn execute_gate_verification(gate_type: &SecurityGateType, ctx: &GateContext) -> Result<bool> {
        match gate_type {
            SecurityGateType::SG1_Authentication => {
                // SG-1: 身份认证验证
                // 实际实现中会验证 JWT/OIDC Token
                Ok(!ctx.user_id.is_empty())
            }
            SecurityGateType::SG2_Authorization => {
                // SG-2: 权限验证
                // 实际实现中会查询 OPA 或权限服务
                Ok(!ctx.user_roles.is_empty() && !ctx.resource_id.is_empty())
            }
            SecurityGateType::SG3_RateLimiting => {
                // SG-3: 限流检查
                // 实际实现中会检查 Redis 中的计数
                Ok(true) // 简化实现
            }
            SecurityGateType::SG4_AuditLogging => {
                // SG-4: 审计日志记录
                // 实际实现中会异步写入审计日志
                log::info!("Audit log: user={} action={} resource={}", 
                    ctx.user_id, ctx.action, ctx.resource_id);
                Ok(true)
            }
        }
    }

    async fn get_gate_config(&self, gate_type: &SecurityGateType) -> Result<GateConfig> {
        let gates = self.gates.read().await;
        gates.get(gate_type)
            .cloned()
            .ok_or_else(|| anyhow!("Gate config not found: {:?}", gate_type))
    }

    async fn update_stats(&self, results: &[GateVerificationResult], total_duration: Duration) {
        let mut stats = self.stats.write().await;
        stats.total_verifications += 1;
        
        let all_passed = results.iter().all(|r| r.passed);
        if all_passed {
            stats.successful_verifications += 1;
        } else {
            stats.failed_verifications += 1;
        }

        let timeout_count = results.iter().filter(|r| 
            r.error_message.as_ref().map_or(false, |m| m.contains("Timeout"))
        ).count();
        stats.timeout_verifications += timeout_count as u64;

        // 更新延迟百分位数 (简化实现)
        let latency = total_duration.as_millis() as f64;
        stats.p50_latency_ms = latency * 0.5;
        stats.p95_latency_ms = latency * 0.95;
        stats.p99_latency_ms = latency;
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> GateStats {
        self.stats.read().await.clone()
    }

    /// 动态启用/禁用闸门
    pub async fn toggle_gate(&self, gate_type: SecurityGateType, enabled: bool) -> Result<()> {
        let mut gates = self.gates.write().await;
        if let Some(config) = gates.get_mut(&gate_type) {
            config.enabled = enabled;
            log::info!("Gate {:?} {}", gate_type, if enabled { "enabled" } else { "disabled" });
            Ok(())
        } else {
            bail!("Gate not found: {:?}", gate_type)
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerificationSummary {
    pub request_id: String,
    pub total_gates: usize,
    pub passed_gates: usize,
    pub failed_gates: usize,
    pub total_latency_ms: u64,
    pub results: Vec<GateVerificationResult>,
    pub all_passed: bool,
}

// ============================================================================
// 策略缓存优化器
// ============================================================================

/// 策略缓存优化器 - 减少重复策略评估
pub struct PolicyCacheOptimizer {
    // 策略缓存：key -> (value, expires_at)
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    // 缓存 TTL
    default_ttl: Duration,
    // 最大缓存条目
    max_entries: usize,
    // 缓存命中统计
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: PolicyResult,
    expires_at: Instant,
    access_count: u64,
    last_accessed: Instant,
}

#[derive(Debug, Clone)]
struct PolicyResult {
    allowed: bool,
    reason: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Default)]
struct CacheStats {
    hits: u64,
    misses: u64,
    evictions: u64,
    expirations: u64,
    hit_rate: f64,
}

impl PolicyCacheOptimizer {
    pub fn new(ttl_secs: u64, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Duration::from_secs(ttl_secs),
            max_entries,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 生成缓存键
    fn generate_cache_key(user_id: &str, resource_id: &str, action: &str, roles: &[String]) -> String {
        format!("{}:{}:{}:{:?}", user_id, resource_id, action, roles)
    }

    /// 查询缓存 (带预取优化)
    pub async fn get_or_evaluate<F, Fut>(&self, ctx: &GateContext, evaluator: F) -> Result<PolicyResult>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<PolicyResult>>,
    {
        let cache_key = Self::generate_cache_key(
            &ctx.user_id,
            &ctx.resource_id,
            &ctx.action,
            &ctx.user_roles,
        );

        // 尝试从缓存获取
        if let Some(entry) = self.get_from_cache(&cache_key).await {
            return Ok(entry);
        }

        // 缓存未命中，执行评估
        let result = evaluator().await?;

        // 写入缓存
        self.put_to_cache(&cache_key, result.clone()).await;

        Ok(result)
    }

    /// 从缓存获取 (带过期检查和预取)
    async fn get_from_cache(&self, key: &str) -> Option<PolicyResult> {
        let mut cache = self.cache.write().await;
        
        if let Some(entry) = cache.get_mut(key) {
            // 检查是否过期
            if Instant::now() > entry.expires_at {
                // 过期条目，标记删除
                let mut stats = self.stats.write().await;
                stats.expirations += 1;
                return None;
            }

            // 更新访问信息
            entry.access_count += 1;
            entry.last_accessed = Instant::now();

            // 更新统计
            {
                let mut stats = self.stats.write().await;
                stats.hits += 1;
                stats.hit_rate = stats.hits as f64 / (stats.hits + stats.misses) as f64 * 100.0;
            }

            return Some(entry.value.clone());
        }

        // 缓存未命中
        {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
            stats.hit_rate = stats.hits as f64 / (stats.hits + stats.misses) as f64 * 100.0;
        }

        None
    }

    /// 写入缓存 (带 LRU 淘汰)
    async fn put_to_cache(&self, key: &str, value: PolicyResult) {
        let mut cache = self.cache.write().await;

        // 如果缓存已满，淘汰最少使用的条目
        if cache.len() >= self.max_entries && !cache.contains_key(key) {
            self.evict_lru(&mut cache).await;
        }

        cache.insert(key.to_string(), CacheEntry {
            value,
            expires_at: Instant::now() + self.default_ttl,
            access_count: 1,
            last_accessed: Instant::now(),
        });
    }

    /// LRU 淘汰
    async fn evict_lru(&self, cache: &mut HashMap<String, CacheEntry>) {
        let mut stats = self.stats.write().await;
        
        // 找到最少使用的条目
        let lru_key = cache.iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(key, _)| key.clone());

        if let Some(key) = lru_key {
            cache.remove(&key);
            stats.evictions += 1;
            log::debug!("Cache eviction: {}", key);
        }
    }

    /// 预取热点策略到缓存
    pub async fn prefetch_hot_policies(&self, hot_keys: Vec<(String, PolicyResult)>) {
        let mut cache = self.cache.write().await;
        
        for (key, value) in hot_keys {
            if cache.len() < self.max_entries {
                cache.insert(key, CacheEntry {
                    value,
                    expires_at: Instant::now() + self.default_ttl,
                    access_count: 0,
                    last_accessed: Instant::now(),
                });
            }
        }

        log::info!("Prefetched {} hot policies", hot_keys.len());
    }

    /// 批量失效缓存
    pub async fn invalidate_patterns(&self, patterns: Vec<String>) {
        let mut cache = self.cache.write().await;
        let mut removed_count = 0;

        for pattern in patterns {
            let keys_to_remove: Vec<String> = cache.keys()
                .filter(|k| k.contains(&pattern))
                .cloned()
                .collect();

            for key in keys_to_remove {
                cache.remove(&key);
                removed_count += 1;
            }
        }

        log::info!("Invalidated {} cache entries", removed_count);
    }

    /// 获取缓存统计
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// 清空缓存
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        log::info!("Cache cleared");
    }
}

// ============================================================================
// 性能监控与优化建议
// ============================================================================

/// 性能监控器
pub struct GatePerformanceMonitor {
    // 延迟直方图
    latency_histogram: Arc<RwLock<Vec<u64>>>,
    // 性能阈值配置
    thresholds: PerformanceThresholds,
}

#[derive(Debug, Clone)]
struct PerformanceThresholds {
    p50_target_ms: u64,
    p95_target_ms: u64,
    p99_target_ms: u64,
    throughput_target_qps: u64,
}

impl GatePerformanceMonitor {
    pub fn new() -> Self {
        Self {
            latency_histogram: Arc::new(RwLock::new(Vec::new())),
            thresholds: PerformanceThresholds {
                p50_target_ms: 50,
                p95_target_ms: 100,
                p99_target_ms: 200,
                throughput_target_qps: 10000,
            },
        }
    }

    /// 记录延迟样本
    pub async fn record_latency(&self, latency_ms: u64) {
        let mut histogram = self.latency_histogram.write().await;
        histogram.push(latency_ms);
        
        // 保持最近 10000 个样本
        if histogram.len() > 10000 {
            histogram.remove(0);
        }
    }

    /// 计算延迟百分位数
    pub async fn calculate_percentiles(&self) -> (f64, f64, f64) {
        let histogram = self.latency_histogram.read().await;
        if histogram.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let mut sorted = histogram.clone();
        sorted.sort();

        let len = sorted.len();
        let p50 = sorted[len * 50 / 100] as f64;
        let p95 = sorted[len * 95 / 100] as f64;
        let p99 = sorted[len * 99 / 100] as f64;

        (p50, p95, p99)
    }

    /// 生成优化建议
    pub async fn generate_optimization_suggestions(&self, current_stats: &GateStats) -> Vec<String> {
        let mut suggestions = Vec::new();
        let (p50, p95, p99) = self.calculate_percentiles().await;

        if p50 > self.thresholds.p50_target_ms as f64 {
            suggestions.push(format!(
                "P50 延迟 ({:.1}ms) 超过目标 ({}ms)，考虑优化热点路径",
                p50, self.thresholds.p50_target_ms
            ));
        }

        if p95 > self.thresholds.p95_target_ms as f64 {
            suggestions.push(format!(
                "P95 延迟 ({:.1}ms) 超过目标 ({}ms)，考虑增加缓存命中率",
                p95, self.thresholds.p95_target_ms
            ));
        }

        if p99 > self.thresholds.p99_target_ms as f64 {
            suggestions.push(format!(
                "P99 延迟 ({:.1}ms) 超过目标 ({}ms)，考虑优化慢查询或增加超时控制",
                p99, self.thresholds.p99_target_ms
            ));
        }

        let cache_hit_rate = current_stats.successful_verifications as f64 
            / (current_stats.total_verifications as f64 + 0.001) * 100.0;
        
        if cache_hit_rate < 95.0 {
            suggestions.push(format!(
                "闸门通过率 ({:.1}%) 较低，检查是否有异常请求模式",
                cache_hit_rate
            ));
        }

        if suggestions.is_empty() {
            suggestions.push("性能指标正常，无需优化".to_string());
        }

        suggestions
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_gate_verifier() {
        let verifier = ParallelSecurityGateVerifier::new(500, 10);
        
        let ctx = GateContext {
            request_id: "req-001".to_string(),
            user_id: "user-123".to_string(),
            user_roles: vec!["admin".to_string()],
            resource_id: "resource-456".to_string(),
            action: "read".to_string(),
            metadata: HashMap::new(),
        };

        let summary = verifier.verify_all_gates(&ctx).await.unwrap();
        
        assert!(summary.all_passed);
        assert_eq!(summary.total_gates, 4);
        assert_eq!(summary.passed_gates, 4);
        assert!(summary.total_latency_ms < 1000);
    }

    #[tokio::test]
    async fn test_policy_cache_optimizer() {
        let cache = PolicyCacheOptimizer::new(60, 1000);
        
        let ctx = GateContext {
            request_id: "req-002".to_string(),
            user_id: "user-456".to_string(),
            user_roles: vec!["viewer".to_string()],
            resource_id: "resource-789".to_string(),
            action: "read".to_string(),
            metadata: HashMap::new(),
        };

        // 第一次访问 (缓存未命中)
        let result1 = cache.get_or_evaluate(&ctx, || async {
            Ok(PolicyResult {
                allowed: true,
                reason: "Test evaluation".to_string(),
                metadata: HashMap::new(),
            })
        }).await.unwrap();

        // 第二次访问 (缓存命中)
        let result2 = cache.get_or_evaluate(&ctx, || async {
            // 这个闭包不应该被执行
            panic!("Should not be called");
        }).await.unwrap();

        assert!(result1.allowed);
        assert!(result2.allowed);

        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!(stats.hit_rate > 0.0);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = PolicyCacheOptimizer::new(60, 1000);
        
        // 预填充缓存
        for i in 0..10 {
            let ctx = GateContext {
                request_id: format!("req-{}", i),
                user_id: format!("user-{}", i),
                user_roles: vec!["admin".to_string()],
                resource_id: "resource-common".to_string(),
                action: "read".to_string(),
                metadata: HashMap::new(),
            };
            
            cache.get_or_evaluate(&ctx, || async {
                Ok(PolicyResult {
                    allowed: true,
                    reason: "Test".to_string(),
                    metadata: HashMap::new(),
                })
            }).await.unwrap();
        }

        // 批量失效
        cache.invalidate_patterns(vec!["resource-common".to_string()]).await;

        let stats = cache.get_stats().await;
        assert!(stats.evictions > 0);
    }
}
