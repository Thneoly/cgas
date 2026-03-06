#!/bin/bash
# 性能基线测量脚本 v2
# Phase 2 Week 5-T2 使用
# 测量优化后的性能指标

set -e

echo "========================================="
echo "Phase 2 性能基线测量 v2"
echo "优化组件：异步并发池 + 增量重放 + 校验缓存 + 对象池"
echo "========================================="
echo ""

# 配置
BATCH_SIZE=${BATCH_SIZE:-100}
REQUEST_COUNT=${REQUEST_COUNT:-1000}
CONCURRENCY=${CONCURRENCY:-10}
WARMUP_COUNT=${WARMUP_COUNT:-100}

echo "配置:"
echo "  Batch 大小：${BATCH_SIZE}"
echo "  请求数量：${REQUEST_COUNT}"
echo "  并发数：${CONCURRENCY}"
echo "  预热请求：${WARMUP_COUNT}"
echo ""

# 检查服务是否运行
echo "检查服务状态..."
if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "❌ 服务未运行，请先启动服务"
    exit 1
fi
echo "✅ 服务运行正常"
echo ""

# 结果文件
RESULTS_DIR="results/performance_baseline_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

# 单指令性能测试
echo "========================================="
echo "1. 单指令性能测试"
echo "========================================="
echo ""

echo "执行预热..."
for i in $(seq 1 $WARMUP_COUNT); do
    curl -s -X POST http://localhost:8080/execute \
        -H "Content-Type: application/json" \
        -d '{"trace_id":"warmup_'$i'","execution_id":"warmup_exec_'$i'","instruction_type":"READ","payload":{},"timestamp":"'$(date -Iseconds)'"}' > /dev/null
done
echo "✅ 预热完成"
echo ""

echo "执行单指令性能测试..."
LATENCIES=()
START_TIME=$(date +%s%N)

for i in $(seq 1 $REQUEST_COUNT); do
    REQ_START=$(date +%s%N)
    
    curl -s -X POST http://localhost:8080/execute \
        -H "Content-Type: application/json" \
        -d '{"trace_id":"trace_'$i'","execution_id":"exec_'$i'","instruction_type":"READ","payload":{},"timestamp":"'$(date -Iseconds)'"}' > /dev/null
    
    REQ_END=$(date +%s%N)
    LATENCY=$(( (REQ_END - REQ_START) / 1000000 ))
    LATENCIES+=($LATENCY)
done

END_TIME=$(date +%s%N)
ELAPSED_MS=$(( (END_TIME - START_TIME) / 1000000 ))

# 计算统计
IFS=$'\n' SORTED_LATENCIES=($(sort -n <<<"${LATENCIES[*]}"))
unset IFS

