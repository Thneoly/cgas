# Phase0 Week3 回放集构建规范 v2.0

**Release**: release-2026-03-05-phase0_week03  
**Role**: QA  
**状态**: 已签署(QA/PM/Dev闭环，待Security/SRE最终确认)  
**最后更新**: 2026-03-19  
**版本**: v2.0 (QA/PM/Dev评审闭环)  
**评审状态**: 条件式批准进入Week4

---

## 执行摘要

Phase0 Week3 回放集构建定稿完成，QA/PM/Dev评审闭环。5项任务中W3T1/W3T2策略设计完成，W3T3技术方案确认，W3T4/W3T5待Security/SRE最终输入。本周为样本采集冲刺周，需新增≥120条样本(累计80→200)。

### 关键成果

| 成果 | 状态 | 责任方 | 评审状态 |
|------|------|--------|----------|
| 样本录入策略(N≥200) | ✅ 完成 | QA | ✅ QA/PM确认 |
| 质量抽检门禁(≥95%) | ✅ 完成 | QA | ✅ QA/PM确认 |
| 回放执行器技术规格 | ✅ 完成 | Dev | ✅ QA/PM/Dev确认 |
| 对抗样本专项 | 🔴 待完成 | Security | ⏳ 待输入 |
| CI集成 | 🔴 待完成 | SRE | ⏳ 待输入 |

### 样本进度追踪

| 阶段 | 时间窗口 | 目标样本数 | 累计样本数 | 状态 |
|------|----------|------------|------------|------|
| Week1 | 2026-03-05 ~ 03-12 | 30条 | 30条 | ✅ 完成 |
| Week2 | 2026-03-12 ~ 03-19 | 50条 | 80条 | ✅ 完成 |
| **Week3** | **2026-03-19 ~ 03-26** | **+120条** | **200条** | 🟡 本周目标 |

### Week4准入条件

- 🟡 样本录入完成(200条)
- 🟡 质量抽检完成(≥95%)
- 🟡 回放执行器代码合并
- ⏳ 对抗样本专项完成(Security)
- ⏳ CI集成完成(SRE)

---

## 1. 回放集目标

构建**200条黄金回放样本集**，覆盖正常/边界/对抗三类场景，集成到CI流程实现自动化回归验证。

### 1.1 样本配比(70/20/10延伸至200条)

| 样本类型 | 占比 | Week1+Week2累计 | Week3新增 | 累计目标 | 状态 |
|----------|------|-----------------|-----------|----------|------|
| **正常样本** | 70% | 56条 | +84条 | 140条 | 🟡 采集中 |
| **边界样本** | 20% | 16条 | +24条 | 40条 | 🟡 采集中 |
| **对抗样本** | 10% | 8条 | +12条 | 20条 | 🟡 采集中 |
| **合计** | **100%** | **80条** | **+120条** | **200条** | 🟡 采集中 |

### 1.2 样本来源说明

| 来源 | 样本数 | 说明 | 状态 |
|------|--------|------|------|
| Week1回放集 | 30条 | 已采集，需复核标注 | ✅ 完成 |
| Week2验证用例 | 50条 | 已采集，需复核标注 | ✅ 完成 |
| Week3新增 | 120条 | 本周采集目标 | 🟡 进行中 |
| **合计** | **200条** | - | - |

---

## 2. 样本录入规范

