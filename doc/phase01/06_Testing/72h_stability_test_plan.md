# Phase 3 Week 4: 72 小时稳定性测试方案

**版本**: v1.0  
**日期**: 2026-03-07  
**责任人**: SRE-Agent  
**状态**: 📋 Week 4 规划  
**release_id**: release-2026-03-07-phase3-week4-stability  
**测试周期**: 2026-03-08 ~ 2026-03-11 (72 小时)  
**参与角色**: SRE, Observability, Dev, QA

---

## 1. 概述

### 1.1 测试目标

在 Phase 3 Week 4 进行 72 小时连续稳定性测试，验证系统在扩展至 30 个监控指标后的长期运行稳定性，评估系统韧性能力和故障恢复能力。

### 1.2 测试范围

| 测试维度 | 测试内容 | 验收标准 |
|---|---|---|
| **长期稳定性** | 72h 连续运行无崩溃 | 零宕机，零数据丢失 |
| **性能稳定性** | 时延/吞吐量波动<10% | P99 时延<250ms |
| **资源稳定性** | 内存/CPU 无泄漏 | 内存增长<5% |
| **故障恢复** | 故障注入后自动恢复 | MTTR<5 分钟 |
| **告警有效性** | 告警准确触发 | 告警准确率>99% |

### 1.3 与 Week 3 基线对比

| 维度 | Week 3 | Week 4 测试 | 变化 |
|---|---|---|---|
| 监控指标数 | 20 个 | 30 个 | +10 个 (第三批) |
| 测试时长 | 7 天基线测量 | 72h 稳定性测试 | 专注稳定性 |
| 测试类型 | 被动观测 | 主动故障注入 | 增加故障测试 |
| 环境 | Staging + Production | Staging | 隔离环境 |

---

## 2. 测试环境配置

### 2.1 环境规格

| 组件 | 规格 | 数量 | 说明 |
|---|---|---|---|
| **Executor** | 4 核 8GB | 3 实例 | 无状态，可横向扩展 |
| **Verifier** | 4 核 8GB | 2 实例 | 无状态，可横向扩展 |
| **Gateway** | 2 核 4GB | 2 实例 | 负载均衡 |
| **Database** | 8 核 16GB | 1 主 2 从 | PostgreSQL 15 |
| **Cache** | 4 核 8GB | 3 节点 | Redis Cluster |
| **Message Queue** | 4 核 8GB | 3 节点 | Kafka Cluster |
| **Monitoring** | 8 核 16GB | 1 套 | Prometheus + Grafana |

### 2.2 测试负载

| 指标 | 目标值 | 说明 |
|---|---|---|
| **平均 QPS** | 5,000 | 高于生产平均 (4,120 QPS) |
| **高峰 QPS** | 10,000 | 达到设计承载 100% |
| **Batch 请求占比** | 20% | 模拟真实场景 |
| **Transaction 请求占比** | 15% | 模拟真实场景 |
| **错误率基线** | <1% | 正常波动范围 |

### 2.3 监控配置

**第三批 10 指标接入** (Week 4 新增):
| 指标名 | 类型 | 采集频率 | P0 告警阈值 |
|---|---|---|---|
| `api_latency` | Histogram | 实时 | >200ms |
| `db_query_time` | Histogram | 实时 | >100ms |
| `cache_latency` | Histogram | 实时 | >50ms |
| `queue_latency` | Histogram | 实时 | >100ms |
| `error_rate_by_type` | Gauge | 30s | >5% |
| `success_rate` | Gauge | 30s | <99% |
| `active_users` | Gauge | 10s | - |
| `requests_per_second` | Gauge | 实时 | - |
| `cpu_per_core` | Gauge | 10s | >85% |
| `memory_per_service` | Gauge | 10s | >90% |

---

## 3. 测试方案设计

### 3.1 测试阶段划分

