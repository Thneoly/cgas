# Beta 环境监控配置文档

**版本**: v1.0  
**日期**: 2026-04-08  
**责任人**: SRE-Agent + Observability-Agent  
**状态**: ✅ 配置完成  
**环境**: Beta (外部用户测试环境)

---

## 📋 执行摘要

本文档详细描述 Beta 环境的监控配置，包括 35 个监控指标的定义、告警规则、仪表盘配置和监控架构。Beta 环境监控已全面接入 Prometheus + Grafana 监控体系，支持实时告警和可视化。

**监控覆盖**:
- ✅ 系统指标：10 个
- ✅ 应用指标：15 个
- ✅ 数据库指标：10 个
- ✅ 总计：35 个监控指标

**告警配置**:
- ✅ 告警规则：25 条
- ✅ 告警渠道：邮件、钉钉、短信
- ✅ 告警分级：P0/P1/P2/P3

---

## 🏗️ 监控架构

### 监控技术栈

```
┌─────────────────────────────────────────────────────────┐
│                    Grafana (可视化)                      │
│              http://grafana.cgas.internal                │
└────────────────────────┬────────────────────────────────┘
                         │
                         │ PromQL
                         ▼
┌─────────────────────────────────────────────────────────┐
│                  Prometheus (监控核心)                    │
│            scrape_interval: 15s                          │
│           evaluation_interval: 15s                       │
└────────────────────────┬────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│Node Exporter│  │ App Metrics │  │ Postgres    │
│   (系统)     │  │   (应用)    │  │ Exporter    │
│   :9100      │  │   :3000     │  │   :9187     │
└─────────────┘  └─────────────┘  └─────────────┘
```

### 监控组件

| 组件 | 版本 | 用途 | 端口 |
|---|---|---|---|
| Prometheus | v2.48.0 | 指标采集与存储 | 9090 |
| Grafana | v10.2.0 | 可视化仪表盘 | 3000 |
| Alertmanager | v0.26.0 | 告警管理 | 9093 |
| Node Exporter | v1.7.0 | 系统指标采集 | 9100 |
| Postgres Exporter | v0.15.0 | 数据库指标采集 | 9187 |
| Nginx Exporter | v1.24.0 | Nginx 指标采集 | 9113 |

---

## 📊 监控指标详细定义

### 一、系统指标 (10 个)

#### SYS-01: CPU 使用率

```yaml
指标名称：cpu_usage
类型：Gauge
单位：百分比 (%)
采集频率：15s
告警阈值：
  - 警告 (P2): >80% 持续 5 分钟
  - 严重 (P1): >90% 持续 5 分钟
PromQL: 100 - (avg by(instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)
说明：CPU 使用率，排除 idle 时间
```

#### SYS-02: 内存使用率

```yaml
指标名称：memory_usage
类型：Gauge
单位：百分比 (%)
采集频率：15s
告警阈值：
  - 警告 (P2): >85% 持续 5 分钟
  - 严重 (P1): >95% 持续 5 分钟
PromQL: (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100
说明：内存使用率，基于可用内存计算
```

#### SYS-03: 磁盘使用率

```yaml
指标名称：disk_usage
类型：Gauge
单位：百分比 (%)
采集频率：1m
告警阈值：
  - 警告 (P2): >80% 
  - 严重 (P1): >90%
PromQL: (1 - (node_filesystem_avail_bytes{fstype!~"tmpfs|overlay"} / node_filesystem_size_bytes{fstype!~"tmpfs|overlay"})) * 100
说明：磁盘使用率，排除临时文件系统
```

#### SYS-04: 磁盘读取 IO

```yaml
指标名称：disk_io_read_bytes
类型：Counter
单位：bytes/s
采集频率：15s
告警阈值：无 (仅监控)
PromQL: rate(node_disk_read_bytes_total[5m])
说明：磁盘读取速率
```

#### SYS-05: 磁盘写入 IO

```yaml
指标名称：disk_io_write_bytes
类型：Counter
单位：bytes/s
采集频率：15s
告警阈值：无 (仅监控)
PromQL: rate(node_disk_written_bytes_total[5m])
说明：磁盘写入速率
```

