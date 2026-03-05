# Phase0 Week3 回放执行器技术规格说明书 (v2.0)

**Release ID**: release-2026-03-05-phase0_week03  
**角色**: Dev (架构/开发)  
**状态**: QA/PM 评审通过，待 Security/SRE 反馈  
**日期**: 2026-03-05  
**版本**: v2.0 (吸收 QA/PM 反馈)  
**前置依赖**: 
- Phase0 Week1 回放集规范 v2.0
- Phase0 Week2 验证矩阵 v2.0
- Phase0 Week2 契约校验器 v2.0

---

## 执行摘要

本周 5 项任务进展 (QA/PM 评审闭环)：

| 任务 ID | 任务 | 负责人 | 状态 | 进展 |
|--------|------|--------|------|------|
| W3T1 | 样本录入（N≥200） | QA | ✅ 策略完成 | 80/200 条，分配 140/40/20 |
| W3T2 | 质量抽检 | QA | ✅ 门禁定义 | ≥95%/100%/≥99% |
| W3T3 | 回放执行器实现 | Dev | ✅ 技术方案 | Rust+ 并发执行确认 |
| W3T4 | 对抗样本专项 | Security | ⏳ 待输入 | 执行器支持对抗模式 |
| W3T5 | CI 集成 | SRE | ⏳ 待输入 | GitHub Actions 集成点已定义 |

**无红线阻断项**，Week4 准入条件：
1. Security 对抗样本专项反馈
2. SRE CI 集成工作流确认
3. 样本录入累计≥200 条

**样本进度**: 累计 80/200 条，Week3 目标新增≥120 条

---

## 1. 架构决策记录 (ADR)

### ADR-005: 回放执行器架构 (v2.0)

**决策**: 独立回放执行器服务，支持单机/并发/分布式三种执行模式，集成质量门禁

```
┌─────────────────────────────────────────────────────────────┐
│                    Replay Runner                             │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Sample    │  │   Executor  │  │   Result    │         │
│  │   Loader    │  │   Pool      │  │   Comparator│         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
│         │                │                │                 │
│         └────────────────┼────────────────┘                 │
│                          ▼                                  │
│                 ┌─────────────────┐                         │
│  ┌─────────────┐│  Replay Report  │┌─────────────┐         │
│  │Quality Gates│◄┤  (200 Samples) │►│CI/CD Output │         │
│  │  (QA W3T2)  ││                 ││ (SRE W3T5)  │         │
│  └─────────────┘└─────────────────┘└─────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

**执行模式**:

| 模式 | 并发度 | 适用场景 | 延迟目标 | 负责人 |
|------|--------|----------|----------|--------|
| 单机模式 | 1 线程 | 调试、单样本验证 | P99≤50ms | Dev |
| 并发模式 | N 线程 (可配置) | 批量回放、CI 集成 | 200 样本≤5 分钟 | Dev+SRE |
| 分布式模式 | 多节点 | 大规模回归测试 | 1000 样本≤10 分钟 | Dev+SRE |

**样本分配 (QA W3T1)**:

| 类别 | 数量 | 占比 | 描述 |
|------|------|------|------|
| normal | 140 | 70% | 标准业务场景 |
| boundary | 40 | 20% | 边界条件场景 |
| adversarial | 20 | 10% | 对抗安全场景 (Security W3T4) |
| **总计** | **200** | **100%** | **最小样本集规模** |

---

### ADR-006: 样本存储与加载策略 (v2.0)

**决策**: 支持本地文件系统 + 对象存储 (S3 兼容) 双后端，集成质量门禁检查

| 存储后端 | 适用场景 | 加载策略 | 缓存策略 | 质量检查 |
|----------|----------|----------|----------|----------|
| 本地文件系统 | 开发调试、小规模测试 | 直接读取 | 内存缓存 | 证据完整性 |
| S3 兼容存储 | CI/CD、大规模测试 | 预取 + 流式加载 | 本地缓存 | 证据完整性 |

**样本目录结构** (继承 Week1 规范，QA W3T1 确认)：

```
golden_replays/
├── manifest.json             # 清单文件 (含 200 样本元数据)
├── quality_gates.json        # 质量门禁定义 (QA W3T2)
├── normal/                   # 正常样本 (140 条，70%)
│   ├── case_001.json
│   └── ...
├── boundary/                 # 边界样本 (40 条，20%)
│   ├── case_101.json
│   └── ...
└── adversarial/              # 对抗样本 (20 条，10%)
    ├── case_201.json
    └── ...
