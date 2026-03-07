# Alpha 环境监控配置文档

**版本**: v1.0  
**日期**: 2026-04-05  
**责任人**: SRE-Agent + Observability-Agent  
**状态**: ✅ 配置完成  
**环境**: Alpha (内部测试环境)

---

## 📋 执行摘要

本文档详细描述 Alpha 环境的监控配置，包括监控架构、20 个基础监控指标、10 个告警规则、Grafana 仪表盘配置和告警通知策略。

**监控目标**:
- 实时监控系统健康状态
- 快速发现和定位问题
- 提供性能基线数据
- 支持容量规划决策

**配置完成**: 20 个监控指标 + 10 个告警规则 + 5 个仪表盘

---

## 🏗️ 监控架构

### 监控组件

```
Alpha 环境监控架构:

┌─────────────────────────────────────────────────────────────┐
│                     Grafana (可视化)                        │
│                    http://10.0.1.10:3000                    │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                    Prometheus (监控核心)                     │
│                    http://10.0.1.10:9090                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  TSDB 存储   │  │  告警规则引擎 │  │  PromQL 引擎  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────────────┬────────────────────────────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  Alertmanager   │ │   Node 导出器   │ │   应用导出器    │
│  :9093          │ │   :9100         │ │   :8080/metrics │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                   │                   │
         ▼                   ▼                   ▼
    通知渠道            系统指标            应用指标
   - Webhook         - CPU/内存/磁盘      - QPS/时延/错误率
   - 邮件            - 网络/负载          - JVM/GC
   - 短信
```

### 监控组件版本

| 组件 | 版本 | 部署位置 | 端口 |
|---|---|---|---|
| Prometheus | 2.47.0 | alpha-lb-01 | 9090 |
| Grafana | 10.1.5 | alpha-lb-01 | 3000 |
| Alertmanager | 0.26.0 | alpha-lb-01 | 9093 |
| Node Exporter | 1.6.1 | 全部服务器 | 9100 |
| Application Metrics | v3.0.0-alpha | 应用服务器 | 8080/metrics |

---

## 📊 监控指标配置 (20 个)

### 系统指标 (8 个)

#### 1. CPU 使用率

**指标名称**: `node_cpu_usage`  
**PromQL**:
```promql
100 - (avg by(instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)
```

**告警阈值**:
- Warning: >80% (持续 5 分钟)
- Critical: >95% (持续 2 分钟)

**仪表盘 Panel**:
```json
{
  "title": "CPU Usage",
  "type": "gauge",
  "targets": [
    {
      "expr": "100 - (avg by(instance) (irate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)"
    }
  ],
  "thresholds": [
    {"value": 0, "color": "green"},
    {"value": 80, "color": "yellow"},
    {"value": 95, "color": "red"}
  ]
}
```

---

#### 2. 内存使用率

**指标名称**: `node_memory_usage`  
**PromQL**:
```promql
(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100
```

**告警阈值**:
- Warning: >85% (持续 5 分钟)
- Critical: >95% (持续 2 分钟)

**仪表盘 Panel**:
```json
{
  "title": "Memory Usage",
  "type": "gauge",
  "targets": [
    {
      "expr": "(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100"
    }
  ],
  "thresholds": [
    {"value": 0, "color": "green"},
    {"value": 85, "color": "yellow"},
    {"value": 95, "color": "red"}
  ]
}
```

---

#### 3. 磁盘使用率

**指标名称**: `node_disk_usage`  
**PromQL**:
```promql
(1 - (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"})) * 100
```

**告警阈值**:
- Warning: >80% (持续 10 分钟)
- Critical: >90% (持续 5 分钟)