### 2.1 样本元数据(JSON Schema v2.0)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["sample_id", "type", "program", "input_hash", "state_root", "env_fingerprint", "created_at", "expected_result", "quality_check"],
  "properties": {
    "sample_id": {"type": "string", "pattern": "^[NBA][0-9]{3}$"},
    "type": {"type": "string", "enum": ["normal", "boundary", "adversarial"]},
    "program": {"type": "string"},
    "input_hash": {"type": "string", "pattern": "^sha256:[a-f0-9]{64}$"},
    "state_root": {"type": "string", "pattern": "^0x[a-f0-9]{64}$"},
    "env_fingerprint": {
      "type": "object",
      "required": ["host", "container", "platform"],
      "properties": {
        "host": {"type": "string"},
        "container": {"type": "string"},
        "platform": {"type": "string"}
      }
    },
    "created_at": {"type": "string", "format": "date-time"},
    "expected_result": {"type": "string", "enum": ["success", "rejected", "replayed"]},
    "actual_result": {"type": "string"},
    "consistency_check": {"type": "string", "enum": ["pass", "fail", "pending"]},
    "quality_check": {
      "type": "object",
      "required": ["status", "checked_at", "checker"],
      "properties": {
        "status": {"type": "string", "enum": ["pass", "fail", "pending"]},
        "checked_at": {"type": "string", "format": "date-time"},
        "checker": {"type": "string"},
        "notes": {"type": "string"}
      }
    },
    "slo_metrics": {
      "type": "object",
      "properties": {
        "latency_p99_ms": {"type": "number"},
        "availability": {"type": "number"},
        "error_rate": {"type": "number"},
        "recovery_time_s": {"type": "number"}
      }
    },
    "audit_compliance": {
      "type": "object",
      "properties": {
        "5.1_traceability": {"type": "boolean"},
        "5.2_integrity": {"type": "boolean"},
        "5.3_tamper_proof": {"type": "boolean"}
      }
    },
    "evidence_refs": {"type": "array", "items": {"type": "string"}},
    "ci_integrated": {"type": "boolean", "default": false}
  }
}
```

### 2.2 样本标注流程

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   采集      │───▶│   标注      │───▶│   质检      │───▶│   入库      │
│  (自动)     │    │  (半自动)   │    │  (抽检)     │    │  (归档)     │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
     │                  │                  │                  │
     ▼                  ▼                  ▼                  ▼
 原始日志          元数据填充         抽检≥95%          回放集v1.0
 执行结果          证据关联           错误复核          CI集成
```

### 2.3 样本标注规范

| 字段 | 填写方式 | 责任人 | 校验规则 |
|------|----------|--------|----------|
| sample_id | 自动生成 | QA | 唯一性校验 |
| type | 手动标注 | QA | 必须为normal/boundary/adversarial |
| program | 自动提取 | Dev | 必须为已注册契约 |
| input_hash | 自动计算 | Dev | SHA256校验 |
| state_root | 自动提取 | Dev | 0x前缀+64hex |
| env_fingerprint | 自动采集 | SRE | 三层结构完整 |
| expected_result | 手动标注 | QA | 必须为success/rejected/replayed |
| quality_check | 质检填写 | QA | 抽检≥95%通过 |

---

## 3. 质量抽检策略

### 3.1 抽检规则

| 指标 | 要求 | 验证方式 | 频率 |
|------|------|----------|------|
| **抽检通过率** | ≥95% | 随机抽检 | 每50条 |
| **标注完整率** | 100% | 全量检查 | 每批次 |
| **证据完整率** | 100% | 全量检查 | 每批次 |
| **一致性通过率** | ≥99% | 回放验证 | 每周 |
| **CI集成率** | 100% | 流水线检查 | 入库前 |

### 3.2 抽检样本量计算

```yaml
总体本量: 200条
置信水平: 95%
允许误差: 5%
最小抽检量: 32条 (根据统计公式计算)
实际抽检量: 40条 (20%抽检率)
```

### 3.3 质检检查清单

| 检查项 | 检查方式 | 通过标准 | 不合格处置 |
|--------|----------|----------|------------|
| 元数据完整性 | 自动校验 | 所有必填字段存在 | 退回补充 |
| 证据四元组 | 自动校验 | 4项证据齐全 | 退回补充 |
| 标注一致性 | 人工复核 | 与预期结果一致 | 退回修正 |
| 回放一致性 | 自动回放 | 相同输入→相同输出 | 调查根因 |
| 安全审计 | 自动校验 | 5.1-5.3证据齐全 | 退回补充 |

### 3.4 质量门禁

| 门禁项 | 阈值 | 状态 | 处置 |
|--------|------|------|------|
| 抽检通过率 | ≥95% | 🟡 待执行 | <95%触发全量复核 |
| 标注错误率 | ≤5% | 🟡 待执行 | >5%触发标注培训 |
| 证据缺失率 | 0% | 🟡 待执行 | >0%触发补充采集 |
| 回放失败率 | ≤1% | 🟡 待执行 | >1%触发根因分析 |

---

## 4. 回放执行器规范(Dev v2.0)

### 4.1 执行器架构

```
┌─────────────────────────────────────────────────────────────┐
│                    回放执行器 (Rust)                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Loader    │  │  Executor   │  │    Comparator       │  │
│  │  (samples)  │──│  (replay)   │──│  (consistency)      │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
         │                   │                    │
         ▼                   ▼                    ▼
    样本库加载          契约执行            一致性比对
    (200条)           (gRPC调用)          (哈希校验)
```

### 4.2 执行器接口(Dev v2.0)

