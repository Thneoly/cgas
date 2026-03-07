# Beta 环境配置文档

**版本**: v1.0  
**日期**: 2026-04-08  
**责任人**: SRE-Agent  
**状态**: ✅ 配置完成  
**环境**: Beta (外部用户测试环境)

---

## 📋 执行摘要

本文档详细描述 Beta 环境的资源配置、网络架构、系统配置和运行时环境。Beta 环境是 Phase 4 Week 2 的核心交付物，用于支持外部种子用户测试和集成验证。

**环境用途**:
- 外部用户测试 (种子用户、早期采用者)
- 集成测试与用户验收测试
- 回滚演练验证
- 性能基线验证 (P99<200ms)

**验收标准**:
- Beta 测试通过率≥98%
- 回滚时间<5 分钟
- 5 台服务器部署成功 (3 应用 +2 数据库)
- 35 个监控指标接入完成

---

## 🖥️ 服务器资源配置

### 应用服务器 (3 台)

| 服务器 ID | 主机名 | CPU | 内存 | 磁盘 | 网络 | IP 地址 | 用途 |
|---|---|---|---|---|---|---|---|
| APP-BETA-01 | beta-app-01.cgas.internal | 8 核 | 16 GB | 200 GB SSD | 1 Gbps | 10.1.2.11 | 应用服务 |
| APP-BETA-02 | beta-app-02.cgas.internal | 8 核 | 16 GB | 200 GB SSD | 1 Gbps | 10.1.2.12 | 应用服务 |
| APP-BETA-03 | beta-app-03.cgas.internal | 8 核 | 16 GB | 200 GB SSD | 1 Gbps | 10.1.2.13 | 应用服务 |

**操作系统**: Ubuntu 22.04 LTS  
**内核版本**: 5.15.0-generic  
**部署组件**:
- CGAS 执行器 (Executor)
- CGAS 验证器 (Verifier)
- CGAS 阻断中间件 (Blocking Middleware)
- Prometheus Node Exporter
- Fluentd 日志 Agent

### 数据库服务器 (2 台)

| 服务器 ID | 主机名 | CPU | 内存 | 磁盘 | 网络 | IP 地址 | 用途 |
|---|---|---|---|---|---|---|---|
| DB-BETA-01 | beta-db-primary.cgas.internal | 16 核 | 32 GB | 500 GB SSD | 1 Gbps | 10.1.2.21 | PostgreSQL 主库 |
| DB-BETA-02 | beta-db-replica.cgas.internal | 16 核 | 32 GB | 500 GB SSD | 1 Gbps | 10.1.2.22 | PostgreSQL 从库 |

**操作系统**: Ubuntu 22.04 LTS  
**内核版本**: 5.15.0-generic  
**数据库版本**: PostgreSQL 15.4  
**复制方式**: 流复制 (Streaming Replication)

### 负载均衡器 (2 台，HA)

| 服务器 ID | 主机名 | CPU | 内存 | 磁盘 | 网络 | IP 地址 | 用途 |
|---|---|---|---|---|---|---|---|
| LB-BETA-01 | beta-lb-01.cgas.internal | 4 核 | 8 GB | 100 GB SSD | 1 Gbps | 10.1.2.1 (VIP) | Nginx 主节点 |
| LB-BETA-02 | beta-lb-02.cgas.internal | 4 核 | 8 GB | 100 GB SSD | 1 Gbps | 10.1.2.2 (VIP) | Nginx 备节点 |

**负载均衡软件**: Nginx 1.24  
**高可用方案**: Keepalived + VRRP  
**虚拟 IP (VIP)**: 10.1.2.100

---

## 🌐 网络架构

### 网络拓扑

