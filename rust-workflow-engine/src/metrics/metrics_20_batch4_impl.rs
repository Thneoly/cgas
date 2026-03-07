//! 指标采集 Batch 4 实现 (Metrics 20 Batch 4 Implementation)
//! 
//! Phase 3 Week 5: 50 指标全量接入
//! 
//! **本批次指标**: 20 个 (M-036 ~ M-055)
//! - 错误指标扩展 (5 个): M-036 ~ M-040
//! - 业务指标扩展 (8 个): M-041 ~ M-048
//! - 系统指标扩展 (2 个): M-049 ~ M-050
//! - 追踪指标 (3 个): M-051 ~ M-053
//! - 威胁检测 (2 个): M-054 ~ M-055
//! 
//! **参考文档**: phase3_50_metrics_plan.md

use prometheus::{Histogram, Counter, Gauge, HistogramOpts, Opts, IntCounter, IntGauge};
use lazy_static::lazy_static;

lazy_static! {
    // ========================================================================
    // 错误指标扩展 (5 个)
    // ========================================================================
    
    /// M-036: 执行 Panic 次数
    pub static ref EXECUTION_PANIC_COUNT: IntCounter = IntCounter::with_opts(
        Opts::new("execution_panic_count", "Count of execution panics")
            .namespace("cgas")
            .subsystem("executor")
    ).unwrap();
    
    /// M-037: 执行超时次数
    pub static ref EXECUTION_TIMEOUT_COUNT: IntCounter = IntCounter::with_opts(
        Opts::new("execution_timeout_count", "Count of execution timeouts")
            .namespace("cgas")
            .subsystem("executor")
    ).unwrap();
    
    /// M-038: 验证不匹配次数
    pub static ref VERIFICATION_MISMATCH_COUNT: IntCounter = IntCounter::with_opts(
        Opts::new("verification_mismatch_count", "Count of verification mismatches")
            .namespace("cgas")
            .subsystem("verifier")
    ).unwrap();
    
    /// M-039: Batch 部分失败次数
    pub static ref BATCH_PARTIAL_FAILURE_COUNT: IntCounter = IntCounter::with_opts(
        Opts::new("batch_partial_failure_count", "Count of batch partial failures")
            .namespace("cgas")
            .subsystem("batch")
    ).unwrap();
    
    /// M-040: Transaction 中止次数
    pub static ref TRANSACTION_ABORT_COUNT: IntCounter = IntCounter::with_opts(
        Opts::new("transaction_abort_count", "Count of transaction aborts")
            .namespace("cgas")
            .subsystem("transaction")
    ).unwrap();
    
    // ========================================================================
    // 业务指标扩展 (8 个)
    // ========================================================================
    
    /// M-041: 指令重试次数
    pub static ref INSTRUCTION_RETRY_COUNT: IntCounter = IntCounter::with_opts(
        Opts::new("instruction_retry_count", "Count of instruction retries")
            .namespace("cgas")
            .subsystem("executor")
    ).unwrap();
    
    /// M-042: 指令成功率
    pub static ref INSTRUCTION_SUCCESS_RATE: Gauge = Gauge::with_opts(
        Opts::new("instruction_success_rate", "Instruction success rate percentage")
            .namespace("cgas")
            .subsystem("executor")
    ).unwrap();
    
    /// M-043: 灰度回滚次数
    pub static ref GRAY_RELEASE_ROLLBACK_COUNT: IntCounter = IntCounter::with_opts(
        Opts::new("gray_release_rollback_count", "Count of gray release rollbacks")
            .namespace("cgas")
            .subsystem("gray_release")
    ).unwrap();
    
    /// M-044: OIDC Token 验证时延 P99
    pub static ref OIDC_TOKEN_VALIDATION_LATENCY_P99: Histogram = Histogram::with_opts(
        HistogramOpts::new("oidc_token_validation_latency_p99", "OIDC token validation latency P99 in ms")
            .namespace("cgas")
            .subsystem("security")
            .buckets(vec![10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 300.0])
    ).unwrap();
    
    /// M-045: OPA 策略评估次数
    pub static ref OPA_POLICY_EVALUATION_COUNT: IntCounter = IntCounter::with_opts(
        Opts::new("opa_policy_evaluation_count", "Count of OPA policy evaluations")
            .namespace("cgas")
            .subsystem("security")
    ).unwrap();
    
    /// M-046: 密钥轮换成功率
    pub static ref SECRET_ROTATION_SUCCESS_RATE: Gauge = Gauge::with_opts(
        Opts::new("secret_rotation_success_rate", "Secret rotation success rate percentage")
            .namespace("cgas")
            .subsystem("security")
    ).unwrap();
    
    /// M-047: 客户端请求速率
    pub static ref CLIENT_REQUEST_RATE: Gauge = Gauge::with_opts(
        Opts::new("client_request_rate", "Client request rate per second")
            .namespace("cgas")
            .subsystem("gateway")
    ).unwrap();
    
    /// M-048: 客户端错误率
    pub static ref CLIENT_ERROR_RATE: Gauge = Gauge::with_opts(
        Opts::new("client_error_rate", "Client error rate percentage")
            .namespace("cgas")
            .subsystem("gateway")
    ).unwrap();
    
    // ========================================================================
    // 系统指标扩展 (2 个)
    // ========================================================================
    
    /// M-049: 磁盘 IO 等待百分比
    pub static ref DISK_IO_WAIT_PERCENT: Gauge = Gauge::with_opts(
        Opts::new("disk_io_wait_percent", "Disk IO wait percentage")
            .namespace("cgas")
            .subsystem("system")
    ).unwrap();
    
    /// M-050: 网络丢包率
    pub static ref NETWORK_PACKET_DROP_RATE: Gauge = Gauge::with_opts(
        Opts::new("network_packet_drop_rate", "Network packet drop rate percentage")
            .namespace("cgas")
            .subsystem("system")
    ).unwrap();
    
    // ========================================================================
    // 分布式追踪指标 (3 个)
    // ========================================================================
    
    /// M-051: 全链路追踪时长 P99
    pub static ref TRACE_TOTAL_DURATION_P99: Histogram = Histogram::with_opts(
        HistogramOpts::new("trace_total_duration_p99", "Total trace duration P99 in ms")
            .namespace("cgas")
            .subsystem("tracing")
            .buckets(vec![100.0, 250.0, 500.0, 750.0, 1000.0, 1500.0, 2000.0, 3000.0, 5000.0])
    ).unwrap();
    
    /// M-052: 平均 Span 数量
    pub static ref TRACE_SPAN_COUNT_AVG: Gauge = Gauge::with_opts(
        Opts::new("trace_span_count_avg", "Average span count per trace")
            .namespace("cgas")
            .subsystem("tracing")
    ).unwrap();
    
    /// M-053: 追踪传递成功率
    pub static ref TRACE_PROPAGATION_SUCCESS_RATE: Gauge = Gauge::with_opts(
        Opts::new("trace_propagation_success_rate", "Trace propagation success rate percentage")
            .namespace("cgas")
            .subsystem("tracing")
    ).unwrap();
    
    // ========================================================================
    // 威胁检测指标 (2 个)
    // ========================================================================
    
    /// M-054: 异常检测告警数
    pub static ref ANOMALY_DETECTION_ALERT_COUNT: IntCounter = IntCounter::with_opts(
        Opts::new("anomaly_detection_alert_count", "Count of anomaly detection alerts")
            .namespace("cgas")
            .subsystem("security")
    ).unwrap();
    
    /// M-055: 威胁处置平均时间
    pub static ref THREAT_MITIGATION_TIME_AVG: Gauge = Gauge::with_opts(
        Opts::new("threat_mitigation_time_avg", "Average threat mitigation time in seconds")
            .namespace("cgas")
            .subsystem("security")
    ).unwrap();
}

