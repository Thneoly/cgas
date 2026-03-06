# Phase 2 接口契约规范 v1.0

**版本**: v1.0  
**日期**: 2026-04-08  
**责任人**: Dev + 架构师  
**状态**: 📋 草案评审中  
**release_id**: release-2026-04-08-phase2_week02  
**参与角色**: PM, Dev, QA, SRE, Security

---

## 1. 目标

定义 Phase 2 接口契约规范，确保：
- 向后兼容性 (Phase 1 接口冻结)
- 新增接口标准化
- 接口版本化管理
- 接口变更可追溯

---

## 2. 设计原则

| 原则 | 描述 | 验证方法 |
|---|---|---|
| **向后兼容** | Phase 1 接口完全兼容 | 兼容性测试 |
| **接口冻结** | 核心接口冻结，扩展需评审 | 变更控制 |
| **版本化管理** | 接口版本清晰标识 | 版本检查 |
| **契约优先** | 先定义接口，后实现 | Spec 评审 |

---

## 3. Phase 1 接口继承

### 3.1 冻结接口清单

| 接口 | 冻结版本 | 冻结周次 | 状态 |
|---|---|---|---|
| ExecuteRequest | Week 2 | Phase 1 | ✅ 冻结 |
| ExecutionResult | Week 2 | Phase 1 | ✅ 冻结 |
| VerifyRequest | Week 3 | Phase 1 | ✅ 冻结 |
| VerifyResponse | Week 3 | Phase 1 | ✅ 冻结 |
| CommitRequest | Week 4 | Phase 1 | ✅ 冻结 |
| CommitResponse | Week 4 | Phase 1 | ✅ 冻结 |

### 3.2 冻结接口契约

**ExecuteRequest (冻结)**:
```rust
pub struct ExecuteRequest {
    pub trace_id: String,
    pub execution_id: String,
    pub instruction_type: InstructionType,
    pub payload: InstructionPayload,
    pub timestamp: String,
}
```

**ExecutionResult (冻结)**:
```rust
pub struct ExecutionResult {
    pub trace_id: String,
    pub execution_id: String,
    pub status: ExecutionStatus,
    pub state_diff: Vec<StateDiffOperation>,
    pub result_hash: String,
    pub state_diff_hash: String,
    pub timestamp: String,
}
```

### 3.3 兼容性保证

| 接口 | Phase 1 | Phase 2 | 兼容性 |
|---|---|---|---|
| ExecuteRequest | v1.0 | v1.0 (继承) | ✅ 完全兼容 |
| ExecutionResult | v1.0 | v1.0 (继承) | ✅ 完全兼容 |
| VerifyRequest | v1.0 | v1.0 (继承) | ✅ 完全兼容 |
| VerifyResponse | v1.0 | v1.0 (继承) | ✅ 完全兼容 |
| CommitRequest | v1.0 | v1.0 (继承) | ✅ 完全兼容 |
| CommitResponse | v1.0 | v1.0 (继承) | ✅ 完全兼容 |

---

## 4. Phase 2 新增接口

### 4.1 Batch 接口

| 接口 | 版本 | 描述 | 状态 |
|---|---|---|---|
| BatchExecuteRequest | v1.0 | Batch 执行请求 | 📋 新增 |
| BatchExecuteResult | v1.0 | Batch 执行结果 | 📋 新增 |
| BatchStatus | v1.0 | Batch 状态枚举 | 📋 新增 |

**BatchExecuteRequest**:
```rust
pub struct BatchExecuteRequest {
    pub trace_id: String,
    pub batch_id: String,
    pub instructions: Vec<ExecuteRequest>,
    pub atomic: bool,
    pub timestamp: String,
}
```

**BatchExecuteResult**:
```rust
pub struct BatchExecuteResult {
    pub trace_id: String,
    pub batch_id: String,
    pub status: BatchStatus,
    pub results: Vec<ExecutionResult>,
    pub batch_hash: String,
    pub timestamp: String,
}
```

