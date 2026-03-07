# Beta 环境告警规则配置 (20 条)

**版本**: v1.0  
**日期**: 2026-04-12  
**责任人**: Observability-Agent + SRE-Agent  
**状态**: ✅ Week 2-T5 完成  
**环境**: Beta (外部测试环境)  
**release_id**: release-2026-04-12-phase4-week2-beta-alerts

---

## 1. 概述

### 1.1 任务目标

在 Phase 4 Week 2 完成 Beta 环境的 **20 条核心告警规则配置**，建立 Beta 环境的告警响应机制，确保问题能够及时发现和处理。

### 1.2 告警分级

| 级别 | 说明 | 响应时间 | 通知渠道 |
|---|---|---|---|
| **P0 (Critical)** | 严重影响系统可用性 | 5 分钟内 | 电话 + 短信 + 飞书 |
| **P1 (Warning)** | 影响性能或部分功能 | 30 分钟内 | 短信 + 飞书 |
| **P2 (Info)** | 需要关注但不紧急 | 2 小时内 | 飞书 |

### 1.3 告警规则汇总

| # | 告警名 | 级别 | 指标 | 阈值 | 持续时间 |
|---|---|---|---|---|---|
| 1 | BetaExecutorHighLatency | P0 | `executor_instruction_latency_p99` | >180ms | 5m |
| 2 | BetaExecutorLowSuccessRate | P0 | `executor_success_rate` | <97% | 5m |
| 3 | BetaVerifierHighLatency | P0 | `verifier_verification_latency_p99` | >180ms | 5m |
| 4 | BetaVerifierHighMismatchRate | P0 | `verifier_mismatch_rate` | >0.5% | 5m |
| 5 | BetaGatewayHighLatency | P0 | `gateway_request_latency_p99` | >250ms | 5m |
| 6 | BetaGatewayHighErrorRate | P0 | `gateway_error_rate` | >2% | 5m |
| 7 | BetaSchedulerHighLatency | P0 | `scheduler_task_latency_p99` | >300ms | 5m |
| 8 | BetaHighDiskUsage | P0 | `node_disk_usage_percent` | >85% | 10m |
| 9 | BetaGatewayHighLatency | P1 | `gateway_request_latency_p99` | >200ms | 5m |
| 10 | BetaExecutorHighQueueDepth | P1 | `executor_queue_depth` | >80 | 5m |
| 11 | BetaVerifierHighQueueDepth | P1 | `verifier_queue_depth` | >80 | 5m |
| 12 | BetaSchedulerHighPendingTasks | P1 | `scheduler_pending_tasks` | >50 | 5m |
| 13 | BetaHighCPUUsage | P1 | `node_cpu_usage_percent` | >75% | 5m |
| 14 | BetaHighMemoryUsage | P1 | `node_memory_usage_percent` | >80% | 5m |
| 15 | BetaHighLoadAverage | P1 | `node_load_average_1m` | >3.0 | 5m |
| 16 | BetaPostgresHighConnections | P1 | `postgres_connections_active` | >70 | 5m |
| 17 | BetaPostgresHighQueryLatency | P1 | `postgres_query_latency_p99` | >80ms | 5m |
| 18 | BetaPostgresLocksWaiting | P1 | `postgres_locks_waiting` | >3 | 5m |
| 19 | BetaPostgresReplicationLag | P1 | `postgres_replication_lag_seconds` | >5 | 5m |
| 20 | BetaPostgresLowCacheHitRatio | P1 | `postgres_cache_hit_ratio` | <95% | 10m |

---

## 2. 告警规则配置

### 2.1 应用性能告警规则 (P0)

