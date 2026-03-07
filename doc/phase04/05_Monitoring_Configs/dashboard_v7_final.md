# Grafana 仪表盘 v7 最终版 - 50 指标全量配置

**版本**: v7.0 Final  
**日期**: 2026-03-14  
**责任人**: SRE-Agent + Observability-Agent  
**状态**: ✅ Week 5 完成  
**release_id**: release-2026-03-14-phase3-week5-dashboard-final  
**参与角色**: SRE, Observability, Dev

---

## 1. 概述

### 1.1 任务目标

在 Phase 3 Week 5 完成剩余 20 个监控指标的仪表盘配置，实现 **50 指标全量接入**，完善 Phase 3 可观测性体系。

### 1.2 50 指标完成状态

| 批次 | 指标数 | 完成时间 | 状态 | 仪表盘 |
|---|---|---|---|---|
| Batch 1 | 10 个 | Week 2 | ✅ 完成 | Overview, Performance, Tracing, System |
| Batch 2 | 10 个 | Week 3 | ✅ 完成 | Consistency, Security |
| Batch 3 | 10 个 | Week 4 | ✅ 完成 | API Performance, User Experience |
| **Batch 4** | **20 个** | **Week 5** | **✅ 完成** | **Batch, Transaction, Business** |
| **总计** | **50 个** | **Week 5** | **✅ 完成** | **10 个仪表盘** |

### 1.3 Batch 4 新增 20 指标清单

| # | 指标 ID | 指标名 | 类型 | P0 告警阈值 | 来源 | 优先级 | 仪表盘 |
|---|---|---|---|---|---|---|---|
| 1 | M-026 | execution_latency_p50 | Histogram | >100ms | Executor | P2 | Performance+ |
| 2 | M-027 | execution_latency_p95 | Histogram | >180ms | Executor | P1 | Performance+ |
| 3 | M-028 | executor_queue_depth | Gauge | >100 | Executor | P1 | Performance+ |
| 4 | M-029 | verification_latency_p50 | Histogram | >100ms | Verifier | P2 | Performance+ |
| 5 | M-030 | verification_queue_depth | Gauge | >100 | Verifier | P1 | Performance+ |
| 6 | M-031 | batch_overhead_percent | Gauge | >20% | Batch | P1 | Batch |
| 7 | M-032 | batch_nested_depth_current | Gauge | >5 | Batch | P1 | Batch |
| 8 | M-033 | transaction_isolation_level | Histogram | - | Transaction | P2 | Transaction |
| 9 | M-034 | transaction_deadlock_count | Counter | >0/h | Transaction | P0 | Transaction |
| 10 | M-036 | execution_panic_count | Counter | >0/h | Executor | P0 | Error |
| 11 | M-037 | execution_timeout_count | Counter | >5/h | Executor | P1 | Error |
| 12 | M-038 | verification_mismatch_count | Counter | >0/h | Verifier | P0 | Error |
| 13 | M-039 | batch_partial_failure_count | Counter | >0/h | Batch | P0 | Error |
| 14 | M-040 | transaction_abort_count | Counter | >10/h | Transaction | P1 | Error |
| 15 | M-041 | instruction_retry_count | Counter | >20/h | Executor | P1 | Business |
| 16 | M-042 | instruction_success_rate | Gauge | <99% | Executor | P1 | Business |
| 17 | M-043 | gray_release_rollback_count | Counter | >0 | Gray Release | P0 | Business |
| 18 | M-044 | oidc_validation_latency_p99 | Histogram | >100ms | Security | P1 | Security+ |
| 19 | M-045 | opa_policy_eval_count | Counter | - | Security | P2 | Security+ |
| 20 | M-046 | secret_rotation_success_rate | Gauge | <100% | Security | P1 | Security+ |

---

## 2. Performance+ 仪表盘 (Batch 4 扩展)

### 2.1 仪表盘配置

**Dashboard UID**: `phase3-performance-plus`  
**标题**: Phase 3 性能监控扩展  
**刷新频率**: 15s  
**时区**: Asia/Shanghai

### 2.2 新增 Panel 配置

#### Panel 1: Execution Latency P50/P95/P99 对比

