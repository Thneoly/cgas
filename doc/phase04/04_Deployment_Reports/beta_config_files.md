# Beta 环境配置文件

**版本**: v1.0  
**日期**: 2026-04-08  
**责任人**: Dev-Agent  
**环境**: Beta (外部用户测试环境)  
**状态**: ✅ 已完成  

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
9. [Week 1 问题修复配置](#9-week-1-问题修复配置)

---

## 1. 应用配置文件

### 1.1 主配置文件 `application-beta.yaml`

```yaml
# CGAS Workflow Engine - Beta 环境配置
# Phase 4 Week 2 使用
# 包含 Week 1 问题修复配置

app:
  name: cgas-workflow-engine
  environment: beta
  version: phase4-beta-v1.0
  release_id: release-2026-04-08-phase4-beta

server:
  host: 0.0.0.0
  port: 8080
  workers: 8
  keep_alive: 75
  compression:
    enabled: true
    min_response_size: 1024
  timeout:
    connect: 30s
    read: 60s
    write: 60s
    idle: 300s

# Tomcat 配置 (性能优化)
tomcat:
  threads:
    max: 400
    min-spare: 50
  accept-count: 200
  max-connections: 8192
  connection-timeout: 20000
  redirect-port: 8443

# 执行器配置 (Beta 环境优化)
executor:
  max_concurrent: 150
  queue_size: 1500
  timeout_ms: 30000
  retry:
    max_attempts: 3
    initial_delay_ms: 100
    max_delay_ms: 5000
    multiplier: 2.0
  circuit_breaker:
    enabled: true
    failure_threshold: 5
    success_threshold: 3
    timeout_ms: 60000

# 验证器配置 (Beta 环境优化)
validator:
  max_concurrent: 150
  queue_size: 1500
  timeout_ms: 30000
  cache:
    enabled: true
    max_size: 15000
    ttl_seconds: 300
    eviction_policy: LRU

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

# 阻断器配置 (Week 1 问题修复：ISSUE-001)
# 修复：优化缓存策略，增加缓存预热
blocker:
  max_concurrent: 200
  queue_size: 2000
  timeout_ms: 30000
  cache:
    enabled: true
    max_size: 20000
    ttl_seconds: 600
    prewarm_enabled: true  # 新增：缓存预热
    prewarm_interval_seconds: 300
    prewarm_batch_size: 100
  rule_engine:
    enabled: true
    hot_reload: true
    reload_interval_seconds: 60

# 配置热加载 (Week 1 问题修复：ISSUE-002)
# 修复：完善配置监听器，支持更多配置类型
config:
  hot_reload:
    enabled: true
    poll_interval_seconds: 30
    watch_enabled: true
    supported_types:
      - yaml
      - json
      - properties
      - xml
    reload_delay_ms: 1000
    max_reload_attempts: 3
    rollback_on_failure: true

# JVM 内存保护 (Week 1 问题修复：ISSUE-003)
# 修复：配置 JVM 内存保护机制
jvm:
  memory:
    heap_max: "4g"
    heap_min: "2g"
    metaspace_max: "512m"
    direct_max: "1g"
  gc:
    type: G1GC
    max_pause_ms: 100
    initiating_heap_occupancy_percent: 45
    reserve_percent: 10
    concurrent_threads: 2
    parallel_threads: 4
  oom_protection:
    enabled: true
    heap_dump_enabled: true
    heap_dump_path: "/var/log/heap_dumps"
    error_file_path: "/var/log/jvm_error_%p.log"
    gc_log_enabled: true
    gc_log_path: "/var/log/gc.log"
  shutdown:
    timeout_seconds: 30
    hook_enabled: true

# 性能优化配置 (Beta 环境增强)
optimization:
  async_pool:
    enabled: true
    core_size: 40
    max_size: 200
    keep_alive_seconds: 60
    queue_capacity: 2000
    rejection_policy: caller_runs
  
  incremental_replay:
    enabled: true
    checkpoint_interval_ms: 1000
    max_batch_size: 100
  
  validation_cache:
    enabled: true
    max_entries: 15000
    expire_after_write_seconds: 300
    refresh_after_write_seconds: 240
  
  connection_pool:
    enabled: true
    max_total: 200
    max_per_route: 50
    connect_timeout_ms: 5000
    socket_timeout_ms: 30000
    connection_ttl_seconds: 300
```

---

## 2. 数据库配置

### 2.1 主数据库配置 `database-beta.yaml`

```yaml
# Beta 环境数据库配置
# Phase 4 Week 2 使用

spring:
  datasource:
    # 主数据库
    primary:
      url: jdbc:postgresql://beta-db-primary.cgas.internal:5432/cgas_beta
      username: cgas_beta_app
      password: ${DB_PRIMARY_PASSWORD}
      driver-class-name: org.postgresql.Driver
      hikari:
        pool-name: BetaPrimaryPool
        maximum-pool-size: 100
        minimum-idle: 20
        connection-timeout: 20000
        idle-timeout: 300000
        max-lifetime: 900000
        connection-test-query: SELECT 1
        validation-timeout: 5000
        leak-detection-threshold: 30000
        initialization-fail-timeout: 60000
    
    # 只读副本
    replica:
      url: jdbc:postgresql://beta-db-replica.cgas.internal:5432/cgas_beta
      username: cgas_beta_readonly
      password: ${DB_REPLICA_PASSWORD}
      driver-class-name: org.postgresql.Driver
      hikari:
        pool-name: BetaReplicaPool
        maximum-pool-size: 50
        minimum-idle: 10
        connection-timeout: 20000
        idle-timeout: 300000
        max-lifetime: 900000
        read-only: true

# JPA 配置
spring.jpa:
  database-platform: org.hibernate.dialect.PostgreSQLDialect
  hibernate:
    ddl-auto: validate
  show-sql: false
  properties:
    hibernate:
      format_sql: true
      jdbc:
        batch_size: 50
        order_inserts: true
        order_updates: true
      query:
        fail_on_pagination_over_collection_fetch: true
        in_clause_parameter_padding: true

# Flyway 迁移配置
flyway:
  enabled: true
  baseline-on-migrate: true
  baseline-version: 0
  locations: classpath:db/migration
  validate-on-migrate: true
  clean-disabled: true
  out-of-order: false
  ignore-missing-migrations: false
  ignore-ignored-migrations: false
  ignore-future-migrations: true

# 数据库性能优化
database:
  optimization:
    # 查询优化
    query:
      default_fetch_size: 100
      max_fetch_size: 1000
      statement_timeout_ms: 30000
      lock_timeout_ms: 10000
    
    # 连接池优化
    pool:
      leak_detection_threshold_ms: 30000
      max_lifetime_ms: 900000
      connection_timeout_ms: 20000
    
    # 缓存优化
    cache:
      query_cache_enabled: true
      query_cache_size: 1000
      result_set_cache_enabled: true
      result_set_cache_size: 500

# 数据库监控
database:
  monitoring:
    slow_query_threshold_ms: 1000
    log_slow_queries: true
    track_connection_pool_metrics: true
    track_query_metrics: true
```

### 2.2 数据库迁移配置 `flyway-beta.conf`

```properties
# Beta 环境 Flyway 配置
# Phase 4 Week 2 使用

flyway.url=jdbc:postgresql://beta-db-primary.cgas.internal:5432/cgas_beta
flyway.user=cgas_beta_migration
flyway.password=${FLYWAY_PASSWORD}

flyway.locations=filesystem:/opt/cgas/migrations/beta
flyway.baselineOnMigrate=true
flyway.baselineVersion=0
flyway.validateOnMigrate=true
flyway.cleanDisabled=true
flyway.outOfOrder=false
flyway.ignoreMissingMigrations=false
flyway.ignoreIgnoredMigrations=false
flyway.ignoreFutureMigrations=true

flyway.placeholders.dbName=cgas_beta
flyway.placeholders.appUser=cgas_beta_app
flyway.placeholders.readonlyUser=cgas_beta_readonly

flyway.callbacks=com.cgas.flyway.BetaMigrationCallback
flyway.mixed=true
flyway.createSchemas=true
flyway.schemas=public
flyway.table=schema_version
flyway.tablespace=
flyway.sqlMigrationPrefix=V
flyway.repeatableSqlMigrationPrefix=R
flyway.sqlMigrationSeparator=__
flyway.sqlMigrationSuffixes=.sql
flyway.encoding=UTF-8
flyway.baselineDescription=Baseline
```

---

## 3. 缓存配置

### 3.1 Redis 配置 `redis-beta.yaml`

```yaml
# Beta 环境 Redis 配置
# Phase 4 Week 2 使用

spring:
  redis:
    host: beta-redis.cgas.internal
    port: 6379
    database: 0
    password: ${REDIS_PASSWORD}
    timeout: 5000ms
    lettuce:
      pool:
        max-active: 100
        max-idle: 50
        min-idle: 10
        max-wait: 3000ms
      cluster:
        refresh:
          adaptive: true
          period: 60000

# 缓存配置
cache:
  # 执行器缓存
  executor:
    enabled: true
    type: redis
    ttl_seconds: 300
    max_entries: 15000
    key_prefix: "cgas:beta:executor:"
  
  # 验证器缓存
  validator:
    enabled: true
    type: redis
    ttl_seconds: 300
    max_entries: 15000
    key_prefix: "cgas:beta:validator:"
  
  # 阻断器缓存 (Week 1 问题修复：ISSUE-001)
  blocker:
    enabled: true
    type: redis
    ttl_seconds: 600
    max_entries: 20000
    key_prefix: "cgas:beta:blocker:"
    prewarm_enabled: true
    prewarm_interval_seconds: 300
  
  # 配置缓存
  config:
    enabled: true
    type: redis
    ttl_seconds: 60
    max_entries: 1000
    key_prefix: "cgas:beta:config:"
  
  # 会话缓存
  session:
    enabled: true
    type: redis
    ttl_seconds: 1800
    max_entries: 10000
    key_prefix: "cgas:beta:session:"

# 缓存监控
cache:
  monitoring:
    enabled: true
    track_hits: true
    track_misses: true
    track_evictions: true
    report_interval_seconds: 60
```

### 3.2 本地缓存配置 `caffeine-beta.yaml`

```yaml
# Beta 环境 Caffeine 本地缓存配置
# Phase 4 Week 2 使用

caffeine:
  # 阻断规则缓存 (Week 1 问题修复：ISSUE-001)
  blocker-rules:
    spec: maximumSize=20000,expireAfterWrite=600s,refreshAfterWrite=540s
    stats_enabled: true
  
  # 配置缓存 (Week 1 问题修复：ISSUE-002)
  config:
    spec: maximumSize=1000,expireAfterWrite=60s,refreshAfterWrite=50s
    stats_enabled: true
  
  # 验证结果缓存
  validation-result:
    spec: maximumSize=15000,expireAfterWrite=300s
    stats_enabled: true
  
  # 用户权限缓存
  user-permission:
    spec: maximumSize=5000,expireAfterWrite=600s
    stats_enabled: true

# 缓存统计
cache:
  stats:
    enabled: true
    export_interval_seconds: 60
    export_to: prometheus
```

---

## 4. 消息队列配置

### 4.1 RabbitMQ 配置 `rabbitmq-beta.yaml`

```yaml
# Beta 环境 RabbitMQ 配置
# Phase 4 Week 2 使用

spring:
  rabbitmq:
    host: beta-rabbitmq.cgas.internal
    port: 5672
    username: cgas_beta
    password: ${RABBITMQ_PASSWORD}
    virtual-host: /cgas_beta
    listener:
      simple:
        acknowledge-mode: auto
        concurrency: 5
        max-concurrency: 20
        prefetch: 10
        retry:
          enabled: true
          initial-interval: 1000
          max-attempts: 3
          max-interval: 10000
          multiplier: 2.0
      direct:
        acknowledge-mode: auto
        prefetch: 10
    template:
      mandatory: true
      retry:
        enabled: true
        initial-interval: 1000
        max-attempts: 3
        max-interval: 10000

# 消息队列配置
mq:
  # 执行队列
  executor:
    exchange: cgas.beta.executor.exchange
    queue: cgas.beta.executor.queue
    routing_key: cgas.beta.executor
    durable: true
    auto_delete: false
    
  # 验证队列
  validator:
    exchange: cgas.beta.validator.exchange
    queue: cgas.beta.validator.queue
    routing_key: cgas.beta.validator
    durable: true
    auto_delete: false
    
  # 阻断队列
  blocker:
    exchange: cgas.beta.blocker.exchange
    queue: cgas.beta.blocker.queue
    routing_key: cgas.beta.blocker
    durable: true
    auto_delete: false
    
  # 死信队列
  dlq:
    exchange: cgas.beta.dlq.exchange
    queue: cgas.beta.dlq.queue
    routing_key: cgas.beta.dlq
    durable: true
    auto_delete: false
    ttl_ms: 86400000

# 消息监控
mq:
  monitoring:
    enabled: true
    track_publish: true
    track_consume: true
    track_ack: true
    track_nack: true
    track_dlq: true
```

---

## 5. 安全配置

### 5.1 安全配置 `security-beta.yaml`

```yaml
# Beta 环境安全配置
# Phase 4 Week 2 使用

# Spring Security 配置
spring:
  security:
    user:
      name: ${SECURITY_ADMIN_USER}
      password: ${SECURITY_ADMIN_PASSWORD}
    oauth2:
      resourceserver:
        jwt:
          issuer-uri: https://auth.cgas.internal
          jwk-set-uri: https://auth.cgas.internal/.well-known/jwks.json

# 认证配置
auth:
  # JWT 配置
  jwt:
    secret: ${JWT_SECRET}
    expiration_ms: 3600000
    refresh_expiration_ms: 86400000
    header: Authorization
    prefix: Bearer
    
  # 会话管理
  session:
    enabled: true
    timeout_seconds: 1800
    max_concurrent_sessions: 5
    prevent_concurrent_login: false
    
  # 密码策略
  password:
    min_length: 8
    require_uppercase: true
    require_lowercase: true
    require_digit: true
    require_special: true
    max_age_days: 90
    history_count: 5

# 授权配置
authz:
  # RBAC 配置
  rbac:
    enabled: true
    cache_enabled: true
    cache_ttl_seconds: 300
    
  # 权限检查
  permission:
    cache_enabled: true
    cache_ttl_seconds: 300
    default_deny: true

# API 安全
api:
  security:
    # CORS 配置
    cors:
      enabled: true
      allowed_origins:
        - https://beta.cgas.internal
        - https://app.cgas.internal
      allowed_methods:
        - GET
        - POST
        - PUT
        - DELETE
        - OPTIONS
      allowed_headers:
        - Authorization
        - Content-Type
        - X-Request-ID
      exposed_headers:
        - X-Request-ID
        - X-Total-Count
      allow_credentials: true
      max_age_seconds: 3600
    
    # CSRF 配置
    csrf:
      enabled: true
      cookie_secure: true
      cookie_http_only: true
    
    # 速率限制
    rate_limit:
      enabled: true
      requests_per_second: 100
      burst_size: 200
      key_type: ip  # ip, user, api_key

# 审计日志
audit:
  enabled: true
  log_authentication: true
  log_authorization: true
  log_data_access: true
  retention_days: 90
```

### 5.2 防火墙配置 `firewall-beta.yaml`

```yaml
# Beta 环境防火墙配置
# Phase 4 Week 2 使用

firewall:
  # IP 白名单
  whitelist:
    enabled: true
    ips:
      - 10.0.0.0/8
      - 172.16.0.0/12
      - 192.168.0.0/16
      - 100.64.0.0/10
    
  # IP 黑名单
  blacklist:
    enabled: true
    ips: []
    auto_block_enabled: true
    auto_block_threshold: 100
    auto_block_duration_seconds: 3600
  
  # 请求过滤
  request_filter:
    enabled: true
    max_body_size: 10485760  # 10MB
    max_header_size: 8192
    max_uri_length: 2048
    blocked_content_types:
      - application/x-msdownload
      - application/x-executable
    
  # SQL 注入防护
  sql_injection:
    enabled: true
    mode: block  # log, block
    patterns:
      - ".*\\b(SELECT|INSERT|UPDATE|DELETE|DROP|TRUNCATE|ALTER)\\b.*"
      - ".*\\b(UNION|JOIN|WHERE|HAVING|GROUP BY)\\b.*"
      - ".*--.*"
      - ".*;.*"
  
  # XSS 防护
  xss:
    enabled: true
    mode: block  # log, block, sanitize
    sanitize_html: true
    
  # 路径遍历防护
  path_traversal:
    enabled: true
    mode: block
    blocked_patterns:
      - ".."
      - "%2e%2e"
      - "%252e%252e"
```

---

## 6. 监控配置

### 6.1 Prometheus 配置 `prometheus-beta.yaml`

```yaml
# Beta 环境 Prometheus 监控配置
# Phase 4 Week 2 使用

global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    environment: beta
    cluster: cgas-beta

scrape_configs:
  # 应用监控
  - job_name: 'cgas-beta-app'
    static_configs:
      - targets:
          - 'executor:8080'
          - 'verifier:8080'
          - 'blocker:8080'
    metrics_path: '/actuator/prometheus'
    scrape_interval: 10s
    
  # 数据库监控
  - job_name: 'cgas-beta-postgres'
    static_configs:
      - targets:
          - 'beta-db-primary:9187'
          - 'beta-db-replica:9187'
    scrape_interval: 15s
    
  # Redis 监控
  - job_name: 'cgas-beta-redis'
    static_configs:
      - targets:
          - 'beta-redis-exporter:9121'
    scrape_interval: 15s
    
  # 系统监控
  - job_name: 'cgas-beta-node'
    static_configs:
      - targets:
          - 'beta-app-01:9100'
          - 'beta-app-02:9100'
          - 'beta-app-03:9100'
          - 'beta-db-primary:9100'
          - 'beta-db-replica:9100'
    scrape_interval: 15s

# 告警规则
rule_files:
  - '/etc/prometheus/rules/beta_alerts.yml'

# Alertmanager 配置
alerting:
  alertmanagers:
    - static_configs:
        - targets:
            - 'beta-alertmanager:9093'
```

### 6.2 Grafana 仪表盘配置 `grafana-beta-dashboard.json`

```json
{
  "dashboard": {
    "title": "CGAS Beta 环境监控",
    "tags": ["cgas", "beta", "production"],
    "timezone": "Asia/Shanghai",
    "refresh": "30s",
    "panels": [
      {
        "title": "QPS (每秒请求数)",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_server_requests_seconds_count{environment=\"beta\"}[1m])",
            "legendFormat": "{{service}}"
          }
        ]
      },
      {
        "title": "P99 时延",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(http_server_requests_seconds_bucket{environment=\"beta\"}[5m]))",
            "legendFormat": "{{service}} P99"
          }
        ],
        "thresholds": [
          {
            "value": 0.2,
            "colorMode": "critical"
          }
        ]
      },
      {
        "title": "错误率",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_server_requests_seconds_count{environment=\"beta\",status=~\"5..\"}[1m]) / rate(http_server_requests_seconds_count{environment=\"beta\"}[1m]) * 100",
            "legendFormat": "{{service}}"
          }
        ]
      },
      {
        "title": "JVM 内存使用",
        "type": "graph",
        "targets": [
          {
            "expr": "jvm_memory_used_bytes{environment=\"beta\",area=\"heap\"}",
            "legendFormat": "{{service}} - {{id}}"
          }
        ]
      },
      {
        "title": "数据库连接池",
        "type": "graph",
        "targets": [
          {
            "expr": "hikaricp_connections{environment=\"beta\"}",
            "legendFormat": "{{service}} - {{state}}"
          }
        ]
      },
      {
        "title": "缓存命中率",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(cache_hits_total{environment=\"beta\"}[1m]) / (rate(cache_hits_total{environment=\"beta\"}[1m]) + rate(cache_misses_total{environment=\"beta\"}[1m])) * 100",
            "legendFormat": "{{cache}}"
          }
        ]
      }
    ]
  }
}
```

---

## 7. 日志配置

### 7.1 Logback 配置 `logback-beta.xml`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<configuration scan="true" scanPeriod="30 seconds">
    
    <!-- 属性定义 -->
    <property name="APP_NAME" value="cgas-beta"/>
    <property name="LOG_PATH" value="/var/log/cgas"/>
    <property name="LOG_PATTERN" value="%d{yyyy-MM-dd HH:mm:ss.SSS} [%thread] %-5level %logger{36} - %msg%n"/>
    
    <!-- 控制台输出 -->
    <appender name="CONSOLE" class="ch.qos.logback.core.ConsoleAppender">
        <encoder>
            <pattern>${LOG_PATTERN}</pattern>
            <charset>UTF-8</charset>
        </encoder>
    </appender>
    
    <!-- 文件输出 - 应用日志 -->
    <appender name="APP_FILE" class="ch.qos.logback.core.rolling.RollingFileAppender">
        <file>${LOG_PATH}/${APP_NAME}-application.log</file>
        <encoder>
            <pattern>${LOG_PATTERN}</pattern>
            <charset>UTF-8</charset>
        </encoder>
        <rollingPolicy class="ch.qos.logback.core.rolling.TimeBasedRollingPolicy">
            <fileNamePattern>${LOG_PATH}/${APP_NAME}-application.%d{yyyy-MM-dd}.%i.log</fileNamePattern>
            <timeBasedFileNamingAndTriggeringPolicy class="ch.qos.logback.core.rolling.SizeAndTimeBasedFNATP">
                <maxFileSize>100MB</maxFileSize>
            </timeBasedFileNamingAndTriggeringPolicy>
            <maxHistory>30</maxHistory>
            <totalSizeCap>10GB</totalSizeCap>
        </rollingPolicy>
    </appender>
    
    <!-- 文件输出 - 错误日志 -->
    <appender name="ERROR_FILE" class="ch.qos.logback.core.rolling.RollingFileAppender">
        <file>${LOG_PATH}/${APP_NAME}-error.log</file>
        <filter class="ch.qos.logback.classic.filter.ThresholdFilter">
            <level>ERROR</level>
        </filter>
        <encoder>
            <pattern>${LOG_PATTERN}</pattern>
            <charset>UTF-8</charset>
        </encoder>
        <rollingPolicy class="ch.qos.logback.core.rolling.TimeBasedRollingPolicy">
            <fileNamePattern>${LOG_PATH}/${APP_NAME}-error.%d{yyyy-MM-dd}.log</fileNamePattern>
            <maxHistory>30</maxHistory>
            <totalSizeCap>5GB</totalSizeCap>
        </rollingPolicy>
    </appender>
    
    <!-- 文件输出 - 审计日志 -->
    <appender name="AUDIT_FILE" class="ch.qos.logback.core.rolling.RollingFileAppender">
        <file>${LOG_PATH}/${APP_NAME}-audit.log</file>
        <encoder>
            <pattern>${LOG_PATTERN}</pattern>
            <charset>UTF-8</charset>
        </encoder>
        <rollingPolicy class="ch.qos.logback.core.rolling.TimeBasedRollingPolicy">
            <fileNamePattern>${LOG_PATH}/${APP_NAME}-audit.%d{yyyy-MM-dd}.log</fileNamePattern>
            <maxHistory>90</maxHistory>
            <totalSizeCap>20GB</totalSizeCap>
        </rollingPolicy>
    </appender>
    
    <!-- 文件输出 - GC 日志 (Week 1 问题修复：ISSUE-003) -->
    <appender name="GC_FILE" class="ch.qos.logback.core.rolling.RollingFileAppender">
        <file>${LOG_PATH}/${APP_NAME}-gc.log</file>
        <encoder>
            <pattern>%msg%n</pattern>
            <charset>UTF-8</charset>
        </encoder>
        <rollingPolicy class="ch.qos.logback.core.rolling.TimeBasedRollingPolicy">
            <fileNamePattern>${LOG_PATH}/${APP_NAME}-gc.%d{yyyy-MM-dd}.log</fileNamePattern>
            <maxHistory>7</maxHistory>
            <totalSizeCap>1GB</totalSizeCap>
        </rollingPolicy>
    </appender>
    
    <!-- 异步日志 -->
    <appender name="ASYNC_APP" class="ch.qos.logback.classic.AsyncAppender">
        <appender-ref ref="APP_FILE"/>
        <queueSize>512</queueSize>
        <discardingThreshold>0</discardingThreshold>
    </appender>
    
    <!-- 日志级别 -->
    <logger name="com.cgas" level="DEBUG" additivity="false">
        <appender-ref ref="CONSOLE"/>
        <appender-ref ref="ASYNC_APP"/>
        <appender-ref ref="ERROR_FILE"/>
    </logger>
    
    <logger name="com.cgas.audit" level="INFO" additivity="false">
        <appender-ref ref="AUDIT_FILE"/>
    </logger>
    
    <logger name="org.springframework" level="INFO"/>
    <logger name="org.hibernate" level="WARN"/>
    <logger name="com.zaxxer.hikari" level="DEBUG"/>
    
    <!-- 根日志级别 -->
    <root level="INFO">
        <appender-ref ref="CONSOLE"/>
        <appender-ref ref="ASYNC_APP"/>
        <appender-ref ref="ERROR_FILE"/>
    </root>
    
</configuration>
```

---

## 8. Kubernetes 资源配置

### 8.1 Deployment 配置 `beta-deployment.yaml`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cgas-beta-executor
  namespace: cgas-beta
  labels:
    app: executor
    environment: beta
    version: phase4-beta-v1.0
spec:
  replicas: 5
  selector:
    matchLabels:
      app: executor
  template:
    metadata:
      labels:
        app: executor
        environment: beta
        version: phase4-beta-v1.0
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/actuator/prometheus"
    spec:
      containers:
      - name: executor
        image: cgas/executor:phase4-beta-v1.0
        imagePullPolicy: Always
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        env:
        - name: SPRING_PROFILES_ACTIVE
          value: "beta"
        - name: SERVER_PORT
          value: "8080"
        - name: JAVA_OPTS
          valueFrom:
            configMapKeyRef:
              name: cgas-beta-jvm-config
              key: JVM_OPTS
        - name: DB_PASSWORD
          valueFrom:
            secretKeyRef:
              name: cgas-beta-db-secret
              key: password
        - name: REDIS_PASSWORD
          valueFrom:
            secretKeyRef:
              name: cgas-beta-redis-secret
              key: password
        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
        - name: logs
          mountPath: /var/log/cgas
        - name: heap-dumps
          mountPath: /var/log/heap_dumps
        resources:
          requests:
            cpu: "1000m"
            memory: "1Gi"
          limits:
            cpu: "4000m"
            memory: "4Gi"
        livenessProbe:
          httpGet:
            path: /actuator/health/liveness
            port: 8080
          initialDelaySeconds: 60
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /actuator/health/readiness
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        startupProbe:
          httpGet:
            path: /actuator/health
            port: 8080
          initialDelaySeconds: 0
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 30
      volumes:
      - name: config
        configMap:
          name: cgas-beta-app-config
      - name: logs
        emptyDir: {}
      - name: heap-dumps
        emptyDir: {}
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchLabels:
                  app: executor
              topologyKey: kubernetes.io/hostname
      terminationGracePeriodSeconds: 60
```

### 8.2 Service 配置 `beta-service.yaml`

```yaml
apiVersion: v1
kind: Service
metadata:
  name: executor
  namespace: cgas-beta
  labels:
    app: executor
    environment: beta
spec:
  type: ClusterIP
  ports:
  - port: 8080
    targetPort: 8080
    protocol: TCP
    name: http
  selector:
    app: executor
---
apiVersion: v1
kind: Service
metadata:
  name: verifier
  namespace: cgas-beta
  labels:
    app: verifier
    environment: beta
spec:
  type: ClusterIP
  ports:
  - port: 8080
    targetPort: 8080
    protocol: TCP
    name: http
  selector:
    app: verifier
---
apiVersion: v1
kind: Service
metadata:
  name: blocker
  namespace: cgas-beta
  labels:
    app: blocker
    environment: beta
spec:
  type: ClusterIP
  ports:
  - port: 8080
    targetPort: 8080
    protocol: TCP
    name: http
  selector:
    app: blocker
---
apiVersion: v1
kind: Service
metadata:
  name: cgas-beta-lb
  namespace: cgas-beta
  labels:
    environment: beta
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
    name: http
  - port: 443
    targetPort: 8443
    protocol: TCP
    name: https
  selector:
    app: executor
  sessionAffinity: ClientIP
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 10800
```

### 8.3 Ingress 配置 `beta-ingress.yaml`

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: cgas-beta-ingress
  namespace: cgas-beta
  annotations:
    kubernetes.io/ingress.class: nginx
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    nginx.ingress.kubernetes.io/proxy-connect-timeout: "30"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "60"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "60"
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-burst: "200"
spec:
  tls:
  - hosts:
    - beta.cgas.internal
    secretName: cgas-beta-tls
  rules:
  - host: beta.cgas.internal
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: cgas-beta-lb
            port:
              number: 80
      - path: /actuator
        pathType: Prefix
        backend:
          service:
            name: executor
            port:
              number: 8080
```

### 8.4 HPA 配置 `beta-hpa.yaml`

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: executor-hpa
  namespace: cgas-beta
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: cgas-beta-executor
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
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
      - type: Pods
        value: 4
        periodSeconds: 15
      selectPolicy: Max
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: verifier-hpa
  namespace: cgas-beta
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: cgas-beta-verifier
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
        averageUtilization: 80
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: blocker-hpa
  namespace: cgas-beta
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: cgas-beta-blocker
  minReplicas: 2
  maxReplicas: 6
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

---

## 9. Week 1 问题修复配置

### 9.1 ISSUE-001: 阻断并发偶发超时修复配置

```yaml
# Week 1 问题修复：ISSUE-001
# 问题：阻断并发偶发超时 (约 5% 概率)
# 原因：阻断规则缓存未命中导致数据库查询延迟
# 修复：优化缓存策略，增加缓存预热

blocker:
  cache:
    # 启用缓存
    enabled: true
    
    # 缓存大小 (从 10000 增加到 20000)
    max_size: 20000
    
    # 缓存 TTL (从 300 秒增加到 600 秒)
    ttl_seconds: 600
    
    # 新增：缓存预热
    prewarm:
      enabled: true
      interval_seconds: 300
      batch_size: 100
      timeout_ms: 5000
    
    # 新增：缓存刷新
    refresh:
      enabled: true
      strategy: async
      refresh_ahead_seconds: 60
    
    # 新增：缓存统计
    stats:
      enabled: true
      export_interval_seconds: 60
  
  # 规则引擎优化
  rule_engine:
    # 启用热加载
    hot_reload: true
    
    # 热加载间隔
    reload_interval_seconds: 60
    
    # 规则缓存
    cache:
      enabled: true
      max_rules: 1000
      ttl_seconds: 300
```

### 9.2 ISSUE-002: 配置热加载部分失败修复配置

```yaml
# Week 1 问题修复：ISSUE-002
# 问题：配置热加载时部分配置项未生效
# 原因：配置监听器未正确处理某些配置类型
# 修复：完善配置监听器，支持更多配置类型

config:
  hot_reload:
    # 启用热加载
    enabled: true
    
    # 轮询间隔
    poll_interval_seconds: 30
    
    # 启用文件监听
    watch_enabled: true
    
    # 支持的配置类型 (新增 xml)
    supported_types:
      - yaml
      - json
      - properties
      - xml
    
    # 热加载延迟
    reload_delay_ms: 1000
    
    # 最大重试次数
    max_reload_attempts: 3
    
    # 失败回滚
    rollback_on_failure: true
    
    # 配置验证
    validation:
      enabled: true
      schema_validation: true
      syntax_validation: true
    
    # 配置监听器
    listener:
      enabled: true
      async: true
      queue_size: 100
      timeout_ms: 5000
    
    # 配置变更通知
    notification:
      enabled: true
      channels:
        - log
        - webhook
      webhook_url: http://beta-monitor.cgas.internal/config-change
```

### 9.3 ISSUE-003: OOM 保护缺失修复配置

```yaml
# Week 1 问题修复：ISSUE-003
# 问题：内存不足时服务直接崩溃
# 原因：未配置 JVM 内存保护机制
# 修复：配置 JVM 内存保护

jvm:
  memory:
    # 堆内存配置
    heap_max: "4g"
    heap_min: "2g"
    
    # 元空间配置
    metaspace_max: "512m"
    
    # 直接内存配置
    direct_max: "1g"
    
    # 堆外内存配置
    off_heap_max: "2g"
  
  # GC 配置
  gc:
    # 使用 G1GC
    type: G1GC
    
    # 最大 GC 暂停时间
    max_pause_ms: 100
    
    # 堆占用阈值
    initiating_heap_occupancy_percent: 45
    
    # 保留内存比例
    reserve_percent: 10
    
    # GC 线程数
    concurrent_threads: 2
    parallel_threads: 4
  
  # OOM 保护配置
  oom_protection:
    # 启用 OOM 保护
    enabled: true
    
    # HeapDump
    heap_dump:
      enabled: true
      path: "/var/log/heap_dumps"
      dump_on_oom: true
      dump_on_gc_threshold: true
      gc_threshold_percent: 90
    
    # 错误日志
    error_file:
      enabled: true
      path: "/var/log/jvm_error_%p.log"
    
    # GC 日志
    gc_log:
      enabled: true
      path: "/var/log/gc.log"
      rotation:
        max_files: 5
        max_size: "10M"
      decorators:
        - time
        - uptime
        - level
        - tags
    
    # 内存告警
    memory_alert:
      enabled: true
      threshold_percent: 85
      alert_interval_seconds: 60
  
  # 优雅关闭
  shutdown:
    # 关闭超时
    timeout_seconds: 30
    
    # 关闭钩子
    hook:
      enabled: true
      order: 100
    
    # 健康检查关闭
    health_check_shutdown:
      enabled: true
      delay_seconds: 5
```

---

## 📝 配置使用说明

### 配置部署流程

```bash
# 1. 创建 ConfigMap
kubectl apply -f beta-configmaps.yaml

# 2. 创建 Secrets
kubectl apply -f beta-secrets.yaml

# 3. 部署应用
kubectl apply -f beta-deployment.yaml

# 4. 验证配置
kubectl get configmap -n cgas-beta
kubectl get secret -n cgas-beta
```

### 配置热加载

```bash
# 修改配置
kubectl edit configmap cgas-beta-app-config -n cgas-beta

# 触发配置刷新 (如未自动刷新)
curl -X POST http://localhost:8080/actuator/refresh

# 查看配置状态
curl http://localhost:8080/actuator/configprops
```

---

## 📊 配置清单

| 配置类别 | 文件名 | 用途 |
|---|---|---|
| 应用配置 | application-beta.yaml | 主应用配置 |
| 数据库配置 | database-beta.yaml | 数据库连接配置 |
| 缓存配置 | redis-beta.yaml | Redis 缓存配置 |
| 消息队列配置 | rabbitmq-beta.yaml | RabbitMQ 配置 |
| 安全配置 | security-beta.yaml | 认证授权配置 |
| 监控配置 | prometheus-beta.yaml | Prometheus 监控 |
| 日志配置 | logback-beta.xml | Logback 日志 |
| K8s 配置 | beta-*.yaml | Kubernetes 资源 |

---

**文档状态**: ✅ Beta 环境配置文件完成  
**创建日期**: 2026-04-08  
**责任人**: Dev-Agent  
**验收人**: SRE-Agent + PM-Agent  
**保管**: 项目文档库  
**分发**: Dev 团队、SRE 团队、运维团队

---

*Beta Config Files v1.0 - 2026-04-08*