```
72h 稳定性测试时间线
├── Phase 1: 预热期 (0-4h)
│   ├── 负载逐步提升至 50%
│   ├── 监控系统验证
│   └── 基线数据采集
├── Phase 2: 稳定期 (4-48h)
│   ├── 维持 100% 负载
│   ├── 持续监控 30 指标
│   └── 记录性能波动
├── Phase 3: 故障注入期 (48-60h)
│   ├── 故障注入测试 (6 场景)
│   ├── 验证自动恢复
│   └── 记录 MTTR
├── Phase 4: 恢复期 (60-68h)
│   ├── 恢复至正常负载
│   ├── 验证系统稳定
│   └── 数据一致性检查
└── Phase 5: 降载期 (68-72h)
    ├── 负载逐步降低
    ├── 最终数据采集
    └── 测试总结
```

### 3.2 负载模式

**阶段 1: 预热期 (0-4h)**
```
时间    负载    说明
0-1h    10%     系统启动，基础功能验证
1-2h    25%     逐步提升，监控告警验证
2-3h    50%     中等负载，性能基线采集
3-4h    75%     接近目标，系统调优
4h+     100%    进入稳定期
```

**阶段 2: 稳定期 (4-48h)**
```
时间      负载    说明
4-12h     100%    白天负载模式 (模拟生产)
12-20h    60%     夜间负载模式 (模拟生产)
20-28h    100%    白天负载模式
28-36h    60%     夜间负载模式
36-44h    100%    白天负载模式
44-48h    60%     夜间负载模式
```

**阶段 4-5: 恢复与降载 (60-72h)**
```
时间      负载    说明
60-64h    100%    故障后恢复验证
64-68h    75%     逐步降载
68-72h    50%→0   测试收尾，数据采集
```

### 3.3 成功标准

| 指标 | 目标值 | 测量方法 | 通过条件 |
|---|---|---|---|
| **系统可用性** | >99.9% | 正常运行时间/总时间 | 宕机时间<43 分钟 |
| **P99 时延** | <250ms | Prometheus Histogram | 99% 请求<250ms |
| **错误率** | <1% | 错误数/总请求数 | 错误率<1% |
| **内存泄漏** | <5% 增长 | (最终 - 初始)/初始 | 增长<5% |
| **CPU 稳定性** | <85% 平均 | 平均 CPU 使用率 | 平均<85% |
| **故障恢复** | MTTR<5min | 故障注入测试 | 平均恢复<5 分钟 |
| **数据一致性** | 100% | 数据校验 | 零数据丢失 |
| **告警准确率** | >99% | 告警验证 | 误报<1% |

---

## 4. 监控告警配置

### 4.1 30 指标监控体系

**首批 10 指标 (Week 2)**:
- cpu_usage, memory_usage, p50_latency, p99_latency, error_rate
- request_count, batch_execution_time, transaction_duration
- cache_hit_rate, queue_depth

**第二批 10 指标 (Week 3)**:
- disk_io, network_latency, gc_pause, thread_count
- connection_pool_usage, cache_eviction_rate, error_breakdown
- retry_count, rate_limit_hits, circuit_breaker_state

**第三批 10 指标 (Week 4)**:
- api_latency, db_query_time, cache_latency, queue_latency
- error_rate_by_type, success_rate, active_users
- requests_per_second, cpu_per_core, memory_per_service

### 4.2 P0 告警配置 (Critical)

