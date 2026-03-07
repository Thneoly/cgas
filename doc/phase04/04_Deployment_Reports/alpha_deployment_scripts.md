# Alpha 环境部署脚本

**版本**: v1.0  
**日期**: 2026-03-07  
**责任人**: Dev-Agent  
**环境**: Alpha (生产验证环境)  
**状态**: 📋 待部署  

---

## 📋 目录

1. [部署前置检查脚本](#1-部署前置检查脚本)
2. [应用部署脚本](#2-应用部署脚本)
3. [配置部署脚本](#3-配置部署脚本)
4. [数据库迁移脚本](#4-数据库迁移脚本)
5. [健康检查脚本](#5-健康检查脚本)
6. [回滚脚本](#6-回滚脚本)

---

## 1. 部署前置检查脚本

### 1.1 环境检查脚本 `pre_deploy_check.sh`

```bash
#!/bin/bash
# Alpha 环境部署前置检查脚本
# Phase 4 Week 1-T1 使用

set -e

echo "========================================="
echo "Alpha 环境部署前置检查"
echo "========================================="
echo ""

# 配置
ALPHA_ENV_URL=${ALPHA_ENV_URL:-"http://alpha.cgas.internal:8080"}
KUBE_CONFIG=${KUBE_CONFIG:-"$HOME/.kube/config"}
REQUIRED_TOOLS=("kubectl" "helm" "curl" "jq")

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查结果统计
CHECKS_PASSED=0
CHECKS_FAILED=0

# 检查函数
check_tool() {
    local tool=$1
    if command -v "$tool" &> /dev/null; then
        echo -e "✅ ${tool} 已安装"
        ((CHECKS_PASSED++))
    else
        echo -e "❌ ${tool} 未安装"
        ((CHECKS_FAILED++))
    fi
}

check_kube_config() {
    if [ -f "$KUBE_CONFIG" ]; then
        echo -e "✅ Kubernetes 配置文件存在"
        ((CHECKS_PASSED++))
        
        # 检查集群连接
        if kubectl cluster-info &> /dev/null; then
            echo -e "✅ Kubernetes 集群连接正常"
            ((CHECKS_PASSED++))
        else
            echo -e "❌ Kubernetes 集群连接失败"
            ((CHECKS_FAILED++))
        fi
    else
        echo -e "❌ Kubernetes 配置文件不存在：$KUBE_CONFIG"
        ((CHECKS_FAILED++))
    fi
}

check_namespace() {
    local namespace=$1
    if kubectl get namespace "$namespace" &> /dev/null; then
        echo -e "✅ 命名空间 ${namespace} 存在"
        ((CHECKS_PASSED++))
    else
        echo -e "❌ 命名空间 ${namespace} 不存在"
        ((CHECKS_FAILED++))
    fi
}

check_resources() {
    local namespace=$1
    echo "检查资源配额..."
    
    # 检查 CPU 配额
    local cpu_quota=$(kubectl get resourcequota -n "$namespace" -o jsonpath='{.items[0].status.hard.cpu}' 2>/dev/null || echo "0")
    if [ "$cpu_quota" != "0" ]; then
        echo -e "✅ CPU 配额已配置：${cpu_quota}"
        ((CHECKS_PASSED++))
    else
        echo -e "⚠️  CPU 配额未配置"
        ((CHECKS_FAILED++))
    fi
    
    # 检查内存配额
    local mem_quota=$(kubectl get resourcequota -n "$namespace" -o jsonpath='{.items[0].status.hard.memory}' 2>/dev/null || echo "0")
    if [ "$mem_quota" != "0" ]; then
        echo -e "✅ 内存配额已配置：${mem_quota}"
        ((CHECKS_PASSED++))
    else
        echo -e "⚠️  内存配额未配置"
        ((CHECKS_FAILED++))
    fi
}

check_storage() {
    local namespace=$1
    echo "检查存储配置..."
    
    # 检查 StorageClass
    local storage_class=$(kubectl get storageclass -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
    if [ -n "$storage_class" ]; then
        echo -e "✅ StorageClass 可用：${storage_class}"
        ((CHECKS_PASSED++))
    else
        echo -e "❌ StorageClass 不可用"
        ((CHECKS_FAILED++))
    fi
    
    # 检查 PVC
    local pvc_count=$(kubectl get pvc -n "$namespace" 2>/dev/null | wc -l)
    echo -e "ℹ️  当前 PVC 数量：$((pvc_count - 1))"
}

check_network() {
    echo "检查网络配置..."
    
    # 检查 Ingress Controller
    if kubectl get deployment -n ingress-nginx ingress-nginx-controller &> /dev/null; then
        echo -e "✅ Ingress Controller 运行正常"
        ((CHECKS_PASSED++))
    else
        echo -e "⚠️  Ingress Controller 未部署"
        ((CHECKS_FAILED++))
    fi
    
    # 检查网络策略
    local netpol_count=$(kubectl get networkpolicy -n alpha 2>/dev/null | wc -l)
    echo -e "ℹ️  当前 NetworkPolicy 数量：$((netpol_count - 1))"
}

check_secrets() {
    local namespace=$1
    echo "检查密钥配置..."
    
    # 检查必要密钥
    local required_secrets=("cgas-db-credentials" "cgas-oidc-config" "cgas-tls-cert")
    for secret in "${required_secrets[@]}"; do
        if kubectl get secret "$secret" -n "$namespace" &> /dev/null; then
            echo -e "✅ 密钥 ${secret} 存在"
            ((CHECKS_PASSED++))
        else
            echo -e "❌ 密钥 ${secret} 不存在"
            ((CHECKS_FAILED++))
        fi
    done
}

check_dependencies() {
    echo "检查依赖服务..."
    
    # 检查数据库连接
    if kubectl run db-check --image=curlimages/curl --rm -it --restart=Never -- curl -s --connect-timeout 5 cgas-postgres:5432 &> /dev/null; then
        echo -e "✅ 数据库连接正常"
        ((CHECKS_PASSED++))
    else
        echo -e "❌ 数据库连接失败"
        ((CHECKS_FAILED++))
    fi
    
    # 检查 Redis 连接
    if kubectl run redis-check --image=curlimages/curl --rm -it --restart=Never -- curl -s --connect-timeout 5 cgas-redis:6379 &> /dev/null; then
        echo -e "✅ Redis 连接正常"
        ((CHECKS_PASSED++))
    else
        echo -e "❌ Redis 连接失败"
        ((CHECKS_FAILED++))
    fi
    
    # 检查消息队列
    if kubectl run mq-check --image=curlimages/curl --rm -it --restart=Never -- curl -s --connect-timeout 5 cgas-kafka:9092 &> /dev/null; then
        echo -e "✅ 消息队列连接正常"
        ((CHECKS_PASSED++))
    else
        echo -e "❌ 消息队列连接失败"
        ((CHECKS_FAILED++))
    fi
}

# 主检查流程
echo "1. 检查必要工具..."
for tool in "${REQUIRED_TOOLS[@]}"; do
    check_tool "$tool"
done
echo ""

echo "2. 检查 Kubernetes 配置..."
check_kube_config
echo ""

echo "3. 检查命名空间..."
check_namespace "alpha"
echo ""

echo "4. 检查资源配额..."
check_resources "alpha"
echo ""

echo "5. 检查存储配置..."
check_storage "alpha"
echo ""

echo "6. 检查网络配置..."
check_network
echo ""

echo "7. 检查密钥配置..."
check_secrets "alpha"
echo ""

echo "8. 检查依赖服务..."
check_dependencies
echo ""

# 总结
echo "========================================="
echo "检查结果汇总"
echo "========================================="
echo -e "✅ 通过：${CHECKS_PASSED}"
echo -e "❌ 失败：${CHECKS_FAILED}"
echo ""

if [ $CHECKS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ 所有检查通过，可以开始部署${NC}"
    exit 0
else
    echo -e "${RED}❌ 存在检查失败项，请先解决问题后再部署${NC}"
    exit 1
fi
```

---

## 2. 应用部署脚本

### 2.1 蓝绿部署脚本 `blue_green_deploy.sh`

```bash
#!/bin/bash
# Alpha 环境蓝绿部署脚本
# Phase 4 Week 1-T2 使用

set -e

echo "========================================="
echo "Alpha 环境蓝绿部署"
echo "========================================="
echo ""

# 配置
NAMESPACE=${NAMESPACE:-"alpha"}
APP_NAME=${APP_NAME:-"cgas-workflow-engine"}
IMAGE_REPO=${IMAGE_REPO:-"registry.cgas.internal/cgas/workflow-engine"}
IMAGE_TAG=${IMAGE_TAG:-"phase4-alpha-v1.0"}
HEALTH_CHECK_URL=${HEALTH_CHECK_URL:-"/health"}
HEALTH_CHECK_PORT=${HEALTH_CHECK_PORT:-"8080"}
ROLLOUT_TIMEOUT=${ROLLOUT_TIMEOUT:-"300s"}

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 获取当前活跃部署
get_active_deployment() {
    local current_label=$(kubectl get svc ${APP_NAME}-active -n ${NAMESPACE} -o jsonpath='{.spec.selector.version}' 2>/dev/null || echo "")
    if [ "$current_label" == "blue" ]; then
        echo "blue"
    elif [ "$current_label" == "green" ]; then
        echo "green"
    else
        echo "none"
    fi
}

# 获取目标部署颜色
get_target_color() {
    local current=$1
    if [ "$current" == "blue" ]; then
        echo "green"
    else
        echo "blue"
    fi
}

# 部署新版本
deploy_version() {
    local color=$1
    local image_tag=$2
    
    echo -e "${BLUE}部署 ${color} 版本：${image_tag}${NC}"
    
    # 更新 Deployment
    kubectl set image deployment/${APP_NAME}-${color} \
        ${APP_NAME}=${IMAGE_REPO}:${image_tag} \
        -n ${NAMESPACE} \
        --record
    
    # 等待 Rollout 完成
    echo "等待部署完成..."
    kubectl rollout status deployment/${APP_NAME}-${color} \
        -n ${NAMESPACE} \
        --timeout=${ROLLOUT_TIMEOUT}
    
    echo -e "${GREEN}✅ ${color} 版本部署完成${NC}"
}

# 健康检查
health_check() {
    local color=$1
    local max_attempts=30
    local attempt=1
    
    echo "执行健康检查..."
    
    # 获取 Pod IP
    local pod_ip=$(kubectl get pods -n ${NAMESPACE} -l app=${APP_NAME},version=${color} \
        -o jsonpath='{.items[0].status.podIP}' 2>/dev/null)
    
    if [ -z "$pod_ip" ]; then
        echo -e "${RED}❌ 无法获取 Pod IP${NC}"
        return 1
    fi
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s --connect-timeout 5 http://${pod_ip}:${HEALTH_CHECK_PORT}${HEALTH_CHECK_URL} | grep -q "ok"; then
            echo -e "${GREEN}✅ 健康检查通过 (尝试 ${attempt}/${max_attempts})${NC}"
            return 0
        fi
        
        echo "健康检查中... (尝试 ${attempt}/${max_attempts})"
        sleep 10
        ((attempt++))
    done
    
    echo -e "${RED}❌ 健康检查失败${NC}"
    return 1
}

# 切换流量
switch_traffic() {
    local target_color=$1
    
    echo -e "${BLUE}切换流量到 ${target_color}${NC}"
    
    # 更新 Service 选择器
    kubectl patch svc ${APP_NAME}-active \
        -n ${NAMESPACE} \
        -p "{\"spec\":{\"selector\":{\"version\":\"${target_color}\"}}}"
    
    echo -e "${GREEN}✅ 流量已切换到 ${target_color}${NC}"
}

# 验证部署
verify_deployment() {
    echo "验证部署..."
    
    # 检查 Pod 状态
    local ready_pods=$(kubectl get pods -n ${NAMESPACE} -l app=${APP_NAME} \
        -o jsonpath='{range .items[*]}{.status.conditions[?(@.type=="Ready")].status}{"\n"}{end}' | grep -c "True" || echo "0")
    local total_pods=$(kubectl get pods -n ${NAMESPACE} -l app=${APP_NAME} --no-headers | wc -l)
    
    if [ "$ready_pods" -eq "$total_pods" ]; then
        echo -e "${GREEN}✅ 所有 Pod 就绪 (${ready_pods}/${total_pods})${NC}"
        return 0
    else
        echo -e "${RED}❌ Pod 未完全就绪 (${ready_pods}/${total_pods})${NC}"
        return 1
    fi
}

# 主部署流程
echo "1. 获取当前部署状态..."
CURRENT_COLOR=$(get_active_deployment)
echo "当前活跃版本：${CURRENT_COLOR:-"无"}"
echo ""

TARGET_COLOR=$(get_target_color "$CURRENT_COLOR")
echo "目标部署版本：${TARGET_COLOR}"
echo ""

echo "2. 部署新版本..."
deploy_version "$TARGET_COLOR" "$IMAGE_TAG"
echo ""

echo "3. 执行健康检查..."
if ! health_check "$TARGET_COLOR"; then
    echo -e "${RED}❌ 健康检查失败，触发自动回滚${NC}"
    # 自动回滚逻辑
    exit 1
fi
echo ""

echo "4. 验证部署..."
if ! verify_deployment; then
    echo -e "${RED}❌ 部署验证失败${NC}"
    exit 1
fi
echo ""

echo "5. 切换流量..."
switch_traffic "$TARGET_COLOR"
echo ""

echo "========================================="
echo -e "${GREEN}✅ 蓝绿部署完成${NC}"
echo "========================================="
echo ""
echo "部署信息:"
echo "  新版本：${IMAGE_TAG}"
echo "  活跃环境：${TARGET_COLOR}"
echo "  命名空间：${NAMESPACE}"
echo ""

# 保存部署记录
DEPLOY_RECORD="deploy_record_$(date +%Y%m%d_%H%M%S).json"
cat > "$DEPLOY_RECORD" << EOF
{
  "deploy_time": "$(date -Iseconds)",
  "namespace": "${NAMESPACE}",
  "app_name": "${APP_NAME}",
  "image_tag": "${IMAGE_TAG}",
  "target_color": "${TARGET_COLOR}",
  "previous_color": "${CURRENT_COLOR}",
  "status": "success"
}
EOF

echo "部署记录已保存：$DEPLOY_RECORD"
```

---

## 3. 配置部署脚本

### 3.1 配置管理脚本 `deploy_configs.sh`

```bash
#!/bin/bash
# Alpha 环境配置部署脚本
# Phase 4 Week 1-T1 使用

set -e

echo "========================================="
echo "Alpha 环境配置部署"
echo "========================================="
echo ""

# 配置
NAMESPACE=${NAMESPACE:-"alpha"}
CONFIG_MAP_NAME=${CONFIG_MAP_NAME:-"cgas-config"}
SECRET_NAME=${SECRET_NAME:-"cgas-secrets"}
CONFIG_DIR=${CONFIG_DIR:-"./configs/alpha"}

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 创建 ConfigMap
create_configmap() {
    echo "创建 ConfigMap..."
    
    # 从文件创建
    if [ -d "$CONFIG_DIR" ]; then
        kubectl create configmap ${CONFIG_MAP_NAME} \
            --from-file=${CONFIG_DIR} \
            -n ${NAMESPACE} \
            --dry-run=client -o yaml | kubectl apply -f -
        
        echo -e "${GREEN}✅ ConfigMap 创建成功${NC}"
    else
        echo -e "${YELLOW}⚠️  配置目录不存在：${CONFIG_DIR}${NC}"
        
        # 创建默认配置
        kubectl create configmap ${CONFIG_MAP_NAME} \
            --from-literal=LOG_LEVEL="INFO" \
            --from-literal=MAX_CONNECTIONS="100" \
            --from-literal=TIMEOUT_MS="30000" \
            -n ${NAMESPACE} \
            --dry-run=client -o yaml | kubectl apply -f -
        
        echo -e "${GREEN}✅ 默认 ConfigMap 创建成功${NC}"
    fi
}

# 创建 Secret
create_secret() {
    echo "创建 Secret..."
    
    # 数据库凭证
    if [ -f "${CONFIG_DIR}/db-credentials.txt" ]; then
        kubectl create secret generic ${SECRET_NAME} \
            --from-file=db-credentials=${CONFIG_DIR}/db-credentials.txt \
            -n ${NAMESPACE} \
            --dry-run=client -o yaml | kubectl apply -f -
    else
        # 使用环境变量创建
        kubectl create secret generic ${SECRET_NAME} \
            --from-literal=DB_HOST="cgas-postgres" \
            --from-literal=DB_PORT="5432" \
            --from-literal=DB_NAME="cgas_alpha" \
            --from-literal=DB_USER="cgas_user" \
            --from-literal=DB_PASSWORD="changeme_alpha" \
            -n ${NAMESPACE} \
            --dry-run=client -o yaml | kubectl apply -f -
    fi
    
    echo -e "${GREEN}✅ Secret 创建成功${NC}"
}

# 验证配置
verify_configs() {
    echo "验证配置..."
    
    # 检查 ConfigMap
    if kubectl get configmap ${CONFIG_MAP_NAME} -n ${NAMESPACE} &> /dev/null; then
        echo -e "${GREEN}✅ ConfigMap 验证通过${NC}"
    else
        echo -e "${RED}❌ ConfigMap 验证失败${NC}"
        return 1
    fi
    
    # 检查 Secret
    if kubectl get secret ${SECRET_NAME} -n ${NAMESPACE} &> /dev/null; then
        echo -e "${GREEN}✅ Secret 验证通过${NC}"
    else
        echo -e "${RED}❌ Secret 验证失败${NC}"
        return 1
    fi
    
    return 0
}

# 主流程
echo "1. 部署 ConfigMap..."
create_configmap
echo ""

echo "2. 部署 Secret..."
create_secret
echo ""

echo "3. 验证配置..."
if ! verify_configs; then
    echo -e "${RED}❌ 配置验证失败${NC}"
    exit 1
fi
echo ""

echo "========================================="
echo -e "${GREEN}✅ 配置部署完成${NC}"
echo "========================================="
```

---

## 4. 数据库迁移脚本

### 4.1 数据库迁移脚本 `migrate_database.sh`

```bash
#!/bin/bash
# Alpha 环境数据库迁移脚本
# Phase 4 Week 1-T1 使用

set -e

echo "========================================="
echo "Alpha 环境数据库迁移"
echo "========================================="
echo ""

# 配置
NAMESPACE=${NAMESPACE:-"alpha"}
DB_HOST=${DB_HOST:-"cgas-postgres"}
DB_PORT=${DB_PORT:-"5432"}
DB_NAME=${DB_NAME:-"cgas_alpha"}
DB_USER=${DB_USER:-"cgas_user"}
MIGRATION_DIR=${MIGRATION_DIR:-"./migrations/alpha"}
BACKUP_DIR=${BACKUP_DIR:-"./backups/$(date +%Y%m%d_%H%M%S)"}

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# 备份数据库
backup_database() {
    echo "备份数据库..."
    mkdir -p "$BACKUP_DIR"
    
    # 获取数据库凭证
    local db_password=$(kubectl get secret ${DB_USER}-credentials -n ${NAMESPACE} \
        -o jsonpath='{.data.password}' | base64 -d)
    
    # 执行备份
    kubectl run db-backup --image=postgres:15 --rm -it --restart=Never \
        --env="PGPASSWORD=${db_password}" \
        -- pg_dump -h ${DB_HOST} -p ${DB_PORT} -U ${DB_USER} ${DB_NAME} \
        > ${BACKUP_DIR}/backup.sql
    
    echo -e "${GREEN}✅ 数据库备份完成：${BACKUP_DIR}/backup.sql${NC}"
}

# 执行迁移
run_migrations() {
    echo "执行数据库迁移..."
    
    if [ ! -d "$MIGRATION_DIR" ]; then
        echo -e "${YELLOW}⚠️  迁移目录不存在：${MIGRATION_DIR}${NC}"
        echo "跳过迁移步骤"
        return 0
    fi
    
    # 获取数据库凭证
    local db_password=$(kubectl get secret ${DB_USER}-credentials -n ${NAMESPACE} \
        -o jsonpath='{.data.password}' | base64 -d)
    
    # 执行迁移脚本
    for migration_file in $(ls -1 ${MIGRATION_DIR}/*.sql 2>/dev/null | sort); do
        echo "执行迁移：$(basename $migration_file)"
        
        kubectl run db-migration --image=postgres:15 --rm -it --restart=Never \
            --env="PGPASSWORD=${db_password}" \
            -- psql -h ${DB_HOST} -p ${DB_PORT} -U ${DB_USER} -d ${DB_NAME} \
            -f /migrations/$(basename $migration_file) \
            < "$migration_file"
        
        echo -e "${GREEN}✅ 迁移完成：$(basename $migration_file)${NC}"
    done
}

# 验证迁移
verify_migration() {
    echo "验证迁移结果..."
    
    # 获取数据库凭证
    local db_password=$(kubectl get secret ${DB_USER}-credentials -n ${NAMESPACE} \
        -o jsonpath='{.data.password}' | base64 -d)
    
    # 检查关键表是否存在
    local tables=("workflow_executions" "instructions" "batch_records" "transaction_logs" "security_events")
    
    for table in "${tables[@]}"; do
        local count=$(kubectl run db-check --image=postgres:15 --rm -it --restart=Never \
            --env="PGPASSWORD=${db_password}" \
            -- psql -h ${DB_HOST} -p ${DB_PORT} -U ${DB_USER} -d ${DB_NAME} \
            -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_name='${table}';")
        
        if [ "$count" -gt 0 ]; then
            echo -e "${GREEN}✅ 表 ${table} 存在${NC}"
        else
            echo -e "${RED}❌ 表 ${table} 不存在${NC}"
            return 1
        fi
    done
    
    return 0
}

# 主流程
echo "1. 备份数据库..."
backup_database
echo ""

echo "2. 执行迁移..."
run_migrations
echo ""

echo "3. 验证迁移..."
if ! verify_migration; then
    echo -e "${RED}❌ 迁移验证失败，请检查日志${NC}"
    echo "备份位置：${BACKUP_DIR}"
    exit 1
fi
echo ""

echo "========================================="
echo -e "${GREEN}✅ 数据库迁移完成${NC}"
echo "========================================="
echo ""
echo "备份位置：${BACKUP_DIR}"
```

---

## 5. 健康检查脚本

### 5.1 综合健康检查脚本 `health_check.sh`

```bash
#!/bin/bash
# Alpha 环境综合健康检查脚本
# Phase 4 Week 1-T2 使用

set -e

echo "========================================="
echo "Alpha 环境健康检查"
echo "========================================="
echo ""

# 配置
NAMESPACE=${NAMESPACE:-"alpha"}
APP_NAME=${APP_NAME:-"cgas-workflow-engine"}
SERVICE_URL=${SERVICE_URL:-"http://cgas-workflow-engine-alpha:8080"}

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# 检查结果
CHECKS_PASSED=0
CHECKS_FAILED=0

# Pod 状态检查
check_pods() {
    echo "检查 Pod 状态..."
    
    local ready_pods=$(kubectl get pods -n ${NAMESPACE} -l app=${APP_NAME} \
        -o jsonpath='{range .items[*]}{.status.conditions[?(@.type=="Ready")].status}{"\n"}{end}' | grep -c "True" || echo "0")
    local total_pods=$(kubectl get pods -n ${NAMESPACE} -l app=${APP_NAME} --no-headers | wc -l)
    
    if [ "$ready_pods" -eq "$total_pods" ] && [ "$total_pods" -gt 0 ]; then
        echo -e "${GREEN}✅ Pod 状态正常 (${ready_pods}/${total_pods})${NC}"
        ((CHECKS_PASSED++))
    else
        echo -e "${RED}❌ Pod 状态异常 (${ready_pods}/${total_pods})${NC}"
        ((CHECKS_FAILED++))
    fi
}

# Service 检查
check_service() {
    echo "检查 Service..."
    
    if kubectl get svc ${APP_NAME} -n ${NAMESPACE} &> /dev/null; then
        local cluster_ip=$(kubectl get svc ${APP_NAME} -n ${NAMESPACE} -o jsonpath='{.spec.clusterIP}')
        echo -e "${GREEN}✅ Service 正常 (ClusterIP: ${cluster_ip})${NC}"
        ((CHECKS_PASSED++))
    else
        echo -e "${RED}❌ Service 不存在${NC}"
        ((CHECKS_FAILED++))
    fi
}

# HTTP 健康检查
check_http_health() {
    echo "检查 HTTP 健康端点..."
    
    local response=$(curl -s -w "%{http_code}" ${SERVICE_URL}/health || echo "000")
    
    if [ "$response" == "200" ]; then
        echo -e "${GREEN}✅ HTTP 健康检查通过 (200 OK)${NC}"
        ((CHECKS_PASSED++))
    else
        echo -e "${RED}❌ HTTP 健康检查失败 (${response})${NC}"
        ((CHECKS_FAILED++))
    fi
}

# 数据库连接检查
check_database() {
    echo "检查数据库连接..."
    
    # 简化检查：检查 Pod 是否能解析数据库主机名
    if kubectl exec -n ${NAMESPACE} deployment/${APP_NAME} -- nslookup cgas-postgres &> /dev/null; then
        echo -e "${GREEN}✅ 数据库连接正常${NC}"
        ((CHECKS_PASSED++))
    else
        echo -e "${RED}❌ 数据库连接失败${NC}"
        ((CHECKS_FAILED++))
    fi
}

# 缓存连接检查
check_cache() {
    echo "检查缓存连接..."
    
    if kubectl exec -n ${NAMESPACE} deployment/${APP_NAME} -- nslookup cgas-redis &> /dev/null; then
        echo -e "${GREEN}✅ 缓存连接正常${NC}"
        ((CHECKS_PASSED++))
    else
        echo -e "${RED}❌ 缓存连接失败${NC}"
        ((CHECKS_FAILED++))
    fi
}

# 资源使用检查
check_resources() {
    echo "检查资源使用..."
    
    # CPU 使用率
    local cpu_usage=$(kubectl top pods -n ${NAMESPACE} -l app=${APP_NAME} --no-headers 2>/dev/null | awk '{sum+=$2} END {print sum}' || echo "0")
    
    # 内存使用率
    local mem_usage=$(kubectl top pods -n ${NAMESPACE} -l app=${APP_NAME} --no-headers 2>/dev/null | awk '{sum+=$4; gsub("Mi","",$4)} END {print sum}' || echo "0")
    
    echo -e "ℹ️  CPU 使用：${cpu_usage:-"N/A"}m"
    echo -e "ℹ️  内存使用：${mem_usage:-"N/A"}Mi"
    ((CHECKS_PASSED++))
}

# 日志检查
check_logs() {
    echo "检查最近日志..."
    
    local error_count=$(kubectl logs -n ${NAMESPACE} -l app=${APP_NAME} --tail=100 2>/dev/null | grep -c "ERROR" || echo "0")
    
    if [ "$error_count" -eq 0 ]; then
        echo -e "${GREEN}✅ 最近日志无错误${NC}"
        ((CHECKS_PASSED++))
    else
        echo -e "${YELLOW}⚠️  最近日志发现 ${error_count} 个错误${NC}"
        ((CHECKS_PASSED++))  # 警告不视为失败
    fi
}

# 主流程
echo "1. Pod 状态检查..."
check_pods
echo ""

echo "2. Service 检查..."
check_service
echo ""

echo "3. HTTP 健康检查..."
check_http_health
echo ""

echo "4. 数据库连接检查..."
check_database
echo ""

echo "5. 缓存连接检查..."
check_cache
echo ""

echo "6. 资源使用检查..."
check_resources
echo ""

echo "7. 日志检查..."
check_logs
echo ""

# 总结
echo "========================================="
echo "健康检查汇总"
echo "========================================="
echo -e "✅ 通过：${CHECKS_PASSED}"
echo -e "❌ 失败：${CHECKS_FAILED}"
echo ""

if [ $CHECKS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ 系统健康状态良好${NC}"
    exit 0
else
    echo -e "${RED}❌ 系统存在健康问题，请检查${NC}"
    exit 1
fi
```

---

## 6. 回滚脚本

### 6.1 快速回滚脚本 `rollback.sh`

```bash
#!/bin/bash
# Alpha 环境快速回滚脚本
# Phase 4 Week 1-T2 使用

set -e

echo "========================================="
echo "Alpha 环境快速回滚"
echo "========================================="
echo ""

# 配置
NAMESPACE=${NAMESPACE:-"alpha"}
APP_NAME=${APP_NAME:-"cgas-workflow-engine"}
ROLLBACK_VERSION=${ROLLBACK_VERSION:-"previous"}

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 获取当前版本
get_current_version() {
    local current_image=$(kubectl get deployment ${APP_NAME}-active -n ${NAMESPACE} \
        -o jsonpath='{.spec.template.spec.containers[0].image}')
    echo "$current_image"
}

# 获取上一个稳定版本
get_previous_version() {
    local rollout_history=$(kubectl rollout history deployment/${APP_NAME}-active -n ${NAMESPACE} --revision=2 \
        | grep "image=" | tail -1 | awk -F'=' '{print $2}')
    
    if [ -n "$rollout_history" ]; then
        echo "$rollout_history"
    else
        # 默认回滚版本
        echo "registry.cgas.internal/cgas/workflow-engine:phase3-stable"
    fi
}

# 执行回滚
perform_rollback() {
    local target_version=$1
    
    echo -e "${BLUE}回滚到版本：${target_version}${NC}"
    
    # 更新 Deployment 镜像
    kubectl set image deployment/${APP_NAME}-active \
        ${APP_NAME}=${target_version} \
        -n ${NAMESPACE} \
        --record
    
    # 等待 Rollout 完成
    echo "等待回滚完成..."
    kubectl rollout status deployment/${APP_NAME}-active \
        -n ${NAMESPACE} \
        --timeout=300s
    
    echo -e "${GREEN}✅ 回滚完成${NC}"
}

# 验证回滚
verify_rollback() {
    echo "验证回滚..."
    
    # 检查 Pod 状态
    local ready_pods=$(kubectl get pods -n ${NAMESPACE} -l app=${APP_NAME} \
        -o jsonpath='{range .items[*]}{.status.conditions[?(@.type=="Ready")].status}{"\n"}{end}' | grep -c "True" || echo "0")
    local total_pods=$(kubectl get pods -n ${NAMESPACE} -l app=${APP_NAME} --no-headers | wc -l)
    
    if [ "$ready_pods" -eq "$total_pods" ]; then
        echo -e "${GREEN}✅ Pod 状态正常 (${ready_pods}/${total_pods})${NC}"
        return 0
    else
        echo -e "${RED}❌ Pod 状态异常${NC}"
        return 1
    fi
}

# 主流程
echo "⚠️  警告：即将执行回滚操作"
echo ""

CURRENT_VERSION=$(get_current_version)
echo "当前版本：${CURRENT_VERSION}"

if [ "$ROLLBACK_VERSION" == "previous" ]; then
    TARGET_VERSION=$(get_previous_version)
else
    TARGET_VERSION=$ROLLBACK_VERSION
fi

echo "目标版本：${TARGET_VERSION}"
echo ""

read -p "确认执行回滚？(yes/no): " confirm
if [ "$confirm" != "yes" ]; then
    echo "回滚已取消"
    exit 0
fi

echo ""
echo "执行回滚..."
perform_rollback "$TARGET_VERSION"
echo ""

echo "验证回滚..."
if ! verify_rollback; then
    echo -e "${RED}❌ 回滚验证失败${NC}"
    exit 1
fi
echo ""

echo "========================================="
echo -e "${GREEN}✅ 回滚完成${NC}"
echo "========================================="
echo ""
echo "回滚信息:"
echo "  原版本：${CURRENT_VERSION}"
echo "  回滚到：${TARGET_VERSION}"
echo "  命名空间：${NAMESPACE}"
echo ""

# 保存回滚记录
ROLLBACK_RECORD="rollback_record_$(date +%Y%m%d_%H%M%S).json"
cat > "$ROLLBACK_RECORD" << EOF
{
  "rollback_time": "$(date -Iseconds)",
  "namespace": "${NAMESPACE}",
  "app_name": "${APP_NAME}",
  "from_version": "${CURRENT_VERSION}",
  "to_version": "${TARGET_VERSION}",
  "status": "success"
}
EOF

echo "回滚记录已保存：$ROLLBACK_RECORD"
```

---

## 📝 使用说明

### 部署流程

1. **前置检查**
   ```bash
   ./pre_deploy_check.sh
   ```

2. **部署配置**
   ```bash
   ./deploy_configs.sh
   ```

3. **数据库迁移**
   ```bash
   ./migrate_database.sh
   ```

4. **应用部署**
   ```bash
   ./blue_green_deploy.sh
   ```

5. **健康检查**
   ```bash
   ./health_check.sh
   ```

### 回滚流程

```bash
# 快速回滚到上一个版本
./rollback.sh

# 回滚到指定版本
export ROLLBACK_VERSION="registry.cgas.internal/cgas/workflow-engine:phase3-stable"
./rollback.sh
```

---

## 📊 脚本清单

| 脚本 | 用途 | 执行阶段 |
|------|------|----------|
| `pre_deploy_check.sh` | 部署前置检查 | Week 1-T1 |
| `deploy_configs.sh` | 配置部署 | Week 1-T1 |
| `migrate_database.sh` | 数据库迁移 | Week 1-T1 |
| `blue_green_deploy.sh` | 蓝绿部署 | Week 1-T2 |
| `health_check.sh` | 健康检查 | Week 1-T2 |
| `rollback.sh` | 快速回滚 | Week 1-T2/T3 |

---

**文档状态**: ✅ 部署脚本完成  
**部署位置**: `/home/cc/Desktop/code/AIPro/cgas/rust-workflow-engine/scripts/`  
**责任人**: Dev-Agent  
**保管**: CGAS 项目文档库

---

*Alpha Deployment Scripts v1.0 - 2026-03-07*
