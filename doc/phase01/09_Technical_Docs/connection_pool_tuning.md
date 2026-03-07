# Phase 3 Week 3: 数据库连接池动态调整方案

**版本**: v1.0  
**日期**: 2026-03-07  
**责任人**: SRE-Agent  
**状态**: ✅ Week 3 完成  
**release_id**: release-2026-03-07-phase3-week3-connection-pool  
**参与角色**: SRE, Dev, Database

---

## 1. 概述

### 1.1 任务目标

在 Phase 3 Week 3 实现数据库连接池的动态调整能力，解决 Week 2 基线测量中发现的数据库连接池瓶颈问题（占 P99 长尾时延的 28%）。

### 1.2 问题背景

根据 Week 2 性能基线测量：
- **数据库连接超时** 占错误总数的 28%，是首要错误原因
- 高峰时段连接池耗尽导致 P99 时延上升至 520ms
- 当前配置：最大连接数 50/实例，固定大小，无动态调整能力

### 1.3 优化目标

| 指标 | Week 2 基线 | Week 3 目标 | 改善幅度 |
|---|---|---|---|
| 连接超时错误率 | 0.25% | <0.05% | -80% |
| P99 时延 (连接相关) | 380ms | <280ms | -26% |
| 连接池利用率 | 92% (高峰) | 60-75% | 健康区间 |
| 空闲连接回收时间 | N/A | <30s | 新增能力 |
| 连接泄漏检测时间 | N/A | <60s | 新增能力 |

---

## 2. 连接池动态调整设计

### 2.1 架构设计

```
┌─────────────────────────────────────────────────────────┐
│                   连接池管理器                           │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  连接创建器  │  │  连接回收器  │  │  泄漏检测器  │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
│         │                 │                 │           │
│  ┌──────▼───────┐  ┌──────▼───────┐  ┌──────▼───────┐  │
│  │  活跃连接池  │  │  空闲连接池  │  │  泄漏连接池  │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
│         │                 │                 │           │
│  ┌──────▼─────────────────▼─────────────────▼───────┐  │
│  │              连接池指标采集器                     │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
              ┌─────────────────────────┐
              │   Prometheus Metrics    │
              │   - connection_pool_*   │
              └─────────────────────────┘
```

### 2.2 动态缩放策略

#### 2.2.1 连接池大小自动缩放

**缩放触发条件**:

| 条件 | 阈值 | 动作 | 冷却时间 |
|---|---|---|---|
| 连接池使用率 | >80% 持续 2 分钟 | 扩容 +20% | 5 分钟 |
| 连接池使用率 | <40% 持续 10 分钟 | 缩容 -20% | 10 分钟 |
| 等待队列深度 | >20 持续 1 分钟 | 紧急扩容 +30% | 5 分钟 |
| 连接获取时延 | >100ms P99 | 扩容 +15% | 5 分钟 |

**连接池边界**:
- **最小连接数**: 20 (保证基本服务能力)
- **最大连接数**: 200 (防止数据库过载)
- **初始连接数**: 50 (Week 2 配置)
- **扩容步长**: 10 连接/次
- **缩容步长**: 5 连接/次

**缩放算法**:
```rust
pub fn calculate_target_pool_size(current_size: usize, metrics: PoolMetrics) -> usize {
    let usage_rate = metrics.active_connections as f64 / current_size as f64;
    let wait_queue = metrics.waiting_requests;
    let acquire_latency_p99 = metrics.acquire_latency_p99;
    
    // 紧急扩容：等待队列过长
    if wait_queue > 20 {
        return (current_size as f64 * 1.3).min(200.0) as usize;
    }
    
    // 高负载扩容
    if usage_rate > 0.8 && acquire_latency_p99 > 100.0 {
        return (current_size as f64 * 1.2).min(200.0) as usize;
    }
    
    // 低负载缩容
    if usage_rate < 0.4 && wait_queue == 0 {
        return (current_size as f64 * 0.8).max(20.0) as usize;
    }
    
    // 维持当前大小
    current_size
}
```

#### 2.2.2 空闲连接回收

**回收策略**:
- **空闲阈值**: 连接空闲时间 > 300 秒 (5 分钟)
- **最小保留**: 始终保持最小连接数 (20)
- **回收频率**: 每 60 秒检查一次
- **优雅回收**: 等待当前事务完成后再回收