```json
{
  "id": 1,
  "title": "⏱️ Execution Latency P50/P95/P99 对比",
  "type": "timeseries",
  "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
  "targets": [
    {
      "expr": "histogram_quantile(0.50, sum(rate(execution_latency_bucket[5m])) by(le))",
      "legendFormat": "P50",
      "refId": "A"
    },
    {
      "expr": "histogram_quantile(0.95, sum(rate(execution_latency_bucket[5m])) by(le))",
      "legendFormat": "P95",
      "refId": "B"
    },
    {
      "expr": "histogram_quantile(0.99, sum(rate(execution_latency_bucket[5m])) by(le))",
      "legendFormat": "P99",
      "refId": "C"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "ms",
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 150, "color": "yellow"},
          {"value": 200, "color": "red"}
        ]
      }
    }
  },
  "options": {
    "tooltip": {"mode": "multi"},
    "legend": {"displayMode": "table", "placement": "right"}
  }
}
```

#### Panel 2: Executor Queue Depth

```json
{
  "id": 2,
  "title": "📊 Executor Queue Depth",
  "type": "gauge",
  "gridPos": {"h": 6, "w": 6, "x": 12, "y": 0},
  "targets": [
    {
      "expr": "executor_queue_depth",
      "legendFormat": "Queue Depth",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "min": 0,
      "max": 150,
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 80, "color": "yellow"},
          {"value": 100, "color": "red"}
        ]
      }
    }
  }
}
```

#### Panel 3: Verification Latency P50/P95/P99 对比

```json
{
  "id": 3,
  "title": "✅ Verification Latency P50/P95/P99 对比",
  "type": "timeseries",
  "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8},
  "targets": [
    {
      "expr": "histogram_quantile(0.50, sum(rate(verification_latency_bucket[5m])) by(le))",
      "legendFormat": "P50",
      "refId": "A"
    },
    {
      "expr": "histogram_quantile(0.95, sum(rate(verification_latency_bucket[5m])) by(le))",
      "legendFormat": "P95",
      "refId": "B"
    },
    {
      "expr": "histogram_quantile(0.99, sum(rate(verification_latency_bucket[5m])) by(le))",
      "legendFormat": "P99",
      "refId": "C"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "ms",
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 150, "color": "yellow"},
          {"value": 200, "color": "red"}
        ]
      }
    }
  }
}
```

#### Panel 4: Verification Queue Depth

```json
{
  "id": 4,
  "title": "📊 Verification Queue Depth",
  "type": "gauge",
  "gridPos": {"h": 6, "w": 6, "x": 12, "y": 8},
  "targets": [
    {
      "expr": "verification_queue_depth",
      "legendFormat": "Queue Depth",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "min": 0,
      "max": 150,
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 80, "color": "yellow"},
          {"value": 100, "color": "red"}
        ]
      }
    }
  }
}
```

---

## 3. Batch 仪表盘

### 3.1 仪表盘配置

**Dashboard UID**: `phase3-batch`  
**标题**: Phase 3 Batch 服务监控  
**刷新频率**: 15s  
**时区**: Asia/Shanghai

### 3.2 仪表盘布局

```
Batch Dashboard
├── Row 1: Batch 核心指标
│   ├── Panel 1: Batch Overhead % - Gauge
│   ├── Panel 2: Batch Nested Depth - Gauge
│   └── Panel 3: Batch Execute Latency P99 - Time Series
├── Row 2: Batch 错误分析
│   ├── Panel 4: Batch Partial Failure Count - Time Series
│   ├── Panel 5: Batch Atomicity Violation Count - Stat
│   └── Panel 6: Batch Sub-Instruction Count - Histogram
└── Row 3: Batch 性能分析
    ├── Panel 7: Batch Size Distribution - Heatmap
    ├── Panel 8: Batch Success Rate - Gauge
    └── Panel 9: Batch Throughput - Time Series
```

### 3.3 关键 Panel 配置

#### Panel 1: Batch Overhead Percent

```json
{
  "id": 1,
  "title": "📦 Batch Overhead %",
  "type": "gauge",
  "gridPos": {"h": 6, "w": 8, "x": 0, "y": 0},
  "targets": [
    {
      "expr": "batch_overhead_percent",
      "legendFormat": "Overhead %",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "percent",
      "min": 0,
      "max": 30,
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 15, "color": "yellow"},
          {"value": 20, "color": "red"}
        ]
      }
    }
  }
}
```

#### Panel 2: Batch Nested Depth Current

