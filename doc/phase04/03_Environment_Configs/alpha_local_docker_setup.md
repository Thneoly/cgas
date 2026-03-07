# Alpha 环境本地 Docker 搭建指南

**版本**: v1.0  
**日期**: 2026-04-01  
**责任人**: SRE-Agent + Dev-Agent  
**状态**: ✅ 就绪  
**环境**: Alpha (本地 Docker 搭建)

---

## 📋 执行摘要

本指南用于指导在**本地开发机器**上使用 Docker Compose 快速搭建 Alpha 环境，无需申请云服务器资源。

**Alpha 环境目标**:
- 内部功能验证
- 性能基线测量
- Alpha 测试通过率≥95%

**本地环境优势**:
- ✅ 快速部署 (10 分钟内就绪)
- ✅ 零云成本
- ✅ 环境可重复
- ✅ 易于销毁和重建

---

## 🖥️ 本地资源配置

### 最低要求

| 资源 | 要求 | 说明 |
|------|------|------|
| CPU | 8 核+ | 建议 12 核+ |
| 内存 | 32 GB+ | 建议 64 GB |
| 磁盘 | 100 GB SSD | 剩余空间 |
| Docker | 20.10+ | Docker Compose v2+ |

### 推荐配置

| 资源 | 推荐 | 用途 |
|------|------|------|
| CPU | 16 核 | 运行多个容器 |
| 内存 | 64 GB | 数据库缓存 + 应用 |
| 磁盘 | 500 GB NVMe SSD | 高性能存储 |

---

## 🐳 Docker Compose 配置

### 1. 完整环境配置 (推荐)

**文件**: `docker-compose.alpha.yml`

```yaml
version: '3.8'

# Alpha 环境 - 本地 Docker 搭建
# 包含：Executor, Verifier, PostgreSQL, Redis, Prometheus, Grafana

services:
  # ==================== 数据库服务 ====================
  postgres:
    image: postgres:15-alpine
    container_name: cgas-alpha-postgres
    environment:
      POSTGRES_DB: cgas
      POSTGRES_USER: cgas
      POSTGRES_PASSWORD: AlphaEnv2026!Secure
      POSTGRES_INITDB_ARGS: "--encoding=UTF-8 --lc-collate=C --lc-ctype=C"
    ports:
      - "5432:5432"
    volumes:
      - cgas_alpha_data:/var/lib/postgresql/data
      - ./scripts/init-db.sql:/docker-entrypoint-initdb.d/init-db.sql
    networks:
      - cgas_alpha_network
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 8G
        reservations:
          cpus: '2.0'
          memory: 4G
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U cgas -d cgas"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  # ==================== 缓存服务 ====================
  redis:
    image: redis:7-alpine
    container_name: cgas-alpha-redis
    command: redis-server --appendonly yes --maxmemory 2gb --maxmemory-policy allkeys-lru
    ports:
      - "6379:6379"
    volumes:
      - cgas_alpha_redis:/data
    networks:
      - cgas_alpha_network
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  # ==================== 执行器服务 ====================
  executor:
    image: cgas-executor:alpha
    container_name: cgas-alpha-executor
    build:
      context: ../../..
      dockerfile: docker/Executor.Dockerfile
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgresql://cgas:AlphaEnv2026!Secure@postgres:5432/cgas
      - REDIS_URL=redis://redis:6379
      - ENVIRONMENT=alpha
      - SERVICE_NAME=executor
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    networks:
      - cgas_alpha_network
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 4G
        reservations:
          cpus: '2.0'
          memory: 2G
    volumes:
      - cgas_alpha_logs:/app/logs
      - /var/run/docker.sock:/var/run/docker.sock
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    restart: unless-stopped

  # ==================== 验证器服务 ====================
  verifier:
    image: cgas-verifier:alpha
    container_name: cgas-alpha-verifier
    build:
      context: ../../..
      dockerfile: docker/Verifier.Dockerfile
    ports:
      - "8081:8080"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgresql://cgas:AlphaEnv2026!Secure@postgres:5432/cgas
      - REDIS_URL=redis://redis:6379
      - ENVIRONMENT=alpha
      - SERVICE_NAME=verifier
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
      executor:
        condition: service_started
    networks:
      - cgas_alpha_network
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 4G
        reservations:
          cpus: '2.0'
          memory: 2G
    volumes:
      - cgas_alpha_logs:/app/logs
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    restart: unless-stopped

  # ==================== 监控服务 ====================
  # Prometheus - 指标收集
  prometheus:
    image: prom/prometheus:v2.45.0
    container_name: cgas-alpha-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.alpha.yml:/etc/prometheus/prometheus.yml
      - cgas_alpha_prometheus:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=7d'
      - '--web.enable-lifecycle'
    networks:
      - cgas_alpha_network
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 4G
    restart: unless-stopped

  # Grafana - 可视化
  grafana:
    image: grafana/grafana:10.0.0
    container_name: cgas-alpha-grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_USER=admin
      - GF_SECURITY_ADMIN_PASSWORD=AlphaGrafana2026
      - GF_INSTALL_PLUGINS=grafana-clock-panel,grafana-simple-json-datasource
    volumes:
      - cgas_alpha_grafana:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources
    depends_on:
      - prometheus
    networks:
      - cgas_alpha_network
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
    restart: unless-stopped

  # OpenTelemetry Collector - 追踪收集
  otel-collector:
    image: otel/opentelemetry-collector-contrib:0.80.0
    container_name: cgas-alpha-otel-collector
    ports:
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP
      - "8889:8889"   # Prometheus metrics
    volumes:
      - ./monitoring/otel-collector.alpha.yml:/etc/otel-collector-config.yaml
    command: ["--config=/etc/otel-collector-config.yaml"]
    networks:
      - cgas_alpha_network
    depends_on:
      - prometheus
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
    restart: unless-stopped

  # Node Exporter - 系统指标
  node-exporter:
    image: prom/node-exporter:v1.6.0
    container_name: cgas-alpha-node-exporter
    ports:
      - "9100:9100"
    volumes:
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /:/rootfs:ro
    command:
      - '--path.procfs=/host/proc'
      - '--path.sysfs=/host/sys'
      - '--collector.filesystem.mount-points-exclude=^/(sys|proc|dev|host|etc)($$|/)'
    networks:
      - cgas_alpha_network
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
    restart: unless-stopped

# ==================== 网络和存储 ====================
networks:
  cgas_alpha_network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.28.0.0/16

volumes:
  cgas_alpha_data:
    driver: local
  cgas_alpha_redis:
    driver: local
  cgas_alpha_logs:
    driver: local
  cgas_alpha_prometheus:
    driver: local
  cgas_alpha_grafana:
    driver: local
```

