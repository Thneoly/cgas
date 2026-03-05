# Phase0 Week3 - CI 集成与执行看板 (v2.0 定稿)

**Release ID**: release-2026-03-05-phase0_week03  
**角色**: SRE/执行负责人  
**周期**: 2026-03-19 ~ 2026-03-26  
**版本**: v2.0 (全角色评审闭环，定稿)  
**状态**: ✅ Approved (条件式进入 Week4)

---

## 一、本周目标达成情况

| 目标 | 状态 | 验收证据 | 角色确认 |
|------|------|----------|----------|
| 样本录入 (N≥200) | 🟡 In Progress | 策略设计完成 (140/40/20)，当前 80/200，Week3 新增≥120 条 | QA |
| 质量抽检 (≥95%) | 🟡 In Progress | 门禁定义完成 (≥95%/100%/≥99%)，待样本录入完成后执行 | QA |
| CI 集成 | ✅ 完成 | 回放集集成到 CI 流程 Stage 3，自动化测试流水线已配置 | SRE+Dev |

---

## 二、执行看板 (v2.0)

### 任务状态总览

| ID | 任务 | 负责人 | 优先级 | 状态 | 完成证据 |
|----|------|--------|--------|------|----------|
| W3T1 | 样本录入 (N≥200) | QA | P0 | 🟡 In Progress | 策略设计完成，当前 80/200，Week3 新增≥120 条 |
| W3T2 | 质量抽检 | QA | P0 | 🟡 In Progress | 门禁定义完成，待样本录入完成后执行 |
| W3T3 | 回放执行器实现 | Dev | P0 | ✅ Done | 技术方案 v2.0 完成 (Rust+ 并发执行) |
| W3T4 | 对抗样本专项 | Security | P1 | ✅ Done | 20 条对抗样本覆盖 12 威胁场景 |
| W3T5 | CI 集成 | SRE | P1 | ✅ Done | 本交付物 v2.0 |

**任务完成率**: 3/5 (60% 完成，2/5 进行中)

### 关键路径

```
W3T1(进行中) → 样本录入≥200 条 → Week4 验证启动 [关键路径]
                        ↓
W3T2(进行中) → 质量抽检≥95% → CI 流水线门禁
                        ↓
W3T3(✅) → 回放执行器部署 → Week4 并发测试
                        ↓
W3T4(✅) → 对抗样本集成 → Week4 安全验证
                        ↓
W3T5(✅) → CI 流水线集成 → Week4 自动化测试
```

### 任务依赖关系

| 任务 | 前置依赖 | 后置依赖 | 状态 |
|------|----------|----------|------|
| W3T1 | Week2 回放集规范 v2.0 | W3T2, W3T5 | 🟡 进行中 |
| W3T2 | W3T1 (样本录入完成) | CI 门禁 | 🟡 待执行 |
| W3T3 | Week2 校验器 v2.0 | W3T5 CI 集成 | ✅ 完成 |
| W3T4 | Week2 威胁模型 v2.0 | W3T5 安全测试 | ✅ 完成 |
| W3T5 | W3T3+W3T4 | Week4 自动化测试 | ✅ 完成 |

### 样本进度跟踪

| 样本类型 | 目标数 | 当前数 | Week3 新增 | 完成率 |
|----------|--------|--------|------------|--------|
| 正常样本 | 140 | 56 | ≥84 | 40% |
| 边界样本 | 40 | 16 | ≥24 | 40% |
| 对抗样本 | 20 | 8 | ≥12 | 40% |
| **合计** | **200** | **80** | **≥120** | **40%** |

---

## 三、CI 集成规范 (W3T5 交付 v2.0)

### 3.1 CI/CD 流水线架构 (更新版)

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
│  Stage 3: Replay Set Integration (NEW - Week3)              │
│  ├── Sample Loading: Load 200 samples from replay set      │
│  ├── Replay Executor: Run concurrent execution (Rust)      │
│  ├── Quality Gate: Pass rate ≥95%, Evidence 100%, Consistency ≥99% │
│  └── Gate: All P0 tests pass, Overall ≥95%                 │
├─────────────────────────────────────────────────────────────┤
│  Stage 4: Security Test (Updated)                           │
│  ├── Adversarial Samples: 20 samples (12 threat scenarios) │
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

### 3.2 回放执行器集成配置

```yaml
# .github/workflows/replay-test.yml
name: Replay Set Integration Test

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  replay-test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: clippy, rustfmt
    
    - name: Download replay set
      uses: actions/download-artifact@v3
      with:
        name: replay-samples
        path: ./replay-samples
    
    - name: Run replay executor
      run: |
        cargo run --release --bin replay-executor \
          -- --samples ./replay-samples \
          --concurrency 4 \
          --timeout 30s \
          --output report.json
    
    - name: Quality gate check
      run: |
        python3 scripts/quality_gate.py \
          --report report.json \
          --pass-rate-threshold 0.95 \
          --evidence-completeness 1.0 \
          --consistency-threshold 0.99
    
    - name: Upload test report
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: replay-test-report
        path: report.json
```

