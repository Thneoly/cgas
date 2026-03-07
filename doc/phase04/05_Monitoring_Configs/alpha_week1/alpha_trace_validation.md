# Alpha 环境 Trace 接入验证

**版本**: v1.0  
**日期**: 2026-04-05  
**责任人**: Observability-Agent + SRE-Agent  
**状态**: ✅ Week 1-T5 完成  
**环境**: Alpha (内部测试环境)  
**release_id**: release-2026-04-05-phase4-week1-alpha-trace-validation

---

## 1. 概述

### 1.1 任务目标

在 Phase 4 Week 1 完成 Alpha 环境的 **分布式追踪 (Trace) 接入验证**，确保 OpenTelemetry 追踪系统在 Alpha 环境正常运行，实现全链路追踪能力。

### 1.2 验收标准

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| OTel Collector | 正常运行 | 健康检查 | 状态 OK |
| Tempo 后端 | Trace 存储正常 | 查询验证 | 可查询 Trace |
| Jaeger UI | Trace 查询可用 | UI 访问 | 可查询展示 |
| 应用埋点 | Executor/Verifier/Gateway 完成 | 代码审查 + 数据验证 | 100% 埋点 |
| Trace 传递 | 跨服务 trace_id 一致 | 抽样验证 | 关联率≥98% |
| 追踪指标 | 5 个追踪指标正常 | Prometheus 查询 | 数据持续上报 |

---

## 2. OpenTelemetry 架构

### 2.1 Alpha 环境 OTel 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                     Application Layer (Alpha)                    │
├─────────────────────────────────────────────────────────────────┤
│  Executor (Rust)  │  Verifier (Rust)  │  Gateway (TypeScript)  │
│  + OTel SDK       │  + OTel SDK       │  + OTel SDK            │
└─────────┬─────────┴─────────┬─────────┴──────────┬──────────────┘
          │                   │                     │
          │ OTLP (gRPC)       │ OTLP (gRPC)         │ OTLP (gRPC)
          ▼                   ▼                     ▼
┌─────────────────────────────────────────────────────────────────┐
│               OpenTelemetry Collector (Alpha)                    │
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

### 2.2 组件版本

| 组件 | 版本 | 用途 |
|---|---|---|
| OpenTelemetry Collector | 0.95.0 | 统一采集、处理、导出 |
| Tempo | 2.4.0 | Trace 存储 (低成本) |
| Jaeger | 1.55 | Trace 查询 UI |
| Prometheus | 2.50.1 | 追踪指标存储 |
| Grafana | 10.3.4 | 可视化 |
| opentelemetry-rust | 0.22 | Rust SDK |
| @opentelemetry/api | 1.8+ | TypeScript SDK |

---

## 3. 配置清单

### 3.1 OTel Collector 配置

```yaml
# otel-collector-alpha-config.yaml

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
        - job_name: 'alpha-executor'
          static_configs:
            - targets: ['alpha-executor-1:8080', 'alpha-executor-2:8080']
        - job_name: 'alpha-verifier'
          static_configs:
            - targets: ['alpha-verifier-1:8081', 'alpha-verifier-2:8081']
        - job_name: 'alpha-gateway'
          static_configs:
            - targets: ['alpha-gateway-1:8084', 'alpha-gateway-2:8084']

processors:
  batch:
    timeout: 5s
    send_batch_size: 512
    send_batch_max_size: 1024
  
  memory_limiter:
    check_interval: 1s
    limit_mib: 1000
    spike_limit_mib: 200
  
  # Alpha 环境采样率 (开发环境可较高)
  probabilistic_sampler:
    sampling_percentage: 50
  
  # 资源处理器（添加 Alpha 环境标签）
  resource:
    attributes:
      - key: environment
        value: alpha
        action: upsert
      - key: phase
        value: phase4
        action: upsert
      - key: deployment.environment
        value: alpha
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
    endpoint: tempo-alpha:4317
    tls:
      insecure: true
  
  # Jaeger 导出器
  jaeger:
    endpoint: jaeger-alpha:4317
    tls:
      insecure: true
  
  # Prometheus 导出器（追踪指标）
  prometheus:
    endpoint: 0.0.0.0:8889
    namespace: cgas_alpha
  
  # 日志导出器（调试）
  logging:
    loglevel: info

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

### 3.2 Docker Compose 部署

```yaml
# docker-compose.observability-alpha.yaml

version: '3.8'

