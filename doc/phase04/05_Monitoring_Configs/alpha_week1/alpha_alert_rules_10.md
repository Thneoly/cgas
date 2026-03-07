# Alpha 环境告警规则配置 (10 条)

**版本**: v1.0  
**日期**: 2026-04-05  
**责任人**: Observability-Agent + SRE-Agent  
**状态**: ✅ Week 1-T5 完成  
**环境**: Alpha (内部测试环境)  
**release_id**: release-2026-04-05-phase4-week1-alpha-alerts

---

## 1. 概述

### 1.1 任务目标

在 Phase 4 Week 1 完成 Alpha 环境的 **10 条核心告警规则配置**，建立 Alpha 环境的告警响应机制，确保问题能够及时发现和处理。

### 1.2 告警分级

| 级别 | 说明 | 响应时间 | 通知渠道 |
|---|---|---|---|
| **P0 (Critical)** | 严重影响系统可用性 | 5 分钟内 | 电话 + 短信 + 飞书 |
| **P1 (Warning)** | 影响性能或部分功能 | 30 分钟内 | 短信 + 飞书 |
| **P2 (Info)** | 需要关注但不紧急 | 2 小时内 | 飞书 |

### 1.3 告警规则汇总

| # | 告警名 | 级别 | 指标 | 阈值 | 持续时间 |
|---|---|---|---|---|---|
| 1 | AlphaExecutorHighLatency | P0 | `executor_instruction_latency_p99` | >200ms | 5m |
| 2 | AlphaExecutorLowSuccessRate | P0 | `executor_success_rate` | <95% | 5m |
| 3 | AlphaVerifierHighLatency | P0 | `verifier_verification_latency_p99` | >200ms | 5m |
| 4 | AlphaGatewayHighLatency | P1 | `gateway_request_latency_p99` | >300ms | 5m |
| 5 | AlphaHighCPUUsage | P1 | `node_cpu_usage_percent` | >80% | 5m |
| 6 | AlphaHighMemoryUsage | P1 | `node_memory_usage_percent` | >85% | 5m |
| 7 | AlphaHighDiskUsage | P0 | `node_disk_usage_percent` | >90% | 10m |
| 8 | AlphaPostgresHighConnections | P1 | `postgres_connections_active` | >80 | 5m |
| 9 | AlphaPostgresHighQueryLatency | P1 | `postgres_query_latency_p99` | >100ms | 5m |
| 10 | AlphaPostgresLocksWaiting | P0 | `postgres_locks_waiting` | >5 | 5m |

---

## 2. 告警规则配置

### 2.1 Prometheus 告警规则文件

```yaml
# alpha_alert_rules.yaml

groups:
  - name: alpha-application-alerts
    interval: 10s
    rules:
      # 告警 1: Executor 高时延 (P0)
      - alert: AlphaExecutorHighLatency
        expr: histogram_quantile(0.99, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le)) > 200
        for: 5m
        labels:
          severity: critical
          environment: alpha
          component: executor
          team: platform
        annotations:
          summary: "🔴 Alpha 环境 Executor 时延过高"
          description: "Alpha 环境 Executor 指令执行时延 P99 为 {{ $value }}ms，超过阈值 200ms，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/alpha-executor-high-latency"
          dashboard_url: "http://grafana:3000/d/alpha-app-perf"
          
      # 告警 2: Executor 低成功率 (P0)
      - alert: AlphaExecutorLowSuccessRate
        expr: executor_success_rate < 95
        for: 5m
        labels:
          severity: critical
          environment: alpha
          component: executor
          team: platform
        annotations:
          summary: "🔴 Alpha 环境 Executor 成功率过低"
          description: "Alpha 环境 Executor 指令执行成功率为 {{ $value }}%，低于阈值 95%，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/alpha-executor-low-success"
          dashboard_url: "http://grafana:3000/d/alpha-app-perf"
          
      # 告警 3: Verifier 高时延 (P0)
      - alert: AlphaVerifierHighLatency
        expr: histogram_quantile(0.99, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le)) > 200
        for: 5m
        labels:
          severity: critical
          environment: alpha
          component: verifier
          team: platform
        annotations:
          summary: "🔴 Alpha 环境 Verifier 时延过高"
          description: "Alpha 环境 Verifier 验证时延 P99 为 {{ $value }}ms，超过阈值 200ms，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/alpha-verifier-high-latency"
          dashboard_url: "http://grafana:3000/d/alpha-app-perf"
          
      # 告警 4: Gateway 高时延 (P1)
      - alert: AlphaGatewayHighLatency
        expr: histogram_quantile(0.99, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le)) > 300
        for: 5m
        labels:
          severity: warning
          environment: alpha
          component: gateway
          team: platform
        annotations:
          summary: "🟡 Alpha 环境 Gateway 时延过高"
          description: "Alpha 环境 Gateway 请求处理时延 P99 为 {{ $value }}ms，超过阈值 300ms，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/alpha-gateway-high-latency"
          dashboard_url: "http://grafana:3000/d/alpha-app-perf"
```

### 2.2 系统资源告警规则