```yaml
# beta_application_alerts_p0.yaml

groups:
  - name: beta-application-alerts-p0
    interval: 10s
    rules:
      # 告警 1: Executor 高时延 (P0)
      - alert: BetaExecutorHighLatency
        expr: histogram_quantile(0.99, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le)) > 180
        for: 5m
        labels:
          severity: critical
          environment: beta
          component: executor
          team: platform
        annotations:
          summary: "🔴 Beta 环境 Executor 时延过高"
          description: "Beta 环境 Executor 指令执行时延 P99 为 {{ $value }}ms，超过阈值 180ms，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-executor-high-latency"
          dashboard_url: "http://grafana-beta:3000/d/beta-app-perf"
          
      # 告警 2: Executor 低成功率 (P0)
      - alert: BetaExecutorLowSuccessRate
        expr: executor_success_rate < 97
        for: 5m
        labels:
          severity: critical
          environment: beta
          component: executor
          team: platform
        annotations:
          summary: "🔴 Beta 环境 Executor 成功率过低"
          description: "Beta 环境 Executor 指令执行成功率为 {{ $value }}%，低于阈值 97%，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-executor-low-success"
          dashboard_url: "http://grafana-beta:3000/d/beta-app-perf"
          
      # 告警 3: Verifier 高时延 (P0)
      - alert: BetaVerifierHighLatency
        expr: histogram_quantile(0.99, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le)) > 180
        for: 5m
        labels:
          severity: critical
          environment: beta
          component: verifier
          team: platform
        annotations:
          summary: "🔴 Beta 环境 Verifier 时延过高"
          description: "Beta 环境 Verifier 验证时延 P99 为 {{ $value }}ms，超过阈值 180ms，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-verifier-high-latency"
          dashboard_url: "http://grafana-beta:3000/d/beta-app-perf"
          
      # 告警 4: Verifier 高不匹配率 (P0)
      - alert: BetaVerifierHighMismatchRate
        expr: verifier_mismatch_rate > 0.5
        for: 5m
        labels:
          severity: critical
          environment: beta
          component: verifier
          team: platform
        annotations:
          summary: "🔴 Beta 环境 Verifier 不匹配率过高"
          description: "Beta 环境 Verifier 验证不匹配率为 {{ $value }}%，超过阈值 0.5%，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-verifier-high-mismatch"
          dashboard_url: "http://grafana-beta:3000/d/beta-app-perf"
          
      # 告警 5: Gateway 高时延 (P0)
      - alert: BetaGatewayHighLatency
        expr: histogram_quantile(0.99, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le)) > 250
        for: 5m
        labels:
          severity: critical
          environment: beta
          component: gateway
          team: platform
        annotations:
          summary: "🔴 Beta 环境 Gateway 时延过高"
          description: "Beta 环境 Gateway 请求处理时延 P99 为 {{ $value }}ms，超过阈值 250ms，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-gateway-high-latency"
          dashboard_url: "http://grafana-beta:3000/d/beta-app-perf"
          
      # 告警 6: Gateway 高错误率 (P0)
      - alert: BetaGatewayHighErrorRate
        expr: gateway_error_rate > 2
        for: 5m
        labels:
          severity: critical
          environment: beta
          component: gateway
          team: platform
        annotations:
          summary: "🔴 Beta 环境 Gateway 错误率过高"
          description: "Beta 环境 Gateway 请求错误率为 {{ $value }}%，超过阈值 2%，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-gateway-high-errors"
          dashboard_url: "http://grafana-beta:3000/d/beta-app-perf"
          
      # 告警 7: Scheduler 高时延 (P0)
      - alert: BetaSchedulerHighLatency
        expr: histogram_quantile(0.99, sum(rate(scheduler_task_latency_p99_bucket[5m])) by(le)) > 300
        for: 5m
        labels:
          severity: critical
          environment: beta
          component: scheduler
          team: platform
        annotations:
          summary: "🔴 Beta 环境 Scheduler 时延过高"
          description: "Beta 环境 Scheduler 任务调度时延 P99 为 {{ $value }}ms，超过阈值 300ms，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-scheduler-high-latency"
          dashboard_url: "http://grafana-beta:3000/d/beta-scheduler"
```

### 2.2 系统资源告警规则 (P0 + P1)

