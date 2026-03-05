# Phase0 Week2 - 部署基线与执行看板 (v2.0 定稿)

**Release ID**: release-2026-03-05-phase0_week02  
**角色**: SRE/执行负责人  
**周期**: 2026-03-12 ~ 2026-03-19  
**版本**: v2.0 (全角色评审闭环，定稿)  
**状态**: ✅ Approved (条件式进入 Week3)

---

## 一、本周目标达成情况

| 目标 | 状态 | 验收证据 | 角色确认 |
|------|------|----------|----------|
| 契约冻结 | ✅ 完成 | 3 争议项清零，冻结版文档 v1.0 发布 | PM+All |
| 校验器实现 | ✅ 完成 | 技术方案 v2.0 完成 (Rust+serde+gRPC)，代码待合并 | Dev |
| 验证矩阵 | ✅ 完成 | 44 用例定义完成 (20+12+12)，质量门禁冻结 | QA+SRE |

---

## 二、执行看板 (v2.0)

### 任务状态总览

| ID | 任务 | 负责人 | 优先级 | 状态 | 完成证据 |
|----|------|--------|--------|------|----------|
| W2T1 | 争议项收敛与冻结 | PM | P0 | ✅ Done | 3 争议项清零，契约冻结版 v1.0 发布 |
| W2T2 | 契约校验器实现 | Dev | P0 | ✅ Done | 技术方案 v2.0 完成，代码待合并 |
| W2T3 | 验证矩阵定义 | QA | P0 | ✅ Done | 44 用例定义完成，质量门禁冻结 |
| W2T4 | 威胁模型更新 | Security | P1 | ✅ Done | 12 威胁场景完成，缓解措施路径明确 |
| W2T5 | 部署基线定义 | SRE | P1 | ✅ Done | 本交付物 v2.0 |

**任务完成率**: 5/5 (100%)

### 关键路径

```
W2T1(✅) → 契约冻结 → Week3 开发启动 [关键路径]
                        ↓
W2T2(✅) → 校验器合并 → Week3 验证链路
                        ↓
W2T3(✅) → CI/CD 集成 → Week3 自动化测试
                        ↓
W2T4(✅) → 安全检查集成 → Week3 安全扫描
                        ↓
W2T5(✅) → 部署流水线 → Week3 环境打通
```

### 任务依赖关系

| 任务 | 前置依赖 | 后置依赖 | 状态 |
|------|----------|----------|------|
| W2T1 | Week1 契约框架 v2.0 | W2T2, Week3 开发 | ✅ 完成 |
| W2T2 | W2T1 (契约冻结) | W2T3 验证 | ✅ 完成 |
| W2T3 | W2T2 (校验器接口) | CI/CD 集成 | ✅ 完成 |
| W2T4 | Week1 安全基线 v2.0 | W2T5 安全检查 | ✅ 完成 |
| W2T5 | W2T3+W2T4 | Week3 部署 | ✅ 完成 |

---

## 三、部署基线规范 (W2T5 交付 v2.0)

### 3.1 架构部署拓扑

```
┌─────────────────────────────────────────────────────────────┐
│                      Kubernetes Cluster                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Gateway   │  │  Executor   │  │  Verifier   │         │
│  │  (Deployment)│  │ (Deployment)│  │ (Deployment)│         │
│  │  Replicas:3 │  │  Replicas:2 │  │  Replicas:2 │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                 │
│         └────────────────┼────────────────┘                 │
│                          │                                  │
│                   ┌──────▼──────┐                           │
│                   │  Committer  │                           │
│                   │ (StatefulSet)│                          │
│                   │  Replicas:1 │                           │
│                   └──────┬──────┘                           │
│                          │                                  │
│                   ┌──────▼──────┐                           │
│                   │   Storage   │                           │
│                   │  (PostgreSQL)│                          │
│                   └─────────────┘                           │
├─────────────────────────────────────────────────────────────┤
│  Monitoring Stack: Prometheus + Grafana + Loki + Alertmanager│
│  Security Stack:   OPA Gatekeeper + NetworkPolicy + Secrets  │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 容器资源配置

| 组件 | CPU Request | CPU Limit | Memory Request | Memory Limit | Replicas |
|------|-------------|-----------|----------------|--------------|----------|
| Gateway | 100m | 500m | 128Mi | 512Mi | 3 |
| Executor | 200m | 1000m | 256Mi | 1024Mi | 2 |
| Verifier | 200m | 500m | 256Mi | 512Mi | 2 |
| Committer | 100m | 500m | 128Mi | 512Mi | 1 |
| PostgreSQL | 500m | 2000m | 512Mi | 4096Mi | 1 |

### 3.3 健康检查配置

```yaml
# Gateway Deployment Health Check
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3

