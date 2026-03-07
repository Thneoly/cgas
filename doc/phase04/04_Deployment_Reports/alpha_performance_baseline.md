# Alpha 环境性能基线测量

**版本**: v1.0  
**日期**: 2026-03-07  
**责任人**: Dev-Agent + SRE-Agent  
**环境**: Alpha (生产验证环境)  
**状态**: 📋 待测量  

---

## 📋 目录

1. [性能基线目标](#1-性能基线目标)
2. [性能指标定义](#2-性能指标定义)
3. [测量工具与脚本](#3-测量工具与脚本)
4. [测量场景](#4-测量场景)
5. [基线数据记录](#5-基线数据记录)
6. [性能优化建议](#6-性能优化建议)

---

## 1. 性能基线目标

### 1.1 Phase 4 性能目标

基于 Phase 3 实测数据，制定 Alpha 环境性能基线目标：

| 指标类别 | 指标名称 | Phase 3 实测 | Alpha 目标 | 阈值 |
|----------|----------|--------------|------------|------|
| **执行性能** | P99 执行时延 | 178ms | <200ms | 300ms |
| **执行性能** | P95 执行时延 | 148ms | <180ms | 250ms |
| **执行性能** | P50 执行时延 | 95ms | <120ms | 150ms |
| **验证性能** | P99 验证时延 | 172ms | <200ms | 300ms |
| **验证性能** | P95 验证时延 | 138ms | <180ms | 250ms |
| **吞吐量** | 峰值 QPS | 4,850 | ≥4,500 | 4,000 |
| **吞吐量** | 持续 QPS | 4,200 | ≥4,000 | 3,500 |
| **缓存** | 缓存命中率 | 97.2% | >95% | 90% |
| **队列** | 执行器队列深度 | 45 | <100 | 200 |
| **队列** | 验证器队列深度 | 38 | <100 | 200 |

### 1.2 资源使用基线

| 资源类型 | 指标 | Phase 3 实测 | Alpha 阈值 | 告警阈值 |
|----------|------|--------------|------------|----------|
| **CPU** | 平均使用率 | 52% | <70% | >80% |
| **CPU** | 峰值使用率 | 68% | <80% | >90% |
| **内存** | 平均使用率 | 64% | <75% | >85% |
| **内存** | 峰值使用率 | 78% | <85% | >90% |
| **磁盘** | IO 等待 | 11% | <20% | >30% |
| **磁盘** | 使用率 | 58% | <70% | >80% |
| **网络** | 丢包率 | 0.15% | <0.5% | >1% |
| **网络** | 带宽使用 | 45% | <60% | >80% |

### 1.3 质量基线

| 指标 | Phase 3 实测 | Alpha 目标 | 阈值 |
|------|--------------|------------|------|
| E2E 通过率 | 100% | ≥99.5% | ≥99% |
| 重放一致率 | 100% | ≥99.97% | ≥99.9% |
| 误报率 | 1.08% | <1.5% | <2% |
| 缺陷逃逸率 | 0.3% | <0.5% | <1% |

---

## 2. 性能指标定义

### 2.1 核心性能指标

#### 执行时延 (Execution Latency)

```
定义：从指令提交到执行完成的时间
测量点：
  - 开始：指令进入执行队列
  - 结束：执行结果返回
计算公式：execution_end_time - execution_start_time
分位数：P50, P90, P95, P99
```

#### 验证时延 (Validation Latency)

```
定义：从验证请求到验证完成的时间
测量点：
  - 开始：验证请求接收
  - 结束：验证结果返回
计算公式：validation_end_time - validation_start_time
分位数：P50, P90, P95, P99
```

#### 吞吐量 (Throughput)

```
定义：单位时间内处理的指令数量
测量方式：
  - 瞬时 QPS：每秒处理的指令数
  - 平均 QPS：测量期间平均每秒处理数
  - 峰值 QPS：测量期间最高 QPS
单位：queries per second (QPS)
```

#### 缓存命中率 (Cache Hit Rate)

```
定义：缓存命中次数占总访问次数的比例
计算公式：cache_hits / (cache_hits + cache_misses) * 100%
目标：>95%
```

### 2.2 资源指标

#### CPU 使用率

```
定义：CPU 时间的占用百分比
测量方式：
  - 用户态 CPU
  - 内核态 CPU
  - IO 等待
告警阈值：>80% (Warning), >90% (Critical)
```

#### 内存使用率

```
定义：内存占用占总内存的比例
测量方式：
  - RSS (Resident Set Size)
  - Heap 使用
  - Stack 使用
告警阈值：>85% (Warning), >90% (Critical)
```

### 2.3 业务指标

#### 工作流执行成功率

```
定义：成功执行的工作流占总工作流的比例
计算公式：successful_workflows / total_workflows * 100%
目标：≥99.5%
```

#### Batch 处理效率

```
定义：Batch 处理的平均指令时延
计算公式：batch_total_time / batch_instruction_count
目标：<20% 开销 (相比单指令)
```

---

## 3. 测量工具与脚本

### 3.1 性能基线测量脚本 `performance_baseline_alpha.sh`

```bash
#!/bin/bash
# Alpha 环境性能基线测量脚本
# Phase 4 Week 1-T1

set -e

echo "========================================="
echo "Alpha 环境性能基线测量"
echo "========================================="
echo ""

# 配置
SERVICE_URL=${SERVICE_URL:-"http://cgas-workflow-engine-alpha:8080"}
BATCH_SIZE=${BATCH_SIZE:-100}
REQUEST_COUNT=${REQUEST_COUNT:-1000}
CONCURRENCY=${CONCURRENCY:-10}
WARMUP_COUNT=${WARMUP_COUNT:-100}
TEST_DURATION=${TEST_DURATION:-300}  # 5 分钟

# 结果目录
RESULTS_DIR="results/performance_baseline_alpha_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "配置:"
echo "  服务地址：${SERVICE_URL}"
echo "  Batch 大小：${BATCH_SIZE}"
echo "  请求数量：${REQUEST_COUNT}"
echo "  并发数：${CONCURRENCY}"
echo "  预热请求：${WARMUP_COUNT}"
echo "  测试时长：${TEST_DURATION}秒"
echo "  结果目录：${RESULTS_DIR}"
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# 检查服务健康
check_health() {
    echo "检查服务健康状态..."
    
    local response=$(curl -s -w "%{http_code}" ${SERVICE_URL}/health || echo "000")
    
    if [ "$response" == "200" ]; then
        echo -e "${GREEN}✅ 服务健康检查通过${NC}"
        return 0
    else
        echo -e "${RED}❌ 服务健康检查失败 (${response})${NC}"
        return 1
    fi
}

# 单指令性能测试
test_single_instruction() {
    echo ""
    echo "========================================="
    echo "1. 单指令性能测试"
    echo "========================================="
    echo ""
    
    echo "执行预热 (${WARMUP_COUNT} 请求)..."
    for i in $(seq 1 $WARMUP_COUNT); do
        curl -s -X POST ${SERVICE_URL}/execute \
            -H "Content-Type: application/json" \
            -d '{"trace_id":"warmup_'$i'","execution_id":"warmup_exec_'$i'","instruction_type":"READ","payload":{},"timestamp":"'$(date -Iseconds)'"}' > /dev/null
    done
    echo -e "${GREEN}✅ 预热完成${NC}"
    echo ""
    
    echo "执行性能测试 (${REQUEST_COUNT} 请求)..."
    
    local latencies_file="${RESULTS_DIR}/single_instruction_latencies.txt"
    local start_time=$(date +%s%N)
    
    for i in $(seq 1 $REQUEST_COUNT); do
        local req_start=$(date +%s%N)
        
        curl -s -X POST ${SERVICE_URL}/execute \
            -H "Content-Type: application/json" \
            -d '{"trace_id":"trace_'$i'","execution_id":"exec_'$i'","instruction_type":"READ","payload":{},"timestamp":"'$(date -Iseconds)'"}' > /dev/null
        
        local req_end=$(date +%s%N)
        local latency=$(( (req_end - req_start) / 1000000 ))
        echo $latency >> "$latencies_file"
    done
    
    local end_time=$(date +%s%N)
    local elapsed_ms=$(( (end_time - start_time) / 1000000 ))
    
    # 计算统计
    calculate_stats "$latencies_file" $elapsed_ms $REQUEST_COUNT "single_instruction"
}

# Batch 性能测试
test_batch_performance() {
    echo ""
    echo "========================================="
    echo "2. Batch 性能测试"
    echo "========================================="
    echo ""
    
    local batch_count=$(( REQUEST_COUNT / BATCH_SIZE ))
    
    echo "执行 Batch 预热..."
    for i in $(seq 1 10); do
        local instructions=""
        for j in $(seq 1 $BATCH_SIZE); do
            if [ -n "$instructions" ]; then
                instructions="${instructions},"
            fi
            instructions="${instructions}{\"trace_id\":\"batch_warmup_${i}_trace_${j}\",\"execution_id\":\"batch_warmup_${i}_exec_${j}\",\"instruction_type\":\"READ\",\"payload\":{},\"timestamp\":\"$(date -Iseconds)\"}"
        done
        
        curl -s -X POST ${SERVICE_URL}/batch/execute \
            -H "Content-Type: application/json" \
            -d "{\"trace_id\":\"batch_warmup_$i\",\"batch_id\":\"batch_warmup_$i\",\"instructions\":[${instructions}],\"atomic\":true,\"timestamp\":\"$(date -Iseconds)\"}" > /dev/null
    done
    echo -e "${GREEN}✅ Batch 预热完成${NC}"
    echo ""
    
    echo "执行 Batch 性能测试 (${batch_count} batches)..."
    
    local latencies_file="${RESULTS_DIR}/batch_latencies.txt"
    local start_time=$(date +%s%N)
    
    for i in $(seq 1 $batch_count); do
        local req_start=$(date +%s%N)
        
        # 构建 Batch 请求
        local instructions=""
        for j in $(seq 1 $BATCH_SIZE); do
            if [ -n "$instructions" ]; then
                instructions="${instructions},"
            fi
            instructions="${instructions}{\"trace_id\":\"batch_${i}_trace_${j}\",\"execution_id\":\"batch_${i}_exec_${j}\",\"instruction_type\":\"READ\",\"payload\":{},\"timestamp\":\"$(date -Iseconds)\"}"
        done
        
        curl -s -X POST ${SERVICE_URL}/batch/execute \
            -H "Content-Type: application/json" \
            -d "{\"trace_id\":\"batch_trace_${i}\",\"batch_id\":\"batch_${i}\",\"instructions\":[${instructions}],\"atomic\":true,\"timestamp\":\"$(date -Iseconds)\"}" > /dev/null
        
        local req_end=$(date +%s%N)
        local latency=$(( (req_end - req_start) / 1000000 ))
        echo $latency >> "$latencies_file"
    done
    
    local end_time=$(date +%s%N)
    local elapsed_ms=$(( (end_time - start_time) / 1000000 ))
    
    calculate_stats "$latencies_file" $elapsed_ms $batch_count "batch_performance"
}

# 计算统计数据
calculate_stats() {
    local file=$1
    local elapsed_ms=$2
    local count=$3
    local test_type=$4
    
    # 排序
    sort -n "$file" -o "${file}.sorted"
    
    # 计算分位数
    local p50_index=$(( count * 50 / 100 ))
    local p90_index=$(( count * 90 / 100 ))
    local p95_index=$(( count * 95 / 100 ))
    local p99_index=$(( count * 99 / 100 ))
    
    local p50=$(sed -n "${p50_index}p" "${file}.sorted")
    local p90=$(sed -n "${p90_index}p" "${file}.sorted")
    local p95=$(sed -n "${p95_index}p" "${file}.sorted")
    local p99=$(sed -n "${p99_index}p" "${file}.sorted")
    
    local avg=$(( elapsed_ms / count ))
    local throughput=$(( count * 1000 / elapsed_ms ))
    
    echo ""
    echo "统计结果:"
    echo "  总耗时：${elapsed_ms}ms"
    echo "  平均时延：${avg}ms"
    echo "  P50 时延：${p50}ms"
    echo "  P90 时延：${p90}ms"
    echo "  P95 时延：${p95}ms"
    echo "  P99 时延：${p99}ms"
    echo "  吞吐量：${throughput} QPS"
    echo ""
    
    # 保存 JSON 结果
    cat > "${RESULTS_DIR}/${test_type}_stats.json" << EOF
{
  "test_type": "${test_type}",
  "test_time": "$(date -Iseconds)",
  "request_count": $count,
  "total_elapsed_ms": $elapsed_ms,
  "avg_latency_ms": $avg,
  "p50_latency_ms": $p50,
  "p90_latency_ms": $p90,
  "p95_latency_ms": $p95,
  "p99_latency_ms": $p99,
  "throughput_qps": $throughput
}
EOF
}

# 并发压力测试
test_concurrency() {
    echo ""
    echo "========================================="
    echo "3. 并发压力测试"
    echo "========================================="
    echo ""
    
    echo "执行并发测试 (并发数：${CONCURRENCY})..."
    
    # 使用 ab 或 wrk 进行压力测试 (如果可用)
    if command -v ab &> /dev/null; then
        ab -n $REQUEST_COUNT -c $CONCURRENCY \
            -T application/json \
            -p test_payload.json \
            ${SERVICE_URL}/execute 2>&1 | tee "${RESULTS_DIR}/concurrency_test.txt"
    else
        echo "ℹ️  ab 未安装，使用简易并发测试"
        
        local start_time=$(date +%s%N)
        
        for i in $(seq 1 $CONCURRENCY); do
            (
                for j in $(seq 1 $((REQUEST_COUNT / CONCURRENCY))); do
                    curl -s -X POST ${SERVICE_URL}/execute \
                        -H "Content-Type: application/json" \
                        -d '{"trace_id":"concurrent_'$i'_'$j'","execution_id":"exec_'$i'_'$j'","instruction_type":"READ","payload":{},"timestamp":"'$(date -Iseconds)'"}' > /dev/null
                done
            ) &
        done
        
        wait
        
        local end_time=$(date +%s%N)
        local elapsed_ms=$(( (end_time - start_time) / 1000000 ))
        local throughput=$(( REQUEST_COUNT * 1000 / elapsed_ms ))
        
        echo ""
        echo "并发测试结果:"
        echo "  总耗时：${elapsed_ms}ms"
        echo "  吞吐量：${throughput} QPS"
    fi
}

# 稳定性测试
test_stability() {
    echo ""
    echo "========================================="
    echo "4. 稳定性测试 (${TEST_DURATION}秒)"
    echo "========================================="
    echo ""
    
    echo "开始稳定性测试..."
    
    local errors=0
    local total_requests=0
    local start_time=$(date +%s)
    local end_time=$((start_time + TEST_DURATION))
    
    while [ $(date +%s) -lt $end_time ]; do
        local response=$(curl -s -w "%{http_code}" -X POST ${SERVICE_URL}/execute \
            -H "Content-Type: application/json" \
            -d '{"trace_id":"stability_'$(date +%s)'","execution_id":"exec_'$(date +%s)'","instruction_type":"READ","payload":{},"timestamp":"'$(date -Iseconds)'"}' 2>/dev/null || echo "000")
        
        ((total_requests++))
        
        if [ "$response" != "200" ]; then
            ((errors++))
        fi
        
        sleep 0.1  # 10ms 间隔
    done
    
    local error_rate=$(( errors * 100 / total_requests ))
    local actual_duration=$(($(date +%s) - start_time))
    local avg_qps=$(( total_requests / actual_duration ))
    
    echo ""
    echo "稳定性测试结果:"
    echo "  测试时长：${actual_duration}秒"
    echo "  总请求数：${total_requests}"
    echo "  错误数：${errors}"
    echo "  错误率：${error_rate}%"
    echo "  平均 QPS：${avg_qps}"
    echo ""
    
    # 保存结果
    cat > "${RESULTS_DIR}/stability_test.json" << EOF
{
  "test_type": "stability",
  "test_time": "$(date -Iseconds)",
  "duration_seconds": $actual_duration,
  "total_requests": $total_requests,
  "errors": $errors,
  "error_rate_percent": $error_rate,
  "avg_qps": $avg_qps
}
EOF
}

# 生成汇总报告
generate_summary() {
    echo ""
    echo "========================================="
    echo "性能基线汇总报告"
    echo "========================================="
    echo ""
    
    # 读取各测试结果
    local single_stats="${RESULTS_DIR}/single_instruction_stats.json"
    local batch_stats="${RESULTS_DIR}/batch_performance_stats.json"
    local stability_stats="${RESULTS_DIR}/stability_test.json"
    
    if [ -f "$single_stats" ]; then
        echo "单指令性能:"
        jq -r '  "  P99 时延: \(.p99_latency_ms)ms (目标：<300ms)\n  吞吐量：\(.throughput_qps) QPS (目标：≥4500)"' "$single_stats"
        echo ""
    fi
    
    if [ -f "$batch_stats" ]; then
        echo "Batch 性能:"
        jq -r '  "  平均 Batch 时延：\(.avg_latency_ms)ms\n  P99 Batch 时延：\(.p99_latency_ms)ms"' "$batch_stats"
        echo ""
    fi
    
    if [ -f "$stability_stats" ]; then
        echo "稳定性:"
        jq -r '  "  错误率：\(.error_rate_percent)% (目标：<0.5%)\n  平均 QPS：\(.avg_qps)"' "$stability_stats"
        echo ""
    fi
    
    # 判断是否达标
    echo "========================================="
    echo "达标判断"
    echo "========================================="
    echo ""
    
    local all_passed=true
    
    if [ -f "$single_stats" ]; then
        local p99=$(jq -r '.p99_latency_ms' "$single_stats")
        if [ $p99 -lt 300 ]; then
            echo -e "${GREEN}✅ 单指令 P99 时延达标 (${p99}ms < 300ms)${NC}"
        else
            echo -e "${RED}❌ 单指令 P99 时延未达标 (${p99}ms >= 300ms)${NC}"
            all_passed=false
        fi
    fi
    
    if [ -f "$stability_stats" ]; then
        local error_rate=$(jq -r '.error_rate_percent' "$stability_stats")
        if [ $error_rate -lt 1 ]; then
            echo -e "${GREEN}✅ 稳定性错误率达标 (${error_rate}% < 1%)${NC}"
        else
            echo -e "${RED}❌ 稳定性错误率未达标 (${error_rate}% >= 1%)${NC}"
            all_passed=false
        fi
    fi
    
    echo ""
    
    if [ "$all_passed" = true ]; then
        echo -e "${GREEN}✅ 性能基线测量完成 - 全部达标${NC}"
    else
        echo -e "${YELLOW}⚠️  性能基线测量完成 - 部分未达标${NC}"
    fi
    
    echo ""
    echo "结果保存在：${RESULTS_DIR}"
}

# 主流程
echo "1. 健康检查..."
if ! check_health; then
    echo -e "${RED}❌ 服务未就绪，终止测试${NC}"
    exit 1
fi
echo ""

echo "2. 单指令性能测试..."
test_single_instruction

echo "3. Batch 性能测试..."
test_batch_performance

echo "4. 并发压力测试..."
test_concurrency

echo "5. 稳定性测试..."
test_stability

echo "6. 生成汇总报告..."
generate_summary

echo ""
echo "========================================="
echo -e "${GREEN}✅ 性能基线测量完成${NC}"
echo "========================================="
```

### 3.2 Prometheus 查询模板 `prometheus_queries.md`

```markdown
# Prometheus 性能查询模板

## 执行时延

### P99 执行时延
```promql
histogram_quantile(0.99, rate(execution_latency_seconds_bucket[5m]))
```

### P95 执行时延
```promql
histogram_quantile(0.95, rate(execution_latency_seconds_bucket[5m]))
```

### 平均执行时延
```promql
rate(execution_latency_seconds_sum[5m]) / rate(execution_latency_seconds_count[5m])
```

## 吞吐量

### 每秒请求数
```promql
rate(instructions_processed_total[5m])
```

### 峰值 QPS (1 小时)
```promql
max_over_time(rate(instructions_processed_total[1m])[1h:1m])
```

## 缓存

### 缓存命中率
```promql
rate(cache_hits_total[5m]) / (rate(cache_hits_total[5m]) + rate(cache_misses_total[5m]))
```

## 队列

### 执行器队列深度
```promql
executor_queue_depth
```

### 验证器队列深度
```promql
validator_queue_depth
```

## 资源

### CPU 使用率
```promql
rate(container_cpu_usage_seconds_total{container="workflow-engine"}[5m]) * 100
```

### 内存使用率
```promql
container_memory_usage_bytes{container="workflow-engine"} / container_spec_memory_limit_bytes{container="workflow-engine"} * 100
```

### 错误率
```promql
rate(errors_total[5m]) / rate(requests_total[5m]) * 100
```
```

---

## 4. 测量场景

### 4.1 场景 1: 单指令性能

```yaml
场景：单指令执行性能
目标：测量单指令的 P99/P95/P50 时延
负载：
  - 请求数：1000
  - 并发数：1
  - 指令类型：READ
预期结果:
  - P99 < 300ms
  - P95 < 250ms
  - P50 < 150ms
```

### 4.2 场景 2: Batch 性能

```yaml
场景：Batch 指令执行性能
目标：测量 Batch 处理的时延和开销
负载:
  - Batch 数量：10
  - Batch 大小：100
  - 并发数：1
预期结果:
  - Batch 开销 < 20%
  - P99 Batch 时延 < 5s
```

### 4.3 场景 3: 并发压力

```yaml
场景：高并发压力测试
目标：测量系统在高并发下的表现
负载:
  - 并发数：10-100
  - 持续时间：5 分钟
  - 请求类型：混合 (READ/WRITE)
预期结果:
  - 错误率 < 0.5%
  - P99 < 500ms
  - 吞吐量 ≥ 4500 QPS
```

### 4.4 场景 4: 稳定性

```yaml
场景：长时间稳定性测试
目标：验证系统长时间运行的稳定性
负载:
  - 持续时间：5-30 分钟
  - 请求速率：100 QPS
  - 请求类型：混合
预期结果:
  - 零故障
  - 错误率 < 0.5%
  - 资源使用稳定
```

---

## 5. 基线数据记录

### 5.1 性能基线数据表

| 测量时间 | 测试场景 | P50(ms) | P90(ms) | P95(ms) | P99(ms) | QPS | 错误率 | 状态 |
|----------|----------|---------|---------|---------|---------|-----|--------|------|
| - | 单指令 | - | - | - | - | - | - | 📋 |
| - | Batch(100) | - | - | - | - | - | - | 📋 |
| - | 并发 (10) | - | - | - | - | - | - | 📋 |
| - | 并发 (50) | - | - | - | - | - | - | 📋 |
| - | 并发 (100) | - | - | - | - | - | - | 📋 |
| - | 稳定性 (5min) | - | - | - | - | - | - | 📋 |

### 5.2 资源使用基线表

| 测量时间 | CPU 平均 | CPU 峰值 | 内存平均 | 内存峰值 | 磁盘 IO | 网络 | 状态 |
|----------|----------|----------|----------|----------|---------|------|------|
| - | - | - | - | - | - | - | 📋 |

---

## 6. 性能优化建议

### 6.1 已实施优化 (Phase 3)

1. **异步并发池**: 减少线程创建开销
2. **增量重放**: 减少重复计算
3. **校验缓存**: 缓存验证结果
4. **对象池**: 复用对象减少 GC

### 6.2 Alpha 环境优化建议

1. **连接池调优**:
   - 数据库连接池：min=10, max=100
   - Redis 连接池：min=10, max=50

2. **JVM/Rust 运行时调优**:
   - 堆内存：2GB
   - GC 策略：G1GC

3. **Kubernetes 资源配置**:
   - CPU 请求：500m, 限制：2000m
   - 内存请求：512Mi, 限制：2Gi

4. **缓存策略**:
   - 验证结果缓存：TTL=300s
   - 配置缓存：TTL=600s

### 6.3 监控告警配置

| 指标 | Warning | Critical | 响应时间 |
|------|---------|----------|----------|
| P99 时延 | >250ms | >300ms | 15 分钟 |
| 错误率 | >1% | >2% | 5 分钟 |
| CPU 使用 | >70% | >85% | 15 分钟 |
| 内存使用 | >75% | >90% | 15 分钟 |
| 队列深度 | >100 | >200 | 10 分钟 |

---

**文档状态**: ✅ 性能基线文档完成  
**测量时间**: Week 1-T1 (2026-04-01)  
**责任人**: Dev-Agent + SRE-Agent  
**保管**: CGAS 项目文档库

---

*Alpha Performance Baseline v1.0 - 2026-03-07*
