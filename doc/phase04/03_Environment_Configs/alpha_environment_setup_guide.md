# Alpha 环境搭建指南

**版本**: v1.0  
**日期**: 2026-04-01  
**责任人**: SRE-Agent  
**状态**: 📋 待执行  
**环境**: Alpha (内部功能验证)

---

## 📋 执行摘要

本指南用于指导 SRE-Agent 完成 Alpha 环境的资源申请、服务器配置、应用部署和验证测试。

**Alpha 环境目标**:
- 内部功能验证
- 性能基线测量
- Alpha 测试通过率≥95%

**时间线**: 2026-04-01 ~ 2026-04-07 (Week 1)

---

## 🖥️ 资源配置

### 服务器需求

| 角色 | 规格 | 数量 | 用途 |
|------|------|------|------|
| 应用服务器 | 8 核 16GB, 200GB SSD | 2 台 | CGAS 应用服务 |
| 数据库服务器 | 16 核 32GB, 500GB SSD | 1 台 | PostgreSQL 主库 |
| **总计** | - | **3 台** | - |

### 网络配置

| 配置项 | 值 |
|--------|-----|
| VPC | cgas-alpha-vpc |
| 子网 | cgas-alpha-subnet (10.0.1.0/24) |
| 安全组 | cgas-alpha-sg |
| 负载均衡 | cgas-alpha-lb (可选) |

---

## 📝 资源申请流程

### 方式一：云平台控制台 (推荐)

#### 1. 登录云平台

```bash
# 阿里云
aliyun configure
aliyun ecs DescribeInstances

# AWS
aws configure
aws ec2 describe-instances

# 腾讯云
tencentcloud configure
tencentcloud cvm DescribeInstances
```

#### 2. 创建应用服务器 (2 台)

**配置模板**:
```yaml
# alpha_app_server.yaml
instance_type: ecs.g6.large  # 阿里云 8 核 16GB
image: ubuntu_22_04_lts
system_disk:
  size: 200  # GB
  category: cloud_ssd
network:
  vpc_id: cgas-alpha-vpc
  vswitch_id: cgas-alpha-subnet
  security_group_id: cgas-alpha-sg
tags:
  - Key: Environment
    Value: Alpha
  - Key: Project
    Value: CGAS
  - Key: Role
    Value: Application
```

**创建命令** (阿里云示例):
```bash
aliyun ecs RunInstances \
  --ImageId ubuntu_22_04_lts \
  --InstanceType ecs.g6.large \
  --SystemDisk.Size 200 \
  --SystemDisk.Category cloud_ssd \
  --VSwitchId vsw-xxx \
  --SecurityGroupId sg-xxx \
  --Amount 2 \
  --InstanceName "cgas-alpha-app-{001,002}"
```

#### 3. 创建数据库服务器 (1 台)

**配置模板**:
```yaml
# alpha_db_server.yaml
instance_type: ecs.g6.4xlarge  # 阿里云 16 核 32GB
image: ubuntu_22_04_lts
system_disk:
  size: 500  # GB
  category: cloud_ssd
network:
  vpc_id: cgas-alpha-vpc
  vswitch_id: cgas-alpha-subnet
  security_group_id: cgas-alpha-sg
tags:
  - Key: Environment
    Value: Alpha
  - Key: Project
    Value: CGAS
  - Key: Role
    Value: Database
```

**创建命令**:
```bash
aliyun ecs RunInstances \
  --ImageId ubuntu_22_04_lts \
  --InstanceType ecs.g6.4xlarge \
  --SystemDisk.Size 500 \
  --SystemDisk.Category cloud_ssd \
  --VSwitchId vsw-xxx \
  --SecurityGroupId sg-xxx \
  --Amount 1 \
  --InstanceName "cgas-alpha-db-001"
```

---

### 方式二：资源申请工单 (企业流程)

#### 资源申请模板

```markdown
# 资源申请工单

**申请人**: SRE-Agent  
**申请日期**: 2026-04-01  
**项目名称**: CGAS Phase 4  
**环境**: Alpha (内部功能验证)  
**使用时间**: 2026-04-01 ~ 2026-04-07  

## 资源需求

### 应用服务器 (2 台)
- CPU: 8 核
- 内存：16 GB
- 磁盘：200 GB SSD
- 操作系统：Ubuntu 22.04 LTS
- 用途：CGAS 应用服务部署

### 数据库服务器 (1 台)
- CPU: 16 核
- 内存：32 GB
- 磁盘：500 GB SSD
- 操作系统：Ubuntu 22.04 LTS
- 数据库：PostgreSQL 15
- 用途：业务数据存储

## 网络配置
- VPC: cgas-alpha-vpc
- 子网：10.0.1.0/24
- 安全组：cgas-alpha-sg

## 审批流程
- [ ] 技术负责人审批
- [ ] 财务审批
- [ ] 运维执行

## 预计成本
- 应用服务器：¥500/台/月 × 2 = ¥1,000/月
- 数据库服务器：¥2,000/台/月 × 1 = ¥2,000/月
- **总计**: ¥3,000/月 (Alpha 环境)
```

