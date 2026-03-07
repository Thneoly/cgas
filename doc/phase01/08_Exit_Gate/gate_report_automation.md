# Phase 3 Gate-Report 自动化生成方案

**版本**: v1.0  
**日期**: 2026-05-12  
**责任人**: Observability-Agent  
**状态**: 📋 设计中  
**release_id**: release-2026-05-12-phase3_week02  
**关联文档**: 
- phase3_50_metrics_plan.md (55 指标规划)
- distributed_tracing.md (分布式追踪设计)
- gate_report_schema.json (Gate-Report Schema)

---

## 1. 设计目标

### 1.1 Phase 2 vs Phase 3 Gate-Report 对比

| 指标 | Phase 2 实际 | Phase 3 目标 | 提升幅度 |
|---|---|---|---|
| 报告生成方式 | 手动 | **100% 自动化** | 新增 |
| 报告生成时间 | 2-4 小时 | <5 分钟 | -95% |
| 数据准确性 | 人工校验 | **自动校验** | 新增 |
| 报告更新频率 | 每周 | **实时** | +672% |
| 证据链完整性 | 80% | ≥99% | +24% |

### 1.2 核心设计原则

| 原则 | 说明 | 验收标准 |
|---|---|---|
| **自动化** | 一键生成，无需人工干预 | 生成时间<5 分钟 |
| **可验证** | 所有数据可追溯到源头 | 100% 指标可查询 |
| **结构化** | JSON + Markdown 双格式 | Schema 校验通过 |
| **实时性** | 支持按需生成 | 数据延迟<5 分钟 |
| **可审计** | 完整证据链 | 审计覆盖率 100% |

---

## 2. Gate-Report Schema 设计

### 2.1 JSON Schema

基于现有 `gate_report_schema.json` 扩展 Phase 3 字段：

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Phase 3 Gate Report",
  "type": "object",
  "required": [
    "release_id",
    "phase",
    "gate_type",
    "timestamp",
    "decision",
    "metrics",
    "evidence"
  ],
  "properties": {
    "release_id": {
      "type": "string",
      "description": "发布 ID"
    },
    "phase": {
      "type": "string",
      "enum": ["phase0", "phase1", "phase2", "phase3", "phase4", "phase5"]
    },
    "gate_type": {
      "type": "string",
      "enum": ["entry", "exit", "midterm"]
    },
    "timestamp": {
      "type": "string",
      "format": "date-time"
    },
    "decision": {
      "type": "string",
      "enum": ["Go", "Conditional Go", "No-Go"]
    },
    "metrics": {
      "type": "object",
      "required": ["total", "passed", "failed", "items"],
      "properties": {
        "total": {"type": "integer"},
        "passed": {"type": "integer"},
        "failed": {"type": "integer"},
        "items": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["id", "name", "value", "threshold", "status"],
            "properties": {
              "id": {"type": "string"},
              "name": {"type": "string"},
              "value": {"type": ["number", "string"]},
              "threshold": {"type": "string"},
              "status": {"type": "string", "enum": ["pass", "fail", "warning"]},
              "source": {"type": "string"},
              "query": {"type": "string"},
              "timestamp": {"type": "string", "format": "date-time"}
            }
          }
        }
      }
    },
    "evidence": {
      "type": "object",
      "properties": {
        "traces": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "trace_id": {"type": "string"},
              "description": {"type": "string"},
              "tempo_url": {"type": "string"},
              "jaeger_url": {"type": "string"}
            }
          }
        },
        "logs": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "log_query": {"type": "string"},
              "loki_url": {"type": "string"},
              "sample_count": {"type": "integer"}
            }
          }
        },
        "dashboards": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "title": {"type": "string"},
              "grafana_url": {"type": "string"},
              "uid": {"type": "string"}
            }
          }
        }
      }
    },
    "exceptions": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["metric_id", "reason", "approval", "expiry"],
        "properties": {
          "metric_id": {"type": "string"},
          "reason": {"type": "string"},
          "approval": {"type": "string"},
          "expiry": {"type": "string", "format": "date-time"}
        }
      }
    },
    "summary": {
      "type": "object",
      "properties": {
        "highlights": {"type": "array", "items": {"type": "string"}},
        "risks": {"type": "array", "items": {"type": "string"}},
        "recommendations": {"type": "array", "items": {"type": "string"}}
      }
    }
  }
}
```

### 2.2 Phase 3 新增字段

```rust
// gate_report_phase3.rs - Phase 3 Gate-Report 扩展

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Phase 3 Gate-Report 结构
#[derive(Debug, Serialize, Deserialize)]
pub struct Phase3GateReport {
    /// 基础信息
    pub release_id: String,
    pub phase: String,
    pub gate_type: GateType,
    pub timestamp: DateTime<Utc>,
    pub decision: GateDecision,
    
