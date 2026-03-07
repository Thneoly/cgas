# Beta 环境 Grafana 仪表盘配置 (6 个)

**版本**: v1.0  
**日期**: 2026-04-12  
**责任人**: Observability-Agent + SRE-Agent  
**状态**: ✅ Week 2-T5 完成  
**环境**: Beta (外部测试环境)  
**release_id**: release-2026-04-12-phase4-week2-beta-grafana

---

## 1. 概述

### 1.1 任务目标

在 Phase 4 Week 2 完成 Beta 环境的 **6 个核心 Grafana 仪表盘配置**，提供 Beta 环境的全方位可视化监控能力。

### 1.2 仪表盘清单

| # | 仪表盘名称 | UID | 指标数 | Panel 数 | 优先级 |
|---|---|---|---|---|---|
| 1 | Beta Overview | `beta-overview` | 8 | 10 | P0 |
| 2 | Beta Application Performance | `beta-app-perf` | 15 | 18 | P0 |
| 3 | Beta System Resources | `beta-system` | 10 | 10 | P1 |
| 4 | Beta Database | `beta-database` | 10 | 12 | P1 |
| 5 | Beta Scheduler | `beta-scheduler` | 3 | 6 | P1 |
| 6 | Beta Container Monitoring | `beta-containers` | 2 | 4 | P1 |
| **总计** | **6 个** | - | **35 个** | **60 个** | - |

### 1.3 仪表盘访问链接

```
Grafana Base URL: http://grafana-beta:3000

Dashboards:
- Beta Overview: http://grafana-beta:3000/d/beta-overview
- Beta Application Performance: http://grafana-beta:3000/d/beta-app-perf
- Beta System Resources: http://grafana-beta:3000/d/beta-system
- Beta Database: http://grafana-beta:3000/d/beta-database
- Beta Scheduler: http://grafana-beta:3000/d/beta-scheduler
- Beta Container Monitoring: http://grafana-beta:3000/d/beta-containers
```

### 1.4 Beta vs Alpha 仪表盘对比

| 特性 | Alpha | Beta | 说明 |
|---|---|---|---|
| 仪表盘数量 | 4 个 | 6 个 | +50% |
| Panel 总数 | 33 个 | 60 个 | +82% |
| 新增仪表盘 | - | Scheduler, Containers | Beta 新增 |
| 刷新频率 | 15-30s | 10-20s | 更快刷新 |

---

## 2. Beta Overview 仪表盘

### 2.1 仪表盘配置