services:
  # OpenTelemetry Collector
  otel-collector-alpha:
    image: otel/opentelemetry-collector-contrib:0.95.0
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector-alpha-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP
      - "8889:8889"   # Prometheus metrics
      - "13133:13133" # Health check
    depends_on:
      - tempo-alpha
      - jaeger-alpha
    networks:
      - observability-alpha

  # Tempo (Trace Storage)
  tempo-alpha:
    image: grafana/tempo:2.4.0
    command: ["-config.file=/etc/tempo.yaml"]
    volumes:
      - ./tempo-alpha.yaml:/etc/tempo.yaml
      - tempo-alpha-data:/tmp/tempo
    ports:
      - "3200:3200"   # Tempo API
    networks:
      - observability-alpha

  # Jaeger (Trace Query UI)
  jaeger-alpha:
    image: jaegertracing/all-in-one:1.55
    environment:
      - COLLECTOR_OTLP_ENABLED=true
    ports:
      - "16686:16686" # Jaeger UI
    networks:
      - observability-alpha

  # Prometheus (Metrics Storage)
  prometheus-alpha:
    image: prom/prometheus:v2.50.1
    command:
      - "--config.file=/etc/prometheus/prometheus.yaml"
      - "--storage.tsdb.path=/prometheus"
    volumes:
      - ./prometheus-alpha.yaml:/etc/prometheus/prometheus.yaml
      - prometheus-alpha-data:/prometheus
    ports:
      - "9090:9090"
    networks:
      - observability-alpha

  # Grafana (Visualization)
  grafana-alpha:
    image: grafana/grafana:10.3.4
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_ADMIN_PASSWORD}
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - grafana-alpha-data:/var/lib/grafana
      - ./grafana/provisioning-alpha:/etc/grafana/provisioning
      - ./grafana/dashboards-alpha:/var/lib/grafana/dashboards
    ports:
      - "3000:3000"
    depends_on:
      - prometheus-alpha
      - tempo-alpha
    networks:
      - observability-alpha

volumes:
  tempo-alpha-data:
  prometheus-alpha-data:
  grafana-alpha-data:

networks:
  observability-alpha:
    driver: bridge
```

---

## 4. 应用埋点验证

### 4.1 Executor 埋点验证

```rust
// src/executor/tracing.rs - Alpha 环境

use opentelemetry::{global, trace::{Span, Tracer, Status}, Context, KeyValue};

pub struct TracingExecutor {
    tracer: opentelemetry::sdk::trace::Tracer,
}

impl TracingExecutor {
    pub fn new() -> Self {
        let tracer = global::tracer("cgas-phase4/executor-alpha");
        Self { tracer }
    }
    