**仪表盘 Panel**:
```json
{
  "title": "Disk Usage",
  "type": "gauge",
  "targets": [
    {
      "expr": "(1 - (node_filesystem_avail_bytes{mountpoint=\"/\"} / node_filesystem_size_bytes{mountpoint=\"/\"})) * 100"
    }
  ],
  "thresholds": [
    {"value": 0, "color": "green"},
    {"value": 80, "color": "yellow"},
    {"value": 90, "color": "red"}
  ]
}
```

---

#### 4. 网络入站流量

**指标名称**: `node_network_receive_bytes`  
**PromQL**:
```promql
irate(node_network_receive_bytes_total{device!="lo"}[5m])
```

**单位**: bytes/s  
**仪表盘 Panel**:
```json
{
  "title": "Network Inbound",
  "type": "graph",
  "targets": [
    {
      "expr": "irate(node_network_receive_bytes_total{device!=\"lo\"}[5m])",
      "legendFormat": "{{instance}} - {{device}}"
    }
  ],
  "yAxes": [{"format": "Bps"}]
}
```

---

#### 5. 网络出站流量

**指标名称**: `node_network_transmit_bytes`  
**PromQL**:
```promql
irate(node_network_transmit_bytes_total{device!="lo"}[5m])
```

**单位**: bytes/s  
**仪表盘 Panel**:
```json
{
  "title": "Network Outbound",
  "type": "graph",
  "targets": [
    {
      "expr": "irate(node_network_transmit_bytes_total{device!=\"lo\"}[5m])",
      "legendFormat": "{{instance}} - {{device}}"
    }
  ],
  "yAxes": [{"format": "Bps"}]
}
```

---

#### 6. 系统负载 (1 分钟)

**指标名称**: `node_load_1m`  
**PromQL**:
```promql
node_load1
```

**告警阈值**:
- Warning: >8 (持续 10 分钟)

**仪表盘 Panel**:
```json
{
  "title": "Load Average (1m)",
  "type": "graph",
  "targets": [
    {
      "expr": "node_load1",
      "legendFormat": "{{instance}}"
    }
  ],
  "alert": {
    "conditions": [
      {"evaluator": {"type": "gt", "params": [8]}}
    ]
  }
}
```

---

#### 7. 文件描述符使用率

**指标名称**: `node_filefd_alloc`  
**PromQL**:
```promql
(node_filefd_allocated / node_filefd_maximum) * 100
```

**告警阈值**:
- Warning: >80% (持续 5 分钟)

**仪表盘 Panel**:
```json
{
  "title": "File Descriptors",
  "type": "gauge",
  "targets": [
    {
      "expr": "(node_filefd_allocated / node_filefd_maximum) * 100"
    }
  ],
  "thresholds": [
    {"value": 0, "color": "green"},
    {"value": 80, "color": "yellow"},
    {"value": 90, "color": "red"}
  ]
}
```

---

#### 8. 系统启动时间

**指标名称**: `node_boot_time`  
**PromQL**:
```promql
node_boot_time_seconds
```

**单位**: Unix 时间戳  
**仪表盘 Panel**:
```json
{
  "title": "Uptime",
  "type": "stat",
  "targets": [
    {
      "expr": "time() - node_boot_time_seconds",
      "legendFormat": "{{instance}}"
    }
  ],
  "unit": "s",
  "decimals": 0
}
```

---

### 应用指标 (7 个)

#### 9. HTTP 请求总数

**指标名称**: `http_requests_total`  
**PromQL**:
```promql
sum(rate(http_requests_total[5m])) by (method, status)
```

**仪表盘 Panel**:
```json
{
  "title": "HTTP Requests",
  "type": "graph",
  "targets": [
    {
      "expr": "sum(rate(http_requests_total[5m])) by (method, status)",
      "legendFormat": "{{method}} - {{status}}"
    }
  ]
}
```

---

#### 10. HTTP 请求时延 (P99)

**指标名称**: `http_request_duration_seconds`  
**PromQL**:
```promql
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))
```

**告警阈值**:
- Warning: >250ms (持续 5 分钟)
- Critical: >500ms (持续 2 分钟)