---

## 🔧 环境配置脚本

### 1. 应用服务器初始化脚本

```bash
#!/bin/bash
# alpha_app_init.sh
# 应用服务器初始化脚本

set -e

echo "=== Alpha 应用服务器初始化 ==="

# 1. 系统更新
apt-get update
apt-get upgrade -y

# 2. 安装基础工具
apt-get install -y \
    curl \
    git \
    vim \
    htop \
    net-tools \
    jq \
    wget

# 3. 安装 Docker
curl -fsSL https://get.docker.com | sh
systemctl enable docker
systemctl start docker

# 4. 安装 Docker Compose
curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" \
  -o /usr/local/bin/docker-compose
chmod +x /usr/local/bin/docker-compose

# 5. 安装 Prometheus Node Exporter
useradd --no-create-home --shell /bin/false node_exporter
wget https://github.com/prometheus/node_exporter/releases/latest/download/node_exporter-*.linux-amd64.tar.gz
tar -xvf node_exporter-*.linux-amd64.tar.gz
mv node_exporter-*.linux-amd64/node_exporter /usr/local/bin/
chown node_exporter:node_exporter /usr/local/bin/node_exporter

cat > /etc/systemd/system/node_exporter.service <<EOF
[Unit]
Description=Node Exporter
Wants=network-online.target
After=network-online.target

[Service]
User=node_exporter
Group=node_exporter
Type=simple
ExecStart=/usr/local/bin/node_exporter

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable node_exporter
systemctl start node_exporter

# 6. 配置防火墙
ufw allow 22/tcp    # SSH
ufw allow 8080/tcp  # CGAS API
ufw allow 9100/tcp  # Node Exporter
ufw --force enable

# 7. 创建 CGAS 目录
mkdir -p /opt/cgas/{config,logs,data}
chown -R $USER:$USER /opt/cgas

echo "=== 应用服务器初始化完成 ==="
```

### 2. 数据库服务器初始化脚本

```bash
#!/bin/bash
# alpha_db_init.sh
# 数据库服务器初始化脚本

set -e

echo "=== Alpha 数据库服务器初始化 ==="

# 1. 系统更新
apt-get update
apt-get upgrade -y

# 2. 安装 PostgreSQL 15
wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add -
echo "deb http://apt.postgresql.org/pub/repos/apt/ $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list
apt-get update
apt-get install -y postgresql-15 postgresql-contrib-15

# 3. 配置 PostgreSQL
cat > /etc/postgresql/15/main/postgresql.conf <<EOF
listen_addresses = '*'
port = 5432
max_connections = 200
shared_buffers = 8GB
effective_cache_size = 24GB
work_mem = 64MB
maintenance_work_mem = 2GB
wal_level = replica
max_wal_senders = 3
wal_keep_size = 1GB
logging_collector = on
log_directory = '/var/log/postgresql'
log_filename = 'postgresql-%Y-%m-%d.log'
log_min_duration_statement = 1000
EOF

# 4. 配置访问控制
cat > /etc/postgresql/15/main/pg_hba.conf <<EOF
# TYPE  DATABASE        USER            ADDRESS                 METHOD
local   all             postgres                                peer
local   all             all                                     peer
host    all             all             127.0.0.1/32            scram-sha-256
host    all             all             ::1/128                 scram-sha-256
host    cgas            cgas            10.0.1.0/24             scram-sha-256
EOF

# 5. 创建 CGAS 数据库和用户
sudo -u postgres psql <<EOF
CREATE DATABASE cgas;
CREATE USER cgas WITH PASSWORD 'AlphaEnv2026!Secure';
GRANT ALL PRIVILEGES ON DATABASE cgas TO cgas;
\\c cgas
GRANT ALL ON SCHEMA public TO cgas;
EOF

# 6. 启动 PostgreSQL
systemctl enable postgresql
systemctl start postgresql

# 7. 配置防火墙
ufw allow 22/tcp    # SSH
ufw allow 5432/tcp  # PostgreSQL
ufw --force enable

# 8. 配置监控
apt-get install -y prometheus-exporter-postgres

echo "=== 数据库服务器初始化完成 ==="
```

### 3. CGAS 应用部署脚本