### 4.2 Transaction 接口

| 接口 | 版本 | 描述 | 状态 |
|---|---|---|---|
| BeginTransactionRequest | v1.0 | 事务开始请求 | 📋 新增 |
| BeginTransactionResponse | v1.0 | 事务开始响应 | 📋 新增 |
| CommitTransactionRequest | v1.0 | 事务提交请求 | 📋 新增 |
| CommitTransactionResponse | v1.0 | 事务提交响应 | 📋 新增 |
| RollbackTransactionRequest | v1.0 | 事务回滚请求 | 📋 新增 |
| RollbackTransactionResponse | v1.0 | 事务回滚响应 | 📋 新增 |
| IsolationLevel | v1.0 | 隔离级别枚举 | 📋 新增 |
| TransactionStatus | v1.0 | 事务状态枚举 | 📋 新增 |

### 4.3 gRPC 服务扩展

**Phase 1 服务 (冻结)**:
```protobuf
// executor.proto (Week 2 冻结)
service ExecutorService {
  rpc Execute(ExecuteRequest) returns (ExecutionResult);
}

// verifier.proto (Week 3 冻结)
service VerifierService {
  rpc Verify(VerifyRequest) returns (VerifyResponse);
  rpc BatchVerify(BatchVerifyRequest) returns (BatchVerifyResponse);
}

// commit.proto (Week 4 冻结)
service CommitService {
  rpc Commit(CommitRequest) returns (CommitResponse);
}
```

**Phase 2 新增服务**:
```protobuf
// batch.proto (Phase 2 新增)
service BatchService {
  rpc BatchExecute(BatchExecuteRequest) returns (BatchExecuteResult);
}

// transaction.proto (Phase 2 新增)
service TransactionService {
  rpc BeginTransaction(BeginTransactionRequest) returns (BeginTransactionResponse);
  rpc CommitTransaction(CommitTransactionRequest) returns (CommitTransactionResponse);
  rpc RollbackTransaction(RollbackTransactionRequest) returns (RollbackTransactionResponse);
}
```

---

## 5. 接口版本化管理

### 5.1 版本号规范

```
接口版本 = {主版本}.{次版本}.{修订版本}
```

| 版本变更 | 说明 | 兼容性 |
|---|---|---|
| 主版本变更 | 不兼容变更 | ❌ 不兼容 |
| 次版本变更 | 向后兼容的功能新增 | ✅ 兼容 |
| 修订版本变更 | 向后兼容的问题修复 | ✅ 兼容 |

### 5.2 接口弃用策略

**弃用流程**:
1. 标记 `@deprecated`
2. 公告弃用 (提前 2 周)
3. 提供迁移指南
4. 保留至少 1 个 Phase 周期

**弃用示例**:
```rust
/// @deprecated 使用 BatchExecuteRequest 替代
pub struct LegacyBatchRequest {
    // ...
}
```

---

## 6. 接口变更控制

### 6.1 变更分类

| 变更类型 | 描述 | 审批要求 |
|---|---|---|
| Class A: 不兼容变更 | 修改/删除已冻结接口 | 门禁官 + 四方 |
| Class B: 兼容新增 | 新增接口/字段 | PM + Dev |
| Class C: 文档修正 | 文档/注释更新 | Dev |

### 6.2 变更流程

```
┌─────────────┐
│ 变更申请    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ 影响分析    │ ← 兼容性检查
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ 变更评审    │ ← 根据分类审批
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ 变更实施    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ 变更验证    │ ← 兼容性测试
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ 变更发布    │
└─────────────┘
```

### 6.3 变更请求模板

```markdown
## 变更请求：ICR-XXX

**接口名称**: [接口名]

**变更类型**: Class A/B/C

**变更描述**: [详细描述]

**变更原因**: [为什么需要变更]

**兼容性影响**: [兼容/不兼容]

**迁移方案**: [如有不兼容，提供迁移方案]

**风险评估**: [低/中/高]

**审批人**: [待填写]
```

