#!/bin/bash
# CGAS Staging Environment Automated Validation Script
# 自动化验证脚本 - 验证 Prometheus 和 Grafana 状态

set -e

# 配置
GRAFANA_URL="http://localhost:3003"
GRAFANA_USER="admin"
GRAFANA_PASS="StagingGrafana2026"
PROMETHEUS_URL="http://localhost:9093"

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "======================================"
echo "CGAS Staging Environment Validation"
echo "======================================"
echo ""

# 1. 检查 Prometheus targets
echo -n "1. Checking Prometheus targets... "
TARGETS_UP=$(curl -sf "${PROMETHEUS_URL}/api/v1/targets" | jq '[.data.activeTargets[] | select(.health == "up")] | length')
TARGETS_TOTAL=$(curl -sf "${PROMETHEUS_URL}/api/v1/targets" | jq '[.data.activeTargets[]] | length')

if [ "$TARGETS_UP" -eq "$TARGETS_TOTAL" ] && [ "$TARGETS_UP" -gt 0 ]; then
    echo -e "${GREEN}✓${NC} ${TARGETS_UP}/${TARGETS_TOTAL} targets healthy"
else
    echo -e "${RED}✗${NC} ${TARGETS_UP}/${TARGETS_TOTAL} targets healthy"
    exit 1
fi

# 2. 检查 Grafana 数据源
echo -n "2. Checking Grafana datasources... "
DATASOURCES=$(curl -sf "${GRAFANA_URL}/api/datasources" -u "${GRAFANA_USER}:${GRAFANA_PASS}" | jq 'length')

if [ "$DATASOURCES" -gt 0 ]; then
    echo -e "${GREEN}✓${NC} ${DATASOURCES} datasources configured"
else
    echo -e "${RED}✗${NC} No datasources configured"
    exit 1
fi

# 3. 检查 Grafana 仪表盘
echo -n "3. Checking Grafana dashboards... "
DASHBOARDS=$(curl -sf "${GRAFANA_URL}/api/search?type=dash-db" -u "${GRAFANA_USER}:${GRAFANA_PASS}" | jq 'length')

if [ "$DASHBOARDS" -gt 0 ]; then
    echo -e "${GREEN}✓${NC} ${DASHBOARDS} dashboards available"
else
    echo -e "${RED}✗${NC} No dashboards available"
    exit 1
fi

# 4. 验证实际数据
echo -n "4. Verifying actual data... "
DATA_AVAILABLE=$(curl -sf "${GRAFANA_URL}/api/datasources/proxy/1/api/v1/query?query=count(up)" -u "${GRAFANA_USER}:${GRAFANA_PASS}" | jq '.data.result | length')

if [ "$DATA_AVAILABLE" -gt 0 ]; then
    echo -e "${GREEN}✓${NC} Data available in Grafana"
else
    echo -e "${RED}✗${NC} No data in Grafana"
    exit 1
fi

# 5. 检查告警规则
echo -n "5. Checking alert rules... "
ALERT_RULES=$(curl -sf "${PROMETHEUS_URL}/api/v1/rules" | jq '.data.groups[0].rules | length')

if [ "$ALERT_RULES" -gt 0 ]; then
    echo -e "${GREEN}✓${NC} ${ALERT_RULES} alert rules loaded"
else
    echo -e "${YELLOW}⚠${NC} No alert rules loaded"
fi

# 6. 等待数据采集
echo ""
echo "6. Waiting for data collection (30 seconds)..."
sleep 30

# 7. 再次验证数据
echo -n "7. Re-verifying data after wait... "
DATA_COUNT=$(curl -sf "${PROMETHEUS_URL}/api/v1/query?query=node_memory_MemTotal_bytes" | jq '.data.result | length')

if [ "$DATA_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓${NC} ${DATA_COUNT} time series available"
else
    echo -e "${RED}✗${NC} No time series available"
    exit 1
fi

echo ""
echo "======================================"
echo -e "${GREEN}All validations passed!${NC}"
echo "======================================"
echo ""
echo "Summary:"
echo "  - Prometheus targets: ${TARGETS_UP}/${TARGETS_TOTAL} healthy"
echo "  - Grafana datasources: ${DATASOURCES}"
echo "  - Grafana dashboards: ${DASHBOARDS}"
echo "  - Alert rules: ${ALERT_RULES}"
echo "  - Time series: ${DATA_COUNT}"
echo ""
echo "Validation completed at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
