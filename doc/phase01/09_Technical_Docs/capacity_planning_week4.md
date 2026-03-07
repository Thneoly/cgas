# Phase 3 Week 4: 容量规划与压测报告

**版本**: v1.0  
**日期**: 2026-03-07  
**责任人**: SRE-Agent  
**状态**: 📋 Week 4 规划  
**release_id**: release-2026-03-07-phase3-week4-capacity  
**参与角色**: SRE, Observability, Dev, Performance Engineering

---

## 1. 概述

### 1.1 容量规划目标

基于 Week 3 性能基线数据和 Week 4 新增 10 指标，进行系统容量评估和压力测试，确定系统承载能力上限，制定容量扩展计划。

### 1.2 规划范围

| 规划维度 | 当前状态 | 目标状态 | 时间范围 |
|---|---|---|---|
| 系统容量 | 10,000 QPS 设计 | 验证实际承载能力 | Week 4 |
| 资源利用 | CPU 43%, 内存 59% | 优化至 CPU<70%, 内存<75% | Week 4-5 |
| 扩展策略 | 手动扩展 | 自动弹性伸缩 | Week 5-6 |
| 成本优化 | 未优化 | 资源利用率提升 30% | Week 6-8 |

### 1.3 与 Week 3 基线对比

| 维度 | Week 3 基线 | Week 4 目标 | 变化 |
|---|---|---|---|
| 平均 QPS | 4,120 | 5,000 (测试) | +21% |
| 高峰 QPS | 8,950 | 10,000 (设计) | +12% |
| P99 时延 | 198ms | <250ms (负载下) | +26% 余量 |
| CPU 使用率 | 43% | <70% (目标) | +63% 余量 |
| 内存使用率 | 59% | <75% (目标) | +27% 余量 |

---

## 2. 当前容量评估

### 2.1 系统资源现状

**计算资源** (基于 Week 3 基线):
| 组件 | 规格 | 实例数 | CPU 平均 | CPU 峰值 | 内存平均 | 内存峰值 |
|---|---|---|---|---|---|---|
| **Executor** | 4 核 8GB | 3 | 45% | 75% | 58% | 72% |
| **Verifier** | 4 核 8GB | 2 | 40% | 68% | 55% | 68% |
| **Gateway** | 2 核 4GB | 2 | 35% | 58% | 52% | 65% |
| **Database** | 8 核 16GB | 1 主 2 从 | 52% | 78% | 68% | 82% |
| **Cache** | 4 核 8GB | 3 节点 | 38% | 62% | 72% | 85% |
| **Message Queue** | 4 核 8GB | 3 节点 | 42% | 65% | 65% | 78% |

**存储资源**:
| 存储类型 | 总容量 | 已使用 | 使用率 | 日增长 | 预计耗尽 |
|---|---|---|---|---|---|
| **数据库** | 500GB | 285GB | 57% | 1.2GB/天 | 179 天 |
| **日志存储** | 200GB | 125GB | 62.5% | 2.5GB/天 | 30 天 |
| **缓存** | 24GB | 18GB | 75% | - | - |
| **消息队列** | 100GB | 45GB | 45% | 0.8GB/天 | 69 天 |

### 2.2 承载能力评估

**基于 Week 3 数据推算**:

| 负载水平 | QPS | CPU 使用率 | 内存使用率 | P99 时延 | 错误率 | 状态 |
|---|---|---|---|---|---|---|
| **当前平均** | 4,120 | 43% | 59% | 198ms | 0.65% | ✅ 健康 |
| **当前高峰** | 8,950 | 68% | 72% | 245ms | 0.85% | ✅ 健康 |
| **设计承载** | 10,000 | 75% | 78% | 280ms (预估) | 1.0% (预估) | ⚠️ 接近上限 |
| **理论极限** | 12,500 | 90% | 88% | 450ms (预估) | 2.5% (预估) | ❌ 不推荐 |

**容量余量分析**:
- **当前至设计承载**: +12% (高峰 QPS 8,950 → 10,000)
- **当前至理论极限**: +40% (高峰 QPS 8,950 → 12,500)
- **安全余量**: 设计承载的 25% (10,000 → 12,500)

### 2.3 瓶颈识别

| 瓶颈类型 | 瓶颈点 | 当前使用率 | 阈值 | 风险等级 | 建议措施 |
|---|---|---|---|---|---|
| **CPU 瓶颈** | Database 主节点 | 78% (峰值) | 80% | 🟡 中 | 优化慢查询，增加只读副本 |
| **内存瓶颈** | Cache 节点 | 85% (峰值) | 85% | 🟡 中 | 增加缓存节点，优化淘汰策略 |
| **IO 瓶颈** | Database 磁盘 | 72% (P99) | 80% | 🟢 低 | 监控增长，提前规划 SSD 升级 |
| **网络瓶颈** | Gateway 出口 | 45% (峰值) | 80% | 🟢 低 | 余量充足 |
| **连接池瓶颈** | Database 连接 | 78% (P99) | 80% | 🟡 中 | 动态调整连接池大小 |

