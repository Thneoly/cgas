# Grafana 仪表盘 v6 首批 10 指标配置

**版本**: v6.1  
**日期**: 2026-03-07  
**责任人**: Observability-Agent + SRE-Agent  
**状态**: ✅ 已完成  
**release_id**: release-2026-03-07-phase3_week02  
**关联文档**: 
- monitoring_dashboard_v6.md (仪表盘 v6 设计)
- phase3_50_metrics_plan.md (50 指标规划)
- otel_integration.md (OpenTelemetry 集成)

---

## 1. 概述

### 1.1 首批 10 指标选择

根据 Phase 3 50 指标规划，首批配置 10 个核心指标仪表盘，覆盖最关键的可观测性需求：

| 优先级 | 指标 ID | 指标名 | 类型 | 告警阈值 | 仪表盘 |
|---|---|---|---|---|---|
| P0 | M-006 | execution_latency_p99 | Histogram | >200ms | Performance |
| P0 | M-007 | verification_latency_p99 | Histogram | >200ms | Performance |
| P0 | M-025 | distributed_trace_coverage | Gauge | <98% | Tracing |
| P0 | M-035 | trace_span_duration_p99 | Histogram | >500ms | Tracing |
| P1 | M-013 | cpu_usage_percent | Gauge | >80% | System |
| P1 | M-014 | memory_usage_percent | Gauge | >85% | System |
| P1 | M-051 | trace_total_duration_p99 | Histogram | >1000ms | Tracing |
| P1 | M-053 | trace_propagation_success_rate | Gauge | <99% | Tracing |
| P2 | M-026 | execution_latency_p50 | Histogram | >100ms | Performance |
| P2 | M-029 | verification_latency_p50 | Histogram | >100ms | Performance |

### 1.2 仪表盘列表

| 仪表盘 | UID | 指标数 | 面板数 | 用途 |
|---|---|---|---|---|
| Phase 3 Overview | `phase3-overview` | 6 | 8 | 总览 |
| Performance | `phase3-performance` | 4 | 6 | 性能监控 |
| Tracing | `phase3-tracing` | 4 | 8 | 分布式追踪 |
| System | `phase3-system` | 2 | 4 | 系统资源 |

---

## 2. Phase 3 Overview 仪表盘

**Dashboard UID**: `phase3-overview`  
**刷新频率**: 15s  
**数据源**: Prometheus, Tempo

### 2.1 完整配置

