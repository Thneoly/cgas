# Beta 环境监控指标接入 (35 个指标)

**版本**: v1.0  
**日期**: 2026-04-12  
**责任人**: Observability-Agent + SRE-Agent  
**状态**: ✅ Week 2-T5 完成  
**环境**: Beta (外部测试环境)  
**release_id**: release-2026-04-12-phase4-week2-beta-monitoring

---

## 1. 概述

### 1.1 任务目标

在 Phase 4 Week 2 完成 Beta 环境的 **35 个核心监控指标接入**，建立 Beta 环境的基础可观测性体系，为 Staging 和 Production 环境部署提供监控基准。

### 1.2 指标分类

| 分类 | 指标数 | 说明 | 优先级 |
|---|---|---|---|
| **应用性能指标** | 15 个 | Executor/Verifier/Gateway/Scheduler 性能 | P0 |
| **系统资源指标** | 10 个 | CPU/内存/磁盘/网络/容器 | P0 |
| **数据库指标** | 10 个 | PostgreSQL 连接/查询/锁/复制 | P1 |
| **总计** | **35 个** | - | - |

### 1.3 Beta 环境与 Alpha 对比

| 特性 | Alpha | Beta | 说明 |
|---|---|---|---|
| 监控指标 | 20 个 | 35 个 | +75% 覆盖 |
| 告警规则 | 10 条 | 20 条 | +100% 覆盖 |
| Grafana 仪表盘 | 4 个 | 6 个 | +50% 可视化 |
| Trace 覆盖率 | ≥98% | ≥99% | 更高要求 |
| 采样率 | 50% | 25% | 降低采样 |

---

## 2. 应用性能指标 (15 个)

### 2.1 Executor 指标 (4 个)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **BETA-APP-001** | `executor_instruction_latency_p99` | Histogram | 实时 | >180ms | 指令执行时延 P99 |
| **BETA-APP-002** | `executor_instruction_latency_p95` | Histogram | 实时 | >150ms | 指令执行时延 P95 |
| **BETA-APP-003** | `executor_queue_depth` | Gauge | 10s | >80 | 执行队列深度 |
| **BETA-APP-004** | `executor_success_rate` | Gauge | 30s | <97% | 指令执行成功率 |

#### 2.1.1 实现代码 (Rust)

```rust
// src/executor/metrics.rs - Beta 环境

use prometheus::{Histogram, Gauge, Counter, HistogramOpts, Opts, register_histogram, register_gauge, register_counter};

lazy_static! {
    /// 指令执行时延 P99
    pub static ref EXECUTOR_INSTRUCTION_LATENCY_P99: Histogram = register_histogram!(
        HistogramOpts::new("executor_instruction_latency_p99", "Executor instruction latency P99 in ms")
            .namespace("cgas_beta")
            .subsystem("executor")
            .buckets(vec![25.0, 50.0, 75.0, 100.0, 125.0, 150.0, 175.0, 200.0, 250.0, 300.0, 400.0, 500.0])
            .help("Executor instruction latency P99 histogram")
    ).unwrap();
    
    /// 指令执行时延 P95
    pub static ref EXECUTOR_INSTRUCTION_LATENCY_P95: Histogram = register_histogram!(
        HistogramOpts::new("executor_instruction_latency_p95", "Executor instruction latency P95 in ms")
            .namespace("cgas_beta")
            .subsystem("executor")
            .buckets(vec![25.0, 50.0, 75.0, 100.0, 125.0, 150.0, 175.0, 200.0, 250.0, 300.0])
    ).unwrap();
    
    /// 执行队列深度
    pub static ref EXECUTOR_QUEUE_DEPTH: Gauge = register_gauge!(
        Opts::new("executor_queue_depth", "Executor queue depth")
            .namespace("cgas_beta")
            .subsystem("executor")
            .help("Current executor queue depth")
    ).unwrap();
    
    /// 指令执行成功率
    pub static ref EXECUTOR_SUCCESS_RATE: Gauge = register_gauge!(
        Opts::new("executor_success_rate", "Executor instruction success rate percentage")
            .namespace("cgas_beta")
            .subsystem("executor")
            .help("Executor instruction success rate in percentage")
    ).unwrap();
}

/// 记录指令执行完成
pub fn record_instruction_completion(duration_ms: u64, success: bool) {
    EXECUTOR_INSTRUCTION_LATENCY_P99.observe(duration_ms as f64);
    EXECUTOR_INSTRUCTION_LATENCY_P95.observe(duration_ms as f64);
    
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

### 2.2 Verifier 指标 (4 个)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **BETA-APP-005** | `verifier_verification_latency_p99` | Histogram | 实时 | >180ms | 验证时延 P99 |
| **BETA-APP-006** | `verifier_verification_latency_p95` | Histogram | 实时 | >150ms | 验证时延 P95 |
| **BETA-APP-007** | `verifier_queue_depth` | Gauge | 10s | >80 | 验证队列深度 |
| **BETA-APP-008** | `verifier_mismatch_rate` | Gauge | 30s | >0.5% | 验证不匹配率 |

#### 2.2.1 实现代码 (Rust)

```rust
// src/verifier/metrics.rs - Beta 环境

