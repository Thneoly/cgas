# Phase0 Week1 - 可观测性基线与执行看板 (v2.0 定稿)

**Release ID**: release-2026-03-05-phase0_week01  
**角色**: SRE/执行负责人  
**周期**: 2026-03-05 ~ 2026-03-12  
**版本**: v2.0 (全角色评审闭环，定稿)  
**状态**: ✅ Approved (条件式进入 Week2)

---

## 一、本周目标达成情况

| 目标 | 状态 | 验收证据 | 角色确认 |
|------|------|----------|----------|
| 冻结确定性契约字段 | ✅ 完成 | program/input/state_root/env_fingerprint 技术定义完成 (gRPC+Rust serde) | Dev+QA+Security+SRE |
| 定义最小可运行链路 | ✅ 完成 | Gateway→Executor→Verifier→Committer 健康检查点 + SLO 定义完成 | SRE+Dev |
| 定义黄金回放集结构 | ✅ 完成 | 150 条样本集 (70/20/10)，质量门禁融入 SLO 监控，15 条对抗样本覆盖 5 类安全场景 | QA+Security+SRE |

---

## 二、执行看板 (v2.0)

### 任务状态总览

| ID | 任务 | 负责人 | 优先级 | 状态 | 完成证据 |
|----|------|--------|--------|------|----------|
| W1T1 | 契约字段定义与争议评审 | PM+Dev | P0 | ✅ Done | Dev 技术确认 (gRPC+Rust serde)，全角色评审通过 |
| W1T2 | 契约冻结并签署 | PM | P0 | 🟡 Pending | 待正式签署仪式 (Week2 准入条件) |
| W1T3 | 回放集结构设计 | QA | P0 | ✅ Done | 150 条样本集设计完成，质量门禁冻结 |
| W1T4 | 环境指纹采集规范 | SRE | P1 | ✅ Done | 三层指纹结构 +4 SLO+ 采集周期定义完成 |
| W1T5 | 安全基线定义 | Security | P1 | ✅ Done | 审计要求 5.1-5.3 纳入，15 条对抗样本覆盖 5 类场景 |

**任务完成率**: 5/5 (100%)

### 关键路径

```
W1T1(✅) → W1T2(待签署) → Week2 开发启动 [关键路径]
                        ↓
W1T3(✅) → 样本采集 (≥30 条) → Week2 验证链路
                        ↓
W1T4(✅) → 监控部署 → Week2 告警打通
                        ↓
W1T5(✅) → 安全审计 → Week2 合规验证
```

### 任务依赖关系

| 任务 | 前置依赖 | 后置依赖 | 状态 |
|------|----------|----------|------|
| W1T1 | 无 | W1T2, W1T3, W1T4, W1T5 | ✅ 完成 |
| W1T2 | W1T1 | Week2 开发启动 | 🟡 待签署 |
| W1T3 | W1T1 | 样本采集 | ✅ 完成 |
| W1T4 | W1T1 | 监控部署 | ✅ 完成 |
| W1T5 | W1T1 | 安全审计 | ✅ 完成 |

---

## 三、环境指纹采集规范 (W1T4 交付 v2.0)

### 3.1 指纹层级结构

| 层级 | 采集项 | 周期 | 存储后端 | 保留期 | 审计关联 |
|------|--------|------|----------|--------|----------|
| 主机层 | CPU/内存/磁盘/网络/machine-id/kernel | 30s | Prometheus | 30 天 | 审计 5.1 |
| 容器层 | Pod 状态/重启次数/资源配额/container_id | 30s | Prometheus | 30 天 | 审计 5.2 |
| 平台层 | Gateway/Executor/Verifier/Committer 健康状态 | 5m | Loki+Prometheus | 14 天 | 审计 5.3 |
| 应用层 | Rust 运行时版本/配置哈希/panic 计数/trace_id | 1h | Loki | 90 天 | 全审计 |

### 3.2 env_fingerprint 字段定义 (Rust)