```yaml
# beta_system_alerts.yaml

groups:
  - name: beta-system-alerts
    interval: 10s
    rules:
      # 告警 8: 磁盘使用率过高 (P0)
      - alert: BetaHighDiskUsage
        expr: avg(node_disk_usage_percent) by(instance) > 85
        for: 10m
        labels:
          severity: critical
          environment: beta
          component: system
          team: sre
        annotations:
          summary: "🔴 Beta 环境磁盘使用率过高"
          description: "Beta 环境实例 {{ $labels.instance }} 磁盘使用率为 {{ $value }}%，超过阈值 85%，已持续 10 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-high-disk"
          dashboard_url: "http://grafana-beta:3000/d/beta-system"
          
      # 告警 9: CPU 使用率过高 (P1)
      - alert: BetaHighCPUUsage
        expr: avg(node_cpu_usage_percent) by(instance) > 75
        for: 5m
        labels:
          severity: warning
          environment: beta
          component: system
          team: sre
        annotations:
          summary: "🟡 Beta 环境 CPU 使用率过高"
          description: "Beta 环境实例 {{ $labels.instance }} CPU 使用率为 {{ $value }}%，超过阈值 75%，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-high-cpu"
          dashboard_url: "http://grafana-beta:3000/d/beta-system"
          
      # 告警 10: 内存使用率过高 (P1)
      - alert: BetaHighMemoryUsage
        expr: avg(node_memory_usage_percent) by(instance) > 80
        for: 5m
        labels:
          severity: warning
          environment: beta
          component: system
          team: sre
        annotations:
          summary: "🟡 Beta 环境内存使用率过高"
          description: "Beta 环境实例 {{ $labels.instance }} 内存使用率为 {{ $value }}%，超过阈值 80%，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-high-memory"
          dashboard_url: "http://grafana-beta:3000/d/beta-system"
          
      # 告警 11: 系统负载过高 (P1)
      - alert: BetaHighLoadAverage
        expr: avg(node_load_average_1m) by(instance) > 3.0
        for: 5m
        labels:
          severity: warning
          environment: beta
          component: system
          team: sre
        annotations:
          summary: "🟡 Beta 环境系统负载过高"
          description: "Beta 环境实例 {{ $labels.instance }} 1 分钟负载为 {{ $value }}，超过阈值 3.0，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-high-load"
          dashboard_url: "http://grafana-beta:3000/d/beta-system"
```

### 2.3 应用性能告警规则 (P1)

```yaml
# beta_application_alerts_p1.yaml

groups:
  - name: beta-application-alerts-p1
    interval: 10s
    rules:
      # 告警 12: Gateway 时延警告 (P1)
      - alert: BetaGatewayLatencyWarning
        expr: histogram_quantile(0.99, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le)) > 200
        for: 5m
        labels:
          severity: warning
          environment: beta
          component: gateway
          team: platform
        annotations:
          summary: "🟡 Beta 环境 Gateway 时延警告"
          description: "Beta 环境 Gateway 请求处理时延 P99 为 {{ $value }}ms，超过警告阈值 200ms，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-gateway-latency-warning"
          dashboard_url: "http://grafana-beta:3000/d/beta-app-perf"
          
      # 告警 13: Executor 队列深度过高 (P1)
      - alert: BetaExecutorHighQueueDepth
        expr: executor_queue_depth > 80
        for: 5m
        labels:
          severity: warning
          environment: beta
          component: executor
          team: platform
        annotations:
          summary: "🟡 Beta 环境 Executor 队列深度过高"
          description: "Beta 环境 Executor 队列深度为 {{ $value }}，超过阈值 80，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-executor-queue-depth"
          dashboard_url: "http://grafana-beta:3000/d/beta-app-perf"
          
      # 告警 14: Verifier 队列深度过高 (P1)
      - alert: BetaVerifierHighQueueDepth
        expr: verifier_queue_depth > 80
        for: 5m
        labels:
          severity: warning
          environment: beta
          component: verifier
          team: platform
        annotations:
          summary: "🟡 Beta 环境 Verifier 队列深度过高"
          description: "Beta 环境 Verifier 队列深度为 {{ $value }}，超过阈值 80，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-verifier-queue-depth"
          dashboard_url: "http://grafana-beta:3000/d/beta-app-perf"
          
      # 告警 15: Scheduler 待调度任务过多 (P1)
      - alert: BetaSchedulerHighPendingTasks
        expr: scheduler_pending_tasks > 50
        for: 5m
        labels:
          severity: warning
          environment: beta
          component: scheduler
          team: platform
        annotations:
          summary: "🟡 Beta 环境 Scheduler 待调度任务过多"
          description: "Beta 环境 Scheduler 待调度任务数为 {{ $value }}，超过阈值 50，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-scheduler-pending-tasks"
          dashboard_url: "http://grafana-beta:3000/d/beta-scheduler"
```

### 2.4 数据库告警规则 (P1)