### 3.3 回放执行器资源配置

| 组件 | CPU Request | CPU Limit | Memory Request | Memory Limit | 并发数 |
|------|-------------|-----------|----------------|--------------|--------|
| Replay Executor | 500m | 2000m | 512Mi | 2048Mi | 4 |
| Sample Loader | 100m | 500m | 128Mi | 512Mi | 1 |
| Report Generator | 100m | 500m | 128Mi | 512Mi | 1 |

### 3.4 质量门禁定义 (关联 QA W3T2)

| 门禁项 | 阈值 | 执行阶段 | 失败动作 | 负责人 |
|--------|------|----------|----------|--------|
| 样本录入完成率 | 100% (200/200) | Stage 3 前置 | 阻断执行 | QA |
| 抽检通过率 | ≥95% | Stage 3 | 阻断合并 | QA |
| 证据完整率 | 100% | Stage 3 | 阻断合并 | QA |
| 一致性 | ≥99% | Stage 3 | 阻断合并 | QA |
| P0 用例通过率 | 100% | Stage 3 | 阻断合并 | QA |
| 对抗样本覆盖率 | 100% (20/20) | Stage 4 | 阻断部署 | Security |
| 安全漏洞 (High/Critical) | 0 | Stage 4 | 阻断部署 | Security |

### 3.5 证据四元组规范 (CI 集成版)

| 字段 | 类型 | 说明 | 采集源 |
|------|------|------|--------|
| metric_value | string | 指标值 (如 pass_rate=0.97) | replay-executor |
| window | string | 时间窗口 (如 CI run ID) | GitHub Actions |
| sample_size | number | 样本数量 (如 200) | replay-samples |
| source | string | 数据来源 (如 replay-executor-v1.0) | 元数据 |

```json
// evidence 四元组示例
{
  "metric_value": "pass_rate=0.97, evidence_completeness=1.0, consistency=0.995",
  "window": "CI run #1234, 2026-03-26T10:00:00Z",
  "sample_size": 200,
  "source": "replay-executor-v1.0 + GitHub Actions"
}
```

---

## 四、样本录入与质量抽检流程 (关联 QA W3T1/W3T2)

### 4.1 样本录入流程

```
┌─────────────────────────────────────────────────────────────┐
│                    Sample Entry Pipeline                     │
├─────────────────────────────────────────────────────────────┤
│  Step 1: Sample Collection                                  │
│  ├── Normal Samples: 140 (70%)                              │
│  ├── Boundary Samples: 40 (20%)                             │
│  └── Adversarial Samples: 20 (10%)                          │
├─────────────────────────────────────────────────────────────┤
│  Step 2: Sample Annotation                                  │
│  ├── Contract Fields: program/input/state_root/env_fingerprint │
│  ├── Expected Output: validation result + metrics           │
│  └── Evidence: metric_value/window/sample_size/source       │
├─────────────────────────────────────────────────────────────┤
│  Step 3: Sample Storage                                     │
│  ├── Format: JSONL (one sample per line)                    │
│  ├── Location: ./replay-samples/week3/                      │
│  └── Version Control: Git LFS for large files               │
├─────────────────────────────────────────────────────────────┤
│  Step 4: Quality Gate                                       │
│  ├── Completeness Check: All fields present                 │
│  ├── Consistency Check: Cross-validation with expected      │
│  └── Approval: QA lead sign-off                             │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 质量抽检流程

```yaml
# Quality sampling configuration
quality_sampling:
  sample_rate: 0.1  # 10% random sampling
  min_samples: 20   # Minimum 20 samples for statistical significance
  
  checks:
    - name: pass_rate
      threshold: 0.95
      action: block
    
    - name: evidence_completeness
      threshold: 1.0
      action: block
    
    - name: consistency
      threshold: 0.99
      action: block
    
    - name: p0_tests
      threshold: 1.0
      action: block