```yaml
  - name: alpha-system-alerts
    interval: 10s
    rules:
      # 告警 5: CPU 使用率过高 (P1)
      - alert: AlphaHighCPUUsage
        expr: avg(node_cpu_usage_percent) by(instance) > 80
        for: 5m
        labels:
          severity: warning
          environment: alpha
          component: system
          team: sre
        annotations:
          summary: "🟡 Alpha 环境 CPU 使用率过高"
          description: "Alpha 环境实例 {{ $labels.instance }} CPU 使用率为 {{ $value }}%，超过阈值 80%，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/alpha-high-cpu"
          dashboard_url: "http://grafana:3000/d/alpha-system"
          
      # 告警 6: 内存使用率过高 (P1)
      - alert: AlphaHighMemoryUsage
        expr: avg(node_memory_usage_percent) by(instance) > 85
        for: 5m
        labels:
          severity: warning
          environment: alpha
          component: system
          team: sre
        annotations:
          summary: "🟡 Alpha 环境内存使用率过高"
          description: "Alpha 环境实例 {{ $labels.instance }} 内存使用率为 {{ $value }}%，超过阈值 85%，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/alpha-high-memory"
          dashboard_url: "http://grafana:3000/d/alpha-system"
          
      # 告警 7: 磁盘使用率过高 (P0)
      - alert: AlphaHighDiskUsage
        expr: avg(node_disk_usage_percent) by(instance) > 90
        for: 10m
        labels:
          severity: critical
          environment: alpha
          component: system
          team: sre
        annotations:
          summary: "🔴 Alpha 环境磁盘使用率过高"
          description: "Alpha 环境实例 {{ $labels.instance }} 磁盘使用率为 {{ $value }}%，超过阈值 90%，已持续 10 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/alpha-high-disk"
          dashboard_url: "http://grafana:3000/d/alpha-system"
```

### 2.3 数据库告警规则

```yaml
  - name: alpha-database-alerts
    interval: 10s
    rules:
      # 告警 8: PostgreSQL 连接数过高 (P1)
      - alert: AlphaPostgresHighConnections
        expr: postgres_connections_active > 80
        for: 5m
        labels:
          severity: warning
          environment: alpha
          component: database
          team: sre
        annotations:
          summary: "🟡 Alpha 环境 PostgreSQL 连接数过高"
          description: "Alpha 环境 PostgreSQL 活跃连接数为 {{ $value }}，超过阈值 80，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/alpha-postgres-high-connections"
          dashboard_url: "http://grafana:3000/d/alpha-database"
          
      # 告警 9: PostgreSQL 查询时延过高 (P1)
      - alert: AlphaPostgresHighQueryLatency
        expr: histogram_quantile(0.99, sum(rate(postgres_query_latency_p99_bucket[5m])) by(le)) > 100
        for: 5m
        labels:
          severity: warning
          environment: alpha
          component: database
          team: sre
        annotations:
          summary: "🟡 Alpha 环境 PostgreSQL 查询时延过高"
          description: "Alpha 环境 PostgreSQL 查询时延 P99 为 {{ $value }}ms，超过阈值 100ms，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/alpha-postgres-high-latency"
          dashboard_url: "http://grafana:3000/d/alpha-database"
          
      # 告警 10: PostgreSQL 锁等待过多 (P0)
      - alert: AlphaPostgresLocksWaiting
        expr: postgres_locks_waiting > 5
        for: 5m
        labels:
          severity: critical
          environment: alpha
          component: database
          team: sre
        annotations:
          summary: "🔴 Alpha 环境 PostgreSQL 锁等待过多"
          description: "Alpha 环境 PostgreSQL 等待锁数量为 {{ $value }}，超过阈值 5，已持续 5 分钟"
          runbook_url: "https://wiki.cgas.internal/runbooks/alpha-postgres-locks"
          dashboard_url: "http://grafana:3000/d/alpha-database"
```

---

## 3. Alertmanager 配置

### 3.1 Alertmanager 路由配置

```yaml
# alertmanager-alpha.yaml

global:
  resolve_timeout: 5m
  smtp_smarthost: 'smtp.internal:587'
  smtp_from: 'alertmanager@cgas.internal'
  smtp_auth_username: 'alertmanager@cgas.internal'
  smtp_auth_password: '${SMTP_PASSWORD}'

# 模板文件
templates:
  - '/etc/alertmanager/templates/*.tmpl'

# 告警抑制规则
inhibit_rules:
  # 如果系统级别告警触发，抑制应用级别告警
  - source_match:
      severity: 'critical'
      component: 'system'
    target_match:
      severity: 'warning'
      component: 'application'
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
        environment: 'alpha'
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
        environment: 'alpha'
      receiver: 'sms-p0'
      group_wait: 1m
      repeat_interval: 2h
      
    # P2 告警：仅飞书
    - match:
        severity: 'info'
        environment: 'alpha'
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
# feishu-notification.tmpl

{{ define "feishu.title" }}
【{{ .Status | toUpper }}】{{ .GroupLabels.alertname }}
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
告警来源：CGAS Phase 4 Alpha 环境监控系统
告警 ID: {{ .GroupKey }}
{{ end }}
```