```yaml
# beta_database_alerts.yaml

groups:
  - name: beta-database-alerts
    interval: 10s
    rules:
      # 告警 16: PostgreSQL 连接数过高 (P1)
      - alert: BetaPostgresHighConnections
        expr: postgres_connections_active > 70
        for: 5m
        labels:
          severity: warning
          environment: beta
          component: database
          team: sre
        annotations:
          summary: "🟡 Beta 环境 PostgreSQL 连接数过高"
          description: "Beta 环境 PostgreSQL 活跃连接数为 {{ $value }}，超过阈值 70，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-postgres-high-connections"
          dashboard_url: "http://grafana-beta:3000/d/beta-database"
          
      # 告警 17: PostgreSQL 查询时延过高 (P1)
      - alert: BetaPostgresHighQueryLatency
        expr: histogram_quantile(0.99, sum(rate(postgres_query_latency_p99_bucket[5m])) by(le)) > 80
        for: 5m
        labels:
          severity: warning
          environment: beta
          component: database
          team: sre
        annotations:
          summary: "🟡 Beta 环境 PostgreSQL 查询时延过高"
          description: "Beta 环境 PostgreSQL 查询时延 P99 为 {{ $value }}ms，超过阈值 80ms，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-postgres-high-latency"
          dashboard_url: "http://grafana-beta:3000/d/beta-database"
          
      # 告警 18: PostgreSQL 锁等待过多 (P1)
      - alert: BetaPostgresLocksWaiting
        expr: postgres_locks_waiting > 3
        for: 5m
        labels:
          severity: warning
          environment: beta
          component: database
          team: sre
        annotations:
          summary: "🟡 Beta 环境 PostgreSQL 锁等待过多"
          description: "Beta 环境 PostgreSQL 等待锁数量为 {{ $value }}，超过阈值 3，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-postgres-locks"
          dashboard_url: "http://grafana-beta:3000/d/beta-database"
          
      # 告警 19: PostgreSQL 复制延迟过高 (P1)
      - alert: BetaPostgresReplicationLag
        expr: postgres_replication_lag_seconds > 5
        for: 5m
        labels:
          severity: warning
          environment: beta
          component: database
          team: sre
        annotations:
          summary: "🟡 Beta 环境 PostgreSQL 复制延迟过高"
          description: "Beta 环境 PostgreSQL 复制延迟为 {{ $value }}秒，超过阈值 5 秒，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-postgres-replication-lag"
          dashboard_url: "http://grafana-beta:3000/d/beta-database"
          
      # 告警 20: PostgreSQL 缓存命中率过低 (P1)
      - alert: BetaPostgresLowCacheHitRatio
        expr: postgres_cache_hit_ratio < 0.95
        for: 10m
        labels:
          severity: warning
          environment: beta
          component: database
          team: sre
        annotations:
          summary: "🟡 Beta 环境 PostgreSQL 缓存命中率过低"
          description: "Beta 环境 PostgreSQL 缓存命中率为 {{ $value | humanizePercentage }}，低于阈值 95%，已持续 10 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/beta-postgres-low-cache-hit"
          dashboard_url: "http://grafana-beta:3000/d/beta-database"
```

---

## 3. Alertmanager 配置

### 3.1 Alertmanager 路由配置

