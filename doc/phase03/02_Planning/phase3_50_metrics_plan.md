# Phase 3 50 指标扩展规划

**版本**: v1.0  
**日期**: 2026-05-12  
**责任人**: SRE-Agent  
**状态**: 📋 规划中  
**release_id**: release-2026-05-12-phase3_metrics  
**参与角色**: SRE, Observability, Dev, Security

---

## 1. 概述

### 1.1 扩展目标

| 维度 | Phase 2 基线 | Phase 3 目标 | 提升幅度 |
|---|---|---|---|
| 监控指标总数 | 25 个 | **50 个** | +100% |
| 性能指标 | 8 个 | 18 个 | +125% |
| 错误指标 | 5 个 | 10 个 | +100% |
| 业务指标 | 6 个 | 14 个 | +133% |
| 系统指标 | 6 个 | 8 个 | +33% |
| 追踪指标 | 0 个 | 5 个 | 新增 |
| 安全指标 | 0 个 | 5 个 | 新增 |

### 1.2 设计原则

| 原则 | 说明 | 验收标准 |
|---|---|---|
| **RED 方法** | Rate, Errors, Duration 为核心 | 所有服务覆盖 RED 三指标 |
| **USE 方法** | Utilization, Saturation, Errors | 所有资源覆盖 USE 三指标 |
| **黄金信号** | 延迟、流量、错误、饱和度 | 关键服务 100% 覆盖 |
| **可行动性** | 每个指标都有明确告警阈值 | 100% 指标可告警 |
| **低开销** | 指标采集开销<1% | 性能影响<1% |

### 1.3 指标分类体系

```
Phase 3 50 指标体系
├── 性能指标 (18 个)
│   ├── 执行性能 (6 个)
│   ├── 验证性能 (4 个)
│   ├── Batch 性能 (4 个)
│   ├── Transaction 性能 (4 个)
│   └── 追踪性能 (3 个)
├── 错误指标 (10 个)
│   ├── 执行错误 (3 个)
│   ├── 验证错误 (2 个)
│   ├── Batch 错误 (2 个)
│   ├── Transaction 错误 (2 个)
│   └── 系统错误 (1 个)
├── 业务指标 (14 个)
│   ├── 指令执行 (4 个)
│   ├── 灰度发布 (3 个)
│   ├── 安全合规 (4 个)
│   └── 用户行为 (3 个)
├── 系统指标 (8 个)
│   ├── 资源使用 (4 个)
│   └── 基础设施 (4 个)
└── 新增专项 (5 个)
    ├── 分布式追踪 (3 个)
    └── 威胁检测 (2 个)
```

---

## 2. Phase 2 继承指标 (25 个)

### 2.1 继承指标清单

| 指标 ID | 指标名 | 类型 | 当前阈值 | 来源 | Phase 3 调整 |
|---|---|---|---|---|---|
| **M-001** | gray_release_consistency_rate | Gauge | <99.9% | 灰度发布 | 阈值收紧至<99.95% |
| **M-002** | gray_release_unverified_submit_rate | Gauge | >0 | 灰度发布 | 保持不变 |
| **M-003** | gray_release_false_positive_rate | Gauge | >5% | 灰度发布 | 阈值收紧至<3% |
| **M-004** | gray_release_latency_increase | Gauge | >30% | 灰度发布 | 阈值收紧至>20% |
| **M-005** | gray_release_error_rate | Counter | >1% | 灰度发布 | 保持不变 |
| **M-006** | execution_latency_p99 | Histogram | >300ms | 执行器 | 阈值收紧至>200ms |
| **M-007** | verification_latency_p99 | Histogram | >300ms | 验证器 | 阈值收紧至>200ms |
| **M-008** | blocking_middleware_overhead | Histogram | >5% | 阻断中间件 | 阈值收紧至>3% |
| **M-009** | verifier_replay_consistency_rate | Gauge | <99.9% | 验证器 | 阈值收紧至<99.95% |
| **M-010** | state_snapshot_consistency_rate | Gauge | <99.9% | 状态快照 | 阈值收紧至<99.95% |
| **M-011** | audit_log_write_success_rate | Gauge | <99% | 审计日志 | 保持不变 |
| **M-012** | gate_verification_pass_rate | Gauge | <100% | 提交闸门 | 保持不变 |
| **M-013** | cpu_usage_percent | Gauge | >80% | 系统 | 保持不变 |
| **M-014** | memory_usage_percent | Gauge | >85% | 系统 | 保持不变 |
| **M-015** | rollback_triggered_count | Counter | >0 | 回滚 | 保持不变 |
| **M-016** | batch_execute_latency_p99 | Histogram | >400ms | Batch 服务 | 阈值收紧至>300ms |
| **M-017** | batch_atomicity_violation_count | Counter | >0 | Batch 服务 | 保持不变 |
| **M-018** | batch_sub_instruction_count | Histogram | - | Batch 服务 | 保持不变 |
| **M-019** | transaction_commit_latency_p99 | Histogram | >400ms | Transaction 服务 | 阈值收紧至>300ms |
| **M-020** | transaction_rollback_count | Counter | >10/h | Transaction 服务 | 阈值收紧至>5/h |
| **M-021** | transaction_timeout_count | Counter | >5/h | Transaction 服务 | 阈值收紧至>3/h |
| **M-022** | zero_trust_auth_failure_count | Counter | >10/h | 零信任模块 | 阈值收紧至>5/h |
| **M-023** | zero_trust_policy_violation_count | Counter | >5/h | 零信任模块 | 阈值收紧至>2/h |
| **M-024** | instruction_type_distribution | Histogram | - | 执行器 | 保持不变 |
| **M-025** | distributed_trace_coverage | Gauge | <95% | 追踪系统 | 阈值收紧至<98% |

