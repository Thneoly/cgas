# Phase 3 分布式追踪全链路设计

**版本**: v1.0  
**日期**: 2026-05-12  
**责任人**: Observability-Agent  
**状态**: 📋 设计中  
**release_id**: release-2026-05-12-phase3_week01  
**关联文档**: 
- phase3_50_metrics_plan.md (55 指标规划)
- observability_50_metrics_architecture.md (可观测性架构)
- monitoring_dashboard_v5.md (Phase 2 仪表盘基线)

---

## 1. 设计目标

### 1.1 Phase 2 vs Phase 3 追踪能力对比

| 指标 | Phase 2 实际 | Phase 3 目标 | 提升幅度 |
|---|---|---|---|
| Trace 覆盖率 | 80% | **≥99%** | +24% |
| 追踪指标数量 | 0 个 | **5 个** | 新增 |
| Span 采集延迟 | <5s | <2s | -60% |
| 全链路关联率 | 75% | ≥98% | +31% |
| Trace 数据保留 | 7 天 | 30 天 | +329% |

### 1.2 核心设计原则

| 原则 | 说明 | 验收标准 |
|---|---|---|
| **零侵入** | 业务代码无感知 | 通过注解/宏自动埋点 |
| **低开销** | 性能影响<1% | 压测验证 |
| **全链路** | trace_id 贯穿所有组件 | 关联率≥98% |
| **可采样** | 支持动态调整采样率 | 1%~100% 可调 |
| **标准化** | OpenTelemetry 协议 | 兼容 Tempo/Jaeger |

---

## 2. OpenTelemetry 集成方案

### 2.1 技术选型

| 组件 | 选型 | 理由 |
|---|---|---|
| 追踪协议 | OpenTelemetry Protocol (OTLP) | CNCF 标准，多后端兼容 |
| 采集器 | OpenTelemetry Collector | 统一采集、处理、导出 |
| 后端存储 | Tempo + Jaeger | Tempo(低成本) + Jaeger(查询) |
| 可视化 | Grafana Tempo 插件 | 与监控仪表盘统一 |
| Rust SDK | opentelemetry-rust | 官方支持，性能优异 |
| TypeScript SDK | @opentelemetry/api | 官方支持 |

### 2.2 架构设计

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

### 2.3 Rust SDK 集成

#### 2.3.1 初始化配置

```rust
// tracing_config.rs - OpenTelemetry 初始化配置

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
            otlp_endpoint: "http://otel-collector:4317".to_string(),
            service_name: "cgas-phase3".to_string(),
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
    
    Ok(tracer_provider)
}

/// 关闭追踪（优雅退出时调用）
pub fn shutdown_opentelemetry() {
    global::shutdown_tracer_provider();
}
```

#### 2.3.2 自动埋点宏

```rust
// tracing_macros.rs - 自动埋点宏定义

use opentelemetry::{global, trace::{Span, Tracer, TraceContextExt}, Context, KeyValue};
use opentelemetry_semantic_conventions as semconv;

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
```

#### 2.3.3 执行器埋点示例

```rust
// executor.rs - 执行器追踪埋点

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
        // ...
        Ok(ExecutionResult::default())
    }
}
```

### 2.4 TypeScript SDK 集成

```typescript
// tracing.ts - Gateway 追踪初始化

import * as api from '@opentelemetry/api';
import { NodeTracerProvider } from '@opentelemetry/sdk-trace-node';
import { BatchSpanProcessor } from '@opentelemetry/sdk-trace-base';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc';
import { Resource } from '@opentelemetry/resources';
import { SemanticResourceAttributes } from '@opentelemetry/semantic-conventions';
import { SimpleSpanProcessor, ConsoleSpanExporter } from '@opentelemetry/sdk-trace-base';

interface TracingConfig {
  otlpEndpoint: string;
  serviceName: string;
  sampleRate: number;
}

export function initTracing(config: TracingConfig): api.TracerProvider {
  const provider = new NodeTracerProvider({
    resource: new Resource({
      [SemanticResourceAttributes.SERVICE_NAME]: config.serviceName,
      [SemanticResourceAttributes.SERVICE_VERSION]: process.env.npm_package_version,
      ['deployment.environment']: 'production',
      ['phase']: 'phase3',
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
      // 业务逻辑
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

### 2.5 Trace Context 传递

```rust
// trace_context.rs - 追踪上下文传递