| 告警名 | 指标 | 表达式 | 阈值 | 持续时间 | 响应时间 | 通知渠道 |
|---|---|---|---|---|---|---|
| SystemDown | 系统健康 | `up == 0` | =0 | 1m | <5min | 电话 + 短信 |
| HighErrorRate | error_rate | `error_rate` | >5% | 5m | <5min | 电话 + 短信 |
| HighP99Latency | p99_latency | `histogram_quantile(0.99, rate(api_latency_bucket[5m]))` | >250ms | 5m | <5min | 电话 + 短信 |
| MemoryCritical | memory_per_service | `memory_per_service` | >90% | 5m | <5min | 电话 + 短信 |
| CPUCritical | cpu_per_core | `cpu_per_core` | >85% | 10m | <5min | 电话 + 短信 |
| DatabaseDown | db_query_time | `db_query_time` | >5s | 2m | <5min | 电话 + 短信 |
| CacheDown | cache_latency | `cache_latency` | >500ms | 5m | <5min | 电话 + 短信 |
| CircuitBreakerOpen | circuit_breaker_state | `circuit_breaker_state == 1` | =1 | 1m | <5min | 电话 + 短信 |

### 4.3 P1 告警配置 (Warning)

| 告警名 | 指标 | 表达式 | 阈值 | 持续时间 | 响应时间 | 通知渠道 |
|---|---|---|---|---|---|---|
| HighP95Latency | api_latency | `histogram_quantile(0.95, rate(api_latency_bucket[5m]))` | >180ms | 10m | <15min | 飞书 |
| HighDBQueryTime | db_query_time | `histogram_quantile(0.99, rate(db_query_time_bucket[5m]))` | >100ms | 10m | <15min | 飞书 |
| HighCacheLatency | cache_latency | `cache_latency` | >50ms | 10m | <15min | 飞书 |
| HighQueueLatency | queue_latency | `queue_latency` | >100ms | 10m | <15min | 飞书 |
| LowSuccessRate | success_rate | `success_rate` | <99% | 10m | <15min | 飞书 |
| HighMemoryUsage | memory_per_service | `memory_per_service` | >80% | 15m | <15min | 飞书 |
| HighCPUUsage | cpu_per_core | `cpu_per_core` | >75% | 15m | <15min | 飞书 |
| HighRetryCount | retry_count | `increase(retry_count[1h])` | >50 | 1h | <15min | 飞书 |
| HighRateLimitHits | rate_limit_hits | `increase(rate_limit_hits[1h])` | >100 | 1h | <15min | 飞书 |

### 4.4 告警通知配置

```yaml
# alerting-notification-config.yml

global:
  resolve_timeout: 5m

route:
  group_by: ['alertname', 'severity']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: 'feishu-default'
  routes:
    - match:
        severity: critical
      receiver: 'phone-sms'
      continue: true
    - match:
        severity: warning
      receiver: 'feishu-warning'

receivers:
  - name: 'feishu-default'
    webhook_configs:
      - url: 'http://feishu-webhook/alerts'
        send_resolved: true

  - name: 'feishu-warning'
    webhook_configs:
      - url: 'http://feishu-webhook/alerts-warning'
        send_resolved: true

  - name: 'phone-sms'
    webhook_configs:
      - url: 'http://phone-sms-gateway/alerts'
        send_resolved: true
```

---

## 5. 故障注入测试方案

### 5.1 故障场景设计

| 场景编号 | 故障类型 | 影响范围 | 预期行为 | 恢复方式 |
|---|---|---|---|---|
| **FI-01** | 数据库主节点宕机 | 写操作 | 自动切换至从节点 | 自动 (30s) |
| **FI-02** | 缓存节点故障 | 缓存读取 | 降级至数据库 | 自动 (10s) |
| **FI-03** | 消息队列拥堵 | 异步处理 | 背压 + 限流 | 自动 (2min) |
| **FI-04** | 网络延迟激增 | 跨服务调用 | 超时 + 重试 + 熔断 | 自动 (1min) |
| **FI-05** | CPU 资源耗尽 | 单实例 | 负载均衡转移 | 自动 (30s) |
| **FI-06** | 内存泄漏模拟 | 单服务 | OOM + 重启 | 自动 (1min) |

### 5.2 故障注入时间表