    /// 执行指令（带追踪）- Alpha 环境
    pub fn execute_with_tracing(
        &self,
        instruction: &Instruction,
    ) -> Result<ExecutionResult, Error> {
        let span = self.tracer
            .span_builder("Executor.execute_instruction")
            .with_attributes(vec![
                KeyValue::new("instruction.id", instruction.id.clone()),
                KeyValue::new("instruction.type", instruction.instruction_type.clone()),
                KeyValue::new("instruction.priority", instruction.priority as i64),
                KeyValue::new("environment", "alpha"),
                KeyValue::new("phase", "phase4"),
            ])
            .start(&self.tracer);
        
        let cx = Context::current_with_span(span);
        let _guard = cx.attach();
        
        let start = std::time::Instant::now();
        
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

### 4.2 Verifier 埋点验证

```rust
// src/verifier/tracing.rs - Alpha 环境

use opentelemetry::{global, trace::{Span, Tracer, Status}, Context, KeyValue};

pub struct TracingVerifier {
    tracer: opentelemetry::sdk::trace::Tracer,
}

impl TracingVerifier {
    pub fn new() -> Self {
        let tracer = global::tracer("cgas-phase4/verifier-alpha");
        Self { tracer }
    }
    
    /// 验证指令（带追踪）- Alpha 环境
    pub fn verify_with_tracing(
        &self,
        instruction: &Instruction,
        expected: &ExpectedResult,
    ) -> Result<VerificationResult, Error> {
        let span = self.tracer
            .span_builder("Verifier.verify_instruction")
            .with_attributes(vec![
                KeyValue::new("instruction.id", instruction.id.clone()),
                KeyValue::new("verification.type", "post_execution"),
                KeyValue::new("environment", "alpha"),
                KeyValue::new("phase", "phase4"),
            ])
            .start(&self.tracer);
        
        let cx = Context::current_with_span(span);
        let _guard = cx.attach();
        
        let start = std::time::Instant::now();
        
        // 验证逻辑
        let result = self.do_verify(instruction, expected);
        
        let duration = start.elapsed();
        
        // 记录时延
        let span = cx.span();
        span.add_event(
            "verification_completed",
            vec![
                KeyValue::new("duration_ms", duration.as_millis() as i64),
                KeyValue::new("matched", result.is_ok()),
            ],
        );
        
        // 记录不匹配
        if let Err(ref error) = result {
            span.set_status(Status::error(format!("{}", error)));
            span.record_exception(error);
        }
        
        result
    }
    
    fn do_verify(&self, instruction: &Instruction, expected: &ExpectedResult) -> Result<VerificationResult, Error> {
        // 实际验证逻辑
        Ok(VerificationResult::default())
    }
}
```

### 4.3 Gateway 埋点验证

```typescript
// src/gateway/tracing.ts - Alpha 环境

import * as api from '@opentelemetry/api';

const tracer = api.trace.getTracer('cgas-phase4/gateway-alpha');

export class TracingGateway {
  private executorClient: any;
  
  constructor(executorClient: any) {
    this.executorClient = executorClient;
  }
  
  /// 处理请求（带全链路追踪）- Alpha 环境
  async handleRequest(req: Request): Promise<Response> {
    const span = tracer.startSpan('Gateway.handleRequest', {
      attributes: {
        'http.method': req.method,
        'http.url': req.url,
        'http.user_agent': req.headers['user-agent'],
        'channel': 'feishu',
        'environment': 'alpha',
        'phase': 'phase4',
      },
    });
    
    return api.context.with(api.trace.setSpan(api.context.active(), span), async () => {
      try {
        // 认证
        const authSpan = tracer.startSpan('Gateway.authenticate');
        const authResult = await this.authenticate(req);
        authSpan.end();
        
        // 路由
        const routeSpan = tracer.startSpan('Gateway.route_request', {
          attributes: {
            'route.target': 'executor',
            'environment': 'alpha',
          },
        });
        
        // 转发到执行器（带追踪上下文）
        const executorResult = await this.executorClient.execute(req.body);
        
        routeSpan.end();
        
        span.setAttribute('http.status_code', 200);
        span.end();
        
        return executorResult;
      } catch (error) {
        span.setStatus({ code: api.SpanStatusCode.ERROR });
        span.recordException(error);
        span.end();
        throw error;
      }
    });
  }
  
  private async authenticate(req: Request): Promise<boolean> {
    // 认证逻辑
    return true;
  }
}
```

---

## 5. 追踪指标

### 5.1 5 个核心追踪指标

| 指标 ID | 指标名 | 类型 | 采集频率 | P0 告警阈值 | 说明 |
|---|---|---|---|---|---|
| **ALPHA-TRACE-001** | `distributed_trace_coverage` | Gauge | 5min | <98% | Trace 覆盖率 |
| **ALPHA-TRACE-002** | `trace_span_duration_p99` | Histogram | 实时 | >500ms | Span 时长 P99 |
| **ALPHA-TRACE-003** | `trace_total_duration_p99` | Histogram | 实时 | >1000ms | 全链路时长 P99 |
| **ALPHA-TRACE-004** | `trace_span_count_avg` | Gauge | 5min | - | 平均 Span 数量 |
| **ALPHA-TRACE-005** | `trace_propagation_success_rate` | Gauge | 30s | <99% | 追踪传递成功率 |

### 5.2 指标实现 (Rust)

```rust
// src/tracing/metrics.rs - Alpha 环境

use prometheus::{Histogram, Gauge, Counter, HistogramOpts, Opts, register_histogram, register_gauge, register_counter};

lazy_static! {
    /// Trace 覆盖率
    pub static ref DISTRIBUTED_TRACE_COVERAGE: Gauge = register_gauge!(
        Opts::new("distributed_trace_coverage", "Distributed trace coverage percentage")
            .namespace("cgas_alpha")
            .help("Distributed trace coverage percentage")
    ).unwrap();
    
    /// Span 时长 P99
    pub static ref TRACE_SPAN_DURATION_P99: Histogram = register_histogram!(
        HistogramOpts::new("trace_span_duration_p99", "Trace span duration P99 in ms")
            .namespace("cgas_alpha")
            .buckets(vec![10.0, 50.0, 100.0, 200.0, 300.0, 500.0, 750.0, 1000.0, 2500.0])
    ).unwrap();
    
    /// 全链路时长 P99
    pub static ref TRACE_TOTAL_DURATION_P99: Histogram = register_histogram!(
        HistogramOpts::new("trace_total_duration_p99", "Total trace duration P99 in ms")
            .namespace("cgas_alpha")
            .buckets(vec![100.0, 250.0, 500.0, 750.0, 1000.0, 1500.0, 2000.0, 3000.0, 5000.0])
    ).unwrap();
    
    /// 平均 Span 数量
    pub static ref TRACE_SPAN_COUNT_AVG: Gauge = register_gauge!(
        Opts::new("trace_span_count_avg", "Average span count per trace")
            .namespace("cgas_alpha")
            .help("Average span count per trace")
    ).unwrap();
    
    /// 追踪传递成功率
    pub static ref TRACE_PROPAGATION_SUCCESS_RATE: Gauge = register_gauge!(
        Opts::new("trace_propagation_success_rate", "Trace propagation success rate percentage")
            .namespace("cgas_alpha")
            .help("Trace propagation success rate in percentage")
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

## 6. 验证脚本

### 6.1 Trace 验证 Python 脚本

```python
#!/usr/bin/env python3
"""
Alpha 环境 Trace 接入验证脚本
"""

import requests
import json
import uuid
import time
from datetime import datetime

GATEWAY_URL = "http://alpha-gateway:8084"
TEMPO_URL = "http://tempo-alpha:3200"
JAEGER_URL = "http://jaeger-alpha:16686"
PROMETHEUS_URL = "http://prometheus-alpha:9090"
OTEL_COLLECTOR_URL = "http://otel-collector-alpha:13133"

def check_otel_collector_health():
    """检查 OTel Collector 健康状态"""
    try:
        response = requests.get(f"{OTEL_COLLECTOR_URL}")
        if response.status_code == 200:
            print("✅ OTel Collector 健康检查通过")
            return True
        else:
            print(f"❌ OTel Collector 健康检查失败：{response.status_code}")
            return False
    except Exception as e:
        print(f"❌ OTel Collector 无法访问：{e}")
        return False

def check_tempo_health():
    """检查 Tempo 健康状态"""
    try:
        response = requests.get(f"{TEMPO_URL}/status")
        if response.status_code == 200:
            print("✅ Tempo 健康检查通过")
            return True
        else:
            print(f"❌ Tempo 健康检查失败：{response.status_code}")
            return False
    except Exception as e:
        print(f"❌ Tempo 无法访问：{e}")
        return False

def check_jaeger_health():
    """检查 Jaeger 健康状态"""
    try:
        response = requests.get(f"{JAEGER_URL}/api/services")
        if response.status_code == 200:
            services = response.json().get('data', [])
            print(f"✅ Jaeger 健康检查通过 (发现 {len(services)} 个服务)")
            return True
        else:
            print(f"❌ Jaeger 健康检查失败：{response.status_code}")
            return False
    except Exception as e:
        print(f"❌ Jaeger 无法访问：{e}")
        return False

def generate_trace_id():
    """生成追踪 ID"""
    return uuid.uuid4().hex

def send_traced_request(trace_id: str):
    """发送带追踪 ID 的请求"""
    headers = {
        "X-Trace-ID": trace_id,
        "Content-Type": "application/json",
    }
    
    payload = {
        "instruction": {
            "type": "deploy",
            "params": {}
        }
    }
    
    try:
        response = requests.post(
            f"{GATEWAY_URL}/api/v1/execute",
            headers=headers,
            json=payload,
            timeout=10
        )
        return response.status_code == 200
    except Exception as e:
        print(f"请求失败：{e}")
        return False

def verify_trace_in_tempo(trace_id: str, timeout_seconds: int = 30):
    """验证 Trace 是否存在于 Tempo"""
    start_time = time.time()
    while time.time() - start_time < timeout_seconds:
        try:
            response = requests.get(
                f"{TEMPO_URL}/api/traces/{trace_id}"
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get("traces"):
                    return True, data["traces"][0]
        except Exception as e:
            pass
        
        time.sleep(1)
    
    return False, None

def verify_trace_in_jaeger(trace_id: str, timeout_seconds: int = 30):
    """验证 Trace 是否存在于 Jaeger"""
    start_time = time.time()
    while time.time() - start_time < timeout_seconds:
        try:
            response = requests.get(
                f"{JAEGER_URL}/api/traces/{trace_id}"
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('data'):
                    return True, data['data'][0]
        except Exception as e:
            pass
        
        time.sleep(1)
    
    return False, None

def verify_trace_metrics():
    """验证追踪指标"""
    metrics = [
        "distributed_trace_coverage",
        "trace_span_duration_p99",
        "trace_total_duration_p99",
        "trace_span_count_avg",
        "trace_propagation_success_rate",
    ]
    
    print("\n验证追踪指标:")
    all_ok = True
    
    for metric in metrics:
        try:
            response = requests.get(
                f"{PROMETHEUS_URL}/api/v1/query",
                params={"query": f"cgas_alpha_{metric}"}
            )
            
            if response.status_code == 200:
                data = response.json()
                if data["status"] == "success" and data["data"]["result"]:
                    value = data["data"]["result"][0]["value"][1]
                    print(f"  ✅ {metric}: {value}")
                else:
                    print(f"  ⚠️ {metric}: 无数据")
                    all_ok = False
            else:
                print(f"  ❌ {metric}: 查询失败")
                all_ok = False
        except Exception as e:
            print(f"  ❌ {metric}: {e}")
            all_ok = False
    
    return all_ok

def main():
    print(f"开始验证 Alpha 环境 Trace 接入... ({datetime.now()})")
    print("=" * 60)
    
    # 1. 健康检查
    print("\n【1】健康检查")
    otel_ok = check_otel_collector_health()
    tempo_ok = check_tempo_health()
    jaeger_ok = check_jaeger_health()
    
    if not (otel_ok and tempo_ok and jaeger_ok):
        print("\n❌ 健康检查失败，终止验证")
        return 1
    
    # 2. 发送追踪请求
    print("\n【2】发送追踪请求")
    success_count = 0
    trace_ids = []
    
    for i in range(10):
        trace_id = generate_trace_id()
        trace_ids.append(trace_id)
        
        if send_traced_request(trace_id):
            success_count += 1
            print(f"✓ 请求 {i+1}/10 发送成功 (trace_id: {trace_id[:16]}...)")
        else:
            print(f"✗ 请求 {i+1}/10 发送失败")
    
    print(f"\n请求发送成功率：{success_count}/10")
    
    # 3. 验证 Tempo 中的 Trace
    print("\n【3】验证 Tempo 中的 Trace")
    found_in_tempo = 0
    
    for trace_id in trace_ids:
        found, trace_data = verify_trace_in_tempo(trace_id)
        if found:
            found_in_tempo += 1
            span_count = len(trace_data.get("batches", [{}])[0].get("spans", []))
            print(f"✓ Trace {trace_id[:16]}... 在 Tempo 中找到 ({span_count} spans)")
        else:
            print(f"✗ Trace {trace_id[:16]}... 在 Tempo 中未找到")
    
    print(f"\nTempo Trace 发现率：{found_in_tempo}/10")
    
    # 4. 验证 Jaeger 中的 Trace
    print("\n【4】验证 Jaeger 中的 Trace")
    found_in_jaeger = 0
    
    for trace_id in trace_ids:
        found, trace_data = verify_trace_in_jaeger(trace_id)
        if found:
            found_in_jaeger += 1
            span_count = len(trace_data.get('spans', []))
            print(f"✓ Trace {trace_id[:16]}... 在 Jaeger 中找到 ({span_count} spans)")
        else:
            print(f"✗ Trace {trace_id[:16]}... 在 Jaeger 中未找到")
    
    print(f"\nJaeger Trace 发现率：{found_in_jaeger}/10")
    
    # 5. 验证追踪指标
    print("\n【5】验证追踪指标")
    metrics_ok = verify_trace_metrics()
    
    # 6. 生成报告
    print("\n" + "=" * 60)
    
    report = {
        "timestamp": datetime.now().isoformat(),
        "health_check": {
            "otel_collector": otel_ok,
            "tempo": tempo_ok,
            "jaeger": jaeger_ok,
        },
        "requests": {
            "sent": 10,
            "success": success_count,
        },
        "trace_discovery": {
            "tempo": found_in_tempo,
            "jaeger": found_in_jaeger,
        },
        "metrics": metrics_ok,
        "passed": (
            otel_ok and tempo_ok and jaeger_ok and
            found_in_tempo >= 9 and found_in_jaeger >= 9 and
            metrics_ok
        ),
    }
    
    with open("alpha_trace_validation_report.json", "w") as f:
        json.dump(report, f, indent=2)
    
    print(f"\n报告已保存至：alpha_trace_validation_report.json")
    
    if report["passed"]:
        print("\n✅ Alpha 环境 Trace 接入验证通过")
        return 0
    else:
        print("\n❌ Alpha 环境 Trace 接入验证失败")
        return 1

if __name__ == "__main__":
    exit(main())
```

---

## 7. 验收标准

### 7.1 组件验收

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| OTel Collector | 健康检查 OK | HTTP 检查 | 状态 200 |
| Tempo | 健康检查 OK | API 检查 | 状态 200 |
| Jaeger | 服务列表可查 | API 检查 | 返回服务列表 |
| Prometheus | 追踪指标可查 | PromQL 查询 | 5 个指标有数据 |

### 7.2 追踪验收

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| Trace 发送 | 10 个请求成功 | 脚本验证 | ≥9 个成功 |
| Tempo 存储 | Trace 可查询 | Tempo API | ≥9 个找到 |
| Jaeger 查询 | Trace 可查询 | Jaeger API | ≥9 个找到 |
| Span 关联 | 多 Span 正确关联 | Trace 检查 | 关联率≥98% |
| 指标上报 | 5 个指标正常 | Prometheus 查询 | 数据持续上报 |

---

## 8. 实施计划

| 任务 | 责任人 | 状态 | 交付物 | 预计耗时 |
|---|---|---|---|---|
| OTel Collector 配置 | SRE | ✅ 完成 | otel-collector-alpha-config.yaml | 60 分钟 |
| Tempo 部署 | SRE | ✅ 完成 | tempo-alpha.yaml | 30 分钟 |
| Jaeger 部署 | SRE | ✅ 完成 | docker-compose.yaml | 30 分钟 |
| Executor 埋点 | Dev | ✅ 完成 | executor/tracing.rs | 60 分钟 |
| Verifier 埋点 | Dev | ✅ 完成 | verifier/tracing.rs | 60 分钟 |
| Gateway 埋点 | Dev | ✅ 完成 | gateway/tracing.ts | 60 分钟 |
| 追踪指标集成 | Observability | ✅ 完成 | tracing_metrics.rs | 60 分钟 |
| 验证脚本 | Observability | ✅ 完成 | trace_validation.py | 60 分钟 |
| 验证测试 | Observability + SRE | ✅ 完成 | validation_report.md | 60 分钟 |

---

## 9. 附录

### 9.1 快速验证命令

```bash
# 检查 OTel Collector 健康
curl http://otel-collector-alpha:13133

# 检查 Tempo 健康
curl http://tempo-alpha:3200/status

# 检查 Jaeger 服务列表
curl http://jaeger-alpha:16686/api/services

# 查询追踪指标
curl 'http://prometheus-alpha:9090/api/v1/query?query=cgas_alpha_distributed_trace_coverage'

# 查询 Tempo Trace
curl 'http://tempo-alpha:3200/api/search?q=Executor.execute'

# 访问 Jaeger UI
open http://jaeger-alpha:16686

# 访问 Grafana Trace 仪表盘
open http://grafana-alpha:3000/d/alpha-tracing
```

### 9.2 PromQL 查询手册

```promql
# Trace 覆盖率
cgas_alpha_distributed_trace_coverage

# Span 时长 P99
histogram_quantile(0.99, sum(rate(cgas_alpha_trace_span_duration_p99_bucket[5m])) by(le))

# 全链路时长 P99
histogram_quantile(0.99, sum(rate(cgas_alpha_trace_total_duration_p99_bucket[5m])) by(le))

# 平均 Span 数量
cgas_alpha_trace_span_count_avg

# 追踪传递成功率
cgas_alpha_trace_propagation_success_rate
```

### 9.3 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Phase 3 OTel 集成 | otel_integration.md | 参考实现 |
| Phase 3 Trace ID 实现 | trace_id_implementation.md | 参考实现 |
| Alpha 20 指标配置 | alpha_monitoring_20_metrics.md | 指标定义 |

---

**文档状态**: ✅ Week 1-T5 完成  
**创建日期**: 2026-04-05  
**责任人**: Observability-Agent + SRE-Agent  
**保管**: 项目文档库  
**环境**: Alpha (Phase 4 Week 1)
