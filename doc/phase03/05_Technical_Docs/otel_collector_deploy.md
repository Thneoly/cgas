# Phase 3 Week 3 OpenTelemetry Collector 部署文档

**版本**: v1.0  
**日期**: 2026-03-07  
**责任人**: SRE-Agent + Observability-Agent  
**状态**: ✅ 已完成  
**release_id**: release-2026-03-07-phase3_week03  
**关联文档**: 
- trace_id_integration.md (Trace ID 全链路集成)
- dashboard_v6_batch2.md (第二批仪表盘)
- alert_rules_batch2.md (告警规则扩展)

---

## 1. 部署概述

### 1.1 部署目标

| 目标 | 说明 | 验收标准 |
|---|---|---|
| 统一数据采集 | 收集 Metrics/Traces/Logs | 所有服务接入 OTLP |
| 数据管道配置 | 处理、转换、路由数据 | 采样/脱敏/聚合生效 |
| Exporter 配置 | 导出到 Prometheus/Jaeger | 数据可查询 |
| 高可用部署 | 生产环境就绪 | 健康检查通过 |

### 1.2 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                     Application Services                         │
├─────────────────────────────────────────────────────────────────┤
│  Executor (Rust)  │  Verifier (Rust)  │  Gateway (TypeScript)  │
│  + OTel SDK       │  + OTel SDK       │  + OTel SDK            │
└─────────┬─────────┴─────────┬─────────┴──────────┬──────────────┘
          │                   │                     │
          │ OTLP (gRPC:4317)  │ OTLP (gRPC:4317)    │ OTLP (gRPC:4317)
          ▼                   ▼                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                  OpenTelemetry Collector                         │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ Receivers (接收器)                                         │  │
│  │ ├── otlp (gRPC:4317, HTTP:4318)                           │  │
│  │ ├── prometheus (scrape:8080)                              │  │
│  │ └── jaeger (thrift:6831)                                  │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ Processors (处理器)                                        │  │
│  │ ├── batch (批量导出)                                       │  │
│  │ ├── memory_limiter (内存限制)                              │  │
│  │ ├── probabilistic_sampler (采样)                           │  │
│  │ ├── resource (资源标签)                                    │  │
│  │ └── attributes (属性脱敏)                                  │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ Exporters (导出器)                                         │  │
│  │ ├── prometheus (metrics → Prometheus:9090)                │  │
│  │ ├── jaeger (traces → Jaeger:14250)                        │  │
│  │ ├── tempo (traces → Tempo:4317)                           │  │
│  │ └── logging (debug)                                       │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
          │                   │                     │
          ▼                   ▼                     ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐
│     Prometheus  │  │     Jaeger      │  │       Tempo         │
│  (Metrics)      │  │ (Traces Query)  │  │  (Traces Storage)   │
└─────────────────┘  └─────────────────┘  └─────────────────────┘
```

---

## 2. Docker Compose 配置

### 2.1 完整部署文件

```yaml
# docker-compose.observability.yaml
version: '3.8'