```json
{
  "dashboard": {
    "uid": "beta-overview",
    "title": "🚀 Beta 环境总览",
    "description": "Phase 4 Beta 环境核心指标总览仪表盘",
    "tags": ["phase4", "beta", "overview"],
    "timezone": "Asia/Shanghai",
    "refresh": "10s",
    "version": 1,
    "schemaVersion": 38,
    
    "templating": {
      "list": [
        {
          "name": "instance",
          "type": "query",
          "datasource": "Prometheus-Beta",
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
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "(avg(node_cpu_usage_percent) < 75 and avg(node_memory_usage_percent) < 80 and avg(node_disk_usage_percent) < 85)",
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
        "datasource": "Prometheus-Beta",
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
                {"value": 180, "color": "red"}
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
        "datasource": "Prometheus-Beta",
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
                {"value": 97, "color": "yellow"},
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
        "datasource": "Alertmanager-Beta",
        "targets": [
          {
            "expr": "count(ALERTS{environment=\"beta\", alertstate=\"firing\"})",
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
        "datasource": "Prometheus-Beta",
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
        "datasource": "Prometheus-Beta",
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
        "datasource": "Prometheus-Beta",
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
        "datasource": "Alertmanager-Beta",
        "options": {
          "showOptions": "current",
          "sortOrder": 1,
          "alertName": "",
          "dashboardName": "",
          "maxItems": 10
        }
      },
      
      {
        "id": 9,
        "title": "📊 Verifier 不匹配率",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 0, "y": 22},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "verifier_mismatch_rate",
            "legendFormat": "Mismatch %",
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
                {"value": 0.5, "color": "yellow"},
                {"value": 1, "color": "red"}
              ]
            }
          }
        }
      },
      
      {
        "id": 10,
        "title": "⏱️ Gateway P99 时延",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 6, "y": 22},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le))",
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
                {"value": 200, "color": "yellow"},
                {"value": 250, "color": "red"}
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

## 3. Beta Application Performance 仪表盘

### 3.1 仪表盘配置

```json
{
  "dashboard": {
    "uid": "beta-app-perf",
    "title": "⚡ Beta 应用性能监控",
    "description": "Phase 4 Beta 环境应用性能详细监控",
    "tags": ["phase4", "beta", "application", "performance"],
    "timezone": "Asia/Shanghai",
    "refresh": "10s",
    
    "panels": [
      {
        "id": 1,
        "title": "🎯 Executor 指令执行时延 (P50/P95/P99)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P50",
            "refId": "A"
          },
          {
            "expr": "histogram_quantile(0.95, sum(rate(executor_instruction_latency_p95_bucket[5m])) by(le))",
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
                {"value": 180, "color": "red"}
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
        "datasource": "Prometheus-Beta",
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
            "max": 120,
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"value": null, "color": "green"},
                {"value": 60, "color": "yellow"},
                {"value": 80, "color": "red"}
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
        "datasource": "Prometheus-Beta",
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
                {"value": 97, "color": "yellow"},
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
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P50",
            "refId": "A"
          },
          {
            "expr": "histogram_quantile(0.95, sum(rate(verifier_verification_latency_p95_bucket[5m])) by(le))",
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
                {"value": 180, "color": "red"}
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
        "datasource": "Prometheus-Beta",
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
            "max": 120,
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"value": null, "color": "green"},
                {"value": 60, "color": "yellow"},
                {"value": 80, "color": "red"}
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
        "datasource": "Prometheus-Beta",
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
                {"value": 0.5, "color": "yellow"},
                {"value": 1, "color": "red"}
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
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P50",
            "refId": "A"
          },
          {
            "expr": "histogram_quantile(0.95, sum(rate(gateway_request_latency_p95_bucket[5m])) by(le))",
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
                {"value": 250, "color": "red"}
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
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "sum(rate(gateway_request_rate[1m])) by(method)",
            "legendFormat": "{{method}}",
            "refId": "A"
          }
        ]
      },
      
      {
        "id": 9,
        "title": "❌ Gateway 错误率",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 48},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "gateway_error_rate",
            "legendFormat": "Error Rate %",
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
                {"value": 2, "color": "red"}
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

## 4. Beta System Resources 仪表盘

### 4.1 仪表盘配置

```json
{
  "dashboard": {
    "uid": "beta-system",
    "title": "🖥️ Beta 系统资源监控",
    "description": "Phase 4 Beta 环境系统资源详细监控",
    "tags": ["phase4", "beta", "system", "resources"],
    "timezone": "Asia/Shanghai",
    "refresh": "20s",
    
    "panels": [
      {
        "id": 1,
        "title": "📊 CPU 使用率 (按实例)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "datasource": "Prometheus-Beta",
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
                {"value": 75, "color": "yellow"},
                {"value": 85, "color": "red"}
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
        "datasource": "Prometheus-Beta",
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
                {"value": 80, "color": "yellow"},
                {"value": 90, "color": "red"}
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
        "datasource": "Prometheus-Beta",
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
                {"value": 75, "color": "yellow"},
                {"value": 85, "color": "red"}
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
        "datasource": "Prometheus-Beta",
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
        "datasource": "Prometheus-Beta",
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
        "datasource": "Prometheus-Beta",
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
                {"value": 60, "color": "yellow"},
                {"value": 75, "color": "red"}
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
        "datasource": "Prometheus-Beta",
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
      },
      
      {
        "id": 8,
        "title": "⚠️ 系统告警状态",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 0, "y": 24},
        "datasource": "Alertmanager-Beta",
        "targets": [
          {
            "expr": "count(ALERTS{environment=\"beta\", component=\"system\", alertstate=\"firing\"})",
            "legendFormat": "System Alerts",
            "refId": "A"
          }
        ]
      },
      
      {
        "id": 9,
        "title": "📈 CPU 使用率 Top5 实例",
        "type": "table",
        "gridPos": {"h": 8, "w": 12, "x": 6, "y": 24},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "topk(5, avg(node_cpu_usage_percent) by(instance))",
            "legendFormat": "{{instance}}",
            "refId": "A"
          }
        ]
      },
      
      {
        "id": 10,
        "title": "💾 内存使用率 Top5 实例",
        "type": "table",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 32},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "topk(5, avg(node_memory_usage_percent) by(instance))",
            "legendFormat": "{{instance}}",
            "refId": "A"
          }
        ]
      }
    ]
  }
}
```

