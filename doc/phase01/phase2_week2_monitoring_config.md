# Phase 2 Week 2 监控指标配置

**版本**: v1.0  
**日期**: 2026-04-07  
**责任人**: SRE  
**状态**: 📋 配置中  
**release_id**: release-2026-04-07-phase2_week02  

---

## 1. 概述

### 1.1 Week 2 监控目标

本周完成 5 个新增监控指标接入，扩展 Phase 1 15 个指标至 20 个。

| 类别 | Phase 1 | Week 2 新增 | Week 2 总计 |
|---|---|---|---|
| 性能指标 | 5 | 3 | 8 |
| 一致性指标 | 3 | 0 | 3 |
| 安全指标 | 4 | 0 | 4 |
| 业务指标 | 3 | 2 | 5 |
| **总计** | **15** | **5** | **20** |

### 1.2 新增指标清单

| 指标 ID | 指标名 | 类型 | 来源 | 优先级 |
|---|---|---|---|---|
| METRIC-2-001 | batch_execute_latency_p99 | Histogram | Batch 服务 | P0 |
| METRIC-2-002 | batch_atomicity_violation_count | Counter | Batch 服务 | P0 |
| METRIC-2-003 | batch_sub_instruction_count | Histogram | Batch 服务 | P1 |
| METRIC-2-004 | instruction_type_distribution | Histogram | 执行器 | P1 |
| METRIC-2-005 | client_version_distribution | Histogram | Gateway | P1 |

---

## 2. Prometheus 配置

### 2.1 指标定义

```yaml
# prometheus-config.yml (Week 2 新增)

# Batch 执行时延 (Histogram)
- metric: batch_execute_latency_p99
  type: Histogram
  buckets: [0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 0.75, 1.0, 2.5, 5.0, 10.0]
  labels: [service, environment, batch_size_range]
  description: "Batch execute latency P99 in seconds"
  alert_threshold: ">0.4"  # 400ms

# Batch 原子性违反次数 (Counter)
- metric: batch_atomicity_violation_count
  type: Counter
  labels: [service, environment, reason]
  description: "Count of batch atomicity violations"
  alert_threshold: ">0"  # 任何违反都告警

# Batch 子指令数量 (Histogram)
- metric: batch_sub_instruction_count
  type: Histogram
  buckets: [1, 5, 10, 20, 50, 75, 100]
  labels: [service, environment, status]
  description: "Distribution of batch sub-instruction counts"

# 指令类型分布 (Histogram)
- metric: instruction_type_distribution
  type: Histogram
  labels: [service, environment, instruction_type]
  description: "Distribution of instruction types"

# 客户端版本分布 (Histogram)
- metric: client_version_distribution
  type: Histogram
  labels: [service, environment, client_version]
  description: "Distribution of client versions"
```

### 2.2 告警规则

```yaml
# alerting-rules.yml (Week 2 新增)

groups:
  - name: phase2_batch_alerts
    interval: 30s
    rules:
      # Batch 执行时延告警
      - alert: BatchExecuteLatencyHigh
        expr: histogram_quantile(0.99, rate(batch_execute_latency_p99_bucket[5m])) > 0.4
        for: 5m
        labels:
          severity: P1
        annotations:
          summary: "Batch 执行时延过高 (P99 > 400ms)"
          description: "Batch 执行 P99 时延 {{ $value }}s 超过阈值 400ms"
      
      # Batch 原子性违反告警
      - alert: BatchAtomicityViolation
        expr: increase(batch_atomicity_violation_count[5m]) > 0
        for: 1m
        labels:
          severity: P0
        annotations:
          summary: "Batch 原子性违反"
          description: "检测到 Batch 原子性违反，原因：{{ $labels.reason }}"
```

---

## 3. Grafana 仪表盘

### 3.1 Batch 仪表盘

**Dashboard ID**: phase2-batch-overview  
**刷新间隔**: 30s  
**时间范围**: 最近 1 小时

#### Panel 1: Batch 执行时延趋势

```json
{
  "title": "Batch Execute Latency (P50/P90/P99)",
  "type": "graph",
  "targets": [
    {
      "expr": "histogram_quantile(0.50, rate(batch_execute_latency_p99_bucket[5m]))",
      "legendFormat": "P50"
    },
    {
      "expr": "histogram_quantile(0.90, rate(batch_execute_latency_p99_bucket[5m]))",
      "legendFormat": "P90"
    },
    {
      "expr": "histogram_quantile(0.99, rate(batch_execute_latency_p99_bucket[5m]))",
      "legendFormat": "P99"
    }
  ],
  "thresholds": [
    {"value": 0.4, "color": "yellow"},
    {"value": 0.5, "color": "red"}
  ]
}
```

#### Panel 2: Batch 原子性违反

```json
{
  "title": "Batch Atomicity Violations",
  "type": "stat",
  "targets": [
    {
      "expr": "sum(increase(batch_atomicity_violation_count[1h]))",
      "legendFormat": "Violations (1h)"
    }
  ],
  "thresholds": [
    {"value": 0, "color": "green"},
    {"value": 1, "color": "red"}
  ]
}
```

#### Panel 3: Batch 子指令数量分布

```json
{
  "title": "Batch Sub-Instruction Count Distribution",
  "type": "heatmap",
  "targets": [
    {
      "expr": "rate(batch_sub_instruction_count_bucket[5m])",
      "format": "heatmap"
    }
  ]
}
```

#### Panel 4: Batch 请求量

