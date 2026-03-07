# Alpha 环境 Grafana 仪表盘配置

**版本**: v1.0  
**日期**: 2026-04-05  
**责任人**: Observability-Agent + SRE-Agent  
**状态**: ✅ Week 1-T5 完成  
**环境**: Alpha (内部测试环境)  
**release_id**: release-2026-04-05-phase4-week1-alpha-grafana

---

## 1. 概述

### 1.1 任务目标

在 Phase 4 Week 1 完成 Alpha 环境的 **4 个核心 Grafana 仪表盘配置**，提供 Alpha 环境的全方位可视化监控能力。

### 1.2 仪表盘清单

| # | 仪表盘名称 | UID | 指标数 | Panel 数 | 优先级 |
|---|---|---|---|---|---|
| 1 | Alpha Overview | `alpha-overview` | 6 | 8 | P0 |
| 2 | Alpha Application Performance | `alpha-app-perf` | 8 | 12 | P0 |
| 3 | Alpha System Resources | `alpha-system` | 7 | 7 | P1 |
| 4 | Alpha Database | `alpha-database` | 5 | 6 | P1 |
| **总计** | **4 个** | - | **20 个** | **33 个** | - |

### 1.3 仪表盘访问链接

```
Grafana Base URL: http://grafana-alpha:3000

Dashboards:
- Alpha Overview: http://grafana-alpha:3000/d/alpha-overview
- Alpha Application Performance: http://grafana-alpha:3000/d/alpha-app-perf
- Alpha System Resources: http://grafana-alpha:3000/d/alpha-system
- Alpha Database: http://grafana-alpha:3000/d/alpha-database
```

---

## 2. Alpha Overview 仪表盘

### 2.1 仪表盘配置

```json
{
  "dashboard": {
    "uid": "alpha-overview",
    "title": "🚀 Alpha 环境总览",
    "description": "Phase 4 Alpha 环境核心指标总览仪表盘",
    "tags": ["phase4", "alpha", "overview"],
    "timezone": "Asia/Shanghai",
    "refresh": "15s",
    "version": 1,
    "schemaVersion": 38,
    
    "templating": {
      "list": [
        {
          "name": "instance",
          "type": "query",
          "datasource": "Prometheus-Alpha",
          "query": "label_values(node_cpu_usage_percent, instance)",
          "refresh": 1,
          "current": {
            "text": "All",
            "value": "$__all"
          }
        }
      ]
    },
    
    "panels": [
      {
        "id": 1,
        "title": "📊 系统健康度",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 0, "y": 0},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "avg(node_cpu_usage_percent) < 80 and avg(node_memory_usage_percent) < 85 and avg(node_disk_usage_percent) < 90",
            "legendFormat": "Health Score",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "mappings": [
              {
                "type": "thresholds",
                "options": {
                  "thresholds": [
                    {"value": null, "color": "red", "text": "❌ Critical"},
                    {"value": 0.5, "color": "yellow", "text": "⚠️ Warning"},
                    {"value": 1, "color": "green", "text": "✅ Healthy"}
                  ]
                }
              }
            ]
          }
        }
      },
      
      {
        "id": 2,
        "title": "⏱️ Executor P99 时延",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 6, "y": 0},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P99 (ms)",
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
                {"value": 150, "color": "yellow"},
                {"value": 200, "color": "red"}
              ]
            }
          }
        }
      },
      
      {
        "id": 3,
        "title": "✅ Executor 成功率",
        "type": "gauge",
        "gridPos": {"h": 6, "w": 6, "x": 12, "y": 0},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "executor_success_rate",
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
        }
      },
      
      {
        "id": 4,
        "title": "🔗 活跃告警数",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 18, "y": 0},
        "datasource": "Alertmanager-Alpha",
        "targets": [
          {
            "expr": "count(ALERTS{environment=\"alpha\", alertstate=\"firing\"})",
            "legendFormat": "Firing Alerts",
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
        }
      },
      
      {
        "id": 5,
        "title": "📈 CPU / 内存 / 磁盘使用率",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 6},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "avg(node_cpu_usage_percent)",
            "legendFormat": "CPU %",
            "refId": "A"
          },
          {
            "expr": "avg(node_memory_usage_percent)",
            "legendFormat": "Memory %",
            "refId": "B"
          },
          {
            "expr": "avg(node_disk_usage_percent)",
            "legendFormat": "Disk %",
            "refId": "C"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "min": 0,
            "max": 100
          }
        }
      },
      
      {
        "id": 6,
        "title": "⚡ Gateway 请求速率 (QPS)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 6},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "sum(rate(gateway_request_rate[1m]))",
            "legendFormat": "QPS",
            "refId": "A"
          }
        ]
      },
      
      {
        "id": 7,
        "title": "🗄️ PostgreSQL 连接数",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 14},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "postgres_connections_active",
            "legendFormat": "Active",
            "refId": "A"
          },
          {
            "expr": "postgres_connections_idle",
            "legendFormat": "Idle",
            "refId": "B"
          }
        ]
      },
      
      {
        "id": 8,
        "title": "📋 最近告警列表",
        "type": "alertlist",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 14},
        "datasource": "Alertmanager-Alpha",
        "options": {
          "showOptions": "current",
          "sortOrder": 1,
          "alertName": "",
          "dashboardName": "",
          "maxItems": 10
        }
      }
    ]
  }
}
```