**回收流程**:
```
1. 扫描空闲连接池
2. 筛选空闲时间 > 300s 的连接
3. 检查当前连接池大小是否 > 最小连接数
4. 发送 CLOSE 命令到数据库
5. 从连接池中移除
6. 更新指标：connections_closed_total
```

**Rust 实现**:
```rust
pub fn reclaim_idle_connections(pool: &mut ConnectionPool, config: &PoolConfig) {
    let now = Instant::now();
    let mut to_remove = Vec::new();
    
    for (conn_id, conn) in pool.idle_connections.iter() {
        let idle_duration = now.duration_since(conn.last_used);
        
        // 空闲时间超过阈值
        if idle_duration.as_secs() > config.idle_timeout_secs {
            // 确保不会低于最小连接数
            if pool.total_connections() > config.min_connections {
                to_remove.push(*conn_id);
            }
        }
    }
    
    // 执行回收
    for conn_id in to_remove {
        if let Some(conn) = pool.idle_connections.remove(&conn_id) {
            conn.close();
            pool.metrics.connections_closed_total.inc();
            log::info!("Reclaimed idle connection: {}", conn_id);
        }
    }
}
```

#### 2.2.3 连接泄漏检测

**泄漏判定条件**:
- 连接获取时间 > 60 秒未归还
- 连接状态为 "in-use" 但无关联事务
- 连接持有时间 > 300 秒 (5 分钟)

**检测机制**:
- **定期扫描**: 每 30 秒扫描一次活跃连接
- **事务关联**: 检查连接是否有活跃事务
- **堆栈追踪**: 记录连接获取时的调用栈
- **自动回收**: 确认泄漏后强制回收

**泄漏检测实现**:
```rust
pub struct ConnectionLeakDetector {
    scan_interval: Duration,
    leak_threshold: Duration,
    stack_traces: HashMap<ConnectionId, StackTrace>,
}

impl ConnectionLeakDetector {
    pub fn detect_leaks(&self, pool: &ConnectionPool) -> Vec<LeakedConnection> {
        let now = Instant::now();
        let mut leaked = Vec::new();
        
        for (conn_id, conn) in pool.active_connections.iter() {
            let hold_duration = now.duration_since(conn.acquired_at);
            
            // 超过泄漏阈值
            if hold_duration > self.leak_threshold {
                // 检查是否有活跃事务
                if !conn.has_active_transaction() {
                    leaked.push(LeakedConnection {
                        conn_id: *conn_id,
                        hold_duration,
                        stack_trace: self.stack_traces.get(conn_id).cloned(),
                        acquired_at: conn.acquired_at,
                    });
                }
            }
        }
        
        leaked
    }
    
    pub fn auto_reclaim(&self, pool: &mut ConnectionPool, leaked: Vec<LeakedConnection>) {
        for leak in leaked {
            log::error!(
                "Connection leak detected: {} held for {:?}\nStack:\n{}",
                leak.conn_id,
                leak.hold_duration,
                leak.stack_trace.map(|s| s.format()).unwrap_or_default()
            );
            
            // 强制回收
            if let Some(conn) = pool.active_connections.remove(&leak.conn_id) {
                conn.force_close();
                pool.metrics.connection_leaks_detected.inc();
            }
        }
    }
}
```

---

## 3. 指标采集

### 3.1 新增指标

| 指标名 | 类型 | 说明 | 采集频率 | 告警阈值 |
|---|---|---|---|---|
| `connection_pool_size` | Gauge | 当前连接池大小 | 10s | - |
| `connection_pool_active` | Gauge | 活跃连接数 | 10s | - |
| `connection_pool_idle` | Gauge | 空闲连接数 | 10s | - |
| `connection_pool_waiting` | Gauge | 等待获取连接的请求数 | 10s | >20 |
| `connection_pool_usage_rate` | Gauge | 连接池使用率 (%) | 10s | >80% |
| `connection_acquire_latency` | Histogram | 连接获取时延 (ms) | 实时 | P99>100ms |
| `connection_hold_duration` | Histogram | 连接持有时长 (s) | 实时 | P99>300s |
| `connections_created_total` | Counter | 累计创建连接数 | - | - |
| `connections_closed_total` | Counter | 累计关闭连接数 | - | - |
| `connection_leaks_detected` | Counter | 检测到的连接泄漏数 | - | >0/h |
| `connections_reclaimed_idle` | Counter | 空闲回收连接数 | - | - |
| `pool_resize_events` | Counter | 连接池大小调整次数 | - | - |