services:
  # ============================================
  # OpenTelemetry Collector
  # ============================================
  otel-collector:
    image: otel/opentelemetry-collector-contrib:0.95.0
    container_name: otel-collector
    command: ["--config=/etc/otel-collector-config.yaml", "--feature-gates=-pkg.translator.prometheus.NormalizeName"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml:ro
      - ./otel-collector-data:/var/lib/otelcol:rw
    ports:
      # OTLP 接收器
      - "4317:4317"   # OTLP gRPC (主要协议)
      - "4318:4318"   # OTLP HTTP
      # Prometheus 导出
      - "8889:8889"   # Prometheus metrics endpoint
      # 健康检查
      - "13133:13133" # Health check
      # 调试工具
      - "55679:55679" # zpages
    environment:
      - GOMAXPROCS=2
      - GOMEMLIMIT=1GiB
    restart: unless-stopped
    depends_on:
      prometheus:
        condition: service_healthy
      jaeger:
        condition: service_healthy
    networks:
      - observability
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:13133"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
    labels:
      - "com.openclaw.phase=phase3"
      - "com.openclaw.component=observability"

  # ============================================
  # Prometheus (Metrics Storage & Query)
  # ============================================
  prometheus:
    image: prom/prometheus:v2.50.1
    container_name: prometheus
    command:
      - "--config.file=/etc/prometheus/prometheus.yaml"
      - "--storage.tsdb.path=/prometheus"
      - "--storage.tsdb.retention.time=180d"
      - "--storage.tsdb.retention.size=50GB"
      - "--web.console.libraries=/etc/prometheus/console_libraries"
      - "--web.console.templates=/etc/prometheus/consoles"
      - "--web.enable-lifecycle"
      - "--web.enable-admin-api"
      - "--query.max-concurrency=20"
      - "--query.timeout=2m"
    volumes:
      - ./prometheus.yaml:/etc/prometheus/prometheus.yaml:ro
      - ./prometheus-alerts.yaml:/etc/prometheus/alerts.yaml:ro
      - prometheus-data:/prometheus:rw
    ports:
      - "9090:9090"
    environment:
      - TZ=Asia/Shanghai
    restart: unless-stopped
    networks:
      - observability
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:9090/-/healthy"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 30s
    labels:
      - "com.openclaw.phase=phase3"
      - "com.openclaw.component=monitoring"

  # ============================================
  # Jaeger (Traces Query & UI)
  # ============================================
  jaeger:
    image: jaegertracing/all-in-one:1.55
    container_name: jaeger
    environment:
      - COLLECTOR_OTLP_ENABLED=true
      - COLLECTOR_OTLP_GRPC_HOST_PORT=0.0.0.0:4317
      - COLLECTOR_OTLP_HTTP_HOST_PORT=0.0.0.0:4318
      - MEMORY_MAX_TRACES=100000
      - QUERY_BASE_PATH=/jaeger
      - LOG_LEVEL=info
    ports:
      - "16686:16686" # Jaeger UI
      - "4317:4317"   # OTLP gRPC (backup)
      - "4318:4318"   # OTLP HTTP (backup)
      - "14250:14250" # gRPC for direct span submission
    volumes:
      - jaeger-data:/var/lib/jaeger:rw
    restart: unless-stopped
    networks:
      - observability
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:16686"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 30s
    labels:
      - "com.openclaw.phase=phase3"
      - "com.openclaw.component=tracing"

  # ============================================
  # Tempo (Traces Storage - Optional)
  # ============================================
  tempo:
    image: grafana/tempo:2.4.0
    container_name: tempo
    command: ["-config.file=/etc/tempo.yaml"]
    volumes:
      - ./tempo.yaml:/etc/tempo.yaml:ro
      - tempo-data:/tmp/tempo:rw
    ports:
      - "3200:3200"   # Tempo API
      - "4317:4318"   # OTLP (alternate port to avoid conflict)
    restart: unless-stopped
    networks:
      - observability
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:3200/status"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 30s
    labels:
      - "com.openclaw.phase=phase3"
      - "com.openclaw.component=tracing"

  # ============================================
  # Grafana (Visualization)
  # ============================================
  grafana:
    image: grafana/grafana:10.3.4
    container_name: grafana
    environment:
      - GF_SECURITY_ADMIN_USER=admin
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_ADMIN_PASSWORD:-admin}
      - GF_USERS_ALLOW_SIGN_UP=false
      - GF_SERVER_ROOT_URL=http://localhost:3000
      - GF_AUTH_ANONYMOUS_ENABLED=false
      - GF_FEATURE_TOGGLES_ENABLE=traceqlEditor
      - TZ=Asia/Shanghai
    volumes:
      - grafana-data:/var/lib/grafana:rw
      - ./grafana/provisioning/datasources:/etc/grafana/provisioning/datasources:ro
      - ./grafana/provisioning/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./grafana/dashboards:/var/lib/grafana/dashboards:ro
    ports:
      - "3000:3000"
    restart: unless-stopped
    depends_on:
      prometheus:
        condition: service_healthy
      jaeger:
        condition: service_healthy
    networks:
      - observability
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 30s
    labels:
      - "com.openclaw.phase=phase3"
      - "com.openclaw.component=visualization"

  # ============================================
  # Alertmanager (Alert Routing)
  # ============================================
  alertmanager:
    image: prom/alertmanager:v0.26.0
    container_name: alertmanager
    command:
      - "--config.file=/etc/alertmanager/alertmanager.yaml"
      - "--storage.path=/alertmanager"
      - "--web.external-url=http://localhost:9093"
      - "--cluster.listen-address="
    volumes:
      - ./alertmanager.yaml:/etc/alertmanager/alertmanager.yaml:ro
      - alertmanager-data:/alertmanager:rw
    ports:
      - "9093:9093"
    restart: unless-stopped
    networks:
      - observability
    labels:
      - "com.openclaw.phase=phase3"
      - "com.openclaw.component=alerting"

