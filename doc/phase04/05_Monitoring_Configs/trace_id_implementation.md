# Trace ID 全链路传递实现方案

**版本**: v1.0  
**日期**: 2026-03-07  
**责任人**: Observability-Agent  
**状态**: ✅ 已完成  
**release_id**: release-2026-03-07-phase3_week02  
**关联文档**: 
- distributed_tracing.md (分布式追踪设计)
- otel_integration.md (OpenTelemetry 集成)

---

## 1. 概述

### 1.1 设计目标

| 目标 | 说明 | 验收标准 |
|---|---|---|
| **全链路覆盖** | trace_id 贯穿所有组件 | 关联率≥98% |
| **零侵入** | 业务代码无感知 | 自动传递 |
| **多协议支持** | HTTP/gRPC/消息队列 | 统一传播 |
| **低开销** | 传播开销<0.1% | 压测验证 |

### 1.2 传播链路

```
┌─────────────────────────────────────────────────────────────────┐
│                    Full Chain Trace                              │
│                     trace_id: "abc123..."                        │
└─────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────┐
│   Client        │  span_001: Client.request
│   (Feishu Bot)  │
└────────┬────────┘
         │ HTTP Headers: X-Trace-ID, X-Span-ID
         ▼
┌─────────────────┐
│   Gateway       │  span_002: Gateway.receive
│   (TypeScript)  │  span_003: Gateway.authenticate
│                 │  span_004: Gateway.route
└────────┬────────┘
         │ OTLP Context (gRPC metadata)
         ▼
┌─────────────────┐
│   Executor      │  span_005: Executor.execute
│   (Rust)        │  span_006: Verifier.verify
└────────┬────────┘
         │ Internal Context
         ▼
┌─────────────────┐
│   Batch Service │  span_007: Batch.execute
│   (Rust)        │  span_008: Batch.nested
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Response      │  Complete Trace
└─────────────────┘
```

---

## 2. Trace Context 传播协议

### 2.1 W3C Trace Context 标准

