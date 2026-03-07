# 标准部署流程 v1.0

**版本**: v1.0  
**日期**: 2026-03-07  
**责任人**: SRE-Agent  
**状态**: ✅ 已发布

---

## 📋 部署流程

### 阶段 1: 准备阶段 (T-1 天)

| 步骤 | 任务 | 负责人 | 检查项 |
|------|------|--------|--------|
| 1.1 | 资源申请 | SRE | 服务器/网络/存储就绪 |
| 1.2 | 配置准备 | Dev | Docker Compose/配置文件 |
| 1.3 | 镜像构建 | Dev | 镜像构建成功并推送 |
| 1.4 | 监控配置 | SRE | Prometheus/Grafana 配置 |
| 1.5 | 告警规则 | SRE | 告警规则配置完成 |

### 阶段 2: 部署阶段 (T 天)

| 步骤 | 任务 | 负责人 | 检查项 |
|------|------|--------|--------|
| 2.1 | 容器启动 | SRE | 所有容器启动成功 |
| 2.2 | 健康检查 | SRE | 所有容器 healthy |
| 2.3 | 数据库初始化 | Dev | 数据库连接正常 |
| 2.4 | 应用验证 | QA | 应用功能正常 |
| 2.5 | 监控验证 | SRE | Prometheus targets healthy |

### 阶段 3: 验证阶段 (T+1 天)

| 步骤 | 任务 | 负责人 | 检查项 |
|------|------|--------|--------|
| 3.1 | 等待数据采集 | SRE | 等待 5 分钟 |
| 3.2 | Prometheus 验证 | SRE | curl 验证 targets |
| 3.3 | Grafana 验证 | SRE | 浏览器验证仪表盘 |
| 3.4 | 告警测试 | SRE | 触发测试告警 |
| 3.5 | 报告生成 | SRE | 生成验证报告 |

---

## ✅ 验证检查点

### 必须验证的项目

- [ ] 所有容器运行正常 (docker ps)
- [ ] Prometheus targets healthy (curl 验证)
- [ ] Grafana 数据源连接正常 (API 验证)
- [ ] Grafana 仪表盘有数据 (浏览器验证)
- [ ] 时间范围匹配 (from=now-5m)
- [ ] 数据源 uid 正确
- [ ] 告警规则加载成功
- [ ] 等待 5 分钟数据采集

### 验证命令

```bash
# 1. 容器状态
docker ps --filter "name=cgas-staging" | wc -l

# 2. Prometheus targets
curl http://localhost:9093/api/v1/targets | jq '[.data.activeTargets[] | {health}] | group_by(.health)'

# 3. Grafana 数据源
curl 'http://localhost:3003/api/datasources' -u admin:PASSWORD

# 4. Grafana 仪表盘
curl 'http://localhost:3003/api/search?type=dash-db' -u admin:PASSWORD

# 5. 实际数据验证
curl 'http://localhost:3003/api/datasources/proxy/1/api/v1/query?query=count(up)' -u admin:PASSWORD
```

---

## 📁 文档清单

| 文档 | 位置 | 状态 |
|------|------|------|
| 部署配置 | docker-compose.staging.yml | ✅ |
| Prometheus 配置 | monitoring/prometheus.staging.yml | ✅ |
| Grafana 仪表盘 | grafana/provisioning/dashboards/staging/ | ✅ |
| 告警规则 | prometheus/alerts.yml | ✅ |
| 验证报告 | doc/phase04/04_Deployment_Reports/ | ✅ |

---

## 🚨 常见问题

| 问题 | 根因 | 解决方案 |
|------|------|---------|
| Prometheus targets down | 配置了不存在的 targets | 只配置实际存在的 exporters |
| Grafana 无数据 | 数据源 uid 不匹配 | 使用实际 uid (PBFA97CFB590B2093) |
| 仪表盘 NoData | 时间范围不匹配 | 使用 from=now-5m |
| 浏览器缓存 | 旧配置缓存 | Ctrl+Shift+R 强制刷新 |

---

**流程制定完成时间**: 2026-03-07  
**流程版本**: v1.0  
**下次更新**: 根据实际部署经验更新