---

## 3. Phase 3 新增指标 (25 个)

### 3.1 性能指标扩展 (10 个新增)

#### 3.1.1 执行性能细分 (3 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-026** | execution_latency_p50 | Histogram | 实时 | >100ms | 执行器 | P50 执行时延基线 |
| **M-027** | execution_latency_p95 | Histogram | 实时 | >180ms | 执行器 | P95 执行时延监控 |
| **M-028** | executor_queue_depth | Gauge | 10s | >100 | 执行器 | 执行队列深度 |

**采集配置**:
```yaml
- metric: execution_latency_p50
  type: Histogram
  buckets: [10, 25, 50, 75, 100, 150, 200, 300, 500]
  labels: [service, environment, instruction_type]
  description: "Execution latency P50 in milliseconds"
  collection_interval: 15s
  
- metric: execution_latency_p95
  type: Histogram
  buckets: [50, 100, 150, 200, 250, 300, 400, 500, 750]
  labels: [service, environment, instruction_type]
  description: "Execution latency P95 in milliseconds"
  collection_interval: 15s
  
- metric: executor_queue_depth
  type: Gauge
  labels: [service, environment, executor_id]
  description: "Current executor queue depth"
  collection_interval: 10s
  alert_threshold: ">100"
```

#### 3.1.2 验证性能细分 (2 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-029** | verification_latency_p50 | Histogram | 实时 | >100ms | 验证器 | P50 验证时延基线 |
| **M-030** | verification_queue_depth | Gauge | 10s | >100 | 验证器 | 验证队列深度 |

**采集配置**:
```yaml
- metric: verification_latency_p50
  type: Histogram
  buckets: [10, 25, 50, 75, 100, 150, 200, 300, 500]
  labels: [service, environment, verifier_type]
  description: "Verification latency P50 in milliseconds"
  collection_interval: 15s
  
- metric: verification_queue_depth
  type: Gauge
  labels: [service, environment, verifier_id]
  description: "Current verification queue depth"
  collection_interval: 10s
  alert_threshold: ">100"
```

#### 3.1.3 Batch 性能细分 (2 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-031** | batch_overhead_percent | Gauge | 30s | >20% | Batch 服务 | Batch 开销占比 |
| **M-032** | batch_nested_depth_current | Gauge | 实时 | >5 | Batch 服务 | 当前嵌套深度 |

**采集配置**:
```yaml
- metric: batch_overhead_percent
  type: Gauge
  labels: [service, environment, batch_size_range]
  description: "Batch processing overhead as percentage of total time"
  collection_interval: 30s
  alert_threshold: ">20"
  
- metric: batch_nested_depth_current
  type: Gauge
  labels: [service, environment, batch_id]
  description: "Current batch nesting depth"
  collection_interval: 1s
  alert_threshold: ">5"
```

#### 3.1.4 Transaction 性能细分 (2 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-033** | transaction_isolation_level_distribution | Histogram | 5min | - | Transaction 服务 | 隔离级别分布 |
| **M-034** | transaction_deadlock_count | Counter | 实时 | >0/h | Transaction 服务 | 死锁检测次数 |

**采集配置**:
```yaml
- metric: transaction_isolation_level_distribution
  type: Histogram
  labels: [service, environment, isolation_level]
  description: "Distribution of transaction isolation levels"
  collection_interval: 5m
  
- metric: transaction_deadlock_count
  type: Counter
  labels: [service, environment, transaction_id]
  description: "Count of detected deadlocks"
  collection_interval: 1s
  alert_threshold: ">0"
```