```rust
// src/replay_executor/lib.rs
pub struct ReplayExecutor {
    samples: Vec<Sample>,
    validator: ContractValidator,
    concurrency: usize,
}

impl ReplayExecutor {
    pub fn new(concurrency: usize) -> Self;
    pub fn load_samples(&mut self, path: &str) -> Result<usize>;
    pub fn execute(&self, sample: &Sample) -> ExecutionResult;
    pub fn execute_batch(&self, samples: &[Sample]) -> Vec<ExecutionResult>;
    pub fn verify_consistency(&self, expected: &Result, actual: &Result) -> bool;
    pub fn generate_report(&self) -> ReplayReport;
}

pub struct ReplayReport {
    pub total_samples: usize,
    pub passed: usize,
    pub failed: usize,
    pub consistency_rate: f64,
    pub slo_compliance: SLOMetrics,
    pub execution_time_ms: u64,
}
```

### 4.3 并发执行逻辑(Dev v2.0)

| 模式 | 并发数 | 使用场景 | 性能目标 |
|------|--------|----------|----------|
| **单线程** | 1 | 调试/问题复现 | - |
| **标准并发** | 4 | 日常CI | P99≤100ms |
| **高并发** | 8 | 全量回放 | 完成时间≤10m |
| **极限并发** | 16 | 性能测试 | 吞吐量最大化 |

### 4.4 执行模式

| 模式 | 说明 | 使用场景 |
|------|------|----------|
| **全量回放** | 执行全部200条样本 | 发布前验证 |
| **增量回放** | 执行新增样本 | 日常CI |
| **回归回放** | 执行失败样本 | 问题修复后 |
| **抽样回放** | 随机抽取40条 | 快速验证 |

---

## 5. 对抗样本专项(Security W3T4)

### 5.1 对抗样本扩展(20条)

基于Week2威胁模型12场景，扩展至20条对抗样本：

| ID | 威胁场景 | 攻击类型 | 预期行为 | 样本数 |
|----|----------|----------|----------|--------|
| **A01** | T01 DDoS | 高频请求(1000/s) | 限流+429 | 2条 |
| **A02** | T02 协议混淆 | 非法gRPC帧 | 拒绝+日志 | 2条 |
| **A03** | T03 未授权 | 伪造token | 拒绝+审计 | 2条 |
| **A04** | T04 代码注入 | 恶意payload | 阻断+告警 | 2条 |
| **A05** | T05 状态篡改 | 伪造state_root | 拒绝+审计 | 2条 |
| **A06** | T06 资源耗尽 | 超大payload(10MB) | 限流+拒绝 | 2条 |
| **A07** | T07 逻辑绕过 | 特殊字符注入 | 拒绝+日志 | 2条 |
| **A08** | T08 数据污染 | 编码攻击 | 拒绝+日志 | 2条 |
| **A09** | T09 时序攻击 | 超时请求 | 超时+回滚 | 2条 |
| **A10** | T10 存储篡改 | 直接DB访问 | 阻断+告警 | 2条 |
| **A11** | T11 事务攻击 | 并发事务冲突 | 重试/失败 | 2条 |
| **A12** | T12 重放攻击 | 重复请求 | 检测+去重 | 2条 |
| **A13-A20** | 扩展场景 | 组合攻击 | 依场景定义 | 8条 |
| **合计** | - | - | - | **28条** |

注：对抗样本目标20条，上述28条为候选集，择优选取20条。待Security确认最终选取。

### 5.2 安全审计证据

| 审计要求 | 证据类型 | 采集点 | 保留期 |
|----------|----------|--------|--------|
| 5.1 可追溯 | 操作日志 | 所有对抗样本执行日志 | 365天 |
| 5.2 完整性 | 哈希校验 | 输入/输出哈希比对 | 365天 |
| 5.3 防篡改 | WORM存储 | 证据库存储 | 365天 |

### 5.3 待Security确认事项

| 事项 | 说明 | 截止 |
|------|------|------|
| 对抗样本最终选取 | 从28条候选集选取20条 | 2026-03-24 |
| 安全审计证据格式 | 证据采集点详细规范 | 2026-03-24 |
| 威胁场景覆盖确认 | 12场景100%覆盖 | 2026-03-24 |

---

## 6. CI集成规范(SRE W3T5)

### 6.1 CI流水线设计

```yaml
# .github/workflows/replay_validation.yml
name: Replay Set Validation

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  replay_test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Load Replay Samples
        run: |
          echo "Loading ${SAMPLE_COUNT} samples..."
          ./scripts/load_samples.sh artifacts/replay_samples/week3/
      
      - name: Execute Replay
        run: |
          ./target/release/replay_executor \
            --samples 200 \
            --concurrency 4 \
            --report reports/replay_report.json
      
      - name: Quality Check
        run: |
          ./scripts/quality_check.sh \
            --threshold 0.95 \
            --report reports/quality_report.json
      
      - name: Upload Report
        uses: actions/upload-artifact@v4
        with:
          name: replay-report
          path: reports/
```