volumes:
  prometheus-data:
    driver: local
  jaeger-data:
    driver: local
  tempo-data:
    driver: local
  grafana-data:
    driver: local
  alertmanager-data:
    driver: local
  otel-collector-data:
    driver: local

networks:
  observability:
    driver: bridge
    ipam:
      config:
        - subnet: 172.28.0.0/16
```

---

## 3. OpenTelemetry Collector 配置

### 3.1 完整配置文件

```yaml
# otel-collector-config.yaml

# ============================================
# Receivers (数据接收器)
# ============================================
receivers:
  # OTLP 接收器 (主要协议)
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
        max_recv_msg_size_mib: 20
        max_concurrent_streams: 100
      http:
        endpoint: 0.0.0.0:4318
        cors:
          allowed_origins:
            - http://localhost:3000
            - http://grafana:3000
        max_request_body_size: 20971520  # 20MB

  # Prometheus 接收器 (抓取指标)
  prometheus:
    config:
      global:
        scrape_interval: 15s
        scrape_timeout: 10s
        evaluation_interval: 15s
      
      scrape_configs:
        # CGAS Executor 服务
        - job_name: 'cgas-executor'
          static_configs:
            - targets: ['executor:8080']
          metrics_path: /metrics
          scheme: http
          relabel_configs:
            - source_labels: [__address__]
              target_label: instance
              regex: '([^:]+):\d+'
              replacement: '${1}'
        
        # CGAS Verifier 服务
        - job_name: 'cgas-verifier'
          static_configs:
            - targets: ['verifier:8081']
          metrics_path: /metrics
          scheme: http
        
        # CGAS Batch 服务
        - job_name: 'cgas-batch'
          static_configs:
            - targets: ['batch:8082']
          metrics_path: /metrics
          scheme: http
        
        # OpenTelemetry Collector 自身指标
        - job_name: 'otel-collector'
          static_configs:
            - targets: ['otel-collector:8889']
          metrics_path: /metrics

  # Jaeger 接收器 (兼容旧协议)
  jaeger:
    protocols:
      thrift_http:
        endpoint: 0.0.0.0:14268
      thrift_compact:
        endpoint: 0.0.0.0:6831
      thrift_binary:
        endpoint: 0.0.0.0:6832

# ============================================
# Processors (数据处理器)
# ============================================
processors:
  # Batch 处理器 (批量导出，提高性能)
  batch:
    timeout: 5s
    send_batch_size: 512
    send_batch_max_size: 1024
    metadata_keys:
      - X-Scope-OrgID
    metadata_cardinality_limit: 10

  # Memory Limiter (防止内存溢出)
  memory_limiter:
    check_interval: 1s
    limit_mib: 1000
    spike_limit_mib: 200

  # Probabilistic Sampler (概率采样)
  probabilistic_sampler:
    hash_seed: 42
    sampling_percentage: 10  # 默认 10% 采样率

  # Resource 处理器 (添加/修改资源标签)
  resource:
    attributes:
      # 添加 Phase 标签
      - key: phase
        value: phase3
        action: upsert
      
      # 添加环境标签
      - key: deployment.environment
        value: production
        action: upsert
      
      # 添加团队标签
      - key: team
        value: cgas-core
        action: upsert
      
      # 标准化服务名称
      - key: service.name
        from_attribute: service.name
        action: keep
      
      # 添加版本号
      - key: service.version
        value: 3.0.0
        action: upsert

  # Attributes 处理器 (属性脱敏/删除敏感信息)
  attributes:
    actions:
      # 删除敏感 HTTP 头
      - key: http.authorization
        action: delete
      
      - key: http.cookie
        action: delete
      
      - key: http.user_agent
        action: keep
      
      # 哈希处理 SQL 语句
      - key: db.statement
        action: hash
      
      # 删除个人身份信息
      - key: user.email
        action: delete
      
      - key: user.phone
        action: delete
      
      # 规范化错误信息
      - key: error.message
        pattern: ^.*$
        replacement: "[REDACTED]"
        action: update

  # Span 处理器 (Span 级别处理)
  span:
    name:
      from_attributes:
        - http.method
        - http.target
      separator: " "
    
    status:
      code: ERROR
      description: "Error detected"

  # Transform 处理器 (高级数据转换)
  transform:
    trace_statements:
      - context: span
        statements:
          # 添加计算字段
          - set(attributes["duration_ms"], EndTime - StartTime)
          # 标准化标签
          - replace_pattern(attributes["http.url"], "\\?.*", "")
    
    metric_statements:
      - context: metric
        statements:
          # 统一指标前缀
          - set(name, Concat(["cgas_", name], ""))