use opentelemetry::{global, trace::TraceContextExt, Context, KeyValue};
use std::collections::HashMap;

/// 从当前 Span 提取追踪上下文
pub fn extract_trace_context() -> HashMap<String, String> {
    let cx = Context::current();
    let span = cx.span();
    let span_context = span.span_context();
    
    let mut context = HashMap::new();
    
    if span_context.is_valid() {
        context.insert("trace_id".to_string(), span_context.trace_id().to_string());
        context.insert("span_id".to_string(), span_context.span_id().to_string());
        context.insert("trace_flags".to_string(), format!("{}", span_context.trace_flags()));
    }
    
    context
}

/// 将追踪上下文注入到 HTTP 请求头
pub fn inject_to_headers(headers: &mut HashMap<String, String>) {
    let context = extract_trace_context();
    
    if let Some(trace_id) = context.get("trace_id") {
        headers.insert("X-Trace-ID".to_string(), trace_id.clone());
    }
    if let Some(span_id) = context.get("span_id") {
        headers.insert("X-Span-ID".to_string(), span_id.clone());
    }
}

/// 从 HTTP 请求头提取追踪上下文
pub fn extract_from_headers(headers: &HashMap<String, String>) -> Option<Context> {
    let trace_id = headers.get("X-Trace-ID")?;
    let span_id = headers.get("X-Span-ID")?;
    
    // 创建父 Span 上下文
    // 实际实现需要使用 opentelemetry 的 propagator
    None
}
```

---

## 3. Trace 覆盖率提升策略

### 3.1 覆盖率现状分析

| 组件 | Phase 2 覆盖率 | 问题 | Phase 3 目标 |
|---|---|---|---|
| Executor | 90% | 部分错误路径未覆盖 | 100% |
| Verifier | 85% | 缓存路径未追踪 | 100% |
| Batch 服务 | 70% | 嵌套 Batch 未追踪 | 100% |
| Transaction 服务 | 60% | 隔离级别切换未追踪 | 100% |
| Gateway | 80% | 错误处理未追踪 | 100% |
| 零信任模块 | 50% | 策略评估未追踪 | 99% |
| **整体** | **80%** | - | **≥99%** |

### 3.2 覆盖率提升方案

#### 3.2.1 关键路径全覆盖

```rust
// coverage_plan.rs - 关键路径埋点计划

/// Phase 3 必须覆盖的关键路径
pub const CRITICAL_PATHS: &[&str] = &[
    // === Executor (100% 覆盖) ===
    "Executor.execute_instruction",
    "Executor.execute_batch",
    "Executor.handle_timeout",
    "Executor.handle_panic",
    
    // === Verifier (100% 覆盖) ===
    "Verifier.verify_result",
    "Verifier.replay_execution",
    "Verifier.check_consistency",
    "Verifier.cache_hit",
    "Verifier.cache_miss",
    
    // === Batch 服务 (100% 覆盖) ===
    "BatchExecutor.execute",
    "BatchExecutor.execute_nested",
    "BatchContext.create",
    "BatchContext.close",
    "BatchContext.commit",
    "BatchContext.rollback",
    
    // === Transaction 服务 (100% 覆盖) ===
    "TransactionManager.begin",
    "TransactionManager.commit",
    "TransactionManager.rollback",
    "TransactionManager.isolation_level_switch",
    "TransactionManager.deadlock_detection",
    
    // === Gateway (100% 覆盖) ===
    "Gateway.receive_request",
    "Gateway.route_request",
    "Gateway.handle_error",
    "Gateway.authenticate",
    
    // === 零信任模块 (99% 覆盖) ===
    "ZeroTrust.authenticate",
    "ZeroTrust.authorize",
    "ZeroTrust.policy_evaluate",
    "ZeroTrust.audit_log",
];

