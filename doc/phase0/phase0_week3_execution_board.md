# Phase 0 Week 3 执行看板 - 回放集构建

**阶段目标**: 构建 N≥200 的黄金回放集

**时间窗**: 2026-03-19 ~ 2026-03-25

---

## 本周目标

1. **样本录入** - 完成 N≥200 样本录入与标注
2. **质量抽检** - 抽检通过率 ≥95%
3. **CI 集成** - 回放集集成到 CI 流程

---

## 任务表

| ID | 任务 | 负责人 | 优先级 | 状态 |
|----|------|--------|--------|------|
| W3T1 | 样本录入（N≥200） | QA | P0 | Todo |
| W3T2 | 质量抽检 | QA | P0 | Todo |
| W3T3 | 回放执行器实现 | Dev | P0 | Todo |
| W3T4 | 对抗样本专项 | Security | P1 | Todo |
| W3T5 | CI 集成 | SRE | P1 | Todo |

---

## 角色启动指令

### PM
**目标**: 输出范围总结

**任务**:
1. 总结 Phase 0 范围
2. 确认交付物完整性
3. 准备 Phase 0 收官报告框架

**交付路径**: `phase0_week3_scope_summary.md`

---

### Dev
**目标**: 实现回放执行器

**任务**:
1. 实现 `ReplayRunner` 结构
2. 支持批量执行
3. 输出执行报告

**交付路径**: `phase0_week3_replay_runner.md`

---

### QA
**目标**: 构建回放集（N≥200）

**任务**:
1. 录入正常样本（120 个）
2. 录入边界样本（50 个）
3. 录入对抗样本（30 个）
4. 质量抽检（≥20%）

**交付路径**: `phase0_week3_replay_set_n200.md`

---

### Security
**目标**: 输出对抗样本专项

**任务**:
1. 设计对抗场景
2. 生成对抗样本
3. 验证阻断有效性

**交付路径**: `phase0_week3_adversarial_samples.md`

---

### SRE
**目标**: CI 集成

**任务**:
1. 配置 CI 触发器
2. 集成回放执行器
3. 配置失败告警

**交付路径**: `phase0_week3_ci_integration.md`

---

## 周末验收清单

### 门禁检查
- [ ] **phase0_replay_set_ready**: 回放集已就绪
- [ ] **phase0_sample_minimum**: 样本量达标

### 交付物检查
- [ ] `phase0_week3_replay_set_n200.md` (N≥200)
- [ ] `phase0_week3_replay_runner.md`
- [ ] `phase0_week3_adversarial_samples.md`
- [ ] `phase0_week3_ci_integration.md`

---

## 下周准入条件

- ✅ 样本量 N≥200
- ✅ 抽检通过率 ≥95%
- ✅ 回放执行器可运行
- ✅ CI 集成完成

---

*最后更新：2026-03-05*