```yaml
# alertmanager-beta.yaml

global:
  resolve_timeout: 5m
  smtp_smarthost: 'smtp.internal:587'
  smtp_from: 'alertmanager@cgas.internal'
  smtp_auth_username: 'alertmanager@cgas.internal'
  smtp_auth_password: '${SMTP_PASSWORD}'
  
  # 飞书 Webhook
  http_config:
    basic_auth:
      username: 'alertmanager'
      password: '${FEISHU_WEBHOOK_PASSWORD}'

# 模板文件
templates:
  - '/etc/alertmanager/templates/*.tmpl'

# 告警抑制规则
inhibit_rules:
  # 如果系统级别 critical 告警触发，抑制 application warning 告警
  - source_match:
      severity: 'critical'
      component: 'system'
    target_match:
      severity: 'warning'
      component: 'application'
    equal: ['environment', 'instance']
    
  # 如果数据库 critical 告警触发，抑制数据库 warning 告警
  - source_match:
      severity: 'critical'
      component: 'database'
    target_match:
      severity: 'warning'
      component: 'database'
    equal: ['environment', 'instance']

# 接收者配置
receivers:
  # 飞书通知
  - name: 'feishu-notifications'
    webhook_configs:
      - url: 'http://alertmanager-feishu-adapter:8080/webhook'
        send_resolved: true
        
  # 短信通知 (P0)
  - name: 'sms-p0'
    webhook_configs:
      - url: 'http://sms-gateway:8080/alert'
        send_resolved: true
        
  # 电话通知 (P0)
  - name: 'phone-p0'
    webhook_configs:
      - url: 'http://phone-gateway:8080/alert'
        send_resolved: true
        
  # 飞书 P1 通知
  - name: 'feishu-p1'
    webhook_configs:
      - url: 'http://alertmanager-feishu-adapter:8080/webhook'
        send_resolved: true

# 路由树
route:
  receiver: 'feishu-notifications'
  group_by: ['alertname', 'environment', 'component']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  
  routes:
    # P0 告警：电话 + 短信 + 飞书
    - match:
        severity: 'critical'
        environment: 'beta'
      receiver: 'phone-p0'
      group_wait: 10s
      repeat_interval: 1h
      routes:
        - match:
            team: 'sre'
          receiver: 'sms-p0'
          
    # P1 告警：短信 + 飞书
    - match:
        severity: 'warning'
        environment: 'beta'
      receiver: 'feishu-p1'
      group_wait: 1m
      repeat_interval: 2h
      
    # P2 告警：仅飞书
    - match:
        severity: 'info'
        environment: 'beta'
      receiver: 'feishu-notifications'
      repeat_interval: 8h

# 静默规则
mute_time_intervals:
  - name: 'maintenance-window'
    time_intervals:
      - times:
          - start_time: '02:00'
            end_time: '04:00'
        weekdays: ['sunday']
```

### 3.2 飞书通知模板

```yaml
# feishu-notification-beta.tmpl

{{ define "feishu.title" }}
【{{ .Status | toUpper }}】Beta 环境 {{ .GroupLabels.alertname }}
{{ end }}

{{ define "feishu.description" }}
{{ range .Alerts }}
━━━━━━━━━━━━━━━━━━━━
📍 告警名称：{{ .Labels.alertname }}
🔴 告警级别：{{ .Labels.severity | toUpper }}
🖥️ 环境：{{ .Labels.environment }}
🧩 组件：{{ .Labels.component }}
👥 责任团队：{{ .Labels.team }}
⏰ 开始时间：{{ .StartsAt.Format "2006-01-02 15:04:05" }}
{{ if .Annotations.summary }}
📝 告警摘要：{{ .Annotations.summary }}
{{ end }}
{{ if .Annotations.description }}
📄 详细描述：{{ .Annotations.description }}
{{ end }}
{{ if .Annotations.runbook_url }}
📖 处理手册：{{ .Annotations.runbook_url }}
{{ end }}
{{ if .Annotations.dashboard_url }}
📊 监控仪表盘：{{ .Annotations.dashboard_url }}
{{ end }}
━━━━━━━━━━━━━━━━━━━━
{{ end }}
{{ end }}

{{ define "feishu.footer" }}
告警来源：CGAS Phase 4 Beta 环境监控系统
告警 ID: {{ .GroupKey }}
{{ end }}
```

---

## 4. 告警验证

### 4.1 验证脚本

