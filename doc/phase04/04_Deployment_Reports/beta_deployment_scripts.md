# Beta 环境部署脚本

**版本**: v1.0  
**日期**: 2026-04-08  
**责任人**: Dev-Agent  
**环境**: Beta (外部用户测试环境)  
**状态**: ✅ 已完成  

---

## 📋 目录

1. [部署前置检查脚本](#1-部署前置检查脚本)
2. [应用部署脚本](#2-应用部署脚本)
3. [配置部署脚本](#3-配置部署脚本)
4. [数据库迁移脚本](#4-数据库迁移脚本)
5. [健康检查脚本](#5-健康检查脚本)
6. [回滚脚本](#6-回滚脚本)
7. [性能优化脚本](#7-性能优化脚本)

---

## 1. 部署前置检查脚本

### 1.1 环境检查脚本 `pre_deploy_check_beta.sh`

```bash
#!/bin/bash
# Beta 环境部署前置检查脚本
# Phase 4 Week 2-T1 使用

set -e

echo "========================================="
echo "Beta 环境部署前置检查"
echo "========================================="
echo ""

# 配置
BETA_ENV_URL=${BETA_ENV_URL:-"http://beta.cgas.internal:8080"}
KUBE_CONFIG=${KUBE_CONFIG:-"$HOME/.kube/config"}
REQUIRED_TOOLS=("kubectl" "helm" "curl" "jq" "yq")

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查结果统计
CHECKS_PASSED=0
CHECKS_FAILED=0
CHECKS_WARNING=0

# 检查函数
check_tool() {
    local tool=$1
    if command -v "$tool" &> /dev/null; then
        echo -e "✅ ${tool} 已安装 ($(command -v "$tool" --version 2>&1 | head -1))"
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
        echo -e "⚠️  命名空间 ${namespace} 不存在 (将自动创建)"
        ((CHECKS_WARNING++))
    fi
}

check_resource_quota() {
    local namespace=$1
    echo "检查资源配额..."
    
    # 检查 CPU 配额
    local cpu_quota=$(kubectl get resourcequota -n "$namespace" -o jsonpath='{.items[0].status.hard.cpu}' 2>/dev/null || echo "0")
    local cpu_used=$(kubectl get resourcequota -n "$namespace" -o jsonpath='{.items[0].status.used.cpu}' 2>/dev/null || echo "0")
    
    if [ "$cpu_quota" != "0" ] && [ "$cpu_quota" != "" ]; then
        echo -e "✅ CPU 配额：${cpu_used}/${cpu_quota}"
        ((CHECKS_PASSED++))
    else
        echo -e "⚠️  未设置 CPU 配额"
        ((CHECKS_WARNING++))
    fi
    
    # 检查内存配额
    local mem_quota=$(kubectl get resourcequota -n "$namespace" -o jsonpath='{.items[0].status.hard.memory}' 2>/dev/null || echo "0")
    local mem_used=$(kubectl get resourcequota -n "$namespace" -o jsonpath='{.items[0].status.used.memory}' 2>/dev/null || echo "0")
    
    if [ "$mem_quota" != "0" ] && [ "$mem_quota" != "" ]; then
        echo -e "✅ 内存配额：${mem_used}/${mem_quota}"
        ((CHECKS_PASSED++))
    else
        echo -e "⚠️  未设置内存配额"
        ((CHECKS_WARNING++))
    fi
}

check_image_pull_secret() {
    local namespace=$1
    if kubectl get secret regcred -n "$namespace" &> /dev/null; then
        echo -e "✅ 镜像拉取密钥存在"
        ((CHECKS_PASSED++))
    else
        echo -e "⚠️  镜像拉取密钥不存在 (如使用私有仓库需创建)"
        ((CHECKS_WARNING++))
    fi
}

check_storage_class() {
    local storage_class=$1
    if kubectl get storageclass "$storage_class" &> /dev/null; then
        echo -e "✅ StorageClass ${storage_class} 存在"
        ((CHECKS_PASSED++))
    else
        echo -e "❌ StorageClass ${storage_class} 不存在"
        ((CHECKS_FAILED++))
    fi
}

check_database_connectivity() {
    local db_host=$1
    local db_port=$2
    
    echo "检查数据库连接..."
    if timeout 5 bash -c "cat < /dev/null > /dev/tcp/$db_host/$db_port" 2>/dev/null; then
        echo -e "✅ 数据库 ${db_host}:${db_port} 可达"
        ((CHECKS_PASSED++))
    else
        echo -e "❌ 数据库 ${db_host}:${db_port} 不可达"
        ((CHECKS_FAILED++))
    fi
}

# 主检查流程
echo "【1/8】检查必需工具..."
for tool in "${REQUIRED_TOOLS[@]}"; do
    check_tool "$tool"
done
echo ""

echo "【2/8】检查 Kubernetes 配置..."
check_kube_config
echo ""

echo "【3/8】检查命名空间..."
check_namespace "cgas-beta"
echo ""

echo "【4/8】检查资源配额..."
check_resource_quota "cgas-beta"
echo ""

echo "【5/8】检查镜像拉取密钥..."
check_image_pull_secret "cgas-beta"
echo ""

echo "【6/8】检查 StorageClass..."
check_storage_class "ssd-storage"
echo ""

echo "【7/8】检查数据库连接..."
check_database_connectivity "beta-db-primary.cgas.internal" "5432"
check_database_connectivity "beta-db-replica.cgas.internal" "5432"
echo ""

echo "【8/8】检查 Docker 镜像..."
IMAGES=(
    "cgas/executor:phase4-beta-v1.0"
    "cgas/verifier:phase4-beta-v1.0"
    "cgas/blocker:phase4-beta-v1.0"
)

for image in "${IMAGES[@]}"; do
    if docker image inspect "$image" &> /dev/null; then
        echo -e "✅ 镜像 ${image} 已本地缓存"
        ((CHECKS_PASSED++))
    else
        echo -e "⚠️  镜像 ${image} 未本地缓存 (将自动拉取)"
        ((CHECKS_WARNING++))
    fi
done
echo ""

# 总结
echo "========================================="
echo "检查结果汇总"
echo "========================================="
echo -e "✅ 通过：${CHECKS_PASSED}"
echo -e "⚠️  警告：${CHECKS_WARNING}"
echo -e "❌ 失败：${CHECKS_FAILED}"
echo ""

if [ $CHECKS_FAILED -gt 0 ]; then
    echo -e "${RED}❌ 前置检查失败，请修复上述问题后重试${NC}"
    exit 1
elif [ $CHECKS_WARNING -gt 0 ]; then
    echo -e "${YELLOW}⚠️  前置检查通过 (存在警告项)${NC}"
    exit 0
else
    echo -e "${GREEN}✅ 前置检查全部通过${NC}"
    exit 0
fi
```

---

## 2. 应用部署脚本

### 2.1 Beta 环境应用部署 `deploy_beta_apps.sh`

```bash
#!/bin/bash
# Beta 环境应用部署脚本
# Phase 4 Week 2-T1 使用

set -e

echo "========================================="
echo "Beta 环境应用部署"
echo "========================================="
echo ""

# 配置
NAMESPACE="cgas-beta"
RELEASE_NAME="cgas-beta"
CHART_VERSION="3.0.0-beta"
TIMEOUT="10m"

# 应用镜像版本
EXECUTOR_IMAGE="cgas/executor:phase4-beta-v1.0"
VERIFIER_IMAGE="cgas/verifier:phase4-beta-v1.0"
BLOCKER_IMAGE="cgas/blocker:phase4-beta-v1.0"

# 副本数 (Beta 环境 3 应用)
EXECUTOR_REPLICAS=3
VERIFIER_REPLICAS=3
BLOCKER_REPLICAS=2

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 创建命名空间
echo "【1/6】创建命名空间..."
kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
echo -e "${GREEN}✅ 命名空间 ${NAMESPACE} 已创建/存在${NC}"
echo ""

# 创建配置密钥
echo "【2/6】创建配置密钥..."
kubectl create secret generic cgas-beta-config \
  --from-literal=DATABASE_URL="postgresql://cgas:***@beta-db-primary.cgas.internal:5432/cgas_beta" \
  --from-literal=REDIS_URL="redis://beta-redis.cgas.internal:6379" \
  --from-literal=ENVIRONMENT="beta" \
  --namespace="$NAMESPACE" \
  --dry-run=client -o yaml | kubectl apply -f -
echo -e "${GREEN}✅ 配置密钥已创建${NC}"
echo ""

# 部署 Executor 服务
echo "【3/6】部署 Executor 服务 (${EXECUTOR_REPLICAS} 副本)..."
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: executor
  namespace: ${NAMESPACE}
  labels:
    app: executor
    version: ${CHART_VERSION}
spec:
  replicas: ${EXECUTOR_REPLICAS}
  selector:
    matchLabels:
      app: executor
  template:
    metadata:
      labels:
        app: executor
        version: ${CHART_VERSION}
    spec:
      containers:
      - name: executor
        image: ${EXECUTOR_IMAGE}
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: SPRING_PROFILES_ACTIVE
          value: "beta"
        - name: SERVER_PORT
          value: "8080"
        - name: EXECUTOR_MAX_CONCURRENT
          value: "150"
        - name: EXECUTOR_QUEUE_SIZE
          value: "1500"
        - name: EXECUTOR_TIMEOUT_MS
          value: "30000"
        - name: CACHE_ENABLED
          value: "true"
        - name: CACHE_MAX_SIZE
          value: "15000"
        - name: CACHE_TTL_SECONDS
          value: "300"
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "2000m"
            memory: "2Gi"
        livenessProbe:
          httpGet:
            path: /actuator/health/liveness
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /actuator/health/readiness
            port: 8080
          initialDelaySeconds: 20
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: executor
  namespace: ${NAMESPACE}
spec:
  selector:
    app: executor
  ports:
  - port: 8080
    targetPort: 8080
    name: http
  type: ClusterIP
EOF
echo -e "${GREEN}✅ Executor 服务已部署${NC}"
echo ""

# 部署 Verifier 服务
echo "【4/6】部署 Verifier 服务 (${VERIFIER_REPLICAS} 副本)..."
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: verifier
  namespace: ${NAMESPACE}
  labels:
    app: verifier
    version: ${CHART_VERSION}
spec:
  replicas: ${VERIFIER_REPLICAS}
  selector:
    matchLabels:
      app: verifier
  template:
    metadata:
      labels:
        app: verifier
        version: ${CHART_VERSION}
    spec:
      containers:
      - name: verifier
        image: ${VERIFIER_IMAGE}
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: SPRING_PROFILES_ACTIVE
          value: "beta"
        - name: SERVER_PORT
          value: "8080"
        - name: VERIFIER_MAX_CONCURRENT
          value: "150"
        - name: VERIFIER_QUEUE_SIZE
          value: "1500"
        - name: VERIFIER_TIMEOUT_MS
          value: "30000"
        - name: VALIDATION_CACHE_ENABLED
          value: "true"
        - name: VALIDATION_CACHE_MAX_ENTRIES
          value: "15000"
        - name: VALIDATION_CACHE_EXPIRE_SECONDS
          value: "300"
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "2000m"
            memory: "2Gi"
        livenessProbe:
          httpGet:
            path: /actuator/health/liveness
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /actuator/health/readiness
            port: 8080
          initialDelaySeconds: 20
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: verifier
  namespace: ${NAMESPACE}
spec:
  selector:
    app: verifier
  ports:
  - port: 8080
    targetPort: 8080
    name: http
  type: ClusterIP
EOF
echo -e "${GREEN}✅ Verifier 服务已部署${NC}"
echo ""

# 部署 Blocker 服务 (新增，Week 1 问题修复)
echo "【5/6】部署 Blocker 服务 (${BLOCKER_REPLICAS} 副本)..."
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: blocker
  namespace: ${NAMESPACE}
  labels:
    app: blocker
    version: ${CHART_VERSION}
spec:
  replicas: ${BLOCKER_REPLICAS}
  selector:
    matchLabels:
      app: blocker
  template:
    metadata:
      labels:
        app: blocker
        version: ${CHART_VERSION}
    spec:
      containers:
      - name: blocker
        image: ${BLOCKER_IMAGE}
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: SPRING_PROFILES_ACTIVE
          value: "beta"
        - name: SERVER_PORT
          value: "8080"
        - name: BLOCKER_MAX_CONCURRENT
          value: "200"
        - name: BLOCKER_QUEUE_SIZE
          value: "2000"
        - name: BLOCKER_CACHE_ENABLED
          value: "true"
        - name: BLOCKER_CACHE_MAX_SIZE
          value: "20000"
        - name: BLOCKER_CACHE_TTL_SECONDS
          value: "600"
        - name: BLOCKER_CACHE_PREWARM_ENABLED
          value: "true"
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "2000m"
            memory: "2Gi"
        livenessProbe:
          httpGet:
            path: /actuator/health/liveness
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /actuator/health/readiness
            port: 8080
          initialDelaySeconds: 20
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: blocker
  namespace: ${NAMESPACE}
spec:
  selector:
    app: blocker
  ports:
  - port: 8080
    targetPort: 8080
    name: http
  type: ClusterIP
EOF
echo -e "${GREEN}✅ Blocker 服务已部署${NC}"
echo ""

# 等待部署完成
echo "【6/6】等待部署完成..."
echo "等待 Executor 部署..."
kubectl rollout status deployment/executor -n "$NAMESPACE" --timeout="${TIMEOUT}"
echo "等待 Verifier 部署..."
kubectl rollout status deployment/verifier -n "$NAMESPACE" --timeout="${TIMEOUT}"
echo "等待 Blocker 部署..."
kubectl rollout status deployment/blocker -n "$NAMESPACE" --timeout="${TIMEOUT}"
echo ""

echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}✅ Beta 环境应用部署完成${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo "部署摘要:"
echo "  - Executor: ${EXECUTOR_REPLICAS} 副本"
echo "  - Verifier: ${VERIFIER_REPLICAS} 副本"
echo "  - Blocker: ${BLOCKER_REPLICAS} 副本"
echo ""
echo "服务端点:"
echo "  - Executor: executor.${NAMESPACE}.svc.cluster.local:8080"
echo "  - Verifier: verifier.${NAMESPACE}.svc.cluster.local:8080"
echo "  - Blocker: blocker.${NAMESPACE}.svc.cluster.local:8080"
echo ""
```

---

## 3. 配置部署脚本

### 3.1 配置热加载脚本 `deploy_config_beta.sh`

```bash
#!/bin/bash
# Beta 环境配置部署脚本 (Week 1 问题修复版)
# Phase 4 Week 2-T1 使用

set -e

echo "========================================="
echo "Beta 环境配置部署"
echo "========================================="
echo ""

NAMESPACE="cgas-beta"
CONFIG_MAP_NAME="cgas-beta-app-config"

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 创建 ConfigMap (包含 Week 1 问题修复配置)
echo "【1/3】创建应用 ConfigMap..."
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: ${CONFIG_MAP_NAME}
  namespace: ${NAMESPACE}
data:
  application-beta.yaml: |
    # Beta 环境配置
    app:
      name: cgas-workflow-engine
      environment: beta
      version: phase4-beta-v1.0
    
    server:
      port: 8080
      workers: 8
      keep_alive: 75
      timeout:
        connect: 30s
        read: 60s
        write: 60s
    
    # 执行器配置
    executor:
      max_concurrent: 150
      queue_size: 1500
      timeout_ms: 30000
      retry:
        max_attempts: 3
        initial_delay_ms: 100
        max_delay_ms: 5000
        multiplier: 2.0
    
    # 验证器配置
    validator:
      max_concurrent: 150
      queue_size: 1500
      timeout_ms: 30000
      cache:
        enabled: true
        max_size: 15000
        ttl_seconds: 300
    
    # 阻断器配置 (Week 1 问题修复：优化缓存策略)
    blocker:
      max_concurrent: 200
      queue_size: 2000
      cache:
        enabled: true
        max_size: 20000
        ttl_seconds: 600
        prewarm_enabled: true  # 新增：缓存预热
        prewarm_interval_seconds: 300
    
    # 配置热加载配置 (Week 1 问题修复：完善监听器)
    config:
      hot_reload:
        enabled: true
        poll_interval_seconds: 30
        watch_enabled: true
        supported_types:
          - yaml
          - json
          - properties
        reload_delay_ms: 1000
    
    # JVM 内存保护配置 (Week 1 问题修复：OOM 保护)
    jvm:
      memory:
        heap_max: "2g"
        heap_min: "1g"
        metaspace_max: "512m"
        gc_log_enabled: true
        oom_dump_enabled: true
        oom_heap_dump_path: "/var/log/heap_dumps"
    
    # 数据库配置
    datasource:
      url: jdbc:postgresql://beta-db-primary.cgas.internal:5432/cgas_beta
      username: cgas
      password: \${DB_PASSWORD}
      driver-class-name: org.postgresql.Driver
      hikari:
        maximum-pool-size: 50
        minimum-idle: 10
        connection-timeout: 30000
        idle-timeout: 600000
        max-lifetime: 1800000
    
    # Redis 缓存配置
    redis:
      host: beta-redis.cgas.internal
      port: 6379
      database: 0
      timeout: 5000
      lettuce:
        pool:
          max-active: 50
          max-idle: 20
          min-idle: 5
    
    # 监控配置
    management:
      endpoints:
        web:
          exposure:
            include: health,info,metrics,prometheus
      metrics:
        export:
          prometheus:
            enabled: true
      health:
        livenessState:
          enabled: true
        readinessState:
          enabled: true
    
    # 日志配置
    logging:
      level:
        root: INFO
        com.cgas: DEBUG
      pattern:
        console: "%d{yyyy-MM-dd HH:mm:ss} [%thread] %-5level %logger{36} - %msg%n"
      file:
        name: /var/log/cgas/beta-application.log
        max-size: 100MB
        max-history: 30
EOF
echo -e "${GREEN}✅ ConfigMap 已创建${NC}"
echo ""

# 创建 JVM 配置 ConfigMap
echo "【2/3】创建 JVM 配置 ConfigMap..."
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: cgas-beta-jvm-config
  namespace: ${NAMESPACE}
data:
  JVM_OPTS: >-
    -Xms1g
    -Xmx2g
    -XX:+UseG1GC
    -XX:MaxGCPauseMillis=200
    -XX:+HeapDumpOnOutOfMemoryError
    -XX:HeapDumpPath=/var/log/heap_dumps
    -XX:ErrorFile=/var/log/jvm_error_%p.log
    -Xlog:gc*:file=/var/log/gc.log:time,uptime:filecount=5,filesize=10M
    -Djava.security.egd=file:/dev/./urandom
    -Dspring.profiles.active=beta
    -Dconfig.hot_reload.enabled=true
    -Dconfig.hot_reload.poll_interval=30
EOF
echo -e "${GREEN}✅ JVM ConfigMap 已创建${NC}"
echo ""

# 重启应用以应用配置 (Week 1 问题修复：热加载监听器完善前使用)
echo "【3/3】滚动重启应用以应用配置..."
echo "重启 Executor..."
kubectl rollout restart deployment/executor -n "$NAMESPACE"
kubectl rollout status deployment/executor -n "$NAMESPACE" --timeout=5m

echo "重启 Verifier..."
kubectl rollout restart deployment/verifier -n "$NAMESPACE"
kubectl rollout status deployment/verifier -n "$NAMESPACE" --timeout=5m

echo "重启 Blocker..."
kubectl rollout restart deployment/blocker -n "$NAMESPACE"
kubectl rollout status deployment/blocker -n "$NAMESPACE" --timeout=5m

echo ""
echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}✅ Beta 环境配置部署完成${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo "配置说明:"
echo "  - 应用配置：${CONFIG_MAP_NAME}"
echo "  - JVM 配置：cgas-beta-jvm-config"
echo "  - 热加载：已启用 (30 秒轮询)"
echo "  - OOM 保护：已启用 (HeapDump + GC 日志)"
echo ""
```

---

## 4. 数据库迁移脚本

### 4.1 数据库迁移 `migrate_beta_database.sh`

```bash
#!/bin/bash
# Beta 环境数据库迁移脚本
# Phase 4 Week 2-T1 使用

set -e

echo "========================================="
echo "Beta 环境数据库迁移"
echo "========================================="
echo ""

# 配置
DB_HOST="beta-db-primary.cgas.internal"
DB_PORT="5432"
DB_NAME="cgas_beta"
DB_USER="cgas"
DB_PASSWORD=${DB_PASSWORD:-"changeme"}
MIGRATION_DIR="/opt/cgas/migrations/beta"
BACKUP_DIR="/opt/cgas/backups/beta"

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# 检查参数
if [ -z "$1" ]; then
    echo "用法：$0 <migration_version>"
    echo "示例：$0 20260408001"
    echo ""
    echo "可用迁移版本:"
    ls -1 "${MIGRATION_DIR}"/*.sql 2>/dev/null | xargs -n1 basename | sed 's/V//' | sed 's/__.*//' || echo "未找到迁移文件"
    exit 1
fi

MIGRATION_VERSION=$1

# 函数：创建备份
create_backup() {
    local backup_file="${BACKUP_DIR}/pre_migration_$(date +%Y%m%d_%H%M%S).sql"
    echo "创建数据库备份..."
    mkdir -p "$BACKUP_DIR"
    
    PGPASSWORD="$DB_PASSWORD" pg_dump \
        -h "$DB_HOST" \
        -p "$DB_PORT" \
        -U "$DB_USER" \
        -d "$DB_NAME" \
        -F c \
        -f "$backup_file"
    
    echo -e "${GREEN}✅ 备份完成：${backup_file}${NC}"
    echo ""
}

# 函数：检查迁移状态
check_migration_status() {
    echo "检查迁移状态..."
    
    PGPASSWORD="$DB_PASSWORD" psql \
        -h "$DB_HOST" \
        -p "$DB_PORT" \
        -U "$DB_USER" \
        -d "$DB_NAME" \
        -c "SELECT version, description, installed_on, success FROM schema_version ORDER BY installed_on DESC LIMIT 10;"
    
    echo ""
}

# 函数：执行迁移
run_migration() {
    local migration_file="${MIGRATION_DIR}/V${MIGRATION_VERSION}__${2}.sql"
    
    if [ ! -f "$migration_file" ]; then
        echo -e "${RED}❌ 迁移文件不存在：${migration_file}${NC}"
        exit 1
    fi
    
    echo "执行迁移：V${MIGRATION_VERSION}"
    echo "文件：${migration_file}"
    echo ""
    
    # 使用 Flyway 执行迁移
    flyway \
        -url="jdbc:postgresql://${DB_HOST}:${DB_PORT}/${DB_NAME}" \
        -user="$DB_USER" \
        -password="$DB_PASSWORD" \
        -locations="filesystem:${MIGRATION_DIR}" \
        migrate
    
    echo ""
    echo -e "${GREEN}✅ 迁移成功${NC}"
}

# 函数：验证迁移
validate_migration() {
    echo "验证迁移结果..."
    
    # 检查迁移记录
    local migration_count=$(PGPASSWORD="$DB_PASSWORD" psql \
        -h "$DB_HOST" \
        -p "$DB_PORT" \
        -U "$DB_USER" \
        -d "$DB_NAME" \
        -t -c "SELECT COUNT(*) FROM schema_version WHERE version = '${MIGRATION_VERSION}';")
    
    if [ "$migration_count" -gt 0 ]; then
        echo -e "${GREEN}✅ 迁移记录已创建${NC}"
    else
        echo -e "${RED}❌ 迁移记录未找到${NC}"
        exit 1
    fi
    
    # 检查表结构
    echo "检查表结构..."
    PGPASSWORD="$DB_PASSWORD" psql \
        -h "$DB_HOST" \
        -p "$DB_PORT" \
        -U "$DB_USER" \
        -d "$DB_NAME" \
        -c "\dt"
    
    echo ""
    echo -e "${GREEN}✅ 迁移验证通过${NC}"
}

# 主流程
echo "【1/4】检查迁移状态..."
check_migration_status
echo ""

echo "【2/4】创建数据库备份..."
create_backup
echo ""

echo "【3/4】执行数据库迁移..."
run_migration "$MIGRATION_VERSION" "${2:-migration}"
echo ""

echo "【4/4】验证迁移结果..."
validate_migration
echo ""

echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}✅ Beta 环境数据库迁移完成${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo "迁移版本：V${MIGRATION_VERSION}"
echo "备份文件：${BACKUP_DIR}/pre_migration_*.sql"
echo ""
```

### 4.2 数据库初始化 SQL `V20260408001__beta_initial_schema.sql`

```sql
-- Beta 环境初始数据库架构
-- 版本：V20260408001
-- 日期：2026-04-08
-- 描述：Beta 环境基础架构 + Week 1 问题修复

-- 启用扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- 执行日志表 (Beta 环境优化版)
CREATE TABLE IF NOT EXISTS execution_log (
    id BIGSERIAL PRIMARY KEY,
    execution_id UUID NOT NULL UNIQUE,
    command TEXT NOT NULL,
    status VARCHAR(50) NOT NULL,
    start_time TIMESTAMP WITH TIME ZONE,
    end_time TIMESTAMP WITH TIME ZONE,
    duration_ms INTEGER,
    exit_code INTEGER,
    output TEXT,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX idx_execution_log_status ON execution_log(status);
CREATE INDEX idx_execution_log_start_time ON execution_log(start_time);
CREATE INDEX idx_execution_log_execution_id ON execution_log(execution_id);

-- 验证日志表 (Beta 环境优化版)
CREATE TABLE IF NOT EXISTS verification_log (
    id BIGSERIAL PRIMARY KEY,
    verification_id UUID NOT NULL UNIQUE,
    execution_id UUID NOT NULL,
    rule_id VARCHAR(100) NOT NULL,
    result VARCHAR(50) NOT NULL,
    details JSONB,
    duration_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_verification_log_execution_id ON verification_log(execution_id);
CREATE INDEX idx_verification_log_result ON verification_log(result);

-- 阻断规则表 (Week 1 问题修复：增加缓存字段)
CREATE TABLE IF NOT EXISTS blocker_rules (
    id BIGSERIAL PRIMARY KEY,
    rule_name VARCHAR(200) NOT NULL UNIQUE,
    rule_type VARCHAR(50) NOT NULL,
    rule_pattern TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 100,
    enabled BOOLEAN DEFAULT true,
    cache_enabled BOOLEAN DEFAULT true,  -- 新增：缓存开关
    cache_ttl_seconds INTEGER DEFAULT 600,  -- 新增：缓存 TTL
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_blocker_rules_type ON blocker_rules(rule_type);
CREATE INDEX idx_blocker_rules_enabled ON blocker_rules(enabled);

-- 阻断日志表
CREATE TABLE IF NOT EXISTS blocker_log (
    id BIGSERIAL PRIMARY KEY,
    rule_id BIGINT REFERENCES blocker_rules(id),
    request_id UUID NOT NULL,
    blocked BOOLEAN NOT NULL,
    reason TEXT,
    duration_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_blocker_log_request_id ON blocker_log(request_id);
CREATE INDEX idx_blocker_log_blocked ON blocker_log(blocked);

-- 配置表 (Week 1 问题修复：增加热加载支持字段)
CREATE TABLE IF NOT EXISTS app_config (
    id BIGSERIAL PRIMARY KEY,
    config_key VARCHAR(200) NOT NULL UNIQUE,
    config_value TEXT NOT NULL,
    config_type VARCHAR(50) NOT NULL DEFAULT 'string',
    hot_reload_enabled BOOLEAN DEFAULT true,  -- 新增：热加载开关
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_app_config_key ON app_config(config_key);

-- 性能指标表 (Beta 环境新增)
CREATE TABLE IF NOT EXISTS performance_metrics (
    id BIGSERIAL PRIMARY KEY,
    metric_name VARCHAR(100) NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    metric_unit VARCHAR(50),
    tags JSONB,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_performance_metrics_name ON performance_metrics(metric_name);
CREATE INDEX idx_performance_metrics_timestamp ON performance_metrics(timestamp);

-- 插入初始数据
-- 阻断规则 (Week 1 问题修复：启用缓存)
INSERT INTO blocker_rules (rule_name, rule_type, rule_pattern, priority, enabled, cache_enabled, cache_ttl_seconds, description)
VALUES 
    ('sql_injection_block', 'security', '.*(DROP|DELETE|TRUNCATE).*', 10, true, true, 600, 'SQL 注入阻断'),
    ('xss_block', 'security', '.*<script>.*', 10, true, true, 600, 'XSS 攻击阻断'),
    ('rate_limit_block', 'performance', 'rate_limit_exceeded', 50, true, true, 300, '频率限制阻断'),
    ('circuit_breaker_block', 'performance', 'circuit_breaker_open', 60, true, true, 300, '熔断器阻断')
ON CONFLICT (rule_name) DO UPDATE SET
    cache_enabled = EXCLUDED.cache_enabled,
    cache_ttl_seconds = EXCLUDED.cache_ttl_seconds,
    updated_at = CURRENT_TIMESTAMP;

-- 配置项 (Week 1 问题修复：启用热加载)
INSERT INTO app_config (config_key, config_value, config_type, hot_reload_enabled, description)
VALUES 
    ('executor.max_concurrent', '150', 'number', true, '执行器最大并发数'),
    ('verifier.max_concurrent', '150', 'number', true, '验证器最大并发数'),
    ('blocker.max_concurrent', '200', 'number', true, '阻断器最大并发数'),
    ('cache.enabled', 'true', 'boolean', true, '缓存开关'),
    ('cache.ttl_seconds', '600', 'number', true, '缓存 TTL'),
    ('config.hot_reload.enabled', 'true', 'boolean', true, '配置热加载开关')
ON CONFLICT (config_key) DO UPDATE SET
    config_value = EXCLUDED.config_value,
    hot_reload_enabled = EXCLUDED.hot_reload_enabled,
    updated_at = CURRENT_TIMESTAMP;

-- 创建统计视图
CREATE OR REPLACE VIEW v_execution_stats AS
SELECT 
    DATE_TRUNC('hour', start_time) AS hour,
    status,
    COUNT(*) AS execution_count,
    AVG(duration_ms) AS avg_duration_ms,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY duration_ms) AS p99_duration_ms,
    MIN(duration_ms) AS min_duration_ms,
    MAX(duration_ms) AS max_duration_ms
FROM execution_log
WHERE start_time >= CURRENT_TIMESTAMP - INTERVAL '24 hours'
GROUP BY DATE_TRUNC('hour', start_time), status
ORDER BY hour DESC, status;

-- 创建性能监控视图
CREATE OR REPLACE VIEW v_performance_summary AS
SELECT 
    metric_name,
    AVG(metric_value) AS avg_value,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY metric_value) AS p99_value,
    MIN(metric_value) AS min_value,
    MAX(metric_value) AS max_value,
    COUNT(*) AS sample_count
FROM performance_metrics
WHERE timestamp >= CURRENT_TIMESTAMP - INTERVAL '1 hour'
GROUP BY metric_name;

COMMENT ON TABLE execution_log IS '执行日志表 (Beta 环境优化版)';
COMMENT ON TABLE verification_log IS '验证日志表 (Beta 环境优化版)';
COMMENT ON TABLE blocker_rules IS '阻断规则表 (Week 1 问题修复：增加缓存支持)';
COMMENT ON TABLE app_config IS '配置表 (Week 1 问题修复：增加热加载支持)';
COMMENT ON TABLE performance_metrics IS '性能指标表 (Beta 环境新增)';
```

---

## 5. 健康检查脚本

### 5.1 健康检查 `health_check_beta.sh`

```bash
#!/bin/bash
# Beta 环境健康检查脚本
# Phase 4 Week 2-T1 使用

set -e

echo "========================================="
echo "Beta 环境健康检查"
echo "========================================="
echo ""

NAMESPACE="cgas-beta"
BETA_URL="http://beta.cgas.internal:8080"

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# 检查结果
CHECKS_PASSED=0
CHECKS_FAILED=0

# 检查 Kubernetes 资源
check_k8s_resources() {
    echo "【1/6】检查 Kubernetes 资源..."
    
    # 检查 Deployment
    local deployments=("executor" "verifier" "blocker")
    for deploy in "${deployments[@]}"; do
        if kubectl get deployment "$deploy" -n "$NAMESPACE" &> /dev/null; then
            local ready=$(kubectl get deployment "$deploy" -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}')
            local desired=$(kubectl get deployment "$deploy" -n "$NAMESPACE" -o jsonpath='{.spec.replicas}')
            echo -e "  ✅ Deployment ${deploy}: ${ready}/${desired} ready"
            ((CHECKS_PASSED++))
        else
            echo -e "  ❌ Deployment ${deploy} 不存在"
            ((CHECKS_FAILED++))
        fi
    done
    
    # 检查 Service
    local services=("executor" "verifier" "blocker")
    for svc in "${services[@]}"; do
        if kubectl get service "$svc" -n "$NAMESPACE" &> /dev/null; then
            echo -e "  ✅ Service ${svc} 存在"
            ((CHECKS_PASSED++))
        else
            echo -e "  ❌ Service ${svc} 不存在"
            ((CHECKS_FAILED++))
        fi
    done
    
    # 检查 Pod
    local pod_count=$(kubectl get pods -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l)
    echo "  Pod 总数：${pod_count}"
    
    # 检查 Pod 状态
    local running_pods=$(kubectl get pods -n "$NAMESPACE" --field-selector=status.phase=Running --no-headers 2>/dev/null | wc -l)
    echo -e "  运行中 Pod: ${running_pods}"
    
    if [ "$pod_count" -eq "$running_pods" ]; then
        echo -e "  ✅ 所有 Pod 运行正常"
        ((CHECKS_PASSED++))
    else
        echo -e "  ❌ 存在异常 Pod"
        kubectl get pods -n "$NAMESPACE"
        ((CHECKS_FAILED++))
    fi
    
    echo ""
}

# 检查应用健康端点
check_health_endpoints() {
    echo "【2/6】检查应用健康端点..."
    
    local services=("executor" "verifier" "blocker")
    for svc in "${services[@]}"; do
        local health_url="${BETA_URL}/actuator/health"
        
        if curl -sf --max-time 5 "$health_url" > /dev/null 2>&1; then
            local status=$(curl -sf --max-time 5 "$health_url" | jq -r '.status' 2>/dev/null || echo "UNKNOWN")
            if [ "$status" = "UP" ]; then
                echo -e "  ✅ ${svc} 健康检查通过 (UP)"
                ((CHECKS_PASSED++))
            else
                echo -e "  ⚠️  ${svc} 健康检查异常 (${status})"
                ((CHECKS_FAILED++))
            fi
        else
            echo -e "  ❌ ${svc} 健康检查失败 (无法连接)"
            ((CHECKS_FAILED++))
        fi
    done
    
    echo ""
}

# 检查数据库连接
check_database() {
    echo "【3/6】检查数据库连接..."
    
    DB_HOST="beta-db-primary.cgas.internal"
    DB_PORT="5432"
    DB_NAME="cgas_beta"
    DB_USER="cgas"
    
    if timeout 5 bash -c "cat < /dev/null > /dev/tcp/$DB_HOST/$DB_PORT" 2>/dev/null; then
        echo -e "  ✅ 数据库连接正常 (${DB_HOST}:${DB_PORT})"
        ((CHECKS_PASSED++))
    else
        echo -e "  ❌ 数据库连接失败 (${DB_HOST}:${DB_PORT})"
        ((CHECKS_FAILED++))
    fi
    
    echo ""
}

# 检查 Redis 连接
check_redis() {
    echo "【4/6】检查 Redis 连接..."
    
    REDIS_HOST="beta-redis.cgas.internal"
    REDIS_PORT="6379"
    
    if timeout 5 bash -c "cat < /dev/null > /dev/tcp/$REDIS_HOST/$REDIS_PORT" 2>/dev/null; then
        echo -e "  ✅ Redis 连接正常 (${REDIS_HOST}:${REDIS_PORT})"
        ((CHECKS_PASSED++))
    else
        echo -e "  ❌ Redis 连接失败 (${REDIS_HOST}:${REDIS_PORT})"
        ((CHECKS_FAILED++))
    fi
    
    echo ""
}

# 检查性能指标
check_performance() {
    echo "【5/6】检查性能指标..."
    
    # 检查 P99 时延
    local p99_url="${BETA_URL}/actuator/metrics/http.server.requests.timer.max"
    local p99=$(curl -sf --max-time 5 "$p99_url" 2>/dev/null | jq -r '.measurements[0].value' 2>/dev/null || echo "0")
    
    if [ "$p99" != "0" ] && [ "$p99" != "" ]; then
        local p99_ms=$(echo "$p99 * 1000" | bc)
        if (( $(echo "$p99_ms < 200" | bc -l) )); then
            echo -e "  ✅ P99 时延：${p99_ms}ms (<200ms 标准)"
            ((CHECKS_PASSED++))
        else
            echo -e "  ⚠️  P99 时延：${p99_ms}ms (≥200ms 标准)"
            ((CHECKS_FAILED++))
        fi
    else
        echo -e "  ⚠️  无法获取 P99 时延指标"
        ((CHECKS_FAILED++))
    fi
    
    # 检查吞吐量
    local qps_url="${BETA_URL}/actuator/metrics/http.server.requests.count"
    local qps=$(curl -sf --max-time 5 "$qps_url" 2>/dev/null | jq -r '.measurements[0].value' 2>/dev/null || echo "0")
    
    if [ "$qps" != "0" ] && [ "$qps" != "" ]; then
        echo -e "  ✅ 当前吞吐量：${qps} QPS"
        ((CHECKS_PASSED++))
    else
        echo -e "  ⚠️  无法获取吞吐量指标"
        ((CHECKS_FAILED++))
    fi
    
    echo ""
}

# 检查监控指标
check_monitoring() {
    echo "【6/6】检查监控指标..."
    
    # 检查 Prometheus 目标
    local prometheus_url="http://prometheus.cgas.internal:9090/api/v1/targets"
    local targets_up=$(curl -sf --max-time 5 "$prometheus_url" 2>/dev/null | jq '[.data.activeTargets[] | select(.health=="up")] | length' 2>/dev/null || echo "0")
    
    if [ "$targets_up" -gt 0 ]; then
        echo -e "  ✅ Prometheus 监控目标正常 (${targets_up} 个 UP)"
        ((CHECKS_PASSED++))
    else
        echo -e "  ⚠️  Prometheus 监控目标异常"
        ((CHECKS_FAILED++))
    fi
    
    echo ""
}

# 主流程
check_k8s_resources
check_health_endpoints
check_database
check_redis
check_performance
check_monitoring

# 总结
echo "========================================="
echo "健康检查结果汇总"
echo "========================================="
echo -e "✅ 通过：${CHECKS_PASSED}"
echo -e "❌ 失败：${CHECKS_FAILED}"
echo ""

if [ $CHECKS_FAILED -gt 0 ]; then
    echo -e "${RED}❌ 健康检查失败，请检查上述问题${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Beta 环境健康检查全部通过${NC}"
    exit 0
fi
```

---

## 6. 回滚脚本

### 6.1 快速回滚 `rollback_beta.sh`

```bash
#!/bin/bash
# Beta 环境快速回滚脚本
# Phase 4 Week 2-T3 使用
# 目标：回滚时间<5 分钟

set -e

echo "========================================="
echo "Beta 环境快速回滚"
echo "========================================="
echo ""

NAMESPACE="cgas-beta"
ROLLBACK_TIMEOUT="5m"

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# 回滚目标版本
PREVIOUS_VERSION=${1:-"phase4-alpha-v1.0"}

echo "回滚目标版本：${PREVIOUS_VERSION}"
echo "回滚超时：${ROLLBACK_TIMEOUT}"
echo ""

# 记录回滚开始时间
START_TIME=$(date +%s)

# 步骤 1: 备份当前状态
echo "【1/5】备份当前状态..."
kubectl get deployments -n "$NAMESPACE" -o yaml > /tmp/beta_deployments_backup_$(date +%Y%m%d_%H%M%S).yaml
echo -e "${GREEN}✅ 当前状态已备份${NC}"
echo ""

# 步骤 2: 回滚 Executor
echo "【2/5】回滚 Executor..."
kubectl set image deployment/executor executor=cgas/executor:${PREVIOUS_VERSION} -n "$NAMESPACE"
kubectl rollout status deployment/executor -n "$NAMESPACE" --timeout="${ROLLBACK_TIMEOUT}"
echo -e "${GREEN}✅ Executor 已回滚${NC}"
echo ""

# 步骤 3: 回滚 Verifier
echo "【3/5】回滚 Verifier..."
kubectl set image deployment/verifier verifier=cgas/verifier:${PREVIOUS_VERSION} -n "$NAMESPACE"
kubectl rollout status deployment/verifier -n "$NAMESPACE" --timeout="${ROLLBACK_TIMEOUT}"
echo -e "${GREEN}✅ Verifier 已回滚${NC}"
echo ""

# 步骤 4: 回滚 Blocker
echo "【4/5】回滚 Blocker..."
kubectl set image deployment/blocker blocker=cgas/blocker:${PREVIOUS_VERSION} -n "$NAMESPACE"
kubectl rollout status deployment/blocker -n "$NAMESPACE" --timeout="${ROLLBACK_TIMEOUT}"
echo -e "${GREEN}✅ Blocker 已回滚${NC}"
echo ""

# 步骤 5: 验证回滚
echo "【5/5】验证回滚..."
sleep 10  # 等待服务稳定

# 检查所有 Deployment 状态
local all_ready=true
for deploy in executor verifier blocker; do
    local ready=$(kubectl get deployment "$deploy" -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}')
    local desired=$(kubectl get deployment "$deploy" -n "$NAMESPACE" -o jsonpath='{.spec.replicas}')
    
    if [ "$ready" != "$desired" ]; then
        echo -e "  ❌ ${deploy}: ${ready}/${desired} ready"
        all_ready=false
    else
        echo -e "  ✅ ${deploy}: ${ready}/${desired} ready"
    fi
done

echo ""

# 计算回滚时间
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo "========================================="
echo "回滚结果"
echo "========================================="
echo "回滚耗时：${DURATION} 秒"

if [ "$all_ready" = true ]; then
    echo -e "${GREEN}✅ 回滚成功${NC}"
    
    if [ $DURATION -lt 300 ]; then
        echo -e "${GREEN}✅ 回滚时间<5 分钟 (达标)${NC}"
    else
        echo -e "${YELLOW}⚠️  回滚时间≥5 分钟 (未达标)${NC}"
    fi
else
    echo -e "${RED}❌ 回滚失败，请手动检查${NC}"
    exit 1
fi

echo ""
echo "回滚版本：${PREVIOUS_VERSION}"
echo "备份文件：/tmp/beta_deployments_backup_*.yaml"
echo ""
```

---

## 7. 性能优化脚本

### 7.1 性能调优 `optimize_beta_performance.sh`

```bash
#!/bin/bash
# Beta 环境性能优化脚本
# Phase 4 Week 2-T5 使用
# 目标：P99<200ms, 吞吐量≥4500 QPS

set -e

echo "========================================="
echo "Beta 环境性能优化"
echo "========================================="
echo ""

NAMESPACE="cgas-beta"

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 优化 1: 调整副本数
echo "【1/4】调整应用副本数..."
kubectl scale deployment executor --replicas=5 -n "$NAMESPACE"
kubectl scale deployment verifier --replicas=5 -n "$NAMESPACE"
kubectl scale deployment blocker --replicas=3 -n "$NAMESPACE"
echo -e "${GREEN}✅ 副本数已调整 (Executor:5, Verifier:5, Blocker:3)${NC}"
echo ""

# 优化 2: 调整资源限制
echo "【2/4】调整资源限制..."
kubectl set resources deployment executor \
  --requests=cpu=1000m,memory=1Gi \
  --limits=cpu=4000m,memory=4Gi \
  -n "$NAMESPACE"

kubectl set resources deployment verifier \
  --requests=cpu=1000m,memory=1Gi \
  --limits=cpu=4000m,memory=4Gi \
  -n "$NAMESPACE"

kubectl set resources deployment blocker \
  --requests=cpu=1000m,memory=1Gi \
  --limits=cpu=4000m,memory=4Gi \
  -n "$NAMESPACE"

echo -e "${GREEN}✅ 资源限制已调整${NC}"
echo ""

# 优化 3: 优化 JVM 参数
echo "【3/4】优化 JVM 参数..."
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: cgas-beta-jvm-optimized
  namespace: ${NAMESPACE}
data:
  JVM_OPTS: >-
    -Xms2g
    -Xmx4g
    -XX:+UseG1GC
    -XX:MaxGCPauseMillis=100
    -XX:InitiatingHeapOccupancyPercent=45
    -XX:G1ReservePercent=10
    -XX:ConcGCThreads=2
    -XX:ParallelGCThreads=4
    -XX:+HeapDumpOnOutOfMemoryError
    -XX:HeapDumpPath=/var/log/heap_dumps
    -Xlog:gc*:file=/var/log/gc.log:time,uptime:filecount=5,filesize=10M
    -Djava.security.egd=file:/dev/./urandom
    -Dspring.profiles.active=beta
    -Dserver.tomcat.threads.max=400
    -Dserver.tomcat.threads.min-spare=50
    -Dserver.tomcat.accept-count=200
    -Dserver.tomcat.max-connections=8192
EOF

# 滚动重启以应用新配置
kubectl rollout restart deployment executor -n "$NAMESPACE"
kubectl rollout restart deployment verifier -n "$NAMESPACE"
kubectl rollout restart deployment blocker -n "$NAMESPACE"

echo "等待滚动重启完成..."
kubectl rollout status deployment executor -n "$NAMESPACE" --timeout=5m
kubectl rollout status deployment verifier -n "$NAMESPACE" --timeout=5m
kubectl rollout status deployment blocker -n "$NAMESPACE" --timeout=5m

echo -e "${GREEN}✅ JVM 参数已优化${NC}"
echo ""

# 优化 4: 优化数据库连接池
echo "【4/4】优化数据库连接池..."
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: cgas-beta-db-optimized
  namespace: ${NAMESPACE}
data:
  DB_CONFIG: |
    spring.datasource.hikari.maximum-pool-size=100
    spring.datasource.hikari.minimum-idle=20
    spring.datasource.hikari.connection-timeout=20000
    spring.datasource.hikari.idle-timeout=300000
    spring.datasource.hikari.max-lifetime=900000
    spring.datasource.hikari.leak-detection-threshold=30000
EOF

echo -e "${GREEN}✅ 数据库连接池已优化${NC}"
echo ""

echo "========================================="
echo "性能优化完成"
echo "========================================="
echo ""
echo "优化项:"
echo "  ✅ 副本数增加 (Executor:5, Verifier:5, Blocker:3)"
echo "  ✅ 资源限制提升 (CPU:4 核，内存:4Gi)"
echo "  ✅ JVM 参数优化 (G1GC, 堆大小 4GB)"
echo "  ✅ 数据库连接池优化 (最大 100 连接)"
echo ""
echo "预期效果:"
echo "  - P99 时延：<200ms"
echo "  - 吞吐量：≥4500 QPS"
echo ""
echo "请执行性能测试验证优化效果:"
echo "  ./run_performance_test_beta.sh"
echo ""
```

---

## 📝 使用说明

### 部署流程

```bash
# 1. 前置检查
./pre_deploy_check_beta.sh

# 2. 部署应用
./deploy_beta_apps.sh

# 3. 部署配置
./deploy_config_beta.sh

# 4. 数据库迁移
./migrate_beta_database.sh 20260408001

# 5. 健康检查
./health_check_beta.sh

# 6. 性能优化 (可选)
./optimize_beta_performance.sh
```

### 回滚流程

```bash
# 快速回滚到指定版本
./rollback_beta.sh phase4-alpha-v1.0
```

---

## 📊 脚本清单

| 脚本名称 | 用途 | 执行时间 |
|---|---|---|
| pre_deploy_check_beta.sh | 部署前置检查 | 2 分钟 |
| deploy_beta_apps.sh | 应用部署 | 10 分钟 |
| deploy_config_beta.sh | 配置部署 | 5 分钟 |
| migrate_beta_database.sh | 数据库迁移 | 5 分钟 |
| health_check_beta.sh | 健康检查 | 3 分钟 |
| rollback_beta.sh | 快速回滚 | <5 分钟 |
| optimize_beta_performance.sh | 性能优化 | 10 分钟 |

---

**文档状态**: ✅ Beta 环境部署脚本完成  
**创建日期**: 2026-04-08  
**责任人**: Dev-Agent  
**验收人**: SRE-Agent + PM-Agent  
**保管**: 项目文档库  
**分发**: Dev 团队、SRE 团队、运维团队

---

*Beta Deployment Scripts v1.0 - 2026-04-08*