**仪表盘 Panel**:
```json
{
  "title": "Request Latency (P99)",
  "type": "graph",
  "targets": [
    {
      "expr": "histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))"
    }
  ],
  "yAxes": [{"format": "s"}],
  "alert": {
    "conditions": [
      {"evaluator": {"type": "gt", "params": [0.25]}}
    ]
  }
}
```

---

#### 11. HTTP 请求时延 (P50)

**指标名称**: `http_request_duration_seconds_p50`  
**PromQL**:
```promql
histogram_quantile(0.50, rate(http_request_duration_seconds_bucket[5m]))
```

**仪表盘 Panel**:
```json
{
  "title": "Request Latency (P50)",
  "type": "graph",
  "targets": [
    {
      "expr": "histogram_quantile(0.50, rate(http_request_duration_seconds_bucket[5m]))"
    }
  ],
  "yAxes": [{"format": "s"}]
}
```

---

#### 12. JVM 内存使用

**指标名称**: `jvm_memory_used_bytes`  
**PromQL**:
```promql
sum(jvm_memory_used_bytes{area="heap"}) / sum(jvm_memory_max_bytes{area="heap"}) * 100
```

**告警阈值**:
- Warning: >80% (持续 5 分钟)

**仪表盘 Panel**:
```json
{
  "title": "JVM Heap Usage",
  "type": "gauge",
  "targets": [
    {
      "expr": "sum(jvm_memory_used_bytes{area=\"heap\"}) / sum(jvm_memory_max_bytes{area=\"heap\"}) * 100"
    }
  ],
  "thresholds": [
    {"value": 0, "color": "green"},
    {"value": 80, "color": "yellow"},
    {"value": 90, "color": "red"}
  ]
}
```

---

#### 13. GC 暂停时间

**指标名称**: `jvm_gc_pause_seconds`  
**PromQL**:
```promql
rate(jvm_gc_pause_seconds_sum[5m]) / rate(jvm_gc_pause_seconds_count[5m])
```

**告警阈值**:
- Warning: >500ms (持续 5 分钟)

**仪表盘 Panel**:
```json
{
  "title": "GC Pause Time",
  "type": "graph",
  "targets": [
    {
      "expr": "rate(jvm_gc_pause_seconds_sum[5m]) / rate(jvm_gc_pause_seconds_count[5m])"
    }
  ],
  "yAxes": [{"format": "s"}]
}
```

---

#### 14. 执行队列大小

**指标名称**: `executor_queue_size`  
**PromQL**:
```promql
cgas_executor_queue_size
```

**告警阈值**:
- Warning: >1000 (持续 5 分钟)

**仪表盘 Panel**:
```json
{
  "title": "Executor Queue Size",
  "type": "graph",
  "targets": [
    {
      "expr": "cgas_executor_queue_size"
    }
  ],
  "alert": {
    "conditions": [
      {"evaluator": {"type": "gt", "params": [1000]}}
    ]
  }
}
```

---

### 数据库指标 (5 个)

#### 15. 数据库状态

**指标名称**: `pg_up`  
**PromQL**:
```promql
pg_up
```

**告警阈值**:
- Critical: =0 (立即)

**仪表盘 Panel**:
```json
{
  "title": "Database Status",
  "type": "stat",
  "targets": [
    {
      "expr": "pg_up",
      "legendFormat": "{{instance}}"
    }
  ],
  "thresholds": [
    {"value": 0, "color": "red"},
    {"value": 1, "color": "green"}
  ],
  "alert": {
    "conditions": [
      {"evaluator": {"type": "eq", "params": [0]}}
    ]
  }
}
```

---

#### 16. 活跃连接数

**指标名称**: `pg_stat_activity_count`  
**PromQL**:
```promql
sum(pg_stat_activity_count{datname="cgas_alpha"})
```

**告警阈值**:
- Warning: >150 (持续 5 分钟)

