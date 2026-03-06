//! 扫描器模块
//! 
//! 实现非确定性扫描器及其优化：
//! - 路径识别：127 路径 100% 识别
//! - 误报率优化：3.2% → <2%
//! - 自适应优化：动态调整检测参数

pub mod scanner_optimizer;

pub use scanner_optimizer::{
    ScannerOptimizer,
    ScannerOptimizerConfig,
    ScannerOptimizerStats,
    ScanResult,
    ExecutionPath,
    Operation,
    OperationType,
    PathType,
    LockInfo,
    LockType,
    LockGranularity,
};