```json
{
  "dashboard": {
    "id": null,
    "uid": "phase3-overview",
    "title": "Phase 3 Overview",
    "tags": ["phase3", "overview", "cgas"],
    "timezone": "browser",
    "refresh": "15s",
    "version": 1,
    "schemaVersion": 38,
    "panels": [
      {
        "id": 1,
        "gridPos": {"h": 4, "w": 8, "x": 0, "y": 0},
        "type": "stat",
        "title": "🚦 Gate Decision",
        "targets": [
          {
            "expr": "gray_release_consistency_rate",
            "refId": "A",
            "datasource": {"type": "prometheus", "uid": "prometheus"}
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "yellow", "value": 99.9},
                {"color": "green", "value": 99.95}
              ]
            },
            "mappings": [
              {
                "type": "range",
                "options": {
                  "from": 99.95,
                  "to": 100,
                  "result": {"text": "✅ Go", "color": "green"}
                }
              },
              {
                "type": "range",
                "options": {
                  "from": 99.9,
                  "to": 99.95,
                  "result": {"text": "⚠️ Conditional", "color": "yellow"}
                }
              },
              {
                "type": "range",
                "options": {
                  "from": 0,
                  "to": 99.9,
                  "result": {"text": "❌ No-Go", "color": "red"}
                }
              }
            ]
          }
        }
      },
      {
        "id": 2,
        "gridPos": {"h": 4, "w": 8, "x": 8, "y": 0},
        "type": "gauge",
        "title": "🔍 Trace Coverage",
        "targets": [
          {
            "expr": "distributed_trace_coverage",
            "refId": "A",
            "datasource": {"type": "prometheus", "uid": "prometheus"}
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "min": 0,
            "max": 100,
            "thresholds": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "yellow", "value": 98},
                {"color": "green", "value": 99}
              ]
            }
          }
        }
      },
      {
        "id": 3,
        "gridPos": {"h": 4, "w": 8, "x": 16, "y": 0},
        "type": "stat",
        "title": "📊 Metrics Pass Rate",
        "targets": [
          {
            "expr": "sum(metric_status == 1) / count(metric_status) * 100",
            "refId": "A",
            "datasource": {"type": "prometheus", "uid": "prometheus"}
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "thresholds": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "yellow", "value": 90},
                {"color": "green", "value": 95}
              ]
            }
          }
        }
      },
      {
        "id": 4,
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 4},
        "type": "timeseries",
        "title": "⏱️ P99 Latency Trend",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(execution_latency_bucket[5m]))",
            "legendFormat": "Execution P99",
            "refId": "A",
            "datasource": {"type": "prometheus", "uid": "prometheus"}
          },
          {
            "expr": "histogram_quantile(0.99, rate(verification_latency_bucket[5m]))",
            "legendFormat": "Verification P99",
            "refId": "B",
            "datasource": {"type": "prometheus", "uid": "prometheus"}
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "ms",
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 180},
                {"color": "red", "value": 200}
              ]
            }
          }
        },
        "options": {
          "tooltip": {"mode": "multi", "sort": "desc"},
          "legend": {"displayMode": "table", "placement": "right"}
        }
      },
      {
        "id": 5,
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 4},
        "type": "timeseries",
        "title": "📈 Throughput (RPS)",
        "targets": [
          {
            "expr": "rate(execution_total[1m])",
            "legendFormat": "Requests/sec",
            "refId": "A",
            "datasource": {"type": "prometheus", "uid": "prometheus"}
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "reqps",
            "thresholds": {
              "steps": [
                {"color": "yellow", "value": 180},
                {"color": "green", "value": 200}
              ]
            }
          }
        }
      },
      {
        "id": 6,
        "gridPos": {"h": 8, "w": 8, "x": 0, "y": 12},
        "type": "timeseries",
        "title": "⚠️ Error Rate",
        "targets": [
          {
            "expr": "rate(execution_errors_total[1m]) / rate(execution_total[1m]) * 100",
            "legendFormat": "Error Rate %",
            "refId": "A",
            "datasource": {"type": "prometheus", "uid": "prometheus"}
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 0.1},
                {"color": "red", "value": 1}
              ]
            }
          }
        }
      },
      {
        "id": 7,
        "gridPos": {"h": 8, "w": 8, "x": 8, "y": 12},
        "type": "stat",
        "title": "🏃 Performance Metrics",
        "targets": [
          {
            "expr": "sum(performance_metric_status == 1)",
            "legendFormat": "Pass",
            "refId": "A",
            "datasource": {"type": "prometheus", "uid": "prometheus"}
          },
          {
            "expr": "sum(performance_metric_status == 0)",
            "legendFormat": "Fail",
            "refId": "B",
            "datasource": {"type": "prometheus", "uid": "prometheus"}
          }
        ],
        "options": {
          "colorMode": "background",
          "graphMode": "none",
          "justifyMode": "center"
        }
      },
      {
        "id": 8,
        "gridPos": {"h": 8, "w": 8, "x": 16, "y": 12},
        "type": "stat",
        "title": "🔍 Tracing Metrics",
        "targets": [
          {
            "expr": "sum(tracing_metric_status == 1)",
            "legendFormat": "OK",
            "refId": "A",
            "datasource": {"type": "prometheus", "uid": "prometheus"}
          }
        ],
        "options": {
          "colorMode": "background",
          "graphMode": "none"
        }
      }
    ],
    "templating": {
      "list": [
        {
          "name": "environment",
          "type": "query",
          "datasource": {"type": "prometheus", "uid": "prometheus"},
          "query": "label_values(deployment_environment)",
          "refresh": 1,
          "current": {"text": "production", "value": "production"}
        }
      ]
    }
  }
}
```

---

## 3. Performance 仪表盘

**Dashboard UID**: `phase3-performance`  
**刷新频率**: 15s  
**数据源**: Prometheus

### 3.1 完整配置

