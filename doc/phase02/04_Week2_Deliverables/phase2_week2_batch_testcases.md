# Phase 2 Batch 指令测试用例

**版本**: v1.0  
**日期**: 2026-04-08  
**责任人**: QA  
**状态**: 📋 待执行  
**release_id**: release-2026-04-08-phase2_week02  

---

## 1. 测试概述

### 1.1 测试目标

验证 Batch 指令功能正确性、原子性保证、性能指标、安全闸门集成。

| 测试类别 | 用例数量 | 优先级 |
|---|---|---|
| 单元测试 | 15 | P0 |
| 集成测试 | 8 | P0 |
| E2E 测试 | 6 | P0 |
| 性能测试 | 5 | P1 |
| 安全测试 | 6 | P0 |
| **总计** | **40** | |

### 1.2 测试环境

| 环境 | 用途 | 配置 |
|---|---|---|
| dev | 单元测试 | 单节点 |
| staging | 集成/E2E | 3 节点集群 |
| performance | 性能压测 | 5 节点集群 |

---

## 2. 单元测试用例 (15 个)

### 2.1 功能测试 (8 个)

| 用例 ID | 用例描述 | 前置条件 | 测试步骤 | 预期结果 | 优先级 |
|---|---|---|---|---|---|
| UT-BATCH-001 | Batch 单条指令执行 | 执行器正常 | 1. 创建含 1 条指令的 Batch<br>2. 执行 Batch | 状态=Success, 结果数=1 | P0 |
| UT-BATCH-002 | Batch 多条指令执行 (10 条) | 执行器正常 | 1. 创建含 10 条指令的 Batch<br>2. 执行 Batch | 状态=Success, 结果数=10 | P0 |
| UT-BATCH-003 | Batch 最大指令数 (100 条) | 执行器正常 | 1. 创建含 100 条指令的 Batch<br>2. 执行 Batch | 状态=Success, 结果数=100 | P0 |
| UT-BATCH-004 | Batch 原子性成功 | 执行器正常 | 1. 创建 atomic=true 的 Batch<br>2. 执行 Batch | 状态=Success | P0 |
| UT-BATCH-005 | Batch 原子性失败回滚 | 执行器第 2 条失败 | 1. 创建 atomic=true 的 Batch<br>2. 执行 Batch | 状态=Failed, 已执行指令回滚 | P0 |
| UT-BATCH-006 | Batch 非原子性部分成功 | 执行器第 2 条失败 | 1. 创建 atomic=false 的 Batch<br>2. 执行 Batch | 状态=PartialFailure | P1 |
| UT-BATCH-007 | Batch 空指令验证 | - | 1. 创建空 Batch<br>2. 执行 Batch | 返回 ValidationError | P0 |
| UT-BATCH-008 | Batch 超大批验证 | - | 1. 创建含 101 条指令的 Batch<br>2. 执行 Batch | 返回 ValidationError | P0 |

### 2.2 哈希验证测试 (3 个)

| 用例 ID | 用例描述 | 前置条件 | 测试步骤 | 预期结果 | 优先级 |
|---|---|---|---|---|---|
| UT-BATCH-009 | Batch 哈希计算正确性 | - | 1. 执行 Batch<br>2. 验证 batch_hash | hash 与预期一致 | P0 |
| UT-BATCH-010 | Batch 哈希确定性 | - | 1. 相同输入执行 2 次<br>2. 比对 hash | 2 次 hash 相同 | P0 |
| UT-BATCH-011 | Batch 哈希唯一性 | - | 1. 不同输入执行 2 次<br>2. 比对 hash | 2 次 hash 不同 | P0 |

### 2.3 异常处理测试 (4 个)

| 用例 ID | 用例描述 | 前置条件 | 测试步骤 | 预期结果 | 优先级 |
|---|---|---|---|---|---|
| UT-BATCH-012 | Batch trace_id 无效验证 | - | 1. 创建 trace_id 为空的 Batch<br>2. 执行 Batch | 返回 ValidationError | P0 |
| UT-BATCH-013 | Batch 执行超时处理 | 执行器超时 | 1. 创建 Batch<br>2. 执行 Batch (超时) | 返回 Timeout 错误 | P1 |
| UT-BATCH-014 | Batch 回滚失败处理 | 回滚失败 | 1. 创建 atomic=true 的 Batch<br>2. 执行 Batch (回滚失败) | 返回 RollbackFailed 错误 | P1 |
| UT-BATCH-015 | Batch 并发执行测试 | 并发请求 | 1. 并发执行 10 个 Batch | 全部成功，无冲突 | P1 |

---

## 3. 集成测试用例 (8 个)

### 3.1 Batch + Executor 集成 (3 个)

| 用例 ID | 用例描述 | 前置条件 | 测试步骤 | 预期结果 | 优先级 |
|---|---|---|---|---|---|
| IT-BATCH-001 | Batch 端到端执行 | 环境就绪 | 1. 创建 Batch<br>2. 执行 Batch<br>3. 验证结果 | 状态=Success, 结果正确 | P0 |
| IT-BATCH-002 | Batch 状态持久化 | 数据库就绪 | 1. 执行 Batch<br>2. 查询数据库 | Batch 记录存在 | P0 |
| IT-BATCH-003 | Batch 结果查询 | Batch 已执行 | 1. 查询 Batch 结果<br>2. 验证结果 | 结果与执行一致 | P0 |