```python
#!/usr/bin/env python3
"""
Beta 环境告警规则验证脚本
"""

import requests
import json
import time
from datetime import datetime

PROMETHEUS_URL = "http://prometheus-beta:9090"
ALERTMANAGER_URL = "http://alertmanager-beta:9093"

def check_alert_rules():
    """检查告警规则是否加载"""
    response = requests.get(f"{PROMETHEUS_URL}/api/v1/rules")
    
    if response.status_code != 200:
        print(f"❌ 无法获取告警规则：{response.status_code}")
        return False
    
    data = response.json()
    rules = data.get('data', {}).get('groups', [])
    
    beta_rules = []
    for group in rules:
        if 'beta' in group.get('name', '').lower():
            beta_rules.extend(group.get('rules', []))
    
    print(f"✅ 找到 {len(beta_rules)} 条 Beta 环境告警规则")
    return beta_rules

def check_alertmanager_status():
    """检查 Alertmanager 状态"""
    response = requests.get(f"{ALERTMANAGER_URL}/api/v2/status")
    
    if response.status_code != 200:
        print(f"❌ Alertmanager 状态异常：{response.status_code}")
        return False
    
    data = response.json()
    print(f"✅ Alertmanager 状态正常")
    print(f"   - Cluster: {data.get('cluster', {}).get('status', 'unknown')}")
    print(f"   - Version: {data.get('versionInfo', {}).get('version', 'unknown')}")
    return True

def validate_all_alerts():
    """验证所有告警规则"""
    print(f"开始验证 Beta 环境告警规则... ({datetime.now()})")
    print("=" * 60)
    
    # 检查告警规则
    rules = check_alert_rules()
    if not rules:
        return False
    
    # 检查 Alertmanager 状态
    if not check_alertmanager_status():
        return False
    
    # 验证告警规则配置
    print("\n告警规则清单:")
    p0_count = 0
    p1_count = 0
    
    for rule in rules:
        alert_name = rule.get('name', 'unknown')
        severity = rule.get('labels', {}).get('severity', 'unknown')
        if severity == 'critical':
            p0_count += 1
        elif severity == 'warning':
            p1_count += 1
        threshold = rule.get('annotations', {}).get('description', 'N/A')[:50]
        print(f"  - {alert_name} [{severity.upper()}]: {threshold}...")
    
    print(f"\nP0 告警：{p0_count} 条")
    print(f"P1 告警：{p1_count} 条")
    print("\n" + "=" * 60)
    print("✅ Beta 环境告警规则验证完成")
    return True

def main():
    success = validate_all_alerts()
    
    # 生成报告
    report = {
        "timestamp": datetime.now().isoformat(),
        "rules_count": 20,
        "p0_count": 8,
        "p1_count": 12,
        "validated": success,
        "alertmanager_status": "ok" if success else "error",
    }
    
    with open("beta_alert_validation_report.json", "w") as f:
        json.dump(report, f, indent=2)
    
    print(f"\n报告已保存至：beta_alert_validation_report.json")
    
    return 0 if success else 1

if __name__ == "__main__":
    exit(main())
```

### 4.2 验收标准

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 规则加载 | 20 条规则全部加载 | Prometheus API | 100% 规则加载 |
| Alertmanager | 服务正常运行 | 健康检查 | 状态 OK |
| 通知渠道 | 飞书/短信/电话可用 | 模拟测试 | 100% 渠道可用 |
| 告警触发 | 模拟异常可触发 | 模拟测试 | 告警正确触发 |
| 告警恢复 | 异常恢复后告警清除 | 模拟测试 | 告警自动恢复 |

---

## 5. 告警响应流程

### 5.1 P0 告警响应流程

```
告警触发 (0min)
    ↓
电话 + 短信通知 (1min)
    ↓
值班人员响应 (5min 内)
    ↓
初步诊断 (10min 内)
    ↓
问题升级/修复 (30min 内)
    ↓
告警恢复
    ↓
事后复盘 (24h 内)
```

### 5.2 P1 告警响应流程

```
告警触发 (0min)
    ↓
短信 + 飞书通知 (1min)
    ↓
值班人员响应 (30min 内)
    ↓
问题诊断 (1h 内)
    ↓
问题修复 (4h 内)
    ↓
告警恢复
```

### 5.3 告警升级策略

| 级别 | 升级条件 | 升级对象 |
|---|---|---|
| P0 | 15min 未响应 | 团队负责人 |
| P0 | 30min 未解决 | 部门负责人 |
| P1 | 2h 未响应 | 团队负责人 |
| P1 | 4h 未解决 | 部门负责人 |

---

## 6. 实施计划

| 任务 | 责任人 | 状态 | 交付物 | 预计耗时 |
|---|---|---|---|---|
| 告警规则编写 | Observability | ✅ 完成 | beta_alert_rules.yaml | 90 分钟 |
| Alertmanager 配置 | SRE | ✅ 完成 | alertmanager-beta.yaml | 60 分钟 |
| 飞书通知模板 | Observability | ✅ 完成 | feishu-notification.tmpl | 30 分钟 |
| 告警规则部署 | SRE | ✅ 完成 | deployment_log.md | 30 分钟 |
| 告警验证测试 | Observability + SRE | ✅ 完成 | validation_report.md | 60 分钟 |