```
Beta 环境网络拓扑:

                    ┌─────────────────┐
                    │   负载均衡器    │
                    │  (Nginx, HA)    │
                    │  VIP: 10.1.2.100│
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
┌───────▼───────┐   ┌────────▼────────┐   ┌───────▼───────┐
│ 应用服务器 1   │   │  应用服务器 2    │   │  应用服务器 3   │
│ 10.1.2.11     │   │  10.1.2.12      │   │  10.1.2.13    │
│ (8 核 16G)     │   │  (8 核 16G)     │   │  (8 核 16G)    │
└───────┬───────┘   └────────┬────────┘   └───────┬───────┘
        │                    │                    │
        └────────────────────┼────────────────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
     ┌────────▼────────┐          ┌────────▼────────┐
     │  数据库主库      │          │  数据库从库      │
     │  10.1.2.21      │          │  10.1.2.22      │
     │  (16 核 32G)     │          │  (16 核 32G)     │
     └─────────────────┘          └─────────────────┘
```

### 网络配置

| 配置项 | 值 | 说明 |
|---|---|---|
| VPC CIDR | 10.1.0.0/16 | Beta 环境 VPC |
| 子网 CIDR | 10.1.2.0/24 | 应用子网 |
| 网关 | 10.1.2.1 | VPC 网关 |
| DNS | 10.1.0.2, 8.8.8.8 | 内部 DNS + 公共 DNS |
| NTP | ntp.aliyun.com | 时间同步服务器 |

### 安全组规则

#### 应用服务器安全组 (sg-beta-app)

| 方向 | 协议 | 端口 | 源/目标 | 说明 |
|---|---|---|---|---|
| 入站 | TCP | 80 | 10.1.2.0/24 | HTTP (来自 LB) |
| 入站 | TCP | 443 | 10.1.2.0/24 | HTTPS (来自 LB) |
| 入站 | TCP | 22 | 10.1.0.0/16 | SSH (来自 VPC) |
| 入站 | TCP | 9100 | 10.1.0.0/16 | Prometheus Exporter |
| 出站 | ALL | ALL | 0.0.0.0/0 | 允许所有出站 |

#### 数据库服务器安全组 (sg-beta-db)

| 方向 | 协议 | 端口 | 源/目标 | 说明 |
|---|---|---|---|---|
| 入站 | TCP | 5432 | 10.1.2.0/24 | PostgreSQL (来自应用) |
| 入站 | TCP | 22 | 10.1.0.0/16 | SSH (来自 VPC) |
| 入站 | TCP | 9100 | 10.1.0.0/16 | Prometheus Exporter |
| 入站 | TCP | 5432 | 10.1.2.21/32 | 主从复制 (DB-01→DB-02) |
| 出站 | ALL | ALL | 0.0.0.0/0 | 允许所有出站 |

#### 负载均衡器安全组 (sg-beta-lb)

| 方向 | 协议 | 端口 | 源/目标 | 说明 |
|---|---|---|---|---|
| 入站 | TCP | 80 | 0.0.0.0/0 | HTTP (公网) |
| 入站 | TCP | 443 | 0.0.0.0/0 | HTTPS (公网) |
| 入站 | TCP | 22 | 10.1.0.0/16 | SSH (来自 VPC) |
| 入站 | TCP | 1936 | 10.1.0.0/16 | Nginx Stats |
| 入站 | VRRP | 112 | 10.1.2.0/24 | Keepalived VRRP |
| 出站 | ALL | ALL | 0.0.0.0/0 | 允许所有出站 |

---

## ⚙️ 系统配置

### 操作系统配置 (所有服务器)

#### 内核参数优化 (/etc/sysctl.conf)

```bash
# 网络优化
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65535
net.ipv4.tcp_max_syn_backlog = 8192
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_keepalive_time = 1200
net.ipv4.ip_local_port_range = 1024 65000

# 文件描述符
fs.file-max = 2097152
fs.nr_open = 2097152

# 内存优化
vm.swappiness = 10
vm.overcommit_memory = 1
vm.overcommit_ratio = 90
```

#### 用户限制 (/etc/security/limits.conf)

```bash
* soft nofile 65535
* hard nofile 65535
* soft nproc 65535
* hard nproc 65535
root soft nofile 65535
root hard nofile 65535
```

#### 系统服务配置