### 6.2 CI门禁集成

| 门禁项 | 阈值 | CI检查点 | 失败处置 |
|--------|------|----------|----------|
| 样本总数 | ≥200条 | 加载阶段 | 阻断流水线 |
| 抽检通过率 | ≥95% | 质检阶段 | 阻断流水线 |
| 回放通过率 | ≥99% | 执行阶段 | 阻断流水线 |
| 证据完整率 | 100% | 报告阶段 | 阻断流水线 |
| P99延迟 | ≤100ms | 性能阶段 | 警告 |

### 6.3 待SRE确认事项

| 事项 | 说明 | 截止 |
|------|------|------|
| CI流水线最终配置 | GitHub Actions完整配置 | 2026-03-25 |
| 门禁检查点实现 | 5门禁检查点脚本 | 2026-03-25 |
| 报告输出格式 | JSON报告规范确认 | 2026-03-25 |

---

## 7. 执行计划

### 7.1 Week3里程碑

| 日期 | 里程碑 | 交付物 | 状态 | 签署方 |
|------|--------|--------|------|--------|
| 2026-03-21 | 样本录入完成(200条) | 样本集v1.0 | 🟡 进行中 | QA |
| 2026-03-22 | 质量抽检完成 | 质检报告 | 🔴 待执行 | QA |
| 2026-03-23 | 回放执行器完成 | 执行器v1.0 | 🟡 技术方案确认 | Dev |
| 2026-03-24 | 对抗样本专项完成 | 对抗样本集 | 🔴 待执行 | Security |
| 2026-03-25 | CI集成完成 | 流水线就绪 | 🔴 待执行 | SRE |
| 2026-03-26 | Week3收尾评审 | 测试报告+风险清单 | 🔴 待完成 | 全角色 |

### 7.2 任务完成状态

| ID | 任务 | 负责人 | 优先级 | 状态 | 评审意见 |
|----|------|--------|--------|------|----------|
| W3T1 | 样本录入(N≥200) | QA | P0 | 🟡 进行中(80/200) | 策略设计完成 |
| W3T2 | 质量抽检 | QA | P0 | 🟡 待执行 | 门禁定义完成 |
| W3T3 | 回放执行器实现 | Dev | P0 | 🟡 技术方案确认 | Rust+并发执行确认 |
| W3T4 | 对抗样本专项 | Security | P1 | 🔴 待执行 | 待Security输入 |
| W3T5 | CI集成 | SRE | P1 | 🔴 待执行 | 待SRE确认 |

### 7.3 Week4预告

| 任务 | 目标 | 交付物 | 责任人 |
|------|------|--------|--------|
| 全量回放验证 | 200条100%执行 | 回放报告 | QA |
| 性能基线测试 | P99延迟基线 | 性能报告 | QA+SRE |
| 安全审计验收 | 5.1-5.3证据验收 | 审计报告 | Security |
| Phase0验收准备 | 全任务收尾 | 验收文档 | PM |

---

## 8. 风险控制

### 8.1 风险台账

| 风险 | 概率 | 影响 | 缓解措施 | 责任人 | 状态 |
|------|------|------|----------|--------|------|
| 样本采集进度滞后 | 中 | 高 | 每日追踪+自动化采集 | QA | 🟢 监控中 |
| 质量抽检不达标 | 中 | 高 | 早期抽检+及时修正 | QA | 🟡 待执行 |
| 回放执行器延迟 | 低 | 中 | 技术方案已确认 | Dev | 🟢 已确认 |
| 对抗样本覆盖不全 | 低 | 中 | 12威胁场景强制覆盖 | Security | 🟡 待执行 |
| CI集成失败 | 中 | 中 | 并行测试环境 | SRE | 🟡 待执行 |

### 8.2 待闭环事项

| 事项 | 责任方 | 截止 | 影响 |
|------|--------|------|------|
| 样本录入(200条) | QA | 2026-03-21 | Week3门禁通过 |
| 质量抽检完成 | QA | 2026-03-22 | 质量门禁确认 |
| 回放执行器代码合并 | Dev | 2026-03-23 | CI集成就绪 |
| 对抗样本专项 | Security | 2026-03-24 | 安全验证就绪 |
| CI集成完成 | SRE | 2026-03-25 | 自动化验证就绪 |

---

## 9. 交付物清单

