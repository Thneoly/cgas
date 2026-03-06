# Phase 2 Week 2 性能基线压测方案

**版本**: v1.0  
**日期**: 2026-04-08  
**责任人**: SRE  
**状态**: 📋 待执行  
**release_id**: release-2026-04-08-phase2_week02  

---

## 1. 压测目标

### 1.1 Week 2 性能目标

建立 Phase 2 性能基线，验证 Batch 指令性能开销<20%。

| 指标 | Phase 1 基线 | Week 2 目标 | 状态 |
|---|---|---|---|
| P99 执行时延 | 423ms | 基线测量 | 📋 待测量 |
| P99 验证时延 | 467ms | 基线测量 | 📋 待测量 |
| Batch 开销 | N/A | <20% | 📋 待验证 |
| 吞吐量 | 基准值 | ≥基准值 | 📋 待测量 |

### 1.2 Phase 2 性能目标

| 指标 | Phase 1 基线 | Phase 2 目标 | 提升 |
|---|---|---|---|
| P99 执行时延 | 423ms | **<300ms** | -29% |
| P99 验证时延 | 467ms | **<300ms** | -36% |
| 阻断开销 | 8.5% | **<5%** | -41% |

---

## 2. 压测环境

### 2.1 环境配置

| 环境 | 节点数 | CPU | 内存 | 用途 |
|---|---|---|---|---|
| performance | 5 | 8 核 | 16GB | 性能压测 |
| staging | 3 | 4 核 | 8GB | 集成测试 |

### 2.2 压测工具

| 工具 | 用途 | 版本 |
|---|---|---|
| k6 | 负载生成 | v0.45 |
| Vegeta | HTTP 压测 | v0.17 |
| Prometheus | 指标采集 | v2.45 |
| Grafana | 仪表盘 | v10.0 |
| Jaeger | 分布式追踪 | v1.45 |

---

## 3. 压测场景

### 3.1 单指令基线测试

| 场景 ID | 场景描述 | 负载 | 持续时间 | 指标 |
|---|---|---|---|---|
| BASE-001 | 单指令执行时延 | 100 请求/秒 | 10 分钟 | P50/P90/P99 |
| BASE-002 | 单指令验证时延 | 100 请求/秒 | 10 分钟 | P50/P90/P99 |
| BASE-003 | 单指令吞吐量 | 渐增负载 | 30 分钟 | 最大吞吐量 |

### 3.2 Batch 性能测试

| 场景 ID | 场景描述 | 负载 | Batch 大小 | 持续时间 | 指标 |
|---|---|---|---|---|---|
| BATCH-001 | Batch 执行时延 (1 条) | 100 请求/秒 | 1 | 10 分钟 | P50/P90/P99 |
| BATCH-002 | Batch 执行时延 (10 条) | 100 请求/秒 | 10 | 10 分钟 | P50/P90/P99 |
| BATCH-003 | Batch 执行时延 (50 条) | 50 请求/秒 | 50 | 10 分钟 | P50/P90/P99 |
| BATCH-004 | Batch 执行时延 (100 条) | 25 请求/秒 | 100 | 10 分钟 | P50/P90/P99 |
| BATCH-005 | Batch 开销对比 | 100 请求/秒 | 1/10/50/100 | 30 分钟 | 开销百分比 |

### 3.3 并发测试

| 场景 ID | 场景描述 | 并发数 | 持续时间 | 指标 |
|---|---|---|---|---|
| CONC-001 | 低并发 (10) | 10 | 10 分钟 | 成功率/时延 |
| CONC-002 | 中并发 (50) | 50 | 10 分钟 | 成功率/时延 |
| CONC-003 | 高并发 (100) | 100 | 10 分钟 | 成功率/时延 |
| CONC-004 | 极限并发 (200) | 200 | 10 分钟 | 成功率/时延 |

### 3.4 稳定性测试

| 场景 ID | 场景描述 | 负载 | 持续时间 | 指标 |
|---|---|---|---|---|
| STAB-001 | 短期稳定性 | 100 请求/秒 | 1 小时 | 故障数 |
| STAB-002 | 中期稳定性 | 100 请求/秒 | 4 小时 | 故障数 |
| STAB-003 | 长期稳定性 | 100 请求/秒 | 24 小时 | 故障数 |

---

## 4. 压测脚本

### 4.1 k6 脚本示例

```javascript
// batch_performance_test.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// 自定义指标
const successRate = new Rate('success_rate');
const batchLatency = new Rate('batch_latency');

export let options = {
  stages: [
    { duration: '5m', target: 50 },   //  ramp up to 50 RPS
    { duration: '10m', target: 50 },  // stay at 50 RPS
    { duration: '5m', target: 100 },  // ramp up to 100 RPS
    { duration: '10m', target: 100 }, // stay at 100 RPS
    { duration: '5m', target: 0 },    // ramp down
  ],
  thresholds: {
    http_req_duration: ['p(99)<400'], // P99 < 400ms
    success_rate: ['rate>0.999'],     // 成功率 > 99.9%
  },
};

export default function() {
  // Batch 执行请求
  const payload = {
    trace_id: `trace_${__VU}_${__ITER}`,
    batch_id: `batch_${__VU}_${__ITER}`,
    instructions: generateInstructions(10), // 10 条指令
    atomic: true,
    timestamp: new Date().toISOString(),
  };
  
  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };
  
  const res = http.post(
    'http://batch-service:8082/batch/execute',
    JSON.stringify(payload),
    params
  );
  
  // 验证响应
  const success = check(res, {
    'status is 200': (r) => r.status === 200,
    'batch success': (r) => r.json('status') === 'SUCCESS',
    'has results': (r) => r.json('results.length') === 10,
  });
  
  successRate.add(success);
  
  sleep(0.1); // 10ms 间隔
}

function generateInstructions(count) {
  const instructions = [];
  for (let i = 0; i < count; i++) {
    instructions.push({
      trace_id: `sub_${__VU}_${__ITER}_${i}`,
      execution_id: `exec_${__VU}_${__ITER}_${i}`,
      instruction_type: 'CREATE',
      payload: { key: `value_${i}` },
      timestamp: new Date().toISOString(),
    });
  }
  return instructions;
}
```