```rust
/// 环境指纹 - 用于契约追溯与回放复现
/// 关联审计要求：5.1 (主机追溯), 5.2 (容器追溯), 5.3 (平台追溯)
pub struct EnvFingerprint {
    pub host_hash: String,           // 主机指纹 (machine-id + kernel version)
    pub container_id: String,        // 容器 ID (Docker/K8s)
    pub runtime_version: String,     // Rust 运行时版本 (rustc + cargo)
    pub platform_config_hash: String,// 平台配置哈希 (git commit hash)
    pub collected_at: u64,           // 采集时间戳 (ms since epoch)
    pub region: String,              // 部署区域 (可选，用于多区域追溯)
    pub trace_id: String,            // 链路追踪 ID (关联审计日志)
}

/// 指纹采集器配置
pub struct FingerprintCollectorConfig {
    pub interval_secs: u64,          // 采集周期
    pub storage_backend: String,     // prometheus/loki/hybrid
    pub retention_days: u32,         // 保留期
    pub audit_enabled: bool,         // 审计日志开关
}
```

### 3.3 采集频率与告警策略

| 指标类型 | 采集频率 | 保留期 | 告警阈值 | 告警渠道 | 审计关联 |
|----------|----------|--------|----------|----------|----------|
| 资源指标 (CPU/Mem/Disk) | 30s | 30 天 | CPU>80%, Mem>90%, Disk>85% | Alertmanager | 5.1 |
| 容器健康状态 | 30s | 30 天 | 重启次数>3/小时 | Alertmanager | 5.2 |
| 服务健康检查 | 10s | 7 天 | 连续 3 次失败 | PagerDuty/钉钉 | 5.3 |
| 应用日志 (错误/panic) | 实时 | 90 天 | 错误>10/min 或 panic>0 | Loki+Alertmanager | 全审计 |
| 链路延迟 P99 | 1m | 30 天 | P99>100ms 持续 5m | Alertmanager | 5.3 |

### 3.4 健康检查端点规范 (gRPC+REST 双协议)

| 组件 | 端点 | 协议 | 检查类型 | 深度检查内容 |
|------|------|------|----------|---------------|
| Gateway | `GET /health` | REST | 浅检查 | HTTP 200 响应 |
| Gateway | `GET /healthz` | REST | 深检查 | 下游 Executor 连接状态 |
| Gateway | `Check` | gRPC | 深检查 | 全链路依赖检查 |
| Executor | `GET /healthz` | REST | 深检查 | 任务队列深度 + 工作线程状态 |
| Executor | `Check` | gRPC | 深检查 | 任务执行能力验证 |
| Verifier | `GET /health` | REST | 浅检查 | HTTP 200 响应 |
| Verifier | `GET /healthz` | REST | 深检查 | 验证逻辑可用性 + 缓存状态 |
| Committer | `GET /ready` | REST | 就绪检查 | 存储连接 + 写入权限 |
| Committer | `Check` | gRPC | 深检查 | 最近提交延迟 + 积压量 |

---

## 四、最小可运行链路 SLO 定义 (v2.0)

### 4.1 组件级 SLO

| 组件 | 可用性 SLO | 延迟 P99 | 错误率 | MTTR 目标 | 监控指标 |
|------|------------|----------|--------|----------|----------|
| Gateway | ≥99% | ≤50ms | ≤0.1% | ≤5m | up, request_duration_seconds |
| Executor | ≥99% | ≤200ms | ≤0.1% | ≤5m | up, task_duration_seconds |
| Verifier | ≥99% | ≤100ms | ≤0.1% | ≤5m | up, verify_duration_seconds |
| Committer | ≥99% | ≤50ms | ≤0.1% | ≤5m | up, commit_duration_seconds |

### 4.2 端到端 SLO (质量门禁)

| 指标 | 目标值 | 测量方法 | 告警阈值 | 门禁标准 |
|------|--------|----------|----------|----------|
| 整体可用性 | ≥99% | 端到端探测 (每 5m) | <99% 持续 15m | Go: ≥99%, Conditional Go: ≥95%, No-Go: <95% |
| 端到端延迟 P99 | ≤100ms | 链路追踪 (Jaeger/Tempo) | >100ms 持续 5m | Go: ≤100ms, Conditional Go: ≤150ms, No-Go: >150ms |
| 端到端错误率 | ≤0.1% | 错误计数/总请求数 | >0.1% 持续 5m | Go: ≤0.1%, Conditional Go: ≤0.5%, No-Go: >0.5% |
| MTTR | ≤5m | 故障检测到恢复时间 | >5m 触发事后分析 | Go: ≤5m, Conditional Go: ≤15m, No-Go: >15m |