# ============================================
# Exporters (数据导出器)
# ============================================
exporters:
  # Prometheus 导出器 (指标导出)
  prometheus:
    endpoint: 0.0.0.0:8889
    namespace: cgas
    const_labels:
      phase: phase3
      environment: production
    send_timestamps: true
    metric_expiration: 180d
    enable_open_metrics: true

  # Jaeger 导出器 (追踪导出)
  jaeger:
    endpoint: jaeger:14250
    tls:
      insecure: true
    sending_queue:
      enabled: true
      queue_size: 10000
    retry_on_failure:
      enabled: true
      initial_interval: 5s
      max_interval: 30s
      max_elapsed_time: 300s

  # Tempo 导出器 (可选)
  tempo:
    endpoint: tempo:4317
    tls:
      insecure: true
    sending_queue:
      enabled: true
      queue_size: 5000
    retry_on_failure:
      enabled: true

  # Logging 导出器 (调试用)
  logging:
    loglevel: info
    sampling_initial: 5
    sampling_thereafter: 100

  # Debug 导出器 (详细调试)
  debug:
    verbosity: detailed
    sampling_initial: 1
    sampling_thereafter: 10

# ============================================
# Extensions (扩展功能)
# ============================================
extensions:
  # Health Check 扩展
  health_check:
    endpoint: 0.0.0.0:13133
    path: /health
    check_collector_pipeline:
      enabled: true
      interval: 5m
      exporter_failure_threshold: 5
  
  # PProf 扩展 (性能分析)
  pprof:
    endpoint: 0.0.0.0:1777
  
  # ZPages 扩展 (调试页面)
  zpages:
    endpoint: 0.0.0.0:55679
  
  # Prometheus Remote Write 扩展
  prometheus_remote_write:
    endpoint: 0.0.0.0:8888
  
  # Bear Token Auth (可选，生产环境使用)
  bearertokenauth:
    token: ${BEARER_TOKEN}

# ============================================
# Service 配置
# ============================================
service:
  # 启用的扩展
  extensions: [health_check, pprof, zpages]
  
  # 遥测配置
  telemetry:
    logs:
      level: info
      development: false
      encoding: json
      output_paths:
        - stdout
      error_output_paths:
        - stderr
    
    metrics:
      level: detailed
      address: 0.0.0.0:8889
      readers:
        - pull:
            exporter:
              prometheus:
                host: 0.0.0.0
                port: 8889
  
  # 数据管道配置
  pipelines:
    # Traces 管道
    traces:
      receivers: [otlp, jaeger]
      processors: [memory_limiter, batch, resource, attributes, probabilistic_sampler]
      exporters: [jaeger, tempo, logging]
    
    # Metrics 管道
    metrics:
      receivers: [otlp, prometheus]
      processors: [memory_limiter, batch, resource, transform]
      exporters: [prometheus, logging]
    
    # Logs 管道 (预留)
    logs:
      receivers: [otlp]
      processors: [memory_limiter, batch, resource, attributes]
      exporters: [logging]

# ============================================
# 配置验证
# ============================================
# 验证配置命令:
# otelcol-contrib --config=otel-collector-config.yaml --dry-run
```

---

## 4. Prometheus 配置

### 4.1 Prometheus 主配置

```yaml
# prometheus.yaml

global:
  scrape_interval: 15s
  evaluation_interval: 15s
  scrape_timeout: 10s
  external_labels:
    phase: phase3
    environment: production
    cluster: cgas-primary

# Alertmanager 配置
alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
      timeout: 10s
      api_version: v2

# 规则文件
rule_files:
  - /etc/prometheus/alerts.yaml
  - /etc/prometheus/recording_rules.yaml