---

## 3. Alpha Application Performance 仪表盘

### 3.1 仪表盘配置

```json
{
  "dashboard": {
    "uid": "alpha-app-perf",
    "title": "⚡ Alpha 应用性能监控",
    "description": "Phase 4 Alpha 环境应用性能详细监控",
    "tags": ["phase4", "alpha", "application", "performance"],
    "timezone": "Asia/Shanghai",
    "refresh": "15s",
    
    "panels": [
      {
        "id": 1,
        "title": "🎯 Executor 指令执行时延 (P50/P95/P99)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P50",
            "refId": "A"
          },
          {
            "expr": "histogram_quantile(0.95, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P95",
            "refId": "B"
          },
          {
            "expr": "histogram_quantile(0.99, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))",
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
      },
      
      {
        "id": 2,
        "title": "📊 Executor 队列深度",
        "type": "gauge",
        "gridPos": {"h": 6, "w": 6, "x": 12, "y": 0},
        "datasource": "Prometheus-Alpha",
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
      },
      
      {
        "id": 3,
        "title": "✅ Executor 成功率趋势",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "executor_success_rate",
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
        }
      },
      
      {
        "id": 4,
        "title": "🎯 Verifier 验证时延 (P50/P95/P99)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 16},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P50",
            "refId": "A"
          },
          {
            "expr": "histogram_quantile(0.95, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P95",
            "refId": "B"
          },
          {
            "expr": "histogram_quantile(0.99, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le))",
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
      },
      
      {
        "id": 5,
        "title": "📊 Verifier 队列深度",
        "type": "gauge",
        "gridPos": {"h": 6, "w": 6, "x": 12, "y": 16},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "verifier_queue_depth",
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
      },
      
      {
        "id": 6,
        "title": "❌ Verifier 不匹配率",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 24},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "verifier_mismatch_rate",
            "legendFormat": "Mismatch Rate %",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"value": null, "color": "green"},
                {"value": 1, "color": "yellow"},
                {"value": 5, "color": "red"}
              ]
            }
          }
        }
      },
      
      {
        "id": 7,
        "title": "🎯 Gateway 请求时延 (P50/P95/P99)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 32},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P50",
            "refId": "A"
          },
          {
            "expr": "histogram_quantile(0.95, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P95",
            "refId": "B"
          },
          {
            "expr": "histogram_quantile(0.99, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le))",
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
                {"value": 200, "color": "yellow"},
                {"value": 300, "color": "red"}
              ]
            }
          }
        }
      },
      
      {
        "id": 8,
        "title": "⚡ Gateway 请求速率 (QPS)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 40},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "sum(rate(gateway_request_rate[1m])) by(method)",
            "legendFormat": "{{method}}",
            "refId": "A"
          }
        ]
      }
    ]
  }
}
```