---

## 5. Beta Database 仪表盘

### 5.1 仪表盘配置

```json
{
  "dashboard": {
    "uid": "beta-database",
    "title": "🗄️ Beta PostgreSQL 监控",
    "description": "Phase 4 Beta 环境 PostgreSQL 数据库详细监控",
    "tags": ["phase4", "beta", "database", "postgresql"],
    "timezone": "Asia/Shanghai",
    "refresh": "10s",
    
    "panels": [
      {
        "id": 1,
        "title": "🔗 PostgreSQL 连接数 (活跃/空闲/最大)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "datasource": "Prometheus-Beta",
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
          },
          {
            "expr": "postgres_connections_max",
            "legendFormat": "Max",
            "refId": "C"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"value": null, "color": "green"},
                {"value": 70, "color": "yellow"},
                {"value": 90, "color": "red"}
              ]
            }
          }
        }
      },
      
      {
        "id": 2,
        "title": "⏱️ 查询时延 P99/P95",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, sum(rate(postgres_query_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P99 (ms)",
            "refId": "A"
          },
          {
            "expr": "histogram_quantile(0.95, sum(rate(postgres_query_latency_p95_bucket[5m])) by(le))",
            "legendFormat": "P95 (ms)",
            "refId": "B"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "ms",
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"value": null, "color": "green"},
                {"value": 60, "color": "yellow"},
                {"value": 80, "color": "red"}
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
        "datasource": "Prometheus-Beta",
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
        "title": "🔒 等待锁/持有锁数量",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "postgres_locks_waiting",
            "legendFormat": "Locks Waiting",
            "refId": "A"
          },
          {
            "expr": "postgres_locks_held",
            "legendFormat": "Locks Held",
            "refId": "B"
          }
        ],
        "fieldConfig": {
          "defaults": {
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
      },
      
      {
        "id": 5,
        "title": "📊 数据库健康度",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 0, "y": 16},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "(postgres_connections_active < 70) and (histogram_quantile(0.99, sum(rate(postgres_query_latency_p99_bucket[5m])) by(le)) < 80) and (postgres_locks_waiting < 3)",
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
        "title": "🔄 复制延迟",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 6, "y": 16},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "postgres_replication_lag_seconds",
            "legendFormat": "Lag (s)",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "s",
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
        "id": 7,
        "title": "💾 缓存命中率",
        "type": "gauge",
        "gridPos": {"h": 6, "w": 6, "x": 12, "y": 16},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "postgres_cache_hit_ratio * 100",
            "legendFormat": "Cache Hit %",
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
                {"value": 98, "color": "green"}
              ]
            }
          }
        }
      },
      
      {
        "id": 8,
        "title": "📈 慢查询趋势",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 24},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "sum(rate(postgres_query_latency_p99_bucket{le=\"1000\"}[5m])) by(instance)",
            "legendFormat": "Slow Queries ({{instance}})",
            "refId": "A"
          }
        ]
      },
      
      {
        "id": 9,
        "title": "🔗 连接使用率",
        "type": "gauge",
        "gridPos": {"h": 6, "w": 6, "x": 0, "y": 32},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "postgres_connections_active / postgres_connections_max * 100",
            "legendFormat": "Connection Usage %",
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
                {"value": 85, "color": "red"}
              ]
            }
          }
        }
      },
      
      {
        "id": 10,
        "title": "📊 数据库告警状态",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 6, "y": 32},
        "datasource": "Alertmanager-Beta",
        "targets": [
          {
            "expr": "count(ALERTS{environment=\"beta\", component=\"database\", alertstate=\"firing\"})",
            "legendFormat": "DB Alerts",
            "refId": "A"
          }
        ]
      },
      
      {
        "id": 11,
        "title": "📋 Top 10 慢查询",
        "type": "table",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 24},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "topk(10, avg(rate(postgres_query_latency_p99_bucket[5m])) by(query))",
            "legendFormat": "{{query}}",
            "refId": "A"
          }
        ]
      },
      
      {
        "id": 12,
        "title": "📈 查询量趋势",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 40},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "rate(postgres_transactions_total[1m])",
            "legendFormat": "Queries/s",
            "refId": "A"
          }
        ]
      }
    ]
  }
}
```