### 3.2 Batch + Verifier 集成 (2 个)

| 用例 ID | 用例描述 | 前置条件 | 测试步骤 | 预期结果 | 优先级 |
|---|---|---|---|---|---|
| IT-BATCH-004 | Batch 重放一致性 | Verifier 就绪 | 1. 执行 Batch<br>2. Verifier 重放<br>3. 比对结果 | 一致率≥99.95% | P0 |
| IT-BATCH-005 | Batch mismatch 检测 | 注入不一致 | 1. 执行 Batch<br>2. 篡改结果<br>3. Verifier 重放 | 检测到 mismatch | P0 |

### 3.3 Batch + SG-1~SG-4 集成 (3 个)

| 用例 ID | 用例描述 | 前置条件 | 测试步骤 | 预期结果 | 优先级 |
|---|---|---|---|---|---|
| IT-BATCH-006 | Batch SG-1 路径验证 | SG-1 启用 | 1. 执行 Batch<br>2. 验证路径 | SG-1 验证通过 | P0 |
| IT-BATCH-007 | Batch SG-3 哈希验证 | SG-3 启用 | 1. 执行 Batch<br>2. 篡改 batch_hash<br>3. 提交 | SG-3 阻断提交 | P0 |
| IT-BATCH-008 | Batch SG-4 重放检查 | SG-4 启用 | 1. 执行 Batch<br>2. 重复提交相同 Batch | SG-4 阻断重放 | P0 |

---

## 4. E2E 测试用例 (6 个)

### 4.1 端到端场景测试 (6 个)

| 用例 ID | 用例描述 | 前置条件 | 测试步骤 | 预期结果 | 优先级 |
|---|---|---|---|---|---|
| E2E-BATCH-001 | 客户端 Batch 创建→执行→验证 | 全链路就绪 | 1. 客户端创建 Batch<br>2. 执行 Batch<br>3. 验证结果 | 全流程成功 | P0 |
| E2E-BATCH-002 | Batch 监控指标采集 | Prometheus 就绪 | 1. 执行 Batch<br>2. 查询监控指标 | 指标数据正确 | P0 |
| E2E-BATCH-003 | Batch 告警触发 | 告警配置就绪 | 1. 执行高延迟 Batch<br>2. 验证告警 | P1 告警触发 | P1 |
| E2E-BATCH-004 | Batch 日志完整性 | 日志系统就绪 | 1. 执行 Batch<br>2. 查询日志 | 日志完整可追溯 | P0 |
| E2E-BATCH-005 | Batch 分布式追踪 | Jaeger 就绪 | 1. 执行 Batch<br>2. 查询 Trace | Trace 完整 | P1 |
| E2E-BATCH-006 | Batch 回滚 E2E | 回滚机制就绪 | 1. 执行 Batch (失败)<br>2. 验证回滚 | 数据回滚成功 | P0 |

---

## 5. 性能测试用例 (5 个)

### 5.1 性能基准测试 (5 个)

| 用例 ID | 用例描述 | 负载 | 测试步骤 | 验收标准 | 优先级 |
|---|---|---|---|---|---|
| PERF-BATCH-001 | Batch 执行时延 (P50/P90/P99) | 1000 请求 | 1. 执行 1000 个 Batch<br>2. 统计时延 | P99<400ms | P0 |
| PERF-BATCH-002 | Batch 吞吐量测试 | 10000 请求 | 1. 执行 10000 个 Batch<br>2. 计算吞吐量 | ≥100 请求/秒 | P1 |
| PERF-BATCH-003 | Batch 并发测试 | 100 并发 | 1. 并发执行 100 个 Batch<br>2. 验证成功率 | 成功率≥99.9% | P0 |
| PERF-BATCH-004 | Batch 大小影响测试 | 1/10/50/100 条指令 | 1. 不同大小 Batch 执行<br>2. 比较时延 | 开销<20% | P1 |
| PERF-BATCH-005 | Batch 稳定性测试 | 72 小时 | 1. 持续执行 Batch<br>2. 监控故障 | 零故障 | P0 |

---

## 6. 安全测试用例 (6 个)

### 6.1 安全闸门测试 (4 个)

| 用例 ID | 用例描述 | 前置条件 | 测试步骤 | 预期结果 | 优先级 |
|---|---|---|---|---|---|
| SEC-BATCH-001 | Batch 未验证提交阻断 | SG-1 启用 | 1. 绕过 Verifier 提交 Batch<br>2. 验证阻断 | SG-1 阻断提交 | P0 |
| SEC-BATCH-002 | Batch 哈希篡改检测 | SG-3 启用 | 1. 篡改 batch_hash<br>2. 提交 Batch | SG-3 阻断提交 | P0 |
| SEC-BATCH-003 | Batch 重放攻击阻断 | SG-4 启用 | 1. 重复提交相同 Batch<br>2. 验证阻断 | SG-4 阻断重放 | P0 |
| SEC-BATCH-004 | Batch 原子性违反告警 | 监控就绪 | 1. 执行 atomic=true 的 Batch (部分失败)<br>2. 验证告警 | 告警触发 | P0 |