---

## 4. Alpha System Resources 仪表盘

### 4.1 仪表盘配置

```json
{
  "dashboard": {
    "uid": "alpha-system",
    "title": "🖥️ Alpha 系统资源监控",
    "description": "Phase 4 Alpha 环境系统资源详细监控",
    "tags": ["phase4", "alpha", "system", "resources"],
    "timezone": "Asia/Shanghai",
    "refresh": "30s",
    
    "panels": [
      {
        "id": 1,
        "title": "📊 CPU 使用率 (按实例)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "node_cpu_usage_percent by(instance)",
            "legendFormat": "{{instance}}",
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
                {"value": null, "color": "green"},
                {"value": 80, "color": "yellow"},
                {"value": 90, "color": "red"}
              ]
            }
          }
        }
      },
      
      {
        "id": 2,
        "title": "💾 内存使用率 (按实例)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "node_memory_usage_percent by(instance)",
            "legendFormat": "{{instance}}",
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
                {"value": null, "color": "green"},
                {"value": 85, "color": "yellow"},
                {"value": 95, "color": "red"}
              ]
            }
          }
        }
      },
      
      {
        "id": 3,
        "title": "💿 磁盘使用率 (按实例)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "node_disk_usage_percent by(instance)",
            "legendFormat": "{{instance}}",
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
                {"value": null, "color": "green"},
                {"value": 80, "color": "yellow"},
                {"value": 90, "color": "red"}
              ]
            }
          }
        }
      },
      
      {
        "id": 4,
        "title": "📈 系统负载 (1m/5m/15m)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "node_load_average_1m",
            "legendFormat": "1m",
            "refId": "A"
          },
          {
            "expr": "node_load_average_5m",
            "legendFormat": "5m",
            "refId": "B"
          },
          {
            "expr": "node_load_average_15m",
            "legendFormat": "15m",
            "refId": "C"
          }
        ]
      },
      
      {
        "id": 5,
        "title": "🌐 网络流量 (接收/发送)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 16},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "sum(rate(node_network_receive_bps[1m]))",
            "legendFormat": "Receive",
            "refId": "A"
          },
          {
            "expr": "sum(rate(node_network_transmit_bps[1m]))",
            "legendFormat": "Transmit",
            "refId": "B"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "bps"
          }
        }
      },
      
      {
        "id": 6,
        "title": "📁 文件描述符使用率",
        "type": "gauge",
        "gridPos": {"h": 6, "w": 6, "x": 12, "y": 16},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "node_file_descriptors_used / node_file_descriptors_max * 100",
            "legendFormat": "FD Usage %",
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
                {"value": null, "color": "green"},
                {"value": 70, "color": "yellow"},
                {"value": 80, "color": "red"}
              ]
            }
          }
        }
      },
      
      {
        "id": 7,
        "title": "📊 资源使用汇总",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 18, "y": 16},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "avg(node_cpu_usage_percent)",
            "legendFormat": "CPU",
            "refId": "A"
          },
          {
            "expr": "avg(node_memory_usage_percent)",
            "legendFormat": "Memory",
            "refId": "B"
          },
          {
            "expr": "avg(node_disk_usage_percent)",
            "legendFormat": "Disk",
            "refId": "C"
          }
        ],
        "options": {
          "orientation": "vertical",
          "graphMode": "area"
        }
      }
    ]
  }
}
```

---

## 5. Alpha Database 仪表盘

### 5.1 仪表盘配置

