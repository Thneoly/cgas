# 提交闸门规则 v1.0（Phase 1）

## 1. 目标
确保所有状态提交均经过独立验证，防止未验证提交。

## 2. 核心规则

| 规则ID | 规则 | 失败动作 |
|---|---|---|
| SG-1 | 未经 Verifier 的提交请求必须拒绝 | 阻断 + 审计记录 |
| SG-2 | Verifier mismatch 必须触发 `REVERT` | 回退未提交状态 |
| SG-3 | 提交前必须具备 `result_hash` 与 `trace_hash` | 阻断 + reasonCode |
| SG-4 | 阻断事件必须包含 `status/reasonCode/evidence` | 否则判定流程不合规 |

## 3. 判定口径
- 指标：`未验证提交率 = unverified_submissions / total_submissions`
- Phase 1 要求：`未验证提交率 = 0`
- 窗口与连续性：按门禁口径执行（14 天 + 连续双周关卡）

## 4. 例外策略
- 默认不允许绕过提交闸门。
- 如发生 `Conditional Go`，必须附：审批人、范围、有效期、补偿控制。

## 5. 审计要求
- 每次阻断必须记录 trace_id/request_id。
- 阻断证据可导出并进入 `gate-report`。
