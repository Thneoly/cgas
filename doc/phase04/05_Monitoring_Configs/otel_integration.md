# OpenTelemetry 集成方案

**版本**: v1.0  
**日期**: 2026-03-07  
**责任人**: Observability-Agent  
**状态**: ✅ 已完成  
**release_id**: release-2026-03-07-phase3_week02  
**关联文档**: 
- distributed_tracing.md (分布式追踪设计)
- phase3_50_metrics_plan.md (50 指标规划)
- monitoring_dashboard_v6.md (仪表盘 v6)

---

## 1. 概述

### 1.1 集成目标

| 目标 | 说明 | 验收标准 |
|---|---|---|
| **零侵入** | 业务代码无感知 | 通过宏/注解自动埋点 |
| **低开销** | 性能影响<1% | 压测验证 |
| **全链路** | trace_id 贯穿所有组件 | 关联率≥98% |
| **标准化** | OpenTelemetry 协议 | 兼容 Tempo/Jaeger |

### 1.2 技术选型

| 组件 | 选型 | 版本 | 理由 |
|---|---|---|---|
| 追踪协议 | OpenTelemetry Protocol (OTLP) | v1.0+ | CNCF 标准，多后端兼容 |
| 采集器 | OpenTelemetry Collector | 0.95.0 | 统一采集、处理、导出 |
| 后端存储 | Tempo + Jaeger | 2.4.0 + 1.55 | Tempo(低成本) + Jaeger(查询) |
| 可视化 | Grafana | 10.3.4 | 与监控仪表盘统一 |
| Rust SDK | opentelemetry-rust | 0.22+ | 官方支持，性能优异 |
| TypeScript SDK | @opentelemetry/api | 1.8+ | 官方支持 |

---

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                     Application Layer                            │
├─────────────────────────────────────────────────────────────────┤
│  Executor (Rust)  │  Verifier (Rust)  │  Gateway (TypeScript)  │
│  + OTel SDK       │  + OTel SDK       │  + OTel SDK            │
└─────────┬─────────┴─────────┬─────────┴──────────┬──────────────┘
          │                   │                     │
          │ OTLP (gRPC)       │ OTLP (gRPC)         │ OTLP (gRPC)
          ▼                   ▼                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                  OpenTelemetry Collector                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ Receivers   │→ │ Processors  │→ │ Exporters               │  │
│  │ - OTLP      │  │ - Batch     │  │ - Tempo                 │  │
│  │ - Prometheus│  │ - Sampling  │  │ - Jaeger                │  │
│  │ - Logs      │  │ - Enrich    │  │ - Prometheus (metrics)  │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
          │                   │                     │
          ▼                   ▼                     ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐
│     Tempo       │  │     Jaeger      │  │    Prometheus       │
│  (Trace Storage)│  │ (Trace Query)   │  │  (Trace Metrics)    │
└─────────────────┘  └─────────────────┘  └─────────────────────┘
          │                   │                     │
          └───────────────────┴─────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │     Grafana     │
                    │  (Visualization)│
                    └─────────────────┘
```

### 2.2 数据流

```
Request → Gateway → Executor → Verifier → Response
   │         │          │          │
   │         │          │          └─→ Span: Verifier.verify
   │         │          └─→ Span: Executor.execute
   │         └─→ Span: Gateway.route
   └─→ Span: Gateway.receive
   
All Spans → OTLP → Collector → Tempo/Jaeger
```

---

## 3. Rust SDK 集成

### 3.1 依赖配置

```toml
# Cargo.toml