# 抓取配置
scrape_configs:
  # Prometheus 自监控
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: /metrics
    scheme: http

  # OpenTelemetry Collector 指标
  - job_name: 'otel-collector'
    static_configs:
      - targets: ['otel-collector:8889']
    metrics_path: /metrics
    honor_labels: true
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
        regex: '([^:]+):\d+'
        replacement: '${1}'

  # CGAS Executor 服务
  - job_name: 'cgas-executor'
    metrics_path: /metrics
    scheme: http
    static_configs:
      - targets: ['executor:8080']
        labels:
          service: executor
          team: cgas-core
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
        regex: '([^:]+):\d+'
        replacement: '${1}'
    
    # 服务发现 (可选，Kubernetes 环境)
    # kubernetes_sd_configs:
    #   - role: pod
    #     selectors:
    #       - role: pod
    #         label: app=executor

  # CGAS Verifier 服务
  - job_name: 'cgas-verifier'
    metrics_path: /metrics
    scheme: http
    static_configs:
      - targets: ['verifier:8081']
        labels:
          service: verifier
          team: cgas-core

  # CGAS Batch 服务
  - job_name: 'cgas-batch'
    metrics_path: /metrics
    scheme: http
    static_configs:
      - targets: ['batch:8082']
        labels:
          service: batch
          team: cgas-core

  # CGAS Gateway 服务
  - job_name: 'cgas-gateway'
    metrics_path: /metrics
    scheme: http
    static_configs:
      - targets: ['gateway:8083']
        labels:
          service: gateway
          team: cgas-core

  # Node Exporter (系统指标)
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']
    metrics_path: /metrics

  # Cadvisor (容器指标)
  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']
    metrics_path: /metrics
```

### 4.2 告警规则配置

```yaml
# prometheus-alerts.yaml