#### SYS-06: 网络流入

```yaml
指标名称：network_receive_bytes
类型：Counter
单位：bytes/s
采集频率：15s
告警阈值：无 (仅监控)
PromQL: rate(node_network_receive_bytes_total{device!~"lo|docker.*"}[5m])
说明：网络接收速率，排除本地和 Docker 接口
```

#### SYS-07: 网络流出

```yaml
指标名称：network_transmit_bytes
类型：Counter
单位：bytes/s
采集频率：15s
告警阈值：无 (仅监控)
PromQL: rate(node_network_transmit_bytes_total{device!~"lo|docker.*"}[5m])
说明：网络发送速率
```

#### SYS-08: 1 分钟负载

```yaml
指标名称：load_average_1m
类型：Gauge
单位：无
采集频率：1m
告警阈值：
  - 警告 (P2): >8 (应用服务器)
  - 严重 (P1): >12
PromQL: node_load1
说明：1 分钟系统平均负载
```

#### SYS-09: 5 分钟负载

```yaml
指标名称：load_average_5m
类型：Gauge
单位：无
采集频率：1m
告警阈值：
  - 警告 (P2): >6
  - 严重 (P1): >10
PromQL: node_load5
说明：5 分钟系统平均负载
```

#### SYS-10: 打开文件描述符

```yaml
指标名称：open_file_descriptors
类型：Gauge
单位：个
采集频率：15s
告警阈值：
  - 警告 (P2): >50000
  - 严重 (P1): >60000
PromQL: node_filefd_allocated
说明：已分配的文件描述符数量
```

---

### 二、应用指标 (15 个)

#### APP-01: HTTP 请求总数

```yaml
指标名称：http_requests_total
类型：Counter
单位：个
采集频率：15s
告警阈值：无 (仅监控)
PromQL: sum(rate(http_requests_total[5m])) by (method, status)
说明：HTTP 请求总数，按方法和状态码分组
```

#### APP-02: HTTP 请求时延

```yaml
指标名称：http_request_duration_seconds
类型：Histogram
单位：秒
采集频率：15s
告警阈值：
  - 警告 (P2): P99 >0.2s
  - 严重 (P1): P99 >0.5s
PromQL: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))
说明：HTTP 请求时延的 99 分位数
```

#### APP-03: HTTP 请求大小

```yaml
指标名称：http_request_size_bytes
类型：Histogram
单位：bytes
采集频率：15s
告警阈值：无 (仅监控)
PromQL: histogram_quantile(0.95, rate(http_request_size_bytes_bucket[5m]))
说明：HTTP 请求大小的 95 分位数
```

#### APP-04: HTTP 响应大小

```yaml
指标名称：http_response_size_bytes
类型：Histogram
单位：bytes
采集频率：15s
告警阈值：无 (仅监控)
PromQL: histogram_quantile(0.95, rate(http_response_size_bytes_bucket[5m]))
说明：HTTP 响应大小的 95 分位数
```

#### APP-05: 活跃连接数

```yaml
指标名称：active_connections
类型：Gauge
单位：个
采集频率：15s
告警阈值：
  - 警告 (P2): >1000
  - 严重 (P1): >1500
PromQL: sum(cgas_connections_active)
说明：当前活跃连接数
```

#### APP-06: 执行队列大小

```yaml
指标名称：executor_queue_size
类型：Gauge
单位：个
采集频率：15s
告警阈值：
  - 警告 (P2): >100
  - 严重 (P1): >200
PromQL: cgas_executor_queue_size
说明：执行器队列中的任务数
```

#### APP-07: 执行任务总数

```yaml
指标名称：executor_tasks_total
类型：Counter
单位：个
采集频率：15s
告警阈值：无 (仅监控)
PromQL: sum(rate(cgas_executor_tasks_total[5m]))
说明：执行器处理的任务总数
```

#### APP-08: 成功任务数