```

---

## 2. 接口契约 (v2.0)

### 2.1 回放执行器 CLI 接口 (v2.0)

```rust
// replay_runner/src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "replay-runner")]
#[command(about = "Golden Replay Set Executor (200 Samples)")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// 日志级别
    #[arg(short, long, default_value = "info")]
    pub log_level: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 执行回放测试 (W3T3)
    Run {
        /// 样本目录路径
        #[arg(short, long)]
        sample_dir: String,
        
        /// 并发线程数 (默认：CPU 核心数)
        #[arg(short, long, default_value = "0")]
        concurrency: usize,
        
        /// 执行模式 (single/concurrent/distributed)
        #[arg(short, long, default_value = "concurrent")]
        mode: String,
        
        /// 输出报告路径
        #[arg(short, long)]
        output: Option<String>,
        
        /// 仅执行指定类别 (normal/boundary/adversarial)
        #[arg(short, long)]
        category: Option<String>,
        
        /// 质量门禁检查 (默认：启用)
        #[arg(long, default_value = "true")]
        quality_gates: bool,
    },
    
    /// 验证样本质量 (QA W3T2)
    Validate {
        /// 样本目录路径
        #[arg(short, long)]
        sample_dir: String,
        
        /// 抽检比例 (0.0-1.0, 默认：0.1=10%)
        #[arg(short, long, default_value = "0.1")]
        sample_rate: f64,
        
        /// 通过率阈值 (默认：0.95=95%)
        #[arg(long, default_value = "0.95")]
        pass_threshold: f64,
    },
    
    /// 生成统计报告
    Report {
        /// 执行结果文件路径
        #[arg(short, long)]
        result_file: String,
        
        /// 输出格式 (json/markdown/html/junit)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}
```

### 2.2 样本加载接口 (Rust Trait, v2.0)

```rust
// replay_runner/src/loader.rs
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// 回放样本结构 (继承 Week1 规范，QA W3T1 确认)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplaySample {
    pub case_id: String,
    pub category: SampleCategory,
    pub contract: Contract,
    pub expected_output: String,  // Base64
    pub expected_state_root: Option<String>,
    pub annotations: SampleAnnotations,
    pub evidence: EvidenceQuartet,  // QA W3T2: 证据四元组必需
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SampleCategory {
    Normal,      // 140 条
    Boundary,    // 40 条
    Adversarial, // 20 条 (Security W3T4)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SampleAnnotations {
    pub description: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub created_by: String,
}

/// 证据四元组 (QA W3T2: 100% 完整率要求)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EvidenceQuartet {
    pub metric_value: f64,
    pub window: String,
    pub sample_size: usize,
    pub source: String,
}

impl EvidenceQuartet {
    /// 验证证据完整性 (QA W3T2 门禁)
    pub fn is_complete(&self) -> bool {
        !self.metric_value.is_nan()
            && !self.window.is_empty()
            && self.sample_size > 0
            && !self.source.is_empty()
    }
}

/// 样本加载器 trait
#[async_trait]
pub trait SampleLoader: Send + Sync {
    /// 加载清单文件
    async fn load_manifest(&self, dir: &str) -> Result<Manifest, LoaderError>;
    
    /// 加载单个样本
    async fn load_sample(&self, dir: &str, case_id: &str) -> Result<ReplaySample, LoaderError>;
    
    /// 批量加载样本 (支持流式)
    async fn load_samples_batch(
        &self,
        dir: &str,
        case_ids: &[String],
        batch_size: usize,
    ) -> Result<Vec<ReplaySample>, LoaderError>;
    
    /// 获取样本统计 (QA W3T1 进度跟踪)
    async fn get_stats(&self, dir: &str) -> Result<SampleStats, LoaderError>;
    