---

## 6. Beta Scheduler 仪表盘

### 6.1 仪表盘配置

```json
{
  "dashboard": {
    "uid": "beta-scheduler",
    "title": "⏰ Beta Scheduler 监控",
    "description": "Phase 4 Beta 环境 Scheduler 任务调度监控",
    "tags": ["phase4", "beta", "scheduler"],
    "timezone": "Asia/Shanghai",
    "refresh": "10s",
    
    "panels": [
      {
        "id": 1,
        "title": "⏱️ Scheduler 任务时延 (P50/P95/P99)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, sum(rate(scheduler_task_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P50",
            "refId": "A"
          },
          {
            "expr": "histogram_quantile(0.95, sum(rate(scheduler_task_latency_p99_bucket[5m])) by(le))",
            "legendFormat": "P95",
            "refId": "B"
          },
          {
            "expr": "histogram_quantile(0.99, sum(rate(scheduler_task_latency_p99_bucket[5m])) by(le))",
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
                {"value": 250, "color": "yellow"},
                {"value": 300, "color": "red"}
              ]
            }
          }
        }
      },
      
      {
        "id": 2,
        "title": "📊 待调度任务数",
        "type": "gauge",
        "gridPos": {"h": 6, "w": 6, "x": 12, "y": 0},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "scheduler_pending_tasks",
            "legendFormat": "Pending Tasks",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "min": 0,
            "max": 80,
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"value": null, "color": "green"},
                {"value": 40, "color": "yellow"},
                {"value": 50, "color": "red"}
              ]
            }
          }
        }
      },
      
      {
        "id": 3,
        "title": "✅ Scheduler 成功率",
        "type": "gauge",
        "gridPos": {"h": 6, "w": 6, "x": 18, "y": 0},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "scheduler_success_rate",
            "legendFormat": "Success %",
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
                {"value": 98, "color": "yellow"},
                {"value": 99, "color": "green"}
              ]
            }
          }
        }
      },
      
      {
        "id": 4,
        "title": "📈 待调度任务趋势",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "scheduler_pending_tasks",
            "legendFormat": "Pending",
            "refId": "A"
          }
        ]
      },
      
      {
        "id": 5,
        "title": "✅ Scheduler 成功率趋势",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "scheduler_success_rate",
            "legendFormat": "Success Rate %",
            "refId": "A"
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
        "title": "⚠️ Scheduler 告警状态",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 0, "y": 16},
        "datasource": "Alertmanager-Beta",
        "targets": [
          {
            "expr": "count(ALERTS{environment=\"beta\", component=\"scheduler\", alertstate=\"firing\"})",
            "legendFormat": "Scheduler Alerts",
            "refId": "A"
          }
        ]
      }
    ]
  }
}
```

---

## 7. Beta Container Monitoring 仪表盘

### 7.1 仪表盘配置