```json
{
  "dashboard": {
    "id": null,
    "uid": "phase3-performance",
    "title": "Phase 3 Performance",
    "tags": ["phase3", "performance", "cgas"],
    "timezone": "browser",
    "refresh": "15s",
    "version": 1,
    "schemaVersion": 38,
    "panels": [
      {
        "id": 1,
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "type": "timeseries",
        "title": "⏱️ Execution Latency Distribution",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(execution_latency_bucket[5m]))",
            "legendFormat": "P99",
            "refId": "A"
          },
          {
            "expr": "histogram_quantile(0.95, rate(execution_latency_bucket[5m]))",
            "legendFormat": "P95",
            "refId": "B"
          },
          {
            "expr": "histogram_quantile(0.50, rate(execution_latency_bucket[5m]))",
            "legendFormat": "P50",
            "refId": "C"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "ms",
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 180},
                {"color": "red", "value": 200}
              ]
            }
          }
        }
      },
      {
        "id": 2,
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0},
        "type": "timeseries",
        "title": "📊 Executor Queue Depth",
        "targets": [
          {
            "expr": "executor_queue_depth",
            "legendFormat": "Queue Depth",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 80},
                {"color": "red", "value": 100}
              ]
            }
          }
        }
      },
      {
        "id": 3,
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8},
        "type": "timeseries",
        "title": "✅ Verification Latency Distribution",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(verification_latency_bucket[5m]))",
            "legendFormat": "P99",
            "refId": "A"
          },
          {
            "expr": "histogram_quantile(0.95, rate(verification_latency_bucket[5m]))",
            "legendFormat": "P95",
            "refId": "B"
          },
          {
            "expr": "histogram_quantile(0.50, rate(verification_latency_bucket[5m]))",
            "legendFormat": "P50",
            "refId": "C"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "ms",
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 180},
                {"color": "red", "value": 200}
              ]
            }
          }
        }
      },
      {
        "id": 4,
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8},
        "type": "timeseries",
        "title": "📊 Verification Queue Depth",
        "targets": [
          {
            "expr": "verification_queue_depth",
            "legendFormat": "Queue Depth",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 80},
                {"color": "red", "value": 100}
              ]
            }
          }
        }
      },
      {
        "id": 5,
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 16},
        "type": "gauge",
        "title": "📦 Batch Overhead",
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
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 15},
                {"color": "red", "value": 20}
              ]
            }
          }
        }
      },
      {
        "id": 6,
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 16},
        "type": "bargauge",
        "title": "📦 Batch Nesting Depth",
        "targets": [
          {
            "expr": "batch_nested_depth_current",
            "legendFormat": "Current Depth",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 3},
                {"color": "red", "value": 5}
              ]
            }
          }
        }
      }
    ]
  }
}
```

---

## 4. Tracing 仪表盘

**Dashboard UID**: `phase3-tracing`  
**刷新频率**: 15s  
**数据源**: Prometheus, Tempo

### 4.1 完整配置