/// 验证覆盖率
pub fn verify_coverage() -> CoverageReport {
    let total_paths = CRITICAL_PATHS.len();
    let covered_paths = count_covered_paths();
    let coverage_rate = covered_paths as f64 / total_paths as f64 * 100.0;
    
    CoverageReport {
        total_paths,
        covered_paths,
        coverage_rate,
        timestamp: chrono::Utc::now(),
    }
}
```

#### 3.2.2 错误路径追踪

```rust
// error_tracing.rs - 错误路径追踪

use opentelemetry::{trace::Status, KeyValue};

/// 统一错误追踪包装器
pub fn trace_error_result<T, E: std::fmt::Display>(
    result: Result<T, E>,
    span: &dyn opentelemetry::trace::Span,
    error_context: &str,
) -> Result<T, E> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            span.set_status(Status::error(format!("{}", error)));
            span.record_error(&opentelemetry::trace::Error::from(error.to_string()));
            span.add_event(
                "error_occurred",
                vec![
                    KeyValue::new("error.context", error_context),
                    KeyValue::new("error.message", error.to_string()),
                ],
            );
            Err(error)
        }
    }
}

/// Panic 捕获与追踪
pub fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let tracer = opentelemetry::global::tracer("cgas-phase3/panic");
        let span = tracer.span("PanicCaught").start();
        
        if let Some(location) = panic_info.location() {
            span.add_event(
                "panic",
                vec![
                    KeyValue::new("panic.file", location.file()),
                    KeyValue::new("panic.line", location.line() as i64),
                ],
            );
        }
        
        if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            span.add_event(
                "panic_message",
                vec![KeyValue::new("panic.message", message.to_string())],
            );
        }
        
        span.end();
    }));
}
```

#### 3.2.3 异步操作追踪

```rust
// async_tracing.rs - 异步操作追踪

use opentelemetry::{global, trace::Tracer, Context, KeyValue};
use std::future::Future;
use std::pin::Pin;

/// 追踪异步函数
pub async fn trace_async<F, T>(
    span_name: &str,
    attributes: Vec<KeyValue>,
    future: F,
) -> T
where
    F: Future<Output = T>,
{
    let tracer = global::tracer("cgas-phase3");
    let span = tracer
        .span_builder(span_name)
        .with_attributes(attributes)
        .start(&tracer);
    
    let cx = Context::current_with_span(span);
    let _guard = cx.attach();
    
    future.await
}

// 使用示例
pub async fn process_batch_async(batch: Batch) -> Result<BatchResult, Error> {
    trace_async(
        "BatchExecutor.process_async",
        vec![
            KeyValue::new("batch.id", batch.id.clone()),
            KeyValue::new("batch.size", batch.instructions.len() as i64),
        ],
        async move {
            // 异步处理逻辑
            Ok(BatchResult::default())
        },
    ).await
}
```

### 3.3 采样策略

```rust
// sampling_strategy.rs - 动态采样策略

use opentelemetry_sdk::trace::{Sampler, ShouldSample};
use opentelemetry::{trace::SpanKind, Context, KeyValue};

/// 动态采样器配置
#[derive(Debug, Clone)]
pub struct DynamicSampler {
    /// 基础采样率
    pub base_rate: f64,
    /// 错误请求采样率（总是采样）
    pub error_rate: f64,
    /// 慢请求采样率（总是采样）
    pub slow_rate: f64,
    /// 慢请求阈值 (ms)
    pub slow_threshold_ms: u64,
}

impl Default for DynamicSampler {
    fn default() -> Self {
        Self {
            base_rate: 0.1,        // 10% 基础采样
            error_rate: 1.0,       // 100% 错误采样
            slow_rate: 1.0,        // 100% 慢请求采样
            slow_threshold_ms: 500, // 500ms 阈值
        }
    }
}