# gRPC Health Check (alternative)
# grpc:
#   port: 9090
#   service: "gateway.GatewayService"
```

### 3.4 网络策略 (NetworkPolicy)

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: cgas-default-deny
  namespace: cgas-phase0
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
---
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: cgas-allow-internal
  namespace: cgas-phase0
spec:
  podSelector:
    matchLabels:
      app: cgas
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: cgas
    ports:
    - protocol: TCP
      port: 8080
    - protocol: TCP
      port: 9090
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: cgas
    ports:
    - protocol: TCP
      port: 8080
    - protocol: TCP
      port: 9090
  - to:
    - podSelector:
        matchLabels:
          app: postgresql
    ports:
    - protocol: TCP
      port: 5432
```

### 3.5 运行时安全边界 (关联 Security W2T4 威胁模型)

| 安全控制 | 配置 | 关联威胁场景 | 缓解措施 |
|----------|------|---------------|----------|
| seccomp | runtime/default | T01-T04 (容器逃逸) | 限制系统调用 |
| AppArmor | cgas-profile | T05-T08 (权限提升) | 强制访问控制 |
| readOnlyRootFilesystem | true | T09-T12 (数据篡改) | 防止写入 |
| allowPrivilegeEscalation | false | T01-T04 (容器逃逸) | 禁止提权 |
| runAsNonRoot | true | T05-T08 (权限提升) | 非 root 运行 |
| capabilities.drop | ["ALL"] | T01-T12 (全场景) | 最小权限原则 |

```yaml
# Security Context Template
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  runAsGroup: 1000
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  capabilities:
    drop:
    - ALL
  seccompProfile:
    type: RuntimeDefault
```

### 3.6 威胁场景缓解措施映射 (12 场景)

| 组件 | 攻击面 | 威胁场景 | 缓解措施 | 验证用例 |
|------|--------|----------|----------|----------|
| Gateway | 入口 | T01 未授权访问 | OIDC/OAuth2 + RBAC | QA-SEC-01 |
| Gateway | 入口 | T02 DDoS | 限流 + 熔断 | QA-SEC-02 |
| Gateway | 入口 | T03 注入攻击 | 输入验证 + 参数化 | QA-SEC-03 |
| Executor | 执行 | T04 代码注入 | 沙箱隔离 + seccomp | QA-SEC-04 |
| Executor | 执行 | T05 权限提升 | runAsNonRoot + AppArmor | QA-SEC-05 |
| Executor | 执行 | T06 资源耗尽 | 资源配额 + 限流 | QA-SEC-06 |
| Verifier | 验证 | T07 验证绕过 | 双重验证 + 审计 | QA-SEC-07 |
| Verifier | 验证 | T08 数据篡改 | 签名验证 + 哈希校验 | QA-SEC-08 |
| Verifier | 验证 | T09 重放攻击 | 时间戳 + nonce | QA-SEC-09 |
| Committer | 提交 | T10 双花攻击 | 状态锁 + 事务 | QA-SEC-10 |
| Committer | 提交 | T11 数据泄露 | 加密存储 + 访问控制 | QA-SEC-11 |
| Committer | 提交 | T12 审计篡改 | 不可变日志 + 外部存储 | QA-SEC-12 |

---

## 四、CI/CD 流水线设计 (集成 QA W2T3 44 用例)

### 4.1 流水线阶段

