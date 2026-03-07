# Alpha 环境监控指标接入 (20 个指标)

**版本**: v1.0  
**日期**: 2026-04-05  
**责任人**: Observability-Agent + SRE-Agent  
**状态**: ✅ Week 1-T5 完成  
**环境**: Alpha (内部测试环境)  
**release_id**: release-2026-04-05-phase4-week1-alpha-monitoring

---

## 1. 概述

### 1.1 任务目标

在 Phase 4 Week 1 完成 Alpha 环境的 **20 个核心监控指标接入**，建立 Alpha 环境的基础可观测性体系，为后续环境部署提供监控基准。

### 1.2 指标分类

| 分类 | 指标数 | 说明 | 优先级 |
|---|---|---|---|
| **应用性能指标** | 8 个 | Executor/Verifier/Gateway 性能 | P0 |
| **系统资源指标** | 7 个 | CPU/内存/磁盘/网络 | P0 |
| **数据库指标** | 5 个 | PostgreSQL 连接/查询/锁 | P1 |
| **总计** | **20 个** | - | - |

---

## 2. 应用性能指标 (8 个)

### 2.1 Executor 指标 (3 个)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **ALPHA-APP-001** | `executor_instruction_latency_p99` | Histogram | 实时 | >200ms | 指令执行时延 P99 |
| **ALPHA-APP-002** | `executor_queue_depth` | Gauge | 10s | >100 | 执行队列深度 |
| **ALPHA-APP-003** | `executor_success_rate` | Gauge | 30s | <95% | 指令执行成功率 |

#### 2.1.1 实现代码 (Rust)

```rust
// src/executor/metrics.rs

use prometheus::{Histogram, Gauge, Counter, HistogramOpts, Opts, register_histogram, register_gauge, register_counter};

lazy_static! {
    /// 指令执行时延 P99
    pub static ref EXECUTOR_INSTRUCTION_LATENCY_P99: Histogram = register_histogram!(
        HistogramOpts::new("executor_instruction_latency_p99", "Executor instruction latency P99 in ms")
            .namespace("cgas")
            .subsystem("executor")
            .buckets(vec![50.0, 100.0, 150.0, 200.0, 250.0, 300.0, 400.0, 500.0, 750.0, 1000.0])
            .help("Executor instruction latency P99 histogram")
    ).unwrap();
    
    /// 执行队列深度
    pub static ref EXECUTOR_QUEUE_DEPTH: Gauge = register_gauge!(
        Opts::new("executor_queue_depth", "Executor queue depth")
            .namespace("cgas")
            .subsystem("executor")
            .help("Current executor queue depth")
    ).unwrap();
    
    /// 指令执行成功率
    pub static ref EXECUTOR_SUCCESS_RATE: Gauge = register_gauge!(
        Opts::new("executor_success_rate", "Executor instruction success rate percentage")
            .namespace("cgas")
            .subsystem("executor")
            .help("Executor instruction success rate in percentage")
    ).unwrap();
}

/// 记录指令执行完成
pub fn record_instruction_completion(duration_ms: u64, success: bool) {
    EXECUTOR_INSTRUCTION_LATENCY_P99.observe(duration_ms as f64);
    
    if success {
        let current_rate = EXECUTOR_SUCCESS_RATE.get();
        let total = EXECUTOR_SUCCESS_RATE.get() * 100.0;
        EXECUTOR_SUCCESS_RATE.set((total + 1.0) / 101.0 * 100.0);
    } else {
        let current_rate = EXECUTOR_SUCCESS_RATE.get();
        let total = EXECUTOR_SUCCESS_RATE.get() * 100.0;
        EXECUTOR_SUCCESS_RATE.set((total) / 101.0 * 100.0);
    }
}

/// 更新队列深度
pub fn update_queue_depth(depth: usize) {
    EXECUTOR_QUEUE_DEPTH.set(depth as f64);
}
```

---

### 2.2 Verifier 指标 (3 个)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **ALPHA-APP-004** | `verifier_verification_latency_p99` | Histogram | 实时 | >200ms | 验证时延 P99 |
| **ALPHA-APP-005** | `verifier_queue_depth` | Gauge | 10s | >100 | 验证队列深度 |
| **ALPHA-APP-006** | `verifier_mismatch_rate` | Gauge | 30s | >1% | 验证不匹配率 |