#### 3.1.5 追踪性能 (1 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-035** | trace_span_duration_p99 | Histogram | 实时 | >500ms | 追踪系统 | Span 时长 P99 |

**采集配置**:
```yaml
- metric: trace_span_duration_p99
  type: Histogram
  buckets: [10, 50, 100, 200, 300, 500, 750, 1000, 2500]
  labels: [service, environment, span_name, operation]
  description: "Trace span duration P99 in milliseconds"
  collection_interval: 15s
  alert_threshold: ">500"
```

### 3.2 错误指标扩展 (5 个新增)

#### 3.2.1 执行错误细分 (2 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-036** | execution_panic_count | Counter | 实时 | >0/h | 执行器 | Panic 次数 |
| **M-037** | execution_timeout_count | Counter | 实时 | >5/h | 执行器 | 执行超时次数 |

**采集配置**:
```yaml
- metric: execution_panic_count
  type: Counter
  labels: [service, environment, panic_location]
  description: "Count of execution panics"
  collection_interval: 1s
  alert_threshold: ">0"
  
- metric: execution_timeout_count
  type: Counter
  labels: [service, environment, instruction_type]
  description: "Count of execution timeouts"
  collection_interval: 1s
  alert_threshold: ">5"
```

#### 3.2.2 验证错误细分 (1 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-038** | verification_mismatch_count | Counter | 实时 | >0/h | 验证器 | 验证不匹配次数 |

**采集配置**:
```yaml
- metric: verification_mismatch_count
  type: Counter
  labels: [service, environment, mismatch_type]
  description: "Count of verification mismatches"
  collection_interval: 1s
  alert_threshold: ">0"
```

#### 3.2.3 Batch 错误细分 (1 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-039** | batch_partial_failure_count | Counter | 实时 | >0/h | Batch 服务 | Batch 部分失败次数 |

**采集配置**:
```yaml
- metric: batch_partial_failure_count
  type: Counter
  labels: [service, environment, batch_id, failure_reason]
  description: "Count of batch partial failures"
  collection_interval: 1s
  alert_threshold: ">0"
```

#### 3.2.4 Transaction 错误细分 (1 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-040** | transaction_abort_count | Counter | 实时 | >10/h | Transaction 服务 | Transaction 中止次数 |

**采集配置**:
```yaml
- metric: transaction_abort_count
  type: Counter
  labels: [service, environment, abort_reason]
  description: "Count of transaction aborts"
  collection_interval: 1s
  alert_threshold: ">10"
```

### 3.3 业务指标扩展 (8 个新增)

#### 3.3.1 指令执行细分 (2 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-041** | instruction_retry_count | Counter | 实时 | >20/h | 执行器 | 指令重试次数 |
| **M-042** | instruction_success_rate | Gauge | 30s | <99% | 执行器 | 指令成功率 |

**采集配置**:
```yaml
- metric: instruction_retry_count
  type: Counter
  labels: [service, environment, instruction_type, retry_reason]
  description: "Count of instruction retries"
  collection_interval: 1s
  alert_threshold: ">20"
  
- metric: instruction_success_rate
  type: Gauge
  labels: [service, environment, instruction_type]
  description: "Instruction success rate percentage"
  collection_interval: 30s
  alert_threshold: "<99"
```

#### 3.3.2 灰度发布细分 (1 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-043** | gray_release_rollback_count | Counter | 实时 | >0 | 灰度发布 | 灰度回滚次数 |

**采集配置**:
```yaml
- metric: gray_release_rollback_count
  type: Counter
  labels: [service, environment, release_id, rollback_reason]
  description: "Count of gray release rollbacks"
  collection_interval: 1s
  alert_threshold: ">0"
```

#### 3.3.3 安全合规则分 (3 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-044** | oidc_token_validation_latency_p99 | Histogram | 实时 | >100ms | 零信任模块 | OIDC 验证时延 |
| **M-045** | opa_policy_evaluation_count | Counter | 实时 | - | 零信任模块 | OPA 策略评估次数 |
| **M-046** | secret_rotation_success_rate | Gauge | 1h | <100% | 密钥管理 | 密钥轮换成功率 |

**采集配置**:
```yaml
- metric: oidc_token_validation_latency_p99
  type: Histogram
  buckets: [10, 25, 50, 75, 100, 150, 200, 300]
  labels: [service, environment, token_type]
  description: "OIDC token validation latency P99 in milliseconds"
  collection_interval: 15s
  alert_threshold: ">100"
  
- metric: opa_policy_evaluation_count
  type: Counter
  labels: [service, environment, policy_name, decision]
  description: "Count of OPA policy evaluations"
  collection_interval: 1s
  
- metric: secret_rotation_success_rate
  type: Gauge
  labels: [service, environment, secret_type]
  description: "Secret rotation success rate percentage"
  collection_interval: 1h
  alert_threshold: "<100"
```