[dependencies]
opentelemetry = "0.22"
opentelemetry_sdk = { version = "0.22", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.15", features = ["grpc-tonic"] }
opentelemetry-semantic-conventions = "0.14"
opentelemetry-prometheus = "0.15"
prometheus = "0.13"
tracing = "0.1"
tracing-opentelemetry = "0.23"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 3.2 初始化配置

```rust
// src/tracing/config.rs

use opentelemetry::{global, KeyValue, trace::TracerProvider};
use opentelemetry_otlp::{WithExportConfig, SpanExporterBuilder};
use opentelemetry_sdk::{
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use opentelemetry_semantic_conventions as semconv;
use std::time::Duration;

/// Phase 3 追踪配置
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// OTLP 导出端点
    pub otlp_endpoint: String,
    /// 服务名称
    pub service_name: String,
    /// 服务版本
    pub service_version: String,
    /// 采样率 (0.0~1.0)
    pub sample_rate: f64,
    /// Batch 导出间隔
    pub batch_interval_ms: u64,
    /// 最大导出队列大小
    pub max_export_queue_size: usize,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://otel-collector:4317".to_string()),
            service_name: std::env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "cgas-phase3".to_string()),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            sample_rate: 0.1, // 默认 10% 采样
            batch_interval_ms: 5000,
            max_export_queue_size: 2048,
        }
    }
}

/// 初始化 OpenTelemetry 追踪
pub fn init_opentelemetry(config: TracingConfig) -> Result<trace::TracerProvider, Box<dyn std::error::Error>> {
    // 创建 OTLP 导出器
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.otlp_endpoint)
        .with_timeout(Duration::from_secs(3))
        .build_span_exporter()?;
    
    // 创建资源标签
    let resource = Resource::new(vec![
        KeyValue::new(semconv::resource::SERVICE_NAME, config.service_name),
        KeyValue::new(semconv::resource::SERVICE_VERSION, config.service_version),
        KeyValue::new(semconv::resource::DEPLOYMENT_ENVIRONMENT, "production"),
        KeyValue::new("phase", "phase3"),
        KeyValue::new("team", "cgas"),
    ]);
    
    // 创建 TracerProvider
    let tracer_provider = trace::TracerProvider::builder()
        .with_config(trace::Config {
            // 采样器：基于采样率
            sampler: Sampler::ParentBased(Box::new(
                Sampler::TraceIdRatioBased(config.sample_rate)
            )),
            ..Default::default()
        })
        .with_resource(resource)
        .with_batch_exporter(
            exporter,
            opentelemetry_sdk::runtime::Tokio,
        )
        .build();
    
    // 设置为全局 TracerProvider
    let _ = global::set_tracer_provider(tracer_provider.clone());
    
    // 初始化 tracing-opentelemetry 集成
    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(tracer_provider.tracer("cgas-phase3"));
    
    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())),
        )
        .init();
    
    Ok(tracer_provider)
}

/// 关闭追踪（优雅退出时调用）
pub fn shutdown_opentelemetry() {
    global::shutdown_tracer_provider();
}
```

### 3.3 自动埋点宏

```rust
// src/tracing/macros.rs

use opentelemetry::{global, trace::{Span, Tracer, TraceContextExt}, Context, KeyValue};

/// 自动追踪函数执行的宏
#[macro_export]
macro_rules! trace_function {
    ($fn_name:expr, $($key:expr => $value:expr),*) => {
        let tracer = global::tracer("cgas-phase3");
        let span = tracer
            .span_builder($fn_name)
            .with_attributes(vec![
                $(KeyValue::new($key, $value)),*
            ])
            .start(&tracer);
        
        let cx = Context::current_with_span(span);
        let _guard = cx.attach();
    };
}

/// 记录执行时延的宏
#[macro_export]
macro_rules! trace_duration {
    ($span:expr, $start:expr, $operation:expr) => {
        let duration = $start.elapsed();
        $span.add_event(
            $operation,
            vec![KeyValue::new("duration_ms", duration.as_millis() as i64)],
        );
    };
}

/// 记录错误的宏
#[macro_export]
macro_rules! trace_error {
    ($span:expr, $error:expr) => {
        use opentelemetry::trace::{Status, TraceContextExt};
        use opentelemetry::Context;
        
        $span.set_status(Status::error(format!("{}", $error)));
        $span.record_exception(&$error);
    };
}

/// 追踪异步函数
#[macro_export]
macro_rules! trace_async {
    ($span_name:expr, $($key:expr => $value:expr),*, $future:expr) => {
        async {
            let tracer = global::tracer("cgas-phase3");
            let span = tracer
                .span_builder($span_name)
                .with_attributes(vec![
                    $(KeyValue::new($key, $value)),*
                ])
                .start(&tracer);
            
            let cx = Context::current_with_span(span);
            let _guard = cx.attach();
            
            $future.await
        }
    };
}
```

### 3.4 执行器埋点示例

```rust
// src/executor/mod.rs

use opentelemetry::{global, trace::{Span, Tracer, Status}, Context, KeyValue};
use std::time::Instant;

pub struct Executor {
    tracer: opentelemetry::sdk::trace::Tracer,
}

impl Executor {
    pub fn new() -> Self {
        let tracer = global::tracer("cgas-phase3/executor");
        Self { tracer }
    }
    
    /// 执行指令（带追踪）
    pub fn execute_instruction(&self, instruction: &Instruction) -> Result<ExecutionResult, Error> {
        let span = self.tracer
            .span_builder("Executor.execute_instruction")
            .with_attributes(vec![
                KeyValue::new("instruction.id", instruction.id.clone()),
                KeyValue::new("instruction.type", instruction.instruction_type.clone()),
                KeyValue::new("instruction.priority", instruction.priority as i64),
            ])
            .start(&self.tracer);
        
        let cx = Context::current_with_span(span);
        let _guard = cx.attach();
        
        let start = Instant::now();
        
        // 执行逻辑
        let result = self.do_execute(instruction);
        
        let duration = start.elapsed();
        
        // 记录时延
        let span = cx.span();
        span.add_event(
            "execution_completed",
            vec![
                KeyValue::new("duration_ms", duration.as_millis() as i64),
                KeyValue::new("success", result.is_ok()),
            ],
        );
        
        // 记录错误
        if let Err(ref error) = result {
            span.set_status(Status::error(format!("{}", error)));
            span.record_exception(error);
        }
        
        result
    }
    
    fn do_execute(&self, instruction: &Instruction) -> Result<ExecutionResult, Error> {
        // 实际执行逻辑
        Ok(ExecutionResult::default())
    }
}
```

---

## 4. TypeScript SDK 集成

### 4.1 依赖配置

```json
// package.json

{
  "dependencies": {
    "@opentelemetry/api": "^1.8.0",
    "@opentelemetry/sdk-trace-node": "^1.21.0",
    "@opentelemetry/sdk-trace-base": "^1.21.0",
    "@opentelemetry/exporter-trace-otlp-grpc": "^0.48.0",
    "@opentelemetry/resources": "^1.21.0",
    "@opentelemetry/semantic-conventions": "^1.21.0"
  }
}
```

### 4.2 初始化配置

```typescript
// src/tracing/index.ts

import * as api from '@opentelemetry/api';
import { NodeTracerProvider } from '@opentelemetry/sdk-trace-node';
import { BatchSpanProcessor } from '@opentelemetry/sdk-trace-base';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc';
import { Resource } from '@opentelemetry/resources';
import { SemanticResourceAttributes } from '@opentelemetry/semantic-conventions';

interface TracingConfig {
  otlpEndpoint: string;
  serviceName: string;
  sampleRate: number;
}

export function initTracing(config: TracingConfig): api.TracerProvider {
  const provider = new NodeTracerProvider({
    resource: new Resource({
      [SemanticResourceAttributes.SERVICE_NAME]: config.serviceName,
      [SemanticResourceAttributes.SERVICE_VERSION]: process.env.npm_package_version || 'unknown',
      ['deployment.environment']: 'production',
      ['phase']: 'phase3',
      ['team']: 'cgas',
    }),
    sampler: {
      shouldSample: (context, traceId, spanName, spanKind, attributes) => {
        // 基于采样率的采样
        const hash = hashTraceId(traceId);
        const shouldSample = hash < config.sampleRate;
        return {
          decision: shouldSample ? api.SamplingDecision.RECORD_AND_SAMPLED : api.SamplingDecision.NOT_RECORD,
        };
      },
    },
  });

  const exporter = new OTLPTraceExporter({
    url: config.otlpEndpoint,
  });

  provider.addSpanProcessor(
    new BatchSpanProcessor(exporter, {
      scheduledDelayMillis: 5000,
      maxExportBatchSize: 512,
    })
  );

  // 开发环境输出到控制台
  if (process.env.NODE_ENV === 'development') {
    const { ConsoleSpanExporter, SimpleSpanProcessor } = require('@opentelemetry/sdk-trace-base');
    provider.addSpanProcessor(
      new SimpleSpanProcessor(new ConsoleSpanExporter())
    );
  }

  provider.register();

  return provider;
}

function hashTraceId(traceId: string): number {
  const hash = traceId.substring(0, 16);
  return parseInt(hash, 16) / 0xFFFFFFFFFFFFFFFF;
}

// 使用示例
export async function handleRequest(req: Request, res: Response) {
  const tracer = api.trace.getTracer('cgas-phase3/gateway');
  
  const span = tracer.startSpan('Gateway.handleRequest', {
    attributes: {
      'http.method': req.method,
      'http.url': req.url,
      'http.user_agent': req.headers['user-agent'],
    },
  });

  return api.context.with(api.trace.setSpan(api.context.active(), span), async () => {
    try {
      const result = await processRequest(req);
      span.setAttribute('http.status_code', res.statusCode);
      span.end();
      return result;
    } catch (error) {
      span.setStatus({ code: api.SpanStatusCode.ERROR });
      span.recordException(error);
      span.end();
      throw error;
    }
  });
}
```

---

## 5. OpenTelemetry Collector 配置

### 5.1 Collector 配置文件

```yaml
# otel-collector-config.yaml

receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318
  
  prometheus:
    config:
      scrape_configs:
        - job_name: 'cgas-executor'
          static_configs:
            - targets: ['executor:8080']
        - job_name: 'cgas-verifier'
          static_configs:
            - targets: ['verifier:8081']
        - job_name: 'cgas-batch'
          static_configs:
            - targets: ['batch:8082']

processors:
  batch:
    timeout: 5s
    send_batch_size: 512
    send_batch_max_size: 1024
  
  memory_limiter:
    check_interval: 1s
    limit_mib: 1000
    spike_limit_mib: 200
  
  # 采样处理器
  probabilistic_sampler:
    sampling_percentage: 10
  
  # 资源处理器（添加通用标签）
  resource:
    attributes:
      - key: phase
        value: phase3
        action: upsert
      - key: deployment.environment
        value: production
        action: upsert
  
  # 属性处理器（脱敏）
  attributes:
    actions:
      - key: http.authorization
        action: delete
      - key: db.statement
        action: hash

exporters:
  # Tempo 导出器
  tempo:
    endpoint: tempo:4317
    tls:
      insecure: true
  
  # Jaeger 导出器
  jaeger:
    endpoint: jaeger:4317
    tls:
      insecure: true
  
  # Prometheus 导出器（追踪指标）
  prometheus:
    endpoint: 0.0.0.0:8889
    namespace: cgas
  
  # 日志导出器（调试）
  logging:
    loglevel: debug

extensions:
  health_check:
    endpoint: 0.0.0.0:13133
  
  pprof:
    endpoint: 0.0.0.0:1777
  
  zpages:
    endpoint: 0.0.0.0:55679

service:
  extensions: [health_check, pprof, zpages]
  
  pipelines:
    traces:
      receivers: [otlp]
      processors: [memory_limiter, batch, resource, attributes, probabilistic_sampler]
      exporters: [tempo, jaeger, logging]
    
    metrics:
      receivers: [otlp, prometheus]
      processors: [memory_limiter, batch, resource]
      exporters: [prometheus, logging]
```

### 5.2 Docker Compose 部署

```yaml
# docker-compose.observability.yaml

version: '3.8'

services:
  # OpenTelemetry Collector
  otel-collector:
    image: otel/opentelemetry-collector-contrib:0.95.0
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP
      - "8889:8889"   # Prometheus metrics
      - "13133:13133" # Health check
    depends_on:
      - tempo
      - jaeger
    networks:
      - observability

  # Tempo (Trace Storage)
  tempo:
    image: grafana/tempo:2.4.0
    command: ["-config.file=/etc/tempo.yaml"]
    volumes:
      - ./tempo.yaml:/etc/tempo.yaml
      - tempo-data:/tmp/tempo
    ports:
      - "3200:3200"   # Tempo API
    networks:
      - observability

  # Jaeger (Trace Query)
  jaeger:
    image: jaegertracing/all-in-one:1.55
    environment:
      - COLLECTOR_OTLP_ENABLED=true
    ports:
      - "16686:16686" # Jaeger UI
    networks:
      - observability

  # Prometheus (Metrics Storage)
  prometheus:
    image: prom/prometheus:v2.50.1
    command:
      - "--config.file=/etc/prometheus/prometheus.yaml"
      - "--storage.tsdb.path=/prometheus"
    volumes:
      - ./prometheus.yaml:/etc/prometheus/prometheus.yaml
      - prometheus-data:/prometheus
    ports:
      - "9090:9090"
    networks:
      - observability

  # Grafana (Visualization)
  grafana:
    image: grafana/grafana:10.3.4
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning
      - ./grafana/dashboards:/var/lib/grafana/dashboards
    ports:
      - "3000:3000"
    depends_on:
      - prometheus
      - tempo
    networks:
      - observability

volumes:
  tempo-data:
  prometheus-data:
  grafana-data:

networks:
  observability:
    driver: bridge
```

---

## 6. 追踪指标

### 6.1 5 个核心追踪指标

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **M-025** | distributed_trace_coverage | Gauge | 5min | <98% | Trace 覆盖率 |
| **M-035** | trace_span_duration_p99 | Histogram | 实时 | >500ms | Span 时长 P99 |
| **M-051** | trace_total_duration_p99 | Histogram | 实时 | >1000ms | 全链路时长 P99 |
| **M-052** | trace_span_count_avg | Gauge | 5min | - | 平均 Span 数量 |
| **M-053** | trace_propagation_success_rate | Gauge | 30s | <99% | 追踪传递成功率 |

### 6.2 指标采集实现

```rust
// src/tracing/metrics.rs

use prometheus::{Histogram, Gauge, Counter, HistogramOpts, Opts, register_histogram, register_gauge, register_counter};

lazy_static! {
    /// Trace 覆盖率
    pub static ref DISTRIBUTED_TRACE_COVERAGE: Gauge = register_gauge!(
        Opts::new("distributed_trace_coverage", "Distributed trace coverage percentage")
    ).unwrap();
    
    /// Span 时长 P99
    pub static ref TRACE_SPAN_DURATION_P99: Histogram = register_histogram!(
        HistogramOpts::new("trace_span_duration_p99", "Trace span duration P99 in ms")
            .buckets(vec![10.0, 50.0, 100.0, 200.0, 300.0, 500.0, 750.0, 1000.0, 2500.0])
    ).unwrap();
    
    /// 全链路时长 P99
    pub static ref TRACE_TOTAL_DURATION_P99: Histogram = register_histogram!(
        HistogramOpts::new("trace_total_duration_p99", "Total trace duration P99 in ms")
            .buckets(vec![100.0, 250.0, 500.0, 750.0, 1000.0, 1500.0, 2000.0, 3000.0, 5000.0])
    ).unwrap();
    
    /// 平均 Span 数量
    pub static ref TRACE_SPAN_COUNT_AVG: Gauge = register_gauge!(
        Opts::new("trace_span_count_avg", "Average span count per trace")
    ).unwrap();
    
    /// 追踪传递成功率
    pub static ref TRACE_PROPAGATION_SUCCESS_RATE: Gauge = register_gauge!(
        Opts::new("trace_propagation_success_rate", "Trace propagation success rate percentage")
    ).unwrap();
}

/// 记录 Trace 完成
pub fn record_trace_completion(
    trace_duration_ms: u64,
    span_count: usize,
    propagation_success: bool,
) {
    TRACE_TOTAL_DURATION_P99.observe(trace_duration_ms as f64);
    TRACE_SPAN_COUNT_AVG.set(span_count as f64);
    
    if propagation_success {
        let current_rate = TRACE_PROPAGATION_SUCCESS_RATE.get();
        TRACE_PROPAGATION_SUCCESS_RATE.set((current_rate * 100.0 + 1.0) / 101.0);
    } else {
        let current_rate = TRACE_PROPAGATION_SUCCESS_RATE.get();
        TRACE_PROPAGATION_SUCCESS_RATE.set((current_rate * 100.0) / 101.0);
    }
}

/// 记录 Span 完成
pub fn record_span_completion(span_duration_ms: u64) {
    TRACE_SPAN_DURATION_P99.observe(span_duration_ms as f64);
}

/// 更新覆盖率
pub fn update_coverage(covered_paths: usize, total_paths: usize) {
    let coverage = covered_paths as f64 / total_paths as f64 * 100.0;
    DISTRIBUTED_TRACE_COVERAGE.set(coverage);
}
```

---

## 7. 验证与验收

### 7.1 验收标准

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| SDK 集成 | Rust+TS 均完成 | 代码审查 | 100% 集成 |
| Collector 部署 | 正常运行 | 健康检查 | 状态 OK |
| 数据导出 | Tempo/Jaeger 可查询 | 查询验证 | 可查询 Trace |
| 指标采集 | 5 个追踪指标正常 | Prometheus 查询 | 数据持续上报 |
| 性能开销 | <1% | 压测对比 | 时延增加<1% |

### 7.2 快速验证命令

```bash
# 检查 Collector 健康状态
curl http://localhost:13133

# 查询 Prometheus 追踪指标
curl 'http://localhost:9090/api/v1/query?query=distributed_trace_coverage'

# 查询 Tempo Trace
curl 'http://localhost:3200/api/search?q=Executor.execute'

# 访问 Jaeger UI
open http://localhost:16686
```

---

## 8. 实施计划

| 任务 | 责任人 | 状态 | 交付物 |
|---|---|---|---|
| Rust SDK 集成 | Dev | ✅ 完成 | tracing_config.rs |
| TypeScript SDK 集成 | Dev | ✅ 完成 | tracing.ts |
| Collector 配置 | SRE | ✅ 完成 | otel-collector-config.yaml |
| Docker Compose 部署 | SRE | ✅ 完成 | docker-compose.observability.yaml |
| 追踪指标集成 | Observability | ✅ 完成 | tracing_metrics.rs |

---

**文档状态**: ✅ 已完成  
**创建日期**: 2026-03-07  
**责任人**: Observability-Agent  
**保管**: 项目文档库