**仪表盘 Panel**:
```json
{
  "title": "Active Connections",
  "type": "graph",
  "targets": [
    {
      "expr": "sum(pg_stat_activity_count{datname=\"cgas_alpha\"})"
    }
  ],
  "alert": {
    "conditions": [
      {"evaluator": {"type": "gt", "params": [150]}}
    ]
  }
}
```

---

#### 17. 慢查询数量

**指标名称**: `pg_slow_queries`  
**PromQL**:
```promql
rate(pg_stat_statements_seconds{total_time>1000}[5m])
```

**告警阈值**:
- Warning: >10/min (持续 5 分钟)

**仪表盘 Panel**:
```json
{
  "title": "Slow Queries",
  "type": "graph",
  "targets": [
    {
      "expr": "rate(pg_slow_queries[5m])"
    }
  ],
  "yAxes": [{"format": "reqps"}],
  "alert": {
    "conditions": [
      {"evaluator": {"type": "gt", "params": [10]}}
    ]
  }
}
```

---

#### 18. 锁等待数量

**指标名称**: `pg_locks_count`  
**PromQL**:
```promql
sum(pg_locks_count{mode=~"ExclusiveLock|RowExclusiveLock"})
```

**告警阈值**:
- Warning: >100 (持续 5 分钟)

**仪表盘 Panel**:
```json
{
  "title": "Lock Waits",
  "type": "graph",
  "targets": [
    {
      "expr": "sum(pg_locks_count{mode=~\"ExclusiveLock|RowExclusiveLock\"})"
    }
  ],
  "alert": {
    "conditions": [
      {"evaluator": {"type": "gt", "params": [100]}}
    ]
  }
}
```

---

#### 19. 数据库吞吐量

**指标名称**: `pg_stat_database_tup_fetched`  
**PromQL**:
```promql
rate(pg_stat_database_tup_fetched{datname="cgas_alpha"}[5m])
```

**仪表盘 Panel**:
```json
{
  "title": "Database Throughput",
  "type": "graph",
  "targets": [
    {
      "expr": "rate(pg_stat_database_tup_fetched{datname=\"cgas_alpha\"}[5m])"
    }
  ],
  "yAxes": [{"format": "ops"}]
}
```

---

### 业务指标 (补充)

#### 20. 指令执行速率

**指标名称**: `cgas_commands_executed_total`  
**PromQL**:
```promql
rate(cgas_commands_executed_total[5m])
```

**仪表盘 Panel**:
```json
{
  "title": "Commands Executed",
  "type": "graph",
  "targets": [
    {
      "expr": "rate(cgas_commands_executed_total[5m])"
    }
  ],
  "yAxes": [{"format": "ops"}]
}
```

---

## 🚨 告警规则配置 (10 个)

### Alertmanager 配置

**配置文件**: `/etc/alertmanager/alertmanager.yml`

```yaml
global:
  resolve_timeout: 5m
  smtp_smarthost: 'smtp.example.com:587'
  smtp_from: 'alertmanager@cgas.internal'
  smtp_auth_username: 'alertmanager@cgas.internal'
  smtp_auth_password: '***'

route:
  group_by: ['alertname', 'severity']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'default-receiver'
  routes:
    - match:
        severity: critical
      receiver: 'critical-receiver'
      continue: true
    - match:
        severity: warning
      receiver: 'warning-receiver'

receivers:
  - name: 'default-receiver'
    webhook_configs:
      - url: 'http://10.0.1.10:5001/webhook'
        send_resolved: true

  - name: 'critical-receiver'
    webhook_configs:
      - url: 'http://10.0.1.10:5001/webhook/critical'
        send_resolved: true
    email_configs:
      - to: 'oncall@cgas.internal'
        send_resolved: true

  - name: 'warning-receiver'
    webhook_configs:
      - url: 'http://10.0.1.10:5001/webhook/warning'
        send_resolved: true
    email_configs:
      - to: 'team@cgas.internal'
        send_resolved: true

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'instance']
```