```bash
# 禁用不必要的服务
systemctl disable bluetooth
systemctl disable cups
systemctl disable ModemManager

# 启用必要服务
systemctl enable chrony
systemctl enable prometheus-node-exporter
systemctl enable fluentd
```

### 应用服务器配置

#### 运行时环境

| 组件 | 版本 | 配置 |
|---|---|---|
| Node.js | v20.10.0 | LTS 版本 |
| PM2 | v5.3.0 | 进程管理器 |
| Nginx (本地) | 1.24.0 | 反向代理 |

#### 应用部署目录

```
/opt/cgas/
├── executor/          # CGAS 执行器
│   ├── bin/
│   ├── config/
│   ├── logs/
│   └── data/
├── verifier/          # CGAS 验证器
│   ├── bin/
│   ├── config/
│   ├── logs/
│   └── data/
├── middleware/        # 阻断中间件
│   ├── bin/
│   ├── config/
│   └── logs/
└── scripts/           # 运维脚本
    ├── deploy.sh
    ├── rollback.sh
    └── health_check.sh
```

#### PM2 配置文件 (/opt/cgas/executor/ecosystem.config.js)

```javascript
module.exports = {
  apps: [{
    name: 'cgas-executor',
    script: '/opt/cgas/executor/bin/server.js',
    instances: 4,
    exec_mode: 'cluster',
    env: {
      NODE_ENV: 'beta',
      PORT: 3000,
      DB_HOST: 'beta-db-primary.cgas.internal',
      DB_PORT: 5432,
      DB_NAME: 'cgas_beta',
      DB_USER: 'cgas_app',
      REDIS_HOST: 'beta-cache-01.cgas.internal',
      REDIS_PORT: 6379,
      LOG_LEVEL: 'info'
    }
  }]
};
```

### 数据库服务器配置

#### PostgreSQL 配置 (/etc/postgresql/15/main/postgresql.conf)

```ini
# 连接设置
listen_addresses = '*'
port = 5432
max_connections = 500

# 内存设置
shared_buffers = 8GB
effective_cache_size = 24GB
work_mem = 64MB
maintenance_work_mem = 1GB

# WAL 设置
wal_level = replica
max_wal_senders = 10
wal_keep_size = 1GB

# 复制设置
hot_standby = on
hot_standby_feedback = on

# 日志设置
log_destination = 'stderr'
logging_collector = on
log_directory = 'log'
log_filename = 'postgresql-%Y-%m-%d.log'
log_rotation_age = 1d
log_rotation_size = 100MB
log_min_duration_statement = 1000
log_checkpoints = on
log_connections = on
log_disconnections = on
log_lock_waits = on

# 性能优化
random_page_cost = 1.1
effective_io_concurrency = 200
```

#### PostgreSQL 主从复制配置

**主库 (DB-BETA-01) pg_hba.conf**:
```
# 允许从库复制连接
host replication replicator 10.1.2.22/32 scram-sha-256
host all all 10.1.2.0/24 scram-sha-256
```

**从库 (DB-BETA-02) standby.signal**:
```
# 创建 standby.signal 文件启用热备
primary_conninfo = 'host=10.1.2.21 port=5432 user=replicator password=xxx'
primary_slot_name = 'beta_replica_slot'
```

#### 数据库初始化

```sql
-- 创建数据库
CREATE DATABASE cgas_beta;

-- 创建应用用户
CREATE USER cgas_app WITH PASSWORD 'xxx';
GRANT CONNECT ON DATABASE cgas_beta TO cgas_app;
GRANT USAGE ON SCHEMA public TO cgas_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO cgas_app;

-- 创建复制用户
CREATE USER replicator WITH REPLICATION PASSWORD 'xxx';

-- 创建复制槽
SELECT pg_create_physical_replication_slot('beta_replica_slot');
```

---

## 🔧 负载均衡器配置

### Nginx 配置 (/etc/nginx/nginx.conf)