#### 3.3.4 用户行为细分 (2 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-047** | client_request_rate | Gauge | 30s | - | Gateway | 客户端请求速率 |
| **M-048** | client_error_rate | Gauge | 30s | >5% | Gateway | 客户端错误率 |

**采集配置**:
```yaml
- metric: client_request_rate
  type: Gauge
  labels: [service, environment, client_version, endpoint]
  description: "Client request rate per second"
  collection_interval: 30s
  
- metric: client_error_rate
  type: Gauge
  labels: [service, environment, client_version, error_type]
  description: "Client error rate percentage"
  collection_interval: 30s
  alert_threshold: ">5"
```

### 3.4 系统指标扩展 (2 个新增)

#### 3.4.1 资源使用细分 (2 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-049** | disk_io_wait_percent | Gauge | 30s | >30% | 系统 | 磁盘 IO 等待 |
| **M-050** | network_packet_drop_rate | Gauge | 30s | >1% | 系统 | 网络丢包率 |

**采集配置**:
```yaml
- metric: disk_io_wait_percent
  type: Gauge
  labels: [service, environment, device]
  description: "Disk IO wait percentage"
  collection_interval: 30s
  alert_threshold: ">30"
  
- metric: network_packet_drop_rate
  type: Gauge
  labels: [service, environment, interface, direction]
  description: "Network packet drop rate percentage"
  collection_interval: 30s
  alert_threshold: ">1"
```

### 3.5 新增专项指标 (5 个)

#### 3.5.1 分布式追踪 (3 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-051** | trace_total_duration_p99 | Histogram | 实时 | >1000ms | 追踪系统 | 全链路时长 P99 |
| **M-052** | trace_span_count_avg | Gauge | 5min | - | 追踪系统 | 平均 Span 数量 |
| **M-053** | trace_propagation_success_rate | Gauge | 30s | <99% | 追踪系统 | 追踪传递成功率 |

**采集配置**:
```yaml
- metric: trace_total_duration_p99
  type: Histogram
  buckets: [100, 250, 500, 750, 1000, 1500, 2000, 3000, 5000]
  labels: [service, environment, trace_type]
  description: "Total trace duration P99 in milliseconds"
  collection_interval: 15s
  alert_threshold: ">1000"
  
- metric: trace_span_count_avg
  type: Gauge
  labels: [service, environment, trace_type]
  description: "Average span count per trace"
  collection_interval: 5m
  
- metric: trace_propagation_success_rate
  type: Gauge
  labels: [service, environment, propagation_method]
  description: "Trace propagation success rate percentage"
  collection_interval: 30s
  alert_threshold: "<99"
```

#### 3.5.2 威胁检测 (2 个新增)

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 来源 | 说明 |
|---|---|---|---|---|---|---|
| **M-054** | anomaly_detection_alert_count | Counter | 实时 | >5/h | 威胁检测 | 异常检测告警数 |
| **M-055** | threat_mitigation_time_avg | Gauge | 1h | - | 威胁检测 | 威胁处置平均时间 |

**采集配置**:
```yaml
- metric: anomaly_detection_alert_count
  type: Counter
  labels: [service, environment, anomaly_type, severity]
  description: "Count of anomaly detection alerts"
  collection_interval: 1s
  alert_threshold: ">5"
  
- metric: threat_mitigation_time_avg
  type: Gauge
  labels: [service, environment, threat_type]
  description: "Average threat mitigation time in seconds"
  collection_interval: 1h
```

---

## 4. 告警规则配置

### 4.1 P0 告警 (严重)

| 告警名 | 指标 | 表达式 | 阈值 | 持续时间 | 响应时间 |
|---|---|---|---|---|---|
| ExecutionP99High | execution_latency_p99 | `histogram_quantile(0.99, rate(execution_latency_bucket[5m]))` | >200ms | 5m | <5min |
| VerificationP99High | verification_latency_p99 | `histogram_quantile(0.99, rate(verification_latency_bucket[5m]))` | >200ms | 5m | <5min |
| BatchAtomicityViolation | batch_atomicity_violation_count | `increase(batch_atomicity_violation_count[5m])` | >0 | 1m | <5min |
| ExecutionPanic | execution_panic_count | `increase(execution_panic_count[5m])` | >0 | 1m | <5min |
| VerificationMismatch | verification_mismatch_count | `increase(verification_mismatch_count[5m])` | >0 | 1m | <5min |
| DeadlockDetected | transaction_deadlock_count | `increase(transaction_deadlock_count[5m])` | >0 | 1m | <5min |
| UnverifiedSubmit | gray_release_unverified_submit_rate | `gray_release_unverified_submit_rate` | >0 | 1m | <5min |
| AuthFailureSpike | zero_trust_auth_failure_count | `increase(zero_trust_auth_failure_count[5m])` | >5 | 5m | <5min |
| AnomalyAlertSpike | anomaly_detection_alert_count | `increase(anomaly_detection_alert_count[5m])` | >5 | 5m | <5min |