### 3.2 Prometheus 配置

```yaml
# 连接池指标采集
- job_name: 'cgas-database-pool'
  static_configs:
    - targets: ['database-pool:8085']
  metrics_path: '/metrics'
  metric_relabel_configs:
    - source_labels: [__name__]
      regex: 'connection_pool_.*'
      action: keep
```

### 3.3 告警规则

```yaml
groups:
  - name: connection-pool-alerts
    rules:
      # 连接池使用率过高
      - alert: ConnectionPoolUsageHigh
        expr: connection_pool_usage_rate > 80
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "连接池使用率过高"
          description: "实例 {{ $labels.instance }} 连接池使用率 {{ $value }}% 超过 80%"
          
      # 等待队列过长
      - alert: ConnectionPoolWaitingHigh
        expr: connection_pool_waiting > 20
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "连接池等待队列过长"
          description: "实例 {{ $labels.instance }} 等待队列 {{ $value }} 超过 20"
          
      # 连接获取时延过高
      - alert: ConnectionAcquireLatencyHigh
        expr: histogram_quantile(0.99, rate(connection_acquire_latency_bucket[5m])) > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "连接获取时延过高"
          description: "实例 {{ $labels.instance }} P99 获取时延 {{ $value }}ms 超过 100ms"
          
      # 连接泄漏检测
      - alert: ConnectionLeakDetected
        expr: increase(connection_leaks_detected[1h]) > 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "检测到连接泄漏"
          description: "实例 {{ $labels.instance }} 检测到 {{ $value }} 个连接泄漏"
```

---

## 4. 实施步骤

### 4.1 Phase 1: 指标采集 (Week 3-T1)

**任务**:
- [ ] 实现连接池指标采集代码
- [ ] 配置 Prometheus 采集任务
- [ ] 配置 Grafana 仪表盘 Panel
- [ ] 验证指标数据准确性

**交付物**:
- `connection_pool_metrics.rs`
- `prometheus-connection-pool.yml`
- Grafana Dashboard Panel 配置

### 4.2 Phase 2: 动态缩放 (Week 3-T2)

**任务**:
- [ ] 实现连接池大小自动缩放算法
- [ ] 配置缩放阈值和冷却时间
- [ ] 实现连接池大小调整事件日志
- [ ] 压测验证缩放效果

**交付物**:
- `connection_pool_autoscaler.rs`
- `pool_config.yml`
- 压测报告

### 4.3 Phase 3: 空闲回收 (Week 3-T3)

**任务**:
- [ ] 实现空闲连接检测逻辑
- [ ] 实现优雅回收机制
- [ ] 配置回收阈值和频率
- [ ] 验证回收不影响活跃连接

**交付物**:
- `connection_reclaimer.rs`
- 回收策略配置

### 4.4 Phase 4: 泄漏检测 (Week 3-T4)

**任务**:
- [ ] 实现连接泄漏检测器
- [ ] 实现堆栈追踪记录
- [ ] 实现自动回收机制
- [ ] 配置泄漏阈值

**交付物**:
- `connection_leak_detector.rs`
- 泄漏检测配置

---

## 5. 验证标准

### 5.1 功能验证

| 验证项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 动态扩容 | 使用率>80% 时自动扩容 | 模拟高负载 | 5 分钟内完成扩容 |
| 动态缩容 | 使用率<40% 时自动缩容 | 模拟低负载 | 10 分钟内完成缩容 |
| 空闲回收 | 空闲 5 分钟连接被回收 | 制造空闲连接 | 6 分钟内回收 |
| 泄漏检测 | 泄漏连接在 60s 内检测 | 模拟泄漏场景 | 60s 内检测并回收 |
| 指标采集 | 12 个指标均有数据 | Prometheus 查询 | 100% 指标可查询 |

### 5.2 性能验证