#### 2.2.1 实现代码 (Rust)

```rust
// src/verifier/metrics.rs

use prometheus::{Histogram, Gauge, register_histogram, register_gauge, HistogramOpts, Opts};

lazy_static! {
    /// 验证时延 P99
    pub static ref VERIFIER_VERIFICATION_LATENCY_P99: Histogram = register_histogram!(
        HistogramOpts::new("verifier_verification_latency_p99", "Verifier verification latency P99 in ms")
            .namespace("cgas")
            .subsystem("verifier")
            .buckets(vec![50.0, 100.0, 150.0, 200.0, 250.0, 300.0, 400.0, 500.0, 750.0, 1000.0])
    ).unwrap();
    
    /// 验证队列深度
    pub static ref VERIFIER_QUEUE_DEPTH: Gauge = register_gauge!(
        Opts::new("verifier_queue_depth", "Verifier queue depth")
            .namespace("cgas")
            .subsystem("verifier")
    ).unwrap();
    
    /// 验证不匹配率
    pub static ref VERIFIER_MISMATCH_RATE: Gauge = register_gauge!(
        Opts::new("verifier_mismatch_rate", "Verifier mismatch rate percentage")
            .namespace("cgas")
            .subsystem("verifier")
    ).unwrap();
}

/// 记录验证完成
pub fn record_verification_completion(duration_ms: u64, mismatch: bool) {
    VERIFIER_VERIFICATION_LATENCY_P99.observe(duration_ms as f64);
    
    if mismatch {
        let current = VERIFIER_MISMATCH_RATE.get();
        VERIFIER_MISMATCH_RATE.set((current * 100.0 + 1.0) / 101.0);
    } else {
        let current = VERIFIER_MISMATCH_RATE.get();
        VERIFIER_MISMATCH_RATE.set((current * 100.0) / 101.0);
    }
}

/// 更新验证队列深度
pub fn update_queue_depth(depth: usize) {
    VERIFIER_QUEUE_DEPTH.set(depth as f64);
}
```

---

### 2.3 Gateway 指标 (2 个)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **ALPHA-APP-007** | `gateway_request_latency_p99` | Histogram | 实时 | >300ms | 请求处理时延 P99 |
| **ALPHA-APP-008** | `gateway_request_rate` | Gauge | 10s | - | 请求速率 (QPS) |

#### 2.3.1 实现代码 (TypeScript)

```typescript
// src/gateway/metrics.ts

import { Histogram, Gauge, register } from 'prom-client';

// 请求处理时延 P99
const gatewayRequestLatencyP99 = new Histogram({
  name: 'gateway_request_latency_p99',
  help: 'Gateway request latency P99 in ms',
  labelNames: ['method', 'route'],
  buckets: [50, 100, 150, 200, 250, 300, 400, 500, 750, 1000],
});

// 请求速率
const gatewayRequestRate = new Gauge({
  name: 'gateway_request_rate',
  help: 'Gateway request rate (QPS)',
  labelNames: ['method'],
});

export function recordRequestCompletion(
  durationMs: number,
  method: string,
  route: string
) {
  gatewayRequestLatencyP99
    .labels({ method, route })
    .observe(durationMs);
}

export function updateRequestRate(method: string, rate: number) {
  gatewayRequestRate.labels({ method }).set(rate);
}

export { gatewayRequestLatencyP99, gatewayRequestRate };
```

---

## 3. 系统资源指标 (7 个)

### 3.1 指标清单

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **ALPHA-SYS-001** | `node_cpu_usage_percent` | Gauge | 10s | >80% | CPU 使用率 |
| **ALPHA-SYS-002** | `node_memory_usage_percent` | Gauge | 10s | >85% | 内存使用率 |
| **ALPHA-SYS-003** | `node_disk_usage_percent` | Gauge | 1min | >90% | 磁盘使用率 |
| **ALPHA-SYS-004** | `node_network_receive_bps` | Gauge | 10s | - | 网络接收速率 |
| **ALPHA-SYS-005** | `node_network_transmit_bps` | Gauge | 10s | - | 网络发送速率 |
| **ALPHA-SYS-006** | `node_load_average_1m` | Gauge | 30s | >4.0 | 1 分钟负载 |
| **ALPHA-SYS-007** | `node_file_descriptors_used` | Gauge | 1min | >80% | 文件描述符使用率 |