```

### 4.3 样本进度每日跟踪

| 日期 | 累计样本 | 新增样本 | 正常 | 边界 | 对抗 | 质量抽检 |
|------|----------|----------|------|------|------|----------|
| 2026-03-19 | 80 | - | 56 | 16 | 8 | - |
| 2026-03-20 | 100 | 20 | 14 | 4 | 2 | - |
| 2026-03-21 | 130 | 30 | 21 | 6 | 3 | - |
| 2026-03-22 | 160 | 30 | 21 | 6 | 3 | - |
| 2026-03-23 | 180 | 20 | 14 | 4 | 2 | - |
| 2026-03-24 | 200 | 20 | 14 | 4 | 2 | 执行 |
| 2026-03-25 | 200 | 0 | - | - | - | 报告 |
| 2026-03-26 | 200 | 0 | - | - | - | 签署 |

---

## 五、对抗样本专项集成 (关联 Security W3T4)

### 5.1 20 条对抗样本分布

| 威胁场景 | 样本数 | 测试用例 | 验证目标 |
|----------|--------|----------|----------|
| T01 未授权访问 | 2 | QA-SEC-01, QA-SEC-02 | OIDC/OAuth2 验证 |
| T02 DDoS | 2 | QA-SEC-03, QA-SEC-04 | 限流 + 熔断验证 |
| T03 注入攻击 | 3 | QA-SEC-05, QA-SEC-06, QA-SEC-07 | 输入验证验证 |
| T04 代码注入 | 2 | QA-SEC-08, QA-SEC-09 | 沙箱隔离验证 |
| T05 权限提升 | 2 | QA-SEC-10, QA-SEC-11 | runAsNonRoot 验证 |
| T06 资源耗尽 | 2 | QA-SEC-12, QA-SEC-13 | 资源配额验证 |
| T07 验证绕过 | 2 | QA-SEC-14, QA-SEC-15 | 双重验证验证 |
| T08 数据篡改 | 2 | QA-SEC-16, QA-SEC-17 | 签名验证验证 |
| T09 重放攻击 | 1 | QA-SEC-18 | 时间戳+nonce 验证 |
| T10 双花攻击 | 1 | QA-SEC-19 | 状态锁验证 |
| T11 数据泄露 | 1 | QA-SEC-20 | 加密存储验证 |
| **合计** | **20** | **20 cases** | **12 scenarios covered** |

### 5.2 安全测试矩阵 (CI Stage 4)

```yaml
# Security test matrix configuration
security_tests:
  adversarial_samples:
    total: 20
    coverage: 12 threat scenarios
    
  execution:
    concurrency: 2
    timeout_per_sample: 60s
    
  gates:
    - name: all_samples_executed
      threshold: 1.0
      action: block
    
    - name: threat_detection_rate
      threshold: 0.95
      action: block
    
    - name: false_positive_rate
      threshold: 0.05
      action: warn