### 6.2 对抗注入测试 (2 个)

| 用例 ID | 用例描述 | 前置条件 | 测试步骤 | 预期结果 | 优先级 |
|---|---|---|---|---|---|
| SEC-BATCH-005 | Batch 恶意指令注入 | 对抗测试环境 | 1. 注入恶意指令到 Batch<br>2. 执行 Batch | 恶意指令被阻断 | P0 |
| SEC-BATCH-006 | Batch 资源耗尽攻击 | 限流配置 | 1. 发送超大批量 Batch<br>2. 验证限流 | 请求被限流 | P1 |

---

## 7. 测试数据

### 7.1 标准 Batch 数据集

| 数据集 ID | 指令数量 | 原子性 | 用途 |
|---|---|---|---|
| DATASET-001 | 1 条 | true | 单条指令测试 |
| DATASET-002 | 10 条 | true | 标准 Batch 测试 |
| DATASET-003 | 50 条 | true | 中等批量测试 |
| DATASET-004 | 100 条 | true | 最大批量测试 |
| DATASET-005 | 10 条 | false | 非原子性测试 |

### 7.2 异常 Batch 数据集

| 数据集 ID | 异常类型 | 描述 | 用途 |
|---|---|---|---|
| DATASET-ERR-001 | 空 Batch | 0 条指令 | 验证空 Batch |
| DATASET-ERR-002 | 超大 Batch | 101 条指令 | 验证大小限制 |
| DATASET-ERR-003 | 无效 trace_id | trace_id 为空 | 验证 trace_id |
| DATASET-ERR-004 | 包含失败指令 | 第 2 条指令失败 | 验证原子性 |

---

## 8. 测试执行计划

### 8.1 Week 2 测试安排

| 时间 | 测试类型 | 用例数量 | 责任人 | 状态 |
|---|---|---|---|---|
| Week 2-T3 | 单元测试 | 15 | Dev+QA | 📋 待开始 |
| Week 2-T4 | 集成测试 | 8 | Dev+QA | 📋 待开始 |
| Week 2-T5 | E2E 测试 | 6 | QA | 📋 待开始 |
| Week 3-T1 | 性能测试 | 5 | SRE+QA | 📋 待开始 |
| Week 3-T2 | 安全测试 | 6 | Security+QA | 📋 待开始 |

### 8.2 测试通过标准

| 测试类型 | 通过率要求 | 阻塞标准 |
|---|---|---|
| 单元测试 | ≥97% | <90% 阻塞发布 |
| 集成测试 | 100% | <100% 阻塞发布 |
| E2E 测试 | 100% | <100% 阻塞发布 |
| 性能测试 | 全部达标 | 1 项不达标需评审 |
| 安全测试 | 100% | <100% 阻塞发布 |

---

## 9. 缺陷管理

### 9.1 缺陷优先级定义

| 优先级 | 描述 | 响应时间 |
|---|---|---|
| P0 | 阻塞性缺陷，无法继续测试 | 立即修复 |
| P1 | 严重缺陷，核心功能失败 | 24 小时内修复 |
| P2 | 一般缺陷，部分功能异常 | 本周内修复 |
| P3 | 轻微缺陷，不影响功能 | 下周内修复 |

### 9.2 缺陷报告模板

```markdown
## 缺陷 ID: BUG-BATCH-XXX

**标题**: [简短描述]

**严重程度**: P0/P1/P2/P3

**测试用例**: [用例 ID]

**复现步骤**:
1. [步骤 1]
2. [步骤 2]
3. [步骤 3]

**预期结果**: [预期行为]

**实际结果**: [实际行为]

**环境**: [测试环境]

**日志/截图**: [附件]

**根因分析**: [Dev 填写]

**修复方案**: [Dev 填写]

**验证结果**: [QA 填写]
```

---

## 10. 附录

### 10.1 测试脚本示例

```python
# test_batch_execute.py

import pytest
import requests

def test_batch_single_instruction():
    """测试单条指令 Batch 执行"""
    payload = {
        "trace_id": "trace_001",
        "batch_id": "batch_001",
        "instructions": [
            {
                "trace_id": "sub_001",
                "execution_id": "exec_001",
                "instruction_type": "CREATE",
                "payload": {},
                "timestamp": "2026-04-08T10:00:00Z"
            }
        ],
        "atomic": True,
        "timestamp": "2026-04-08T10:00:00Z"
    }
    
    response = requests.post(
        "http://localhost:8082/batch/execute",
        json=payload
    )
    
    assert response.status_code == 200
    result = response.json()
    assert result["status"] == "SUCCESS"
    assert len(result["results"]) == 1
```

### 10.2 参考文档

- Phase 2 Batch 设计文档
- Phase 2 Batch 实现指南
- Phase 2 测试矩阵 v1

---

**文档状态**: 📋 待执行  
**责任人**: QA  
**保管**: 项目文档库