---

## 3. 压力测试方案

### 3.1 压测目标

| 测试类型 | 目标 | 负载 | 持续时间 | 验收标准 |
|---|---|---|---|---|
| **基准测试** | 建立性能基线 | 4,000 QPS | 1 小时 | P99<200ms, 错误率<1% |
| **负载测试** | 验证设计承载 | 10,000 QPS | 2 小时 | P99<300ms, 错误率<1.5% |
| **压力测试** | 探索系统极限 | 12,000+ QPS | 1 小时 | 找到崩溃点 |
| **稳定性测试** | 验证长期稳定 | 8,000 QPS | 24 小时 | 无内存泄漏，性能稳定 |
| **突发测试** | 验证弹性能力 | 4,000→10,000→4,000 | 30 分钟 | 自动扩缩容正常 |

### 3.2 压测环境

**环境配置** (与生产 1:1):
| 组件 | 规格 | 实例数 | 版本 |
|---|---|---|---|
| **Executor** | 4 核 8GB | 3 | v3.2.0 |
| **Verifier** | 4 核 8GB | 2 | v3.2.0 |
| **Gateway** | 2 核 4GB | 2 | v3.2.0 |
| **Database** | 8 核 16GB, SSD | 1 主 2 从 | PostgreSQL 15.2 |
| **Cache** | 4 核 8GB | 3 节点 | Redis 7.0 Cluster |
| **Message Queue** | 4 核 8GB | 3 节点 | Kafka 3.4 |

**压测工具**:
- **主要工具**: wrk2 (精确 QPS 控制)
- **辅助工具**: k6 (场景化测试)
- **监控工具**: Prometheus + Grafana
- **分析工具**: pyroscope (性能剖析)

### 3.3 压测场景设计

**场景 1: 基准测试**
```yaml
name: baseline-test
description: 建立性能基线
load_pattern:
  type: constant
  qps: 4000
  duration: 1h
request_mix:
  - endpoint: /api/v1/execute
    method: POST
    percentage: 65%
  - endpoint: /api/v1/batch
    method: POST
    percentage: 20%
  - endpoint: /api/v1/transaction
    method: POST
    percentage: 15%
success_criteria:
  - p99_latency < 200ms
  - error_rate < 1%
  - cpu_usage < 60%
  - memory_usage < 70%
```

**场景 2: 负载测试**
```yaml
name: load-test
description: 验证设计承载能力
load_pattern:
  type: constant
  qps: 10000
  duration: 2h
request_mix:
  - endpoint: /api/v1/execute
    method: POST
    percentage: 65%
  - endpoint: /api/v1/batch
    method: POST
    percentage: 20%
  - endpoint: /api/v1/transaction
    method: POST
    percentage: 15%
success_criteria:
  - p99_latency < 300ms
  - error_rate < 1.5%
  - cpu_usage < 80%
  - memory_usage < 85%
```

**场景 3: 压力测试 (极限探索)**
```yaml
name: stress-test
description: 探索系统极限
load_pattern:
  type: ramp-up
  start_qps: 8000
  end_qps: 15000
  ramp_duration: 30m
  hold_duration: 30m
request_mix:
  - endpoint: /api/v1/execute
    method: POST
    percentage: 65%
  - endpoint: /api/v1/batch
    method: POST
    percentage: 20%
  - endpoint: /api/v1/transaction
    method: POST
    percentage: 15%
success_criteria:
  - identify_breaking_point: true
  - document_failure_mode: true
```

**场景 4: 稳定性测试**
```yaml
name: stability-test
description: 24 小时长期稳定性验证
load_pattern:
  type: constant
  qps: 8000
  duration: 24h
request_mix:
  - endpoint: /api/v1/execute
    method: POST
    percentage: 65%
  - endpoint: /api/v1/batch
    method: POST
    percentage: 20%
  - endpoint: /api/v1/transaction
    method: POST
    percentage: 15%
success_criteria:
  - no_memory_leak: true  # 内存增长<5%
  - no_performance_degradation: true  # P99 波动<10%
  - no_error_spike: true  # 错误率<1%
```