```nginx
user www-data;
worker_processes auto;
worker_rlimit_nofile 65535;
pid /run/nginx.pid;

events {
    worker_connections 65535;
    use epoll;
    multi_accept on;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # 日志格式
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for" '
                    'rt=$request_time uct="$upstream_connect_time" '
                    'uht="$upstream_header_time" urt="$upstream_response_time"';

    access_log /var/log/nginx/access.log main;
    error_log /var/log/nginx/error.log warn;

    # 性能优化
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    client_max_body_size 100M;

    # Gzip 压缩
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml application/json application/javascript application/xml;

    # 上游服务器 (应用服务器池)
    upstream cgas_beta_app {
        least_conn;
        server 10.1.2.11:3000 weight=1 max_fails=3 fail_timeout=30s;
        server 10.1.2.12:3000 weight=1 max_fails=3 fail_timeout=30s;
        server 10.1.2.13:3000 weight=1 max_fails=3 fail_timeout=30s;
        keepalive 32;
    }

    server {
        listen 80;
        server_name beta.cgas.internal;

        location / {
            proxy_pass http://cgas_beta_app;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection 'upgrade';
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_cache_bypass $http_upgrade;
            proxy_connect_timeout 60s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
        }

        location /health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }

        location /nginx_status {
            stub_status on;
            access_log off;
            allow 10.1.0.0/16;
            deny all;
        }
    }
}
```

### Keepalived 配置 (/etc/keepalived/keepalived.conf)

**主节点 (LB-BETA-01)**:
```
vrrp_script check_nginx {
    script "/usr/local/bin/check_nginx.sh"
    interval 2
    fall 3
    rise 2
}

vrrp_instance VI_BETA {
    state MASTER
    interface eth0
    virtual_router_id 51
    priority 100
    advert_int 1

    authentication {
        auth_type PASS
        auth_pass cgas2026
    }

    virtual_ipaddress {
        10.1.2.100/24
    }

    track_script {
        check_nginx
    }
}
```

**备节点 (LB-BETA-02)**:
```
vrrp_instance VI_BETA {
    state BACKUP
    interface eth0
    virtual_router_id 51
    priority 90
    advert_int 1

    authentication {
        auth_type PASS
        auth_pass cgas2026
    }

    virtual_ipaddress {
        10.1.2.100/24
    }
}
```

---

## 📊 监控配置

### 监控指标 (35 个)

#### 系统指标 (10 个)

| 指标 ID | 指标名称 | 类型 | 告警阈值 | 说明 |
|---|---|---|---|---|
| SYS-01 | cpu_usage | Gauge | >80% | CPU 使用率 |
| SYS-02 | memory_usage | Gauge | >85% | 内存使用率 |
| SYS-03 | disk_usage | Gauge | >80% | 磁盘使用率 |
| SYS-04 | disk_io_read | Counter | - | 磁盘读取 IO |
| SYS-05 | disk_io_write | Counter | - | 磁盘写入 IO |
| SYS-06 | network_in | Counter | - | 网络流入 |
| SYS-07 | network_out | Counter | - | 网络流出 |
| SYS-08 | load_average_1m | Gauge | >8 | 1 分钟负载 |
| SYS-09 | load_average_5m | Gauge | >6 | 5 分钟负载 |
| SYS-10 | open_file_descriptors | Gauge | >50000 | 打开文件数 |

#### 应用指标 (15 个)

| 指标 ID | 指标名称 | 类型 | 告警阈值 | 说明 |
|---|---|---|---|---|
| APP-01 | http_requests_total | Counter | - | HTTP 请求总数 |
| APP-02 | http_request_duration_seconds | Histogram | P99>0.2s | 请求时延 |
| APP-03 | http_request_size_bytes | Histogram | - | 请求大小 |
| APP-04 | http_response_size_bytes | Histogram | - | 响应大小 |
| APP-05 | active_connections | Gauge | >1000 | 活跃连接数 |
| APP-06 | executor_queue_size | Gauge | >100 | 执行队列大小 |
| APP-07 | executor_tasks_total | Counter | - | 执行任务总数 |
| APP-08 | executor_tasks_success | Counter | - | 成功任务数 |
| APP-09 | executor_tasks_failed | Counter | >1% | 失败任务数 |
| APP-10 | verifier_checks_total | Counter | - | 验证检查总数 |
| APP-11 | verifier_checks_passed | Counter | - | 通过检查数 |
| APP-12 | verifier_checks_failed | Counter | >1% | 失败检查数 |
| APP-13 | middleware_blocks_total | Counter | - | 阻断总数 |
| APP-14 | error_rate | Gauge | >1% | 错误率 |
| APP-15 | uptime_seconds | Counter | - | 运行时间 |

