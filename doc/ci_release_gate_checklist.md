# CI/发布前门禁检查清单（机器可执行版）

## 1. 目标
将评审门禁（第 8 章）转为可自动判定规则，输出统一 `gate-report.json`，作为合并与发布阻断条件。

---

## 2. 执行时机
- `pre-merge`：PR 合并前，验证规则完整性、样本量、核心红线。
- `nightly`：每日回归，更新滚动窗口指标并追踪趋势。
- `pre-release`：版本发布前，执行全量门禁（含连续达标与置信约束）。

---

## 3. 输入数据契约（最小集）
来自监控/审计数据仓（或导出的聚合 JSON），字段建议如下：

| 字段 | 类型 | 说明 |
|---|---|---|
| `window_days` | number | 统计窗口天数（7 或 14） |
| `total_submissions` | number | 全部提交请求数 |
| `unverified_submissions` | number | 未经 Verifier 即提交数 |
| `high_risk_requests` | number | 高危请求总数 |
| `high_risk_false_allows` | number | 高危误放行数 |
| `verify_requests` | number | 进入验证流程请求数 |
| `verify_consistent` | number | 验证一致请求数 |
| `verify_p95_ms` | number | 验证延迟 P95 |
| `high_risk_block_p95_ms` | number | 高危拦截延迟 P95 |
| `rollback_total` | number | 回滚总次数 |
| `rollback_success` | number | 回滚成功次数 |
| `policy_change_total` | number | 高危策略变更总次数 |
| `policy_change_closed_loop` | number | 完整闭环（提案-测试-灰度-回滚）次数 |
| `undeclared_cap_calls` | number | 未声明能力调用次数 |
| `undeclared_cap_blocked` | number | 被拦截的未声明能力调用次数 |
| `core_replay_samples` | number | 核心回放样本量 |
| `high_risk_samples` | number | 高危样本量 |
| `checkpoint_passes` | number | 连续通过双周关卡次数 |

### 3.1 字段映射与命名归一化（必须）
为避免实现歧义，规则引擎统一使用 `snake_case` 运行时上下文。配置文件中的 `camelCase` 键需要在加载时映射为运行时变量。

| 来源 | 原始键名 | 运行时变量（规则表达式） |
|---|---|---|
| 指标输入 JSON | `window_days` | `window_days` |
| 指标输入 JSON | `checkpoint_passes` | `checkpoint_passes` |
| 配置 YAML | `windowDays` | `window_days`（覆盖输入默认值） |
| 配置 YAML | `requiredCheckpointPasses` | `required_checkpoint_passes` |
| 配置 YAML | `isHighRiskRelease` | `is_high_risk_release` |

说明：同名字段冲突时，优先级为 `CI 参数 > 配置 YAML > 指标输入 JSON`。

---

## 4. 规则表达（可执行）

## 4.1 计算定义
- `unverified_submit_rate = unverified_submissions / total_submissions`
- `high_risk_false_allow_rate = high_risk_false_allows / high_risk_requests`
- `verify_consistency_rate = verify_consistent / verify_requests`
- `rollback_success_rate = rollback_success / rollback_total`
- `undeclared_cap_block_rate = undeclared_cap_blocked / undeclared_cap_calls`
- `policy_closed_loop_rate = policy_change_closed_loop / policy_change_total`

说明：分母为 0 时统一按“不可判定”处理，默认 `FAIL`（仅允许走例外流程）。

## 4.2 红线规则（Hard Gates）

| Gate | 规则 | 判定 |
|---|---|---|
| `gate_trust_unverified_zero` | `unverified_submissions == 0` 且 `window_days >= 14` 且 `checkpoint_passes >= 2` | 否则阻断 |
| `gate_security_false_allow` | `high_risk_samples >= 1000` 且 `high_risk_false_allow_rate <= 0.001` 且 `high_risk_false_allow_u95 <= 0.001` | 否则阻断 |
| `gate_governance_closed_loop` | `policy_change_total == 0 OR policy_closed_loop_rate == 1.0` | 否则阻断 |
| `gate_sample_minimum` | `core_replay_samples >= 200` 且（`is_high_risk_release == true` 时 `high_risk_samples >= 1000`） | 否则阻断 |

---

## 5. 置信约束计算（建议）
对比例指标使用 Wilson 上置信界（95%）作为 `u95`：

- 输入：成功数 `k`，样本数 `n`，`z = 1.96`
- 记 `p = k / n`
- 上置信界：

$$
U = \frac{p + \frac{z^2}{2n} + z\sqrt{\frac{p(1-p)}{n} + \frac{z^2}{4n^2}}}{1 + \frac{z^2}{n}}
$$

实现建议：在门禁脚本中统一计算 `high_risk_false_allow_u95`、`verify_consistency_l95`（如需要）。

---

## 6. 阶段门禁映射

| 阶段 | 必须通过规则 |
|---|---|
| Phase 0 | `gate_sample_minimum`（核心样本） |
| Phase 1 | `gate_trust_unverified_zero` + `gate_sample_minimum` |
| Phase 2 | `gate_security_false_allow` + `gate_trust_unverified_zero` |
| Phase 3 | Phase 2 全部 + `verify_p95_ms < 2000` |
| Phase 4 | Phase 3 全部 + `undeclared_cap_block_rate == 1.0` |
| Phase 5 | Phase 4 全部 + `gate_governance_closed_loop` + `rollback_success_rate >= 0.99` |

---

## 7. 失败处置（自动化动作）

| Gate 失败 | 自动动作 | 恢复条件 |
|---|---|---|
| `gate_trust_unverified_zero` | 冻结功能发布，仅允许修复分支 | 14 天窗口恢复达标 + 连续 2 个双周关卡通过 |
| `gate_security_false_allow` | 阻断发布，触发安全事件单（P0/P1） | 误放行率与 u95 均达标 |
| `gate_governance_closed_loop` | 阻断策略上线 | 补齐提案-测试-灰度-回滚证据链 |
| `gate_sample_minimum` | 阻断“通过判定”，只允许补样/修复 | 样本量达标 |

---

## 8. 输出物
- 机器输出：`gate-report.json`（见 [doc/gate_report_schema.json](gate_report_schema.json)）。
- 人类可读：`gate-report.md`（摘要 + 失败 Gate + 责任人 + SLA）。
- 每条 Gate 至少输出：`status`、`reasonCode`、`evidence`（用于审计与自动工单归因）。

---

## 9. 最小落地建议
1. 在 CI 增加 `gates` 任务（读取聚合指标 JSON）。
2. 执行规则引擎（推荐 Python/Node 单文件先行）。
3. 输出 `gate-report.json` 并作为发布阻断条件。
4. 失败时自动创建工单并附带 Gate 失败上下文。
