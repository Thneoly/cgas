# Week 6 例外审批模板（Conditional Go 专用）

## 1. 例外基本信息
- 例外编号：{{id}}
- 申请日期：{{date}}
- 申请角色：{{role}}
- 审批人：{{approver}}

## 2. 例外范围
- 涉及指标/规则：{{gate_or_metric}}
- 影响范围：{{scope}}
- 生效环境：{{env}}
- 有效期：{{effective_period}}

## 3. 补偿控制
- 控制措施 1：{{control_1}}
- 控制措施 2：{{control_2}}
- 控制措施 3：{{control_3}}

## 4. 闭环计划

| 未达标项 | 责任人 | 修复 ETA | 复验标准 |
|---|---|---|---|
| {{issue}} | {{owner}} | {{eta}} | {{verify}} |

## 5. 审批结论
- `批准 / 驳回`
- 备注：{{notes}}