```json
{
  "id": 2,
  "title": "📦 Batch Nested Depth",
  "type": "gauge",
  "gridPos": {"h": 6, "w": 8, "x": 8, "y": 0},
  "targets": [
    {
      "expr": "batch_nested_depth_current",
      "legendFormat": "Current Depth",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "min": 0,
      "max": 10,
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 3, "color": "yellow"},
          {"value": 5, "color": "red"}
        ]
      }
    }
  }
}
```

#### Panel 4: Batch Partial Failure Count

```json
{
  "id": 4,
  "title": "⚠️ Batch Partial Failure Count",
  "type": "timeseries",
  "gridPos": {"h": 8, "w": 12, "x": 0, "y": 6},
  "targets": [
    {
      "expr": "increase(batch_partial_failure_count[5m])",
      "legendFormat": "Failures (5m)",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 1, "color": "yellow"},
          {"value": 5, "color": "red"}
        ]
      }
    }
  },
  "alert": {
    "name": "BatchPartialFailure-P0",
    "conditions": [
      {
        "evaluator": {"params": [0], "type": "gt"},
        "query": {"params": ["A", "5m", "now"]}
      }
    ]
  }
}
```

---

## 4. Transaction 仪表盘

### 4.1 仪表盘配置

**Dashboard UID**: `phase3-transaction`  
**标题**: Phase 3 Transaction 服务监控  
**刷新频率**: 15s  
**时区**: Asia/Shanghai

### 4.2 仪表盘布局

```
Transaction Dashboard
├── Row 1: Transaction 核心指标
│   ├── Panel 1: Transaction Commit Latency P99 - Time Series
│   ├── Panel 2: Transaction Deadlock Count - Stat
│   └── Panel 3: Transaction Rollback Count - Time Series
├── Row 2: Transaction 隔离性
│   ├── Panel 4: Isolation Level Distribution - Pie Chart
│   ├── Panel 5: Transaction Abort Count - Time Series
│   └── Panel 6: Transaction Timeout Count - Stat
└── Row 3: Transaction 性能
    ├── Panel 7: Transaction Duration P50/P95/P99 - Time Series
    ├── Panel 8: Transaction Throughput - Time Series
    └── Panel 9: Transaction Success Rate - Gauge
```

### 4.3 关键 Panel 配置

#### Panel 2: Transaction Deadlock Count

```json
{
  "id": 2,
  "title": "🔒 Transaction Deadlock Count",
  "type": "stat",
  "gridPos": {"h": 6, "w": 8, "x": 8, "y": 0},
  "targets": [
    {
      "expr": "increase(transaction_deadlock_count[1h])",
      "legendFormat": "Deadlocks (1h)",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 1, "color": "yellow"},
          {"value": 5, "color": "red"}
        ]
      },
      "mappings": [
        {
          "type": "range",
          "options": {
            "from": 0,
            "to": 0,
            "text": "✅ No Deadlocks",
            "color": "green"
          }
        },
        {
          "type": "range",
          "options": {
            "from": 1,
            "to": 100,
            "text": "❌ Deadlocks Detected",
            "color": "red"
          }
        }
      ]
    }
  },
  "alert": {
    "name": "TransactionDeadlock-P0",
    "conditions": [
      {
        "evaluator": {"params": [0], "type": "gt"},
        "query": {"params": ["A", "1h", "now"]}
      }
    ]
  }
}
```

#### Panel 5: Transaction Abort Count

```json
{
  "id": 5,
  "title": "🚫 Transaction Abort Count",
  "type": "timeseries",
  "gridPos": {"h": 8, "w": 12, "x": 0, "y": 6},
  "targets": [
    {
      "expr": "increase(transaction_abort_count[5m])",
      "legendFormat": "Aborts (5m)",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 5, "color": "yellow"},
          {"value": 10, "color": "red"}
        ]
      }
    }
  }
}
```

---

## 5. Error 仪表盘

### 5.1 仪表盘配置

**Dashboard UID**: `phase3-errors`  
**标题**: Phase 3 错误监控  
**刷新频率**: 15s  
**时区**: Asia/Shanghai

### 5.2 仪表盘布局

```
Error Dashboard
├── Row 1: 执行错误
│   ├── Panel 1: Execution Panic Count - Stat
│   ├── Panel 2: Execution Timeout Count - Time Series
│   └── Panel 3: Execution Error Rate - Gauge
├── Row 2: 验证错误
│   ├── Panel 4: Verification Mismatch Count - Time Series
│   ├── Panel 5: Verification Error Rate - Gauge
│   └── Panel 6: Verification Mismatch by Type - Bar Chart
└── Row 3: 批处理与事务错误
    ├── Panel 7: Batch Partial Failure Count - Time Series
    ├── Panel 8: Transaction Abort Count - Time Series
    └── Panel 9: Total Error Trend - Time Series
```