impl ShouldSample for DynamicSampler {
    fn should_sample(
        &self,
        parent_context: Option<&Context>,
        _trace_id: opentelemetry::trace::TraceId,
        name: &str,
        _span_kind: &SpanKind,
        attributes: &[KeyValue],
        _links: &[opentelemetry::trace::Link],
    ) -> opentelemetry_sdk::trace::SamplingResult {
        // 检查是否有父 Span
        if let Some(parent) = parent_context {
            let parent_span = parent.span();
            if parent_span.span_context().is_sampled() {
                // 父 Span 已采样，子 Span 也采样
                return opentelemetry_sdk::trace::SamplingResult {
                    decision: opentelemetry_sdk::trace::SamplingDecision::RecordAndSample,
                    attributes: Vec::new(),
                    trace_state: Default::default(),
                };
            }
        }
        
        // 基于 Span 名称的特殊采样
        if name.contains("error") || name.contains("panic") {
            return opentelemetry_sdk::trace::SamplingResult {
                decision: opentelemetry_sdk::trace::SamplingDecision::RecordAndSample,
                attributes: Vec::new(),
                trace_state: Default::default(),
            };
        }
        
        // 基础采样率
        let should_sample = rand::random::<f64>() < self.base_rate;
        
        let decision = if should_sample {
            opentelemetry_sdk::trace::SamplingDecision::RecordAndSample
        } else {
            opentelemetry_sdk::trace::SamplingDecision::Drop
        };
        
        opentelemetry_sdk::trace::SamplingResult {
            decision,
            attributes: Vec::new(),
            trace_state: Default::default(),
        }
    }
}
```

---

## 4. Trace 层级设计

### 4.1 完整 Trace 层级

```
Phase 3 完整 Trace 层级 (trace_id: "abc123..."):

Trace (trace_id: "abc123...", service: "cgas-phase3")
│
├── Span: Gateway.receive_request (span_001)
│   ├── attributes:
│   │   ├── http.method = "POST"
│   │   ├── http.url = "/api/v1/execute"
│   │   ├── http.user_agent = "Feishu-Bot/1.0"
│   │   └── channel = "feishu"
│   ├── events:
│   │   └── request_received { timestamp: 100ms }
│   │
│   ├── Span: Gateway.authenticate (span_002)
│   │   ├── attributes:
│   │   │   ├── auth.method = "OIDC"
│   │   │   └── auth.result = "success"
│   │   └── duration: 15ms
│   │
│   ├── Span: Gateway.route_request (span_003)
│   │   ├── attributes:
│   │   │   └── route.target = "executor"
│   │   └── duration: 2ms
│   │
│   └── Span: Batch.execute (span_004)
│       ├── attributes:
│       │   ├── batch.id = "batch_001"
│       │   ├── batch.size = 5
│       │   └── batch.depth = 0
│       │
│       ├── Span: BatchContext.create (span_005)
│       │   ├── attributes:
│       │   │   └── context.id = "ctx_001"
│       │   └── duration: 3ms
│       │
│       ├── Span: Executor.execute (span_006)
│       │   ├── attributes:
│       │   │   ├── instruction.id = "instr_001"
│       │   │   └── instruction.type = "deploy"
│       │   │
│       │   ├── Span: Verifier.verify (span_007)
│       │   │   ├── attributes:
│       │   │   │   ├── verification.type = "replay"
│       │   │   │   └── cache.hit = true
│       │   │   ├── events:
│       │   │   │   └── cache_hit { timestamp: 120ms }
│       │   │   └── duration: 25ms
│       │   │
│       │   ├── Span: Commit.commit (span_008)
│       │   │   ├── attributes:
│       │   │   │   └── commit.hash = "sha256:abc..."
│       │   │   └── duration: 10ms
│       │   │
│       │   └── duration: 50ms
│       │
│       ├── Span: NestedBatch.execute (span_009)
│       │   ├── attributes:
│       │   │   ├── batch.id = "batch_002"
│       │   │   ├── batch.depth = 1
│       │   │   └── batch.parent = "batch_001"
│       │   │
│       │   ├── Span: Executor.execute (span_010)
│       │   │   ├── attributes:
│       │   │   │   └── instruction.id = "instr_002"
│       │   │   └── duration: 45ms
│       │   │
│       │   └── duration: 80ms
│       │
│       ├── Span: BatchContext.close (span_011)
│       │   ├── attributes:
│       │   │   └── context.status = "committed"
│       │   └── duration: 5ms
│       │
│       └── duration: 150ms
│
├── Span: Monitoring.record (span_012)
│   ├── attributes:
│   │   ├── metrics.count = 50
│   │   └── exporter = "prometheus"
│   └── duration: 20ms
│
└── Total Duration: 185ms
```

### 4.2 Span 属性规范

```rust
// span_attributes.rs - Span 属性规范