    /// 50 指标评估
    pub metrics: MetricsSummary,
    
    /// 追踪证据
    pub tracing_evidence: TracingEvidence,
    
    /// 日志证据
    pub log_evidence: LogEvidence,
    
    /// 仪表盘链接
    pub dashboards: Vec<DashboardLink>,
    
    /// 例外审批
    pub exceptions: Vec<Exception>,
    
    /// 摘要
    pub summary: ReportSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GateType {
    Entry,
    Midterm,
    Exit,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GateDecision {
    #[serde(rename = "Go")]
    Go,
    #[serde(rename = "Conditional Go")]
    ConditionalGo,
    #[serde(rename = "No-Go")]
    NoGo,
}

/// 50 指标汇总
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub warning: usize,
    pub items: Vec<MetricItem>,
    
    // Phase 3 新增分类统计
    pub by_category: CategoryBreakdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryBreakdown {
    pub performance: CategoryMetrics,
    pub consistency: CategoryMetrics,
    pub security: CategoryMetrics,
    pub business: CategoryMetrics,
    pub tracing: CategoryMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryMetrics {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
}

/// 单个指标项
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricItem {
    pub id: String,          // M-001, M-002, ...
    pub name: String,        // 指标名称
    pub category: String,    // 性能/一致性/安全/业务/追踪
    pub value: MetricValue,
    pub threshold: String,   // 告警阈值
    pub status: MetricStatus,
    pub source: String,      // 数据来源 (Prometheus/Tempo/Loki)
    pub query: String,       // 查询语句
    pub timestamp: DateTime<Utc>,
    pub trend: Option<Trend>, // 趋势分析
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MetricValue {
    Number(f64),
    String(String),
    Percentage(f64),
    Duration(u64), // 毫秒
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MetricStatus {
    #[serde(rename = "pass")]
    Pass,
    #[serde(rename = "fail")]
    Fail,
    #[serde(rename = "warning")]
    Warning,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trend {
    pub direction: String, // "up", "down", "stable"
    pub change_percent: f64,
    pub period: String,    // "24h", "7d", "30d"
}

/// 追踪证据
#[derive(Debug, Serialize, Deserialize)]
pub struct TracingEvidence {
    pub coverage_rate: f64,
    pub propagation_success_rate: f64,
    pub sample_traces: Vec<SampleTrace>,
    pub critical_paths_coverage: CriticalPathsCoverage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SampleTrace {
    pub trace_id: String,
    pub description: String,
    pub duration_ms: u64,
    pub span_count: usize,
    pub tempo_url: String,
    pub jaeger_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CriticalPathsCoverage {
    pub total_paths: usize,
    pub covered_paths: usize,
    pub missing_paths: Vec<String>,
}

/// 日志证据
#[derive(Debug, Serialize, Deserialize)]
pub struct LogEvidence {
    pub collection_rate: f64,
    pub sample_queries: Vec<LogQuery>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogQuery {
    pub description: String,
    pub query: String,
    pub loki_url: String,
    pub sample_count: usize,
}

/// 仪表盘链接
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardLink {
    pub title: String,
    pub grafana_url: String,
    pub uid: String,
}

/// 例外审批
#[derive(Debug, Serialize, Deserialize)]
pub struct Exception {
    pub metric_id: String,
    pub reason: String,
    pub approval: String,
    pub expiry: DateTime<Utc>,
    pub compensating_controls: Vec<String>,
}

/// 报告摘要
#[derive(Debug, Serialize, Deserialize)]
pub struct ReportSummary {
    pub highlights: Vec<String>,
    pub risks: Vec<String>,
    pub recommendations: Vec<String>,
}
```

---

## 3. 自动化生成流程

### 3.1 整体流程

```
┌─────────────────────────────────────────────────────────────────┐
│                    Gate-Report 自动化生成流程                     │
└─────────────────────────────────────────────────────────────────┘

1. 触发阶段
   ├── 定时触发 (每日 8:00 AM)
   ├── 事件触发 (发布前)
   └── 手动触发 (API 调用)
   
2. 数据采集阶段
   ├── Prometheus 指标查询 (50 个指标)
   ├── Tempo 追踪查询 (Trace 证据)
   ├── Loki 日志查询 (日志证据)
   └── Grafana 仪表盘链接
   
3. 数据验证阶段
   ├── Schema 校验
   ├── 阈值比对
   ├── 趋势分析
   └── 异常检测
   
4. 决策生成阶段
   ├── 指标通过率计算
   ├── 例外审批检查
   ├── 风险评估
   └── Go/Conditional/No-Go 决策
   
5. 报告生成阶段
   ├── JSON 格式报告
   ├── Markdown 格式报告
   └── PDF 格式报告 (可选)
   
6. 发布阶段
   ├── 存储到文档库
   ├── 通知相关人员
   └── 更新 Gate 状态
```

### 3.2 Rust 实现

```rust
// gate_report_generator.rs - Gate-Report 自动生成器

use crate::gate_report_phase3::*;
use reqwest::Client;
use serde_json::json;
use chrono::Utc;
use std::error::Error;

pub struct GateReportGenerator {
    client: Client,
    prometheus_url: String,
    tempo_url: String,
    loki_url: String,
    grafana_url: String,
}

impl GateReportGenerator {
    pub fn new(
        prometheus_url: String,
        tempo_url: String,
        loki_url: String,
        grafana_url: String,
    ) -> Self {
        Self {
            client: Client::new(),
            prometheus_url,
            tempo_url,
            loki_url,
            grafana_url,
        }
    }
    
    /// 生成 Gate-Report
    pub async fn generate_report(
        &self,
        release_id: &str,
        phase: &str,
        gate_type: GateType,
    ) -> Result<Phase3GateReport, Box<dyn Error>> {
        println!("Generating Gate-Report for {} {} ({})...", phase, gate_type, release_id);
        
        // 1. 采集 50 个指标
        let metrics = self.collect_all_metrics().await?;
        
        // 2. 采集追踪证据
        let tracing_evidence = self.collect_tracing_evidence().await?;
        
        // 3. 采集日志证据
        let log_evidence = self.collect_log_evidence().await?;
        
        // 4. 获取仪表盘链接
        let dashboards = self.get_dashboard_links();
        
        // 5. 生成决策
        let decision = self.generate_decision(&metrics, &tracing_evidence);
        
        // 6. 生成摘要
        let summary = self.generate_summary(&metrics, &decision);
        
        // 7. 组装报告
        let report = Phase3GateReport {
            release_id: release_id.to_string(),
            phase: phase.to_string(),
            gate_type,
            timestamp: Utc::now(),
            decision,
            metrics,
            tracing_evidence,
            log_evidence,
            dashboards,
            exceptions: vec![], // 从配置加载
            summary,
        };
        
        Ok(report)
    }
    
    /// 采集所有 50 个指标
    async fn collect_all_metrics(&self) -> Result<MetricsSummary, Box<dyn Error>> {
        let mut items = Vec::new();
        let mut passed = 0;
        let mut failed = 0;
        let mut warning = 0;
        
        // 性能指标 (18 个)
        let performance_metrics = self.collect_performance_metrics().await?;
        for metric in performance_metrics {
            match metric.status {
                MetricStatus::Pass => passed += 1,
                MetricStatus::Fail => failed += 1,
                MetricStatus::Warning => warning += 1,
            }
            items.push(metric);
        }
        
        // 错误指标 (10 个)
        let error_metrics = self.collect_error_metrics().await?;
        for metric in error_metrics {
            match metric.status {
                MetricStatus::Pass => passed += 1,
                MetricStatus::Fail => failed += 1,
                MetricStatus::Warning => warning += 1,
            }
            items.push(metric);
        }
        
        // 业务指标 (14 个)
        let business_metrics = self.collect_business_metrics().await?;
        for metric in business_metrics {
            match metric.status {
                MetricStatus::Pass => passed += 1,
                MetricStatus::Fail => failed += 1,
                MetricStatus::Warning => warning += 1,
            }
            items.push(metric);
        }
        
        // 系统指标 (8 个)
        let system_metrics = self.collect_system_metrics().await?;
        for metric in system_metrics {
            match metric.status {
                MetricStatus::Pass => passed += 1,
                MetricStatus::Fail => failed += 1,
                MetricStatus::Warning => warning += 1,
            }
            items.push(metric);
        }
        
        // 追踪指标 (5 个)
        let tracing_metrics = self.collect_tracing_metrics().await?;
        for metric in tracing_metrics {
            match metric.status {
                MetricStatus::Pass => passed += 1,
                MetricStatus::Fail => failed += 1,
                MetricStatus::Warning => warning += 1,
            }
            items.push(metric);
        }
        
        let total = items.len();
        
        Ok(MetricsSummary {
            total,
            passed,
            failed,
            warning,
            items,
            by_category: CategoryBreakdown {
                performance: CategoryMetrics { total: 18, passed: 0, failed: 0 },
                consistency: CategoryMetrics { total: 0, passed: 0, failed: 0 },
                security: CategoryMetrics { total: 0, passed: 0, failed: 0 },
                business: CategoryMetrics { total: 14, passed: 0, failed: 0 },
                tracing: CategoryMetrics { total: 5, passed: 0, failed: 0 },
            },
        })
    }
    
    /// 采集性能指标
    async fn collect_performance_metrics(&self) -> Result<Vec<MetricItem>, Box<dyn Error>> {
        let mut metrics = Vec::new();
        
        // M-006: execution_latency_p99
        let exec_p99 = self.query_prometheus(
            "histogram_quantile(0.99, rate(execution_latency_bucket[5m]))"
        ).await?;
        
        metrics.push(MetricItem {
            id: "M-006".to_string(),
            name: "execution_latency_p99".to_string(),
            category: "performance".to_string(),
            value: MetricValue::Number(exec_p99),
            threshold: ">200ms".to_string(),
            status: if exec_p99 < 200.0 { MetricStatus::Pass } else { MetricStatus::Fail },
            source: "Prometheus".to_string(),
            query: "histogram_quantile(0.99, rate(execution_latency_bucket[5m]))".to_string(),
            timestamp: Utc::now(),
            trend: Some(self.calculate_trend("execution_latency_p99").await?),
        });
        
        // M-007: verification_latency_p99
        // ... 类似采集其他指标
        
        Ok(metrics)
    }
    
    /// 采集追踪指标
    async fn collect_tracing_metrics(&self) -> Result<Vec<MetricItem>, Box<dyn Error>> {
        let mut metrics = Vec::new();
        
        // M-025: distributed_trace_coverage
        let coverage = self.query_prometheus(
            "distributed_trace_coverage"
        ).await?;
        
        metrics.push(MetricItem {
            id: "M-025".to_string(),
            name: "distributed_trace_coverage".to_string(),
            category: "tracing".to_string(),
            value: MetricValue::Percentage(coverage),
            threshold: "<98%".to_string(),
            status: if coverage >= 98.0 { MetricStatus::Pass } else { MetricStatus::Fail },
            source: "Prometheus".to_string(),
            query: "distributed_trace_coverage".to_string(),
            timestamp: Utc::now(),
            trend: None,
        });
        
        // M-035: trace_span_duration_p99
        // M-051: trace_total_duration_p99
        // M-052: trace_span_count_avg
        // M-053: trace_propagation_success_rate
        // ... 类似采集
        
        Ok(metrics)
    }
    
    /// 查询 Prometheus
    async fn query_prometheus(&self, query: &str) -> Result<f64, Box<dyn Error>> {
        let url = format!("{}/api/v1/query", self.prometheus_url);
        let response = self.client
            .get(&url)
            .query(&[("query", query)])
            .send()
            .await?;
        
        let json: serde_json::Value = response.json().await?;
        
        if json["status"] == "success" {
            if let Some(value) = json["data"]["result"][0]["value"][1].as_str() {
                return Ok(value.parse()?);
            }
        }
        
        Err("Query failed".into())
    }
    
    /// 采集追踪证据
    async fn collect_tracing_evidence(&self) -> Result<TracingEvidence, Box<dyn Error>> {
        // 查询覆盖率
        let coverage_rate = self.query_prometheus("distributed_trace_coverage").await?;
        
        // 查询传递成功率
        let propagation_rate = self.query_prometheus("trace_propagation_success_rate").await?;
        
        // 获取样本 Trace
        let sample_traces = self.get_sample_traces().await?;
        
        // 检查关键路径覆盖
        let critical_paths = self.check_critical_paths().await?;
        
        Ok(TracingEvidence {
            coverage_rate,
            propagation_success_rate: propagation_rate,
            sample_traces,
            critical_paths_coverage: critical_paths,
        })
    }
    
    /// 获取样本 Trace
    async fn get_sample_traces(&self) -> Result<Vec<SampleTrace>, Box<dyn Error>> {
        let url = format!("{}/api/search", self.tempo_url);
        let response = self.client
            .get(&url)
            .query(&[("limit", "5")])
            .send()
            .await?;
        
        let traces: Vec<SampleTrace> = response.json().await?;
        Ok(traces)
    }
    
    /// 检查关键路径覆盖
    async fn check_critical_paths(&self) -> Result<CriticalPathsCoverage, Box<dyn Error>> {
        let critical_paths = vec![
            "Executor.execute_instruction",
            "Verifier.verify_result",
            "BatchExecutor.execute",
            "TransactionManager.commit",
            "Gateway.handleRequest",
        ];
        
        let mut covered = Vec::new();
        let mut missing = Vec::new();
        
        for path in critical_paths {
            if self.path_exists_in_traces(path).await? {
                covered.push(path.to_string());
            } else {
                missing.push(path.to_string());
            }
        }
        
        Ok(CriticalPathsCoverage {
            total_paths: critical_paths.len(),
            covered_paths: covered.len(),
            missing_paths: missing,
        })
    }
    
    /// 采集日志证据
    async fn collect_log_evidence(&self) -> Result<LogEvidence, Box<dyn Error>> {
        let sample_queries = vec![
            LogQuery {
                description: "错误日志".to_string(),
                query: "{level=\"error\"}".to_string(),
                loki_url: format!("{}/explore?query={}", self.loki_url, "{level=\\\"error\\\"}"),
                sample_count: 100,
            },
            LogQuery {
                description: "Panic 日志".to_string(),
                query: "{level=\"panic\"}".to_string(),
                loki_url: format!("{}/explore?query={}", self.loki_url, "{level=\\\"panic\\\"}"),
                sample_count: 10,
            },
        ];
        
        Ok(LogEvidence {
            collection_rate: 100.0,
            sample_queries,
        })
    }
    
    /// 获取仪表盘链接
    fn get_dashboard_links(&self) -> Vec<DashboardLink> {
        vec![
            DashboardLink {
                title: "Phase 3 性能监控".to_string(),
                grafana_url: format!("{}/d/performance", self.grafana_url),
                uid: "phase3-performance".to_string(),
            },
            DashboardLink {
                title: "Phase 3 一致性监控".to_string(),
                grafana_url: format!("{}/d/consistency", self.grafana_url),
                uid: "phase3-consistency".to_string(),
            },
            DashboardLink {
                title: "Phase 3 安全监控".to_string(),
                grafana_url: format!("{}/d/security", self.grafana_url),
                uid: "phase3-security".to_string(),
            },
            DashboardLink {
                title: "Phase 3 追踪监控".to_string(),
                grafana_url: format!("{}/d/tracing", self.grafana_url),
                uid: "phase3-tracing".to_string(),
            },
        ]
    }
    
    /// 生成决策
    fn generate_decision(&self, metrics: &MetricsSummary, tracing: &TracingEvidence) -> GateDecision {
        // 计算通过率
        let pass_rate = metrics.passed as f64 / metrics.total as f64 * 100.0;
        
        // 检查关键指标
        let has_critical_failure = metrics.items.iter().any(|m| {
            m.status == MetricStatus::Fail && m.id.starts_with("M-00") // 关键指标
        });
        
        // 检查 Trace 覆盖率
        let trace_coverage_ok = tracing.coverage_rate >= 98.0;
        
        if has_critical_failure || !trace_coverage_ok {
            GateDecision::NoGo
        } else if pass_rate >= 95.0 {
            GateDecision::Go
        } else {
            GateDecision::ConditionalGo
        }
    }
    
    /// 生成摘要
    fn generate_summary(&self, metrics: &MetricsSummary, decision: &GateDecision) -> ReportSummary {
        let mut highlights = Vec::new();
        let mut risks = Vec::new();
        let mut recommendations = Vec::new();
        
        // 亮点
        highlights.push(format!("指标通过率：{}/{} ({:.1}%)", metrics.passed, metrics.total, metrics.passed as f64 / metrics.total as f64 * 100.0));
        
        // 风险
        if metrics.failed > 0 {
            risks.push(format!("{} 个指标未达标", metrics.failed));
        }
        
        // 建议
        match decision {
            GateDecision::Go => {
                recommendations.push("建议按计划推进发布".to_string());
            }
            GateDecision::ConditionalGo => {
                recommendations.push("建议修复未达标指标后重新评估".to_string());
            }
            GateDecision::NoGo => {
                recommendations.push("建议暂停发布，优先修复关键问题".to_string());
            }
        }
        
        ReportSummary {
            highlights,
            risks,
            recommendations,
        }
    }
    
    /// 计算趋势
    async fn calculate_trend(&self, metric_name: &str) -> Result<Trend, Box<dyn Error>> {
        // 查询 24 小时前数据
        let query = format!("{} offset 24h", metric_name);
        let past_value = self.query_prometheus(&query).await?;
        
        // 查询当前数据
        let current_value = self.query_prometheus(metric_name).await?;
        
        let change = (current_value - past_value) / past_value * 100.0;
        
        let direction = if change > 5.0 {
            "up".to_string()
        } else if change < -5.0 {
            "down".to_string()
        } else {
            "stable".to_string()
        };
        
        Ok(Trend {
            direction,
            change_percent: change,
            period: "24h".to_string(),
        })
    }
}
```

---

## 4. Markdown 报告生成

### 4.1 模板设计

```rust
// markdown_generator.rs - Markdown 报告生成

use crate::gate_report_phase3::*;
use chrono::Utc;

pub fn generate_markdown_report(report: &Phase3GateReport) -> String {
    let mut md = String::new();
    
    // 标题
    md.push_str(&format!("# {} {} Gate Report\n\n", report.phase.to_uppercase(), report.gate_type));
    md.push_str(&format!("**Release ID**: {}\n\n", report.release_id));
    md.push_str(&format!("**Generated**: {}\n\n", report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
    
    // 决策
    md.push_str("## 🚦 Gate Decision\n\n");
    match report.decision {
        GateDecision::Go => {
            md.push_str("### ✅ Go\n\n");
            md.push_str("所有关键指标达标，建议按计划推进发布。\n\n");
        }
        GateDecision::ConditionalGo => {
            md.push_str("### ⚠️ Conditional Go\n\n");
            md.push_str("部分指标未达标，建议修复后重新评估。\n\n");
        }
        GateDecision::NoGo => {
            md.push_str("### ❌ No-Go\n\n");
            md.push_str("关键指标未达标，建议暂停发布。\n\n");
        }
    }
    
    // 指标汇总
    md.push_str("## 📊 Metrics Summary\n\n");
    md.push_str(&format!(
        "| Total | Passed | Failed | Warning | Pass Rate |\n|---|---|---|---|---|\n| {} | {} | {} | {} | {:.1}% |\n\n",
        report.metrics.total,
        report.metrics.passed,
        report.metrics.failed,
        report.metrics.warning,
        report.metrics.passed as f64 / report.metrics.total as f64 * 100.0
    ));
    
    // 按分类统计
    md.push_str("### By Category\n\n");
    md.push_str("| Category | Total | Passed | Failed |\n|---|---|---|---|\n");
    md.push_str(&format!(
        "| Performance | {} | {} | {} |\n",
        report.metrics.by_category.performance.total,
        report.metrics.by_category.performance.passed,
        report.metrics.by_category.performance.failed
    ));
    // ... 其他分类
    
    md.push_str("\n");
    
    // 关键指标详情
    md.push_str("## 📈 Key Metrics\n\n");
    
    // 性能指标
    md.push_str("### Performance Metrics\n\n");
    md.push_str("| ID | Metric | Value | Threshold | Status | Trend |\n|---|---|---|---|---|---|\n");
    
    for metric in &report.metrics.items {
        if metric.category == "performance" {
            let status_icon = match metric.status {
                MetricStatus::Pass => "✅",
                MetricStatus::Fail => "❌",
                MetricStatus::Warning => "⚠️",
            };
            
            let trend_icon = match &metric.trend {
                Some(t) if t.direction == "up" => "📈",
                Some(t) if t.direction == "down" => "📉",
                _ => "➡️",
            };
            
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} {} | {} |\n",
                metric.id,
                metric.name,
                format_metric_value(&metric.value),
                metric.threshold,
                status_icon,
                match metric.status { MetricStatus::Pass => "Pass", _ => "Fail" },
                trend_icon
            ));
        }
    }
    
    md.push_str("\n");
    
    // 追踪指标
    md.push_str("### Tracing Metrics\n\n");
    md.push_str("| ID | Metric | Value | Threshold | Status |\n|---|---|---|---|---|\n");
    
    for metric in &report.metrics.items {
        if metric.category == "tracing" {
            let status_icon = match metric.status {
                MetricStatus::Pass => "✅",
                MetricStatus::Fail => "❌",
                MetricStatus::Warning => "⚠️",
            };
            
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                metric.id,
                metric.name,
                format_metric_value(&metric.value),
                metric.threshold,
                status_icon
            ));
        }
    }
    
    md.push_str("\n");
    
    // 追踪证据
    md.push_str("## 🔍 Tracing Evidence\n\n");
    md.push_str(&format!("- **Coverage Rate**: {:.1}%\n", report.tracing_evidence.coverage_rate));
    md.push_str(&format!("- **Propagation Success Rate**: {:.1}%\n", report.tracing_evidence.propagation_success_rate));
    md.push_str(&format!(
        "- **Critical Paths**: {}/{}\n\n",
        report.tracing_evidence.critical_paths_coverage.covered_paths,
        report.tracing_evidence.critical_paths_coverage.total_paths
    ));
    
    // 样本 Trace
    if !report.tracing_evidence.sample_traces.is_empty() {
        md.push_str("### Sample Traces\n\n");
        md.push_str("| Trace ID | Duration | Spans | Links |\n|---|---|---|---|\n");
        
        for trace in &report.tracing_evidence.sample_traces {
            md.push_str(&format!(
                "| {} | {}ms | {} | [Tempo]({}) |\n",
                &trace.trace_id[..16],
                trace.duration_ms,
                trace.span_count,
                trace.tempo_url
            ));
        }
        
        md.push_str("\n");
    }
    
    // 仪表盘
    md.push_str("## 📊 Dashboards\n\n");
    for dashboard in &report.dashboards {
        md.push_str(&format!("- [{}]({})\n", dashboard.title, dashboard.grafana_url));
    }
    
    md.push_str("\n");
    
    // 摘要
    md.push_str("## 📝 Summary\n\n");
    
    md.push_str("### Highlights\n\n");
    for highlight in &report.summary.highlights {
        md.push_str(&format!("- {}\n", highlight));
    }
    md.push_str("\n");
    
    if !report.summary.risks.is_empty() {
        md.push_str("### Risks\n\n");
        for risk in &report.summary.risks {
            md.push_str(&format!("- {}\n", risk));
        }
        md.push_str("\n");
    }
    
    md.push_str("### Recommendations\n\n");
    for rec in &report.summary.recommendations {
        md.push_str(&format!("- {}\n", rec));
    }
    md.push_str("\n");
    
    // 脚注
    md.push_str("---\n\n");
    md.push_str(&format!("*Generated automatically by Gate-Report Generator at {}*\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    md
}

fn format_metric_value(value: &MetricValue) -> String {
    match value {
        MetricValue::Number(n) => format!("{:.2}", n),
        MetricValue::String(s) => s.clone(),
        MetricValue::Percentage(p) => format!("{:.1}%", p),
        MetricValue::Duration(d) => format!("{}ms", d),
    }
}
```

### 4.2 示例输出

```markdown
# PHASE3 Exit Gate Report

**Release ID**: release-2026-05-12-phase3_week01

**Generated**: 2026-05-12 08:00:00 UTC

## 🚦 Gate Decision

### ✅ Go

所有关键指标达标，建议按计划推进发布。

## 📊 Metrics Summary

| Total | Passed | Failed | Warning | Pass Rate |
|---|---|---|---|---|
| 50 | 48 | 0 | 2 | 96.0% |

### By Category

| Category | Total | Passed | Failed |
|---|---|---|---|
| Performance | 18 | 18 | 0 |
| Error | 10 | 10 | 0 |
| Business | 14 | 13 | 0 |
| System | 8 | 7 | 0 |
| Tracing | 5 | 5 | 0 |

## 📈 Key Metrics

### Performance Metrics

| ID | Metric | Value | Threshold | Status | Trend |
|---|---|---|---|---|---|
| M-006 | execution_latency_p99 | 185.2ms | >200ms | ✅ Pass | 📉 |
| M-007 | verification_latency_p99 | 178.5ms | >200ms | ✅ Pass | ➡️ |
| M-016 | batch_execute_latency_p99 | 285.3ms | >300ms | ✅ Pass | 📉 |
| M-019 | transaction_commit_latency_p99 | 292.1ms | >300ms | ✅ Pass | ➡️ |

### Tracing Metrics

| ID | Metric | Value | Threshold | Status |
|---|---|---|---|---|
| M-025 | distributed_trace_coverage | 99.2% | <98% | ✅ Pass |
| M-035 | trace_span_duration_p99 | 425.8ms | >500ms | ✅ Pass |
| M-051 | trace_total_duration_p99 | 892.3ms | >1000ms | ✅ Pass |
| M-052 | trace_span_count_avg | 12.5 | - | ✅ Pass |
| M-053 | trace_propagation_success_rate | 99.5% | <99% | ✅ Pass |

## 🔍 Tracing Evidence

- **Coverage Rate**: 99.2%
- **Propagation Success Rate**: 99.5%
- **Critical Paths**: 5/5

### Sample Traces

| Trace ID | Duration | Spans | Links |
|---|---|---|---|
| abc123def456... | 892ms | 16 | [Tempo](http://tempo:3200/trace/abc123) |
| def789ghi012... | 756ms | 14 | [Tempo](http://tempo:3200/trace/def789) |
| jkl345mno678... | 1023ms | 18 | [Tempo](http://tempo:3200/trace/jkl345) |

## 📊 Dashboards

- [Phase 3 性能监控](http://grafana:3000/d/performance)
- [Phase 3 一致性监控](http://grafana:3000/d/consistency)
- [Phase 3 安全监控](http://grafana:3000/d/security)
- [Phase 3 追踪监控](http://grafana:3000/d/tracing)

## 📝 Summary

### Highlights

- 指标通过率：48/50 (96.0%)
- Trace 覆盖率：99.2% (≥99% 目标)
- 全链路时长 P99: 892ms (<1000ms 目标)

### Recommendations

- 建议按计划推进发布

---

*Generated automatically by Gate-Report Generator at 2026-05-12 08:05:00 UTC*
```

---

## 5. 部署与配置

### 5.1 CLI 工具

```rust
// main.rs - Gate-Report CLI

use clap::{Parser, Subcommand};
use gate_report_generator::GateReportGenerator;

#[derive(Parser)]
#[command(name = "gate-report")]
#[command(about = "Phase 3 Gate-Report 自动生成工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 生成 Gate-Report
    Generate {
        #[arg(short, long)]
        release_id: String,
        
        #[arg(short, long, default_value = "phase3")]
        phase: String,
        
        #[arg(short, long, default_value = "exit")]
        gate_type: String,
        
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },
    
    /// 验证 Schema
    Validate {
        #[arg(short, long)]
        input: String,
    },
    
    /// 定时生成 (Cron 模式)
    Cron {
        #[arg(short, long, default_value = "0 8 * * *")]
        schedule: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    let generator = GateReportGenerator::new(
        std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| "http://localhost:9090".to_string()),
        std::env::var("TEMPO_URL").unwrap_or_else(|_| "http://localhost:3200".to_string()),
        std::env::var("LOKI_URL").unwrap_or_else(|_| "http://localhost:3100".to_string()),
        std::env::var("GRAFANA_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
    );
    
    match cli.command {
        Commands::Generate { release_id, phase, gate_type, format } => {
            let gate_type = match gate_type.as_str() {
                "entry" => GateType::Entry,
                "midterm" => GateType::Midterm,
                "exit" => GateType::Exit,
                _ => GateType::Exit,
            };
            
            let report = generator.generate_report(&release_id, &phase, gate_type).await?;
            
            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&report)?;
                    println!("{}", json);
                }
                "markdown" => {
                    let md = markdown_generator::generate_markdown_report(&report);
                    println!("{}", md);
                }
                _ => {
                    eprintln!("Unknown format: {}", format);
                }
            }
        }
        
        Commands::Validate { input } => {
            // Schema 验证逻辑
            println!("Validating {}", input);
        }
        
        Commands::Cron { schedule } => {
            // Cron 定时生成逻辑
            println!("Starting cron with schedule: {}", schedule);
        }
    }
    
    Ok(())
}
```

### 5.2 Docker 部署

```dockerfile
# Dockerfile.gate-report

FROM rust:1.77 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/gate-report /usr/local/bin/

ENTRYPOINT ["gate-report"]
```

### 5.3 Kubernetes CronJob

```yaml
# k8s/gate-report-cronjob.yaml

apiVersion: batch/v1
kind: CronJob
metadata:
  name: gate-report-generator
  namespace: cgas
spec:
  schedule: "0 8 * * *"  # 每天 8:00 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: gate-report
            image: cgas/gate-report:v1.0
            args: ["generate", "--release-id", "release-2026-05-12", "--phase", "phase3", "--gate-type", "exit"]
            env:
            - name: PROMETHEUS_URL
              value: "http://prometheus:9090"
            - name: TEMPO_URL
              value: "http://tempo:3200"
            - name: LOKI_URL
              value: "http://loki:3100"
            - name: GRAFANA_URL
              value: "http://grafana:3000"
          restartPolicy: OnFailure
```

---

## 6. 验收标准

### 6.1 功能验收

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 报告生成时间 | <5 分钟 | 计时测试 | 从触发到完成<5 分钟 |
| 指标覆盖率 | 100% | 检查 50 个指标 | 所有指标都有数据 |
| 数据准确性 | 误差<1% | 与源数据比对 | 随机抽样 10 个指标 |
| Schema 校验 | 100% 通过 | JSON Schema 验证 | 无校验错误 |
| Markdown 渲染 | 无格式错误 | 人工检查 | 格式正确 |

### 6.2 性能验收

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| Prometheus 查询延迟 | <10s | 查询 50 个指标 | 总查询时间<10s |
| Tempo 查询延迟 | <5s | 查询 Trace | 单次查询<5s |
| 报告生成延迟 | <2s | JSON→Markdown 转换 | 转换时间<2s |

---

## 7. 实施计划

| 周次 | 任务 | 责任人 | 状态 | 交付物 |
|---|---|---|---|---|
| Week 2-T1 | Schema 设计与评审 | Observability | 📋 待开始 | gate_report_schema_v2.json |
| Week 2-T2 | Rust 实现 | Dev+Observability | 📋 待开始 | gate_report_generator.rs |
| Week 2-T3 | Markdown 模板 | Observability | 📋 待开始 | markdown_generator.rs |
| Week 3-T1 | CLI 工具 | Dev | 📋 待开始 | gate-report CLI |
| Week 3-T2 | 集成测试 | QA | 📋 待开始 | test_report.md |
| Week 3-T3 | 部署配置 | SRE | 📋 待开始 | k8s/gate-report-cronjob.yaml |

---

## 8. 附录

### 8.1 参考文档

| 文档 | 链接 |
|---|---|
| JSON Schema | https://json-schema.org/ |
| Prometheus Query API | https://prometheus.io/docs/prometheus/latest/querying/api/ |
| Tempo Search API | https://grafana.com/docs/tempo/latest/api_docs/ |

### 8.2 相关文档

| 文档 | 路径 |
|---|---|
| Phase 3 50 指标规划 | phase3_50_metrics_plan.md |
| 分布式追踪设计 | distributed_tracing.md |
| Gate-Report Schema | gate_report_schema.json |

---

**文档状态**: 📋 设计中  
**创建日期**: 2026-05-12  
**责任人**: Observability-Agent  
**保管**: 项目文档库