### 5.3 关键 Panel 配置

#### Panel 1: Execution Panic Count

```json
{
  "id": 1,
  "title": "🔥 Execution Panic Count",
  "type": "stat",
  "gridPos": {"h": 6, "w": 8, "x": 0, "y": 0},
  "targets": [
    {
      "expr": "increase(execution_panic_count[1h])",
      "legendFormat": "Panics (1h)",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 1, "color": "red"}
        ]
      },
      "mappings": [
        {
          "type": "range",
          "options": {
            "from": 0,
            "to": 0,
            "text": "✅ No Panics",
            "color": "green"
          }
        },
        {
          "type": "range",
          "options": {
            "from": 1,
            "to": 100,
            "text": "❌ Panics Detected",
            "color": "red"
          }
        }
      ]
    }
  },
  "alert": {
    "name": "ExecutionPanic-P0",
    "conditions": [
      {
        "evaluator": {"params": [0], "type": "gt"},
        "query": {"params": ["A", "1h", "now"]}
      }
    ]
  }
}
```

#### Panel 4: Verification Mismatch Count

```json
{
  "id": 4,
  "title": "❌ Verification Mismatch Count",
  "type": "timeseries",
  "gridPos": {"h": 8, "w": 12, "x": 0, "y": 6},
  "targets": [
    {
      "expr": "increase(verification_mismatch_count[5m])",
      "legendFormat": "Mismatches (5m)",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 1, "color": "yellow"},
          {"value": 5, "color": "red"}
        ]
      }
    }
  },
  "alert": {
    "name": "VerificationMismatch-P0",
    "conditions": [
      {
        "evaluator": {"params": [0], "type": "gt"},
        "query": {"params": ["A", "5m", "now"]}
      }
    ]
  }
}
```

---

## 6. Business 仪表盘

### 6.1 仪表盘配置

**Dashboard UID**: `phase3-business`  
**标题**: Phase 3 业务指标监控  
**刷新频率**: 30s  
**时区**: Asia/Shanghai

### 6.2 仪表盘布局

```
Business Dashboard
├── Row 1: 指令执行
│   ├── Panel 1: Instruction Success Rate - Gauge
│   ├── Panel 2: Instruction Retry Count - Time Series
│   └── Panel 3: Instruction Throughput - Time Series
├── Row 2: 灰度发布
│   ├── Panel 4: Gray Release Rollback Count - Stat
│   ├── Panel 5: Gray Release Consistency Rate - Gauge
│   └── Panel 6: Gray Release Coverage - Stat
└── Row 3: 安全合规
    ├── Panel 7: OIDC Validation Latency P99 - Time Series
    ├── Panel 8: OPA Policy Evaluation Count - Time Series
    └── Panel 9: Secret Rotation Success Rate - Gauge
```

### 6.3 关键 Panel 配置

#### Panel 1: Instruction Success Rate

```json
{
  "id": 1,
  "title": "✅ Instruction Success Rate",
  "type": "gauge",
  "gridPos": {"h": 6, "w": 8, "x": 0, "y": 0},
  "targets": [
    {
      "expr": "instruction_success_rate",
      "legendFormat": "Success Rate %",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "percent",
      "min": 0,
      "max": 100,
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "red"},
          {"value": 95, "color": "yellow"},
          {"value": 99, "color": "green"}
        ]
      }
    }
  },
  "alert": {
    "name": "InstructionSuccessLow-P1",
    "conditions": [
      {
        "evaluator": {"params": [99], "type": "lt"},
        "query": {"params": ["A", "10m", "now"]}
      }
    ]
  }
}
```

#### Panel 4: Gray Release Rollback Count

