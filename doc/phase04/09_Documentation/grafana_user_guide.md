# Grafana 监控平台使用指南

**版本**: v1.0  
**日期**: 2026-03-07  
**适用环境**: Alpha / Beta / Staging  

---

## 📋 目录

1. [快速入门](#快速入门)
2. [登录 Grafana](#登录-grafana)
3. [查看仪表盘](#查看仪表盘)
4. [常用操作](#常用操作)
5. [告警管理](#告警管理)
6. [故障排查](#故障排查)

---

## 快速入门

### 环境访问地址

| 环境 | URL | 用户名 | 密码 |
|------|-----|--------|------|
| **Alpha** | http://localhost:3001 | admin | AlphaGrafana2026 |
| **Beta** | http://localhost:3002 | admin | BetaGrafana2026 |
| **Staging** | http://localhost:3003 | admin | StagingGrafana2026 |

### 公网访问 (如已配置)

```
http://<您的公网 IP>:3001  # Alpha
http://<您的公网 IP>:3002  # Beta
http://<您的公网 IP>:3003  # Staging
```

---

## 登录 Grafana

### 步骤

1. **打开浏览器** 访问对应环境的 URL
2. **输入用户名**: `admin`
3. **输入密码**: 见上表
4. **点击 "Sign In"**

### 首次登录

首次登录后会提示修改密码，建议：
- ✅ 记录新密码
- ✅ 保存在密码管理器中
- ✅ 分享给团队成员

---

## 查看仪表盘

### Staging 环境 (9 个仪表盘)

| 仪表盘 | 说明 | 访问路径 |
|--------|------|---------|
| **Staging Environment Overview** | 环境总览 | Home → Staging Overview |
| **System Overview** | 系统资源监控 | Home → System Overview |
| **Database Monitoring** | 数据库监控 | Home → Database Monitoring |
| **Redis Monitoring** | Redis 缓存监控 | Home → Redis Monitoring |
| **Nginx Monitoring** | 负载均衡监控 | Home → Nginx Monitoring |
| **Application Metrics** | 应用指标监控 | Home → Application Metrics |
| **Alerts Overview** | 告警总览 | Home → Alerts Overview |
| **Prometheus Monitoring** | Prometheus 自监控 | Home → Prometheus Monitoring |
| **Grafana Monitoring** | Grafana 自监控 | Home → Grafana Monitoring |

### Alpha/Beta 环境 (各 1 个仪表盘)

| 环境 | 仪表盘 | 说明 |
|------|--------|------|
| **Alpha** | Alpha Environment Overview | CPU/内存/Targets |
| **Beta** | Beta Environment Overview | CPU/内存/Targets |

---

## 常用操作

### 1. 切换时间范围

**位置**: 右上角时间选择器

**常用预设**:
- `Last 5 minutes` - 最近 5 分钟
- `Last 15 minutes` - 最近 15 分钟
- `Last 1 hour` - 最近 1 小时
- `Last 6 hours` - 最近 6 小时
- `Last 24 hours` - 最近 24 小时

**自定义**:
- 点击时间选择器
- 选择 `Custom`
- 设置开始和结束时间
- 点击 `Apply`

### 2. 刷新数据

**位置**: 右上角刷新按钮

**刷新频率**:
- `Off` - 不刷新
- `5s` - 每 5 秒刷新
- `10s` - 每 10 秒刷新 (推荐)
- `30s` - 每 30 秒刷新
- `1m` - 每分钟刷新

### 3. 查看指标详情

**操作**:
1. 鼠标悬停在图表上
2. 查看具体数值和时间戳
3. 点击图例可隐藏/显示对应曲线

### 4. 搜索仪表盘

**操作**:
1. 点击左侧菜单 📊 **Dashboards**
2. 在搜索框输入关键词
3. 点击仪表盘名称打开

---

## 告警管理

### 查看告警

**路径**: Alerting → Alert rules

**告警状态**:
- 🟢 **Normal** - 正常
- 🟡 **Pending** - 警告中
- 🔴 **Firing** - 告警触发

### 当前告警规则 (Staging)

| 级别 | 数量 | 说明 |
|------|------|------|
| **P0 (严重)** | 8 条 | 服务宕机、高 CPU、高内存等 |
| **P1 (警告)** | 16 条 | 中等负载、连接数高等 |

### P0 告警列表

| 告警名称 | 触发条件 | 处理建议 |
|---------|---------|---------|
| ServiceDown | 服务宕机 1 分钟 | 检查容器状态 |
| HighCPUUsage | CPU>85% 持续 5 分钟 | 检查负载来源 |
| HighMemoryUsage | 内存>85% 持续 5 分钟 | 检查内存泄漏 |
| DiskSpaceLow | 磁盘<15% 持续 5 分钟 | 清理磁盘空间 |
| PostgresDown | 数据库宕机 1 分钟 | 重启数据库 |
| RedisDown | Redis 宕机 1 分钟 | 重启 Redis |
| HighErrorRate | 错误率>5% 持续 5 分钟 | 检查应用日志 |
| ContainerRestarted | 容器 5 分钟内重启 | 检查容器日志 |

---

## 故障排查

### 问题 1: 仪表盘显示 NoData

**可能原因**:
1. 时间范围不对
2. Prometheus 数据采集中断
3. 数据源配置错误

**解决方法**:
```bash
# 1. 检查 Prometheus targets
curl http://localhost:9093/api/v1/targets | jq '.data.activeTargets[] | {health}'

# 2. 检查是否有数据
curl 'http://localhost:9093/api/v1/query?query=up' | jq '.data.result | length'

# 3. 刷新浏览器 (Ctrl+Shift+R)
```

### 问题 2: 无法登录

**可能原因**:
1. 密码错误
2. 用户被锁定
3. Grafana 服务异常

**解决方法**:
```bash
# 1. 检查 Grafana 状态
docker ps --filter "name=grafana"

# 2. 查看 Grafana 日志
docker logs cgas-staging-grafana --tail 50

# 3. 重启 Grafana
docker restart cgas-staging-grafana
```

### 问题 3: 告警不触发

**可能原因**:
1. 告警规则未加载
2. 阈值设置过高
3. 通知渠道未配置

**解决方法**:
```bash
# 1. 检查告警规则
curl http://localhost:9093/api/v1/rules | jq '.data.groups[0].rules | length'

# 2. 查看告警状态
curl http://localhost:9093/api/v1/alerts | jq '.data.alerts | length'
```

---

## 团队成员管理

### 添加新用户

**路径**: Configuration → Users → Add user

**步骤**:
1. 输入用户名
2. 输入邮箱
3. 设置初始密码
4. 选择角色 (Viewer/Editor/Admin)
5. 点击 Add

### 角色权限

| 角色 | 查看仪表盘 | 编辑仪表盘 | 管理告警 | 管理用户 |
|------|-----------|-----------|---------|---------|
| **Viewer** | ✅ | ❌ | ❌ | ❌ |
| **Editor** | ✅ | ✅ | ❌ | ❌ |
| **Admin** | ✅ | ✅ | ✅ | ✅ |

---

## 最佳实践

### 日常检查

**每天**:
- [ ] 查看 Alerts Overview 仪表盘
- [ ] 检查是否有 P0 告警
- [ ] 查看 CPU/内存使用趋势

**每周**:
- [ ] 检查磁盘使用趋势
- [ ] 审查告警规则有效性
- [ ] 清理过期告警

### 告警响应

**P0 告警**:
1. 立即响应 (<5 分钟)
2. 查看告警详情
3. 定位问题根源
4. 执行修复操作
5. 记录事故报告

**P1 告警**:
1. 尽快响应 (<30 分钟)
2. 评估影响范围
3. 安排时间修复
4. 跟踪修复进度

---

## 快速参考卡

### 常用 URL

```
Alpha Grafana:  http://localhost:3001
Beta Grafana:   http://localhost:3002
Staging Grafana:http://localhost:3003

Alpha Prometheus: http://localhost:9091
Beta Prometheus:  http://localhost:9092
Staging Prometheus: http://localhost:9093
```

### 常用命令

```bash
# 检查容器状态
docker ps --filter "name=cgas-staging"

# 查看 Prometheus targets
curl http://localhost:9093/api/v1/targets | jq '[.data.activeTargets[] | {job, health}]'

# 查看告警规则数量
curl http://localhost:9093/api/v1/rules | jq '.data.groups[0].rules | length'

# 重启 Grafana
docker restart cgas-staging-grafana
```

### 默认密码

```
Alpha:  admin / AlphaGrafana2026
Beta:   admin / BetaGrafana2026
Staging: admin / StagingGrafana2026
```

---

## 联系支持

**问题反馈**:
- 文档问题：提交 Issue
- 系统问题：联系 SRE 团队
- 紧急问题：查看 P0 告警通知

**文档更新**:
- 版本：v1.0
- 最后更新：2026-03-07
- 维护者：SRE Team

---

**祝使用愉快！** 📊