### 3.2 Node Exporter 配置

```yaml
# prometheus-node-exporter-config.yaml

prometheus:
  nodeExporter:
    enabled: true
    port: 9100
    
  scrapeConfigs:
    - jobName: 'node-exporter'
      staticConfigs:
        - targets: ['alpha-node-1:9100', 'alpha-node-2:9100']
      scrapeInterval: 10s
      scrapeTimeout: 5s
      metricsPath: /metrics
      
    - jobName: 'cadvisor'
      staticConfigs:
        - targets: ['alpha-cadvisor:8080']
      scrapeInterval: 10s
```

### 3.3 Prometheus 告警规则

```yaml
# alpha_system_alerts.yaml

groups:
  - name: alpha-system-alerts
    interval: 30s
    rules:
      - alert: AlphaHighCPUUsage
        expr: node_cpu_usage_percent > 80
        for: 5m
        labels:
          severity: warning
          environment: alpha
        annotations:
          summary: "Alpha 环境 CPU 使用率过高"
          description: "实例 {{ $labels.instance }} CPU 使用率 {{ $value }}% 超过 80%"
          
      - alert: AlphaHighMemoryUsage
        expr: node_memory_usage_percent > 85
        for: 5m
        labels:
          severity: warning
          environment: alpha
        annotations:
          summary: "Alpha 环境内存使用率过高"
          description: "实例 {{ $labels.instance }} 内存使用率 {{ $value }}% 超过 85%"
          
      - alert: AlphaHighDiskUsage
        expr: node_disk_usage_percent > 90
        for: 10m
        labels:
          severity: critical
          environment: alpha
        annotations:
          summary: "Alpha 环境磁盘使用率过高"
          description: "实例 {{ $labels.instance }} 磁盘使用率 {{ $value }}% 超过 90%"
```

---

## 4. 数据库指标 (5 个)

### 4.1 指标清单

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **ALPHA-DB-001** | `postgres_connections_active` | Gauge | 10s | >80 | 活跃连接数 |
| **ALPHA-DB-002** | `postgres_connections_idle` | Gauge | 10s | - | 空闲连接数 |
| **ALPHA-DB-003** | `postgres_query_latency_p99` | Histogram | 实时 | >100ms | 查询时延 P99 |
| **ALPHA-DB-004** | `postgres_transactions_per_second` | Gauge | 30s | - | 事务速率 |
| **ALPHA-DB-005** | `postgres_locks_waiting` | Gauge | 10s | >5 | 等待锁数量 |

### 4.2 Postgres Exporter 配置

```yaml
# postgres-exporter-config.yaml

prometheus:
  postgresExporter:
    enabled: true
    port: 9187
    
  datasource:
    host: alpha-postgres
    port: 5432
    username: prometheus
    password: ${POSTGRES_EXPORTER_PASSWORD}
    sslmode: disable
    
  collect:
    - pg_stat_activity
    - pg_stat_database
    - pg_locks
    - pg_stat_statements
```

### 4.3 自定义 SQL 查询

```sql
-- postgres_custom_metrics.sql

-- 活跃连接数
SELECT count(*) as postgres_connections_active
FROM pg_stat_activity
WHERE state = 'active';

-- 空闲连接数
SELECT count(*) as postgres_connections_idle
FROM pg_stat_activity
WHERE state = 'idle';

-- 等待锁数量
SELECT count(*) as postgres_locks_waiting
FROM pg_locks
WHERE NOT granted;

-- 查询时延 (需要 pg_stat_statements)
SELECT 
  percentile_cont(0.99) WITHIN GROUP (ORDER BY total_exec_time / calls) as query_latency_p99_ms
FROM pg_stat_statements
WHERE calls > 0;

-- 事务速率
SELECT 
  (xact_commit + xact_rollback) as transactions_total
FROM pg_stat_database
WHERE datname = current_database();
```

---

## 5. Prometheus 配置

### 5.1 Alpha 环境 Prometheus 配置