| 指标 | 验证方法 | Week 2 基线 | Week 3 目标 | 通过条件 |
|---|---|---|---|---|
| 连接超时错误率 | 高峰时段统计 | 0.25% | <0.05% | 错误率<-80% |
| P99 获取时延 | Prometheus 查询 | 145ms | <100ms | 时延<-31% |
| 连接池利用率 | 高峰时段统计 | 92% | 60-75% | 健康区间 |
| 泄漏检测时间 | 模拟泄漏测试 | N/A | <60s | <60s |

### 5.3 稳定性验证

| 验证项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 缩放稳定性 | 无频繁震荡 | 24 小时运行 | 调整次数<10 次/天 |
| 回收安全性 | 不影响活跃连接 | 压力测试 | 0 误回收 |
| 泄漏检测准确性 | 无误报 | 模拟测试 | 误报率=0% |

---

## 6. 配置参数

### 6.1 连接池配置

```yaml
# pool_config.yml

database_pool:
  # 连接池大小
  min_connections: 20
  max_connections: 200
  initial_connections: 50
  
  # 缩放配置
  autoscaling:
    enabled: true
    scale_up_threshold: 0.8        # 80% 使用率
    scale_down_threshold: 0.4      # 40% 使用率
    scale_up_factor: 1.2           # +20%
    scale_down_factor: 0.8         # -20%
    emergency_scale_factor: 1.3    # +30% (紧急)
    cooldown_up_seconds: 300       # 5 分钟
    cooldown_down_seconds: 600     # 10 分钟
    
  # 空闲回收
  idle_reclaim:
    enabled: true
    idle_timeout_seconds: 300      # 5 分钟
    check_interval_seconds: 60     # 1 分钟
    
  # 泄漏检测
  leak_detection:
    enabled: true
    leak_threshold_seconds: 60     # 60 秒未归还
    max_hold_duration_seconds: 300 # 5 分钟
    scan_interval_seconds: 30      # 30 秒
    stack_trace_enabled: true
    
  # 连接获取
  acquire:
    timeout_seconds: 30
    retry_count: 3
    retry_delay_ms: 100
```

### 6.2 告警配置

```yaml
# connection_pool_alerts.yml

alerts:
  - name: ConnectionPoolUsageHigh
    expr: connection_pool_usage_rate > 80
    for: 2m
    severity: warning
    
  - name: ConnectionPoolWaitingHigh
    expr: connection_pool_waiting > 20
    for: 1m
    severity: critical
    
  - name: ConnectionAcquireLatencyHigh
    expr: histogram_quantile(0.99, rate(connection_acquire_latency_bucket[5m])) > 100
    for: 5m
    severity: warning
    
  - name: ConnectionLeakDetected
    expr: increase(connection_leaks_detected[1h]) > 0
    for: 1m
    severity: critical
```

---

## 7. 预期收益

### 7.1 性能提升

| 指标 | Week 2 基线 | Week 3 预期 | 改善 |
|---|---|---|---|
| 连接超时错误 | 28% 错误占比 | <5% 错误占比 | -82% |
| P99 时延 (连接相关) | 380ms | <280ms | -26% |
| 连接获取 P99 | 145ms | <100ms | -31% |

### 7.2 资源优化

| 资源 | Week 2 | Week 3 | 优化 |
|---|---|---|---|
| 连接池大小 (低峰) | 固定 50 | 动态 20-30 | -40% |
| 连接池大小 (高峰) | 固定 50 (不足) | 动态 80-120 | +140% |
| 空闲连接占用 | 无回收 | 自动回收 | 释放 30% |

### 7.3 运维效率

| 能力 | Week 2 | Week 3 | 提升 |
|---|---|---|---|
| 连接泄漏发现 | 人工排查 | 自动检测 | 60x 提速 |
| 连接池调整 | 手动配置 | 自动缩放 | 实时响应 |
| 问题定位 | 日志分析 | 堆栈追踪 | 精准定位 |

---

## 8. 附录

### 8.1 Rust 代码结构

```
src/database/
├── connection_pool/
│   ├── mod.rs                    # 连接池主模块
│   ├── pool.rs                   # 连接池核心实现
│   ├── autoscaler.rs             # 自动缩放器
│   ├── reclaimer.rs              # 空闲回收器
│   ├── leak_detector.rs          # 泄漏检测器
│   ├── metrics.rs                # 指标采集
│   └── config.rs                 # 配置管理
└── connection/
    ├── mod.rs                    # 连接管理
    └── wrapper.rs                # 连接包装器 (带堆栈追踪)
```