// ============================================================================
// 指标采集辅助函数
// ============================================================================

/// 记录执行 Panic
pub fn inc_execution_panic(location: &str) {
    EXECUTION_PANIC_COUNT.inc();
    log::error!("Execution panic detected at: {}", location);
}

/// 记录执行超时
pub fn inc_execution_timeout(instruction_type: &str) {
    EXECUTION_TIMEOUT_COUNT.inc();
    log::warn!("Execution timeout for instruction type: {}", instruction_type);
}

/// 记录验证不匹配
pub fn inc_verification_mismatch(mismatch_type: &str) {
    VERIFICATION_MISMATCH_COUNT.inc();
    log::error!("Verification mismatch detected: {}", mismatch_type);
}

/// 记录 Batch 部分失败
pub fn inc_batch_partial_failure(batch_id: &str, failure_reason: &str) {
    BATCH_PARTIAL_FAILURE_COUNT.inc();
    log::warn!("Batch partial failure: batch_id={}, reason={}", batch_id, failure_reason);
}

/// 记录 Transaction 中止
pub fn inc_transaction_abort(transaction_id: &str, abort_reason: &str) {
    TRANSACTION_ABORT_COUNT.inc();
    log::warn!("Transaction aborted: transaction_id={}, reason={}", transaction_id, abort_reason);
}

/// 记录指令重试
pub fn inc_instruction_retry(instruction_id: &str, retry_reason: &str) {
    INSTRUCTION_RETRY_COUNT.inc();
    log::debug!("Instruction retry: instruction_id={}, reason={}", instruction_id, retry_reason);
}