use prometheus::{Histogram, Gauge, register_histogram, register_gauge, HistogramOpts, Opts};

lazy_static! {
    /// 验证时延 P99
    pub static ref VERIFIER_VERIFICATION_LATENCY_P99: Histogram = register_histogram!(
        HistogramOpts::new("verifier_verification_latency_p99", "Verifier verification latency P99 in ms")
            .namespace("cgas_beta")
            .subsystem("verifier")
            .buckets(vec![25.0, 50.0, 75.0, 100.0, 125.0, 150.0, 175.0, 200.0, 250.0, 300.0, 400.0, 500.0])
    ).unwrap();
    
    /// 验证时延 P95
    pub static ref VERIFIER_VERIFICATION_LATENCY_P95: Histogram = register_histogram!(
        HistogramOpts::new("verifier_verification_latency_p95", "Verifier verification latency P95 in ms")
            .namespace("cgas_beta")
            .subsystem("verifier")
            .buckets(vec![25.0, 50.0, 75.0, 100.0, 125.0, 150.0, 175.0, 200.0, 250.0, 300.0])
    ).unwrap();
    
    /// 验证队列深度
    pub static ref VERIFIER_QUEUE_DEPTH: Gauge = register_gauge!(
        Opts::new("verifier_queue_depth", "Verifier queue depth")
            .namespace("cgas_beta")
            .subsystem("verifier")
    ).unwrap();
    
    /// 验证不匹配率
    pub static ref VERIFIER_MISMATCH_RATE: Gauge = register_gauge!(
        Opts::new("verifier_mismatch_rate", "Verifier mismatch rate percentage")
            .namespace("cgas_beta")
            .subsystem("verifier")
    ).unwrap();
}