```json
{
  "title": "Batch Request Rate",
  "type": "graph",
  "targets": [
    {
      "expr": "rate(batch_execute_latency_p99_count[5m])",
      "legendFormat": "Requests/s"
    }
  ]
}
```

---

## 4. 指标采集实现

### 4.1 Rust 代码集成

```rust
// metrics.rs (Week 2 新增)

use prometheus::{Histogram, Counter, HistogramOpts, Opts, register_histogram, register_counter};

// Batch 执行时延 Histogram
lazy_static! {
    pub static ref BATCH_EXECUTE_LATENCY: Histogram = register_histogram!(
        HistogramOpts::new(
            "batch_execute_latency_p99",
            "Batch execute latency in seconds"
        )
        .buckets(vec![0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 0.75, 1.0, 2.5, 5.0, 10.0])
    ).unwrap();
    
    pub static ref BATCH_ATOMICITY_VIOLATION: Counter = register_counter!(
        Opts::new(
            "batch_atomicity_violation_count",
            "Count of batch atomicity violations"
        )
    ).unwrap();
    
    pub static ref BATCH_SUB_INSTRUCTION_COUNT: Histogram = register_histogram!(
        HistogramOpts::new(
            "batch_sub_instruction_count",
            "Distribution of batch sub-instruction counts"
        )
        .buckets(vec![1.0, 5.0, 10.0, 20.0, 50.0, 75.0, 100.0])
    ).unwrap();
}

// 指标采集点
pub fn observe_batch_execute(latency_secs: f64, sub_instruction_count: usize) {
    BATCH_EXECUTE_LATENCY.observe(latency_secs);
    BATCH_SUB_INSTRUCTION_COUNT.observe(sub_instruction_count as f64);
}

pub fn inc_batch_atomicity_violation(reason: &str) {
    BATCH_ATOMICITY_VIOLATION.inc();
    // 记录日志用于根因分析
    log::warn!("Batch atomicity violation detected, reason: {}", reason);
}
```

### 4.2 采集点位置

| 采集点 | 位置 | 指标 | 触发条件 |
|---|---|---|---|
| Batch 执行完成 | batch_executor.rs | batch_execute_latency_p99 | 每次 Batch 执行 |
| Batch 执行完成 | batch_executor.rs | batch_sub_instruction_count | 每次 Batch 执行 |
| 原子性违反 | batch_executor.rs | batch_atomicity_violation_count | atomic=true 且部分失败 |
| 指令执行 | executor.rs | instruction_type_distribution | 每次指令执行 |
| 请求入口 | gateway.rs | client_version_distribution | 每次请求 |

---

## 5. 监控接入计划

### 5.1 Week 2 接入时间表

| 时间 | 任务 | 责任人 | 状态 |
|---|---|---|---|
| Week 2-T1 | Prometheus 指标定义 | SRE | 📋 待开始 |
| Week 2-T2 | Rust 代码指标采集集成 | Dev | 📋 待开始 |
| Week 2-T3 | Grafana 仪表盘配置 | SRE | 📋 待开始 |
| Week 2-T4 | 告警规则配置 + 测试 | SRE | 📋 待开始 |
| Week 2-T5 | 指标验证 + 文档 | SRE | 📋 待开始 |

### 5.2 验证标准

| 验证项 | 标准 | 验证方法 |
|---|---|---|
| 指标采集 | 5 个指标均有数据 | Prometheus 查询验证 |
| 告警触发 | 阈值正确触发 | 模拟告警测试 |
| 仪表盘显示 | 4 个 Panel 正常显示 | Grafana 检查 |
| 数据准确性 | 与日志一致 | 抽样比对 |

---

## 6. Phase 2 监控扩展路线图

### 6.1 Week 1-6 监控扩展

| 周次 | 新增指标 | 累计指标 | 重点 |
|---|---|---|---|
| Week 1 | 0 | 15 | Phase 1 继承 |
| Week 2 | 5 | 20 | Batch 监控 |
| Week 3 | 5 | 25 | Transaction 监控 + 零信任 |
| Week 4 | 0 | 25 | 性能优化监控 |
| Week 5 | 0 | 25 | 稳定性监控 |
| Week 6 | 0 | 25 | Exit Gate 验证 |

### 6.2 Week 3 预告 (Transaction 监控)

| 指标 ID | 指标名 | 类型 | 计划周次 |
|---|---|---|---|
| METRIC-3-001 | transaction_commit_latency_p99 | Histogram | Week 3 |
| METRIC-3-002 | transaction_rollback_count | Counter | Week 3 |
| METRIC-3-003 | transaction_timeout_count | Counter | Week 3 |
| METRIC-3-004 | zero_trust_auth_failure_count | Counter | Week 3 |
| METRIC-3-005 | zero_trust_policy_violation_count | Counter | Week 3 |

---

## 7. 附录

### 7.1 Prometheus 配置完整示例

```yaml
# prometheus.yml (完整配置)

global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'cgas-executor'
    static_configs:
      - targets: ['executor:8080']
    metrics_path: '/metrics'
    
  - job_name: 'cgas-verifier'
    static_configs:
      - targets: ['verifier:8081']
    metrics_path: '/metrics'
    
  - job_name: 'cgas-batch'
    static_configs:
      - targets: ['batch:8082']
    metrics_path: '/metrics'
```

### 7.2 相关文档

- Phase 2 SRE 规划 v1
- Phase 2 Batch 设计文档
- Phase 1 监控指标配置清单

---

**文档状态**: 📋 配置中  
**责任人**: SRE  
**保管**: 项目文档库