```
故障注入期 (48-60h)
├── 48h: FI-01 数据库主节点宕机
│   ├── 注入时长：5 分钟
│   ├── 预期恢复：<30 秒
│   └── 验证：数据一致性检查
├── 50h: FI-02 缓存节点故障
│   ├── 注入时长：10 分钟
│   ├── 预期恢复：<10 秒
│   └── 验证：缓存命中率恢复
├── 52h: FI-03 消息队列拥堵
│   ├── 注入时长：15 分钟
│   ├── 预期恢复：<2 分钟
│   └── 验证：队列深度恢复
├── 54h: FI-04 网络延迟激增
│   ├── 注入时长：10 分钟
│   ├── 预期恢复：<1 分钟
│   └── 验证：熔断器状态恢复
├── 56h: FI-05 CPU 资源耗尽
│   ├── 注入时长：5 分钟
│   ├── 预期恢复：<30 秒
│   └── 验证：负载均衡正常
└── 58h: FI-06 内存泄漏模拟
    ├── 注入时长：5 分钟
    ├── 预期恢复：<1 分钟
    └── 验证：服务重启成功
```

### 5.3 故障注入执行脚本

```bash
#!/bin/bash
# chaos-injection-scripts.sh

# FI-01: 数据库主节点宕机
inject_db_failover() {
    echo "[$(date)] FI-01: 开始数据库主节点宕机测试"
    kubectl exec postgres-master-0 -- pg_ctl stop -m fast
    sleep 30
    # 验证从节点接管
    kubectl exec postgres-slave-0 -- "SELECT pg_is_in_recovery();"
    echo "[$(date)] FI-01: 测试完成，恢复主节点"
    kubectl exec postgres-master-0 -- pg_ctl start
}

# FI-02: 缓存节点故障
inject_cache_failure() {
    echo "[$(date)] FI-02: 开始缓存节点故障测试"
    kubectl scale deployment redis-node-0 --replicas=0
    sleep 60
    # 验证降级至数据库
    curl -s http://gateway:8080/health/cache | grep "degraded"
    echo "[$(date)] FI-02: 测试完成，恢复缓存节点"
    kubectl scale deployment redis-node-0 --replicas=1
}

# FI-03: 消息队列拥堵
inject_queue_congestion() {
    echo "[$(date)] FI-03: 开始消息队列拥堵测试"
    # 发送大量消息
    for i in {1..10000}; do
        kafka-produce.sh --topic test --message "stress-$i" &
    done
    sleep 900
    # 验证背压机制
    curl -s http://gateway:8080/metrics/queue_depth
    echo "[$(date)] FI-03: 测试完成"
}

# FI-04: 网络延迟激增
inject_network_latency() {
    echo "[$(date)] FI-04: 开始网络延迟激增测试"
    kubectl exec executor-0 -- tc qdisc add dev eth0 root netem delay 500ms
    sleep 600
    # 验证熔断器触发
    curl -s http://gateway:8080/metrics/circuit_breaker_state
    echo "[$(date)] FI-04: 测试完成，恢复网络"
    kubectl exec executor-0 -- tc qdisc del dev eth0 root netem
}

# FI-05: CPU 资源耗尽
inject_cpu_stress() {
    echo "[$(date)] FI-05: 开始 CPU 资源耗尽测试"
    kubectl exec executor-0 -- stress-ng --cpu 4 --cpu-method matrixprod --timeout 300s
    sleep 300
    # 验证负载均衡转移
    curl -s http://gateway:8080/metrics/load_balancer
    echo "[$(date)] FI-05: 测试完成"
}

# FI-06: 内存泄漏模拟
inject_memory_leak() {
    echo "[$(date)] FI-06: 开始内存泄漏模拟测试"
    kubectl exec verifier-0 -- stress-ng --vm 2 --vm-bytes 2G --timeout 300s
    sleep 300
    # 验证 OOM 重启
    kubectl get pod verifier-0 -o jsonpath='{.status.containerStatuses[0].restartCount}'
    echo "[$(date)] FI-06: 测试完成"
}

# 主执行流程
case "$1" in
    "fi-01") inject_db_failover ;;
    "fi-02") inject_cache_failure ;;
    "fi-03") inject_queue_congestion ;;
    "fi-04") inject_network_latency ;;
    "fi-05") inject_cpu_stress ;;
    "fi-06") inject_memory_leak ;;
    "all")
        inject_db_failover
        sleep 7200
        inject_cache_failure
        sleep 7200
        inject_queue_congestion
        sleep 7200
        inject_network_latency
        sleep 7200
        inject_cpu_stress
        sleep 7200
        inject_memory_leak
        ;;
    *) echo "Usage: $0 {fi-01|fi-02|fi-03|fi-04|fi-05|fi-06|all}" ;;
esac
```

