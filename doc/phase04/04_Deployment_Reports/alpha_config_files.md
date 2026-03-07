# Alpha 环境配置文件

**版本**: v1.0  
**日期**: 2026-03-07  
**责任人**: Dev-Agent  
**环境**: Alpha (生产验证环境)  
**状态**: 📋 待部署  

---

## 📋 目录

1. [应用配置文件](#1-应用配置文件)
2. [数据库配置](#2-数据库配置)
3. [缓存配置](#3-缓存配置)
4. [消息队列配置](#4-消息队列配置)
5. [安全配置](#5-安全配置)
6. [监控配置](#6-监控配置)
7. [日志配置](#7-日志配置)
8. [Kubernetes 资源配置](#8-kubernetes-资源配置)

---

## 1. 应用配置文件

### 1.1 主配置文件 `application.yaml`

```yaml
# CGAS Workflow Engine - Alpha 环境配置
# Phase 4 Week 1 使用

app:
  name: cgas-workflow-engine
  environment: alpha
  version: phase4-alpha-v1.0
  release_id: release-2026-04-01-phase4-alpha

server:
  host: 0.0.0.0
  port: 8080
  workers: 4
  keep_alive: 75
  timeout:
    connect: 30s
    read: 60s
    write: 60s

# 执行器配置
executor:
  max_concurrent: 100
  queue_size: 1000
  timeout_ms: 30000
  retry:
    max_attempts: 3
    initial_delay_ms: 100
    max_delay_ms: 5000
    multiplier: 2.0

# 验证器配置
validator:
  max_concurrent: 100
  queue_size: 1000
  timeout_ms: 30000
  cache:
    enabled: true
    max_size: 10000
    ttl_seconds: 300

# Batch 配置
batch:
  max_size: 100
  max_nested_depth: 5
  atomic: true
  timeout_ms: 60000
  
# Transaction 配置
transaction:
  isolation_level: read_committed
  max_duration_ms: 120000
  deadlock_detection: true
  timeout_ms: 90000

# 性能优化配置
optimization:
  async_pool:
    enabled: true
    core_size: 20
    max_size: 100
    keep_alive_seconds: 60
    queue_capacity: 1000
  
  incremental_replay:
    enabled: true
    checkpoint_interval_ms: 1000
  
  validation_cache:
    enabled: true
    max_entries: 10000
    expire_after_write_seconds: 300
  
  object_pool:
    enabled: true
    max_objects: 1000
    min_idle: 10
    max_idle: 100
```

---

## 2. 数据库配置

### 2.1 PostgreSQL 配置 `database.yaml`

```yaml
# CGAS Workflow Engine - 数据库配置
# Alpha 环境

database:
  type: postgresql
  host: cgas-postgres.alpha.svc.cluster.local
  port: 5432
  name: cgas_alpha
  schema: public
  
  # 连接池配置
  pool:
    min_connections: 10
    max_connections: 100
    connection_timeout_ms: 5000
    idle_timeout_ms: 600000
    max_lifetime_ms: 1800000
  
  # SSL 配置
  ssl:
    enabled: false
    mode: prefer
    ca_cert: /etc/ssl/certs/ca-certificates.crt
  
  # 读写分离 (Alpha 环境暂不启用)
  replication:
    enabled: false
    read_replicas: []
  
  # 迁移配置
  migration:
    enabled: true
    table: schema_migrations
    lock_timeout_ms: 30000

# 查询配置
query:
  default_limit: 100
  max_limit: 1000
  slow_query_threshold_ms: 1000
  log_slow_queries: true

# 备份配置
backup:
  enabled: true
  schedule: "0 2 * * *"  # 每天凌晨 2 点
  retention_days: 7
  storage_path: /var/backups/postgres
```

### 2.2 数据库初始化 SQL `001_initial_schema.sql`

```sql
-- CGAS Workflow Engine - Alpha 环境数据库初始化
-- Phase 4 Week 1

-- 启用扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- 工作流执行表
CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    trace_id VARCHAR(128) NOT NULL,
    execution_id VARCHAR(128) NOT NULL,
    workflow_id VARCHAR(128),
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    
    CONSTRAINT unique_trace_execution UNIQUE (trace_id, execution_id)
);

CREATE INDEX idx_workflow_executions_trace_id ON workflow_executions(trace_id);
CREATE INDEX idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX idx_workflow_executions_created_at ON workflow_executions(created_at);

-- 指令表
CREATE TABLE IF NOT EXISTS instructions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    instruction_type VARCHAR(64) NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    sequence_number INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    executed_at TIMESTAMPTZ,
    result JSONB,
    error_message TEXT,
    
    CONSTRAINT unique_execution_sequence UNIQUE (execution_id, sequence_number)
);

CREATE INDEX idx_instructions_execution_id ON instructions(execution_id);
CREATE INDEX idx_instructions_status ON instructions(status);
CREATE INDEX idx_instructions_type ON instructions(instruction_type);

-- Batch 记录表
CREATE TABLE IF NOT EXISTS batch_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_id VARCHAR(128) NOT NULL,
    trace_id VARCHAR(128) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    total_instructions INTEGER NOT NULL,
    completed_instructions INTEGER NOT NULL DEFAULT 0,
    atomic BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    
    CONSTRAINT unique_batch_id UNIQUE (batch_id)
);

CREATE INDEX idx_batch_records_batch_id ON batch_records(batch_id);
CREATE INDEX idx_batch_records_status ON batch_records(status);

-- Transaction 日志表
CREATE TABLE IF NOT EXISTS transaction_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_id VARCHAR(128) NOT NULL,
    execution_ids UUID[] NOT NULL,
    isolation_level VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    committed_at TIMESTAMPTZ,
    rolled_back_at TIMESTAMPTZ,
    error_message TEXT,
    
    CONSTRAINT unique_transaction_id UNIQUE (transaction_id)
);

CREATE INDEX idx_transaction_logs_transaction_id ON transaction_logs(transaction_id);
CREATE INDEX idx_transaction_logs_status ON transaction_logs(status);

-- 安全事件表
CREATE TABLE IF NOT EXISTS security_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type VARCHAR(64) NOT NULL,
    severity VARCHAR(32) NOT NULL,
    trace_id VARCHAR(128),
    execution_id UUID,
    details JSONB NOT NULL DEFAULT '{}',
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed BOOLEAN NOT NULL DEFAULT false,
    processed_at TIMESTAMPTZ,
    
    CONSTRAINT valid_severity CHECK (severity IN ('low', 'medium', 'high', 'critical'))
);

CREATE INDEX idx_security_events_type ON security_events(event_type);
CREATE INDEX idx_security_events_severity ON security_events(severity);
CREATE INDEX idx_security_events_detected_at ON security_events(detected_at);
CREATE INDEX idx_security_events_processed ON security_events(processed);

-- 监控指标表
CREATE TABLE IF NOT EXISTS metrics_data (
    id BIGSERIAL PRIMARY KEY,
    metric_name VARCHAR(128) NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    labels JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_metrics_data_name ON metrics_data(metric_name);
CREATE INDEX idx_metrics_data_timestamp ON metrics_data(timestamp);
CREATE INDEX idx_metrics_data_labels ON metrics_data USING GIN(labels);

-- 性能优化：分区表 (按月分区)
CREATE TABLE IF NOT EXISTS instructions_archive (
    LIKE instructions INCLUDING ALL
) PARTITION BY RANGE (created_at);

-- 审计日志表
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    action VARCHAR(64) NOT NULL,
    resource_type VARCHAR(64),
    resource_id UUID,
    user_id VARCHAR(128),
    details JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
```

---

## 3. 缓存配置

### 3.1 Redis 配置 `cache.yaml`

```yaml
# CGAS Workflow Engine - 缓存配置
# Alpha 环境

cache:
  type: redis
  host: cgas-redis.alpha.svc.cluster.local
  port: 6379
  database: 0
  password: ${REDIS_PASSWORD}  # 从 Secret 读取
  
  # 连接池配置
  pool:
    min_idle: 10
    max_active: 100
    max_idle: 50
    connection_timeout_ms: 5000
    read_timeout_ms: 3000
    write_timeout_ms: 3000
  
  # TLS 配置
  tls:
    enabled: false
  
  # 集群配置 (Alpha 环境暂不启用)
  cluster:
    enabled: false
    nodes: []

# 缓存策略
cache_policies:
  # 验证结果缓存
  validation_result:
    enabled: true
    ttl_seconds: 300
    max_entries: 10000
    eviction_policy: lru
  
  # 执行结果缓存
  execution_result:
    enabled: true
    ttl_seconds: 3600
    max_entries: 5000
    eviction_policy: lru
  
  # 配置缓存
  config:
    enabled: true
    ttl_seconds: 600
    max_entries: 1000
    eviction_policy: lru
  
  # 会话缓存
  session:
    enabled: true
    ttl_seconds: 1800
    max_entries: 2000
    eviction_policy: lru

# 缓存监控
monitoring:
  enabled: true
  collect_stats: true
  stats_interval_seconds: 60
```

---

## 4. 消息队列配置

### 4.1 Kafka 配置 `messaging.yaml`

```yaml
# CGAS Workflow Engine - 消息队列配置
# Alpha 环境

messaging:
  type: kafka
  brokers:
    - cgas-kafka-0.cgas-kafka.alpha.svc.cluster.local:9092
    - cgas-kafka-1.cgas-kafka.alpha.svc.cluster.local:9092
    - cgas-kafka-2.cgas-kafka.alpha.svc.cluster.local:9092
  
  # 生产者配置
  producer:
    acks: all
    retries: 3
    batch_size: 16384
    linger_ms: 1
    buffer_memory: 33554432
    compression_type: snappy
    max_in_flight_requests: 5
  
  # 消费者配置
  consumer:
    group_id: cgas-workflow-engine-alpha
    auto_offset_reset: latest
    enable_auto_commit: false
    max_poll_records: 500
    session_timeout_ms: 30000
    heartbeat_interval_ms: 10000
    max_poll_interval_ms: 300000
  
  # TLS 配置
  security:
    protocol: PLAINTEXT
    ssl_enabled: false

# Topic 配置
topics:
  workflow_events:
    name: cgas.workflow.events
    partitions: 12
    replication_factor: 3
    retention_ms: 604800000  # 7 天
  
  execution_results:
    name: cgas.execution.results
    partitions: 12
    replication_factor: 3
    retention_ms: 259200000  # 3 天
  
  security_events:
    name: cgas.security.events
    partitions: 6
    replication_factor: 3
    retention_ms: 2592000000  # 30 天
  
  audit_logs:
    name: cgas.audit.logs
    partitions: 6
    replication_factor: 3
    retention_ms: 7776000000  # 90 天

# 事件处理配置
event_handling:
  max_concurrent_consumers: 10
  poll_timeout_ms: 100
  commit_interval_ms: 5000
  error_handling:
    max_retries: 3
    retry_backoff_ms: 1000
    dead_letter_queue: cgas.workflow.dlq
```

---

## 5. 安全配置

### 5.1 安全闸门配置 `security_gates.yaml`

```yaml
# CGAS Workflow Engine - 安全闸门配置
# Alpha 环境

security_gates:
  # SG-1: 输入验证
  sg1_input_validation:
    enabled: true
    strict_mode: true
    max_payload_size_bytes: 1048576  # 1MB
    allowed_instruction_types:
      - READ
      - WRITE
      - UPDATE
      - DELETE
      - COMPUTE
      - BATCH
      - TRANSACTION
    validate_schema: true
    reject_unknown_fields: true
  
  # SG-2: 身份验证
  sg2_authentication:
    enabled: true
    provider: oidc
    oidc:
      issuer: https://auth.cgas.internal
      client_id: cgas-workflow-engine-alpha
      client_secret: ${OIDC_CLIENT_SECRET}
      redirect_uri: https://alpha.cgas.internal/callback
      scopes:
        - openid
        - profile
        - email
        - workflow:execute
        - workflow:read
    token_validation:
      verify_signature: true
      verify_expiration: true
      verify_issuer: true
      clock_skew_seconds: 60
  
  # SG-3: 授权检查
  sg3_authorization:
    enabled: true
    provider: opa
    opa:
      url: http://cgas-opa.alpha.svc.cluster.local:8181
      policy_path: v1/data/cgas/authorization
      timeout_ms: 5000
    default_action: deny
    cache:
      enabled: true
      ttl_seconds: 300
  
  # SG-4: 威胁检测
  sg4_threat_detection:
    enabled: true
    detection_rules:
      - name: sql_injection
        enabled: true
        severity: critical
      - name: xss_attack
        enabled: true
        severity: high
      - name: command_injection
        enabled: true
        severity: critical
      - name: path_traversal
        enabled: true
        severity: high
      - name: rate_limiting
        enabled: true
        severity: medium
        max_requests_per_minute: 1000
      - name: anomaly_detection
        enabled: true
        severity: medium
        baseline_window_hours: 24
    alerting:
      enabled: true
      channels:
        - type: webhook
          url: ${SECURITY_ALERT_WEBHOOK}
        - type: email
          recipients:
            - security@cgas.internal

# 审计配置
audit:
  enabled: true
  log_level: info
  include_request_body: false
  include_response_body: false
  mask_sensitive_fields: true
  sensitive_fields:
    - password
    - token
    - secret
    - api_key
    - authorization
  storage:
    type: database
    retention_days: 90

# 速率限制配置
rate_limiting:
  enabled: true
  strategy: sliding_window
  default_limits:
    requests_per_second: 100
    requests_per_minute: 1000
    requests_per_hour: 10000
  per_user_limits:
    requests_per_second: 20
    requests_per_minute: 200
    requests_per_hour: 2000
  burst_allowance: 50
```

### 5.2 OIDC 配置 `oidc_config.json`

```json
{
  "issuer": "https://auth.cgas.internal",
  "authorization_endpoint": "https://auth.cgas.internal/oauth2/authorize",
  "token_endpoint": "https://auth.cgas.internal/oauth2/token",
  "userinfo_endpoint": "https://auth.cgas.internal/oauth2/userinfo",
  "jwks_uri": "https://auth.cgas.internal/.well-known/jwks.json",
  "scopes_supported": [
    "openid",
    "profile",
    "email",
    "workflow:execute",
    "workflow:read",
    "workflow:admin"
  ],
  "response_types_supported": [
    "code",
    "token",
    "id_token"
  ],
  "grant_types_supported": [
    "authorization_code",
    "client_credentials",
    "refresh_token"
  ],
  "token_endpoint_auth_methods_supported": [
    "client_secret_basic",
    "client_secret_post"
  ],
  "id_token_signing_alg_values_supported": [
    "RS256",
    "RS384",
    "RS512"
  ]
}
```

---

## 6. 监控配置

### 6.1 Prometheus 配置 `prometheus.yaml`

```yaml
# CGAS Workflow Engine - Prometheus 监控配置
# Alpha 环境

global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    environment: alpha
    cluster: cgas
    application: workflow-engine

scrape_configs:
  # 应用指标
  - job_name: 'cgas-workflow-engine'
    static_configs:
      - targets: ['cgas-workflow-engine-alpha:8080']
    metrics_path: /metrics
    scrape_interval: 10s
    
  # 数据库指标
  - job_name: 'postgres'
    static_configs:
      - targets: ['cgas-postgres-exporter:9187']
    
  # Redis 指标
  - job_name: 'redis'
    static_configs:
      - targets: ['cgas-redis-exporter:9121']
    
  # Kafka 指标
  - job_name: 'kafka'
    static_configs:
      - targets: ['cgas-kafka-exporter:9308']
    
  # Node 指标
  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']

# 告警规则
rule_files:
  - /etc/prometheus/rules/*.yaml

# Alertmanager 配置
alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - cgas-alertmanager:9093
```

### 6.2 监控指标配置 `metrics_config.yaml`

```yaml
# CGAS Workflow Engine - 监控指标配置
# Alpha 环境 (56 个核心指标)

metrics:
  # 性能指标 (14 个)
  performance:
    - name: execution_latency_seconds
      type: histogram
      buckets: [0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0]
      labels: [instruction_type, status]
      
    - name: validation_latency_seconds
      type: histogram
      buckets: [0.05, 0.1, 0.2, 0.5, 1.0]
      labels: [validator_type]
      
    - name: batch_execution_latency_seconds
      type: histogram
      buckets: [0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
      labels: [batch_size_range]
      
    - name: transaction_duration_seconds
      type: histogram
      buckets: [0.5, 1.0, 2.0, 5.0, 10.0, 30.0]
      labels: [isolation_level, status]
      
    - name: throughput_per_second
      type: gauge
      labels: [operation_type]
      
    - name: queue_depth
      type: gauge
      labels: [queue_name]
      
    - name: active_connections
      type: gauge
      labels: [connection_type]
      
    - name: cache_hit_ratio
      type: gauge
      labels: [cache_name]
      
    - name: error_rate
      type: gauge
      labels: [error_type, severity]
      
    - name: retry_count
      type: counter
      labels: [operation_type, result]
      
    - name: timeout_count
      type: counter
      labels: [operation_type]
      
    - name: cpu_usage_percent
      type: gauge
      labels: [pod_name]
      
    - name: memory_usage_bytes
      type: gauge
      labels: [pod_name]
      
    - name: gc_pause_seconds
      type: histogram
      buckets: [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]
      labels: [gc_type]

  # 质量指标 (12 个)
  quality:
    - name: e2e_success_rate
      type: gauge
      labels: [workflow_type]
      
    - name: instruction_success_rate
      type: gauge
      labels: [instruction_type]
      
    - name: batch_success_rate
      type: gauge
      labels: [atomic]
      
    - name: transaction_success_rate
      type: gauge
      labels: [isolation_level]
      
    - name: consistency_check_failures
      type: counter
      labels: [check_type]
      
    - name: replay_consistency_rate
      type: gauge
      labels: [scenario_type]
      
    - name: false_positive_rate
      type: gauge
      labels: [detection_type]
      
    - name: defect_escape_count
      type: counter
      labels: [severity, phase]
      
    - name: code_review_coverage
      type: gauge
      labels: [component]
      
    - name: test_coverage
      type: gauge
      labels: [test_type]
      
    - name: schema_validation_errors
      type: counter
      labels: [field_type]
      
    - name: data_integrity_violations
      type: counter
      labels: [violation_type]

  # 安全指标 (14 个)
  security:
    - name: authentication_attempts
      type: counter
      labels: [result, method]
      
    - name: authorization_decisions
      type: counter
      labels: [action, result]
      
    - name: security_events_detected
      type: counter
      labels: [event_type, severity]
      
    - name: threat_detection_accuracy
      type: gauge
      labels: [threat_type]
      
    - name: rate_limit_violations
      type: counter
      labels: [limit_type]
      
    - name: invalid_input_attempts
      type: counter
      labels: [validation_rule]
      
    - name: token_validation_failures
      type: counter
      labels: [failure_reason]
      
    - name: policy_evaluation_errors
      type: counter
      labels: [policy_name]
      
    - name: suspicious_activity_count
      type: counter
      labels: [activity_type]
      
    - name: vulnerability_scan_results
      type: gauge
      labels: [severity]
      
    - name: security_patch_level
      type: gauge
      labels: [component]
      
    - name: encryption_status
      type: gauge
      labels: [data_type, location]
      
    - name: audit_log_coverage
      type: gauge
      labels: [event_category]
      
    - name: security_gate_pass_rate
      type: gauge
      labels: [gate_id]

  # 资源指标 (8 个)
  resources:
    - name: pod_cpu_usage
      type: gauge
      labels: [pod_name, namespace]
      
    - name: pod_memory_usage
      type: gauge
      labels: [pod_name, namespace]
      
    - name: pod_restart_count
      type: counter
      labels: [pod_name, reason]
      
    - name: disk_usage_bytes
      type: gauge
      labels: [mount_point]
      
    - name: network_bytes_transmitted
      type: counter
      labels: [interface, direction]
      
    - name: network_packet_errors
      type: counter
      labels: [interface, error_type]
      
    - name: database_connections
      type: gauge
      labels: [state]
      
    - name: database_query_duration
      type: histogram
      buckets: [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
      labels: [query_type]

  # 业务指标 (8 个)
  business:
    - name: workflow_executions_total
      type: counter
      labels: [workflow_type, status]
      
    - name: instructions_processed
      type: counter
      labels: [instruction_type, status]
      
    - name: batches_processed
      type: counter
      labels: [atomic, status]
      
    - name: transactions_completed
      type: counter
      labels: [outcome]
      
    - name: user_sessions_active
      type: gauge
      labels: [user_type]
      
    - name: api_requests_total
      type: counter
      labels: [endpoint, method, status_code]
      
    - name: sla_compliance_rate
      type: gauge
      labels: [sla_type]
      
    - name: business_transaction_value
      type: histogram
      buckets: [1, 10, 100, 1000, 10000, 100000]
      labels: [transaction_type]
```

---

## 7. 日志配置

### 7.1 日志配置 `logging.yaml`

```yaml
# CGAS Workflow Engine - 日志配置
# Alpha 环境

logging:
  # 基础配置
  level: info
  format: json
  output:
    - type: stdout
      format: json
    - type: file
      path: /var/log/cgas/workflow-engine.log
      format: json
      rotate:
        max_size_mb: 100
        max_backups: 10
        max_age_days: 7
        compress: true
  
  # 结构化日志字段
  fields:
    - timestamp
    - level
    - message
    - trace_id
    - execution_id
    - user_id
    - component
    - operation
    - duration_ms
    - status
  
  # 日志采样
  sampling:
    enabled: true
    rate: 0.1  # 10% 采样率
    debug_rate: 1.0  # Debug 级别 100% 采样
  
  # 敏感信息脱敏
  redaction:
    enabled: true
    patterns:
      - pattern: "password[\"']?\\s*[:=]\\s*[\"']?[^\"',\\s]+"
        replacement: "password=***REDACTED***"
      - pattern: "token[\"']?\\s*[:=]\\s*[\"']?[^\"',\\s]+"
        replacement: "token=***REDACTED***"
      - pattern: "api_key[\"']?\\s*[:=]\\s*[\"']?[^\"',\\s]+"
        replacement: "api_key=***REDACTED***"
      - pattern: "\\b\\d{16}\\b"  # 信用卡号
        replacement: "***REDACTED***"
  
  # 日志级别覆盖
  overrides:
    components:
      database: debug
      security: debug
      executor: info
      validator: info
    operations:
      health_check: warn
      metrics_collection: warn

# 分布式追踪配置
tracing:
  enabled: true
  provider: opentelemetry
  sampling_rate: 0.1  # 10% 采样
  propagation_format: w3c
  exporters:
    - type: otlp
      endpoint: http://cgas-tempo.alpha.svc.cluster.local:4317
      protocol: grpc
  
  # Span 配置
  span:
    include_attributes:
      - http.method
      - http.url
      - http.status_code
      - db.statement
      - db.operation
      - rpc.service
      - rpc.method
    max_attributes: 128
    max_events: 128
    max_links: 128

# 日志聚合配置
aggregation:
  enabled: true
  Loki:
    url: http://cgas-loki.alpha.svc.cluster.local:3100
    labels:
      - app
      - environment
      - level
      - component
    batch_size: 100
    batch_timeout_ms: 5000
```

---

## 8. Kubernetes 资源配置

### 8.1 Deployment 配置 `deployment.yaml`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cgas-workflow-engine
  namespace: alpha
  labels:
    app: cgas-workflow-engine
    version: phase4-alpha-v1.0
    environment: alpha
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: cgas-workflow-engine
  template:
    metadata:
      labels:
        app: cgas-workflow-engine
        version: phase4-alpha-v1.0
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: cgas-workflow-engine
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchLabels:
                  app: cgas-workflow-engine
              topologyKey: kubernetes.io/hostname
      containers:
      - name: workflow-engine
        image: registry.cgas.internal/cgas/workflow-engine:phase4-alpha-v1.0
        imagePullPolicy: Always
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        - containerPort: 8081
          name: metrics
          protocol: TCP
        env:
        - name: RUST_LOG
          value: "info"
        - name: APP_ENVIRONMENT
          value: "alpha"
        - name: DB_HOST
          valueFrom:
            secretKeyRef:
              name: cgas-db-credentials
              key: DB_HOST
        - name: DB_PASSWORD
          valueFrom:
            secretKeyRef:
              name: cgas-db-credentials
              key: DB_PASSWORD
        - name: REDIS_HOST
          value: "cgas-redis.alpha.svc.cluster.local"
        - name: KAFKA_BROKERS
          value: "cgas-kafka-0.cgas-kafka.alpha.svc.cluster.local:9092,cgas-kafka-1.cgas-kafka.alpha.svc.cluster.local:9092,cgas-kafka-2.cgas-kafka.alpha.svc.cluster.local:9092"
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "2000m"
            memory: "2Gi"
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        volumeMounts:
        - name: config
          mountPath: /etc/cgas
          readOnly: true
        - name: logs
          mountPath: /var/log/cgas
      volumes:
      - name: config
        configMap:
          name: cgas-config
      - name: logs
        emptyDir: {}
      terminationGracePeriodSeconds: 60
```

### 8.2 Service 配置 `service.yaml`

```yaml
apiVersion: v1
kind: Service
metadata:
  name: cgas-workflow-engine
  namespace: alpha
  labels:
    app: cgas-workflow-engine
spec:
  type: ClusterIP
  ports:
  - port: 8080
    targetPort: 8080
    name: http
    protocol: TCP
  - port: 8081
    targetPort: 8081
    name: metrics
    protocol: TCP
  selector:
    app: cgas-workflow-engine
---
apiVersion: v1
kind: Service
metadata:
  name: cgas-workflow-engine-active
  namespace: alpha
  labels:
    app: cgas-workflow-engine
spec:
  type: ClusterIP
  ports:
  - port: 8080
    targetPort: 8080
    name: http
    protocol: TCP
  selector:
    app: cgas-workflow-engine
    version: blue  # 蓝绿部署切换
```

### 8.3 HorizontalPodAutoscaler `hpa.yaml`

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: cgas-workflow-engine
  namespace: alpha
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: cgas-workflow-engine
  minReplicas: 3
  maxReplicas: 20
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
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "1000"
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
      - type: Pods
        value: 4
        periodSeconds: 60
      selectPolicy: Max
```

### 8.4 NetworkPolicy `networkpolicy.yaml`

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: cgas-workflow-engine
  namespace: alpha
spec:
  podSelector:
    matchLabels:
      app: cgas-workflow-engine
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    - podSelector:
        matchLabels:
          app: cgas-frontend
    ports:
    - protocol: TCP
      port: 8080
  - from:
    - namespaceSelector:
        matchLabels:
          name: monitoring
    ports:
    - protocol: TCP
      port: 8081
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: alpha
      podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - namespaceSelector:
        matchLabels:
          name: alpha
      podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
  - to:
    - namespaceSelector:
        matchLabels:
          name: alpha
      podSelector:
        matchLabels:
          app: kafka
    ports:
    - protocol: TCP
      port: 9092
  - to:
    - namespaceSelector:
        matchLabels:
          name: alpha
      podSelector:
        matchLabels:
          app: opa
    ports:
    - protocol: TCP
      port: 8181
  - to:
    - namespaceSelector:
        matchLabels:
          name: monitoring
    ports:
    - protocol: TCP
      port: 4317
    - protocol: TCP
      port: 3100
  - to:
    - namespaceSelector: {}
    ports:
    - protocol: UDP
      port: 53
```

---

## 📝 配置部署清单

| 配置文件 | 用途 | 部署方式 |
|----------|------|----------|
| `application.yaml` | 应用主配置 | ConfigMap |
| `database.yaml` | 数据库配置 | ConfigMap |
| `cache.yaml` | 缓存配置 | ConfigMap |
| `messaging.yaml` | 消息队列配置 | ConfigMap |
| `security_gates.yaml` | 安全闸门配置 | ConfigMap |
| `prometheus.yaml` | Prometheus 配置 | ConfigMap |
| `metrics_config.yaml` | 监控指标配置 | ConfigMap |
| `logging.yaml` | 日志配置 | ConfigMap |
| `deployment.yaml` | Kubernetes Deployment | kubectl apply |
| `service.yaml` | Kubernetes Service | kubectl apply |
| `hpa.yaml` | 自动扩缩容 | kubectl apply |
| `networkpolicy.yaml` | 网络策略 | kubectl apply |

---

## 🔐 密钥管理

### 需要创建的 Secret

```bash
# 数据库凭证
kubectl create secret generic cgas-db-credentials \
  --from-literal=DB_HOST="cgas-postgres.alpha.svc.cluster.local" \
  --from-literal=DB_PORT="5432" \
  --from-literal=DB_NAME="cgas_alpha" \
  --from-literal=DB_USER="cgas_user" \
  --from-literal=DB_PASSWORD="<secure_password>" \
  -n alpha

# Redis 凭证
kubectl create secret generic cgas-redis-credentials \
  --from-literal=REDIS_PASSWORD="<secure_password>" \
  -n alpha

# OIDC 配置
kubectl create secret generic cgas-oidc-config \
  --from-literal=OIDC_CLIENT_ID="cgas-workflow-engine-alpha" \
  --from-literal=OIDC_CLIENT_SECRET="<secure_secret>" \
  --from-file=oidc_config.json \
  -n alpha

# TLS 证书
kubectl create secret tls cgas-tls-cert \
  --cert=/path/to/tls.crt \
  --key=/path/to/tls.key \
  -n alpha
```

---

**文档状态**: ✅ 配置文件完成  
**配置位置**: `/home/cc/Desktop/code/AIPro/cgas/rust-workflow-engine/configs/alpha/`  
**责任人**: Dev-Agent  
**保管**: CGAS 项目文档库

---

*Alpha Configuration Files v1.0 - 2026-03-07*
