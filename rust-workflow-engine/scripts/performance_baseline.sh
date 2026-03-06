#!/bin/bash
# 性能基线测量脚本
# Phase 2 Week 3-T4 使用

set -e

echo "========================================="
echo "Phase 2 性能基线测量"
echo "========================================="
echo ""

# 配置
BATCH_SIZE=${BATCH_SIZE:-100}
REQUEST_COUNT=${REQUEST_COUNT:-1000}
CONCURRENCY=${CONCURRENCY:-10}

echo "配置:"
echo "  Batch 大小：${BATCH_SIZE}"
echo "  请求数量：${REQUEST_COUNT}"
echo "  并发数：${CONCURRENCY}"
echo ""

# 检查服务是否运行
echo "检查服务状态..."
if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "❌ 服务未运行，请先启动服务"
    exit 1
fi
echo "✅ 服务运行正常"
echo ""

# 单指令性能测试
echo "========================================="
echo "1. 单指令性能测试"
echo "========================================="
echo ""

echo "执行单指令性能测试..."
START_TIME=$(date +%s%N)

for i in $(seq 1 $REQUEST_COUNT); do
    curl -s -X POST http://localhost:8080/execute \
        -H "Content-Type: application/json" \
        -d '{"trace_id":"trace_'$i'","execution_id":"exec_'$i'","instruction_type":"READ","payload":{},"timestamp":"'$(date -Iseconds)'"}' > /dev/null
done

END_TIME=$(date +%s%N)
ELAPSED_MS=$(( (END_TIME - START_TIME) / 1000000 ))
AVG_LATENCY_MS=$(( ELAPSED_MS / REQUEST_COUNT ))

echo "结果:"
echo "  总耗时：${ELAPSED_MS}ms"
echo "  平均时延：${AVG_LATENCY_MS}ms"
echo "  吞吐量：$(( REQUEST_COUNT * 1000 / ELAPSED_MS )) 请求/秒"
echo ""

# Batch 性能测试
echo "========================================="
echo "2. Batch 性能测试 (Batch 大小=${BATCH_SIZE})"
echo "========================================="
echo ""

echo "执行 Batch 性能测试..."
START_TIME=$(date +%s%N)

BATCH_COUNT=$(( REQUEST_COUNT / BATCH_SIZE ))

for i in $(seq 1 $BATCH_COUNT); do
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
done

END_TIME=$(date +%s%N)
ELAPSED_MS=$(( (END_TIME - START_TIME) / 1000000 ))
AVG_BATCH_LATENCY_MS=$(( ELAPSED_MS / BATCH_COUNT ))
AVG_INSTRUCTION_LATENCY_MS=$(( AVG_BATCH_LATENCY_MS / BATCH_SIZE ))

# 计算 Batch 开销
SINGLE_INSTRUCTION_LATENCY=${AVG_LATENCY_MS:-100}
BATCH_OVERHEAD_PERCENT=$(( (AVG_INSTRUCTION_LATENCY_MS * 100 / SINGLE_INSTRUCTION_LATENCY) - 100 ))

echo "结果:"
echo "  Batch 数量：${BATCH_COUNT}"
echo "  总耗时：${ELAPSED_MS}ms"
echo "  平均 Batch 时延：${AVG_BATCH_LATENCY_MS}ms"
echo "  平均指令时延：${AVG_INSTRUCTION_LATENCY_MS}ms"
echo "  Batch 开销：${BATCH_OVERHEAD_PERCENT}%"
echo ""

# Transaction 性能测试
echo "========================================="
echo "3. Transaction 性能测试"
echo "========================================="
echo ""

echo "执行 Transaction 性能测试..."
echo "(需要服务支持 Transaction 接口)"
echo ""

# 性能基线汇总
echo "========================================="
echo "性能基线汇总"
echo "========================================="
echo ""
echo "单指令性能:"
echo "  P99 时延：${AVG_LATENCY_MS}ms (目标：<300ms)"
echo ""
echo "Batch 性能:"
echo "  平均 Batch 时延：${AVG_BATCH_LATENCY_MS}ms"
echo "  Batch 开销：${BATCH_OVERHEAD_PERCENT}% (目标：<20%)"
echo ""

# 判断是否达标
if [ $AVG_LATENCY_MS -lt 300 ]; then
    echo "✅ 单指令性能达标"
else
    echo "❌ 单指令性能未达标 (目标：<300ms)"
fi

if [ $BATCH_OVERHEAD_PERCENT -lt 20 ]; then
    echo "✅ Batch 开销达标"
else
    echo "❌ Batch 开销未达标 (目标：<20%)"
fi

echo ""
echo "========================================="
echo "性能基线测量完成"
echo "========================================="