---

### 告警规则文件

**配置文件**: `/etc/prometheus/rules/alpha_alerts.yml`

```yaml
groups:
  - name: alpha_system_alerts
    rules:
      - alert: HighCPUUsage
        expr: 100 - (avg by(instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage on {{ $labels.instance }}"
          description: "CPU usage is {{ $value }}% (threshold: 80%)"
          runbook_url: "https://wiki.cgas.internal/runbooks/high-cpu"

      - alert: CriticalCPUUsage
        expr: 100 - (avg by(instance) (irate(node_cpu_seconds_total{mode="idle"}[2m])) * 100) > 95
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Critical CPU usage on {{ $labels.instance }}"
          description: "CPU usage is {{ $value }}% (threshold: 95%)"
          runbook_url: "https://wiki.cgas.internal/runbooks/critical-cpu"

      - alert: HighMemoryUsage
        expr: (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100 > 85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage on {{ $labels.instance }}"
          description: "Memory usage is {{ $value }}% (threshold: 85%)"
          runbook_url: "https://wiki.cgas.internal/runbooks/high-memory"

      - alert: HighDiskUsage
        expr: (1 - (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"})) * 100 > 80
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High disk usage on {{ $labels.instance }}"
          description: "Disk usage is {{ $value }}% (threshold: 80%)"
          runbook_url: "https://wiki.cgas.internal/runbooks/high-disk"

      - alert: HighLoad
        expr: node_load1 > 8
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High system load on {{ $labels.instance }}"
          description: "Load 1m: {{ $value }} (threshold: 8)"
          runbook_url: "https://wiki.cgas.internal/runbooks/high-load"

  - name: alpha_application_alerts
    rules:
      - alert: HighRequestLatency
        expr: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) > 0.25
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High request latency"
          description: "P99 latency is {{ $value }}s (threshold: 250ms)"
          runbook_url: "https://wiki.cgas.internal/runbooks/high-latency"

      - alert: CriticalRequestLatency
        expr: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[2m])) > 0.5
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Critical request latency"
          description: "P99 latency is {{ $value }}s (threshold: 500ms)"
          runbook_url: "https://wiki.cgas.internal/runbooks/critical-latency"

      - alert: HighErrorRate
        expr: sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m])) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate"
          description: "Error rate is {{ $value | humanizePercentage }} (threshold: 5%)"
          runbook_url: "https://wiki.cgas.internal/runbooks/high-error-rate"

      - alert: HighJVMHeapUsage
        expr: sum(jvm_memory_used_bytes{area="heap"}) / sum(jvm_memory_max_bytes{area="heap"}) * 100 > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High JVM heap usage"
          description: "JVM heap usage is {{ $value }}% (threshold: 80%)"
          runbook_url: "https://wiki.cgas.internal/runbooks/high-jvm-heap"

      - alert: ExecutorQueueFull
        expr: cgas_executor_queue_size > 1000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Executor queue is filling up"
          description: "Queue size: {{ $value }} (threshold: 1000)"
          runbook_url: "https://wiki.cgas.internal/runbooks/executor-queue"

  - name: alpha_database_alerts
    rules:
      - alert: DatabaseDown
        expr: pg_up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Database is down"
          description: "PostgreSQL on {{ $labels.instance }} is not responding"
          runbook_url: "https://wiki.cgas.internal/runbooks/database-down"

      - alert: HighDatabaseConnections
        expr: sum(pg_stat_activity_count{datname="cgas_alpha"}) > 150
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High database connections"
          description: "Active connections: {{ $value }} (threshold: 150)"
          runbook_url: "https://wiki.cgas.internal/runbooks/high-db-connections"

      - alert: SlowQueries
        expr: rate(pg_slow_queries[5m]) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High number of slow queries"
          description: "Slow queries: {{ $value }}/min (threshold: 10/min)"
          runbook_url: "https://wiki.cgas.internal/runbooks/slow-queries"

      - alert: HighLockWaits
        expr: sum(pg_locks_count{mode=~"ExclusiveLock|RowExclusiveLock"}) > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High number of lock waits"
          description: "Lock waits: {{ $value }} (threshold: 100)"
          runbook_url: "https://wiki.cgas.internal/runbooks/high-lock-waits"
```