P50_INDEX=$(( ${#SORTED_LATENCIES[@]} * 50 / 100 ))
P90_INDEX=$(( ${#SORTED_LATENCIES[@]} * 90 / 100 ))
P99_INDEX=$(( ${#SORTED_LATENCIES[@]} * 99 / 100 ))

P50_LATENCY=${SORTED_LATENCIES[$P50_INDEX]}
P90_LATENCY=${SORTED_LATENCIES[$P90_INDEX]}
P99_LATENCY=${SORTED_LATENCIES[$P99_INDEX]}
AVG_LATENCY=$(( ELAPSED_MS / REQUEST_COUNT ))

echo "结果:"
echo "  总耗时：${ELAPSED_MS}ms"
echo "  平均时延：${AVG_LATENCY}ms"
echo "  P50 时延：${P50_LATENCY}ms"
echo "  P90 时延：${P90_LATENCY}ms"
echo "  P99 时延：${P99_LATENCY}ms"
echo "  吞吐量：$(( REQUEST_COUNT * 1000 / ELAPSED_MS )) 请求/秒"
echo ""

# 保存结果
cat > "$RESULTS_DIR/single_instruction.json" << EOF
{
  "test_type": "single_instruction",
  "request_count": $REQUEST_COUNT,
  "total_elapsed_ms": $ELAPSED_MS,
  "avg_latency_ms": $AVG_LATENCY,
  "p50_latency_ms": $P50_LATENCY,
  "p90_latency_ms": $P90_LATENCY,
  "p99_latency_ms": $P99_LATENCY,
  "throughput_rps": $(( REQUEST_COUNT * 1000 / ELAPSED_MS ))
}
EOF

# Batch 性能测试
echo "========================================="
echo "2. Batch 性能测试 (Batch 大小=${BATCH_SIZE})"
echo "========================================="
echo ""

echo "执行 Batch 预热..."
for i in $(seq 1 10); do
    INSTRUCTIONS=""
    for j in $(seq 1 $BATCH_SIZE); do
        if [ -n "$INSTRUCTIONS" ]; then
            INSTRUCTIONS="${INSTRUCTIONS},"
        fi
        INSTRUCTIONS="${INSTRUCTIONS}{\"trace_id\":\"batch_warmup_${i}_trace_${j}\",\"execution_id\":\"batch_warmup_${i}_exec_${j}\",\"instruction_type\":\"READ\",\"payload\":{},\"timestamp\":\"$(date -Iseconds)\"}"
    done
    
    curl -s -X POST http://localhost:8080/batch/execute \
        -H "Content-Type: application/json" \
        -d "{\"trace_id\":\"batch_warmup_$i\",\"batch_id\":\"batch_warmup_$i\",\"instructions\":[${INSTRUCTIONS}],\"atomic\":true,\"timestamp\":\"$(date -Iseconds)\"}" > /dev/null
done
echo "✅ Batch 预热完成"
echo ""

echo "执行 Batch 性能测试..."
BATCH_COUNT=$(( REQUEST_COUNT / BATCH_SIZE ))
BATCH_LATENCIES=()
START_TIME=$(date +%s%N)

for i in $(seq 1 $BATCH_COUNT); do
    REQ_START=$(date +%s%N)
    
    # 构建 Batch 请求
    INSTRUCTIONS=""
    for j in $(seq 1 $BATCH_SIZE); do
        if [ -n "$INSTRUCTIONS" ]; then
            INSTRUCTIONS="${INSTRUCTIONS},"
        fi
        INSTRUCTIONS="${INSTRUCTIONS}{\"trace_id\":\"batch_${i}_trace_${j}\",\"execution_id\":\"batch_${i}_exec_${j}\",\"instruction_type\":\"READ\",\"payload\":{},\"timestamp\":\"$(date -Iseconds)\"}"
    done
    
    curl -s -X POST http://localhost:8080/batch/execute \
        -H "Content-Type: application/json" \
        -d "{\"trace_id\":\"batch_trace_${i}\",\"batch_id\":\"batch_${i}\",\"instructions\":[${INSTRUCTIONS}],\"atomic\":true,\"timestamp\":\"$(date -Iseconds)\"}" > /dev/null
    
    REQ_END=$(date +%s%N)
    LATENCY=$(( (REQ_END - REQ_START) / 1000000 ))
    BATCH_LATENCIES+=($LATENCY)
done

END_TIME=$(date +%s%N)
ELAPSED_MS=$(( (END_TIME - START_TIME) / 1000000 ))

# 计算 Batch 统计
IFS=$'\n' SORTED_BATCH_LATENCIES=($(sort -n <<<"${BATCH_LATENCIES[*]}"))
unset IFS

P50_INDEX=$(( ${#SORTED_BATCH_LATENCIES[@]} * 50 / 100 ))
P90_INDEX=$(( ${#SORTED_BATCH_LATENCIES[@]} * 90 / 100 ))
P99_INDEX=$(( ${#SORTED_BATCH_LATENCIES[@]} * 99 / 100 ))

P50_BATCH_LATENCY=${SORTED_BATCH_LATENCIES[$P50_INDEX]}
P90_BATCH_LATENCY=${SORTED_BATCH_LATENCIES[$P90_INDEX]}
P99_BATCH_LATENCY=${SORTED_BATCH_LATENCIES[$P99_INDEX]}
AVG_BATCH_LATENCY=$(( ELAPSED_MS / BATCH_COUNT ))
AVG_INSTRUCTION_LATENCY=$(( AVG_BATCH_LATENCY / BATCH_SIZE ))

# 计算 Batch 开销
SINGLE_INSTRUCTION_LATENCY=${P99_LATENCY:-100}
BATCH_OVERHEAD_PERCENT=$(( (AVG_INSTRUCTION_LATENCY * 100 / SINGLE_INSTRUCTION_LATENCY) - 100 ))

echo "结果:"
echo "  Batch 数量：${BATCH_COUNT}"
echo "  总耗时：${ELAPSED_MS}ms"
echo "  平均 Batch 时延：${AVG_BATCH_LATENCY}ms"
echo "  平均指令时延：${AVG_INSTRUCTION_LATENCY}ms"
echo "  P50 Batch 时延：${P50_BATCH_LATENCY}ms"
echo "  P90 Batch 时延：${P90_BATCH_LATENCY}ms"
echo "  P99 Batch 时延：${P99_BATCH_LATENCY}ms"
echo "  Batch 开销：${BATCH_OVERHEAD_PERCENT}%"
echo ""

# 保存结果
cat > "$RESULTS_DIR/batch_performance.json" << EOF
{
  "test_type": "batch_performance",
  "batch_size": $BATCH_SIZE,
  "batch_count": $BATCH_COUNT,
  "total_elapsed_ms": $ELAPSED_MS,
  "avg_batch_latency_ms": $AVG_BATCH_LATENCY,
  "avg_instruction_latency_ms": $AVG_INSTRUCTION_LATENCY,
  "p50_batch_latency_ms": $P50_BATCH_LATENCY,
  "p90_batch_latency_ms": $P90_BATCH_LATENCY,
  "p99_batch_latency_ms": $P99_BATCH_LATENCY,
  "batch_overhead_percent": $BATCH_OVERHEAD_PERCENT
}
EOF

# 性能基线汇总
echo "========================================="
echo "性能基线汇总"
echo "========================================="
echo ""
echo "单指令性能:"
echo "  P50 时延：${P50_LATENCY}ms"
echo "  P90 时延：${P90_LATENCY}ms"
echo "  P99 时延：${P99_LATENCY}ms (目标：<300ms)"
echo "  吞吐量：$(( REQUEST_COUNT * 1000 / ELAPSED_MS )) 请求/秒"
echo ""
echo "Batch 性能:"
echo "  平均 Batch 时延：${AVG_BATCH_LATENCY}ms"
echo "  平均指令时延：${AVG_INSTRUCTION_LATENCY}ms"
echo "  P99 Batch 时延：${P99_BATCH_LATENCY}ms"
echo "  Batch 开销：${BATCH_OVERHEAD_PERCENT}% (目标：<20%)"
echo ""

# 判断是否达标
echo "========================================="
echo "达标判断"
echo "========================================="
echo ""

PASSED=true

if [ $P99_LATENCY -lt 300 ]; then
    echo "✅ 单指令 P99 时延达标 (${P99_LATENCY}ms < 300ms)"
else
    echo "❌ 单指令 P99 时延未达标 (${P99_LATENCY}ms >= 300ms)"
    PASSED=false
fi

if [ $BATCH_OVERHEAD_PERCENT -lt 20 ]; then
    echo "✅ Batch 开销达标 (${BATCH_OVERHEAD_PERCENT}% < 20%)"
else
    echo "❌ Batch 开销未达标 (${BATCH_OVERHEAD_PERCENT}% >= 20%)"
    PASSED=false
fi

echo ""

# 保存汇总结果
cat > "$RESULTS_DIR/summary.json" << EOF
{
  "test_date": "$(date -Iseconds)",
  "single_instruction": {
    "p50_latency_ms": $P50_LATENCY,
    "p90_latency_ms": $P90_LATENCY,
    "p99_latency_ms": $P99_LATENCY,
    "throughput_rps": $(( REQUEST_COUNT * 1000 / ELAPSED_MS )),
    "target_p99_ms": 300,
    "passed": $([ $P99_LATENCY -lt 300 ] && echo "true" || echo "false")
  },
  "batch_performance": {
    "batch_size": $BATCH_SIZE,
    "avg_batch_latency_ms": $AVG_BATCH_LATENCY,
    "avg_instruction_latency_ms": $AVG_INSTRUCTION_LATENCY,
    "p99_batch_latency_ms": $P99_BATCH_LATENCY,
    "batch_overhead_percent": $BATCH_OVERHEAD_PERCENT,
    "target_overhead_percent": 20,
    "passed": $([ $BATCH_OVERHEAD_PERCENT -lt 20 ] && echo "true" || echo "false")
  },
  "overall_passed": $PASSED
}
EOF

echo "========================================="
if [ "$PASSED" = true ]; then
    echo "✅ 性能基线测量完成 - 全部达标"
else
    echo "❌ 性能基线测量完成 - 部分未达标"
fi
echo "========================================="
echo ""
echo "结果保存在：$RESULTS_DIR"
echo ""