/// 记录验证完成
pub fn record_verification_completion(duration_ms: u64, mismatch: bool) {
    VERIFIER_VERIFICATION_LATENCY_P99.observe(duration_ms as f64);
    VERIFIER_VERIFICATION_LATENCY_P95.observe(duration_ms as f64);
    
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

### 2.3 Gateway 指标 (4 个)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **BETA-APP-009** | `gateway_request_latency_p99` | Histogram | 实时 | >250ms | 请求处理时延 P99 |
| **BETA-APP-010** | `gateway_request_latency_p95` | Histogram | 实时 | >200ms | 请求处理时延 P95 |
| **BETA-APP-011** | `gateway_request_rate` | Gauge | 10s | - | 请求速率 (QPS) |
| **BETA-APP-012** | `gateway_error_rate` | Gauge | 30s | >1% | 请求错误率 |

#### 2.3.1 实现代码 (TypeScript)

```typescript
// src/gateway/metrics.ts - Beta 环境

import { Histogram, Gauge, Counter, register } from 'prom-client';

// 请求处理时延 P99
const gatewayRequestLatencyP99 = new Histogram({
  name: 'gateway_request_latency_p99',
  help: 'Gateway request latency P99 in ms',
  labelNames: ['method', 'route'],
  buckets: [25, 50, 75, 100, 125, 150, 175, 200, 250, 300, 400, 500],
});

// 请求处理时延 P95
const gatewayRequestLatencyP95 = new Histogram({
  name: 'gateway_request_latency_p95',
  help: 'Gateway request latency P95 in ms',
  labelNames: ['method', 'route'],
  buckets: [25, 50, 75, 100, 125, 150, 175, 200, 250, 300],
});

// 请求速率
const gatewayRequestRate = new Gauge({
  name: 'gateway_request_rate',
  help: 'Gateway request rate (QPS)',
  labelNames: ['method'],
});

// 错误率
const gatewayErrorRate = new Gauge({
  name: 'gateway_error_rate',
  help: 'Gateway error rate percentage',
  labelNames: ['method', 'route'],
});

export function recordRequestCompletion(
  durationMs: number,
  method: string,
  route: string,
  success: boolean
) {
  gatewayRequestLatencyP99
    .labels({ method, route })
    .observe(durationMs);
  gatewayRequestLatencyP95
    .labels({ method, route })
    .observe(durationMs);
  
  if (!success) {
    const current = gatewayErrorRate.labels({ method, route }).get() || 0;
    gatewayErrorRate.labels({ method, route }).set((current * 100 + 1) / 101);
  }
}

export function updateRequestRate(method: string, rate: number) {
  gatewayRequestRate.labels({ method }).set(rate);
}

export { gatewayRequestLatencyP99, gatewayRequestLatencyP95, gatewayRequestRate, gatewayErrorRate };
```

---

### 2.4 Scheduler 指标 (3 个) - Beta 新增

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **BETA-APP-013** | `scheduler_task_latency_p99` | Histogram | 实时 | >300ms | 任务调度时延 P99 |
| **BETA-APP-014** | `scheduler_pending_tasks` | Gauge | 10s | >50 | 待调度任务数 |
| **BETA-APP-015** | `scheduler_success_rate` | Gauge | 30s | <98% | 任务调度成功率 |

#### 2.4.1 实现代码 (Rust)

```rust
// src/scheduler/metrics.rs - Beta 环境

use prometheus::{Histogram, Gauge, HistogramOpts, Opts, register_histogram, register_gauge};

lazy_static! {
    /// 任务调度时延 P99
    pub static ref SCHEDULER_TASK_LATENCY_P99: Histogram = register_histogram!(
        HistogramOpts::new("scheduler_task_latency_p99", "Scheduler task latency P99 in ms")
            .namespace("cgas_beta")
            .subsystem("scheduler")
            .buckets(vec![50.0, 100.0, 150.0, 200.0, 250.0, 300.0, 400.0, 500.0, 750.0, 1000.0])
    ).unwrap();
    
    /// 待调度任务数
    pub static ref SCHEDULER_PENDING_TASKS: Gauge = register_gauge!(
        Opts::new("scheduler_pending_tasks", "Scheduler pending tasks count")
            .namespace("cgas_beta")
            .subsystem("scheduler")
    ).unwrap();
    
    /// 任务调度成功率
    pub static ref SCHEDULER_SUCCESS_RATE: Gauge = register_gauge!(
        Opts::new("scheduler_success_rate", "Scheduler task success rate percentage")
            .namespace("cgas_beta")
            .subsystem("scheduler")
    ).unwrap();
}

/// 记录任务调度完成
pub fn record_task_completion(duration_ms: u64, success: bool) {
    SCHEDULER_TASK_LATENCY_P99.observe(duration_ms as f64);
    
    if success {
        let current = SCHEDULER_SUCCESS_RATE.get();
        SCHEDULER_SUCCESS_RATE.set((current * 100.0 + 1.0) / 101.0);
    } else {
        let current = SCHEDULER_SUCCESS_RATE.get();
        SCHEDULER_SUCCESS_RATE.set((current * 100.0) / 101.0);
    }
}

/// 更新待调度任务数
pub fn update_pending_tasks(count: usize) {
    SCHEDULER_PENDING_TASKS.set(count as f64);
}
```

---

## 3. 系统资源指标 (10 个)

### 3.1 指标清单

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **BETA-SYS-001** | `node_cpu_usage_percent` | Gauge | 10s | >75% | CPU 使用率 |
| **BETA-SYS-002** | `node_memory_usage_percent` | Gauge | 10s | >80% | 内存使用率 |
| **BETA-SYS-003** | `node_disk_usage_percent` | Gauge | 1min | >85% | 磁盘使用率 |
| **BETA-SYS-004** | `node_network_receive_bps` | Gauge | 10s | - | 网络接收速率 |
| **BETA-SYS-005** | `node_network_transmit_bps` | Gauge | 10s | - | 网络发送速率 |
| **BETA-SYS-006** | `node_load_average_1m` | Gauge | 30s | >3.0 | 1 分钟负载 |
| **BETA-SYS-007** | `node_load_average_5m` | Gauge | 30s | >2.5 | 5 分钟负载 |
| **BETA-SYS-008** | `node_file_descriptors_used` | Gauge | 1min | >75% | 文件描述符使用率 |
| **BETA-SYS-009** | `container_cpu_usage_percent` | Gauge | 10s | >80% | 容器 CPU 使用率 |
| **BETA-SYS-010** | `container_memory_usage_bytes` | Gauge | 10s | - | 容器内存使用 |

### 3.2 Node Exporter 配置

```yaml
# prometheus-node-exporter-config-beta.yaml

prometheus:
  nodeExporter:
    enabled: true
    port: 9100
    
  scrapeConfigs:
    - jobName: 'node-exporter-beta'
      static_configs:
        - targets: ['beta-node-1:9100', 'beta-node-2:9100', 'beta-node-3:9100']
      scrapeInterval: 10s
      scrapeTimeout: 5s
      metricsPath: /metrics
      
    - jobName: 'cadvisor-beta'
      static_configs:
        - targets: ['beta-cadvisor:8080']
      scrapeInterval: 10s
```

### 3.3 Prometheus 告警规则

```yaml
# beta_system_alerts.yaml

groups:
  - name: beta-system-alerts
    interval: 30s
    rules:
      - alert: BetaHighCPUUsage
        expr: node_cpu_usage_percent > 75
        for: 5m
        labels:
          severity: warning
          environment: beta
        annotations:
          summary: "Beta 环境 CPU 使用率过高"
          description: "实例 {{ $labels.instance }} CPU 使用率 {{ $value }}% 超过 75%"
          
      - alert: BetaHighMemoryUsage
        expr: node_memory_usage_percent > 80
        for: 5m
        labels:
          severity: warning
          environment: beta
        annotations:
          summary: "Beta 环境内存使用率过高"
          description: "实例 {{ $labels.instance }} 内存使用率 {{ $value }}% 超过 80%"
          
      - alert: BetaHighDiskUsage
        expr: node_disk_usage_percent > 85
        for: 10m
        labels:
          severity: critical
          environment: beta
        annotations:
          summary: "Beta 环境磁盘使用率过高"
          description: "实例 {{ $labels.instance }} 磁盘使用率 {{ $value }}% 超过 85%"
```

---

## 4. 数据库指标 (10 个)

### 4.1 指标清单

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **BETA-DB-001** | `postgres_connections_active` | Gauge | 10s | >70 | 活跃连接数 |
| **BETA-DB-002** | `postgres_connections_idle` | Gauge | 10s | - | 空闲连接数 |
| **BETA-DB-003** | `postgres_connections_max` | Gauge | 1min | - | 最大连接数 |
| **BETA-DB-004** | `postgres_query_latency_p99` | Histogram | 实时 | >80ms | 查询时延 P99 |
| **BETA-DB-005** | `postgres_query_latency_p95` | Histogram | 实时 | >60ms | 查询时延 P95 |
| **BETA-DB-006** | `postgres_transactions_per_second` | Gauge | 30s | - | 事务速率 |
| **BETA-DB-007** | `postgres_locks_waiting` | Gauge | 10s | >3 | 等待锁数量 |
| **BETA-DB-008** | `postgres_locks_held` | Gauge | 10s | - | 持有锁数量 |
| **BETA-DB-009** | `postgres_replication_lag_seconds` | Gauge | 10s | >5 | 复制延迟 |
| **BETA-DB-010** | `postgres_cache_hit_ratio` | Gauge | 1min | <95% | 缓存命中率 |

### 4.2 Postgres Exporter 配置

```yaml
# postgres-exporter-config-beta.yaml

prometheus:
  postgresExporter:
    enabled: true
    port: 9187
    
  datasource:
    host: beta-postgres
    port: 5432
    username: prometheus
    password: ${POSTGRES_EXPORTER_PASSWORD}
    sslmode: disable
    
  collect:
    - pg_stat_activity
    - pg_stat_database
    - pg_locks
    - pg_stat_statements
    - pg_replication
    - pg_stat_user_tables
```

### 4.3 自定义 SQL 查询

```sql
-- postgres_custom_metrics_beta.sql

-- 活跃连接数
SELECT count(*) as postgres_connections_active
FROM pg_stat_activity
WHERE state = 'active';

-- 空闲连接数
SELECT count(*) as postgres_connections_idle
FROM pg_stat_activity
WHERE state = 'idle';

-- 最大连接数
SHOW max_connections;

-- 等待锁数量
SELECT count(*) as postgres_locks_waiting
FROM pg_locks
WHERE NOT granted;

-- 持有锁数量
SELECT count(*) as postgres_locks_held
FROM pg_locks
WHERE granted;

-- 查询时延 (需要 pg_stat_statements)
SELECT 
  percentile_cont(0.99) WITHIN GROUP (ORDER BY total_exec_time / calls) as query_latency_p99_ms,
  percentile_cont(0.95) WITHIN GROUP (ORDER BY total_exec_time / calls) as query_latency_p95_ms
FROM pg_stat_statements
WHERE calls > 0;

-- 事务速率
SELECT 
  (xact_commit + xact_rollback) as transactions_total
FROM pg_stat_database
WHERE datname = current_database();

-- 复制延迟 (从库)
SELECT 
  EXTRACT(EPOCH FROM (now() - pg_last_xact_replay_timestamp())) as replication_lag_seconds
FROM pg_stat_wal_receiver;

-- 缓存命中率
SELECT 
  sum(heap_blks_hit) / (sum(heap_blks_hit) + sum(heap_blks_read)) as cache_hit_ratio
FROM pg_statio_user_tables;
```

---

## 5. Prometheus 配置

### 5.1 Beta 环境 Prometheus 配置

```yaml
# prometheus-beta.yaml

global:
  scrape_interval: 10s
  evaluation_interval: 10s
  external_labels:
    environment: beta
    phase: phase4

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager-beta:9093']

rule_files:
  - /etc/prometheus/rules/beta_app_alerts.yaml
  - /etc/prometheus/rules/beta_system_alerts.yaml
  - /etc/prometheus/rules/beta_db_alerts.yaml

scrape_configs:
  # Executor 指标
  - job_name: 'beta-executor'
    static_configs:
      - targets: ['beta-executor-1:8080', 'beta-executor-2:8080', 'beta-executor-3:8080']
    metrics_path: /metrics
    scrape_interval: 10s
    
  # Verifier 指标
  - job_name: 'beta-verifier'
    static_configs:
      - targets: ['beta-verifier-1:8081', 'beta-verifier-2:8081', 'beta-verifier-3:8081']
    metrics_path: /metrics
    scrape_interval: 10s
    
  # Gateway 指标
  - job_name: 'beta-gateway'
    static_configs:
      - targets: ['beta-gateway-1:8084', 'beta-gateway-2:8084']
    metrics_path: /metrics
    scrape_interval: 10s
    
  # Scheduler 指标
  - job_name: 'beta-scheduler'
    static_configs:
      - targets: ['beta-scheduler-1:8085']
    metrics_path: /metrics
    scrape_interval: 10s
    
  # Node Exporter (系统指标)
  - job_name: 'beta-node-exporter'
    static_configs:
      - targets: ['beta-node-1:9100', 'beta-node-2:9100', 'beta-node-3:9100']
    scrape_interval: 10s
    
  # cAdvisor (容器指标)
  - job_name: 'beta-cadvisor'
    static_configs:
      - targets: ['beta-cadvisor:8080']
    scrape_interval: 10s
    
  # Postgres Exporter (数据库指标)
  - job_name: 'beta-postgres-exporter'
    static_configs:
      - targets: ['beta-postgres-exporter:9187']
    scrape_interval: 10s
```

---

## 6. 指标验收标准

### 6.1 数据采集验收

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 指标可查询 | 35 个指标均有数据 | Prometheus 查询 | 100% 指标可查询 |
| 数据新鲜度 | 延迟<20s | 时间戳检查 | 最新数据<20s |
| Labels 完整 | 所有 Labels 正确 | 指标检查 | 100% Labels 存在 |
| 数值准确性 | 与日志一致 | 抽样比对 | 误差<0.5% |

### 6.2 快速验证命令

```bash
# 验证 Executor 指标
curl 'http://prometheus-beta:9090/api/v1/query?query=executor_instruction_latency_p99'

# 验证 Verifier 指标
curl 'http://prometheus-beta:9090/api/v1/query?query=verifier_verification_latency_p99'

# 验证 Gateway 指标
curl 'http://prometheus-beta:9090/api/v1/query?query=gateway_request_latency_p99'

# 验证 Scheduler 指标
curl 'http://prometheus-beta:9090/api/v1/query?query=scheduler_task_latency_p99'

# 验证系统指标
curl 'http://prometheus-beta:9090/api/v1/query?query=node_cpu_usage_percent'

# 验证数据库指标
curl 'http://prometheus-beta:9090/api/v1/query?query=postgres_connections_active'

# 验证所有 Beta 环境指标
curl 'http://prometheus-beta:9090/api/v1/query?query={environment="beta"}' | jq '.data.result | length'
```

---

## 7. 实施计划

| 任务 | 责任人 | 状态 | 交付物 | 预计耗时 |
|---|---|---|---|---|
| Executor 指标集成 | Dev | ✅ 完成 | executor/metrics.rs | 90 分钟 |
| Verifier 指标集成 | Dev | ✅ 完成 | verifier/metrics.rs | 90 分钟 |
| Gateway 指标集成 | Dev | ✅ 完成 | gateway/metrics.ts | 90 分钟 |
| Scheduler 指标集成 | Dev | ✅ 完成 | scheduler/metrics.rs | 60 分钟 |
| Node Exporter 部署 | SRE | ✅ 完成 | node-exporter-config.yaml | 30 分钟 |
| cAdvisor 部署 | SRE | ✅ 完成 | cadvisor-config.yaml | 30 分钟 |
| Postgres Exporter 部署 | SRE | ✅ 完成 | postgres-exporter-config.yaml | 30 分钟 |
| Prometheus 配置更新 | SRE | ✅ 完成 | prometheus-beta.yaml | 30 分钟 |
| 告警规则配置 | Observability | ✅ 完成 | beta_alert_rules.yaml | 90 分钟 |
| 指标验证 | Observability + SRE | ✅ 完成 | validation_report.md | 60 分钟 |

---

## 8. 指标汇总

### 8.1 完整指标列表

| # | 指标 ID | 指标名 | 类型 | 分类 | P0 告警阈值 |
|---|---|---|---|---|---|
| 1 | BETA-APP-001 | `executor_instruction_latency_p99` | Histogram | 应用性能 | >180ms |
| 2 | BETA-APP-002 | `executor_instruction_latency_p95` | Histogram | 应用性能 | >150ms |
| 3 | BETA-APP-003 | `executor_queue_depth` | Gauge | 应用性能 | >80 |
| 4 | BETA-APP-004 | `executor_success_rate` | Gauge | 应用性能 | <97% |
| 5 | BETA-APP-005 | `verifier_verification_latency_p99` | Histogram | 应用性能 | >180ms |
| 6 | BETA-APP-006 | `verifier_verification_latency_p95` | Histogram | 应用性能 | >150ms |
| 7 | BETA-APP-007 | `verifier_queue_depth` | Gauge | 应用性能 | >80 |
| 8 | BETA-APP-008 | `verifier_mismatch_rate` | Gauge | 应用性能 | >0.5% |
| 9 | BETA-APP-009 | `gateway_request_latency_p99` | Histogram | 应用性能 | >250ms |
| 10 | BETA-APP-010 | `gateway_request_latency_p95` | Histogram | 应用性能 | >200ms |
| 11 | BETA-APP-011 | `gateway_request_rate` | Gauge | 应用性能 | - |
| 12 | BETA-APP-012 | `gateway_error_rate` | Gauge | 应用性能 | >1% |
| 13 | BETA-APP-013 | `scheduler_task_latency_p99` | Histogram | 应用性能 | >300ms |
| 14 | BETA-APP-014 | `scheduler_pending_tasks` | Gauge | 应用性能 | >50 |
| 15 | BETA-APP-015 | `scheduler_success_rate` | Gauge | 应用性能 | <98% |
| 16 | BETA-SYS-001 | `node_cpu_usage_percent` | Gauge | 系统资源 | >75% |
| 17 | BETA-SYS-002 | `node_memory_usage_percent` | Gauge | 系统资源 | >80% |
| 18 | BETA-SYS-003 | `node_disk_usage_percent` | Gauge | 系统资源 | >85% |
| 19 | BETA-SYS-004 | `node_network_receive_bps` | Gauge | 系统资源 | - |
| 20 | BETA-SYS-005 | `node_network_transmit_bps` | Gauge | 系统资源 | - |
| 21 | BETA-SYS-006 | `node_load_average_1m` | Gauge | 系统资源 | >3.0 |
| 22 | BETA-SYS-007 | `node_load_average_5m` | Gauge | 系统资源 | >2.5 |
| 23 | BETA-SYS-008 | `node_file_descriptors_used` | Gauge | 系统资源 | >75% |
| 24 | BETA-SYS-009 | `container_cpu_usage_percent` | Gauge | 系统资源 | >80% |
| 25 | BETA-SYS-010 | `container_memory_usage_bytes` | Gauge | 系统资源 | - |
| 26 | BETA-DB-001 | `postgres_connections_active` | Gauge | 数据库 | >70 |
| 27 | BETA-DB-002 | `postgres_connections_idle` | Gauge | 数据库 | - |
| 28 | BETA-DB-003 | `postgres_connections_max` | Gauge | 数据库 | - |
| 29 | BETA-DB-004 | `postgres_query_latency_p99` | Histogram | 数据库 | >80ms |
| 30 | BETA-DB-005 | `postgres_query_latency_p95` | Histogram | 数据库 | >60ms |
| 31 | BETA-DB-006 | `postgres_transactions_per_second` | Gauge | 数据库 | - |
| 32 | BETA-DB-007 | `postgres_locks_waiting` | Gauge | 数据库 | >3 |
| 33 | BETA-DB-008 | `postgres_locks_held` | Gauge | 数据库 | - |
| 34 | BETA-DB-009 | `postgres_replication_lag_seconds` | Gauge | 数据库 | >5 |
| 35 | BETA-DB-010 | `postgres_cache_hit_ratio` | Gauge | 数据库 | <95% |

### 8.2 Grafana 仪表盘映射

| 仪表盘 | 指标数 | Panel 数 | UID |
|---|---|---|---|
| Beta Overview | 8 | 10 | `beta-overview` |
| Beta Application Performance | 15 | 18 | `beta-app-perf` |
| Beta System Resources | 10 | 10 | `beta-system` |
| Beta Database | 10 | 12 | `beta-database` |
| Beta Scheduler | 3 | 6 | `beta-scheduler` |
| Beta Container Monitoring | 2 | 4 | `beta-containers` |

---

## 9. 附录

### 9.1 PromQL 查询手册

```promql
# === 应用性能指标 ===

# Executor 指令执行时延 P99/P95
histogram_quantile(0.99, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.95, sum(rate(executor_instruction_latency_p95_bucket[5m])) by(le))

# Executor 队列深度
executor_queue_depth

# Executor 成功率
executor_success_rate

# Verifier 验证时延 P99/P95
histogram_quantile(0.99, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.95, sum(rate(verifier_verification_latency_p95_bucket[5m])) by(le))

# Verifier 队列深度
verifier_queue_depth

# Verifier 不匹配率
verifier_mismatch_rate

# Gateway 请求时延 P99/P95
histogram_quantile(0.99, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.95, sum(rate(gateway_request_latency_p95_bucket[5m])) by(le))

# Gateway 请求速率
gateway_request_rate

# Gateway 错误率
gateway_error_rate

# Scheduler 任务时延 P99
histogram_quantile(0.99, sum(rate(scheduler_task_latency_p99_bucket[5m])) by(le))

# Scheduler 待调度任务数
scheduler_pending_tasks

# Scheduler 成功率
scheduler_success_rate

# === 系统资源指标 ===

# CPU 使用率
node_cpu_usage_percent

# 内存使用率
node_memory_usage_percent

# 磁盘使用率
node_disk_usage_percent

# 网络接收/发送速率
node_network_receive_bps
node_network_transmit_bps

# 系统负载
node_load_average_1m
node_load_average_5m

# 文件描述符使用率
node_file_descriptors_used / node_file_descriptors_max * 100

# 容器 CPU/内存
container_cpu_usage_percent
container_memory_usage_bytes

# === 数据库指标 ===

# 连接数
postgres_connections_active
postgres_connections_idle
postgres_connections_max

# 查询时延 P99/P95
histogram_quantile(0.99, sum(rate(postgres_query_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.95, sum(rate(postgres_query_latency_p95_bucket[5m])) by(le))

# 事务速率
rate(postgres_transactions_total[1m])

# 锁
postgres_locks_waiting
postgres_locks_held

# 复制延迟
postgres_replication_lag_seconds

# 缓存命中率
postgres_cache_hit_ratio
```

### 9.2 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Alpha 20 指标配置 | alpha_week1/alpha_monitoring_20_metrics.md | 参考实现 |
| Phase 3 50 指标配置 | dashboard_v7_final.md | 参考实现 |
| OpenTelemetry 集成 | otel_integration.md | 追踪集成 |
| Phase 4 详细计划 | phase4_detailed_plan_v2.md | 项目计划 |

---

**文档状态**: ✅ Week 2-T5 完成  
**创建日期**: 2026-04-12  
**责任人**: Observability-Agent + SRE-Agent  
**保管**: 项目文档库  
**环境**: Beta (Phase 4 Week 2)