### 4.2 P1 告警 (高)

| 告警名 | 指标 | 表达式 | 阈值 | 持续时间 | 响应时间 |
|---|---|---|---|---|---|
| ExecutionP95High | execution_latency_p95 | `histogram_quantile(0.95, rate(execution_latency_bucket[5m]))` | >180ms | 5m | <15min |
| BatchLatencyHigh | batch_execute_latency_p99 | `histogram_quantile(0.99, rate(batch_execute_latency_bucket[5m]))` | >300ms | 5m | <15min |
| TransactionLatencyHigh | transaction_commit_latency_p99 | `histogram_quantile(0.99, rate(transaction_commit_latency_bucket[5m]))` | >300ms | 5m | <15min |
| ExecutorQueueDeep | executor_queue_depth | `executor_queue_depth` | >100 | 5m | <15min |
| VerificationQueueDeep | verification_queue_depth | `verification_queue_depth` | >100 | 5m | <15min |
| BatchOverheadHigh | batch_overhead_percent | `batch_overhead_percent` | >20% | 10m | <15min |
| TransactionRollbackHigh | transaction_rollback_count | `increase(transaction_rollback_count[1h])` | >5 | 1h | <15min |
| InstructionRetryHigh | instruction_retry_count | `increase(instruction_retry_count[1h])` | >20 | 1h | <15min |
| InstructionSuccessLow | instruction_success_rate | `instruction_success_rate` | <99% | 10m | <15min |
| TracePropagationLow | trace_propagation_success_rate | `trace_propagation_success_rate` | <99% | 10m | <15min |

### 4.3 P2 告警 (中)

| 告警名 | 指标 | 表达式 | 阈值 | 持续时间 | 响应时间 |
|---|---|---|---|---|---|
| CPUHigh | cpu_usage_percent | `cpu_usage_percent` | >80% | 10m | <1h |
| MemoryHigh | memory_usage_percent | `memory_usage_percent` | >85% | 10m | <1h |
| DiskIOWaitHigh | disk_io_wait_percent | `disk_io_wait_percent` | >30% | 10m | <1h |
| NetworkDropHigh | network_packet_drop_rate | `network_packet_drop_rate` | >1% | 10m | <1h |
| TransactionTimeoutHigh | transaction_timeout_count | `increase(transaction_timeout_count[1h])` | >3 | 1h | <1h |
| PolicyViolationHigh | zero_trust_policy_violation_count | `increase(zero_trust_policy_violation_count[1h])` | >2 | 1h | <1h |
| TraceCoverageLow | distributed_trace_coverage | `distributed_trace_coverage` | <98% | 1h | <1h |
| SecretRotationFail | secret_rotation_success_rate | `secret_rotation_success_rate` | <100% | 1h | <1h |
| ClientErrorHigh | client_error_rate | `client_error_rate` | >5% | 10m | <1h |

---

## 5. 采集实现方案

### 5.1 Rust 代码集成