### 5.4 故障恢复验证

| 验证项 | 验证方法 | 通过标准 | 工具 |
|---|---|---|---|
| **自动恢复** | 检查服务状态 | 服务自动恢复 | kubectl get pods |
| **数据一致性** | 数据校验 | 零数据丢失 | 数据校验脚本 |
| **性能恢复** | 性能指标检查 | P99 恢复至基线 | Prometheus |
| **告警触发** | 告警记录检查 | 告警准确触发 | Alertmanager |
| **告警恢复** | 告警清除检查 | 告警自动清除 | Alertmanager |

---

## 6. 测试执行计划

### 6.1 人员分工

| 角色 | 职责 | 人员 | 联系方式 |
|---|---|---|---|
| **测试指挥** | 整体协调，决策 | SRE-Lead | @sre-lead |
| **执行工程师** | 测试执行，记录 | SRE-Engineer | @sre-eng |
| **监控工程师** | 监控告警，响应 | Observability | @obs-eng |
| **开发支持** | 问题排查，修复 | Dev-Team | @dev-team |
| **QA 验证** | 结果验证，报告 | QA-Team | @qa-team |

### 6.2 时间安排

| 时间 | 阶段 | 主要活动 | 参与人员 |
|---|---|---|---|
| **2026-03-08 09:00** | 测试启动 | 环境准备，负载启动 | 全体 |
| **2026-03-08 13:00** | 预热完成 | 进入稳定期 | 执行工程师 |
| **2026-03-08~03-10** | 稳定期 | 持续监控，数据记录 | 监控工程师 |
| **2026-03-10 09:00** | 故障注入开始 | 执行 6 个故障场景 | 执行工程师 |
| **2026-03-10 21:00** | 故障注入完成 | 进入恢复期 | 全体 |
| **2026-03-11 05:00** | 恢复期完成 | 进入降载期 | 执行工程师 |
| **2026-03-11 09:00** | 测试结束 | 数据收集，总结 | 全体 |

### 6.3 沟通机制

**即时沟通**:
- 飞书群：Phase3-Week4-稳定性测试
- 紧急联系人：@sre-lead (电话)

**状态同步**:
- 每 4 小时：监控状态同步
- 故障注入时：实时同步
- 每日 09:00/21:00: 日报同步

**升级机制**:
- P0 故障：立即电话通知
- P1 故障：15 分钟内飞书通知
- P2 故障：1 小时内飞书通知

---

## 7. 风险评估与缓解

### 7.1 测试风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|---|---|---|---|
| 测试环境不稳定 | 低 | 高 | 提前 24h 预热验证 |
| 监控数据丢失 | 低 | 高 | 双份存储，实时备份 |
| 故障注入失控 | 中 | 高 | 设置熔断机制，手动停止 |
| 性能数据异常 | 中 | 中 | 多次测试取平均值 |
| 人员响应不及时 | 低 | 中 | 轮班制度，AB 角 |

### 7.2 熔断机制

**自动熔断条件**:
- 错误率>10% 持续 5 分钟
- P99 时延>500ms 持续 5 分钟
- 系统宕机>1 分钟
- 数据不一致检测到