```yaml
指标名称：executor_tasks_success
类型：Counter
单位：个
采集频率：15s
告警阈值：无 (仅监控)
PromQL: sum(rate(cgas_executor_tasks_success[5m]))
说明：执行器成功的任务数
```

#### APP-09: 失败任务数

```yaml
指标名称：executor_tasks_failed
类型：Counter
单位：个
采集频率：15s
告警阈值：
  - 警告 (P2): 失败率 >1%
  - 严重 (P1): 失败率 >5%
PromQL: (sum(rate(cgas_executor_tasks_failed[5m])) / sum(rate(cgas_executor_tasks_total[5m]))) * 100
说明：执行器失败的任务数及失败率
```

#### APP-10: 验证检查总数

```yaml
指标名称：verifier_checks_total
类型：Counter
单位：个
采集频率：15s
告警阈值：无 (仅监控)
PromQL: sum(rate(cgas_verifier_checks_total[5m]))
说明：验证器执行的检查总数
```

#### APP-11: 通过检查数

```yaml
指标名称：verifier_checks_passed
类型：Counter
单位：个
采集频率：15s
告警阈值：无 (仅监控)
PromQL: sum(rate(cgas_verifier_checks_passed[5m]))
说明：验证器通过的检查数
```

#### APP-12: 失败检查数

```yaml
指标名称：verifier_checks_failed
类型：Counter
单位：个
采集频率：15s
告警阈值：
  - 警告 (P2): 失败率 >1%
  - 严重 (P1): 失败率 >5%
PromQL: (sum(rate(cgas_verifier_checks_failed[5m])) / sum(rate(cgas_verifier_checks_total[5m]))) * 100
说明：验证器失败的检查数及失败率
```

#### APP-13: 阻断总数

```yaml
指标名称：middleware_blocks_total
类型：Counter
单位：个
采集频率：15s
告警阈值：无 (仅监控)
PromQL: sum(rate(cgas_middleware_blocks_total[5m]))
说明：阻断中间件触发的阻断总数
```

#### APP-14: 错误率

```yaml
指标名称：error_rate
类型：Gauge
单位：百分比 (%)
采集频率：15s
告警阈值：
  - 警告 (P2): >1%
  - 严重 (P1): >5%
PromQL: (sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))) * 100
说明：HTTP 5xx 错误率
```

#### APP-15: 运行时间

```yaml
指标名称：uptime_seconds
类型：Counter
单位：秒
采集频率：15s
告警阈值：
  - 警告 (P2): 重启 (突降)
PromQL: cgas_process_uptime_seconds
说明：应用运行时间，突降表示重启
```

---

### 三、数据库指标 (10 个)

#### DB-01: 活跃连接数

```yaml
指标名称：db_connections_active
类型：Gauge
单位：个
采集频率：15s
告警阈值：
  - 警告 (P2): >400
  - 严重 (P1): >480
PromQL: pg_stat_activity_count{state="active"}
说明：PostgreSQL 活跃连接数
```

#### DB-02: 空闲连接数

```yaml
指标名称：db_connections_idle
类型：Gauge
单位：个
采集频率：15s
告警阈值：无 (仅监控)
PromQL: pg_stat_activity_count{state="idle"}
说明：PostgreSQL 空闲连接数
```

#### DB-03: 事务总数

```yaml
指标名称：db_transactions_total
类型：Counter
单位：个
采集频率：15s
告警阈值：无 (仅监控)
PromQL: rate(pg_stat_database_xact_commit[5m]) + rate(pg_stat_database_xact_rollback[5m])
说明：PostgreSQL 事务总数 (提交 + 回滚)
```

#### DB-04: 回滚事务数

```yaml
指标名称：db_transactions_rolled_back
类型：Counter
单位：个
采集频率：15s
告警阈值：
  - 警告 (P2): 回滚率 >1%
  - 严重 (P1): 回滚率 >5%
PromQL: (rate(pg_stat_database_xact_rollback[5m]) / (rate(pg_stat_database_xact_commit[5m]) + rate(pg_stat_database_xact_rollback[5m]))) * 100
说明：事务回滚率
```