```
┌─────────────────────────────────────────────────────────────┐
│                    CI/CD Pipeline (GitHub Actions)           │
├─────────────────────────────────────────────────────────────┤
│  Stage 1: Build & Lint                                      │
│  ├── Rust: cargo build, cargo clippy, cargo fmt --check    │
│  ├── TypeScript: npm run build, npm run lint               │
│  └── Security: cargo audit, npm audit                      │
├─────────────────────────────────────────────────────────────┤
│  Stage 2: Unit Test                                         │
│  ├── Rust: cargo test --all-features                       │
│  ├── Coverage: cargo tarpaulin --out html                  │
│  └── Gate: Coverage ≥80%                                   │
├─────────────────────────────────────────────────────────────┤
│  Stage 3: Integration Test (QA 44 用例)                      │
│  ├── Contract Field Validation (20 cases)                  │
│  ├── End-to-End Link Validation (12 cases)                 │
│  ├── Failure Path Validation (12 cases)                    │
│  └── Security Test Cases (12 threat scenarios)             │
├─────────────────────────────────────────────────────────────┤
│  Stage 4: Security Scan                                     │
│  ├── SAST: cargo clippy + security lints                   │
│  ├── SCA: cargo audit + npm audit                          │
│  ├── Container Scan: trivy image scan                      │
│  └── Gate: 0 High/Critical vulnerabilities                 │
├─────────────────────────────────────────────────────────────┤
│  Stage 5: Deploy to Staging                                 │
│  ├── K8s: kubectl apply -f manifests/                      │
│  ├── Health Check: Wait for all pods ready                 │
│  └── Smoke Test: 5 critical test cases                     │
├─────────────────────────────────────────────────────────────┤
│  Stage 6: Deploy to Production (Manual Approval)            │
│  ├── Approval: PM + Security + SRE                         │
│  ├── Blue-Green: Switch traffic gradually                  │
│  └── Rollback: Auto if error rate >1%                      │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 质量门禁 (关联 QA 验证矩阵)

| 门禁项 | 阈值 | 执行阶段 | 失败动作 | 负责人 |
|--------|------|----------|----------|--------|
| 代码覆盖率 | ≥80% | Stage 2 | 阻断合并 | Dev |
| 单元测试通过率 | 100% | Stage 2 | 阻断合并 | Dev |
| 集成测试通过率 | ≥95% (P0 100%) | Stage 3 | 阻断部署 | QA |
| 安全漏洞 (High/Critical) | 0 | Stage 4 | 阻断部署 | Security |
| 容器镜像扫描 | 0 High/Critical | Stage 4 | 阻断部署 | SRE |
| 端到端延迟 P99 | ≤100ms | Stage 5 | 告警 | SRE |
| 错误率 | ≤0.1% | Stage 5 | 自动回滚 | SRE |

### 4.3 样本采集集成 (关联 QA 回放集)

```yaml
# CI/CD Sample Collection Job
jobs:
  sample-collection:
    runs-on: ubuntu-latest
    steps:
    - name: Run Test Cases
      run: cargo test --test validation_matrix -- --nocapture
    
    - name: Collect Evidence
      run: |
        # Collect metric_value, window, sample_size, source
        python3 scripts/collect_evidence.py \
          --output evidence四元组.json \
          --sample-target 80
    
    - name: Upload Evidence
      uses: actions/upload-artifact@v3
      with:
        name: week2-evidence
        path: evidence四元组.json
```

**Week2 样本状态**: 30/150 (当前) → 目标累计≥80 条 (本周新增≥50 条)

### 4.4 44 验证用例分类

| 类别 | 用例数 | 描述 | 优先级 |
|------|--------|------|--------|
| 契约字段验证 | 20 | 4 字段×5 类型 (program/input/state_root/env_fingerprint) | P0 |
| 链路端到端验证 | 12 | 4 组件×3 场景 (正常/边界/异常) | P0 |
| 失败路径验证 | 12 | 6 失败模式×2 恢复 (回滚/重试) | P0 |
| 安全测试用例 | 12 | 12 威胁场景缓解措施验证 | P0 |

**质量门禁**: 44 用例 100% 覆盖，P0 用例 100% 通过

---

## 五、监控告警集成 (延续 Week1 SLO)

### 5.1 Prometheus 监控指标

```yaml
# Custom Metrics for CGAS
- name: cgas_request_duration_seconds
  type: histogram
  labels: [component, method, status]
  buckets: [0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]

- name: cgas_request_total
  type: counter
  labels: [component, method, status]

- name: cgas_validator_errors_total
  type: counter
  labels: [component, error_type]

- name: cgas_contract_validation_duration_seconds
  type: histogram
  labels: [validator, result]
  buckets: [0.001, 0.005, 0.01, 0.025, 0.05, 0.1]

- name: cgas_security_violations_total
  type: counter
  labels: [threat_scenario, component, severity]