**场景 5: 突发测试 (弹性验证)**
```yaml
name: spike-test
description: 验证弹性伸缩能力
load_pattern:
  type: spike
  phases:
    - qps: 4000
      duration: 10m
    - qps: 10000
      duration: 10m
    - qps: 4000
      duration: 10m
request_mix:
  - endpoint: /api/v1/execute
    method: POST
    percentage: 65%
success_criteria:
  - autoscaling_triggered: true
  - recovery_time < 5m
  - no_service_disruption: true
```

### 3.4 压测执行计划

| 时间 | 场景 | 目标 | 参与人员 |
|---|---|---|---|
| **Day 1 09:00** | 基准测试 | 建立基线 | SRE + Performance |
| **Day 1 14:00** | 负载测试 | 验证设计承载 | SRE + Performance |
| **Day 2 09:00** | 压力测试 | 探索极限 | SRE + Dev |
| **Day 2 14:00** | 稳定性测试 | 24h 验证 | SRE (轮班) |
| **Day 3 14:00** | 突发测试 | 弹性验证 | SRE + Performance |
| **Day 4 09:00** | 数据分析 | 报告输出 | SRE + Performance |

---

## 4. 容量扩展策略

### 4.1 短期扩展 (Week 4-5)

**垂直扩展 (Scale Up)**:
| 组件 | 当前规格 | 建议规格 | 提升幅度 | 成本增加 | 优先级 |
|---|---|---|---|---|---|
| **Database 主节点** | 8 核 16GB | 16 核 32GB | +100% | +150% | P0 |
| **Cache 节点** | 4 核 8GB | 8 核 16GB | +100% | +100% | P1 |
| **Executor** | 4 核 8GB | 保持不变 | - | - | - |

**水平扩展 (Scale Out)**:
| 组件 | 当前实例数 | 建议实例数 | 提升幅度 | 成本增加 | 优先级 |
|---|---|---|---|---|---|
| **Executor** | 3 | 5 | +67% | +67% | P1 |
| **Verifier** | 2 | 3 | +50% | +50% | P1 |
| **Database 只读副本** | 2 | 3 | +50% | +50% | P1 |

### 4.2 中期扩展 (Week 6-8)

**自动弹性伸缩**:
```yaml
# autoscaling-config.yml

apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: executor-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: executor
  minReplicas: 3
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 75
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 50
          periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Percent
          value: 100
          periodSeconds: 60
```

**数据库读写分离优化**:
| 优化项 | 当前状态 | 目标状态 | 预期收益 |
|---|---|---|---|
| 读请求路由 | 50% 至只读副本 | 90% 至只读副本 | 主节点负载 -40% |
| 写请求优化 | 无批处理 | 批处理写入 | 写吞吐 +50% |
| 连接池优化 | 固定 50 连接 | 动态 50-150 连接 | 连接等待 -60% |

### 4.3 长期扩展 (Week 9-12)

**架构优化**:
| 优化方向 | 方案 | 预期收益 | 实施难度 |
|---|---|---|---|
| **缓存分层** | L1 (本地) + L2 (Redis) | 缓存命中率 95%→98% | 中 |
| **数据库分片** | 按业务维度分片 | 单库负载 -50% | 高 |
| **异步化改造** | 同步→异步 | 响应时延 -30% | 中 |
| **CDN 加速** | 静态资源 CDN | 源站负载 -20% | 低 |

**成本优化**:
| 优化项 | 当前成本 | 目标成本 | 节省幅度 |
|---|---|---|---|
| **计算资源** | ¥50,000/月 | ¥40,000/月 | -20% |
| **存储资源** | ¥15,000/月 | ¥12,000/月 | -20% |
| **网络资源** | ¥10,000/月 | ¥8,000/月 | -20% |
| **总计** | ¥75,000/月 | ¥60,000/月 | -20% |

---

## 5. 容量规划模型

### 5.1 流量预测模型

**基于历史数据预测** (Week 3 基线):
```
当前流量:
- 平均 QPS: 4,120
- 高峰 QPS: 8,950
- 日请求总量: 356M

增长率假设:
- 自然增长: 7%/月 (基于 Week 2→Week 3)
- 业务活动: +20% (特殊活动)
- 保守估计: 10%/月

3 个月后预测:
- 平均 QPS: 4,120 × (1.10)^3 = 5,484
- 高峰 QPS: 8,950 × (1.10)^3 = 11,915
- 日请求总量: 356M × (1.10)^3 = 474M
```