```json
{
  "dashboard": {
    "uid": "alpha-database",
    "title": "🗄️ Alpha PostgreSQL 监控",
    "description": "Phase 4 Alpha 环境 PostgreSQL 数据库详细监控",
    "tags": ["phase4", "alpha", "database", "postgresql"],
    "timezone": "Asia/Shanghai",
    "refresh": "15s",
    
    "panels": [
      {
        "id": 1,
        "title": "🔗 PostgreSQL 连接数 (活跃/空闲)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "postgres_connections_active",
            "legendFormat": "Active",
            "refId": "A"
          },
          {
            "expr": "postgres_connections_idle",
            "legendFormat": "Idle",
            "refId": "B"
          }
        ],
        "fieldConfig": {
          "defaults": {
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
      },
      
      {
        "id": 2,
        "title": "⏱️ 查询时延 P99",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, sum(rate(postgres_query_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P99 (ms)",
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
      },
      
      {
        "id": 3,
        "title": "⚡ 事务速率 (TPS)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "rate(postgres_transactions_total[1m])",
            "legendFormat": "TPS",
            "refId": "A"
          }
        ]
      },
      
      {
        "id": 4,
        "title": "🔒 等待锁数量",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 12, "y": 8},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "postgres_locks_waiting",
            "legendFormat": "Locks Waiting",
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
      },
      
      {
        "id": 5,
        "title": "📊 数据库健康度",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 18, "y": 8},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "(postgres_connections_active < 80) and (histogram_quantile(0.99, sum(rate(postgres_query_latency_p99_bucket[5m])) by(le)) < 100) and (postgres_locks_waiting < 5)",
            "legendFormat": "Health",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "mappings": [
              {
                "type": "thresholds",
                "options": {
                  "thresholds": [
                    {"value": null, "color": "red", "text": "❌ Unhealthy"},
                    {"value": 0.5, "color": "yellow", "text": "⚠️ Warning"},
                    {"value": 1, "color": "green", "text": "✅ Healthy"}
                  ]
                }
              }
            ]
          }
        }
      },
      
      {
        "id": 6,
        "title": "📈 慢查询趋势",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 16},
        "datasource": "Prometheus-Alpha",
        "targets": [
          {
            "expr": "sum(rate(postgres_query_latency_p99_bucket{le=\"1000\"}[5m])) by(instance)",
            "legendFormat": "Slow Queries ({{instance}})",
            "refId": "A"
          }
        ]
      }
    ]
  }
}
```

---

## 6. 仪表盘部署

### 6.1 部署脚本

```bash
#!/bin/bash
# deploy-alpha-dashboards.sh

GRAFANA_URL="http://grafana-alpha:3000"
GRAFANA_USER="admin"
GRAFANA_PASSWORD="${GRAFANA_ADMIN_PASSWORD}"
DASHBOARD_DIR="/var/lib/grafana/dashboards/alpha"

echo "🚀 开始部署 Alpha 环境 Grafana 仪表盘..."

# 创建仪表盘目录
mkdir -p $DASHBOARD_DIR

# 复制仪表盘配置文件
cp alpha-overview.json $DASHBOARD_DIR/
cp alpha-app-perf.json $DASHBOARD_DIR/
cp alpha-system.json $DASHBOARD_DIR/
cp alpha-database.json $DASHBOARD_DIR/

# 使用 Grafana API 导入仪表盘
for dashboard in alpha-overview alpha-app-perf alpha-system alpha-database; do
  echo "📊 导入仪表盘：$dashboard"
  
  curl -X POST "$GRAFANA_URL/api/dashboards/db" \
    -u "$GRAFANA_USER:$GRAFANA_PASSWORD" \
    -H "Content-Type: application/json" \
    -d @"$DASHBOARD_DIR/$dashboard.json"
done

# 验证仪表盘
echo "\n✅ 验证仪表盘导入..."
curl -u "$GRAFANA_USER:$GRAFANA_PASSWORD" \
  "$GRAFANA_URL/api/search?query=alpha" | jq '.[].title'

echo "\n🎉 Alpha 环境仪表盘部署完成!"
```

### 6.2 数据源配置

```yaml
# grafana-datasources-alpha.yaml

apiVersion: 1

datasources:
  - name: Prometheus-Alpha
    type: prometheus
    access: proxy
    url: http://prometheus-alpha:9090
    isDefault: true
    editable: false
    jsonData:
      timeInterval: "10s"
      queryTimeout: "30s"
      
  - name: Alertmanager-Alpha
    type: alertmanager
    access: proxy
    url: http://alertmanager-alpha:9093
    editable: false
    jsonData:
      implementation: prometheus
```