**手动熔断条件**:
- 测试指挥判断风险过高
- 生产环境受影响
- 测试目标已达成

**熔断操作**:
```bash
# 紧急停止测试
kubectl scale deployment load-generator --replicas=0

# 停止故障注入
kubectl delete job chaos-injection-*

# 恢复至正常状态
kubectl apply -f baseline-config.yml
```

---

## 8. 验收标准

### 8.1 稳定性验收

| 验收项 | 标准 | 测量方法 | 通过条件 |
|---|---|---|---|
| 系统可用性 | >99.9% | 正常运行时间/总时间 | 宕机<43 分钟 |
| 性能稳定性 | 波动<10% | P99 时延标准差 | 标准差<10% |
| 资源稳定性 | 无泄漏 | 内存增长曲线 | 增长<5% |
| 错误率 | <1% | 错误数/总请求数 | 错误率<1% |

### 8.2 故障恢复验收

| 验收项 | 标准 | 测量方法 | 通过条件 |
|---|---|---|---|
| MTTR | <5 分钟 | 故障注入测试 | 平均恢复<5 分钟 |
| 自动恢复率 | 100% | 故障场景测试 | 6/6 场景自动恢复 |
| 数据一致性 | 100% | 数据校验 | 零数据丢失 |
| 告警准确率 | >99% | 告警记录检查 | 误报<1% |

### 8.3 监控验收

| 验收项 | 标准 | 测量方法 | 通过条件 |
|---|---|---|---|
| 30 指标覆盖 | 100% | Prometheus 查询 | 30/30 指标可查询 |
| 数据新鲜度 | <30s | 时间戳检查 | 最新数据<30s |
| 告警有效性 | 100% | 告警测试 | 16/16 告警规则有效 |
| 仪表盘完整 | 100% | Grafana 检查 | 所有 Panel 正常 |

---

## 9. 附录

### 9.1 负载生成配置

```yaml
# load-generator-config.yml

apiVersion: batch/v1
kind: Job
metadata:
  name: load-generator
spec:
  template:
    spec:
      containers:
      - name: wrk
        image: wrk-benchmark:latest
        command: ["wrk"]
        args:
          - "-t4"
          - "-c400"
          - "-d4320m"  # 72 小时
          - "-s"
          - "/scripts/benchmark.lua"
          - "http://gateway:8080/api/v1/execute"
        env:
          - name: TARGET_QPS
            value: "5000"
          - name: BATCH_RATIO
            value: "0.2"
          - name: TRANSACTION_RATIO
            value: "0.15"
      restartPolicy: Never
```

### 9.2 监控查询手册

```promql
# === 核心指标 ===
# P99 API 时延
histogram_quantile(0.99, sum(rate(api_latency_bucket[5m])) by(le))

# 成功率
success_rate

# 活跃用户数
active_users

# 每秒请求数
requests_per_second

# === 系统资源 ===
# 每核心 CPU 使用率
cpu_per_core

# 每服务内存使用率
memory_per_service

# === 依赖组件 ===
# 数据库查询时延 P99
histogram_quantile(0.99, sum(rate(db_query_time_bucket[5m])) by(le))

# 缓存时延
cache_latency

# 队列时延
queue_latency

# === 错误分析 ===
# 按类型错误率
error_rate_by_type

# 错误率
error_rate
```

### 9.3 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Week 3 性能基线 | performance_baseline_week3.md | 基线参考 |
| 第三批 10 指标实现 | metrics_10_batch3_impl.md | 指标实现 |
| Phase 3 50 指标规划 | phase3_50_metrics_plan.md | 完整指标体系 |
| 故障注入最佳实践 | chaos_engineering_guide.md | 故障注入参考 |

---

**文档状态**: 📋 Week 4 规划  
**创建日期**: 2026-03-07  
**责任人**: SRE-Agent  
**保管**: 项目文档库