| 交付物 | 路径 | 状态 | 责任人 |
|--------|------|------|--------|
| 回放集规范 | phase0_week3_replay_set_n200.md | ✅ 完成v2.0 | QA |
| 样本集(200条) | artifacts/replay_samples/week3/ | 🟡 采集中(80/200) | QA |
| 质检报告 | reports/quality/week3/ | 🔴 待生成 | QA |
| 回放执行器 | src/replay_executor/ | 🟡 技术方案确认 | Dev |
| 对抗样本集 | artifacts/adversarial_samples/ | 🔴 待执行 | Security |
| CI流水线 | .github/workflows/replay_validation.yml | 🔴 待执行 | SRE |
| 回放报告 | reports/replay/week3/ | 🔴 待生成 | QA |

---

## 10. 证据四元组规范(Week3)

### 10.1 证据四元组模板

```yaml
证据四元组:
  metric_value: "<指标值，如样本200条/抽检96%/回放99%/P99=85ms>"
  window: "<时间窗口，如Phase0 Week3 2026-03-19至2026-03-26>"
  sample_size: "<样本数量，如200>"
  source: "<证据来源，如reports/replay/week3/replay_report.json>"
```

### 10.2 Week3证据采集点

| 阶段 | 证据类型 | 采集方式 | 存储位置 | 关联审计 |
|------|----------|----------|----------|----------|
| SampleLoad | 加载日志 | 自动采集 | artifacts/sample_load/ | 5.1 可追溯 |
| QualityCheck | 质检报告 | 自动采集 | reports/quality/ | 5.2 完整性 |
| ReplayExecute | 执行日志 | 自动采集 | artifacts/replay_exec/ | 5.1 可追溯 |
| ConsistencyVerify | 比对结果 | 自动采集 | artifacts/consistency/ | 5.3 防篡改 |
| CIIntegration | 流水线日志 | 自动采集 | artifacts/ci_logs/ | 5.1-5.3 |

### 10.3 证据完整性校验

```yaml
校验规则:
  - 200条样本必须100%有证据四元组
  - 质检报告必须包含抽检样本详情
  - 回放报告必须包含一致性比对结果
  - CI流水线日志必须包含门禁检查结果
  - 所有证据必须启用WORM存储
```

---

## 11. 签署栏

### 11.1 角色签署

| 角色 | 姓名 | 日期 | 意见 | 签署 |
|------|------|------|------|------|
| QA | | 2026-03-19 | 回放集构建规范v2.0完成，200样本目标确认，质量门禁冻结 | ✅ |
| PM | | 2026-03-19 | 5项任务进展确认，W3T1/W3T2/W3T3已闭环，条件式批准进入Week4 | ✅ |
| Dev | | 2026-03-19 | 回放执行器技术规格v2.0确认，Rust+并发执行方案确认 | ✅ |
| Security | | | 待补充对抗样本专项方案 | ⏳ |
| SRE | | | 待补充CI集成方案 | ⏳ |

### 11.2 签署声明

本规范v2.0经QA/PM/Dev评审闭环，待Security/SRE最终确认。确认以下内容:

1. **样本目标**: 200条(正常140/边界40/对抗20)，当前累计80条，Week3新增≥120条
2. **质量门禁**: 抽检通过率≥95%、回放通过率≥99%、证据完整率100%
3. **回放执行器**: Rust实现，支持并发执行(1/4/8/16并发数可选)
4. **待闭环事项**: Security对抗样本专项(W3T4)、SRE CI集成(W3T5)
5. **Week4准入**: 样本录入完成、质量抽检完成、回放执行器代码合并、对抗样本专项完成、CI集成完成

---

## 版本历史

| 版本 | 日期 | 变更说明 | 签署人 |
|------|------|----------|--------|
| v1.0 | 2026-03-19 | 初始版本，基于Week1/Week2规范延伸，200样本目标定义 | QA |
| v2.0 | 2026-03-19 | QA/PM/Dev评审闭环，吸收PM(5任务进展)、Dev(回放执行器v2.0 Rust+并发)全部反馈 | QA/PM/Dev |

---

**文档控制**

- **保密级别**: 内部
- **保留期限**: 项目结束后3年
- **分发列表**: PM/Dev/QA/SRE/Security
- **变更控制**: 任何变更需经全角色评审并更新版本号

---

**QA备注**: 本规范v2.0已QA/PM/Dev签署。当前样本累计80/200条，Week3目标新增≥120条。待Security对抗样本专项(W3T4)、SRE CI集成(W3T5)完成后最终签署并进入Week4。