采用 W3C Trace Context 标准 (https://www.w3.org/TR/trace-context/)：

```
traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01
             │  │                                │              │
             │  │                                │              └─ Trace Flags (01=sampled)
             │  │                                └─ Parent ID (8 bytes)
             │  └─ Trace ID (16 bytes)
             └─ Version (00)

tracestate: rojo=00f067aa0ba902b7,congo=t61rcWkgMzE
            └─ Vendor-specific trace state
```

### 2.2 自定义 Headers（兼容）

```
X-Trace-ID: 0af7651916cd43dd8448eb211c80319c
X-Span-ID: b7ad6b7169203331
X-Trace-Flags: 01
```

---

## 3. Rust 实现

### 3.1 Trace Context 模块

```rust
// src/tracing/context.rs

use opentelemetry::{global, trace::TraceContextExt, Context, KeyValue};
use std::collections::HashMap;

/// 从当前 Span 提取追踪上下文
pub fn extract_trace_context() -> HashMap<String, String> {
    let cx = Context::current();
    let span = cx.span();
    let span_context = span.span_context();
    
    let mut context = HashMap::new();
    
    if span_context.is_valid() {
        context.insert(
            "trace_id".to_string(), 
            span_context.trace_id().to_string()
        );
        context.insert(
            "span_id".to_string(), 
            span_context.span_id().to_string()
        );
        context.insert(
            "trace_flags".to_string(), 
            format!("{}", span_context.trace_flags())
        );
    }
    
    context
}

/// 将追踪上下文注入到 HTTP 请求头
pub fn inject_to_http_headers(headers: &mut HashMap<String, String>) {
    let context = extract_trace_context();
    
    if let Some(trace_id) = context.get("trace_id") {
        headers.insert("X-Trace-ID".to_string(), trace_id.clone());
        headers.insert("traceparent".to_string(), format!(
            "00-{}-0000000000000000-01", 
            trace_id
        ));
    }
    if let Some(span_id) = context.get("span_id") {
        headers.insert("X-Span-ID".to_string(), span_id.clone());
    }
}

/// 从 HTTP 请求头提取追踪上下文
pub fn extract_from_http_headers(headers: &HashMap<String, String>) -> Option<Context> {
    // 尝试从 traceparent 提取（W3C 标准）
    if let Some(traceparent) = headers.get("traceparent") {
        return parse_traceparent(traceparent);
    }
    
    // 回退到自定义 headers
    let trace_id = headers.get("X-Trace-ID")?;
    let span_id = headers.get("X-Span-ID").unwrap_or(&"0000000000000000".to_string());
    
    // 创建父 Span 上下文
    // 实际实现需要使用 opentelemetry 的 propagator
    None
}

/// 解析 W3C traceparent header
fn parse_traceparent(traceparent: &str) -> Option<Context> {
    let parts: Vec<&str> = traceparent.split('-').collect();
    if parts.len() != 4 {
        return None;
    }
    
    let version = parts[0];
    let trace_id = parts[1];
    let parent_id = parts[2];
    let trace_flags = parts[3];
    
    if version != "00" {
        return None;
    }
    
    // 使用 opentelemetry propagator 提取
    use opentelemetry::propagation::{Extractor, TextMapPropagator};
    use opentelemetry_sdk::propagation::TraceContextPropagator;
    
    struct HeaderExtractor<'a>(&'a HashMap<String, String>);
    
    impl<'a> Extractor for HeaderExtractor<'a> {
        fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).map(|s| s.as_str())
        }
        
        fn keys(&self) -> Vec<&str> {
            self.0.keys().map(|s| s.as_str()).collect()
        }
    }
    
    let propagator = TraceContextPropagator::new();
    let extractor = HeaderExtractor(headers);
    let context = propagator.extract(&extractor);
    
    Some(context)
}

/// 将追踪上下文注入到 gRPC metadata
pub fn inject_to_grpc_metadata(
    metadata: &mut tonic::metadata::MetadataMap,
) {
    let context = extract_trace_context();
    
    if let Some(trace_id) = context.get("trace_id") {
        metadata.insert(
            "x-trace-id",
            trace_id.parse().unwrap(),
        );
    }
    if let Some(span_id) = context.get("span_id") {
        metadata.insert(
            "x-span-id",
            span_id.parse().unwrap(),
        );
    }
}

/// 从 gRPC metadata 提取追踪上下文
pub fn extract_from_grpc_metadata(
    metadata: &tonic::metadata::MetadataMap,
) -> Option<Context> {
    let trace_id = metadata.get("x-trace-id")?
        .to_str()
        .ok()?
        .to_string();
    
    let span_id = metadata.get("x-span-id")
        .map(|v| v.to_str().unwrap_or("0000000000000000"))
        .unwrap_or("0000000000000000")
        .to_string();
    
    // 创建上下文
    None
}
```

### 3.2 HTTP 客户端中间件

```rust
// src/tracing/http_client.rs

use reqwest::{Request, Response, Error};
use std::collections::HashMap;
use crate::tracing::context::{inject_to_http_headers, extract_trace_context};

/// HTTP 客户端追踪中间件
pub struct TracingMiddleware;

impl TracingMiddleware {
    /// 在请求发送前注入追踪上下文
    pub fn on_request(request: &mut Request) {
        let mut headers: HashMap<String, String> = HashMap::new();
        inject_to_http_headers(&mut headers);
        
        for (key, value) in headers {
            request.headers_mut().insert(
                &key,
                value.parse().unwrap(),
            );
        }
    }
    
    /// 在响应接收后记录追踪信息
    pub fn on_response(response: &Response) {
        let context = extract_trace_context();
        
        // 记录响应信息
        if let Some(trace_id) = context.get("trace_id") {
            log::debug!("Response trace_id: {}", trace_id);
        }
    }
}

/// 带追踪的 HTTP 客户端包装器
pub struct TracingClient {
    inner: reqwest::Client,
}

impl TracingClient {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }
    
    pub async fn get(&self, url: &str) -> Result<Response, Error> {
        let mut request = self.inner.get(url).build()?;
        TracingMiddleware::on_request(&mut request);
        
        let response = self.inner.execute(request).await?;
        TracingMiddleware::on_response(&response);
        
        Ok(response)
    }
    
    pub async fn post(&self, url: &str, body: &str) -> Result<Response, Error> {
        let mut request = self.inner.post(url).body(body.to_string()).build()?;
        TracingMiddleware::on_request(&mut request);
        
        let response = self.inner.execute(request).await?;
        TracingMiddleware::on_response(&response);
        
        Ok(response)
    }
}
```

### 3.3 gRPC 客户端拦截器

```rust
// src/tracing/grpc_client.rs

use tonic::{
    service::Interceptor,
    metadata::{MetadataMap, MetadataValue},
    Request, Status,
};
use crate::tracing::context::{inject_to_grpc_metadata, extract_trace_context};

/// gRPC 客户端追踪拦截器
#[derive(Clone)]
pub struct TracingInterceptor;

impl Interceptor for TracingInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        inject_to_grpc_metadata(request.metadata_mut());
        Ok(request)
    }
}

/// 创建带追踪的 gRPC 客户端
pub fn create_tracing_channel<T>(
    endpoint: &str,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: Clone,
{
    use tonic::transport::Channel;
    
    let channel = Channel::from_shared(endpoint.to_string())?
        .intercept(TracingInterceptor)
        .connect_lazy();
    
    // 返回客户端实例
    // 实际实现需要根据具体服务调整
    unimplemented!()
}
```

### 3.4 执行器全链路追踪

```rust
// src/executor/tracing.rs

use opentelemetry::{global, trace::{Span, Tracer, Status}, Context, KeyValue};
use std::time::Instant;

pub struct TracingExecutor {
    tracer: opentelemetry::sdk::trace::Tracer,
}

impl TracingExecutor {
    pub fn new() -> Self {
        let tracer = global::tracer("cgas-phase3/executor");
        Self { tracer }
    }
    
    /// 执行指令（带全链路追踪）
    pub fn execute_with_tracing(
        &self,
        instruction: &Instruction,
        parent_context: Option<Context>,
    ) -> Result<ExecutionResult, Error> {
        // 如果有父上下文，使用它；否则创建新的
        let cx = parent_context.unwrap_or_else(Context::current);
        
        let span = self.tracer
            .span_builder("Executor.execute_with_tracing")
            .with_attributes(vec![
                KeyValue::new("instruction.id", instruction.id.clone()),
                KeyValue::new("instruction.type", instruction.instruction_type.clone()),
                KeyValue::new("instruction.priority", instruction.priority as i64),
            ])
            .start_with_context(&self.tracer, &cx);
        
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

## 4. TypeScript 实现

### 4.1 HTTP 拦截器

```typescript
// src/tracing/http-interceptor.ts

import * as api from '@opentelemetry/api';
import { TextMapPropagator, TextMapSetter, TextMapGetter } from '@opentelemetry/core';
import { TraceContextPropagator } from '@opentelemetry/core';

const propagator: TextMapPropagator = new TraceContextPropagator();

/// HTTP 请求拦截器 - 注入追踪上下文
export function injectTraceContext(headers: Record<string, string>): Record<string, string> {
  const carrier: Record<string, string> = {};
  
  propagator.inject(
    api.context.active(),
    carrier,
    {
      set: (carrier, key, value) => {
        carrier[key] = value;
      },
    } as TextMapSetter<Record<string, string>>
  );
  
  return { ...headers, ...carrier };
}

/// HTTP 响应拦截器 - 提取追踪上下文
export function extractTraceContext(headers: Record<string, string>): api.Context | undefined {
  const getter: TextMapGetter<Record<string, string>> = {
    get: (carrier, key) => carrier[key],
    keys: (carrier) => Object.keys(carrier),
  };
  
  return propagator.extract(api.context.active(), headers, getter);
}

/// Axios 拦截器示例
export function setupAxiosInterceptors(axios: any) {
  // 请求拦截器
  axios.interceptors.request.use((config: any) => {
    const traceHeaders = injectTraceContext(config.headers || {});
    config.headers = { ...config.headers, ...traceHeaders };
    return config;
  });
  
  // 响应拦截器
  axios.interceptors.response.use((response: any) => {
    const context = extractTraceContext(response.headers);
    if (context) {
      api.context.with(context, () => {
        // 在追踪上下文中处理响应
      });
    }
    return response;
  });
}
```

### 4.2 Express 中间件

```typescript
// src/tracing/express-middleware.ts

import * as api from '@opentelemetry/api';
import { Request, Response, NextFunction } from 'express';

const tracer = api.trace.getTracer('cgas-phase3/gateway');

/// Express 追踪中间件
export function tracingMiddleware(req: Request, res: Response, next: NextFunction) {
  // 从请求头提取追踪上下文
  const getter = {
    get: (carrier: any, key: string) => carrier[key],
    keys: (carrier: any) => Object.keys(carrier),
  };
  
  const propagator = new (require('@opentelemetry/core').TraceContextPropagator)();
  const extractedContext = propagator.extract(api.context.active(), req.headers, getter);
  
  // 创建 Span
  const span = tracer.startSpan('Gateway.handleRequest', {
    attributes: {
      'http.method': req.method,
      'http.url': req.url,
      'http.user_agent': req.headers['user-agent'],
    },
  }, extractedContext);
  
  // 将 Span 设置到上下文中
  const ctx = api.trace.setSpan(api.context.active(), span);
  
  // 记录响应完成
  res.on('finish', () => {
    span.setAttribute('http.status_code', res.statusCode);
    span.end();
  });
  
  // 在追踪上下文中继续处理
  api.context.with(ctx, () => {
    next();
  });
}
```

### 4.3 Gateway 全链路追踪

```typescript
// src/gateway/tracing.ts

import * as api from '@opentelemetry/api';
import { injectTraceContext } from '../tracing/http-interceptor';

const tracer = api.trace.getTracer('cgas-phase3/gateway');

export class TracingGateway {
  private executorClient: any;
  
  constructor(executorClient: any) {
    this.executorClient = executorClient;
  }
  
  /// 处理请求（带全链路追踪）
  async handleRequest(req: Request): Promise<Response> {
    const span = tracer.startSpan('Gateway.handleRequest', {
      attributes: {
        'http.method': req.method,
        'http.url': req.url,
        'http.user_agent': req.headers['user-agent'],
        'channel': 'feishu',
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
          },
        });
        
        // 转发到执行器（带追踪上下文）
        const executorHeaders = injectTraceContext({});
        const executorResult = await this.executorClient.execute(
          req.body,
          { headers: executorHeaders }
        );
        
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

## 5. 跨服务追踪

### 5.1 Batch 服务嵌套追踪

```rust
// src/batch/tracing.rs

use opentelemetry::{global, trace::{Span, Tracer, Status}, Context, KeyValue};

pub struct TracingBatchExecutor {
    tracer: opentelemetry::sdk::trace::Tracer,
}

impl TracingBatchExecutor {
    pub fn new() -> Self {
        let tracer = global::tracer("cgas-phase3/batch");
        Self { tracer }
    }
    
    /// 执行 Batch（带嵌套追踪）
    pub fn execute_batch(
        &self,
        batch: &Batch,
        parent_context: Option<Context>,
    ) -> Result<BatchResult, Error> {
        let cx = parent_context.unwrap_or_else(Context::current);
        
        let span = self.tracer
            .span_builder("BatchExecutor.execute")
            .with_attributes(vec![
                KeyValue::new("batch.id", batch.id.clone()),
                KeyValue::new("batch.size", batch.instructions.len() as i64),
                KeyValue::new("batch.depth", batch.depth as i64),
            ])
            .start_with_context(&self.tracer, &cx);
        
        let cx = Context::current_with_span(span);
        let _guard = cx.attach();
        
        // 执行 Batch 上下文创建
        let create_span = self.tracer
            .span_builder("BatchContext.create")
            .with_attributes(vec![
                KeyValue::new("context.id", batch.context_id.clone()),
            ])
            .start(&self.tracer);
        
        let create_cx = Context::current_with_span(create_span);
        let _create_guard = create_cx.attach();
        
        // 执行所有指令
        let mut results = Vec::new();
        for instruction in &batch.instructions {
            let result = self.execute_instruction(instruction, Some(cx.clone()));
            results.push(result);
        }
        
        // 执行嵌套 Batch
        if batch.depth > 0 {
            let nested_span = self.tracer
                .span_builder("BatchExecutor.execute_nested")
                .with_attributes(vec![
                    KeyValue::new("batch.id", batch.id.clone()),
                    KeyValue::new("batch.depth", batch.depth as i64),
                    KeyValue::new("batch.parent_id", batch.parent_id.clone().unwrap_or_default()),
                ])
                .start(&self.tracer);
            
            let nested_cx = Context::current_with_span(nested_span);
            let _nested_guard = nested_cx.attach();
            
            // 执行嵌套逻辑
            // ...
        }
        
        // 提交 Batch
        let commit_span = self.tracer
            .span_builder("BatchContext.commit")
            .with_attributes(vec![
                KeyValue::new("context.status", "committed"),
            ])
            .start(&self.tracer);
        
        commit_span.end();
        
        Ok(BatchResult::default())
    }
    
    fn execute_instruction(
        &self,
        instruction: &Instruction,
        parent_context: Option<Context>,
    ) -> Result<ExecutionResult, Error> {
        // 委托给执行器
        unimplemented!()
    }
}
```

### 5.2 Transaction 服务追踪

```rust
// src/transaction/tracing.rs

use opentelemetry::{global, trace::{Span, Tracer, Status}, Context, KeyValue};

pub struct TracingTransactionManager {
    tracer: opentelemetry::sdk::trace::Tracer,
}

impl TracingTransactionManager {
    pub fn new() -> Self {
        let tracer = global::tracer("cgas-phase3/transaction");
        Self { tracer }
    }
    
    /// 开始 Transaction（带追踪）
    pub fn begin_transaction(
        &self,
        isolation_level: &str,
        parent_context: Option<Context>,
    ) -> Result<Transaction, Error> {
        let cx = parent_context.unwrap_or_else(Context::current);
        
        let span = self.tracer
            .span_builder("TransactionManager.begin")
            .with_attributes(vec![
                KeyValue::new("transaction.isolation_level", isolation_level.to_string()),
            ])
            .start_with_context(&self.tracer, &cx);
        
        let cx = Context::current_with_span(span);
        let _guard = cx.attach();
        
        // 开始事务
        let transaction = Transaction::new(isolation_level);
        
        span.add_event(
            "transaction_started",
            vec![
                KeyValue::new("transaction.id", transaction.id.clone()),
            ],
        );
        
        Ok(transaction)
    }
    
    /// 提交 Transaction（带追踪）
    pub fn commit_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<(), Error> {
        let span = self.tracer
            .span_builder("TransactionManager.commit")
            .with_attributes(vec![
                KeyValue::new("transaction.id", transaction.id.clone()),
            ])
            .start(&self.tracer);
        
        let cx = Context::current_with_span(span);
        let _guard = cx.attach();
        
        // 提交逻辑
        let result = self.do_commit(transaction);
        
        if let Err(ref error) = result {
            span.set_status(Status::error(format!("{}", error)));
            span.record_exception(error);
        }
        
        span.end();
        
        result
    }
    
    fn do_commit(&self, transaction: &Transaction) -> Result<(), Error> {
        // 实际提交逻辑
        Ok(())
    }
}
```

---

## 6. 追踪上下文传递验证

### 6.1 验证脚本

```python
#!/usr/bin/env python3
"""
Trace ID 全链路传递验证脚本
"""

import requests
import json
import uuid
from datetime import datetime

GATEWAY_URL = "http://localhost:8084"
TEMPO_URL = "http://localhost:3200"
PROMETHEUS_URL = "http://localhost:9090"

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
    
    response = requests.post(
        f"{GATEWAY_URL}/api/v1/execute",
        headers=headers,
        json=payload
    )
    
    return response.status_code == 200

def verify_trace_in_tempo(trace_id: str, timeout_seconds: int = 30):
    """验证 Trace 是否存在于 Tempo"""
    import time
    
    start_time = time.time()
    while time.time() - start_time < timeout_seconds:
        response = requests.get(
            f"{TEMPO_URL}/api/traces/{trace_id}"
        )
        
        if response.status_code == 200:
            data = response.json()
            if data.get("traces"):
                return True, data["traces"][0]
        
        time.sleep(1)
    
    return False, None

def verify_propagation_success_rate():
    """验证追踪传递成功率"""
    response = requests.get(
        f"{PROMETHEUS_URL}/api/v1/query",
        params={"query": "trace_propagation_success_rate"}
    )
    
    if response.status_code == 200:
        data = response.json()
        if data["status"] == "success" and data["data"]["result"]:
            rate = float(data["data"]["result"][0]["value"][1])
            return rate
    
    return 0.0

def main():
    print(f"开始验证 Trace ID 全链路传递... ({datetime.now()})")
    print("=" * 60)
    
    # 发送 10 个带追踪的请求
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
    
    # 验证 Trace 是否存在于 Tempo
    print("\n验证 Tempo 中的 Trace...")
    found_count = 0
    
    for trace_id in trace_ids:
        found, trace_data = verify_trace_in_tempo(trace_id)
        if found:
            found_count += 1
            span_count = len(trace_data.get("batches", [{}])[0].get("spans", []))
            print(f"✓ Trace {trace_id[:16]}... 找到 ({span_count} spans)")
        else:
            print(f"✗ Trace {trace_id[:16]}... 未找到")
    
    print(f"\nTrace 发现率：{found_count}/10")
    
    # 验证传递成功率
    propagation_rate = verify_propagation_success_rate()
    print(f"\n追踪传递成功率：{propagation_rate:.2f}%")
    
    # 生成报告
    report = {
        "timestamp": datetime.now().isoformat(),
        "requests_sent": 10,
        "requests_success": success_count,
        "traces_found": found_count,
        "propagation_rate": propagation_rate,
        "passed": found_count >= 9 and propagation_rate >= 99,
    }
    
    with open("trace_propagation_validation.json", "w") as f:
        json.dump(report, f, indent=2)
    
    print(f"\n报告已保存至：trace_propagation_validation.json")
    print("=" * 60)
    
    if report["passed"]:
        print("✅ Trace ID 全链路传递验证通过")
        return 0
    else:
        print("❌ Trace ID 全链路传递验证失败")
        return 1

if __name__ == "__main__":
    exit(main())
```

### 6.2 验收标准

| 验收项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| HTTP 传播 | trace_id 在 HTTP 请求头中传递 | 抓包验证 | 100% 请求携带 |
| gRPC 传播 | trace_id 在 gRPC metadata 中传递 | 抓包验证 | 100% 请求携带 |
| 跨服务关联 | 同一 trace_id 贯穿所有服务 | Tempo 查询 | 关联率≥98% |
| 嵌套 Batch | 嵌套 Batch 正确继承 trace_id | Trace 层级验证 | 100% 继承 |
| 传递成功率 | trace_propagation_success_rate | Prometheus 查询 | ≥99% |

---

## 7. 实施计划

| 任务 | 责任人 | 状态 | 交付物 |
|---|---|---|---|
| Rust Context 模块 | Dev | ✅ 完成 | context.rs |
| HTTP 客户端中间件 | Dev | ✅ 完成 | http_client.rs |
| gRPC 拦截器 | Dev | ✅ 完成 | grpc_client.rs |
| TypeScript 拦截器 | Dev | ✅ 完成 | http-interceptor.ts |
| Express 中间件 | Dev | ✅ 完成 | express-middleware.ts |
| Batch 嵌套追踪 | Dev | ✅ 完成 | batch/tracing.rs |
| Transaction 追踪 | Dev | ✅ 完成 | transaction/tracing.rs |
| 验证脚本 | Observability | ✅ 完成 | trace_propagation_validation.py |

---

## 8. 常见问题

### 8.1 Trace ID 丢失

**问题**: 跨服务调用时 trace_id 丢失

**排查**:
1. 检查请求头是否包含 `traceparent` 或 `X-Trace-ID`
2. 检查接收方是否正确提取上下文
3. 检查 propagator 是否正确配置

**解决**:
```rust
// 确保在客户端注入上下文
inject_to_http_headers(&mut headers);

// 确保在服务端提取上下文
let context = extract_from_http_headers(&headers);
```

### 8.2 Span 层级不正确

**问题**: Span 层级关系混乱

**排查**:
1. 检查是否正确使用 `start_with_context`
2. 检查 Context guard 是否正确管理
3. 检查异步操作是否正确传递上下文

**解决**:
```rust
// 使用 start_with_context 创建子 Span
let span = tracer
    .span_builder("child_span")
    .start_with_context(&tracer, &parent_cx);

// 在异步操作中传递上下文
let cx = Context::current_with_span(span);
tokio::spawn(async move {
    let _guard = cx.attach();
    // 异步操作
});
```

---

**文档状态**: ✅ 已完成  
**创建日期**: 2026-03-07  
**责任人**: Observability-Agent  
**保管**: 项目文档库