---

## 📈 Grafana 仪表盘配置

### 仪表盘 1: 系统概览 (System Overview)

**UID**: `alpha-system-overview`  
**刷新间隔**: 30s  
**时间范围**: 最近 1 小时

**Panel 布局**:
```
┌─────────────┬─────────────┬─────────────┬─────────────┐
│   CPU (%)   │  Memory (%) │  Disk (%)   │   Uptime    │
│   Gauge     │   Gauge     │   Gauge     │    Stat     │
└─────────────┴─────────────┴─────────────┴─────────────┘
┌───────────────────────────────────────────────────────┐
│              Load Average (1m, 5m, 15m)               │
│                      Graph                            │
└───────────────────────────────────────────────────────┘
┌───────────────────────────┬───────────────────────────┐
│    Network Inbound        │    Network Outbound       │
│        Graph              │        Graph              │
└───────────────────────────┴───────────────────────────┘
┌───────────────────────────────────────────────────────┐
│              File Descriptor Usage                    │
│                      Graph                            │
└───────────────────────────────────────────────────────┘
```

---

### 仪表盘 2: 应用性能监控 (Application Performance)

**UID**: `alpha-app-performance`  
**刷新间隔**: 15s  
**时间范围**: 最近 30 分钟

**Panel 布局**:
```
┌─────────────┬─────────────┬─────────────┬─────────────┐
│  QPS        │  P99 (ms)   │  P50 (ms)   │  Error Rate │
│   Stat      │   Stat      │   Stat      │   Stat      │
└─────────────┴─────────────┴─────────────┴─────────────┘
┌───────────────────────────────────────────────────────┐
│              HTTP Requests by Status                  │
│                      Graph                            │
└───────────────────────────────────────────────────────┘
┌───────────────────────────┬───────────────────────────┐
│    Request Latency        │    JVM Heap Usage         │
│   (P50, P90, P99)         │        Gauge              │
│        Graph              │                           │
└───────────────────────────┴───────────────────────────┘
┌───────────────────────────────────────────────────────┐
│              GC Pause Time                            │
│                      Graph                            │
└───────────────────────────────────────────────────────┘
┌───────────────────────────────────────────────────────┐
│              Executor Queue Size                      │
│                      Graph                            │
└───────────────────────────────────────────────────────┘
```

---

### 仪表盘 3: 数据库监控 (Database Monitoring)

**UID**: `alpha-database`  
**刷新间隔**: 30s  
**时间范围**: 最近 1 小时

**Panel 布局**:
```
┌─────────────┬─────────────┬─────────────┬─────────────┐
│   Status    │ Connections │  Throughput │ Slow Queries│
│   Stat      │   Stat      │   Stat      │   Stat      │
└─────────────┴─────────────┴─────────────┴─────────────┘
┌───────────────────────────────────────────────────────┐
│              Active Connections                       │
│                      Graph                            │
└───────────────────────────────────────────────────────┘
┌───────────────────────────┬───────────────────────────┐
│    Database Throughput    │    Slow Queries/min       │
│        Graph              │        Graph              │
└───────────────────────────┴───────────────────────────┘
┌───────────────────────────────────────────────────────┐
│              Lock Waits                               │
│                      Graph                            │
└───────────────────────────────────────────────────────┘
```

---

### 仪表盘 4: 告警状态 (Alert Status)

**UID**: `alpha-alerts`  
**刷新间隔**: 30s  
**时间范围**: 最近 6 小时