---

## 🚀 快速启动指南

### 1. 前置检查

```bash
# 检查 Docker 版本
docker --version  # 应该 >= 20.10
docker-compose version  # 应该 >= v2.0

# 检查系统资源
free -h  # 确保 >= 32GB 可用内存
df -h /  # 确保 >= 100GB 可用磁盘
```

### 2. 准备目录结构

```bash
# 创建目录
mkdir -p /opt/cgas-alpha/{scripts,monitoring/grafana/{dashboards,datasources},logs}
cd /opt/cgas-alpha

# 复制配置文件
cp /home/cc/Desktop/code/AIPro/cgas/doc/phase04/03_Environment_Configs/docker-compose.alpha.yml .
```

### 3. 数据库初始化脚本

创建 `scripts/init-db.sql`:

```sql
-- CGAS Alpha 环境数据库初始化
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- 创建 CGAS 专用 schema
CREATE SCHEMA IF NOT EXISTS cgas;

-- 授予权限
GRANT ALL ON SCHEMA cgas TO cgas;
GRANT ALL ON ALL TABLES IN SCHEMA cgas TO cgas;
GRANT ALL ON ALL SEQUENCES IN SCHEMA cgas TO cgas;

-- 创建基础表 (如果不存在)
-- (具体表结构由应用迁移脚本管理)
```

### 4. 监控配置

创建 `monitoring/prometheus.alpha.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'executor'
    static_configs:
      - targets: ['executor:8080']
    metrics_path: '/metrics'

  - job_name: 'verifier'
    static_configs:
      - targets: ['verifier:8080']
    metrics_path: '/metrics'

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis:9121']

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
```

创建 `monitoring/otel-collector.alpha.yml`:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024

exporters:
  prometheus:
    endpoint: "0.0.0.0:8889"
    namespace: cgas

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus]
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus]
```

### 5. 启动环境

```bash
# 启动所有服务
docker-compose -f docker-compose.alpha.yml up -d

# 查看启动日志
docker-compose -f docker-compose.alpha.yml logs -f

# 检查服务状态
docker-compose -f docker-compose.alpha.yml ps
```

### 6. 验证服务

```bash
# 等待 60 秒让服务完全启动
sleep 60

# 检查执行器健康
curl -f http://localhost:8080/health
# 期望：{"status":"healthy"}

# 检查验证器健康
curl -f http://localhost:8081/health
# 期望：{"status":"healthy"}

# 检查数据库连接
docker exec cgas-alpha-postgres psql -U cgas -d cgas -c "SELECT 1"
# 期望：1

# 检查 Redis 连接
docker exec cgas-alpha-redis redis-cli ping
# 期望：PONG

# 检查 Prometheus
curl -f http://localhost:9090/api/v1/targets | jq '.data.activeTargets | length'
# 期望：> 0