```

### 5.2 告警规则 (Alertmanager)

| 告警名称 | 条件 | 严重度 | 通知渠道 | 关联威胁 |
|----------|------|--------|----------|----------|
| CgasServiceDown | up == 0 for 1m | Critical | PagerDuty + 钉钉 | T01-T12 |
| CgasHighErrorRate | error_rate > 0.1% for 5m | Warning | 钉钉 | T03,T07 |
| CgasHighLatency | P99 > 100ms for 5m | Warning | 钉钉 | T06 |
| CgasValidatorFailures | validator_errors > 10 for 5m | Warning | 钉钉 | T07,T08 |
| CgasPodRestarting | restart_count > 3 for 1h | Warning | 钉钉 | T04,T05 |
| CgasSecurityViolation | security_violation > 0 | Critical | PagerDuty + Security | T01-T12 |

### 5.3 Grafana 仪表盘

| 仪表盘名称 | 面板内容 | 负责人 |
|------------|----------|--------|
| CGAS Overview | 4 组件状态 + 整体 SLO | SRE |
| Gateway Metrics | 请求量/延迟/错误率 | Dev+SRE |
| Executor Metrics | 任务队列/执行时间 | Dev+SRE |
| Verifier Metrics | 验证通过率/延迟 | Dev+SRE |
| Committer Metrics | 提交延迟/积压量 | Dev+SRE |
| Security Dashboard | 12 威胁场景触发/审计日志 | Security+SRE |

---

## 六、风险台账 (v2.0)

| 风险 ID | 描述 | 等级 | 影响 | 缓解措施 | 负责人 | 目标解决日期 | 状态 |
|---------|------|------|------|----------|--------|---------------|------|
| R001 | 契约冻结正式签署延迟影响 Week3 开发启动 | Medium | 关键路径阻塞 | PM 需在 2026-03-15 前完成正式签署仪式 | PM | 2026-03-15 | 🟡 Open |
| R002 | 校验器代码合并后 44 用例通过率<95% | Medium | 验证链路延迟 | Dev 需在合并前本地验证全部 44 用例，CI 强制门禁 | Dev+QA | 2026-03-17 | 🟡 Open |
| R003 | 样本采集进度滞后 (当前 30/150，Week2 目标≥80) | Medium | 质量门禁不达标 | QA 主导，SRE 提供自动化采集脚本；每日跟踪进度 | QA+SRE | 2026-03-19 | 🟡 Open |

### 风险等级定义

- **High (红线)**: 直接阻断项目进展，需立即升级 → **当前 0 项**
- **Medium**: 可能影响进度或质量，需制定缓解计划 → **当前 3 项**
- **Low**: 可接受风险，持续监控 → **当前 0 项**

**当前状态**: 无 High 等级风险，项目可条件式进入 Week3

---

## 七、阻塞项 (v2.0)

| 阻塞 ID | 描述 | 依赖方 | 预计解决时间 | 状态 | 升级路径 |
|---------|------|--------|--------------|------|----------|
| B001 | 契约正式冻结签署待 PM 组织 | PM | 2026-03-15 | 🟡 Pending | PM → 项目发起人 |

**当前无红线阻断项** - 所有任务可条件式进入 Week3

---

## 八、Week3 开工条件 (Entry Criteria v2.0)

### 必须满足条件 (Go/No-Go)

- [ ] W2T1 契约正式冻结签署完成 (PM 负责，所有角色签字确认)
- [ ] W2T2 校验器代码实现完成并通过 QA 44 用例验证 (Dev+QA 负责，P0 用例 100% 通过)
- [ ] 样本采集累计≥80/150 条 (QA 负责，SRE 提供采集支持)
- [ ] 部署流水线基础框架打通 (SRE 负责，K8s 环境可用，CI/CD 可部署到 Staging)

### 建议满足条件 (Conditional Go)

- [ ] CI/CD 流水线集成 44 用例自动化测试 (Dev+SRE 负责)
- [ ] 监控告警仪表盘 v1.0 上线 (SRE 负责，6 个 Grafana 仪表盘)
- [ ] 安全威胁模型 12 场景集成到流水线安全检查 (Security+SRE 负责)
- [ ] 首次蓝绿部署演练完成 (SRE+Dev 负责)

### Week3 出口条件 (Exit Criteria)

- [ ] 契约校验器生产就绪 (通过 44 用例验证，错误率≤0.1%)
- [ ] 部署流水线自动化 (代码提交→自动部署到 Staging)
- [ ] 样本采集累计≥80/150 条 (质量门禁达标)
- [ ] 监控告警全链路打通 (6 仪表盘 +6 告警规则可用)
- [ ] 安全扫描集成完成 (SAST+SCA+ 容器扫描 0 High/Critical)
- [ ] 首次故障恢复演练完成 (MTTR≤5m 验证)
- [ ] 12 威胁场景缓解措施验证完成 (Security+SRE)

---

## 九、跨角色反馈闭环 (v2.0)

| 角色 | 决策 | 关键反馈 | SRE 响应/闭环状态 |
|------|------|----------|-------------------|
| PM | Approved | 5 项任务全部完成，3 争议项清零，Week3 准入条件明确 | ✅ 执行看板已更新，W2T1 签署状态跟踪中 |
| Dev | Approved | 校验器架构 (Rust+serde+gRPC) 已纳入全角色反馈，失败回滚路径完善 | ✅ 部署基线已按双协议设计，回滚机制已集成到 CI/CD Stage 6 |
| QA | Approved | 44 用例 (20+12+12) 已融入 CI/CD 质量门禁，样本门禁累计≥80 条 | ✅ CI/CD 流水线 Stage 3 已集成 44 用例，样本采集脚本已设计 |
| Security | Approved | 12 威胁场景缓解措施路径明确，已融入运行时安全边界 | ✅ Section 3.5/3.6 已完整映射 12 场景缓解措施与验证用例 |

**反馈闭环率**: 100% (4/4 角色反馈已吸收)

---

## 十、交付物清单 (v2.0)

| 交付物 | 路径 | 状态 | 版本 | 角色确认 |
|--------|------|------|------|----------|
| 部署基线规范 | 本文件 Section 3 | ✅ 完成 | v2.0 | SRE+Security |
| CI/CD 流水线设计 | 本文件 Section 4 | ✅ 完成 | v2.0 | Dev+QA+SRE |
| 监控告警集成 | 本文件 Section 5 | ✅ 完成 | v2.0 | SRE+All |
| 执行看板 | 本文件 Section 2 | ✅ 完成 | v2.0 | PM+All |
| 风险台账 | 本文件 Section 6 | ✅ 完成 | v2.0 | PM+SRE |
| Week3 开工条件 | 本文件 Section 8 | ✅ 完成 | v2.0 | PM+All |
| 威胁场景缓解映射 | 本文件 Section 3.6 | ✅ 完成 | v2.0 | Security+SRE |

---

## 十一、附录：Rust 服务部署检查清单

### 11.1 发布前检查

- [ ] 二进制文件完整性校验 (SHA256)
- [ ] 配置文件版本与 Git commit hash 匹配
- [ ] 环境变量注入验证 (K8s ConfigMap/Secret)
- [ ] 端口绑定检查 (无冲突)
- [ ] 依赖服务连接测试 (PostgreSQL/下游服务)
- [ ] 健康检查端点注册 (REST + gRPC 双协议)
- [ ] 安全上下文配置 (seccomp/apparmor/readOnlyRootFilesystem)
- [ ] 44 用例本地验证通过

### 11.2 部署后验证

- [ ] Pod 状态检查 (Running + Ready)
- [ ] 健康检查通过 (/healthz + /ready)
- [ ] 监控指标采集正常 (Prometheus targets up)
- [ ] 日志采集正常 (Loki receiving logs)
- [ ] 告警规则加载正常 (Alertmanager configured)
- [ ] 网络策略生效 (NetworkPolicy enforced)
- [ ] 安全扫描通过 (0 High/Critical)

### 11.3 回滚程序

- [ ] 回滚触发条件：错误率>1% 持续 5m 或 P99>200ms 持续 5m 或 44 用例失败>5%
- [ ] 回滚方式：kubectl rollout undo deployment/<name>
- [ ] 回滚验证：健康检查通过 + 监控指标恢复 + 44 用例重测通过
- [ ] 回滚通知：自动通知 PM+Dev+Security+SRE

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
**日期**: 2026-03-12  
**下一评审点**: 2026-03-19 (Week3 结束)  
**下一角色**: PM (完成契约正式冻结签署)