**容量需求推算**:
| 时间 | 预测 QPS | CPU 需求 | 内存需求 | 建议配置 |
|---|---|---|---|---|
| **当前** | 4,120 (平均) / 8,950 (高峰) | 43% / 68% | 59% / 72% | 现有配置 |
| **1 个月后** | 4,532 / 9,845 | 47% / 75% | 62% / 76% | 监控观察 |
| **3 个月后** | 5,484 / 11,915 | 57% / 88% | 68% / 85% | 需要扩展 |
| **6 个月后** | 7,273 / 15,800 | 76% / >100% | 78% / >100% | 必须扩展 |

### 5.2 资源需求计算

**计算公式**:
```
所需实例数 = (预测 QPS × 单请求资源消耗) / (单实例容量 × 目标利用率)

其中:
- 单请求 CPU 消耗: 0.018 核/QPS (基于 Week 3 数据)
- 单请求内存消耗: 2.5 MB/QPS
- 单实例容量: 4 核 / 8GB
- 目标利用率: CPU 70%, 内存 75%
```

**Executor 实例数计算**:
| 场景 | QPS | CPU 需求 | 内存需求 | 实例数 (向上取整) |
|---|---|---|---|---|
| 当前高峰 | 8,950 | 161 核 | 22.4GB | 3 实例 (4 核×3=12 核，需优化) |
| 3 个月后 | 11,915 | 214 核 | 29.8GB | 5 实例 (4 核×5=20 核) |
| 6 个月后 | 15,800 | 284 核 | 39.5GB | 8 实例 (4 核×8=32 核) |

**修正后计算** (考虑实际测量):
| 场景 | QPS | 实测 CPU | 实测内存 | 建议实例数 |
|---|---|---|---|---|
| 当前高峰 | 8,950 | 68% (4 核×3) | 72% (8GB×3) | 3 实例 ✅ |
| 设计承载 | 10,000 | 75% (预估) | 78% (预估) | 3 实例 ⚠️ |
| 3 个月后 | 11,915 | 88% (预估) | 85% (预估) | 5 实例 📈 |

### 5.3 成本效益分析

**扩展方案对比**:
| 方案 | 容量提升 | 成本增加 | 实施周期 | 风险 | 推荐度 |
|---|---|---|---|---|---|
| **垂直扩展 Database** | +50% | +150% | 1 天 | 低 | ⭐⭐⭐⭐ |
| **水平扩展 Executor** | +67% | +67% | 1 小时 | 低 | ⭐⭐⭐⭐⭐ |
| **自动弹性伸缩** | +233% (3→10) | 按需付费 | 1 周 | 中 | ⭐⭐⭐⭐⭐ |
| **架构优化 (缓存分层)** | +30% | +20% | 2 周 | 中 | ⭐⭐⭐ |
| **数据库分片** | +100% | +50% | 4 周 | 高 | ⭐⭐ |

**ROI 分析**:
```
方案：水平扩展 Executor (3→5 实例)

成本:
- 新增 2 实例 × ¥5,000/月 = ¥10,000/月
- 一次性实施成本：¥5,000

收益:
- 容量提升：67% (支持 QPS 8,950→15,000)
- 避免宕机损失：¥50,000/次 (假设每月 1 次)
- 支持业务增长：10%/月自然增长

ROI 计算:
- 月度收益：¥50,000 (避免宕机) + ¥20,000 (业务增长) = ¥70,000
- 月度成本：¥10,000
- 月度净收益：¥60,000
- ROI: 600%
- 投资回收期：0.5 个月
```

---

## 6. 风险与缓解

### 6.1 容量风险

| 风险 | 可能性 | 影响 | 缓解措施 | 责任人 |
|---|---|---|---|---|
| 流量增长超预期 | 中 | 高 | 提前 2 周扩容，设置 70% 告警 | SRE |
| 数据库瓶颈 | 高 | 高 | 增加只读副本，优化慢查询 | Dev + DBA |
| 缓存穿透 | 低 | 高 | Bloom Filter，降级策略 | Dev |
| 资源竞争 | 中 | 中 | 资源隔离，QoS 保障 | SRE |
| 成本超预算 | 中 | 中 | 成本监控，自动伸缩 | FinOps |

### 6.2 压测风险

| 风险 | 可能性 | 影响 | 缓解措施 | 责任人 |
|---|---|---|---|---|
| 压测影响生产 | 低 | 高 | 隔离环境，限流保护 | SRE |
| 数据污染 | 低 | 高 | 测试数据标记，定期清理 | QA |
| 监控数据丢失 | 低 | 中 | 双份存储，实时备份 | Observability |
| 压测结果不准确 | 中 | 中 | 多次测试取平均 | Performance |

---

## 7. 建议与行动计划

### 7.1 P0 行动 (Week 4)

