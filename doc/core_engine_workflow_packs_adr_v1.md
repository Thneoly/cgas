# Core Engine / Workflow Packs 重构 ADR v1

**版本**: v1.0 (Draft)
**日期**: 2026-03-10
**责任人**: Architect
**状态**: 📋 草案评审中
**适用范围**: rust-workflow-engine, report-viewer, doc/workflow assets

---

## 1. 背景与问题陈述

当前仓库同时承载了两类目标：

1. 通用 Agent 基础设施目标：
   - 可验证执行
   - 确定性契约
   - 验证后提交
   - 可治理演化

2. 特定 Phase 协作流程目标：
   - PM / Dev / QA / Security / SRE 多角色协作
   - Phase01 周计划与文档交付
   - 固定门禁与固定交付路径

这两类目标被实现到了同一层，导致“产品壳”侵入“执行内核”。

当前偏离主要体现在以下事实：

- 核心模型使用固定角色枚举，而非通用 actor/capability 抽象。
- 默认调度链路是固定组织审批流，而非 program/policy 驱动的动作图。
- 门禁函数绑定到 PM/Dev/QA/Security 等角色名，而非绑定到契约结果、验证结果、风险等级。
- Prompt 构造与交付路径写死为 Phase01 文档生产流程，而非 workflow pack 配置。
- Mock executor 输出默认 `PM -> Dev -> QA -> Security -> SRE`，强化了组织流转假设。

这意味着当前系统更像“特定组织的多角色工作流引擎”，而不是“可复用的 Agent 执行基础设施”。

---

## 2. 决策目标

本 ADR 目标不是重写全部代码，而是恢复正确的架构边界：

1. 让核心执行层重新回到“通用、稳定、可验证”的抽象。
2. 把 Phase01/Phase02/Phase03 等特定流程下沉为可替换的 workflow packs。
3. 避免后续每增加一种组织流程，都改动核心 Rust 域模型。
4. 保留已有可用资产：契约、事务、验证、报告、Docker、已有测试与文档。

---

## 3. 架构决策

### 3.1 ADR-CE-001: 采用双层架构，而非继续单层演进

**决策**:
将现有系统拆分为两层：

- `core engine`
- `workflow packs`

**理由**:

- `core engine` 应承载稳定语义：状态、契约、执行、验证、提交、审计。
- `workflow packs` 应承载可变语义：角色、提示词、交付物、流程模板、阶段门禁。
- 两者变化频率不同，耦合会直接抬高维护成本和设计漂移风险。

**目标结构**:

```text
Application / UI / CLI
        |
        v
Workflow Pack Loader
        |
        v
Core Engine API
        |
        v
Contract / Executor / Verifier / Committer / Audit
```

---

### 3.2 ADR-CE-002: Core Engine 不再暴露固定角色模型

**决策**:
核心层废止以 `Role::PM / Dev / QA / SRE / Security / Blackboard` 为中心的域模型，改为更通用的执行抽象。

**核心抽象建议**:

```text
ActorId
CapabilitySet
TaskNode
TaskState
ArtifactType
PolicyDecision
ExecutionContract
VerificationResult
CommitDecision
```

**约束**:

- 核心层不得出现 PM、QA、SRE 这类组织角色名。
- 核心层只接受 pack 注入的 actor/capability/task graph 数据。
- 核心层调度只基于状态机、策略结果和图边，而非硬编码默认顺序。

**理由**:

- 组织结构是业务配置，不是执行内核语义。
- 角色名天然不稳定，能力集合更适合作为长期抽象。

---

### 3.3 ADR-CE-003: Workflow Pack 成为唯一的流程定制入口

**决策**:
Phase 流程、角色定义、提示词、交付模板、门禁规则全部收口到 workflow pack。

**pack 内容建议**:

```text
pack.yaml
actors.yaml
task_graph.yaml
prompts/
schemas/
gates/
deliverables/
```

**pack 负责定义**:

- 有哪些 actor
- actor 拥有哪些 capability
- 初始任务节点是什么
- 任务图如何流转
- 每类 artifact 的 schema
- 哪些 gate 在何时触发
- 默认交付物模板与路径

**core engine 负责执行**:

- 加载 pack
- 校验 pack schema
- 执行 task graph
- 记录 trace / result_hash / state_diff
- 调用 verifier / committer

---

### 3.4 ADR-CE-004: Gate 从“角色审批”迁移到“契约判定”

**决策**:
门禁从当前的角色绑定函数，重构为规则驱动的 gate evaluator。

**不再推荐的 gate 形式**:

- `pm_dev_qa_approved`
- `security_or_exception`

**推荐 gate 形式**:

- `required_artifact_types_present`
- `required_verification_passed`
- `risk_threshold_satisfied`
- `required_policy_controls_passed`
- `required_approvals_present`
- `no_unverified_commit`

**理由**:

- “谁批准”属于流程配置。
- “满足哪些条件才能继续”才是稳定规则。

---

### 3.5 ADR-CE-005: 保留当前仓库并原地重构，不立即新建项目

**决策**:
优先在当前仓库内完成分层重构，不新建 greenfield 项目。

**理由**:

1. 当前仓库已有可复用核心资产：
   - 确定性契约
   - 事务执行语义
   - 验证与哈希相关模块
   - 安全/策略模块
   - 报告查看器
   - Docker 和环境配置

2. 直接新建项目的主要风险：
   - 重复建设基础设施
   - 文档与代码进一步分裂
   - 现有 Phase 工程资产失去迁移路径
   - 团队短期内会把“重新搭框架”误当成“解决架构问题”

3. 当前问题本质是边界错误，不是技术选型彻底失败。

**例外条件**:
仅在满足以下条件时，才建议新建项目：

- 核心域模型已无法兼容迁移，重构成本高于 60%-70% 重写成本。
- 需要在语言/运行时层面彻底切换技术路线。
- 必须保留现有工作流系统继续稳定产出，且不能承受大规模重构风险。

在当前仓库状态下，上述条件都未被证明成立，因此不建议直接新建项目。

---

## 4. 方案对比：重构现有仓库 vs 新建项目

| 维度 | 原地分层重构 | 新建项目 |
|---|---|---|
| 交付速度 | 中 | 低 |
| 风险可控性 | 中高 | 中 |
| 现有资产复用 | 高 | 低 |
| 设计纯度 | 中高 | 高 |
| 团队切换成本 | 低 | 高 |
| 回退能力 | 高 | 中 |
| 文档连续性 | 高 | 低 |

**结论**:

- 近期目标是恢复架构边界与交付节奏，应选“原地分层重构”。
- 新建项目只适合作为“第二阶段内核提纯”方案，而不是当前第一反应。

---

## 5. 目标分层定义

### 5.1 Core Engine 边界

Core Engine 应包含：

- Contract
- State model
- Task graph runtime
- Deterministic execution
- Verification API
- Commit / rollback
- Audit / trace / metrics hooks
- Policy evaluation interface
- Pack loading interface

Core Engine 不应包含：

- Phase01/02/03 prompt 内容
- PM/Dev/QA/SRE 等角色定义
- 周度 deliverable 文件名
- 固定审批顺序
- 任何特定组织模板

### 5.2 Workflow Pack 边界

Workflow Pack 应包含：

- actor 定义
- capability 映射
- task graph
- gate 规则配置
- prompt 模板
- artifact schema
- deliverable 模板
- pack 级默认环境配置

Workflow Pack 不应包含：

- 核心状态提交逻辑
- 结果验证算法
- trace/result/state_diff 哈希算法
- 核心事务语义

---

## 6. 拟议目录结构