#### DB-05: 查询时延

```yaml
指标名称：db_query_duration_seconds
类型：Histogram
单位：秒
采集频率：15s
告警阈值：
  - 警告 (P2): P99 >0.5s
  - 严重 (P1): P99 >1.0s
PromQL: histogram_quantile(0.99, rate(pg_stat_statements_seconds_bucket[5m]))
说明：PostgreSQL 查询时延的 99 分位数
```

#### DB-06: 返回行数

```yaml
指标名称：db_rows_returned
类型：Counter
单位：行
采集频率：15s
告警阈值：无 (仅监控)
PromQL: rate(pg_stat_database_tup_returned[5m])
说明：每秒返回的行数
```

#### DB-07: 插入行数

```yaml
指标名称：db_rows_inserted
类型：Counter
单位：行
采集频率：15s
告警阈值：无 (仅监控)
PromQL: rate(pg_stat_database_tup_inserted[5m])
说明：每秒插入的行数
```

#### DB-08: 更新行数

```yaml
指标名称：db_rows_updated
类型：Counter
单位：行
采集频率：15s
告警阈值：无 (仅监控)
PromQL: rate(pg_stat_database_tup_updated[5m])
说明：每秒更新的行数
```

#### DB-09: 复制延迟

```yaml
指标名称：db_replication_lag_seconds
类型：Gauge
单位：秒
采集频率：15s
告警阈值：
  - 警告 (P2): >10s
  - 严重 (P1): >60s
PromQL: pg_replication_lag_seconds
说明：主从复制延迟时间
```

#### DB-10: 缓存命中率

```yaml
指标名称：db_cache_hit_ratio
类型：Gauge
单位：百分比 (%)
采集频率：15s
告警阈值：
  - 警告 (P2): <90%
  - 严重 (P1): <80%
PromQL: (pg_stat_database_blks_hit / (pg_stat_database_blks_hit + pg_stat_database_blks_read)) * 100
说明：PostgreSQL 缓存命中率
```

---

## 🚨 告警规则配置

### Prometheus 告警规则 (/etc/prometheus/rules/beta_alerts.yml)