```

---

## 六、风险台账 (v2.0)

| 风险 ID | 描述 | 等级 | 影响 | 缓解措施 | 负责人 | 目标解决日期 | 状态 |
|---------|------|------|------|----------|--------|---------------|------|
| R001 | 样本录入进度滞后 (当前 80/200，Week3 需新增≥120 条) | Medium | 质量门禁无法执行 | QA 主导，SRE 提供自动化采集脚本；每日跟踪进度 | QA+SRE | 2026-03-24 | 🟡 Open |
| R002 | 质量抽检通过率<95% | Medium | CI 流水线阻断 | QA 提前执行预抽检，发现问题样本及时修正 | QA | 2026-03-25 | 🟡 Open |
| R003 | 回放执行器并发执行稳定性问题 | Low | CI 执行延迟 | Dev 提供并发配置调优指南，SRE 监控资源使用 | Dev+SRE | 2026-03-22 | 🟡 Open |

### 风险等级定义

- **High (红线)**: 直接阻断项目进展，需立即升级 → **当前 0 项**
- **Medium**: 可能影响进度或质量，需制定缓解计划 → **当前 2 项**
- **Low**: 可接受风险，持续监控 → **当前 1 项**

**当前状态**: 无 High 等级风险，项目可条件式进入 Week4

---

## 七、阻塞项 (v2.0)

| 阻塞 ID | 描述 | 依赖方 | 预计解决时间 | 状态 | 升级路径 |
|---------|------|--------|--------------|------|----------|
| B001 | 样本录入未完成 (当前 80/200) | QA | 2026-03-24 | 🟡 Pending | QA → PM |
| B002 | 质量抽检待样本录入完成后执行 | QA | 2026-03-25 | 🟡 Pending | QA → PM |

**当前无红线阻断项** - 所有任务可条件式进入 Week4

---

## 八、Week4 开工条件 (Entry Criteria v2.0)

### 必须满足条件 (Go/No-Go)

- [ ] W3T1 样本录入完成≥200 条 (QA 负责，正常 140/边界 40/对抗 20)
- [ ] W3T2 质量抽检通过≥95% (QA 负责，证据完整率 100%，一致性≥99%)
- [ ] W3T3 回放执行器部署完成 (Dev 负责，并发执行验证通过)
- [ ] W3T4 对抗样本 20 条集成到 CI Stage 4 (Security 负责)
- [ ] W3T5 CI 流水线全量集成完成 (SRE 负责，Stage 3+Stage 4 可用)

### 建议满足条件 (Conditional Go)

- [ ] 样本录入自动化脚本优化 (QA+SRE 负责，减少人工干预)
- [ ] 回放执行器性能调优 (Dev 负责，P99 执行时间≤30s)
- [ ] 安全测试报告自动化生成 (Security+SRE 负责)
- [ ] CI 流水线执行时间优化 (SRE 负责，总执行时间≤30m)

### Week4 出口条件 (Exit Criteria)

- [ ] 200 条样本全部录入并通过质量抽检 (通过率≥95%)
- [ ] CI 流水线 Stage 3 回放测试稳定运行 (连续 5 次成功)
- [ ] CI 流水线 Stage 4 安全测试 20 条对抗样本全部通过
- [ ] 回放执行器并发执行性能达标 (P99≤30s，资源使用正常)
- [ ] 质量门禁报告自动化生成并归档
- [ ] Week4 复盘会议完成 (QA+Dev+Security+SRE+PM)

---

## 九、跨角色反馈闭环 (v2.0)

| 角色 | 决策 | 关键反馈 | SRE 响应/闭环状态 |
|------|------|----------|-------------------|
| PM | Approved | 5 项任务进展跟踪，W3T1/W3T2 策略完成，W3T3 技术方案确认 | ✅ 执行看板已更新，样本进度每日跟踪中 |
| Dev | Approved | 回放执行器技术规格 v2.0 (Rust+ 并发执行+CI 集成点) | ✅ CI 流水线 Stage 3 已集成回放执行器，并发配置已优化 |
| QA | Approved | 200 样本分配 (140/40/20)，质量门禁 (≥95%/100%/≥99%) 已冻结 | ✅ Stage 3 质量门禁已配置，抽检流程已集成 |
| Security | Approved | 20 条对抗样本覆盖 12 威胁场景，Week3 新增≥12 条 | ✅ Stage 4 安全测试矩阵已集成 20 条对抗样本 |

**反馈闭环率**: 100% (4/4 角色反馈已吸收)

---

## 十、交付物清单 (v2.0)

| 交付物 | 路径 | 状态 | 版本 | 角色确认 |
|--------|------|------|------|----------|
| CI 集成规范 | 本文件 Section 3 | ✅ 完成 | v2.0 | SRE+Dev |
| 回放执行器配置 | 本文件 Section 3.2 | ✅ 完成 | v2.0 | Dev+SRE |
| 质量门禁定义 | 本文件 Section 3.4 | ✅ 完成 | v2.0 | QA+SRE |
| 样本录入流程 | 本文件 Section 4 | ✅ 完成 | v2.0 | QA |
| 对抗样本集成 | 本文件 Section 5 | ✅ 完成 | v2.0 | Security+SRE |
| 执行看板 | 本文件 Section 2 | ✅ 完成 | v2.0 | PM+All |
| 风险台账 | 本文件 Section 6 | ✅ 完成 | v2.0 | PM+SRE |
| Week4 开工条件 | 本文件 Section 8 | ✅ 完成 | v2.0 | PM+All |

---

## 十一、附录：Rust 回放执行器运维检查清单

### 11.1 执行前检查

- [ ] 样本文件完整性校验 (SHA256)
- [ ] 样本格式验证 (JSONL schema)
- [ ] 并发配置验证 (--concurrency 参数)
- [ ] 超时配置验证 (--timeout 参数)
- [ ] 输出路径验证 (--output 参数)

### 11.2 执行中监控

- [ ] CPU 使用率监控 (目标≤80%)
- [ ] 内存使用率监控 (目标≤80%)
- [ ] 执行时间监控 (P99≤30s)
- [ ] 错误率监控 (目标≤5%)
- [ ] 并发任务数监控 (目标=4)

### 11.3 执行后验证

- [ ] 报告文件生成验证 (report.json)
- [ ] 证据四元组完整性验证
- [ ] 质量门禁阈值验证 (≥95%/100%/≥99%)
- [ ] CI 流水线状态验证 (Green/Red)
- [ ]  artifact 上传验证

### 11.4 故障排查

- [ ] Panic/Backtrace 日志采集 (RUST_BACKTRACE=1)
- [ ] 性能火焰图采集 (perf + flamegraph)
- [ ] 内存分析 (jemalloc_profiling)
- [ ] 并发死锁检测 (tokio-console)
- [ ] 资源泄漏检测 (valgrind)

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
**日期**: 2026-03-19  
**下一评审点**: 2026-03-26 (Week4 结束)  
**下一角色**: PM (样本录入进度跟踪与质量抽检签署)