# 检查 Grafana
curl -f http://localhost:3000/api/health
# 期望：{"commit":"xxx","database":"ok","version":"10.0.0"}
```

---

## 📊 访问服务

| 服务 | URL | 凭证 |
|------|-----|------|
| CGAS Executor API | http://localhost:8080 | - |
| CGAS Verifier API | http://localhost:8081 | - |
| PostgreSQL | localhost:5432 | cgas / AlphaEnv2026!Secure |
| Redis | localhost:6379 | - |
| Prometheus | http://localhost:9090 | - |
| Grafana | http://localhost:3000 | admin / AlphaGrafana2026 |

---

## 🔧 常用操作

### 查看日志

```bash
# 查看所有服务日志
docker-compose -f docker-compose.alpha.yml logs -f

# 查看特定服务日志
docker-compose -f docker-compose.alpha.yml logs -f executor
docker-compose -f docker-compose.alpha.yml logs -f verifier
```

### 重启服务

```bash
# 重启单个服务
docker-compose -f docker-compose.alpha.yml restart executor

# 重启所有服务
docker-compose -f docker-compose.alpha.yml restart
```

### 停止环境

```bash
# 停止所有服务 (保留数据)
docker-compose -f docker-compose.alpha.yml down

# 停止并删除数据 (彻底清理)
docker-compose -f docker-compose.alpha.yml down -v
```

### 进入容器

```bash
# 进入执行器容器
docker exec -it cgas-alpha-executor /bin/bash

# 进入数据库容器
docker exec -it cgas-alpha-postgres psql -U cgas -d cgas
```

### 资源监控

```bash
# 查看容器资源使用
docker stats cgas-alpha-executor cgas-alpha-verifier cgas-alpha-postgres

# 查看 Docker 整体资源
docker system df
```

---

## ✅ 环境验证清单

| 检查项 | 命令 | 期望结果 |
|--------|------|---------|
| Docker 版本 | `docker --version` | >= 20.10 |
| 服务状态 | `docker-compose ps` | 所有服务 Up |
| Executor 健康 | `curl localhost:8080/health` | {"status":"healthy"} |
| Verifier 健康 | `curl localhost:8081/health` | {"status":"healthy"} |
| 数据库连接 | `psql -h localhost -U cgas` | 连接成功 |
| Redis 连接 | `redis-cli ping` | PONG |
| Prometheus 目标 | `curl localhost:9090/api/v1/targets` | >0 个目标 |
| Grafana 登录 | `curl localhost:3000/api/health` | 健康 |

---

## 📈 性能基线测量

### 压测脚本

```bash
# 安装 k6
sudo apt-get install -y k6

# 运行压测
k6 run /opt/cgas-alpha/scripts/alpha_load_test.js
```

### 关键指标

| 指标 | 目标 | 测量命令 |
|------|------|---------|
| P99 时延 | <200ms | `curl -w "@format.txt" http://localhost:8080/api/v1/execute` |
| 吞吐量 | >1000 QPS | k6 压测 |
| 错误率 | <0.1% | Prometheus 查询 |
| CPU 使用率 | <70% | `docker stats` |
| 内存使用率 | <80% | `docker stats` |

---

## 🚨 故障排查

### 常见问题

| 问题 | 可能原因 | 解决方案 |
|------|---------|---------|
| 服务启动失败 | 端口冲突 | `lsof -i :8080` 检查端口占用 |
| 数据库连接失败 | 密码错误 | 检查 DATABASE_URL 配置 |
| 内存不足 | 资源限制过高 | 调整 docker-compose 资源限制 |
| 监控无数据 | Prometheus 配置错误 | 检查 prometheus.alpha.yml |

### 日志位置

```bash
# 应用日志
/opt/cgas-alpha/logs/

# Docker 日志
docker-compose -f docker-compose.alpha.yml logs > alpha_logs.txt

# 系统日志
journalctl -u docker.service
```

---

## 📝 环境交付物

| 交付物 | 路径 | 状态 |
|--------|------|------|
| docker-compose.alpha.yml | 03_Environment_Configs/ | ✅ 已创建 |
| prometheus.alpha.yml | 05_Monitoring_Configs/ | 📋 待创建 |
| otel-collector.alpha.yml | 05_Monitoring_Configs/ | 📋 待创建 |
| alpha_deployment_report.md | 04_Deployment_Reports/ | 📋 待创建 |
| alpha_validation_report.md | 04_Deployment_Reports/ | 📋 待创建 |

---

## 💡 最佳实践

1. **定期备份数据**
   ```bash
   docker run --rm -v cgas_alpha_data:/data -v $(pwd):/backup ubuntu tar czf /backup/alpha_data_backup.tar.gz /data
   ```

2. **资源限制**
   - 生产环境不要使用本地 Docker
   - 本地环境仅用于开发和测试

3. **镜像更新**
   ```bash
   docker-compose -f docker-compose.alpha.yml pull
   docker-compose -f docker-compose.alpha.yml up -d --build
   ```

4. **清理未使用资源**
   ```bash
   docker system prune -a
   ```

---

**文档状态**: ✅ 就绪  
**责任人**: SRE-Agent + Dev-Agent  
**执行时间**: 10-15 分钟 (首次启动)

---

*Alpha Local Docker Setup Guide v1.0 - 2026-04-01*