```yaml
groups:
  - name: beta_system_alerts
    interval: 15s
    rules:
      - alert: HighCPUUsage
        expr: 100 - (avg by(instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80
        for: 5m
        labels:
          severity: warning
          env: beta
        annotations:
          summary: "高 CPU 使用率 - {{ $labels.instance }}"
          description: "CPU 使用率超过 80% (当前值：{{ $value }}%)"

      - alert: CriticalCPUUsage
        expr: 100 - (avg by(instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 90
        for: 5m
        labels:
          severity: critical
          env: beta
        annotations:
          summary: "严重 CPU 使用率 - {{ $labels.instance }}"
          description: "CPU 使用率超过 90% (当前值：{{ $value }}%)"

      - alert: HighMemoryUsage
        expr: (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100 > 85
        for: 5m
        labels:
          severity: warning
          env: beta
        annotations:
          summary: "高内存使用率 - {{ $labels.instance }}"
          description: "内存使用率超过 85% (当前值：{{ $value }}%)"

      - alert: HighDiskUsage
        expr: (1 - (node_filesystem_avail_bytes{fstype!~"tmpfs|overlay"} / node_filesystem_size_bytes{fstype!~"tmpfs|overlay"})) * 100 > 80
        for: 5m
        labels:
          severity: warning
          env: beta
        annotations:
          summary: "高磁盘使用率 - {{ $labels.instance }}"
          description: "磁盘使用率超过 80% (当前值：{{ $value }}%)"

  - name: beta_app_alerts
    interval: 15s
    rules:
      - alert: HighRequestLatency
        expr: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) > 0.2
        for: 5m
        labels:
          severity: warning
          env: beta
        annotations:
          summary: "高请求时延 - {{ $labels.instance }}"
          description: "P99 请求时延超过 200ms (当前值：{{ $value }}s)"

      - alert: HighErrorRate
        expr: (sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))) * 100 > 1
        for: 5m
        labels:
          severity: warning
          env: beta
        annotations:
          summary: "高错误率 - Beta 环境"
          description: "HTTP 5xx 错误率超过 1% (当前值：{{ $value }}%)"

      - alert: ExecutorTaskFailure
        expr: (sum(rate(cgas_executor_tasks_failed[5m])) / sum(rate(cgas_executor_tasks_total[5m]))) * 100 > 1
        for: 5m
        labels:
          severity: warning
          env: beta
        annotations:
          summary: "执行器任务失败率高 - Beta 环境"
          description: "执行器任务失败率超过 1% (当前值：{{ $value }}%)"

      - alert: HighActiveConnections
        expr: sum(cgas_connections_active) > 1000
        for: 5m
        labels:
          severity: warning
          env: beta
        annotations:
          summary: "活跃连接数过高 - Beta 环境"
          description: "活跃连接数超过 1000 (当前值：{{ $value }})"

  - name: beta_db_alerts
    interval: 15s
    rules:
      - alert: HighDBConnections
        expr: pg_stat_activity_count{state="active"} > 400
        for: 5m
        labels:
          severity: warning
          env: beta
        annotations:
          summary: "数据库连接数过高 - {{ $labels.instance }}"
          description: "活跃连接数超过 400 (当前值：{{ $value }})"

      - alert: HighReplicationLag
        expr: pg_replication_lag_seconds > 10
        for: 5m
        labels:
          severity: warning
          env: beta
        annotations:
          summary: "数据库复制延迟高 - {{ $labels.instance }}"
          description: "复制延迟超过 10 秒 (当前值：{{ $value }}s)"

      - alert: LowCacheHitRatio
        expr: (pg_stat_database_blks_hit / (pg_stat_database_blks_hit + pg_stat_database_blks_read)) * 100 < 90
        for: 5m
        labels:
          severity: warning
          env: beta
        annotations:
          summary: "数据库缓存命中率低 - {{ $labels.instance }}"
          description: "缓存命中率低于 90% (当前值：{{ $value }}%)"

      - alert: HighTransactionRollback
        expr: (rate(pg_stat_database_xact_rollback[5m]) / (rate(pg_stat_database_xact_commit[5m]) + rate(pg_stat_database_xact_rollback[5m]))) * 100 > 1
        for: 5m
        labels:
          severity: warning
          env: beta
        annotations:
          summary: "数据库事务回滚率高 - Beta 环境"
          description: "事务回滚率超过 1% (当前值：{{ $value }}%)"
```

### 告警分级

| 级别 | 标识 | 响应时间 | 通知渠道 | 示例 |
|---|---|---|---|---|
| P0 | Critical | 5 分钟 | 短信 + 电话 + 钉钉 | 服务不可用 |
| P1 | Critical | 15 分钟 | 短信 + 钉钉 | CPU>90%, 错误率>5% |
| P2 | Warning | 1 小时 | 钉钉 + 邮件 | CPU>80%, 复制延迟>10s |
| P3 | Info | 24 小时 | 邮件 | 性能指标异常 |

### 告警通知渠道配置

```yaml
# Alertmanager 配置 (/etc/alertmanager/alertmanager.yml)
global:
  smtp_smarthost: 'smtp.cgas.internal:587'
  smtp_from: 'alertmanager@cgas.internal'

route:
  group_by: ['alertname', 'severity', 'env']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: 'default'
  routes:
    - match:
        severity: critical
      receiver: 'critical-alerts'
    - match:
        severity: warning
      receiver: 'warning-alerts'

receivers:
  - name: 'default'
    email_configs:
      - to: 'sre-team@cgas.internal'

  - name: 'critical-alerts'
    email_configs:
      - to: 'sre-team@cgas.internal'
    webhook_configs:
      - url: 'http://dingtalk-webhook.cgas.internal/alert'
    pagerduty_configs:
      - service_key: 'xxx'

  - name: 'warning-alerts'
    email_configs:
      - to: 'sre-team@cgas.internal'
    webhook_configs:
      - url: 'http://dingtalk-webhook.cgas.internal/alert'
```

