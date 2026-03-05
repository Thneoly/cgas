# Phase 0 Week 2 执行看板 - 契约冻结

**阶段目标**: 完成确定性契约 v1.0 冻结，实现校验器

**时间窗**: 2026-03-12 ~ 2026-03-18

---

## 本周目标

1. **契约冻结** - 收敛争议项，发布冻结版文档
2. **校验器实现** - 完成契约校验器代码
3. **验证矩阵** - 定义校验测试用例

---

## 任务表

| ID | 任务 | 负责人 | 优先级 | 状态 |
|----|------|--------|--------|------|
| W2T1 | 争议项收敛与冻结 | PM | P0 | Todo |
| W2T2 | 契约校验器实现 | Dev | P0 | Todo |
| W2T3 | 验证矩阵定义 | QA | P0 | Todo |
| W2T4 | 威胁模型更新 | Security | P1 | Todo |
| W2T5 | 部署基线定义 | SRE | P1 | Todo |

---

## 角色启动指令

### PM
**目标**: 输出《确定性契约 v1.0（冻结版）》

**任务**:
1. 组织争议项评审会
2. 决策争议项（冻结/降级为 backlog）
3. 发布冻结版文档
4. 定义变更流程

**交付路径**: `phase0_week2_contract_frozen_v1.md`

---

### Dev
**目标**: 实现契约校验器

**任务**:
1. 实现 `ContractValidator` 结构
2. 实现字段完整性校验
3. 实现禁止项扫描
4. 输出校验器文档

**交付路径**: `phase0_week2_contract_validator.md`

---

### QA
**目标**: 输出验证矩阵

**任务**:
1. 定义校验测试用例
2. 制定通过标准
3. 设计自动化校验流程

**交付路径**: `phase0_week2_validation_matrix.md`

---

### Security
**目标**: 更新威胁模型

**任务**:
1. 基于冻结契约更新威胁模型
2. 识别新增攻击面
3. 制定缓解措施

**交付路径**: `phase0_week2_threat_model.md`

---

### SRE
**目标**: 输出部署基线

**任务**:
1. 定义部署环境要求
2. 制定健康检查标准
3. 设计回滚策略

**交付路径**: `phase0_week2_deployment_baseline.md`

---

## 周末验收清单

### 门禁检查
- [ ] **phase0_contract_frozen**: 契约冻结版已发布
- [ ] **phase0_validator_ready**: 校验器已实现并可运行

### 交付物检查
- [ ] `phase0_week2_contract_frozen_v1.md`
- [ ] `phase0_week2_contract_validator.md`
- [ ] `phase0_week2_validation_matrix.md`
- [ ] `phase0_week2_threat_model.md`
- [ ] `phase0_week2_deployment_baseline.md`

---

## 下周准入条件

- ✅ 契约冻结版签字完成
- ✅ 校验器可运行
- ✅ 验证矩阵完整
- ✅ 威胁模型无高危未决项

---

*最后更新：2026-03-05*