```json
{
  "dashboard": {
    "id": null,
    "uid": "phase3-tracing",
    "title": "Phase 3 Distributed Tracing",
    "tags": ["phase3", "tracing", "opentelemetry", "cgas"],
    "timezone": "browser",
    "refresh": "15s",
    "version": 1,
    "schemaVersion": 38,
    "panels": [
      {
        "id": 1,
        "gridPos": {"h": 6, "w": 8, "x": 0, "y": 0},
        "type": "gauge",
        "title": "🔍 Trace Coverage Rate",
        "targets": [
          {
            "expr": "distributed_trace_coverage",
            "legendFormat": "Coverage %",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "min": 0,
            "max": 100,
            "thresholds": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "yellow", "value": 98},
                {"color": "green", "value": 99}
              ]
            }
          }
        }
      },
      {
        "id": 2,
        "gridPos": {"h": 6, "w": 8, "x": 8, "y": 0},
        "type": "gauge",
        "title": "🔍 Trace Propagation Success Rate",
        "targets": [
          {
            "expr": "trace_propagation_success_rate",
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
              "steps": [
                {"color": "red", "value": 0},
                {"color": "yellow", "value": 98},
                {"color": "green", "value": 99}
              ]
            }
          }
        }
      },
      {
        "id": 3,
        "gridPos": {"h": 6, "w": 8, "x": 16, "y": 0},
        "type": "stat",
        "title": "🔍 Average Spans per Trace",
        "targets": [
          {
            "expr": "trace_span_count_avg",
            "legendFormat": "Span Count",
            "refId": "A"
          }
        ]
      },
      {
        "id": 4,
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 6},
        "type": "timeseries",
        "title": "⏱️ Span Duration P99",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(trace_span_duration_p99_bucket[5m]))",
            "legendFormat": "Span P99",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "ms",
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 400},
                {"color": "red", "value": 500}
              ]
            }
          }
        }
      },
      {
        "id": 5,
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 6},
        "type": "timeseries",
        "title": "⏱️ Total Trace Duration P99",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(trace_total_duration_p99_bucket[5m]))",
            "legendFormat": "Trace P99",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "ms",
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 800},
                {"color": "red", "value": 1000}
              ]
            }
          }
        }
      },
      {
        "id": 6,
        "gridPos": {"h": 8, "w": 24, "x": 0, "y": 14},
        "type": "table",
        "title": "🛤️ Critical Paths Coverage",
        "targets": [
          {
            "expr": "critical_path_coverage",
            "format": "table",
            "instant": true,
            "refId": "A"
          }
        ],
        "transformations": [
          {
            "id": "organize",
            "options": {
              "excludeByName": ["Time", "__name__"],
              "renameByName": {
                "path": "Path",
                "Value": "Status"
              }
            }
          },
          {
            "id": "mappings",
            "options": {
              "mappings": [
                {
                  "type": "value",
                  "options": {
                    "1": {"text": "✅", "color": "green"},
                    "0": {"text": "❌", "color": "red"}
                  }
                }
              ]
            }
          }
        ]
      },
      {
        "id": 7,
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 22},
        "type": "table",
        "title": "🔍 Recent Traces",
        "targets": [
          {
            "queryType": "search",
            "refId": "A",
            "datasource": {"type": "tempo", "uid": "tempo"}
          }
        ],
        "transformations": [
          {
            "id": "limit",
            "options": {"limit": 10}
          }
        ],
        "links": [
          {
            "title": "View in Tempo",
            "url": "http://tempo:3200/trace/${__data.fields.traceID}",
            "targetBlank": true
          }
        ]
      },
      {
        "id": 8,
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 22},
        "type": "stat",
        "title": "📊 Trace Volume (24h)",
        "targets": [
          {
            "expr": "sum(increase(trace_total_duration_p99_count[24h]))",
            "legendFormat": "Total Traces",
            "refId": "A"
          }
        ]
      }
    ],
    "templating": {
      "list": [
        {
          "name": "service",
          "type": "query",
          "datasource": {"type": "prometheus", "uid": "prometheus"},
          "query": "label_values(service_name)",
          "refresh": 1
        }
      ]
    }
  }
}
```

---

## 5. System 仪表盘

**Dashboard UID**: `phase3-system`  
**刷新频率**: 30s  
**数据源**: Prometheus

### 5.1 完整配置

```json
{
  "dashboard": {
    "id": null,
    "uid": "phase3-system",
    "title": "Phase 3 System Resources",
    "tags": ["phase3", "system", "resources", "cgas"],
    "timezone": "browser",
    "refresh": "30s",
    "version": 1,
    "schemaVersion": 38,
    "panels": [
      {
        "id": 1,
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "type": "gauge",
        "title": "💻 CPU Usage",
        "targets": [
          {
            "expr": "cpu_usage_percent",
            "legendFormat": "CPU %",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "min": 0,
            "max": 100,
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 60},
                {"color": "red", "value": 80}
              ]
            }
          }
        }
      },
      {
        "id": 2,
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0},
        "type": "gauge",
        "title": "💾 Memory Usage",
        "targets": [
          {
            "expr": "memory_usage_percent",
            "legendFormat": "Memory %",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "min": 0,
            "max": 100,
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 70},
                {"color": "red", "value": 85}
              ]
            }
          }
        }
      },
      {
        "id": 3,
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8},
        "type": "gauge",
        "title": "💽 Disk IO Wait",
        "targets": [
          {
            "expr": "disk_io_wait_percent",
            "legendFormat": "IO Wait %",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 20},
                {"color": "red", "value": 30}
              ]
            }
          }
        }
      },
      {
        "id": 4,
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8},
        "type": "gauge",
        "title": "🌐 Network Packet Drop",
        "targets": [
          {
            "expr": "network_packet_drop_rate",
            "legendFormat": "Drop Rate %",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "thresholds": {
              "steps": [
                {"color": "green", "value": 0},
                {"color": "yellow", "value": 0.5},
                {"color": "red", "value": 1}
              ]
            }
          }
        }
      }
    ]
  }
}
```

---

## 6. 数据源配置

### 6.1 Prometheus 数据源

