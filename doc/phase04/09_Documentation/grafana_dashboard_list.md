# Grafana 仪表盘清单

**版本**: v1.0  
**日期**: 2026-03-07  

---

## Staging 环境 (9 个仪表盘)

### 01. Staging Environment Overview

**UID**: `staging-overview`  
**路径**: Home → Staging Overview  
**说明**: 环境总览仪表盘

**面板**:
1. Prometheus Targets - 监控目标数量
2. CPU 使用率 - 平均 CPU 使用百分比
3. 内存使用率 - 平均内存使用百分比
4. 数据库实例数 - PostgreSQL 实例数量
5. 应用健康状态 - 应用服务器健康度时间序列
6. 数据库健康状态 - PostgreSQL 健康度时间序列

**访问 URL**: http://localhost:3003/d/staging-overview

---

### 02. System Overview

**UID**: `system-overview`  
**路径**: Home → System Overview  
**说明**: 系统资源监控

**面板**:
1. CPU Usage - CPU 使用率统计
2. Memory Usage - 内存使用率统计
3. Disk Usage - 磁盘使用率统计
4. Uptime - 系统运行时间

**访问 URL**: http://localhost:3003/d/system-overview

---

### 03. Database Monitoring

**UID**: `database-monitoring`  
**路径**: Home → Database Monitoring  
**说明**: PostgreSQL 数据库监控

**面板**:
1. PostgreSQL Up - 数据库运行状态
2. Active Connections - 活跃连接数
3. Query Duration - 查询耗时时间序列

**访问 URL**: http://localhost:3003/d/database-monitoring

---

### 04. Redis Monitoring

**UID**: `redis-monitoring`  
**路径**: Home → Redis Monitoring  
**说明**: Redis 缓存监控

**面板**:
1. Redis Up - Redis 运行状态
2. Connected Clients - 连接客户端数
3. Memory Usage - 内存使用量 (MB)
4. Hit Rate - 缓存命中率

**访问 URL**: http://localhost:3003/d/redis-monitoring

---

### 05. Nginx Load Balancer

**UID**: `nginx-monitoring`  
**路径**: Home → Nginx Monitoring  
**说明**: Nginx 负载均衡监控

**面板**:
1. Active Connections - 活跃连接数
2. Requests/sec - 每秒请求数
3. Handled Connections - 已处理连接数

**访问 URL**: http://localhost:3003/d/nginx-monitoring

---

### 06. Application Metrics

**UID**: `application-metrics`  
**路径**: Home → Application Metrics  
**说明**: 应用指标监控

**面板**:
1. App Uptime - 应用运行时间
2. Total Requests - 总请求数
3. Total Errors - 总错误数
4. Error Rate - 错误率

**访问 URL**: http://localhost:3003/d/application-metrics

---

### 07. Alerts Overview

**UID**: `alerts-overview`  
**路径**: Home → Alerts Overview  
**说明**: 告警总览

**面板**:
1. Firing Alerts - 正在触发的告警列表
2. Total Active Alerts - 活跃告警总数
3. P0 Alerts - P0 级别告警数

**访问 URL**: http://localhost:3003/d/alerts-overview

---

### 08. Prometheus Monitoring

**UID**: `prometheus-monitoring`  
**路径**: Home → Prometheus Monitoring  
**说明**: Prometheus 自监控

**面板**:
1. Targets Up - 正常运行的 targets 数量
2. Targets Down - 宕机的 targets 数量
3. Total Targets - targets 总数
4. Scrape Duration - 抓取耗时
5. Samples Scraped - 抓取样本数

**访问 URL**: http://localhost:3003/d/prometheus-monitoring

---

### 09. Grafana Monitoring

**UID**: `grafana-monitoring`  
**路径**: Home → Grafana Monitoring  
**说明**: Grafana 自监控

**面板**:
1. Datasources - 数据源数量
2. Active Users - 活跃用户数
3. Total Dashboards - 仪表盘总数
4. Total Panels - 面板总数

**访问 URL**: http://localhost:3003/d/grafana-monitoring

---

## Alpha 环境 (1 个仪表盘)

### Alpha Environment Overview

**UID**: `alpha-overview`  
**路径**: Home → Alpha Overview  
**说明**: Alpha 环境总览

**面板**:
1. CPU Usage - CPU 使用率
2. Memory Usage - 内存使用率
3. Targets Up - 监控目标数量
4. CPU Usage Over Time - CPU 时间序列
5. Memory Usage Over Time - 内存时间序列

**访问 URL**: http://localhost:3001/d/alpha-overview

---

## Beta 环境 (1 个仪表盘)

### Beta Environment Overview

**UID**: `beta-overview`  
**路径**: Home → Beta Overview  
**说明**: Beta 环境总览

**面板**:
1. CPU Usage - CPU 使用率
2. Memory Usage - 内存使用率
3. Targets Up - 监控目标数量
4. CPU Usage Over Time - CPU 时间序列
5. Memory Usage Over Time - 内存时间序列

**访问 URL**: http://localhost:3002/d/beta-overview

---

## 仪表盘统计

| 环境 | 仪表盘数量 | 面板总数 | 告警规则 |
|------|-----------|---------|---------|
| **Alpha** | 1 | 5 | 0 |
| **Beta** | 1 | 5 | 0 |
| **Staging** | 9 | 33 | 24 |
| **总计** | **11** | **43** | **24** |

---

## 快速访问链接

### Staging 环境

```
总览：http://localhost:3003/d/staging-overview
系统：http://localhost:3003/d/system-overview
数据库：http://localhost:3003/d/database-monitoring
Redis: http://localhost:3003/d/redis-monitoring
Nginx: http://localhost:3003/d/nginx-monitoring
应用：http://localhost:3003/d/application-metrics
告警：http://localhost:3003/d/alerts-overview
Prometheus: http://localhost:3003/d/prometheus-monitoring
Grafana: http://localhost:3003/d/grafana-monitoring
```

### Alpha 环境

```
总览：http://localhost:3001/d/alpha-overview
```

### Beta 环境

```
总览：http://localhost:3002/d/beta-overview
```

---

**文档维护**: SRE Team  
**最后更新**: 2026-03-07