```json
{
  "id": 4,
  "title": "🔄 Gray Release Rollback Count",
  "type": "stat",
  "gridPos": {"h": 6, "w": 8, "x": 0, "y": 6},
  "targets": [
    {
      "expr": "increase(gray_release_rollback_count[24h])",
      "legendFormat": "Rollbacks (24h)",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 1, "color": "red"}
        ]
      },
      "mappings": [
        {
          "type": "range",
          "options": {
            "from": 0,
            "to": 0,
            "text": "✅ No Rollbacks",
            "color": "green"
          }
        },
        {
          "type": "range",
          "options": {
            "from": 1,
            "to": 100,
            "text": "❌ Rollbacks Detected",
            "color": "red"
          }
        }
      ]
    }
  },
  "alert": {
    "name": "GrayReleaseRollback-P0",
    "conditions": [
      {
        "evaluator": {"params": [0], "type": "gt"},
        "query": {"params": ["A", "24h", "now"]}
      }
    ]
  }
}
```

#### Panel 7: OIDC Validation Latency P99

```json
{
  "id": 7,
  "title": "🔐 OIDC Validation Latency P99",
  "type": "timeseries",
  "gridPos": {"h": 8, "w": 12, "x": 0, "y": 12},
  "targets": [
    {
      "expr": "histogram_quantile(0.99, rate(oidc_token_validation_latency_p99_bucket[5m]))",
      "legendFormat": "P99",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "ms",
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "green"},
          {"value": 80, "color": "yellow"},
          {"value": 100, "color": "red"}
        ]
      }
    }
  }
}
```

#### Panel 9: Secret Rotation Success Rate

```json
{
  "id": 9,
  "title": "🔑 Secret Rotation Success Rate",
  "type": "gauge",
  "gridPos": {"h": 6, "w": 8, "x": 0, "y": 20},
  "targets": [
    {
      "expr": "secret_rotation_success_rate",
      "legendFormat": "Success Rate %",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "percent",
      "min": 0,
      "max": 100,
      "thresholds": {
        "mode": "absolute",
        "steps": [
          {"value": null, "color": "red"},
          {"value": 100, "color": "green"}
        ]
      }
    }
  },
  "alert": {
    "name": "SecretRotationFail-P2",
    "conditions": [
      {
        "evaluator": {"params": [100], "type": "lt"},
        "query": {"params": ["A", "1h", "now"]}
      }
    ]
  }
}
```

---

## 7. 仪表盘汇总

### 7.1 完整仪表盘列表

| # | 仪表盘名称 | UID | 指标数 | Panel 数 | 优先级 |
|---|---|---|---|---|---|
| 1 | Phase 3 Overview | `phase3-overview` | 6 | 8 | P0 |
| 2 | Phase 3 Performance | `phase3-performance` | 4 | 6 | P0 |
| 3 | Phase 3 Performance+ | `phase3-performance-plus` | 6 | 8 | P1 |
| 4 | Phase 3 Tracing | `phase3-tracing` | 5 | 8 | P0 |
| 5 | Phase 3 System | `phase3-system` | 4 | 4 | P1 |
| 6 | Phase 3 API Performance | `phase3-api-performance` | 6 | 12 | P1 |
| 7 | Phase 3 User Experience | `phase3-user-experience` | 4 | 12 | P2 |
| 8 | Phase 3 Batch | `phase3-batch` | 4 | 9 | P1 |
| 9 | Phase 3 Transaction | `phase3-transaction` | 5 | 9 | P1 |
| 10 | Phase 3 Errors | `phase3-errors` | 6 | 9 | P0 |
| 11 | Phase 3 Business | `phase3-business` | 6 | 9 | P1 |
| 12 | Phase 3 Security+ | `phase3-security-plus` | 4 | 8 | P1 |

**总计**: 12 个仪表盘，50 个指标，102 个 Panel

### 7.2 仪表盘访问链接

```
Grafana Base URL: http://grafana:3000

Dashboards:
- Overview: http://grafana:3000/d/phase3-overview
- Performance: http://grafana:3000/d/phase3-performance
- Performance+: http://grafana:3000/d/phase3-performance-plus
- Tracing: http://grafana:3000/d/phase3-tracing
- System: http://grafana:3000/d/phase3-system
- API Performance: http://grafana:3000/d/phase3-api-performance
- User Experience: http://grafana:3000/d/phase3-user-experience
- Batch: http://grafana:3000/d/phase3-batch
- Transaction: http://grafana:3000/d/phase3-transaction
- Errors: http://grafana:3000/d/phase3-errors
- Business: http://grafana:3000/d/phase3-business
- Security+: http://grafana:3000/d/phase3-security-plus
```

---

## 8. 验证标准

### 8.1 仪表盘验证