---

## 7. 接口测试要求

### 7.1 接口测试覆盖

| 测试类型 | 覆盖要求 | 验证方法 |
|---|---|---|
| 契约测试 | 100% 接口 | Pact/OpenAPI |
| 兼容性测试 | 100% 冻结接口 | 回归测试 |
| 集成测试 | 100% 服务间调用 | E2E 测试 |

### 7.2 契约测试示例

```rust
#[test]
fn test_batch_execute_request_contract() {
    // 验证字段存在性
    let request = BatchExecuteRequest {
        trace_id: "trace_1".to_string(),
        batch_id: "batch_1".to_string(),
        instructions: vec![],
        atomic: true,
        timestamp: "2026-04-08T10:00:00Z".to_string(),
    };
    
    // 验证序列化
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("trace_id"));
    assert!(json.contains("batch_id"));
    assert!(json.contains("instructions"));
    assert!(json.contains("atomic"));
    assert!(json.contains("timestamp"));
    
    // 验证反序列化
    let parsed: BatchExecuteRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.trace_id, request.trace_id);
    assert_eq!(parsed.batch_id, request.batch_id);
}
```

---

## 8. 接口文档规范

### 8.1 文档要求

| 要求 | 描述 | 验证方法 |
|---|---|---|
| 完整字段说明 | 每个字段有说明 | 文档审查 |
| 示例代码 | 每个接口有示例 | 文档审查 |
| 错误码说明 | 所有错误码有说明 | 文档审查 |
| 版本历史 | 变更记录完整 | 版本检查 |

### 8.2 接口文档模板

```markdown
## 接口名：BatchExecute

**版本**: v1.0

**描述**: 执行批量指令

**请求**:
```rust
pub struct BatchExecuteRequest {
    /// Batch 级 trace ID
    pub trace_id: String,
    // ... 其他字段
}
```

**响应**:
```rust
pub struct BatchExecuteResult {
    /// Batch 级 trace ID
    pub trace_id: String,
    // ... 其他字段
}
```

**错误码**:
| 错误码 | 说明 |
|---|---|
| 1001 | 验证失败 |
| 1002 | 空 Batch |

**示例**:
```rust
let request = BatchExecuteRequest { ... };
let result = client.batch_execute(request).await?;
```
```

---

## 9. Spec 符合性验证

### 9.1 验证清单

| 验证项 | 验证方法 | 责任人 | 状态 |
|---|---|---|---|
| Phase 1 接口兼容性 | 兼容性测试 | Dev+QA | 📋 待验证 |
| Phase 2 新增接口 | 代码审查 | Dev | 📋 待验证 |
| gRPC 服务定义 | Proto 审查 | Dev | 📋 待验证 |
| 接口文档完整性 | 文档审查 | Tech Writer | 📋 待验证 |
| 契约测试覆盖 | 测试报告 | QA | 📋 待验证 |

### 9.2 Spec 版本控制

| 版本 | 变更日期 | 变更内容 | 批准人 |
|---|---|---|---|
| v1.0 | 2026-04-08 | 初始版本 | 📋 待批准 |

---

## 10. 附录

### 10.1 术语表

| 术语 | 定义 |
|---|---|
| 接口契约 | 接口输入输出定义 |
| 向后兼容 | 新版本兼容旧版本 |
| 接口冻结 | 接口不再变更 |
| 契约测试 | 验证接口契约的测试 |

### 10.2 参考文档

- Phase 1 最小指令集规范 v1
- Phase 2 PRD v2
- Phase 2 ADR v4
- Phase 2 Batch 规范 v1
- Phase 2 Transaction 规范 v1

---

**文档状态**: 📋 草案评审中  
**评审计划**: Week 2-T4 接口契约评审会议  
**责任人**: Dev + 架构师  
**保管**: 项目文档库