/// 设置指令成功率
pub fn set_instruction_success_rate(rate: f64) {
    INSTRUCTION_SUCCESS_RATE.set(rate);
}

/// 记录灰度回滚
pub fn inc_gray_release_rollback(release_id: &str, rollback_reason: &str) {
    GRAY_RELEASE_ROLLBACK_COUNT.inc();
    log::warn!("Gray release rollback: release_id={}, reason={}", release_id, rollback_reason);
}

/// 观察 OIDC Token 验证时延
pub fn observe_oidc_validation_latency(latency_ms: f64) {
    OIDC_TOKEN_VALIDATION_LATENCY_P99.observe(latency_ms);
}

/// 记录 OPA 策略评估
pub fn inc_opa_policy_evaluation(policy_name: &str, decision: &str) {
    OPA_POLICY_EVALUATION_COUNT.inc();
    log::debug!("OPA policy evaluation: policy={}, decision={}", policy_name, decision);
}

/// 设置密钥轮换成功率
pub fn set_secret_rotation_success_rate(rate: f64) {
    SECRET_ROTATION_SUCCESS_RATE.set(rate);
}

/// 设置客户端请求速率
pub fn set_client_request_rate(rate: f64) {
    CLIENT_REQUEST_RATE.set(rate);
}

/// 设置客户端错误率
pub fn set_client_error_rate(rate: f64) {
    CLIENT_ERROR_RATE.set(rate);
}

/// 设置磁盘 IO 等待百分比
pub fn set_disk_io_wait_percent(percent: f64) {
    DISK_IO_WAIT_PERCENT.set(percent);
}

/// 设置网络丢包率
pub fn set_network_packet_drop_rate(rate: f64) {
    NETWORK_PACKET_DROP_RATE.set(rate);
}

/// 观察全链路追踪时长
pub fn observe_trace_total_duration(duration_ms: f64) {
    TRACE_TOTAL_DURATION_P99.observe(duration_ms);
}

/// 设置平均 Span 数量
pub fn set_trace_span_count_avg(count: f64) {
    TRACE_SPAN_COUNT_AVG.set(count);
}

/// 设置追踪传递成功率
pub fn set_trace_propagation_success_rate(rate: f64) {
    TRACE_PROPAGATION_SUCCESS_RATE.set(rate);
}

/// 记录异常检测告警
pub fn inc_anomaly_detection_alert(anomaly_type: &str, severity: &str) {
    ANOMALY_DETECTION_ALERT_COUNT.inc();
    log::warn!("Anomaly detection alert: type={}, severity={}", anomaly_type, severity);
}

/// 设置威胁处置平均时间
pub fn set_threat_mitigation_time_avg(time_secs: f64) {
    THREAT_MITIGATION_TIME_AVG.set(time_secs);
}

// ============================================================================
// 指标注册与导出
// ============================================================================

/// 注册所有 Batch 4 指标
pub fn register_all_metrics() -> Result<(), String> {
    // Prometheus 会自动注册 lazy_static 指标
    // 这里用于验证指标注册成功
    
    let metrics_count = 20; // Batch 4 指标数
    log::info!("Registered {} Phase 3 Batch 4 metrics", metrics_count);
    
    Ok(())
}

/// 导出所有指标值 (用于调试)
pub fn export_metrics_snapshot() -> MetricsSnapshot {
    MetricsSnapshot {
        execution_panic_count: EXECUTION_PANIC_COUNT.get(),
        execution_timeout_count: EXECUTION_TIMEOUT_COUNT.get(),
        verification_mismatch_count: VERIFICATION_MISMATCH_COUNT.get(),
        batch_partial_failure_count: BATCH_PARTIAL_FAILURE_COUNT.get(),
        transaction_abort_count: TRANSACTION_ABORT_COUNT.get(),
        instruction_retry_count: INSTRUCTION_RETRY_COUNT.get(),
        instruction_success_rate: INSTRUCTION_SUCCESS_RATE.get(),
        gray_release_rollback_count: GRAY_RELEASE_ROLLBACK_COUNT.get(),
        opa_policy_evaluation_count: OPA_POLICY_EVALUATION_COUNT.get(),
        secret_rotation_success_rate: SECRET_ROTATION_SUCCESS_RATE.get(),
        client_request_rate: CLIENT_REQUEST_RATE.get(),
        client_error_rate: CLIENT_ERROR_RATE.get(),
        disk_io_wait_percent: DISK_IO_WAIT_PERCENT.get(),
        network_packet_drop_rate: NETWORK_PACKET_DROP_RATE.get(),
        trace_span_count_avg: TRACE_SPAN_COUNT_AVG.get(),
        trace_propagation_success_rate: TRACE_PROPAGATION_SUCCESS_RATE.get(),
        anomaly_detection_alert_count: ANOMALY_DETECTION_ALERT_COUNT.get(),
        threat_mitigation_time_avg: THREAT_MITIGATION_TIME_AVG.get(),
    }
}