**Panel 布局**:
```
┌───────────────────────────────────────────────────────┐
│              Active Alerts                            │
│                    Table                              │
│  Alert Name | Severity | Instance | Duration | Value  │
└───────────────────────────────────────────────────────┘
┌───────────────────────────┬───────────────────────────┐
│  Alerts by Severity       │  Alert Timeline           │
│        Pie Chart          │        Graph              │
└───────────────────────────┴───────────────────────────┘
┌───────────────────────────────────────────────────────┐
│              Alert History (24h)                      │
│                      Table                            │
└───────────────────────────────────────────────────────┘
```

---

### 仪表盘 5: 业务指标 (Business Metrics)

**UID**: `alpha-business`  
**刷新间隔**: 60s  
**时间范围**: 最近 1 小时

**Panel 布局**:
```
┌─────────────┬─────────────┬─────────────┬─────────────┐
│ Commands    │ Verifications│ Blockings  │ Success Rate│
│   Stat      │   Stat      │   Stat      │   Stat      │
└─────────────┴─────────────┴─────────────┴─────────────┘
┌───────────────────────────────────────────────────────┐
│              Commands Executed (5m rate)              │
│                      Graph                            │
└───────────────────────────────────────────────────────┘
┌───────────────────────────┬───────────────────────────┐
│  Verification Success     │  Blocking Rate            │
│        Graph              │        Graph              │
└───────────────────────────┴───────────────────────────┘
```

---

## 📧 告警通知策略

### 通知渠道配置

| 渠道 | 配置 | 适用告警级别 | 响应 SLA |
|---|---|---|---|
| Webhook | http://10.0.1.10:5001/webhook | 全部 | - |
| 邮件 | oncall@cgas.internal | Critical | 5 分钟 |
| 邮件 | team@cgas.internal | Warning | 1 小时 |
| 短信 | (集成中) | Critical | 5 分钟 |
| 电话 | (集成中) | Critical | 5 分钟 |

### 告警分级与响应

| 级别 | 颜色 | 通知渠道 | 响应 SLA | 升级策略 |
|---|---|---|---|---|
| Critical | 红色 | 邮件 + 短信 + 电话 | 5 分钟 | 15 分钟未响应升级 |
| Warning | 黄色 | 邮件 + 消息 | 1 小时 | 4 小时未响应升级 |
| Info | 蓝色 | 消息 | 24 小时 | - |

### 告警静默规则

```yaml
# 计划维护期间静默
- matchers:
    - alertname =~ ".+"
  startsAt: '2026-04-15T02:00:00Z'
  endsAt: '2026-04-15T04:00:00Z'
  createdBy: 'maintenance'
  comment: 'Planned maintenance window'

# 测试告警静默
- matchers:
    - alertname = "TestAlert"
  startsAt: '2026-04-02T00:00:00Z'
  endsAt: '2026-12-31T23:59:59Z'
  createdBy: 'sre-agent'
  comment: 'Test alert silence'
```

---

## 🔧 监控运维手册

### 日常巡检清单

| 检查项 | 频率 | 方法 | 标准 |
|---|---|---|---|
| Prometheus 状态 | 每日 | systemctl status prometheus | Active |
| Grafana 状态 | 每日 | systemctl status grafana-server | Active |
| Alertmanager 状态 | 每日 | systemctl status alertmanager | Active |
| 监控目标状态 | 每日 | 检查 Targets 页面 | 全部 UP |
| 告警规则状态 | 每日 | 检查 Rules 页面 | 全部正常 |
| 存储空间使用 | 每周 | 检查 TSDB 大小 | <80% |
| 告警历史分析 | 每周 | 检查 Alerts 页面 | 无重复告警 |

### 常见问题处理

#### 问题 1: Prometheus 目标 DOWN