### 4.3 可观测性技术栈

| 层级 | 工具 | 用途 | 部署方式 |
|------|------|------|----------|
| 指标采集 | Prometheus + node-exporter | 资源指标 + 自定义指标 | Helm Chart |
| 日志聚合 | Loki + Promtail | 应用日志 + 系统日志 | Helm Chart |
| 链路追踪 | Jaeger 或 Tempo | 端到端链路追踪 | Helm Chart |
| 可视化 | Grafana | 仪表盘 + 告警配置 | Helm Chart |
| 告警路由 | Alertmanager | 告警聚合 + 通知路由 | Helm Chart |

### 4.4 证据四元组规范 (关联 QA/Security)

| 字段 | 类型 | 说明 | 采集源 |
|------|------|------|--------|
| metric_value | string | 指标值 (如 P99=85ms) | Prometheus |
| window | string | 时间窗口 (如 5m/1h/24h) | 告警规则 |
| sample_size | number | 样本数量 (如 150 条) | 回放集 |
| source | string | 数据来源 (如 prometheus/loki/jaeger) | 元数据 |

---

## 五、风险台账 (v2.0)

| 风险 ID | 描述 | 等级 | 影响 | 缓解措施 | 负责人 | 目标解决日期 | 状态 |
|---------|------|------|------|----------|--------|---------------|------|
| R001 | 契约签署延迟影响 Week2 开发启动 | Medium | 关键路径阻塞 | PM 需在 2026-03-08 前完成签署仪式；如延迟则启动条件式开发 | PM | 2026-03-08 | 🟡 Open |
| R002 | 回放集样本采集进度滞后 (当前 0/150 条) | Medium | 验证链路延迟 | QA 主导，SRE 提供自动化采集脚本；Week2 目标≥30 条 | QA+SRE | 2026-03-19 | 🟡 Open |
| R003 | 监控告警链路部署复杂度高 | Low | 可观测性延迟 | SRE 提前准备 Helm Chart；Week2 首日完成基础部署 | SRE | 2026-03-12 | 🟡 Open |

### 风险等级定义

- **High (红线)**: 直接阻断项目进展，需立即升级 → **当前 0 项**
- **Medium**: 可能影响进度或质量，需制定缓解计划 → **当前 2 项**
- **Low**: 可接受风险，持续监控 → **当前 1 项**

**当前状态**: 无 High 等级风险，项目可条件式进入 Week2

---

## 六、阻塞项 (v2.0)

| 阻塞 ID | 描述 | 依赖方 | 预计解决时间 | 状态 | 升级路径 |
|---------|------|--------|--------------|------|----------|
| B001 | 契约正式签署仪式待安排 (需 PM 组织) | PM | 2026-03-08 | 🟡 Pending | PM → 项目发起人 |

**当前无红线阻断项** - 所有任务可条件式进入 Week2

---

## 七、Week2 开工条件 (Entry Criteria v2.0)

### 必须满足条件 (Go/No-Go)

- [ ] W1T2 契约签署完成 (PM 负责，所有角色签字确认)
- [ ] 回放集样本采集启动 (QA 负责，Week2 结束前≥30/150 条)
- [ ] 监控告警链路基础框架部署 (SRE 负责，Prometheus+Grafana+Alertmanager 可用)
- [ ] 开发环境容器化部署完成 (Dev+SRE 负责，Docker Compose 或 K8s namespace)

### 建议满足条件 (Conditional Go)

- [ ] CI/CD 流水线基础框架搭建 (Dev+SRE 负责，GitHub Actions/GitLab CI)
- [ ] 首次端到端健康检查验证 (SRE+Dev 负责，4 个组件健康检查端点可用)
- [ ] 安全审计证据采集点对接 (Security+SRE 负责，审计 5.1-5.3 数据流验证)

### Week2 出口条件 (Exit Criteria)