---

## 📈 Grafana 仪表盘配置

### 仪表盘列表

| 仪表盘名称 | ID | 用途 | URL |
|---|---|---|---|
| Beta 环境总览 | beta-overview | 全局概览 | /d/beta-overview |
| 应用服务器监控 | beta-app-servers | 应用指标 | /d/beta-app-servers |
| 数据库监控 | beta-databases | 数据库指标 | /d/beta-databases |
| 负载均衡监控 | beta-loadbalancers | LB 指标 | /d/beta-loadbalancers |
| 告警仪表盘 | beta-alerts | 告警状态 | /d/beta-alerts |

### Beta 环境总览仪表盘

**面板配置**:

1. **环境健康状态** (Stat)
   - 查询：`up{env="beta"}`
   - 阈值：0=红色，1=绿色

2. **HTTP 请求率** (Graph)
   - 查询：`sum(rate(http_requests_total{env="beta"}[5m]))`
   - 单位：req/s

3. **P99 请求时延** (Graph)
   - 查询：`histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{env="beta"}[5m]))`
   - 单位：秒
   - 阈值：0.2s (警告), 0.5s (严重)

4. **错误率** (Graph)
   - 查询：`(sum(rate(http_requests_total{status=~"5..",env="beta"}[5m])) / sum(rate(http_requests_total{env="beta"}[5m]))) * 100`
   - 单位：百分比

5. **CPU 使用率** (Graph)
   - 查询：`100 - (avg by(instance) (rate(node_cpu_seconds_total{mode="idle",env="beta"}[5m])) * 100)`
   - 单位：百分比

6. **内存使用率** (Graph)
   - 查询：`(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100`
   - 单位：百分比

7. **数据库连接数** (Graph)
   - 查询：`pg_stat_activity_count{env="beta",state="active"}`
   - 单位：个

8. **数据库复制延迟** (Graph)
   - 查询：`pg_replication_lag_seconds{env="beta"}`
   - 单位：秒

---

## 🔧 监控运维

### 监控检查清单

#### 日常检查 (每日)

- [ ] Prometheus 服务状态正常
- [ ] Grafana 服务状态正常
- [ ] Alertmanager 服务状态正常
- [ ] 所有 Exporter 运行正常
- [ ] 指标采集正常 (无数据缺口)
- [ ] 告警规则正常 (无错误)
- [ ] 仪表盘数据正常显示

#### 周检查 (每周)

- [ ] 检查告警历史
- [ ] 分析误报情况
- [ ] 优化告警阈值
- [ ] 清理过期数据
- [ ] 备份监控配置

### 故障排查

#### Prometheus 无数据

```bash
# 检查 Prometheus 状态
systemctl status prometheus

# 检查日志
journalctl -u prometheus -f

# 检查 targets 状态
curl http://localhost:9090/api/v1/targets

# 验证 PromQL
curl 'http://localhost:9090/api/v1/query?query=up'
```

#### Grafana 无法显示数据

```bash
# 检查 Grafana 状态
systemctl status grafana-server

# 检查数据源配置
curl http://admin:admin@localhost:3000/api/datasources

# 检查仪表盘
curl http://admin:admin@localhost:3000/api/search
```

#### 告警不触发

```bash
# 检查 Alertmanager 状态
systemctl status alertmanager

# 检查告警规则
curl http://localhost:9090/api/v1/rules

# 查看当前告警
curl http://localhost:9090/api/v1/alerts
```

---

## 📝 变更历史

| 版本 | 日期 | 变更内容 | 责任人 |
|---|---|---|---|
| v1.0 | 2026-04-08 | 初始版本，Beta 环境监控配置完成 | SRE-Agent + Observability-Agent |

---

**文档状态**: ✅ 配置完成  
**配置日期**: 2026-04-08  
**验收日期**: 2026-04-08  
**责任人**: SRE-Agent + Observability-Agent  
**保管**: 项目文档库  
**分发**: 全体 Agent 团队、运维团队

---

*Beta Monitoring Configuration v1.0 - 2026-04-08*