| 验证项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 仪表盘加载 | <3s | 人工测试 | 所有仪表盘<3s |
| Panel 显示 | 102 个 Panel 正常 | Grafana 检查 | 100% Panel 正常 |
| 数据刷新 | 15-30s | 观察验证 | 刷新正常 |
| 阈值标识 | 阈值线正确 | 视觉检查 | 阈值线正确 |
| 告警集成 | 50 个告警规则生效 | 模拟测试 | 告警触发正常 |

### 8.2 指标采集验证

| 验证项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 指标可查询 | 50 个指标均有数据 | Prometheus 查询 | 100% 指标可查询 |
| 数据新鲜度 | 延迟<30s | 时间戳检查 | 最新数据<30s |
| Labels 完整 | 所有 Labels 正确 | 指标检查 | 100% Labels 存在 |
| 数值准确性 | 与日志一致 | 抽样比对 | 误差<1% |

---

## 9. 部署步骤

### 9.1 仪表盘导入

```bash
# 1. 创建仪表盘目录
mkdir -p /var/lib/grafana/dashboards/phase3-batch4

# 2. 复制 Batch 4 仪表盘配置
cp phase3-performance-plus.json /var/lib/grafana/dashboards/phase3-batch4/
cp phase3-batch.json /var/lib/grafana/dashboards/phase3-batch4/
cp phase3-transaction.json /var/lib/grafana/dashboards/phase3-batch4/
cp phase3-errors.json /var/lib/grafana/dashboards/phase3-batch4/
cp phase3-business.json /var/lib/grafana/dashboards/phase3-batch4/
cp phase3-security-plus.json /var/lib/grafana/dashboards/phase3-batch4/

# 3. 重启 Grafana (或等待自动刷新)
docker-compose restart grafana

# 4. 验证仪表盘加载
curl -u admin:admin http://localhost:3000/api/search?query=phase3
```

### 9.2 告警规则更新

```bash
# 1. 复制 Batch 4 告警规则
cp alert-rules-batch4.yml /etc/prometheus/rules/

# 2. 验证 Prometheus 配置
curl -X POST http://localhost:9090/-/reload

# 3. 检查告警规则
curl 'http://localhost:9090/api/v1/rules' | jq '.data.groups[].rules[].name'
```

---

## 10. 附录

### 10.1 快速查询手册

```promql
# === Batch 4 新增指标查询 ===

# Execution Latency P50
histogram_quantile(0.50, sum(rate(execution_latency_bucket[5m])) by(le))

# Execution Latency P95
histogram_quantile(0.95, sum(rate(execution_latency_bucket[5m])) by(le))

# Executor Queue Depth
executor_queue_depth

# Verification Latency P50
histogram_quantile(0.50, sum(rate(verification_latency_bucket[5m])) by(le))

# Verification Queue Depth
verification_queue_depth

# Batch Overhead %
batch_overhead_percent

# Batch Nested Depth
batch_nested_depth_current

# Transaction Deadlock Count
increase(transaction_deadlock_count[1h])

# Execution Panic Count
increase(execution_panic_count[1h])

# Execution Timeout Count
increase(execution_timeout_count[1h])

# Verification Mismatch Count
increase(verification_mismatch_count[5m])

# Batch Partial Failure Count
increase(batch_partial_failure_count[5m])

# Transaction Abort Count
increase(transaction_abort_count[5m])

# Instruction Retry Count
increase(instruction_retry_count[1h])

# Instruction Success Rate
instruction_success_rate

# Gray Release Rollback Count
increase(gray_release_rollback_count[24h])

# OIDC Validation Latency P99
histogram_quantile(0.99, rate(oidc_token_validation_latency_p99_bucket[5m]))

# OPA Policy Evaluation Count
increase(opa_policy_evaluation_count[1h])

# Secret Rotation Success Rate
secret_rotation_success_rate
```

### 10.2 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| 首批 10 指标接入 | dashboard_v6_batch1.md | Week 2 实现参考 |
| 第二批 10 指标接入 | metrics_10_batch2_impl.md | Week 3 实现参考 |
| 第三批 10 指标接入 | dashboard_v6_batch3.md | Week 4 实现参考 |
| Phase 3 50 指标规划 | phase3_50_metrics_plan.md | 完整指标体系 |

---

**文档状态**: ✅ Week 5 完成  
**创建日期**: 2026-03-14  
**责任人**: SRE-Agent + Observability-Agent  
**保管**: 项目文档库