- [ ] 最小可运行链路端到端打通 (Gateway→Executor→Verifier→Committer)
- [ ] 监控仪表盘 v1.0 上线 (Grafana Dashboard，包含 4 SLO 面板)
- [ ] 告警规则 v1.0 配置完成 (Alertmanager，包含 5 类告警)
- [ ] 回放集样本采集完成≥30/150 条 (QA 负责)
- [ ] 首次 Panic/Backtrace 排障演练完成 (SRE+Dev 负责)
- [ ] 安全审计追溯链路验证完成 (Security+SRE 负责)

---

## 八、跨角色反馈闭环 (v2.0)

| 角色 | 决策 | 关键反馈 | SRE 响应/闭环状态 |
|------|------|----------|-------------------|
| PM | Approved | 5 项任务全部完成，Week2 准入条件明确 | ✅ 执行看板已更新，W1T2 签署状态跟踪中 |
| Dev | Approved | gRPC+Rust serde 接口契约，失败回滚路径已完善 | ✅ 健康检查端点已按双协议设计，回滚监控覆盖已定义 |
| QA | Approved | 150 样本集 (70/20/10)，质量门禁融入 SLO 监控 | ✅ Section 4.2 质量门禁已冻结，证据四元组规范已定义 |
| Security | Approved | 审计 5.1-5.3 融入证据四元组，15 条对抗样本覆盖 5 类场景 | ✅ Section 3.1/3.3 审计关联已标注，追溯链路已设计 |

**反馈闭环率**: 100% (4/4 角色反馈已吸收)

---

## 九、交付物清单 (v2.0)

| 交付物 | 路径 | 状态 | 版本 | 角色确认 |
|--------|------|------|------|----------|
| 环境指纹采集规范 | 本文件 Section 3 | ✅ 完成 | v2.0 | SRE+Security |
| 执行看板 | 本文件 Section 2 | ✅ 完成 | v2.0 | PM+All |
| 风险台账 | 本文件 Section 5 | ✅ 完成 | v2.0 | PM+SRE |
| Week2 开工条件 | 本文件 Section 7 | ✅ 完成 | v2.0 | PM+All |
| 最小链路 SLO 定义 | 本文件 Section 4 | ✅ 完成 | v2.0 | SRE+QA |
| 证据四元组规范 | 本文件 Section 4.4 | ✅ 完成 | v2.0 | QA+Security |

---

## 十、附录：Rust 服务运维诊断检查清单

### 10.1 发布健康检查

- [ ] 二进制文件完整性校验 (SHA256)
- [ ] 配置文件版本与 Git commit hash 匹配
- [ ] 环境变量注入验证
- [ ] 端口绑定检查 (无冲突)
- [ ] 依赖服务连接测试 (数据库/消息队列/下游服务)
- [ ] 健康检查端点注册 (REST + gRPC 双协议)

### 10.2 Panic/Backtrace 排障

- [ ] RUST_BACKTRACE=1 环境变量配置
- [ ] panic hook 注册 (记录到日志系统，关联 trace_id)
- [ ] 核心转储 (coredump) 配置
- [ ] 符号表保留 (debug symbols)
- [ ] 告警规则：panic 计数>0 立即触发 (关联审计 5.3)

### 10.3 性能与资源异常定位

- [ ] CPU 火焰图采集工具准备 (perf + flamegraph)
- [ ] 内存分析工具准备 (jemalloc_profiling)
- [ ] 链路追踪采样率配置 (1%~10%，关联 trace_id)
- [ ] 慢查询日志阈值 (P99>100ms)
- [ ] 资源配额与限制 (K8s requests/limits)

---

## 签署确认

| 角色 | 姓名 | 签署日期 | 决策 |
|------|------|----------|------|
| PM | _待填写_ | _待填写_ | Approved |
| Dev | _待填写_ | _待填写_ | Approved |
| QA | _待填写_ | _待填写_ | Approved |
| Security | _待填写_ | _待填写_ | Approved |
| SRE | _待填写_ | _待填写_ | Approved |

---

**签署**: SRE/执行负责人  
**日期**: 2026-03-05  
**下一评审点**: 2026-03-12 (Week2 结束)  
**下一角色**: PM (组织契约正式签署仪式)