```rust
// metrics_phase3.rs - Phase 3 新增指标采集

use prometheus::{Histogram, Counter, Gauge, HistogramOpts, Opts, register_histogram, register_counter, register_gauge};

lazy_static! {
    // === 性能指标 ===
    
    // 执行性能
    pub static ref EXECUTION_LATENCY_P50: Histogram = register_histogram!(
        HistogramOpts::new("execution_latency_p50", "Execution latency P50 in ms")
            .buckets(vec![10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 300.0, 500.0])
    ).unwrap();
    
    pub static ref EXECUTION_LATENCY_P95: Histogram = register_histogram!(
        HistogramOpts::new("execution_latency_p95", "Execution latency P95 in ms")
            .buckets(vec![50.0, 100.0, 150.0, 200.0, 250.0, 300.0, 400.0, 500.0, 750.0])
    ).unwrap();
    
    pub static ref EXECUTOR_QUEUE_DEPTH: Gauge = register_gauge!(
        Opts::new("executor_queue_depth", "Current executor queue depth")
    ).unwrap();
    
    // 验证性能
    pub static ref VERIFICATION_LATENCY_P50: Histogram = register_histogram!(
        HistogramOpts::new("verification_latency_p50", "Verification latency P50 in ms")
            .buckets(vec![10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 300.0, 500.0])
    ).unwrap();
    
    pub static ref VERIFICATION_QUEUE_DEPTH: Gauge = register_gauge!(
        Opts::new("verification_queue_depth", "Current verification queue depth")
    ).unwrap();
    
    // Batch 性能
    pub static ref BATCH_OVERHEAD_PERCENT: Gauge = register_gauge!(
        Opts::new("batch_overhead_percent", "Batch processing overhead percentage")
    ).unwrap();
    
    pub static ref BATCH_NESTED_DEPTH_CURRENT: Gauge = register_gauge!(
        Opts::new("batch_nested_depth_current", "Current batch nesting depth")
    ).unwrap();
    
    // Transaction 性能
    pub static ref TRANSACTION_DEADLOCK_COUNT: Counter = register_counter!(
        Opts::new("transaction_deadlock_count", "Count of detected deadlocks")
    ).unwrap();
    
    // 追踪性能
    pub static ref TRACE_SPAN_DURATION_P99: Histogram = register_histogram!(
        HistogramOpts::new("trace_span_duration_p99", "Trace span duration P99 in ms")
            .buckets(vec![10.0, 50.0, 100.0, 200.0, 300.0, 500.0, 750.0, 1000.0, 2500.0])
    ).unwrap();
    
    // === 错误指标 ===
    
    pub static ref EXECUTION_PANIC_COUNT: Counter = register_counter!(
        Opts::new("execution_panic_count", "Count of execution panics")
    ).unwrap();
    
    pub static ref EXECUTION_TIMEOUT_COUNT: Counter = register_counter!(
        Opts::new("execution_timeout_count", "Count of execution timeouts")
    ).unwrap();
    
    pub static ref VERIFICATION_MISMATCH_COUNT: Counter = register_counter!(
        Opts::new("verification_mismatch_count", "Count of verification mismatches")
    ).unwrap();
    
    pub static ref BATCH_PARTIAL_FAILURE_COUNT: Counter = register_counter!(
        Opts::new("batch_partial_failure_count", "Count of batch partial failures")
    ).unwrap();
    
    pub static ref TRANSACTION_ABORT_COUNT: Counter = register_counter!(
        Opts::new("transaction_abort_count", "Count of transaction aborts")
    ).unwrap();
    
    // === 业务指标 ===
    
    pub static ref INSTRUCTION_RETRY_COUNT: Counter = register_counter!(
        Opts::new("instruction_retry_count", "Count of instruction retries")
    ).unwrap();
    
    pub static ref INSTRUCTION_SUCCESS_RATE: Gauge = register_gauge!(
        Opts::new("instruction_success_rate", "Instruction success rate percentage")
    ).unwrap();
    
    pub static ref GRAY_RELEASE_ROLLBACK_COUNT: Counter = register_counter!(
        Opts::new("gray_release_rollback_count", "Count of gray release rollbacks")
    ).unwrap();
    
    pub static ref OIDC_TOKEN_VALIDATION_LATENCY_P99: Histogram = register_histogram!(
        HistogramOpts::new("oidc_token_validation_latency_p99", "OIDC token validation latency P99 in ms")
            .buckets(vec![10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 300.0])
    ).unwrap();
    
    pub static ref OPA_POLICY_EVALUATION_COUNT: Counter = register_counter!(
        Opts::new("opa_policy_evaluation_count", "Count of OPA policy evaluations")
    ).unwrap();
    
    pub static ref SECRET_ROTATION_SUCCESS_RATE: Gauge = register_gauge!(
        Opts::new("secret_rotation_success_rate", "Secret rotation success rate percentage")
    ).unwrap();
    
    pub static ref CLIENT_REQUEST_RATE: Gauge = register_gauge!(
        Opts::new("client_request_rate", "Client request rate per second")
    ).unwrap();
    
    pub static ref CLIENT_ERROR_RATE: Gauge = register_gauge!(
        Opts::new("client_error_rate", "Client error rate percentage")
    ).unwrap();
    
    // === 系统指标 ===
    
    pub static ref DISK_IO_WAIT_PERCENT: Gauge = register_gauge!(
        Opts::new("disk_io_wait_percent", "Disk IO wait percentage")
    ).unwrap();
    
    pub static ref NETWORK_PACKET_DROP_RATE: Gauge = register_gauge!(
        Opts::new("network_packet_drop_rate", "Network packet drop rate percentage")
    ).unwrap();
    
    // === 追踪指标 ===
    
    pub static ref TRACE_TOTAL_DURATION_P99: Histogram = register_histogram!(
        HistogramOpts::new("trace_total_duration_p99", "Total trace duration P99 in ms")
            .buckets(vec![100.0, 250.0, 500.0, 750.0, 1000.0, 1500.0, 2000.0, 3000.0, 5000.0])
    ).unwrap();
    
    pub static ref TRACE_SPAN_COUNT_AVG: Gauge = register_gauge!(
        Opts::new("trace_span_count_avg", "Average span count per trace")
    ).unwrap();
    
    pub static ref TRACE_PROPAGATION_SUCCESS_RATE: Gauge = register_gauge!(
        Opts::new("trace_propagation_success_rate", "Trace propagation success rate percentage")
    ).unwrap();
    
    // === 威胁检测指标 ===
    
    pub static ref ANOMALY_DETECTION_ALERT_COUNT: Counter = register_counter!(
        Opts::new("anomaly_detection_alert_count", "Count of anomaly detection alerts")
    ).unwrap();
    
    pub static ref THREAT_MITIGATION_TIME_AVG: Gauge = register_gauge!(
        Opts::new("threat_mitigation_time_avg", "Average threat mitigation time in seconds")
    ).unwrap();
}

// === 指标采集辅助函数 ===

pub fn observe_execution_latency(latency_ms: f64) {
    EXECUTION_LATENCY_P50.observe(latency_ms);
    EXECUTION_LATENCY_P95.observe(latency_ms);
}

pub fn set_executor_queue_depth(depth: usize) {
    EXECUTOR_QUEUE_DEPTH.set(depth as f64);
}

pub fn observe_verification_latency(latency_ms: f64) {
    VERIFICATION_LATENCY_P50.observe(latency_ms);
}

pub fn set_verification_queue_depth(depth: usize) {
    VERIFICATION_QUEUE_DEPTH.set(depth as f64);
}

pub fn inc_execution_panic(location: &str) {
    EXECUTION_PANIC_COUNT.inc();
    log::error!("Execution panic detected at: {}", location);
}

pub fn inc_execution_timeout(instruction_type: &str) {
    EXECUTION_TIMEOUT_COUNT.inc();
    log::warn!("Execution timeout for instruction type: {}", instruction_type);
}

pub fn inc_verification_mismatch(mismatch_type: &str) {
    VERIFICATION_MISMATCH_COUNT.inc();
    log::error!("Verification mismatch detected: {}", mismatch_type);
}

pub fn inc_transaction_deadlock(transaction_id: &str) {
    TRANSACTION_DEADLOCK_COUNT.inc();
    log::error!("Deadlock detected for transaction: {}", transaction_id);
}

pub fn set_batch_nested_depth(depth: u8) {
    BATCH_NESTED_DEPTH_CURRENT.set(depth as f64);
}

pub fn observe_trace_span_duration(duration_ms: f64) {
    TRACE_SPAN_DURATION_P99.observe(duration_ms);
}

pub fn set_trace_propagation_success_rate(rate: f64) {
    TRACE_PROPAGATION_SUCCESS_RATE.set(rate);
}

pub fn inc_anomaly_detection_alert(anomaly_type: &str, severity: &str) {
    ANOMALY_DETECTION_ALERT_COUNT.inc();
    log::warn!("Anomaly detection alert: type={}, severity={}", anomaly_type, severity);
}
```

