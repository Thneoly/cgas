//! 指标采集模块 (Metrics Collection Module)
//! 
//! Phase 3 Week 5: 50 指标全量接入
//! 
//! **模块结构**:
//! - Batch 1 (10 指标): M-001 ~ M-010 (基础指标)
//! - Batch 2 (10 指标): M-011 ~ M-020 (扩展指标)
//! - Batch 3 (10 指标): M-021 ~ M-030 (性能指标)
//! - Batch 4 (20 指标): M-031 ~ M-050 (全量指标)
//! 
//! **参考文档**: phase3_50_metrics_plan.md

pub mod metrics_20_batch4_impl;

pub use metrics_20_batch4_impl::*;

/// 注册所有 Phase 3 指标
pub fn register_all_phase3_metrics() -> Result<(), String> {
    // Batch 4: 20 个新增指标
    metrics_20_batch4_impl::register_all_metrics()?;
    
    log::info!("All Phase 3 metrics registered successfully");
    Ok(())
}

/// 导出指标快照 (用于调试)
pub fn export_metrics_snapshot() -> metrics_20_batch4_impl::MetricsSnapshot {
    metrics_20_batch4_impl::export_metrics_snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_metrics() {
        let result = register_all_phase3_metrics();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_export_snapshot() {
        let snapshot = export_metrics_snapshot();
        assert!(snapshot.instruction_success_rate >= 0.0);
        assert!(snapshot.instruction_success_rate <= 100.0);
    }
}