#### 数据库指标 (10 个)

| 指标 ID | 指标名称 | 类型 | 告警阈值 | 说明 |
|---|---|---|---|---|
| DB-01 | db_connections_active | Gauge | >400 | 活跃连接数 |
| DB-02 | db_connections_idle | Gauge | - | 空闲连接数 |
| DB-03 | db_transactions_total | Counter | - | 事务总数 |
| DB-04 | db_transactions_rolled_back | Counter | >1% | 回滚事务数 |
| DB-05 | db_query_duration_seconds | Histogram | P99>0.5s | 查询时延 |
| DB-06 | db_rows_returned | Counter | - | 返回行数 |
| DB-07 | db_rows_inserted | Counter | - | 插入行数 |
| DB-08 | db_rows_updated | Counter | - | 更新行数 |
| DB-09 | db_replication_lag_seconds | Gauge | >10s | 复制延迟 |
| DB-10 | db_cache_hit_ratio | Gauge | <90% | 缓存命中率 |

### Prometheus 配置 (/etc/prometheus/prometheus.yml)

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - beta-alertmanager.cgas.internal:9093

rule_files:
  - /etc/prometheus/rules/beta_alerts.yml

scrape_configs:
  - job_name: 'beta-node'
    static_configs:
      - targets: ['10.1.2.11:9100', '10.1.2.12:9100', '10.1.2.13:9100']
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
        regex: '(.*):.*'
        replacement: '${1}'

  - job_name: 'beta-app'
    static_configs:
      - targets: ['10.1.2.11:3000', '10.1.2.12:3000', '10.1.2.13:3000']
    metrics_path: '/metrics'

  - job_name: 'beta-db'
    static_configs:
      - targets: ['10.1.2.21:9187', '10.1.2.22:9187']

  - job_name: 'beta-nginx'
    static_configs:
      - targets: ['10.1.2.1:9113', '10.1.2.2:9113']
```

---

## 🔐 安全配置

### SSH 安全配置

```bash
# /etc/ssh/sshd_config
PermitRootLogin prohibit-password
PasswordAuthentication no
PubkeyAuthentication yes
MaxAuthTries 3
ClientAliveInterval 300
ClientAliveCountMax 2
AllowUsers cgas-admin cgas-deploy
```

### 防火墙配置 (UFW)

```bash
# 应用服务器
ufw default deny incoming
ufw default allow outgoing
ufw allow from 10.1.2.0/24 to any port 80
ufw allow from 10.1.2.0/24 to any port 443
ufw allow from 10.1.0.0/16 to any port 22
ufw enable

# 数据库服务器
ufw default deny incoming
ufw default allow outgoing
ufw allow from 10.1.2.0/24 to any port 5432
ufw allow from 10.1.0.0/16 to any port 22
ufw enable
```

---

## 📝 变更历史

| 版本 | 日期 | 变更内容 | 责任人 |
|---|---|---|---|
| v1.0 | 2026-04-08 | 初始版本，Beta 环境配置完成 | SRE-Agent |

---

**文档状态**: ✅ 配置完成  
**配置日期**: 2026-04-08  
**验收日期**: 2026-04-08  
**责任人**: SRE-Agent  
**保管**: 项目文档库  
**分发**: 全体 Agent 团队、运维团队

---

*Beta Environment Configuration v1.0 - 2026-04-08*