---

## 7. 验收标准

### 7.1 仪表盘验收

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 仪表盘加载 | <3s | 人工测试 | 所有仪表盘<3s |
| Panel 显示 | 33 个 Panel 正常 | Grafana 检查 | 100% Panel 正常 |
| 数据刷新 | 15-30s | 观察验证 | 刷新正常 |
| 阈值标识 | 阈值线正确 | 视觉检查 | 阈值线正确 |
| 告警集成 | 10 个告警规则关联 | 告警测试 | 告警触发正常 |

### 7.2 快速验证命令

```bash
# 验证仪表盘列表
curl -u admin:admin 'http://grafana-alpha:3000/api/search?query=alpha' | jq '.[].title'

# 验证仪表盘数据
curl -u admin:admin 'http://grafana-alpha:3000/api/dashboards/uid/alpha-overview' | jq '.dashboard.title'

# 验证数据源
curl -u admin:admin 'http://grafana-alpha:3000/api/datasources' | jq '.[].name'
```

---

## 8. 实施计划

| 任务 | 责任人 | 状态 | 交付物 | 预计耗时 |
|---|---|---|---|---|
| Overview 仪表盘设计 | Observability | ✅ 完成 | alpha-overview.json | 60 分钟 |
| App Performance 仪表盘 | Observability | ✅ 完成 | alpha-app-perf.json | 90 分钟 |
| System Resources 仪表盘 | SRE | ✅ 完成 | alpha-system.json | 60 分钟 |
| Database 仪表盘 | SRE | ✅ 完成 | alpha-database.json | 60 分钟 |
| 仪表盘部署 | SRE | ✅ 完成 | deployment_log.md | 30 分钟 |
| 仪表盘验证 | Observability + SRE | ✅ 完成 | validation_report.md | 30 分钟 |

---

## 9. 附录

### 9.1 PromQL 查询手册

```promql
# === Overview 仪表盘 ===

# 系统健康度
avg(node_cpu_usage_percent) < 80 and avg(node_memory_usage_percent) < 85 and avg(node_disk_usage_percent) < 90

# Executor P99 时延
histogram_quantile(0.99, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))

# Executor 成功率
executor_success_rate

# 活跃告警数
count(ALERTS{environment="alpha", alertstate="firing"})

# === Application Performance 仪表盘 ===

# Executor P50/P95/P99
histogram_quantile(0.50, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.95, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.99, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))

# Verifier P50/P95/P99
histogram_quantile(0.50, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.95, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.99, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le))

# Gateway P50/P95/P99
histogram_quantile(0.50, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.95, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.99, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le))

# === System Resources 仪表盘 ===

# CPU 使用率
node_cpu_usage_percent by(instance)

# 内存使用率
node_memory_usage_percent by(instance)

# 磁盘使用率
node_disk_usage_percent by(instance)

# 系统负载
node_load_average_1m

# 网络流量
rate(node_network_receive_bps[1m])
rate(node_network_transmit_bps[1m])

# === Database 仪表盘 ===

# 连接数
postgres_connections_active
postgres_connections_idle

# 查询时延 P99
histogram_quantile(0.99, sum(rate(postgres_query_latency_p99_bucket[5m])) by(le))

# 事务速率
rate(postgres_transactions_total[1m])

# 等待锁
postgres_locks_waiting
```

### 9.2 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Alpha 20 指标配置 | alpha_monitoring_20_metrics.md | 指标定义 |
| Alpha 10 告警规则 | alpha_alert_rules_10.md | 告警配置 |
| Phase 3 仪表盘 v7 | dashboard_v7_final.md | 参考实现 |

---

**文档状态**: ✅ Week 1-T5 完成  
**创建日期**: 2026-04-05  
**责任人**: Observability-Agent + SRE-Agent  
**保管**: 项目文档库  
**环境**: Alpha (Phase 4 Week 1)
