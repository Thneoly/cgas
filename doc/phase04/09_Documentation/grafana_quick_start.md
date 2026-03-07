# Grafana 快速入门指南

**5 分钟上手 Grafana 监控**

---

## 🚀 第一步：登录

### 1. 打开浏览器

访问 Staging 环境：
```
http://localhost:3003
```

### 2. 输入凭证

```
用户名：admin
密码：StagingGrafana2026
```

### 3. 点击 Sign In

---

## 📊 第二步：查看仪表盘

### 默认仪表盘

登录后会自动显示 **Staging Environment Overview**

你会看到 6 个面板：
1. **Prometheus Targets** - 监控目标状态
2. **CPU 使用率** - 系统 CPU 使用
3. **内存使用率** - 系统内存使用
4. **CPU 使用率 (按实例)** - 各节点 CPU
5. **内存使用量 (按实例)** - 各节点内存
6. **所有 Targets 健康状态** - 监控目标健康度

---

## 🔍 第三步：基本操作

### 切换时间范围

点击右上角时间选择器 → 选择 `Last 1 hour`

### 刷新数据

点击时间范围旁边的刷新按钮 → 选择 `10s`

### 查看其他仪表盘

点击左侧菜单 📊 **Dashboards** → 选择任意仪表盘

---

## 🎯 第四步：查看告警

### 查看当前告警

点击左侧菜单 🔔 **Alerting** → **Alert rules**

### 告警状态说明

- 🟢 **Normal** - 正常
- 🟡 **Pending** - 警告中 (即将触发)
- 🔴 **Firing** - 已触发 (需要处理)

---

## ✅ 完成！

现在你已经学会了：
- ✅ 登录 Grafana
- ✅ 查看仪表盘
- ✅ 切换时间范围
- ✅ 刷新数据
- ✅ 查看告警

---

## 📱 常用访问地址

| 环境 | URL | 用途 |
|------|-----|------|
| Staging | http://localhost:3003 | 完整监控 (9 个仪表盘) |
| Alpha | http://localhost:3001 | Alpha 环境监控 |
| Beta | http://localhost:3002 | Beta 环境监控 |

---

## 🆘 需要帮助？

查看完整文档：`grafana_user_guide.md`
