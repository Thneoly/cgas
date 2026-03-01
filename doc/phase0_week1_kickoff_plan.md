# Phase 0 首周开工计划（Week 1 Kickoff）

## 0. 目标
在 5 个工作日内完成 Phase 0 的三项起步工作：
1. 冻结《确定性契约 v1.0》
2. 建立 `N >= 200` 黄金回放集
3. 打通 `gate-report.json` 产出链路

对应上位文档：
- [doc/review_framework_one_pager.md](doc/review_framework_one_pager.md)
- [doc/phase_detailed_execution_blueprint.md](doc/phase_detailed_execution_blueprint.md)

---

## 1. 周计划（Day 1 ~ Day 5）

| Day | 目标 | 关键任务 | 当日产出 | 当日验收 |
|---|---|---|---|---|
| Day 1 | 契约框架定稿 | 明确字段、哈希口径、版本策略、禁止项清单 | 《确定性契约 v1.0（草案）》 | 必须角色签字齐全（Runtime/Platform/QA/Security） |
| Day 2 | 契约冻结 | 完成争议项收敛、形成冻结版与变更流程 | 《确定性契约 v1.0（冻结版）》 | 冻结字段覆盖率 = 100% |
| Day 3 | 回放集设计 | 定义样本配比、标注规范、目录结构与元数据格式 | 《黄金回放集规范 v1.0》 | 样本规范可被脚本读取并校验 |
| Day 4 | 回放集构建 | 录入并标注样本，覆盖正常/边界/对抗 | `core-replay-set` 初版（N>=200） | 样本量达标且质量抽检通过 >= 95% |
| Day 5 | 门禁链路打通 | 聚合指标输入 -> 规则判定 -> schema 校验 -> 输出报告 | `gate-report.json` + `gate-report.md` | 连续 3 次产出 `gate-report.json` 均通过 schema 校验（通过率 = 100%） |

---

## 2. 任务拆解（可直接建 Jira）

## 2.1 Epic A：确定性契约 v1.0

### Story A1：字段定义与版本化
- 必填字段：`program`, `input`, `state_root`, `env_fingerprint`
- 版本字段：`contract_version`
- 变更策略：新增字段向后兼容，移除字段必须走评审例外

### Story A2：哈希与禁止项
- 哈希口径固定：输入规范化（排序、空值处理、编码）
- 禁止项：隐式时钟、隐式随机、未声明外部依赖

### Story A3：冻结与发布
- 输出冻结版文档
- 输出契约校验清单（机器可校验）

**DoD（完成定义）**
- 契约文档冻结并签署
- 字段覆盖率 `= 100%`
- 至少 1 个契约校验脚本可运行

---

## 2.2 Epic B：黄金回放集（N>=200）

### Story B1：样本结构与标注规范
- 样本分类：正常/边界/对抗
- 建议配比：`60% / 25% / 15%`
- 每条样本元数据：`id`, `type`, `risk_level`, `expected_status`, `notes`

### Story B2：样本采集与清洗
- 从现有任务、历史问题、威胁场景中生成样本
- 去重与质量过滤

### Story B3：质量抽检
- 抽检比例建议：`>= 20%`
- 错标修正闭环

**DoD（完成定义）**
- 样本总量 `>= 200`
- 覆盖三类样本且配比偏差 `<= 10%`
- 抽检通过率 `>= 95%`

---

## 2.3 Epic C：gate-report 产出链路

### Story C1：输入适配
- 读取聚合指标 JSON（字段口径对齐）
- 数据完整性校验（缺失字段直接失败）

### Story C2：规则执行
- 按 `gate_rules.example.yaml` 执行规则
- 计算比例指标与置信约束（Wilson 上界）

### Story C3：输出与校验
- 输出 `gate-report.json`
- 按 `gate_report_schema.json` 做 schema 校验
- 生成 `gate-report.md` 摘要

**DoD（完成定义）**
- `gate-report.json` 结构校验通过率 `= 100%`
- 字段映射归一化已验证（`camelCase -> snake_case`）
- 至少覆盖 1 组 PASS、1 组 FAIL 示例
- 失败场景可输出明确阻断原因

---

## 3. 首周验收清单（周五关卡）
- [ ] 《确定性契约 v1.0》冻结并归档
- [ ] 黄金回放集 `N >= 200`，分类与标注达标
- [ ] `gate-report.json` 产出链路打通并可重复运行
- [ ] 关键问题清单（Top 5）与下周修复计划已形成

---

## 4. 风险与应对（Week 1）

| 风险 | 触发信号 | 应对策略 |
|---|---|---|
| 契约争议无法收敛 | Day 2 仍有高优先争议项 > 3 个 | 冻结最小字段集，争议项降级为 v1.1 backlog |
| 样本质量不足 | 抽检通过率 < 95% | 增加对抗样本审阅人，先保质再扩量 |
| 门禁链路打不通 | schema 校验持续失败 | 降级为最小可运行路径：先产出 json，再补 markdown |

---

## 5. 角色建议（可替换为真实负责人）
- `Owner-Runtime`：契约与执行字段
- `Owner-QA`：回放集与标注质量
- `Owner-Platform`：门禁规则执行与报告链路
- `Owner-SRE`：作业调度与可观测

---

## 6. 下周衔接（Week 2 预告）
- 进入 Phase 1 准备：最小指令集实现与未验证提交硬阻断。
- 启动“隐式非确定性扫描器”在阻断模式的灰度演练。