### 5.2 采集点位置

| 采集点 | 位置 | 指标 | 触发条件 |
|---|---|---|---|
| 指令执行完成 | executor.rs | execution_latency_p50/p95 | 每次指令执行 |
| 执行队列更新 | executor.rs | executor_queue_depth | 队列变化时 |
| 验证完成 | verifier.rs | verification_latency_p50 | 每次验证 |
| 验证队列更新 | verifier.rs | verification_queue_depth | 队列变化时 |
| Batch 执行 | batch_executor.rs | batch_overhead_percent, batch_nested_depth_current | 每次 Batch 执行 |
| Transaction 死锁检测 | transaction_manager.rs | transaction_deadlock_count | 死锁检测 |
| Panic 捕获 | panic_hook.rs | execution_panic_count | Panic 发生时 |
| 执行超时 | executor.rs | execution_timeout_count | 超时发生时 |
| 验证不匹配 | verifier.rs | verification_mismatch_count | 验证失败时 |
| Batch 部分失败 | batch_executor.rs | batch_partial_failure_count | 部分失败时 |
| Transaction 中止 | transaction_manager.rs | transaction_abort_count | 中止时 |
| 指令重试 | executor.rs | instruction_retry_count | 重试时 |
| OIDC 验证 | auth_middleware.rs | oidc_token_validation_latency_p99 | 每次验证 |
| OPA 评估 | opa_client.rs | opa_policy_evaluation_count | 每次评估 |
| 密钥轮换 | secret_manager.rs | secret_rotation_success_rate | 轮换完成时 |
| 请求入口 | gateway.rs | client_request_rate, client_error_rate | 每次请求 |
| Span 结束 | tracing.rs | trace_span_duration_p99 | Span 结束时 |
| 追踪传递 | tracing.rs | trace_propagation_success_rate | 传递完成时 |
| 全链路追踪 | tracing.rs | trace_total_duration_p99, trace_span_count_avg | Trace 结束时 |
| 异常检测 | threat_detection.rs | anomaly_detection_alert_count | 告警触发时 |
| 威胁处置 | threat_detection.rs | threat_mitigation_time_avg | 处置完成时 |
| 磁盘 IO | node_exporter | disk_io_wait_percent | 定期采集 |
| 网络丢包 | node_exporter | network_packet_drop_rate | 定期采集 |