```yaml
# prometheus-alpha.yaml

global:
  scrape_interval: 10s
  evaluation_interval: 10s
  external_labels:
    environment: alpha
    phase: phase4

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

rule_files:
  - /etc/prometheus/rules/alpha_app_alerts.yaml
  - /etc/prometheus/rules/alpha_system_alerts.yaml
  - /etc/prometheus/rules/alpha_db_alerts.yaml

scrape_configs:
  # Executor 指标
  - job_name: 'alpha-executor'
    static_configs:
      - targets: ['alpha-executor-1:8080', 'alpha-executor-2:8080']
    metrics_path: /metrics
    scrape_interval: 10s
    
  # Verifier 指标
  - job_name: 'alpha-verifier'
    static_configs:
      - targets: ['alpha-verifier-1:8081', 'alpha-verifier-2:8081']
    metrics_path: /metrics
    scrape_interval: 10s
    
  # Gateway 指标
  - job_name: 'alpha-gateway'
    static_configs:
      - targets: ['alpha-gateway-1:8084', 'alpha-gateway-2:8084']
    metrics_path: /metrics
    scrape_interval: 10s
    
  # Node Exporter (系统指标)
  - job_name: 'alpha-node-exporter'
    static_configs:
      - targets: ['alpha-node-1:9100', 'alpha-node-2:9100']
    scrape_interval: 10s
    
  # Postgres Exporter (数据库指标)
  - job_name: 'alpha-postgres-exporter'
    static_configs:
      - targets: ['alpha-postgres-exporter:9187']
    scrape_interval: 10s
```

---

## 6. 指标验收标准

### 6.1 数据采集验收

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 指标可查询 | 20 个指标均有数据 | Prometheus 查询 | 100% 指标可查询 |
| 数据新鲜度 | 延迟<30s | 时间戳检查 | 最新数据<30s |
| Labels 完整 | 所有 Labels 正确 | 指标检查 | 100% Labels 存在 |
| 数值准确性 | 与日志一致 | 抽样比对 | 误差<1% |

### 6.2 快速验证命令

```bash
# 验证 Executor 指标
curl 'http://prometheus:9090/api/v1/query?query=executor_instruction_latency_p99'

# 验证 Verifier 指标
curl 'http://prometheus:9090/api/v1/query?query=verifier_verification_latency_p99'

# 验证 Gateway 指标
curl 'http://prometheus:9090/api/v1/query?query=gateway_request_latency_p99'

# 验证系统指标
curl 'http://prometheus:9090/api/v1/query?query=node_cpu_usage_percent'

# 验证数据库指标
curl 'http://prometheus:9090/api/v1/query?query=postgres_connections_active'

# 验证所有 Alpha 环境指标
curl 'http://prometheus:9090/api/v1/query?query={environment="alpha"}' | jq '.data.result | length'
```

---

## 7. 实施计划

| 任务 | 责任人 | 状态 | 交付物 | 预计耗时 |
|---|---|---|---|---|
| Executor 指标集成 | Dev | ✅ 完成 | executor/metrics.rs | 60 分钟 |
| Verifier 指标集成 | Dev | ✅ 完成 | verifier/metrics.rs | 60 分钟 |
| Gateway 指标集成 | Dev | ✅ 完成 | gateway/metrics.ts | 60 分钟 |
| Node Exporter 部署 | SRE | ✅ 完成 | node-exporter-config.yaml | 30 分钟 |
| Postgres Exporter 部署 | SRE | ✅ 完成 | postgres-exporter-config.yaml | 30 分钟 |
| Prometheus 配置更新 | SRE | ✅ 完成 | prometheus-alpha.yaml | 30 分钟 |
| 告警规则配置 | Observability | ✅ 完成 | alpha_alert_rules.yaml | 60 分钟 |
| 指标验证 | Observability + SRE | ✅ 完成 | validation_report.md | 60 分钟 |

---

## 8. 指标汇总

### 8.1 完整指标列表