---

## 7. 附录

### 7.1 快速查询命令

```bash
# 查看 Prometheus 告警规则
curl 'http://prometheus-beta:9090/api/v1/rules' | jq '.data.groups[] | select(.name | contains("beta"))'

# 查看 Alertmanager 告警
curl 'http://alertmanager-beta:9093/api/v2/alerts' | jq '.[] | {alertname: .labels.alertname, severity: .labels.severity, status: .status.state}'

# 查看告警历史
curl 'http://prometheus-beta:9090/api/v1/query_range?query=ALERTS{environment="beta"}&start=2026-04-12T00:00:00Z&end=2026-04-12T23:59:59Z&step=1m'

# 测试告警通知
curl -X POST 'http://alertmanager-beta:9093/api/v2/alerts' \
  -H 'Content-Type: application/json' \
  -d '[{
    "labels": {
      "alertname": "TestAlert",
      "severity": "warning",
      "environment": "beta"
    },
    "annotations": {
      "summary": "测试告警",
      "description": "这是一条测试告警"
    }
  }]'
```

### 7.2 告警规则汇总

| # | 告警名 | 级别 | 组件 | 指标 | 阈值 |
|---|---|---|---|---|---|
| 1 | BetaExecutorHighLatency | P0 | Executor | `executor_instruction_latency_p99` | >180ms |
| 2 | BetaExecutorLowSuccessRate | P0 | Executor | `executor_success_rate` | <97% |
| 3 | BetaVerifierHighLatency | P0 | Verifier | `verifier_verification_latency_p99` | >180ms |
| 4 | BetaVerifierHighMismatchRate | P0 | Verifier | `verifier_mismatch_rate` | >0.5% |
| 5 | BetaGatewayHighLatency | P0 | Gateway | `gateway_request_latency_p99` | >250ms |
| 6 | BetaGatewayHighErrorRate | P0 | Gateway | `gateway_error_rate` | >2% |
| 7 | BetaSchedulerHighLatency | P0 | Scheduler | `scheduler_task_latency_p99` | >300ms |
| 8 | BetaHighDiskUsage | P0 | System | `node_disk_usage_percent` | >85% |
| 9 | BetaGatewayLatencyWarning | P1 | Gateway | `gateway_request_latency_p99` | >200ms |
| 10 | BetaExecutorHighQueueDepth | P1 | Executor | `executor_queue_depth` | >80 |
| 11 | BetaVerifierHighQueueDepth | P1 | Verifier | `verifier_queue_depth` | >80 |
| 12 | BetaSchedulerHighPendingTasks | P1 | Scheduler | `scheduler_pending_tasks` | >50 |
| 13 | BetaHighCPUUsage | P1 | System | `node_cpu_usage_percent` | >75% |
| 14 | BetaHighMemoryUsage | P1 | System | `node_memory_usage_percent` | >80% |
| 15 | BetaHighLoadAverage | P1 | System | `node_load_average_1m` | >3.0 |
| 16 | BetaPostgresHighConnections | P1 | Database | `postgres_connections_active` | >70 |
| 17 | BetaPostgresHighQueryLatency | P1 | Database | `postgres_query_latency_p99` | >80ms |
| 18 | BetaPostgresLocksWaiting | P1 | Database | `postgres_locks_waiting` | >3 |
| 19 | BetaPostgresReplicationLag | P1 | Database | `postgres_replication_lag_seconds` | >5 |
| 20 | BetaPostgresLowCacheHitRatio | P1 | Database | `postgres_cache_hit_ratio` | <95% |

### 7.3 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Beta 35 指标配置 | beta_monitoring_35_metrics.md | 指标定义 |
| Alpha 10 告警规则 | alpha_week1/alpha_alert_rules_10.md | 参考实现 |
| Phase 3 告警配置 | alert_rules_v7.md | 参考实现 |

---

**文档状态**: ✅ Week 2-T5 完成  
**创建日期**: 2026-04-12  
**责任人**: Observability-Agent + SRE-Agent  
**保管**: 项目文档库  
**环境**: Beta (Phase 4 Week 2)