use opentelemetry::KeyValue;

/// 通用 Span 属性
pub mod common {
    use super::*;
    
    pub fn service_name(name: &str) -> KeyValue {
        KeyValue::new("service.name", name.to_string())
    }
    
    pub fn service_version(version: &str) -> KeyValue {
        KeyValue::new("service.version", version.to_string())
    }
    
    pub fn deployment_environment(env: &str) -> KeyValue {
        KeyValue::new("deployment.environment", env.to_string())
    }
    
    pub fn phase(phase: &str) -> KeyValue {
        KeyValue::new("phase", phase.to_string())
    }
}

/// Executor Span 属性
pub mod executor {
    use super::*;
    
    pub fn instruction_id(id: &str) -> KeyValue {
        KeyValue::new("instruction.id", id.to_string())
    }
    
    pub fn instruction_type(ty: &str) -> KeyValue {
        KeyValue::new("instruction.type", ty.to_string())
    }
    
    pub fn instruction_priority(priority: u8) -> KeyValue {
        KeyValue::new("instruction.priority", priority as i64)
    }
    
    pub fn execution_result(success: bool) -> KeyValue {
        KeyValue::new("execution.result", if success { "success" } else { "failure" })
    }
    
    pub fn execution_duration_ms(duration: u64) -> KeyValue {
        KeyValue::new("execution.duration_ms", duration as i64)
    }
}

/// Batch Span 属性
pub mod batch {
    use super::*;
    
    pub fn batch_id(id: &str) -> KeyValue {
        KeyValue::new("batch.id", id.to_string())
    }
    
    pub fn batch_size(size: usize) -> KeyValue {
        KeyValue::new("batch.size", size as i64)
    }
    
    pub fn batch_depth(depth: u8) -> KeyValue {
        KeyValue::new("batch.depth", depth as i64)
    }
    
    pub fn batch_parent_id(parent_id: &str) -> KeyValue {
        KeyValue::new("batch.parent_id", parent_id.to_string())
    }
    
    pub fn batch_status(status: &str) -> KeyValue {
        KeyValue::new("batch.status", status.to_string())
    }
}

/// Transaction Span 属性
pub mod transaction {
    use super::*;
    
    pub fn transaction_id(id: &str) -> KeyValue {
        KeyValue::new("transaction.id", id.to_string())
    }
    
    pub fn isolation_level(level: &str) -> KeyValue {
        KeyValue::new("transaction.isolation_level", level.to_string())
    }
    
    pub fn transaction_status(status: &str) -> KeyValue {
        KeyValue::new("transaction.status", status.to_string())
    }
}

/// Verifier Span 属性
pub mod verifier {
    use super::*;
    
    pub fn verification_type(ty: &str) -> KeyValue {
        KeyValue::new("verification.type", ty.to_string())
    }
    
    pub fn cache_hit(hit: bool) -> KeyValue {
        KeyValue::new("verification.cache_hit", hit)
    }
    
    pub fn consistency_check(passed: bool) -> KeyValue {
        KeyValue::new("verification.consistency_passed", passed)
    }
}
```

---

## 5. 追踪指标

### 5.1 5 个核心追踪指标

根据 `phase3_50_metrics_plan.md`，Phase 3 新增 5 个追踪指标：

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **M-025** | distributed_trace_coverage | Gauge | 5min | <98% | Trace 覆盖率 |
| **M-035** | trace_span_duration_p99 | Histogram | 实时 | >500ms | Span 时长 P99 |
| **M-051** | trace_total_duration_p99 | Histogram | 实时 | >1000ms | 全链路时长 P99 |
| **M-052** | trace_span_count_avg | Gauge | 5min | - | 平均 Span 数量 |
| **M-053** | trace_propagation_success_rate | Gauge | 30s | <99% | 追踪传递成功率 |

### 5.2 指标采集实现

```rust
// tracing_metrics.rs - 追踪指标采集

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

## 6. OpenTelemetry Collector 配置

### 6.1 Collector 配置文件

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