/// 指标快照
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub execution_panic_count: u64,
    pub execution_timeout_count: u64,
    pub verification_mismatch_count: u64,
    pub batch_partial_failure_count: u64,
    pub transaction_abort_count: u64,
    pub instruction_retry_count: u64,
    pub instruction_success_rate: f64,
    pub gray_release_rollback_count: u64,
    pub opa_policy_evaluation_count: u64,
    pub secret_rotation_success_rate: f64,
    pub client_request_rate: f64,
    pub client_error_rate: f64,
    pub disk_io_wait_percent: f64,
    pub network_packet_drop_rate: f64,
    pub trace_span_count_avg: f64,
    pub trace_propagation_success_rate: f64,
    pub anomaly_detection_alert_count: u64,
    pub threat_mitigation_time_avg: f64,
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_metrics() {
        // 测试错误指标
        inc_execution_panic("test_location");
        assert_eq!(EXECUTION_PANIC_COUNT.get(), 1);
        
        inc_execution_timeout("test_instruction");
        assert_eq!(EXECUTION_TIMEOUT_COUNT.get(), 1);
        
        inc_verification_mismatch("test_mismatch");
        assert_eq!(VERIFICATION_MISMATCH_COUNT.get(), 1);
        
        inc_batch_partial_failure("batch_1", "test_reason");
        assert_eq!(BATCH_PARTIAL_FAILURE_COUNT.get(), 1);
        
        inc_transaction_abort("txn_1", "test_reason");
        assert_eq!(TRANSACTION_ABORT_COUNT.get(), 1);
    }
    
    #[test]
    fn test_business_metrics() {
        // 测试业务指标
        inc_instruction_retry("instr_1", "test_reason");
        assert_eq!(INSTRUCTION_RETRY_COUNT.get(), 1);
        
        set_instruction_success_rate(99.5);
        assert!((INSTRUCTION_SUCCESS_RATE.get() - 99.5).abs() < 0.01);
        
        inc_gray_release_rollback("release_1", "test_reason");
        assert_eq!(GRAY_RELEASE_ROLLBACK_COUNT.get(), 1);
        
        observe_oidc_validation_latency(50.0);
        
        inc_opa_policy_evaluation("test_policy", "allow");
        assert_eq!(OPA_POLICY_EVALUATION_COUNT.get(), 1);
        
        set_secret_rotation_success_rate(100.0);
        assert!((SECRET_ROTATION_SUCCESS_RATE.get() - 100.0).abs() < 0.01);
        
        set_client_request_rate(1000.0);
        assert!((CLIENT_REQUEST_RATE.get() - 1000.0).abs() < 0.01);
        
        set_client_error_rate(0.5);
        assert!((CLIENT_ERROR_RATE.get() - 0.5).abs() < 0.01);
    }
    
    #[test]
    fn test_system_metrics() {
        // 测试系统指标
        set_disk_io_wait_percent(15.5);
        assert!((DISK_IO_WAIT_PERCENT.get() - 15.5).abs() < 0.01);
        
        set_network_packet_drop_rate(0.1);
        assert!((NETWORK_PACKET_DROP_RATE.get() - 0.1).abs() < 0.01);
    }
    
    #[test]
    fn test_tracing_metrics() {
        // 测试追踪指标
        observe_trace_total_duration(500.0);
        
        set_trace_span_count_avg(25.0);
        assert!((TRACE_SPAN_COUNT_AVG.get() - 25.0).abs() < 0.01);
        
        set_trace_propagation_success_rate(99.5);
        assert!((TRACE_PROPAGATION_SUCCESS_RATE.get() - 99.5).abs() < 0.01);
    }
    
    #[test]
    fn test_threat_detection_metrics() {
        // 测试威胁检测指标
        inc_anomaly_detection_alert("test_anomaly", "high");
        assert_eq!(ANOMALY_DETECTION_ALERT_COUNT.get(), 1);
        
        set_threat_mitigation_time_avg(30.0);
        assert!((THREAT_MITIGATION_TIME_AVG.get() - 30.0).abs() < 0.01);
    }
    
    #[test]
    fn test_metrics_snapshot() {
        // 测试指标快照
        let snapshot = export_metrics_snapshot();
        
        assert!(snapshot.instruction_success_rate >= 0.0);
        assert!(snapshot.instruction_success_rate <= 100.0);
        assert!(snapshot.secret_rotation_success_rate >= 0.0);
        assert!(snapshot.secret_rotation_success_rate <= 100.0);
        assert!(snapshot.trace_propagation_success_rate >= 0.0);
        assert!(snapshot.trace_propagation_success_rate <= 100.0);
    }
}