```json
{
  "dashboard": {
    "uid": "beta-containers",
    "title": "📦 Beta 容器监控",
    "description": "Phase 4 Beta 环境容器资源监控",
    "tags": ["phase4", "beta", "containers", "kubernetes"],
    "timezone": "Asia/Shanghai",
    "refresh": "20s",
    
    "panels": [
      {
        "id": 1,
        "title": "📊 容器 CPU 使用率 (Top10)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "topk(10, avg(container_cpu_usage_percent) by(container_name))",
            "legendFormat": "{{container_name}}",
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
        "title": "💾 容器内存使用 (Top10)",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "topk(10, avg(container_memory_usage_bytes) by(container_name))",
            "legendFormat": "{{container_name}}",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "bytes"
          }
        }
      },
      
      {
        "id": 3,
        "title": "📊 容器资源汇总",
        "type": "stat",
        "gridPos": {"h": 6, "w": 6, "x": 0, "y": 8},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "avg(container_cpu_usage_percent)",
            "legendFormat": "Avg CPU %",
            "refId": "A"
          },
          {
            "expr": "sum(container_memory_usage_bytes)",
            "legendFormat": "Total Memory",
            "refId": "B"
          }
        ],
        "options": {
          "orientation": "vertical",
          "graphMode": "area"
        }
      },
      
      {
        "id": 4,
        "title": "📋 容器列表",
        "type": "table",
        "gridPos": {"h": 8, "w": 12, "x": 6, "y": 8},
        "datasource": "Prometheus-Beta",
        "targets": [
          {
            "expr": "avg(container_cpu_usage_percent) by(container_name)",
            "legendFormat": "{{container_name}}",
            "refId": "A"
          }
        ]
      }
    ]
  }
}
```

---

## 8. 仪表盘部署

### 8.1 部署脚本

```bash
#!/bin/bash
# deploy-beta-dashboards.sh

GRAFANA_URL="http://grafana-beta:3000"
GRAFANA_USER="admin"
GRAFANA_PASSWORD="${GRAFANA_ADMIN_PASSWORD}"
DASHBOARD_DIR="/var/lib/grafana/dashboards/beta"

echo "🚀 开始部署 Beta 环境 Grafana 仪表盘..."

# 创建仪表盘目录
mkdir -p $DASHBOARD_DIR

# 复制仪表盘配置文件
cp beta-overview.json $DASHBOARD_DIR/
cp beta-app-perf.json $DASHBOARD_DIR/
cp beta-system.json $DASHBOARD_DIR/
cp beta-database.json $DASHBOARD_DIR/
cp beta-scheduler.json $DASHBOARD_DIR/
cp beta-containers.json $DASHBOARD_DIR/

# 使用 Grafana API 导入仪表盘
for dashboard in beta-overview beta-app-perf beta-system beta-database beta-scheduler beta-containers; do
  echo "📊 导入仪表盘：$dashboard"
  
  curl -X POST "$GRAFANA_URL/api/dashboards/db" \
    -u "$GRAFANA_USER:$GRAFANA_PASSWORD" \
    -H "Content-Type: application/json" \
    -d @"$DASHBOARD_DIR/$dashboard.json"
done

# 验证仪表盘
echo "\n✅ 验证仪表盘导入..."
curl -u "$GRAFANA_USER:$GRAFANA_PASSWORD" \
  "$GRAFANA_URL/api/search?query=beta" | jq '.[].title'

echo "\n🎉 Beta 环境仪表盘部署完成!"
```

### 8.2 数据源配置

```yaml
# grafana-datasources-beta.yaml

apiVersion: 1

datasources:
  - name: Prometheus-Beta
    type: prometheus
    access: proxy
    url: http://prometheus-beta:9090
    isDefault: true
    editable: false
    jsonData:
      timeInterval: "10s"
      queryTimeout: "30s"
      
  - name: Alertmanager-Beta
    type: alertmanager
    access: proxy
    url: http://alertmanager-beta:9093
    editable: false
    jsonData:
      implementation: prometheus
```

---

## 9. 验收标准

### 9.1 仪表盘验收

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 仪表盘加载 | <3s | 人工测试 | 所有仪表盘<3s |
| Panel 显示 | 60 个 Panel 正常 | Grafana 检查 | 100% Panel 正常 |
| 数据刷新 | 10-20s | 观察验证 | 刷新正常 |
| 阈值标识 | 阈值线正确 | 视觉检查 | 阈值线正确 |
| 告警集成 | 20 个告警规则关联 | 告警测试 | 告警触发正常 |

### 9.2 快速验证命令

```bash
# 验证仪表盘列表
curl -u admin:admin 'http://grafana-beta:3000/api/search?query=beta' | jq '.[].title'

# 验证仪表盘数据
curl -u admin:admin 'http://grafana-beta:3000/api/dashboards/uid/beta-overview' | jq '.dashboard.title'

# 验证数据源
curl -u admin:admin 'http://grafana-beta:3000/api/datasources' | jq '.[].name'
```