### 6.2 Docker Compose 部署

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
      - "4317:4317"   # OTLP gRPC
    networks:
      - observability

  # Jaeger (Trace Query)
  jaeger:
    image: jaegertracing/all-in-one:1.55
    environment:
      - COLLECTOR_OTLP_ENABLED=true
    ports:
      - "16686:16686" # Jaeger UI
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP
    networks:
      - observability

  # Prometheus (Metrics Storage)
  prometheus:
    image: prom/prometheus:v2.50.1
    command:
      - "--config.file=/etc/prometheus/prometheus.yaml"
      - "--storage.tsdb.path=/prometheus"
      - "--web.console.libraries=/etc/prometheus/console_libraries"
      - "--web.console.templates=/etc/prometheus/consoles"
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

## 7. 验证与验收

### 7.1 验收标准

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| Trace 覆盖率 | ≥99% | 关键路径扫描 | 所有 CRITICAL_PATHS 覆盖 |
| 追踪传递成功率 | ≥99% | 链路追踪验证 | 跨服务 trace_id 一致 |
| Span 时长 P99 | <500ms | Prometheus 查询 | histogram_quantile(0.99) |
| 全链路时长 P99 | <1000ms | Tempo 查询 | Trace 总时长 |
| 采集开销 | <1% | 压测对比 | 开启/关闭追踪性能对比 |
| 数据保留 | 30 天 | 历史查询 | 可查询 30 天前 Trace |

### 7.2 验证脚本

```python
#!/usr/bin/env python3
"""
Phase 3 Trace 覆盖率验证脚本
"""

import requests
import json
from datetime import datetime, timedelta

TEMPO_URL = "http://localhost:3200"
PROMETHEUS_URL = "http://localhost:9090"

# 关键路径列表
CRITICAL_PATHS = [
    "Executor.execute_instruction",
    "Verifier.verify_result",
    "BatchExecutor.execute",
    "TransactionManager.commit",
    "Gateway.handleRequest",
]

def check_trace_coverage():
    """检查 Trace 覆盖率"""
    query = "distributed_trace_coverage"
    response = requests.get(
        f"{PROMETHEUS_URL}/api/v1/query",
        params={"query": query}
    )
    
    if response.status_code == 200:
        data = response.json()
        if data["status"] == "success":
            coverage = float(data["data"]["result"][0]["value"][1])
            return coverage
    
    return 0.0

def check_span_duration_p99():
    """检查 Span 时长 P99"""
    query = "histogram_quantile(0.99, rate(trace_span_duration_p99_bucket[5m]))"
    response = requests.get(
        f"{PROMETHEUS_URL}/api/v1/query",
        params={"query": query}
    )
    
    if response.status_code == 200:
        data = response.json()
        if data["status"] == "success":
            p99 = float(data["data"]["result"][0]["value"][1])
            return p99
    
    return 0.0

def check_trace_propagation():
    """检查追踪传递成功率"""
    query = "trace_propagation_success_rate"
    response = requests.get(
        f"{PROMETHEUS_URL}/api/v1/query",
        params={"query": query}
    )
    
    if response.status_code == 200:
        data = response.json()
        if data["status"] == "success":
            rate = float(data["data"]["result"][0]["value"][1])
            return rate
    
    return 0.0

def verify_critical_paths():
    """验证关键路径覆盖"""
    covered_paths = []
    missing_paths = []
    
    for path in CRITICAL_PATHS:
        # 查询 Tempo 是否存在该 Span
        response = requests.get(
            f"{TEMPO_URL}/api/search",
            params={"q": path, "limit": 1}
        )
        
        if response.status_code == 200:
            data = response.json()
            if data.get("traces"):
                covered_paths.append(path)
            else:
                missing_paths.append(path)
    
    return covered_paths, missing_paths

def main():
    print(f"开始验证 Phase 3 分布式追踪... ({datetime.now()})")
    print("=" * 60)
    
    # 检查覆盖率
    coverage = check_trace_coverage()
    print(f"\n1. Trace 覆盖率：{coverage:.2f}%")
    if coverage >= 99:
        print("   ✅ 通过 (≥99%)")
    else:
        print("   ❌ 未通过 (<99%)")
    
    # 检查 Span 时长 P99
    p99 = check_span_duration_p99()
    print(f"\n2. Span 时长 P99: {p99:.2f}ms")
    if p99 < 500:
        print("   ✅ 通过 (<500ms)")
    else:
        print("   ❌ 未通过 (≥500ms)")
    
    # 检查追踪传递成功率
    propagation_rate = check_trace_propagation()
    print(f"\n3. 追踪传递成功率：{propagation_rate:.2f}%")
    if propagation_rate >= 99:
        print("   ✅ 通过 (≥99%)")
    else:
        print("   ❌ 未通过 (<99%)")
    
    # 验证关键路径
    covered, missing = verify_critical_paths()
    print(f"\n4. 关键路径覆盖：{len(covered)}/{len(CRITICAL_PATHS)}")
    if covered:
        print("   已覆盖路径:")
        for path in covered:
            print(f"     ✅ {path}")
    if missing:
        print("   缺失路径:")
        for path in missing:
            print(f"     ❌ {path}")
    
    # 生成报告
    report = {
        "timestamp": datetime.now().isoformat(),
        "coverage_rate": coverage,
        "span_duration_p99": p99,
        "propagation_rate": propagation_rate,
        "critical_paths_covered": len(covered),
        "critical_paths_total": len(CRITICAL_PATHS),
        "missing_paths": missing,
        "passed": coverage >= 99 and p99 < 500 and propagation_rate >= 99 and len(missing) == 0,
    }
    
    with open("tracing_validation_report.json", "w") as f:
        json.dump(report, f, indent=2)
    
    print(f"\n报告已保存至：tracing_validation_report.json")
    print("=" * 60)
    
    if report["passed"]:
        print("✅ Phase 3 分布式追踪验证通过")
        return 0
    else:
        print("❌ Phase 3 分布式追踪验证失败")
        return 1

if __name__ == "__main__":
    exit(main())
```