```bash
#!/bin/bash
# alpha_cgas_deploy.sh
# CGAS 应用部署脚本

set -e

echo "=== CGAS Alpha 环境部署 ==="

# 1. 克隆代码
cd /opt/cgas
git clone https://github.com/Thneoly/cgas.git
cd cgas

# 2. 构建 Docker 镜像
docker build -t cgas-executor:alpha -f docker/Executor.Dockerfile .
docker build -t cgas-verifier:alpha -f docker/Verifier.Dockerfile .

# 3. 创建 Docker Compose 配置
cat > /opt/cgas/docker-compose.alpha.yml <<EOF
version: '3.8'

services:
  executor:
    image: cgas-executor:alpha
    container_name: cgas-executor
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://cgas:AlphaEnv2026!Secure@10.0.1.3:5432/cgas
      - ENVIRONMENT=alpha
      - LOG_LEVEL=info
    volumes:
      - /opt/cgas/logs:/app/logs
    restart: unless-stopped

  verifier:
    image: cgas-verifier:alpha
    container_name: cgas-verifier
    ports:
      - "8081:8080"
    environment:
      - DATABASE_URL=postgresql://cgas:AlphaEnv2026!Secure@10.0.1.3:5432/cgas
      - ENVIRONMENT=alpha
      - LOG_LEVEL=info
    volumes:
      - /opt/cgas/logs:/app/logs
    restart: unless-stopped
EOF

# 4. 启动服务
docker-compose -f /opt/cgas/docker-compose.alpha.yml up -d

# 5. 健康检查
echo "等待服务启动..."
sleep 30

curl -f http://localhost:8080/health || exit 1
curl -f http://localhost:8081/health || exit 1

echo "=== CGAS Alpha 环境部署完成 ==="
```

---

## ✅ 环境验证清单

### 服务器验证

| 检查项 | 命令 | 期望结果 |
|--------|------|---------|
| CPU 核心数 | `lscpu \| grep "^CPU(s):"` | 8 (应用) / 16 (数据库) |
| 内存大小 | `free -h \| grep Mem` | 16G (应用) / 32G (数据库) |
| 磁盘空间 | `df -h /` | >150G (应用) / >450G (数据库) |
| 操作系统 | `cat /etc/os-release` | Ubuntu 22.04 |
| Docker 版本 | `docker --version` | 已安装 |
| PostgreSQL 版本 | `psql --version` | 15 (数据库服务器) |

### 网络验证

| 检查项 | 命令 | 期望结果 |
|--------|------|---------|
| SSH 连接 | `ssh user@10.0.1.x` | 成功 |
| 应用端口 | `curl http://10.0.1.x:8080/health` | 200 OK |
| 数据库端口 | `telnet 10.0.1.3 5432` | 连接成功 |
| 监控端口 | `curl http://10.0.1.x:9100/metrics` | 返回指标 |

### 应用验证

| 检查项 | 命令 | 期望结果 |
|--------|------|---------|
| 执行器服务 | `curl http://10.0.1.1:8080/health` | {"status":"healthy"} |
| 验证器服务 | `curl http://10.0.1.2:8081/health` | {"status":"healthy"} |
| 数据库连接 | `psql -h 10.0.1.3 -U cgas -d cgas -c "SELECT 1"` | 1 |
| Docker 容器 | `docker ps` | executor, verifier 运行中 |

---

## 📊 监控配置

### Prometheus 配置

```yaml
# prometheus.alpha.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'alpha-executor'
    static_configs:
      - targets: ['10.0.1.1:9100']
    labels:
      environment: alpha
      role: executor

  - job_name: 'alpha-verifier'
    static_configs:
      - targets: ['10.0.1.2:9100']
    labels:
      environment: alpha
      role: verifier

  - job_name: 'alpha-database'
    static_configs:
      - targets: ['10.0.1.3:9187']
    labels:
      environment: alpha
      role: database
```

### Grafana 仪表盘导入

1. 登录 Grafana (http://10.0.1.1:3000)
2. 导入仪表盘 ID: 1860 (Node Exporter Full)
3. 导入仪表盘 ID: 9628 (PostgreSQL Database)
4. 导入仪表盘 ID: 10280 (Docker Container Dashboard)

---

## 📝 环境交付物

| 交付物 | 路径 | 状态 |
|--------|------|------|
| alpha_environment_config.md | 03_Environment_Configs/ | 📋 待创建 |
| alpha_deployment_report.md | 04_Deployment_Reports/ | 📋 待创建 |
| alpha_monitoring_config.md | 05_Monitoring_Configs/ | 📋 待创建 |
| alpha_validation_report.md | 04_Deployment_Reports/ | 📋 待创建 |

---

## 🚨 故障排查

### 常见问题

| 问题 | 可能原因 | 解决方案 |
|------|---------|---------|
| SSH 连接失败 | 安全组未开放 22 端口 | 检查安全组配置 |
| 应用启动失败 | 数据库连接失败 | 检查 DATABASE_URL 配置 |
| 监控无数据 | Node Exporter 未启动 | `systemctl status node_exporter` |
| 数据库连接失败 | pg_hba.conf 配置错误 | 检查访问控制配置 |

---

**文档状态**: 📋 待执行  
**责任人**: SRE-Agent  
**执行时间**: 2026-04-01 ~ 2026-04-07

---

*Alpha Environment Setup Guide v1.0 - 2026-04-01*