groups:
  # ============================================
  # OpenTelemetry Collector 告警
  # ============================================
  - name: otel-collector-alerts
    interval: 30s
    rules:
      # Collector 宕机
      - alert: OtelCollectorDown
        expr: up{job="otel-collector"} == 0
        for: 1m
        labels:
          severity: critical
          component: otel-collector
        annotations:
          summary: "OpenTelemetry Collector 宕机"
          description: "Collector {{ $labels.instance }} 已宕机 {{ $value | humanizeDuration }}"
          runbook_url: "https://wiki.cgas.local/runbooks/otel-collector-down"
      
      # Collector 高延迟
      - alert: OtelCollectorHighLatency
        expr: histogram_quantile(0.99, rate(otelcol_exporter_sent_spans_latency_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
          component: otel-collector
        annotations:
          summary: "Collector 导出延迟过高"
          description: "P99 延迟 {{ $value | humanizeDuration }} 超过阈值 1s"
      
      # Collector 内存过高
      - alert: OtelCollectorHighMemory
        expr: process_resident_memory_bytes{job="otel-collector"} > 1073741824
        for: 5m
        labels:
          severity: warning
          component: otel-collector
        annotations:
          summary: "Collector 内存使用过高"
          description: "内存使用 {{ $value | humanize_bytes }} 超过 1GB"
      
      # Collector 导出失败
      - alert: OtelCollectorExportFailures
        expr: rate(otelcol_exporter_failed_spans[5m]) > 10
        for: 5m
        labels:
          severity: warning
          component: otel-collector
        annotations:
          summary: "Collector 导出失败率高"
          description: "失败率 {{ $value | humanize }} spans/s"

  # ============================================
  # 服务可用性告警
  # ============================================
  - name: service-availability-alerts
    interval: 30s
    rules:
      # 服务宕机
      - alert: ServiceDown
        expr: up{job=~"cgas-.*"} == 0
        for: 1m
        labels:
          severity: critical
          component: service
        annotations:
          summary: "CGAS 服务 {{ $labels.service }} 宕机"
          description: "服务 {{ $labels.service }} ({{ $labels.instance }}) 已宕机 {{ $value | humanizeDuration }}"
      
      # 服务高错误率
      - alert: ServiceHighErrorRate
        expr: rate(http_requests_total{job=~"cgas-.*",status=~"5.."}[5m]) / rate(http_requests_total{job=~"cgas-.*"}[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
          component: service
        annotations:
          summary: "服务 {{ $labels.service }} 错误率过高"
          description: "错误率 {{ $value | humanizePercentage }} 超过 5%"
      
      # 服务高延迟
      - alert: ServiceHighLatency
        expr: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{job=~"cgas-.*"}[5m])) > 0.5
        for: 5m
        labels:
          severity: warning
          component: service
        annotations:
          summary: "服务 {{ $labels.service }} 延迟过高"
          description: "P99 延迟 {{ $value | humanizeDuration }} 超过 500ms"

  # ============================================
  # 系统资源告警
  # ============================================
  - name: system-resource-alerts
    interval: 30s
    rules:
      # CPU 使用率过高
      - alert: HighCPUUsage
        expr: 100 - (avg by(instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80
        for: 10m
        labels:
          severity: warning
          component: system
        annotations:
          summary: "主机 {{ $labels.instance }} CPU 使用率过高"
          description: "CPU 使用率 {{ $value | humanizePercentage }} 超过 80%"
      
      # 内存使用率过高
      - alert: HighMemoryUsage
        expr: (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100 > 85
        for: 10m
        labels:
          severity: warning
          component: system
        annotations:
          summary: "主机 {{ $labels.instance }} 内存使用率过高"
          description: "内存使用率 {{ $value | humanizePercentage }} 超过 85%"
      
      # 磁盘空间不足
      - alert: LowDiskSpace
        expr: (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"}) * 100 < 15
        for: 10m
        labels:
          severity: critical
          component: system
        annotations:
          summary: "主机 {{ $labels.instance }} 磁盘空间不足"
          description: "可用磁盘空间 {{ $value | humanizePercentage }}"
```

---

## 5. 部署与验证

### 5.1 部署步骤

```bash
#!/bin/bash
# deploy-observability.sh

set -e

echo "=========================================="
echo "Phase 3 Observability Stack Deployment"
echo "=========================================="

# 1. 创建必要目录
echo "[1/6] Creating directories..."
mkdir -p otel-collector-data
mkdir -p prometheus-data
mkdir -p jaeger-data
mkdir -p tempo-data
mkdir -p grafana-data
mkdir -p alertmanager-data
mkdir -p grafana/provisioning/datasources
mkdir -p grafana/provisioning/dashboards
mkdir -p grafana/dashboards

# 2. 设置权限
echo "[2/6] Setting permissions..."
chmod -R 755 otel-collector-data prometheus-data jaeger-data tempo-data grafana-data alertmanager-data

# 3. 环境变量配置
echo "[3/6] Configuring environment..."
export GRAFANA_ADMIN_PASSWORD="${GRAFANA_ADMIN_PASSWORD:-admin}"
export BEARER_TOKEN="${BEARER_TOKEN:-}"

# 4. 启动服务
echo "[4/6] Starting services..."
docker-compose -f docker-compose.observability.yaml up -d

# 5. 等待服务就绪
echo "[5/6] Waiting for services to be ready..."
echo "  - Waiting for Prometheus..."
until curl -s http://localhost:9090/-/healthy > /dev/null; do
  sleep 2
done
echo "  ✓ Prometheus is ready"

echo "  - Waiting for Jaeger..."
until curl -s http://localhost:16686 > /dev/null; do
  sleep 2
done
echo "  ✓ Jaeger is ready"

echo "  - Waiting for Grafana..."
until curl -s http://localhost:3000/api/health > /dev/null; do
  sleep 2
done
echo "  ✓ Grafana is ready"

echo "  - Waiting for OTEL Collector..."
until curl -s http://localhost:13133 > /dev/null; do
  sleep 2
done
echo "  ✓ OTEL Collector is ready"

# 6. 健康检查
echo "[6/6] Running health checks..."
echo ""
echo "Service Status:"
docker-compose -f docker-compose.observability.yaml ps

echo ""
echo "=========================================="
echo "✅ Deployment Complete!"
echo "=========================================="
echo ""
echo "Access URLs:"
echo "  - Grafana:      http://localhost:3000 (admin/admin)"
echo "  - Prometheus:   http://localhost:9090"
echo "  - Jaeger:       http://localhost:16686"
echo "  - OTEL Collector Health: http://localhost:13133"
echo ""
echo "Next Steps:"
echo "  1. Configure datasources in Grafana"
echo "  2. Import dashboards from grafana/dashboards/"
echo "  3. Verify metrics in Prometheus"
echo "  4. Test trace ingestion"
echo ""
```

### 5.2 验证脚本

```bash
#!/bin/bash
# verify-observability.sh

set -e

echo "=========================================="
echo "Phase 3 Observability Verification"
echo "=========================================="

# 1. 检查容器状态
echo "[1/5] Checking container status..."
docker-compose -f docker-compose.observability.yaml ps

# 2. 验证 Prometheus
echo ""
echo "[2/5] Verifying Prometheus..."
PROMETHEUS_HEALTH=$(curl -s http://localhost:9090/-/healthy)
if [ "$PROMETHEUS_HEALTH" = "Prometheus Server is Healthy." ]; then
  echo "  ✓ Prometheus is healthy"
else
  echo "  ✗ Prometheus health check failed"
  exit 1
fi

# 检查 targets
PROMETHEUS_TARGETS=$(curl -s "http://localhost:9090/api/v1/targets" | jq '.data.activeTargets | length')
echo "  ✓ Active targets: $PROMETHEUS_TARGETS"

# 3. 验证 Jaeger
echo ""
echo "[3/5] Verifying Jaeger..."
JAEGER_HEALTH=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:16686)
if [ "$JAEGER_HEALTH" = "200" ]; then
  echo "  ✓ Jaeger is healthy"
else
  echo "  ✗ Jaeger health check failed (HTTP $JAEGER_HEALTH)"
  exit 1
fi

# 4. 验证 OTEL Collector
echo ""
echo "[4/5] Verifying OTEL Collector..."
OTEL_HEALTH=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:13133)
if [ "$OTEL_HEALTH" = "200" ]; then
  echo "  ✓ OTEL Collector is healthy"
else
  echo "  ✗ OTEL Collector health check failed (HTTP $OTEL_HEALTH)"
  exit 1
fi

# 检查 Collector 指标
OTEL_METRICS=$(curl -s http://localhost:8889/metrics | grep -c "otelcol")
echo "  ✓ OTEL metrics available: $OTEL_METRICS metrics"

# 5. 验证 Grafana
echo ""
echo "[5/5] Verifying Grafana..."
GRAFANA_HEALTH=$(curl -s http://localhost:3000/api/health | jq -r '.commit')
if [ -n "$GRAFANA_HEALTH" ]; then
  echo "  ✓ Grafana is healthy (commit: $GRAFANA_HEALTH)"
else
  echo "  ✗ Grafana health check failed"
  exit 1
fi

# 检查 datasources
GRAFANA_DATASOURCES=$(curl -s -u admin:admin http://localhost:3000/api/datasources | jq 'length')
echo "  ✓ Configured datasources: $GRAFANA_DATASOURCES"

echo ""
echo "=========================================="
echo "✅ All Verifications Passed!"
echo "=========================================="
```

---

## 6. 运维指南

### 6.1 日常运维命令

```bash
# 查看服务状态
docker-compose -f docker-compose.observability.yaml ps

# 查看日志
docker-compose -f docker-compose.observability.yaml logs -f otel-collector
docker-compose -f docker-compose.observability.yaml logs -f prometheus
docker-compose -f docker-compose.observability.yaml logs -f jaeger

# 重启服务
docker-compose -f docker-compose.observability.yaml restart otel-collector

# 停止服务
docker-compose -f docker-compose.observability.yaml down

# 清理数据 (谨慎使用!)
docker-compose -f docker-compose.observability.yaml down -v

# 更新配置 (Prometheus 热重载)
curl -X POST http://localhost:9090/-/reload

# 备份数据
docker run --rm -v observability_prometheus-data:/data -v $(pwd):/backup alpine tar czf /backup/prometheus-backup.tar.gz /data
```

### 6.2 故障排查

```bash
# 检查 Collector 配置
docker-compose -f docker-compose.observability.yaml exec otel-collector otelcol-contrib --config=/etc/otel-collector-config.yaml --dry-run

# 查看 Collector 详细日志
docker-compose -f docker-compose.observability.yaml logs --tail=100 otel-collector

# 检查 Prometheus targets
curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | select(.health!="up")'

# 测试 OTLP 端点
grpcurl -plaintext -d '{"resource_spans":[]}' localhost:4317 opentelemetry.proto.collector.trace.v1.TraceService/Export

# 检查内存使用
docker stats otel-collector prometheus jaeger grafana --no-stream
```

---

## 7. 性能调优

### 7.1 Collector 调优参数

```yaml
# 生产环境推荐配置
processors:
  batch:
    timeout: 5s              # 批量超时
    send_batch_size: 512     # 批量大小
    send_batch_max_size: 1024 # 最大批量
  
  memory_limiter:
    limit_mib: 1000          # 内存限制 (根据容器限制调整)
    spike_limit_mib: 200     # 峰值缓冲
  
  probabilistic_sampler:
    sampling_percentage: 10  # 采样率 (根据流量调整)
```

### 7.2 Prometheus 调优

```yaml
# prometheus.yaml 优化
global:
  scrape_interval: 15s       # 抓取间隔 (可调整至 30s 降低负载)
  scrape_timeout: 10s        # 抓取超时
  
# 存储优化
storage.tsdb.retention.time: 180d  # 数据保留时间
storage.tsdb.retention.size: 50GB  # 数据保留大小

# 查询优化
query.max-concurrency: 20    # 最大并发查询
query.timeout: 2m            # 查询超时
```

---

## 8. 安全配置

### 8.1 网络安全

```yaml
# docker-compose.observability.yaml (网络隔离)
networks:
  observability:
    driver: bridge
    internal: false  # 生产环境建议设为 true，通过反向代理访问
    ipam:
      config:
        - subnet: 172.28.0.0/16
```

### 8.2 认证配置

```yaml
# Grafana 认证
environment:
  - GF_AUTH_ANONYMOUS_ENABLED=false
  - GF_AUTH_BASIC_ENABLED=true
  - GF_AUTH_OAUTH_AUTO_LOGIN=false

# Prometheus 认证 (通过 Nginx 反向代理)
# nginx.conf:
# location / {
#   auth_basic "Prometheus";
#   auth_basic_user_file /etc/nginx/.htpasswd;
#   proxy_pass http://prometheus:9090;
# }
```

---

## 9. 交付清单

### 9.1 交付文件

| 文件名 | 用途 | 状态 |
|---|---|---|
| docker-compose.observability.yaml | 主部署文件 | ✅ 完成 |
| otel-collector-config.yaml | Collector 配置 | ✅ 完成 |
| prometheus.yaml | Prometheus 配置 | ✅ 完成 |
| prometheus-alerts.yaml | 告警规则 | ✅ 完成 |
| tempo.yaml | Tempo 配置 | ✅ 完成 |
| alertmanager.yaml | Alertmanager 配置 | ✅ 完成 |
| deploy-observability.sh | 部署脚本 | ✅ 完成 |
| verify-observability.sh | 验证脚本 | ✅ 完成 |

### 9.2 验收标准

| 验收项 | 标准 | 验证方法 | 状态 |
|---|---|---|---|
| Collector 健康 | HTTP 200 | curl localhost:13133 | ✅ |
| Prometheus 健康 | 返回 healthy | curl localhost:9090/-/healthy | ✅ |
| Jaeger 健康 | HTTP 200 | curl localhost:16686 | ✅ |
| Grafana 健康 | API 正常 | curl localhost:3000/api/health | ✅ |
| 指标采集 | 所有 targets UP | Prometheus UI | ✅ |
| 追踪采集 | Traces 可查询 | Jaeger UI | ✅ |
| 告警规则 | 规则加载成功 | Prometheus Rules UI | ✅ |

---

## 10. 附录

### 10.1 端口汇总

| 服务 | 端口 | 协议 | 用途 |
|---|---|---|---|
| OTEL Collector | 4317 | gRPC | OTLP 追踪接收 |
| OTEL Collector | 4318 | HTTP | OTLP 追踪/指标接收 |
| OTEL Collector | 8889 | HTTP | Prometheus 导出 |
| OTEL Collector | 13133 | HTTP | 健康检查 |
| Prometheus | 9090 | HTTP | Web UI + API |
| Jaeger | 16686 | HTTP | Web UI |
| Jaeger | 4317 | gRPC | OTLP 接收 (备用) |
| Jaeger | 14250 | gRPC | Span 接收 |
| Tempo | 3200 | HTTP | API |
| Grafana | 3000 | HTTP | Web UI |
| Alertmanager | 9093 | HTTP | Web UI + API |

### 10.2 参考文档

- [OpenTelemetry Collector 官方文档](https://opentelemetry.io/docs/collector/)
- [Prometheus 官方文档](https://prometheus.io/docs/)
- [Jaeger 官方文档](https://www.jaegertracing.io/docs/)
- [Grafana 官方文档](https://grafana.com/docs/)

---

**文档状态**: ✅ 已完成  
**创建日期**: 2026-03-07  
**责任人**: SRE-Agent + Observability-Agent  
**保管**: 项目文档库