**排查步骤**:
```bash
# 1. 检查目标服务器
ssh 10.0.1.11
systemctl status node_exporter

# 2. 检查网络连通性
curl -s http://10.0.1.11:9100/metrics

# 3. 检查防火墙规则
sudo ufw status | grep 9100

# 4. 重启 Node Exporter
sudo systemctl restart node_exporter
```

---

#### 问题 2: 告警未触发

**排查步骤**:
```bash
# 1. 检查告警规则
curl -s http://localhost:9090/api/v1/rules | jq

# 2. 检查 Alertmanager 状态
systemctl status alertmanager

# 3. 检查 Alertmanager 日志
journalctl -u alertmanager -n 50

# 4. 测试告警 (手动触发)
curl -X POST http://localhost:9093/api/v2/alerts \
  -H "Content-Type: application/json" \
  -d '[{"labels":{"alertname":"TestAlert","severity":"warning"}}]'
```

---

#### 问题 3: Grafana 仪表盘加载慢

**排查步骤**:
```bash
# 1. 检查 Prometheus 查询性能
curl -s 'http://localhost:9090/api/v1/query_range?query=up&start=2026-04-02T00:00:00Z&end=2026-04-02T12:00:00Z&step=15s' | jq '.status'

# 2. 优化 PromQL 查询 (减少时间范围、增加步长)
# 3. 检查 Grafana 日志
journalctl -u grafana-server -n 50

# 4. 清理过期数据 (保留 7 天)
curl -X POST http://localhost:9090/api/v1/admin/tsdb/delete_series \
  -d 'match[]=up' -d 'start=2026-03-26T00:00:00Z' -d 'end=2026-03-27T00:00:00Z'
```

---

## ✅ 监控配置验收清单

### Prometheus 配置验收

- [x] Prometheus 服务运行正常
- [x] 4 个监控目标全部 UP
- [x] 20 个监控指标配置完成
- [x] 告警规则文件加载成功
- [x] TSDB 存储配置正常

### Alertmanager 配置验收

- [x] Alertmanager 服务运行正常
- [x] 10 个告警规则配置完成
- [x] 通知渠道配置完成
- [x] 告警路由配置正常
- [x] 告警静默规则配置完成

### Grafana 配置验收

- [x] Grafana 服务运行正常
- [x] 5 个仪表盘配置完成
- [x] 数据源配置正常
- [x] 告警面板配置完成
- [x] 用户权限配置完成

### 告警测试验收

- [x] CPU 告警测试通过
- [x] 内存告警测试通过
- [x] 磁盘告警测试通过
- [x] 数据库宕机告警测试通过
- [x] 高延迟告警测试通过
- [x] 通知渠道测试通过

---

## 📚 附录

### 参考文档

| 文档 | 路径 | 状态 |
|---|---|---|
| alpha_environment_config.md | 本文档同目录 | ✅ 已交付 |
| prometheus_docs | https://prometheus.io/docs/ | ✅ 参考 |
| grafana_docs | https://grafana.com/docs/ | ✅ 参考 |
| alertmanager_docs | https://prometheus.io/docs/alerting/ | ✅ 参考 |

### 配置文件备份

所有配置文件已备份到:
- `/backup/monitoring/prometheus.yml`
- `/backup/monitoring/alertmanager.yml`
- `/backup/monitoring/alpha_alerts.yml`
- `/backup/monitoring/grafana-dashboards/`

### 变更记录

| 版本 | 日期 | 变更内容 | 变更人 |
|---|---|---|---|
| v1.0 | 2026-04-05 | 初始版本 | SRE-Agent |

---

**监控状态**: ✅ Alpha 环境监控配置完成  
**配置日期**: 2026-04-05  
**责任人**: SRE-Agent + Observability-Agent  
**验收人**: QA-Agent  
**保管**: 项目文档库  
**分发**: 全体 Agent 团队、运维团队

---

*Alpha Environment Monitoring Configuration v1.0 - 2026-04-05*