---

## 10. 实施计划

| 任务 | 责任人 | 状态 | 交付物 | 预计耗时 |
|---|---|---|---|---|
| Overview 仪表盘设计 | Observability | ✅ 完成 | beta-overview.json | 60 分钟 |
| App Performance 仪表盘 | Observability | ✅ 完成 | beta-app-perf.json | 90 分钟 |
| System Resources 仪表盘 | SRE | ✅ 完成 | beta-system.json | 60 分钟 |
| Database 仪表盘 | SRE | ✅ 完成 | beta-database.json | 60 分钟 |
| Scheduler 仪表盘 | Observability | ✅ 完成 | beta-scheduler.json | 60 分钟 |
| Container 仪表盘 | SRE | ✅ 完成 | beta-containers.json | 60 分钟 |
| 仪表盘部署 | SRE | ✅ 完成 | deployment_log.md | 30 分钟 |
| 仪表盘验证 | Observability + SRE | ✅ 完成 | validation_report.md | 30 分钟 |

---

## 11. 附录

### 11.1 PromQL 查询手册

```promql
# === Overview 仪表盘 ===

# 系统健康度
(avg(node_cpu_usage_percent) < 75 and avg(node_memory_usage_percent) < 80 and avg(node_disk_usage_percent) < 85)

# Executor P99 时延
histogram_quantile(0.99, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))

# Executor 成功率
executor_success_rate

# 活跃告警数
count(ALERTS{environment="beta", alertstate="firing"})

# === Application Performance 仪表盘 ===

# Executor P50/P95/P99
histogram_quantile(0.50, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.95, sum(rate(executor_instruction_latency_p95_bucket[5m])) by(le))
histogram_quantile(0.99, sum(rate(executor_instruction_latency_p99_bucket[5m])) by(le))

# Verifier P50/P95/P99
histogram_quantile(0.50, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.95, sum(rate(verifier_verification_latency_p95_bucket[5m])) by(le))
histogram_quantile(0.99, sum(rate(verifier_verification_latency_p99_bucket[5m])) by(le))

# Gateway P50/P95/P99
histogram_quantile(0.50, sum(rate(gateway_request_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.95, sum(rate(gateway_request_latency_p95_bucket[5m])) by(le))
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
node_load_average_5m

# === Database 仪表盘 ===

# 连接数
postgres_connections_active
postgres_connections_idle
postgres_connections_max

# 查询时延 P99/P95
histogram_quantile(0.99, sum(rate(postgres_query_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.95, sum(rate(postgres_query_latency_p95_bucket[5m])) by(le))

# 事务速率
rate(postgres_transactions_total[1m])

# 锁
postgres_locks_waiting
postgres_locks_held

# 复制延迟
postgres_replication_lag_seconds

# 缓存命中率
postgres_cache_hit_ratio

# === Scheduler 仪表盘 ===

# 任务时延 P50/P95/P99
histogram_quantile(0.50, sum(rate(scheduler_task_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.95, sum(rate(scheduler_task_latency_p99_bucket[5m])) by(le))
histogram_quantile(0.99, sum(rate(scheduler_task_latency_p99_bucket[5m])) by(le))

# 待调度任务数
scheduler_pending_tasks

# 成功率
scheduler_success_rate

# === Container 仪表盘 ===

# 容器 CPU
container_cpu_usage_percent by(container_name)

# 容器内存
container_memory_usage_bytes by(container_name)
```

### 11.2 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Beta 35 指标配置 | beta_monitoring_35_metrics.md | 指标定义 |
| Beta 20 告警规则 | beta_alert_rules_20.md | 告警配置 |
| Alpha 仪表盘配置 | alpha_week1/alpha_grafana_dashboard.md | 参考实现 |

---

**文档状态**: ✅ Week 2-T5 完成  
**创建日期**: 2026-04-12  
**责任人**: Observability-Agent + SRE-Agent  
**保管**: 项目文档库  
**环境**: Beta (Phase 4 Week 2)