---

## 6. 接入计划

### 6.1 时间规划

| 周次 | 任务 | 责任人 | 状态 | 交付物 |
|---|---|---|---|---|
| Week 1-T1 | 25 个新增指标定义评审 | SRE+Observability | 📋 待开始 | phase3_50_metrics_plan.md |
| Week 1-T2 | Prometheus 指标配置 | SRE | 📋 待开始 | prometheus-phase3.yml |
| Week 2-T1 | Rust 代码指标采集集成 | Dev | 📋 待开始 | metrics_phase3.rs |
| Week 2-T2 | 告警规则配置 | SRE | 📋 待开始 | alerting-rules-phase3.yml |
| Week 3-T1 | Grafana 仪表盘 v6 配置 | SRE+Observability | 📋 待开始 | monitoring_dashboard_v6.md |
| Week 3-T2 | 分布式追踪集成 | Observability | 📋 待开始 | distributed_tracing.md |
| Week 4-T1 | 50 指标全量验证 | SRE+QA | 📋 待开始 | metrics_validation_report.md |
| Week 5-T1 | 性能基线 v5 测量 | SRE | 📋 待开始 | performance_baseline_v5.md |

### 6.2 验证标准

| 验证项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 指标采集 | 50 个指标均有数据 | Prometheus 查询 | 100% 指标可查询 |
| 告警触发 | 阈值正确触发 | 模拟告警测试 | 100% 告警规则有效 |
| 仪表盘显示 | 50 个指标全部展示 | Grafana 检查 | 所有 Panel 正常 |
| 数据准确性 | 与日志一致 | 抽样比对 | 误差<1% |
| 采集开销 | 性能影响<1% | 压测对比 | 时延增加<1% |
| 追踪覆盖 | trace_id 全链路 | 链路追踪验证 | 覆盖率≥98% |

---

## 7. 附录

### 7.1 完整指标清单

| 类别 | 指标数 | 指标 ID 范围 |
|---|---|---|
| Phase 2 继承 | 25 | M-001 ~ M-025 |
| 性能指标新增 | 10 | M-026 ~ M-035 |
| 错误指标新增 | 5 | M-036 ~ M-040 |
| 业务指标新增 | 8 | M-041 ~ M-048 |
| 系统指标新增 | 2 | M-049 ~ M-050 |
| 追踪指标新增 | 3 | M-051 ~ M-053 |
| 威胁检测新增 | 2 | M-054 ~ M-055 |
| **总计** | **55** | **M-001 ~ M-055** |

注：实际目标 50 个，规划 55 个以提供缓冲，最终可根据实际情况调整。

### 7.2 Prometheus 配置完整示例

```yaml
# prometheus-phase3.yml

global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'cgas-executor'
    static_configs:
      - targets: ['executor:8080']
    metrics_path: '/metrics'
    metric_relabel_configs:
      - source_labels: [__name__]
        regex: 'execution_.*'
        action: keep
        
  - job_name: 'cgas-verifier'
    static_configs:
      - targets: ['verifier:8081']
    metrics_path: '/metrics'
    
  - job_name: 'cgas-batch'
    static_configs:
      - targets: ['batch:8082']
    metrics_path: '/metrics'
    
  - job_name: 'cgas-transaction'
    static_configs:
      - targets: ['transaction:8083']
    metrics_path: '/metrics'
    
  - job_name: 'cgas-gateway'
    static_configs:
      - targets: ['gateway:8084']
    metrics_path: '/metrics'
    
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
```

### 7.3 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Phase 2 监控配置 | phase2_week2_monitoring_config.md | 继承配置参考 |
| Phase 2 仪表盘 v5 | monitoring_dashboard_v5.md | 仪表盘演进基线 |
| Phase 3 PRD v3 | phase3_prd_v3.md | 需求来源 |
| Phase 3 多 Agent 启动 | phase3_multiagent_kickoff.md | 任务来源 |

---

**文档状态**: 📋 规划中  
**创建日期**: 2026-05-12  
**责任人**: SRE-Agent  
**保管**: 项目文档库