| 行动 | 描述 | 预期收益 | 完成时间 |
|---|---|---|---|
| **Database 扩容** | 8 核 16GB→16 核 32GB | 主节点负载 -40% | Week 4 |
| **Executor 扩容** | 3 实例→5 实例 | 容量 +67% | Week 4 |
| **慢查询优化** | Top 10 慢查询优化 | P99 时延 -20% | Week 4 |
| **连接池优化** | 动态 50-150 连接 | 连接等待 -60% | Week 4 |

### 7.2 P1 行动 (Week 5-6)

| 行动 | 描述 | 预期收益 | 完成时间 |
|---|---|---|---|
| **自动伸缩部署** | HPA 配置上线 | 弹性应对流量 | Week 5 |
| **只读副本增加** | 2→3 只读副本 | 读负载 -30% | Week 5 |
| **缓存分层** | L1+L2 缓存架构 | 命中率 95%→98% | Week 6 |
| **CDN 加速** | 静态资源 CDN | 源站负载 -20% | Week 6 |

### 7.3 P2 行动 (Week 7-12)

| 行动 | 描述 | 预期收益 | 完成时间 |
|---|---|---|---|
| **数据库分片** | 按业务分片 | 单库负载 -50% | Week 9-12 |
| **异步化改造** | 同步→异步 | 响应时延 -30% | Week 8-10 |
| **成本优化** | 资源优化配置 | 总成本 -20% | Week 7-12 |

---

## 8. 附录

### 8.1 压测脚本示例

```bash
#!/bin/bash
# load-test-script.sh

# 基准测试
run_baseline_test() {
    echo "[$(date)] 开始基准测试 (4000 QPS)"
    wrk -t4 -c400 -d3600s -s baseline.lua http://gateway:8080/api/v1/execute
    echo "[$(date)] 基准测试完成"
}

# 负载测试
run_load_test() {
    echo "[$(date)] 开始负载测试 (10000 QPS)"
    wrk -t8 -c800 -d7200s -s load.lua http://gateway:8080/api/v1/execute
    echo "[$(date)] 负载测试完成"
}

# 压力测试
run_stress_test() {
    echo "[$(date)] 开始压力测试 (8000→15000 QPS)"
    k6 run --duration 30m --vus 1000 stress_test.js
    echo "[$(date)] 压力测试完成"
}

# 稳定性测试
run_stability_test() {
    echo "[$(date)] 开始稳定性测试 (8000 QPS, 24h)"
    wrk -t8 -c600 -d86400s -s stability.lua http://gateway:8080/api/v1/execute
    echo "[$(date)] 稳定性测试完成"
}

# 突发测试
run_spike_test() {
    echo "[$(date)] 开始突发测试"
    k6 run spike_test.js
    echo "[$(date)] 突发测试完成"
}

case "$1" in
    "baseline") run_baseline_test ;;
    "load") run_load_test ;;
    "stress") run_stress_test ;;
    "stability") run_stability_test ;;
    "spike") run_spike_test ;;
    "all")
        run_baseline_test
        sleep 3600
        run_load_test
        sleep 7200
        run_stress_test
        sleep 3600
        run_stability_test
        sleep 86400
        run_spike_test
        ;;
    *) echo "Usage: $0 {baseline|load|stress|stability|spike|all}" ;;
esac
```

### 8.2 监控查询手册

```promql
# === 容量监控 ===
# CPU 使用率趋势
avg by(instance) (rate(node_cpu_seconds_total{mode!="idle"}[5m])) * 100

# 内存使用率趋势
container_memory_usage_bytes / container_spec_memory_limit_bytes * 100

# QPS 趋势
sum(rate(requests_total[5m]))

# P99 时延趋势
histogram_quantile(0.99, sum(rate(api_latency_bucket[5m])) by(le))

# === 容量预测 ===
# CPU 使用率预测 (线性回归)
predict_linear(avg by(instance) (rate(node_cpu_seconds_total{mode!="idle"}[1h]))[24h:], 3*24*3600)

# 内存增长预测
predict_linear(container_memory_usage_bytes[24h:], 7*24*3600)

# QPS 增长预测
predict_linear(sum(rate(requests_total[1h]))[7d:], 30*24*3600)
```

### 8.3 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Week 3 性能基线 | performance_baseline_week3.md | 基线数据 |
| 72h 稳定性测试方案 | 72h_stability_test_plan.md | 测试方案 |
| 第三批 10 指标实现 | metrics_10_batch3_impl.md | 监控指标 |
| Phase 3 50 指标规划 | phase3_50_metrics_plan.md | 完整指标体系 |

---

**文档状态**: 📋 Week 4 规划  
**创建日期**: 2026-03-07  
**责任人**: SRE-Agent  
**保管**: 项目文档库