---

## 8. 实施计划

### 8.1 时间规划

| 周次 | 任务 | 责任人 | 状态 | 交付物 |
|---|---|---|---|---|
| Week 1-T1 | OpenTelemetry SDK 集成 | Dev | 📋 待开始 | tracing_config.rs |
| Week 1-T2 | 关键路径埋点 | Dev+Observability | 📋 待开始 | 埋点代码 |
| Week 1-T3 | Collector 部署 | SRE | 📋 待开始 | otel-collector-config.yaml |
| Week 2-T1 | Trace 覆盖率提升 | Dev+Observability | 📋 待开始 | coverage_report.md |
| Week 2-T2 | 追踪指标接入 | Observability | 📋 待开始 | tracing_metrics.rs |
| Week 3-T1 | Grafana 仪表盘 v6 | SRE+Observability | 📋 待开始 | monitoring_dashboard_v6.md |
| Week 3-T2 | 全链路验证 | QA+Observability | 📋 待开始 | tracing_validation_report.md |

### 8.2 风险与缓解

| 风险 | 影响 | 概率 | 缓解措施 |
|---|---|---|---|
| 性能开销>1% | 高 | 中 | 动态采样、异步导出 |
| Trace 丢失 | 中 | 低 | 多后端导出、重试机制 |
| 埋点遗漏 | 中 | 中 | 自动化扫描、代码审查 |
| 数据量过大 | 高 | 中 | 采样策略、数据保留策略 |

---

## 9. 附录

### 9.1 参考文档

| 文档 | 链接 |
|---|---|
| OpenTelemetry 官方文档 | https://opentelemetry.io/docs/ |
| opentelemetry-rust | https://github.com/open-telemetry/opentelemetry-rust |
| Tempo 文档 | https://grafana.com/docs/tempo/ |
| Jaeger 文档 | https://www.jaegertracing.io/docs/ |

### 9.2 相关文档

| 文档 | 路径 |
|---|---|
| Phase 3 50 指标规划 | phase3_50_metrics_plan.md |
| 可观测性架构设计 | observability_50_metrics_architecture.md |
| Phase 2 监控仪表盘 v5 | monitoring_dashboard_v5.md |

---

**文档状态**: 📋 设计中  
**创建日期**: 2026-05-12  
**责任人**: Observability-Agent  
**保管**: 项目文档库