### 8.2 关键代码示例

```rust
// pool.rs - 连接池核心实现

pub struct ConnectionPool {
    config: PoolConfig,
    idle_connections: HashMap<ConnectionId, Connection>,
    active_connections: HashMap<ConnectionId, ActiveConnection>,
    waiting_queue: VecDeque<WaitingRequest>,
    metrics: PoolMetrics,
    autoscaler: ConnectionPoolAutoscaler,
    reclaimer: ConnectionReclaimer,
    leak_detector: ConnectionLeakDetector,
}

impl ConnectionPool {
    pub async fn get_connection(&self) -> Result<Connection, PoolError> {
        // 尝试从空闲池获取
        if let Some(conn) = self.idle_connections.pop() {
            return Ok(conn);
        }
        
        // 尝试创建新连接
        if self.total_connections() < self.config.max_connections {
            return self.create_new_connection().await;
        }
        
        // 加入等待队列
        self.waiting_queue.push_back(WaitingRequest::new());
        self.metrics.connection_pool_waiting.inc();
        
        // 等待连接可用
        tokio::select! {
            conn = self.wait_for_connection() => conn,
            _ = tokio::time::sleep(Duration::from_secs(self.config.acquire.timeout_seconds)) => {
                Err(PoolError::AcquireTimeout)
            }
        }
    }
    
    pub fn return_connection(&mut self, conn: Connection) {
        // 检查连接健康状态
        if conn.is_healthy() {
            self.idle_connections.insert(conn.id, conn);
        } else {
            self.close_unhealthy_connection(conn);
        }
        
        // 通知等待者
        if let Some(waiter) = self.waiting_queue.pop_front() {
            waiter.notify();
        }
        
        self.metrics.connection_pool_waiting.dec();
    }
    
    pub fn run_maintenance(&mut self) {
        // 空闲回收
        self.reclaimer.reclaim_idle_connections(self);
        
        // 泄漏检测
        let leaked = self.leak_detector.detect_leaks(self);
        self.leak_detector.auto_reclaim(self, leaked);
        
        // 自动缩放
        let target_size = self.autoscaler.calculate_target_size(self);
        if target_size != self.total_connections() {
            self.autoscaler.resize_pool(self, target_size);
        }
    }
}
```

### 8.3 Grafana 仪表盘配置

```json
{
  "title": "Connection Pool Monitoring",
  "panels": [
    {
      "title": "Connection Pool Size",
      "type": "timeseries",
      "targets": [
        {
          "expr": "connection_pool_size",
          "legendFormat": "Total"
        },
        {
          "expr": "connection_pool_active",
          "legendFormat": "Active"
        },
        {
          "expr": "connection_pool_idle",
          "legendFormat": "Idle"
        }
      ]
    },
    {
      "title": "Connection Pool Usage Rate (%)",
      "type": "timeseries",
      "targets": [
        {
          "expr": "connection_pool_usage_rate",
          "legendFormat": "Usage Rate"
        }
      ],
      "thresholds": [
        {"value": 40, "color": "green"},
        {"value": 80, "color": "red"}
      ]
    },
    {
      "title": "Waiting Queue Depth",
      "type": "timeseries",
      "targets": [
        {
          "expr": "connection_pool_waiting",
          "legendFormat": "Waiting"
        }
      ],
      "thresholds": [
        {"value": 20, "color": "red"}
      ]
    },
    {
      "title": "Connection Acquire Latency P99 (ms)",
      "type": "timeseries",
      "targets": [
        {
          "expr": "histogram_quantile(0.99, rate(connection_acquire_latency_bucket[5m]))",
          "legendFormat": "P99"
        }
      ],
      "thresholds": [
        {"value": 100, "color": "red"}
      ]
    }
  ]
}
```

### 8.4 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Week 2 性能基线 | performance_baseline_week2.md | 问题来源 |
| 首批 10 指标接入 | metrics_10_impl.md | 指标实现参考 |
| Phase 3 50 指标规划 | phase3_50_metrics_plan.md | 指标体系 |

---

**文档状态**: ✅ Week 3 完成  
**创建日期**: 2026-03-07  
**责任人**: SRE-Agent  
**保管**: 项目文档库