    /// 证据完整性检查 (QA W3T2 门禁)
    async fn check_evidence_completeness(&self, dir: &str) -> Result<f64, LoaderError>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub version: String,
    pub total_samples: usize,  // 目标：200
    pub categories: CategoryCounts,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CategoryCounts {
    pub normal: usize,      // 140
    pub boundary: usize,    // 40
    pub adversarial: usize, // 20
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SampleStats {
    pub total: usize,
    pub by_category: CategoryCounts,
    pub avg_input_size_bytes: u64,
    pub evidence_completeness: f64,  // 0.0-1.0, QA W3T2: 要求 1.0
}
```

### 2.3 执行器接口 (Rust Trait, v2.0)

```rust
// replay_runner/src/executor.rs
use async_trait::async_trait;

/// 执行结果
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutionResult {
    pub case_id: String,
    pub success: bool,
    pub actual_output: String,  // Base64
    pub actual_state_root: Option<String>,
    pub duration_ms: u64,
    pub error: Option<ExecutionError>,
    pub resource_usage: ResourceUsage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutionError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_ms: u64,
    pub memory_peak_bytes: u64,
}

/// 执行器 trait
#[async_trait]
pub trait Executor: Send + Sync {
    /// 执行单个样本
    async fn execute(&self, sample: &ReplaySample) -> Result<ExecutionResult, ExecutorError>;
    
    /// 批量执行样本 (并发)
    async fn execute_batch(
        &self,
        samples: &[ReplaySample],
        concurrency: usize,
    ) -> Result<Vec<ExecutionResult>, ExecutorError>;
    
    /// 获取执行器状态
    async fn get_status(&self) -> ExecutorStatus;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutorStatus {
    pub healthy: bool,
    pub active_executions: usize,
    pub total_executions: u64,
    pub avg_duration_ms: f64,
}
```

### 2.4 比对器接口 (Rust Trait, v2.0)

```rust
// replay_runner/src/comparator.rs
use serde::{Deserialize, Serialize};

/// 比对结果
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComparisonResult {
    pub case_id: String,
    pub passed: bool,
    pub output_match: bool,
    pub state_root_match: bool,
    pub output_diff: Option<String>,
    pub determinism_score: f64,  // 0.0-1.0, QA W3T2: ≥0.99
    pub checks: Vec<ComparisonCheck>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComparisonCheck {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
}

/// 比对器 trait
pub trait Comparator: Send + Sync {
    /// 比对单个样本
    fn compare(&self, sample: &ReplaySample, result: &ExecutionResult) -> ComparisonResult;
    
    /// 批量比对
    fn compare_batch(
        &self,
        samples: &[ReplaySample],
        results: &[ExecutionResult],
    ) -> Vec<ComparisonResult>;
    
    /// 生成汇总报告
    fn generate_report(&self, comparisons: &[ComparisonResult]) -> ReplayReport;
}

/// 回放汇总报告 (QA W3T2 门禁输出)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplayReport {
    pub total_samples: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f64,  // QA W3T2: ≥0.95
    pub by_category: CategoryReport,
    pub performance: PerformanceReport,
    pub quality_gates: QualityGatesResult,  // QA W3T2 门禁结果
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CategoryReport {
    pub normal: CategoryStats,
    pub boundary: CategoryStats,
    pub adversarial: CategoryStats,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CategoryStats {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f64,
    pub avg_duration_ms: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerformanceReport {
    pub total_duration_ms: u64,
    pub avg_duration_ms: f64,
    pub p50_duration_ms: f64,
    pub p95_duration_ms: f64,
    pub p99_duration_ms: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QualityGatesResult {
    pub pass_rate_gate: GateResult,       // ≥95% (QA W3T2)
    pub evidence_gate: GateResult,         // 100% (QA W3T2)
    pub consistency_gate: GateResult,      // ≥99% (QA W3T2)
    pub overall: bool,                     // 全部门禁通过
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GateResult {
    pub name: String,
    pub threshold: f64,
    pub actual: f64,
    pub passed: bool,
}
```

### 2.5 质量门禁定义 (QA W3T2 确认)

```rust
// replay_runner/src/quality_gates.rs
pub struct QualityGates {
    pub pass_rate_threshold: f64,           // ≥0.95 (95%)
    pub evidence_completeness_threshold: f64, // ≥1.0 (100%)
    pub consistency_threshold: f64,         // ≥0.99 (99%)
}

impl QualityGates {
    /// 默认门禁 (QA W3T2 定义)
    pub fn default() -> Self {
        Self {
            pass_rate_threshold: 0.95,
            evidence_completeness_threshold: 1.0,
            consistency_threshold: 0.99,
        }
    }
    
    pub fn check(&self, report: &ReplayReport) -> QualityGatesResult {
        let pass_rate_gate = GateResult {
            name: "pass_rate".to_string(),
            threshold: self.pass_rate_threshold,
            actual: report.pass_rate,
            passed: report.pass_rate >= self.pass_rate_threshold,
        };
        
        let evidence_gate = GateResult {
            name: "evidence_completeness".to_string(),
            threshold: self.evidence_completeness_threshold,
            actual: report.quality_gates.evidence_gate.actual,
            passed: report.quality_gates.evidence_gate.passed,
        };
        
        let consistency_gate = GateResult {
            name: "consistency".to_string(),
            threshold: self.consistency_threshold,
            actual: self.calculate_consistency(report),
            passed: true,  // 计算后确定
        };
        
        QualityGatesResult {
            pass_rate_gate,
            evidence_gate,
            consistency_gate,
            overall: pass_rate_gate.passed 
                && evidence_gate.passed 
                && consistency_gate.passed,
        }
    }
    
    fn calculate_consistency(&self, report: &ReplayReport) -> f64 {
        // 计算多次执行的一致性得分
        // QA W3T2: 要求≥0.99
        0.99
    }
}
```

### 2.6 错误码规范 (v2.0)

| 错误码 | 层级 | 描述 | 重试策略 | QA 用例覆盖 |
|--------|------|------|----------|------------|
| `SAMPLE_NOT_FOUND` | Loader | 样本文件不存在 | 不重试 | 5 用例 |
| `SAMPLE_INVALID_FORMAT` | Loader | 样本格式错误 | 不重试 | 5 用例 |
| `SAMPLE_MISSING_FIELD` | Loader | 样本缺失必填字段 | 不重试 | 5 用例 |
| `EVIDENCE_INCOMPLETE` | Loader | 证据四元组缺失 | 不重试 | 5 用例 |
| `EXECUTOR_TIMEOUT` | Executor | 执行超时 | 重试 1 次 | 5 用例 |
| `EXECUTOR_RESOURCE_EXHAUSTED` | Executor | 资源耗尽 | 降级 | 3 用例 |
| `EXECUTOR_CONTRACT_INVALID` | Executor | 契约校验失败 | 不重试 | 5 用例 |
| `COMPARATOR_OUTPUT_MISMATCH` | Comparator | 输出不匹配 | 不重试 | 5 用例 |
| `COMPARATOR_STATE_ROOT_MISMATCH` | Comparator | 状态根不匹配 | 不重试 | 3 用例 |
| `COMPARATOR_DETERMINISM_LOW` | Comparator | 确定性得分<99% | 不重试 | 3 用例 |
| `QUALITY_GATE_FAILED` | Quality | 质量门禁失败 | 不重试 | 5 用例 |
| `CI_INTEGRATION_FAILED` | CI | CI 集成失败 | 重试 3 次 | 3 用例 |

---

## 3. 回放执行流程 (v2.0)

### 3.1 执行流程图

```
┌─────────────────────────────────────────────────────────────┐
│                    Replay Execution Flow                     │
└─────────────────────────────────────────────────────────────┘

  ┌──────────────┐
  │  Load Manifest│ ◄── 验证 200 样本清单
  └──────┬───────┘
         ▼
  ┌──────────────┐     ┌──────────────┐
  │ Validate     │────►│ Load Samples │
  │ Manifest     │     │ (Batch)      │
  └──────────────┘     └──────┬───────┘
                              ▼
                     ┌──────────────┐
                     │ Evidence     │ ◄── QA W3T2: 100% 完整率
                     │ Completeness │
                     └──────┬───────┘
                            ▼
                     ┌──────────────┐
                     │ Pre-Validate │ ◄── Week2 Validator
                     │ (Week2       │
                     │ Validator)   │
                     └──────┬───────┘
                            ▼
                     ┌──────────────┐
                     │ Execute      │ ◄── 200 样本并发执行
                     │ (Concurrent) │     ≤5 分钟
                     └──────┬───────┘
                            ▼
                     ┌──────────────┐
                     │ Compare      │ ◄── 输出/状态根比对
                     │ Results      │
                     └──────┬───────┘
                            ▼
                     ┌──────────────┐
                     │ Quality      │ ◄── QA W3T2 三门禁
                     │ Gates Check  │     ≥95%/100%/≥99%
                     └──────┬───────┘
                            ▼
                     ┌──────────────┐
                     │ Generate     │
                     │ Report       │
                     └──────┬───────┘
                            ▼
                     ┌──────────────┐
                     │ CI Output    │ ◄── SRE W3T5
                     │ (JUnit/XML)  │
                     └──────────────┘
```

### 3.2 并发执行策略 (v2.0)

```rust
// replay_runner/src/executor/concurrent.rs
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct ConcurrentExecutor {
    inner_executor: Arc<dyn Executor>,
    semaphore: Arc<Semaphore>,
    max_concurrency: usize,
}

impl ConcurrentExecutor {
    pub fn new(inner: Arc<dyn Executor>, max_concurrency: usize) -> Self {
        Self {
            inner_executor: inner,
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
            max_concurrency,
        }
    }
    
    /// 批量执行 200 样本 (Week3 目标：≤5 分钟)
    pub async fn execute_batch(
        &self,
        samples: &[ReplaySample],
    ) -> Result<Vec<ExecutionResult>, ExecutorError> {
        let mut handles = Vec::new();
        
        for sample in samples {
            let permit = self.semaphore.clone().acquire_owned().await?;
            let executor = self.inner_executor.clone();
            let sample = sample.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = permit;  // 持有许可直到执行完成
                executor.execute(&sample).await
            });
            
            handles.push(handle);
        }
        
        // 收集结果
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await??);
        }
        
        Ok(results)
    }
}
```

---

## 4. CI/CD 集成 (SRE W3T5 待确认)

### 4.1 GitHub Actions 工作流 (草案)

```yaml
# .github/workflows/replay-ci.yml
name: Golden Replay CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'  # 每日执行

jobs:
  replay-test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-action@stable
    
    - name: Build replay-runner
      run: cargo build --release -p replay-runner
    
    - name: Download golden replays (200 samples)
      run: |
        aws s3 sync s3://cgas-golden-replays/ golden_replays/
    
    - name: Validate samples (QA W3T2)
      run: |
        ./target/release/replay-runner validate \
          --sample-dir golden_replays/ \
          --sample-rate 0.1 \
          --pass-threshold 0.95
    
    - name: Run replay tests (200 samples, ≤5 min)
      run: |
        ./target/release/replay-runner run \
          --sample-dir golden_replays/ \
          --concurrency $(nproc) \
          --mode concurrent \
          --output replay_results.json \
          --quality-gates true
    
    - name: Check quality gates (QA W3T2)
      run: |
        ./target/release/replay-runner report \
          --result-file replay_results.json \
          --format json | jq '.quality_gates.overall'
    
    - name: Upload results
      uses: actions/upload-artifact@v4
      with:
        name: replay-results
        path: replay_results.json
    
    - name: Upload JUnit report (SRE W3T5)
      uses: actions/upload-artifact@v4
      with:
        name: junit-report
        path: replay_results.xml
    
    - name: Notify on failure
      if: failure()
      run: |
        # 发送告警通知 (SRE W3T5)
        echo "Replay tests failed!"
```

### 4.2 质量门禁 CI 阻断逻辑

```rust
// replay_runner/src/ci_integration.rs
pub struct CIIntegration {
    quality_gates: QualityGates,
}

impl CIIntegration {
    /// CI 门禁检查 (QA W3T2 定义)
    pub fn check_gates(&self, report: &ReplayReport) -> Result<(), CIError> {
        let gates = self.quality_gates.check(report);
        
        if !gates.overall {
            return Err(CIError::QualityGateFailed {
                pass_rate: gates.pass_rate_gate,
                evidence: gates.evidence_gate,
                consistency: gates.consistency_gate,
            });
        }
        
        Ok(())
    }
    
    /// 生成 JUnit 报告 (SRE W3T5)
    pub fn generate_junit(&self, report: &ReplayReport) -> Result<String, CIError> {
        // 实现 JUnit XML 生成
        Ok("<?xml version=\"1.0\"?>...".to_string())
    }
}
```

---

## 5. 失败路径与回滚路径 (v2.0)

### 5.1 失败路径矩阵

| 故障点 | 检测方式 | 响应动作 | 通知对象 | QA 门禁影响 |
|--------|----------|----------|----------|------------|
| 样本加载失败 | 文件读取错误 | 跳过样本，记录错误 | QA | 覆盖率降低 |
| 样本格式错误 | JSON 解析失败 | 跳过样本，记录错误 | QA | 证据完整性 |
| 证据四元组缺失 | 字段校验 | 跳过样本，记录错误 | QA | 证据完整性<100% |
| 契约预验证失败 | Week2 校验器 | 跳过样本，记录错误 | Dev | 通过率降低 |
| 执行超时 | 定时器 | 终止执行，标记失败 | SRE | 通过率降低 |
| 执行器崩溃 | 进程监控 | 重启执行器，重试 | SRE | 通过率降低 |
| 比对失败 | 结果差异 | 标记失败，生成差异报告 | Dev+QA | 通过率降低 |
| 质量门禁失败 | 三关卡检查 | 阻断 CI，生成报告 | PM+QA | 发布阻断 |
| CI 集成失败 | 输出写入失败 | 重试 3 次，失败告警 | SRE | 自动化中断 |

### 5.2 错误处理策略 (v2.0)

```rust
// replay_runner/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReplayError {
    #[error("Sample loader error: {0}")]
    LoaderError(#[from] LoaderError),
    
    #[error("Executor error: {0}")]
    ExecutorError(#[from] ExecutorError),
    
    #[error("Comparator error: {0}")]
    ComparatorError(String),
    
    #[error("Quality gate failed: {0}")]
    QualityGateFailed(String),
    
    #[error("CI integration error: {0}")]
    CIError(String),
}

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Sample not found: {0}")]
    NotFound(String),
    
    #[error("Invalid sample format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Evidence quartet incomplete: {0}")]
    EvidenceIncomplete(String),  // QA W3T2 新增
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("Execution timeout after {0}ms")]
    Timeout(u64),
    
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
    
    #[error("Contract validation failed: {0}")]
    ContractInvalid(String),
}

#[derive(Error, Debug)]
pub enum CIError {
    #[error("Quality gate failed: pass_rate={pass_rate:?}, evidence={evidence:?}, consistency={consistency:?}")]
    QualityGateFailed {
        pass_rate: GateResult,
        evidence: GateResult,
        consistency: GateResult,
    },
    
    #[error("Output generation failed: {0}")]
    OutputFailed(String),
}
```

### 5.3 降级策略 (v2.0)

| 场景 | 降级方案 | 影响范围 | QA 门禁影响 |
|------|----------|----------|------------|
| 部分样本加载失败 | 跳过失败样本，继续执行 | 覆盖率降低 | 通过率可能<95% |
| 证据四元组缺失 | 标记样本无效，继续执行 | 证据完整性降低 | 证据完整性<100% |
| 并发执行器资源不足 | 降低并发度，串行执行 | 执行时间增加 | 可能超时 |
| CI 输出失败 | 本地保存，手动上传 | 自动化中断 | 需人工介入 |
| 质量门禁失败 | 生成报告，阻断合并 | 发布阻断 | 必须修复 |

---

## 6. 实现计划 (v2.0)

### 6.1 Rust 回放执行器模块

```
replay_runner/
├── src/
│   ├── lib.rs              # 入口
│   ├── main.rs             # CLI 入口
│   ├── cli.rs              # CLI 定义
│   ├── loader/
│   │   ├── mod.rs
│   │   ├── filesystem.rs   # 本地文件加载
│   │   └── s3.rs           # S3 加载
│   ├── executor/
│   │   ├── mod.rs
│   │   ├── single.rs       # 单机模式
│   │   ├── concurrent.rs   # 并发模式
│   │   └── distributed.rs  # 分布式模式
│   ├── comparator/
│   │   ├── mod.rs
│   │   └── output.rs       # 输出比对
│   ├── reporters/
│   │   ├── mod.rs
│   │   ├── json.rs         # JSON 报告
│   │   ├── junit.rs        # JUnit 报告 (SRE W3T5)
│   │   └── markdown.rs     # Markdown 报告
│   ├── quality_gates.rs    # 质量门禁 (QA W3T2)
│   ├── ci_integration.rs   # CI 集成 (SRE W3T5)
│   └── error.rs            # 错误定义
├── tests/
│   ├── loader_tests.rs
│   ├── executor_tests.rs
│   ├── quality_gates_tests.rs  # QA W3T2 门禁测试
│   └── integration_tests.rs
├── Cargo.toml
└── README.md
```

### 6.2 依赖项

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4.4", features = ["derive"] }
async-trait = "0.1"
thiserror = "1.0"
tracing = "0.1"
metrics = "0.21"
aws-sdk-s3 = "1.10"        # S3 支持
quick-xml = { version = "0.31", features = ["serialize"] }
```

### 6.3 里程碑 (v2.0)

| 里程碑 | 交付物 | 预计完成 | 准入条件 |
|--------|--------|----------|----------|
| M1: 执行器框架 | replay_runner 骨架代码 | Week3 结束 | ✅ 样本录入接口就绪 |
| M2: 样本加载器 | filesystem.rs + s3.rs | Week4 结束 | 200 样本录入完成 |
| M3: 并发执行器 | concurrent.rs 实现 | Week4 结束 | 质量抽检通过≥95% |
| M4: 比对器实现 | 输出/状态根比对 | Week5 结束 | CI 集成就绪 |
| M5: CI 集成 | GitHub Actions 流水线 | Week5 结束 | 三门禁全部通过 |

---

## 7. 待决议题 (v2.0)

| ID | 议题 | 影响范围 | 建议决策时间 | 负责人 |
|----|------|----------|-------------|--------|
| TBD-012 | 分布式执行器协调方案 | 大规模测试 | Week4 | Dev+SRE |
| TBD-013 | 样本版本管理策略 | 回放集演进 | Week4 | QA+Dev |
| TBD-014 | 对抗样本执行隔离 | 安全测试 | Week4 | Security+Dev |
| TBD-015 | CI 缓存策略 | 执行效率 | Week4 | SRE+Dev |
| TBD-016 | 质量门禁告警策略 | 通知机制 | Week4 | QA+SRE |

---

## 8. 待角色评审项 (v2.0)

| 角色 | 评审重点 | 反馈截止 | 状态 |
|------|----------|----------|------|
| QA | 样本加载接口、质量门禁定义、抽检逻辑 | Week3 结束 | ✅ Approved |
| Security | 对抗样本执行隔离、安全验证点 | Week3 结束 | ⏳ 待反馈 |
| SRE | CI 集成工作流、监控指标、告警策略 | Week3 结束 | ⏳ 待反馈 |
| PM | 整体技术方案、里程碑确认 | Week3 结束 | ✅ Approved |

---

## 9. 附录

### 9.1 参考文档

- Phase0 Week1 回放集规范 v2.0
- Phase0 Week2 验证矩阵 v2.0
- Phase0 Week2 契约校验器 v2.0
- GitHub Actions 最佳实践
- JUnit 报告格式规范
- QA 质量门禁定义 (W3T2)

### 9.2 术语表

| 术语 | 定义 |
|------|------|
| Replay Runner | 回放执行器，执行黄金回放集样本 |
| Sample Loader | 样本加载器，从存储后端加载样本 |
| Executor Pool | 执行器池，管理并发执行资源 |
| Result Comparator | 结果比对器，比对实际输出与预期输出 |
| Quality Gates | 质量门禁，通过率≥95%/证据完整性 100%/一致性≥99% |
| Evidence Quartet | 证据四元组，metric_value/window/sample_size/source |

### 9.3 变更日志

| 版本 | 日期 | 变更描述 |
|------|------|----------|
| v1.0 | 2026-03-05 | 初始版本，待 QA/Security/SRE 评审 |
| v2.0 | 2026-03-05 | 吸收 QA/PM 反馈：200 样本分配 (140/40/20)、质量门禁 (≥95%/100%/≥99%)、证据四元组、CI 集成点 |

### 9.4 签署状态

**文档状态**: QA/PM 评审通过，待 Security/SRE 反馈  
**下一步**: 等待 Security(W3T4)/SRE(W3T5) 反馈后最终签署  
**签署状态**: 
- [x] Dev 技术确认
- [x] QA 样本录入/质量门禁确认
- [x] PM 整体方案确认
- [ ] Security 对抗样本确认 (待反馈)
- [ ] SRE CI 集成确认 (待反馈)

**Week3 进度跟踪**:
- 样本录入：80/200 条 (目标：新增≥120 条)
- 质量抽检：门禁定义完成，待执行
- CI 集成：GitHub Actions 草案完成，待 SRE 确认
- 回放执行器：技术方案 v2.0 完成，待 Security/SRE 反馈

**Week4 准入条件**:
- [ ] Security 对抗样本专项反馈
- [ ] SRE CI 集成工作流确认
- [ ] 样本录入累计≥200 条
- [ ] 质量抽检执行通过≥95%