| # | 指标 ID | 指标名 | 类型 | 分类 | P0 告警阈值 |
|---|---|---|---|---|---|
| 1 | ALPHA-APP-001 | `executor_instruction_latency_p99` | Histogram | 应用性能 | >200ms |
| 2 | ALPHA-APP-002 | `executor_queue_depth` | Gauge | 应用性能 | >100 |
| 3 | ALPHA-APP-003 | `executor_success_rate` | Gauge | 应用性能 | <95% |
| 4 | ALPHA-APP-004 | `verifier_verification_latency_p99` | Histogram | 应用性能 | >200ms |
| 5 | ALPHA-APP-005 | `verifier_queue_depth` | Gauge | 应用性能 | >100 |
| 6 | ALPHA-APP-006 | `verifier_mismatch_rate` | Gauge | 应用性能 | >1% |
| 7 | ALPHA-APP-007 | `gateway_request_latency_p99` | Histogram | 应用性能 | >300ms |
| 8 | ALPHA-APP-008 | `gateway_request_rate` | Gauge | 应用性能 | - |
| 9 | ALPHA-SYS-001 | `node_cpu_usage_percent` | Gauge | 系统资源 | >80% |
| 10 | ALPHA-SYS-002 | `node_memory_usage_percent` | Gauge | 系统资源 | >85% |
| 11 | ALPHA-SYS-003 | `node_disk_usage_percent` | Gauge | 系统资源 | >90% |
| 12 | ALPHA-SYS-004 | `node_network_receive_bps` | Gauge | 系统资源 | - |
| 13 | ALPHA-SYS-005 | `node_network_transmit_bps` | Gauge | 系统资源 | - |
| 14 | ALPHA-SYS-006 | `node_load_average_1m` | Gauge | 系统资源 | >4.0 |
| 15 | ALPHA-SYS-007 | `node_file_descriptors_used` | Gauge | 系统资源 | >80% |
| 16 | ALPHA-DB-001 | `postgres_connections_active` | Gauge | 数据库 | >80 |
| 17 | ALPHA-DB-002 | `postgres_connections_idle` | Gauge | 数据库 | - |
| 18 | ALPHA-DB-003 | `postgres_query_latency_p99` | Histogram | 数据库 | >100ms |
| 19 | ALPHA-DB-004 | `postgres_transactions_per_second` | Gauge | 数据库 | - |
| 20 | ALPHA-DB-005 | `postgres_locks_waiting` | Gauge | 数据库 | >5 |

### 8.2 Grafana 仪表盘映射

| 仪表盘 | 指标数 | Panel 数 | UID |
|---|---|---|---|
| Alpha Overview | 6 | 8 | `alpha-overview` |
| Alpha Application Performance | 8 | 12 | `alpha-app-perf` |
| Alpha System Resources | 7 | 7 | `alpha-system` |
| Alpha Database | 5 | 6 | `alpha-database` |

---

## 9. 附录

### 9.1 PromQL 查询手册

```promql
# === 应用性能指标 ===

# Executor 指令执行时延 P99
histogram_quantile(0.99, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))

# Executor 队列深度
executor_queue_depth

# Executor 成功率
executor_success_rate

# Verifier 验证时延 P99
histogram_quantile(0.99, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le))

# Verifier 队列深度
verifier_queue_depth

# Verifier 不匹配率
verifier_mismatch_rate

# Gateway 请求时延 P99
histogram_quantile(0.99, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le))

# Gateway 请求速率
gateway_request_rate

# === 系统资源指标 ===

# CPU 使用率
node_cpu_usage_percent

# 内存使用率
node_memory_usage_percent

# 磁盘使用率
node_disk_usage_percent

# 网络接收速率
node_network_receive_bps

# 网络发送速率
node_network_transmit_bps

# 1 分钟负载
node_load_average_1m

# 文件描述符使用率
node_file_descriptors_used / node_file_descriptors_max * 100

# === 数据库指标 ===

# 活跃连接数
postgres_connections_active

# 空闲连接数
postgres_connections_idle

# 查询时延 P99
histogram_quantile(0.99, sum(rate(postgres_query_latency_p99_bucket[5m])) by(le))

# 事务速率
rate(postgres_transactions_total[1m])

# 等待锁数量
postgres_locks_waiting
```

### 9.2 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Phase 3 50 指标配置 | dashboard_v7_final.md | 参考实现 |
| OpenTelemetry 集成 | otel_integration.md | 追踪集成 |
| Phase 4 详细计划 | phase4_detailed_plan_v2.md | 项目计划 |

---

**文档状态**: ✅ Week 1-T5 完成  
**创建日期**: 2026-04-05  
**责任人**: Observability-Agent + SRE-Agent  
**保管**: 项目文档库  
**环境**: Alpha (Phase 4 Week 1)