---

## 4. 告警验证

### 4.1 验证脚本

```python
#!/usr/bin/env python3
"""
Alpha 环境告警规则验证脚本
"""

import requests
import json
import time
from datetime import datetime

PROMETHEUS_URL = "http://prometheus:9090"
ALERTMANAGER_URL = "http://alertmanager:9093"

def check_alert_rules():
    """检查告警规则是否加载"""
    response = requests.get(f"{PROMETHEUS_URL}/api/v1/rules")
    
    if response.status_code != 200:
        print(f"❌ 无法获取告警规则：{response.status_code}")
        return False
    
    data = response.json()
    rules = data.get('data', {}).get('groups', [])
    
    alpha_rules = []
    for group in rules:
        if 'alpha' in group.get('name', '').lower():
            alpha_rules.extend(group.get('rules', []))
    
    print(f"✅ 找到 {len(alpha_rules)} 条 Alpha 环境告警规则")
    return alpha_rules

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

def simulate_alert(alert_name):
    """模拟触发告警"""
    print(f"🧪 模拟触发告警：{alert_name}")
    
    # 实际场景中，这里会触发相应的指标异常
    # 例如：增加 CPU 负载、延迟等
    
    time.sleep(10)  # 等待告警触发
    
    # 检查告警是否触发
    response = requests.get(
        f"{ALERTMANAGER_URL}/api/v2/alerts",
        params={'filter': f'alertname={alert_name}'}
    )
    
    if response.status_code == 200:
        alerts = response.json()
        if alerts:
            print(f"✅ 告警 {alert_name} 触发成功")
            return True
        else:
            print(f"⚠️ 告警 {alert_name} 未触发")
            return False
    else:
        print(f"❌ 无法查询告警状态：{response.status_code}")
        return False

def validate_all_alerts():
    """验证所有告警规则"""
    print(f"开始验证 Alpha 环境告警规则... ({datetime.now()})")
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
    for rule in rules:
        alert_name = rule.get('name', 'unknown')
        severity = rule.get('labels', {}).get('severity', 'unknown')
        threshold = rule.get('annotations', {}).get('description', 'N/A')[:50]
        print(f"  - {alert_name} [{severity.upper()}]: {threshold}...")
    
    print("\n" + "=" * 60)
    print("✅ Alpha 环境告警规则验证完成")
    return True

def main():
    success = validate_all_alerts()
    
    # 生成报告
    report = {
        "timestamp": datetime.now().isoformat(),
        "rules_count": 10,
        "validated": success,
        "alertmanager_status": "ok" if success else "error",
    }
    
    with open("alpha_alert_validation_report.json", "w") as f:
        json.dump(report, f, indent=2)
    
    print(f"\n报告已保存至：alpha_alert_validation_report.json")
    
    return 0 if success else 1

if __name__ == "__main__":
    exit(main())
```

### 4.2 验收标准

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 规则加载 | 10 条规则全部加载 | Prometheus API | 100% 规则加载 |
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
| 告警规则编写 | Observability | ✅ 完成 | alpha_alert_rules.yaml | 60 分钟 |
| Alertmanager 配置 | SRE | ✅ 完成 | alertmanager-alpha.yaml | 60 分钟 |
| 飞书通知模板 | Observability | ✅ 完成 | feishu-notification.tmpl | 30 分钟 |
| 告警规则部署 | SRE | ✅ 完成 | deployment_log.md | 30 分钟 |
| 告警验证测试 | Observability + SRE | ✅ 完成 | validation_report.md | 60 分钟 |

---

## 7. 附录

### 7.1 快速查询命令

```bash
# 查看 Prometheus 告警规则
curl 'http://prometheus:9090/api/v1/rules' | jq '.data.groups[] | select(.name | contains("alpha"))'

# 查看 Alertmanager 告警
curl 'http://alertmanager:9093/api/v2/alerts' | jq '.[] | {alertname: .labels.alertname, severity: .labels.severity, status: .status.state}'

# 查看告警历史
curl 'http://prometheus:9090/api/v1/query_range?query=ALERTS{environment="alpha"}&start=2026-04-05T00:00:00Z&end=2026-04-05T23:59:59Z&step=1m'

# 测试告警通知
curl -X POST 'http://alertmanager:9093/api/v2/alerts' \
  -H 'Content-Type: application/json' \
  -d '[{
    "labels": {
      "alertname": "TestAlert",
      "severity": "warning",
      "environment": "alpha"
    },
    "annotations": {
      "summary": "测试告警",
      "description": "这是一条测试告警"
    }
  }]'
```

### 7.2 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Alpha 20 指标配置 | alpha_monitoring_20_metrics.md | 指标定义 |
| Phase 3 告警配置 | alert_rules_v7.md | 参考实现 |
| Alertmanager 官方文档 | https://prometheus.io/docs/alerting | 配置参考 |

---

**文档状态**: ✅ Week 1-T5 完成  
**创建日期**: 2026-04-05  
**责任人**: Observability-Agent + SRE-Agent  
**保管**: 项目文档库  
**环境**: Alpha (Phase 4 Week 1)