```yaml
# grafana-datasources.yaml

apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
    uid: prometheus
  
  - name: Tempo
    type: tempo
    access: proxy
    url: http://tempo:3200
    editable: false
    uid: tempo
    jsonData:
      httpMethod: GET
      tracesToLogs:
        datasourceUid: loki
        tags: ['trace_id']
      tracesToMetrics:
        datasourceUid: prometheus
        tags: [{ key: 'service.name', value: 'service' }]
```

### 6.2 仪表盘自动导入

```yaml
# grafana-dashboards-config.yaml

apiVersion: 1

providers:
  - name: 'Phase 3 Dashboards'
    orgId: 1
    folder: 'Phase 3'
    folderUid: 'phase3'
    type: file
    disableDeletion: false
    updateIntervalSeconds: 30
    allowUiUpdates: true
    options:
      path: /var/lib/grafana/dashboards/phase3
```

---

## 7. 告警规则

### 7.1 Prometheus 告警规则

```yaml
# prometheus-alerts-phase3.yaml

groups:
  - name: Phase3_Performance
    interval: 1m
    rules:
      - alert: ExecutionP99High
        expr: histogram_quantile(0.99, rate(execution_latency_bucket[5m])) > 200
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "执行 P99 时延过高"
          description: "当前值：{{ $value }}ms"
      
      - alert: VerificationP99High
        expr: histogram_quantile(0.99, rate(verification_latency_bucket[5m])) > 200
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "验证 P99 时延过高"
          description: "当前值：{{ $value }}ms"
      
      - alert: TraceCoverageLow
        expr: distributed_trace_coverage < 98
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Trace 覆盖率过低"
          description: "当前值：{{ $value }}%"
      
      - alert: TracePropagationLow
        expr: trace_propagation_success_rate < 99
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Trace 传递成功率过低"
          description: "当前值：{{ $value }}%"
      
      - alert: CPUHigh
        expr: cpu_usage_percent > 80
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "CPU 使用率过高"
          description: "当前值：{{ $value }}%"
      
      - alert: MemoryHigh
        expr: memory_usage_percent > 85
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "内存使用率过高"
          description: "当前值：{{ $value }}%"
```

---

## 8. 部署与验证

### 8.1 部署步骤

```bash
# 1. 创建仪表盘目录
mkdir -p /var/lib/grafana/dashboards/phase3

# 2. 复制仪表盘配置文件
cp phase3-overview.json /var/lib/grafana/dashboards/phase3/
cp phase3-performance.json /var/lib/grafana/dashboards/phase3/
cp phase3-tracing.json /var/lib/grafana/dashboards/phase3/
cp phase3-system.json /var/lib/grafana/dashboards/phase3/

# 3. 复制数据源配置
cp grafana-datasources.yaml /etc/grafana/provisioning/datasources/
cp grafana-dashboards-config.yaml /etc/grafana/provisioning/dashboards/

# 4. 复制告警规则
cp prometheus-alerts-phase3.yaml /etc/prometheus/rules/

# 5. 重启服务
docker-compose restart grafana prometheus
```

### 8.2 验证命令

```bash
# 检查仪表盘是否加载
curl -u admin:admin http://localhost:3000/api/search?query=phase3

# 检查数据源
curl -u admin:admin http://localhost:3000/api/datasources

# 查询 Prometheus 指标
curl 'http://localhost:9090/api/v1/query?query=distributed_trace_coverage'

# 检查告警规则
curl 'http://localhost:9090/api/v1/rules'
```

---

## 9. 实施计划

| 任务 | 责任人 | 状态 | 交付物 |
|---|---|---|---|
| 仪表盘设计 | Observability | ✅ 完成 | dashboard_v6_batch1.md |
| Overview 配置 | SRE | ✅ 完成 | phase3-overview.json |
| Performance 配置 | SRE | ✅ 完成 | phase3-performance.json |
| Tracing 配置 | Observability | ✅ 完成 | phase3-tracing.json |
| System 配置 | SRE | ✅ 完成 | phase3-system.json |
| 告警规则配置 | SRE | ✅ 完成 | prometheus-alerts-phase3.yaml |
| 数据源配置 | SRE | ✅ 完成 | grafana-datasources.yaml |

---

**文档状态**: ✅ 已完成  
**创建日期**: 2026-03-07  
**责任人**: Observability-Agent + SRE-Agent  
**保管**: 项目文档库