```text
rust-workflow-engine/
  src/
    core/
      contract/
      runtime/
      verifier/
      commit/
      audit/
      policy/
      pack/
    adapters/
      cli/
      mock/
      file_store/
    legacy_phase_workflow/
      compatibility/

workflow-packs/
  phase01/
    pack.yaml
    actors.yaml
    task_graph.yaml
    prompts/
    gates/
    schemas/
    deliverables/
  phase02/
  phase03/

report-viewer/
  src/
    pack-aware views
```

说明：

- `legacy_phase_workflow` 是迁移过渡层，不是长期核心目录。
- `workflow-packs` 可先放在仓库根目录，后续再决定是否独立发布。

---

## 7. 迁移策略

### Phase A: 边界冻结

目标：停止继续把 Phase 语义写入核心层。

动作：

- 停止新增固定角色到核心 `Role` 模型。
- 停止新增角色绑定 gate。
- 停止在 executor 中新增 Phase 特定 prompt/交付逻辑。

### Phase B: 建立 Core API

目标：先抽接口，再迁移实现。

动作：

- 定义 `PackDefinition`、`ActorDefinition`、`TaskNodeDefinition`。
- 定义通用 gate evaluator 接口。
- 定义 actor/capability 到 task graph 的执行接口。

### Phase C: Phase01 Pack 外迁

目标：让现有主流程先跑在 pack 上。

动作：

- 把 Phase01 角色、prompt、deliverables、gates 迁到 `workflow-packs/phase01`。
- 保留兼容适配层，确保当前 demo 与已有交付不立即中断。

### Phase D: 清理 Legacy 内核耦合

目标：从核心层删除组织语义。

动作：

- 删除角色专属默认顺序。
- 删除硬编码 deliverable path 映射。
- 删除 phase01 identity prompt 进入核心执行路径。

---

## 8. 风险与缓解

### 风险 1: 迁移期间功能回归

缓解：

- 保留 compatibility adapter。
- 以 pack 加载方式复刻现有 Phase01 行为，先等价迁移再做抽象优化。

### 风险 2: 抽象过度，短期无法交付

缓解：

- 先迁移现有单一场景 pack，不同时设计通用 marketplace。
- 先实现最小 `core + phase01 pack` 闭环。

### 风险 3: 团队继续在旧层上加逻辑

缓解：

- 新增代码评审规则：任何新增 Phase 语义必须进入 pack，不得进入 core。

---

## 9. 成功判定标准

满足以下条件，视为本次架构纠偏成功：

1. Core 层不再包含 PM/Dev/QA/SRE/Security 等固定角色枚举。
2. Phase01 可以作为一个 pack 被加载并运行。
3. Gate 可由配置驱动，而非必须通过角色名绑定。
4. 现有 deterministic contract / verification / commit 语义保持不变。
5. report-viewer 能区分 core-level execution data 与 workflow-level pack data。

---

## 10. 本 ADR 的最终建议

**建议结论**: 采用“原地重构 + 双层拆分”方案，不建议现在直接重建新项目。

原因很简单：

- 你们当前不是“没有内核”，而是“内核被 workflow 模板污染”。
- 这类问题最有效的解法是重新建立边界，而不是放弃已有资产。
- 如果现在直接新建项目，大概率会在 4-8 周后重建出另一套相似耦合，只是目录更干净。

因此，当前最优路径是：

```text
先拆层 -> 再迁移 -> 再清理 -> 最后再决定是否独立 core 仓库
```

而不是：

```text
立即重写 -> 暂停现有交付 -> 在新仓库里重新踩同一类边界问题
```

---

## 11. 后续执行项

### P0

- 建立 `PackDefinition` 与 `GateEvaluator` 接口
- 冻结核心层新增角色/阶段语义
- 输出 Phase01 pack 迁移清单

### P1

- 将 `phase01_role_identity_prompt` 外迁到 pack
- 将默认 deliverable path 外迁到 pack
- 将 `fallback_next_role` 外迁为 task graph 配置

### P2

- 清理 legacy role enum
- 迁移 report-viewer 为 pack-aware 模型
- 评估是否在 Phase 4 前拆出独立 `core-engine` crate