### 4.2 Vegeta 攻击文件

```
# batch_attack.txt

POST http://batch-service:8082/batch/execute
Content-Type: application/json

{"trace_id":"trace_1","batch_id":"batch_1","instructions":[...],"atomic":true,"timestamp":"2026-04-08T10:00:00Z"}
```

**执行命令**:
```bash
vegeta attack -targets=batch_attack.txt -rate=100/s -duration=10m | vegeta report
```

---

## 5. 监控指标

### 5.1 核心指标

| 指标名 | 类型 | 采集频率 | 告警阈值 |
|---|---|---|---|
| batch_execute_latency_p99 | Histogram | 实时 | >400ms |
| batch_execute_latency_p50 | Histogram | 实时 | >200ms |
| batch_success_rate | Gauge | 实时 | <99.9% |
| batch_throughput | Counter | 实时 | - |
| batch_error_count | Counter | 实时 | >0 |

### 5.2 资源指标

| 指标名 | 类型 | 采集频率 | 告警阈值 |
|---|---|---|---|
| cpu_usage_percent | Gauge | 15s | >80% |
| memory_usage_percent | Gauge | 15s | >85% |
| network_io_bytes | Counter | 15s | - |
| disk_io_ops | Counter | 15s | - |

---

## 6. 压测执行计划

### 6.1 Week 2 压测安排

| 时间 | 压测类型 | 场景 | 责任人 | 状态 |
|---|---|---|---|---|
| Week 2-T3 | 单指令基线 | BASE-001/002/003 | SRE | 📋 待开始 |
| Week 2-T4 | Batch 性能 | BATCH-001/002/003/004/005 | SRE+Dev | 📋 待开始 |
| Week 2-T5 | 并发测试 | CONC-001/002/003/004 | SRE | 📋 待开始 |

### 6.2 压测报告

**输出**: `phase2_week2_performance_report.md`

**内容**:
- 压测环境配置
- 压测场景执行结果
- 性能指标统计 (P50/P90/P99)
- 与 Phase 1 基线对比
- 瓶颈分析与优化建议
- 结论与下一步行动

---

## 7. 性能验收标准

### 7.1 Week 2 验收标准

| 指标 | 目标值 | 验收标准 |
|---|---|---|
| Batch 执行时延 (1 条) | <450ms | P99 < 450ms |
| Batch 执行时延 (10 条) | <500ms | P99 < 500ms |
| Batch 开销 | <20% | 相比单条<20% |
| 成功率 | ≥99.9% | 所有场景 |

### 7.2 Phase 2 Exit 验收标准

| 指标 | Phase 2 目标 | 验收标准 |
|---|---|---|
| P99 执行时延 | <300ms | P99 < 300ms |
| P99 验证时延 | <300ms | P99 < 300ms |
| 阻断开销 | <5% | 开销 < 5% |

---

## 8. 性能优化预案

### 8.1 性能瓶颈识别

| 瓶颈类型 | 识别方法 | 优化措施 |
|---|---|---|
| CPU 瓶颈 | CPU>80% | 异步并发、对象池 |
| 内存瓶颈 | 内存>85% | 减少分配、流式处理 |
| 网络瓶颈 | 网络 IO 高 | 批量处理、压缩 |
| 锁竞争 | 等待时间长 | 细粒度锁、无锁设计 |

### 8.2 优化措施

| 优化项 | 预期提升 | 实施难度 | 周次 |
|---|---|---|---|
| 异步并发池 | -20% | 中 | Week 2 |
| 增量重放 | -15% | 高 | Week 3 |
| 校验缓存 | -10% | 低 | Week 3 |
| 对象池复用 | -5% | 低 | Week 4 |

---

## 9. 附录

### 9.1 压测检查清单

| 检查项 | 状态 | 备注 |
|---|---|---|
| 压测环境就绪 | 📋 | 5 节点集群 |
| 监控配置完成 | 📋 | Prometheus+Grafana |
| 压测脚本准备 | 📋 | k6/Vegeta |
| 基线数据准备 | 📋 | Phase 1 数据 |
| 告警配置完成 | 📋 | P0/P1/P2 告警 |

### 9.2 参考文档

- Phase 2 SRE 规划 v1
- Phase 2 Batch 设计文档
- Phase 1 性能基线报告 v3

---

**文档状态**: 📋 待执行  
**责任人**: SRE  
**保管**: 项目文档库
