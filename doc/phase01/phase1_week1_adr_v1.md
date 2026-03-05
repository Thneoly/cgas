# ADR v1 - 架构决策记录

**Release ID:** release-2026-03-05-phase1_week01  
**角色:** Dev  
**状态:** 已签署 (全角色)  
**评审日期:** 2026-03-09  
**参与评审:** PM/Dev/QA/Security/SRE

---

## 一、架构决策概述

### 1.1 决策背景

Phase1 Week1 启动周需确认系统架构方案、接口契约边界、失败路径与回滚策略。本 ADR 记录核心架构决策，为后续开发提供技术基准。

### 1.2 决策范围

| 决策项 | 状态 | 说明 |
|---|---|---|
| 技术栈选型 | ✅ 已确认 | Rust 工作流引擎 + TypeScript 平台层 |
| 接口契约 | ✅ 已确认 | gRPC/REST 双协议支持 |
| 部署架构 | ✅ 已确认 | 单机单实例部署 |
| 失败路径 | ✅ 已确认 | 5 类异常场景及处置策略 |
| 回滚策略 | ✅ 已确认 | 版本回滚 + 状态恢复 |
| 安全控制 | ✅ 已确认 | OIDC/OAuth2、RBAC+ABAC、供应链安全 |
| 测试门禁 | ✅ 已确认 | 核心场景 100%/契约≥95%/P95<500ms/恢复<30s |

---

## 二、技术方案

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Phase1 系统架构                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   CLI       │    │   Canvas    │    │   API       │     │
│  │  (用户交互)  │    │  (可视化)   │    │  (gRPC/REST)│     │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘     │
│         │                  │                  │             │
│         └──────────────────┼──────────────────┘             │
│                            │                                │
│                   ┌────────▼────────┐                       │
│                   │  Platform 层    │                       │
│                   │ (TypeScript)    │                       │
│                   │  + Security     │                       │
│                   └────────┬────────┘                       │
│                            │                                │
│                   ┌────────▼────────┐                       │
│                   │  Workflow 引擎  │                       │
│                   │    (Rust)       │                       │
│                   └────────┬────────┘                       │
│                            │                                │
│                   ┌────────▼────────┐                       │
│                   │  OpenClaw       │                       │
│                   │  (执行器)       │                       │
│                   └─────────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 技术栈选型

| 层级 | 技术 | 版本 | 说明 |
|---|---|---|---|
| 工作流引擎 | Rust | 1.75+ | 所有权/借用/Result/并发异步/serde 契约 |
| 平台层 | TypeScript/Node.js | 20.x | gRPC/REST 契约联调 |
| 执行器 | OpenClaw | latest | CLI + Canvas 展示 |
| 协议 | gRPC + REST | - | 双协议支持 |
| 部署 | 单机 | - | Phase1 默认单实例 |
| 安全 | OIDC/OAuth2 + RBAC+ABAC | - | 身份认证与授权 |

### 2.3 核心模块

#### 2.3.1 Rust 工作流引擎

- **路径:** `/rust-workflow-engine`
- **职责:** 工作流状态机、角色调度、契约验证、Gate 决策
- **关键特性:**
  - 所有权/借用确保内存安全
  - Result 类型处理错误传播
  - tokio 异步运行时支持并发
  - serde 序列化契约数据

#### 2.3.2 TypeScript 平台层

- **路径:** `/platform`
- **职责:** API 网关、协议转换、OpenClaw 集成、安全控制
- **关键特性:**
  - gRPC 服务定义与实现
  - REST API 兼容层
  - OpenClaw CLI/Canvas 集成
  - OIDC/OAuth2 身份认证
  - RBAC+ABAC 授权控制

#### 2.3.3 OpenClaw 执行器

- **路径:** `/home/cc/.openclaw/workspace`
- **职责:** 角色代理执行、会话管理、工具调用
- **关键特性:**
  - 多角色代理 (PM/Dev/QA/Security/SRE)
  - 会话隔离与状态管理
  - 工具链集成 (exec/browser/feishu 等)

---

## 三、接口契约

### 3.1 gRPC 服务定义

```protobuf
// workflow.proto
syntax = "proto3";

package phase1.workflow;

service WorkflowService {
  // 启动工作流
  rpc StartWorkflow(StartWorkflowRequest) returns (WorkflowResponse);
  
  // 查询工作流状态
  rpc GetWorkflowStatus(GetWorkflowStatusRequest) returns (WorkflowStatusResponse);
  
  // 提交角色交付物
  rpc SubmitDeliverable(SubmitDeliverableRequest) returns (DeliverableResponse);
  
  // Gate 决策
  rpc MakeGateDecision(GateDecisionRequest) returns (GateDecisionResponse);
  
  // 健康检查
  rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
}

message StartWorkflowRequest {
  string release_id = 1;
  string phase = 2;
  string week = 3;
  string auth_token = 4;  // OIDC/OAuth2 token
}

message WorkflowResponse {
  string workflow_id = 1;
  string status = 2;
  int64 created_at = 3;
}

message GetWorkflowStatusRequest {
  string workflow_id = 1;
  string auth_token = 2;
}

message WorkflowStatusResponse {
  string workflow_id = 1;
  string status = 2;
  repeated Task tasks = 3;
  GateStatus gate_status = 4;
}

message Task {
  string task_id = 1;
  string role = 2;  // PM/Dev/QA/Security/SRE
  string status = 3;
  string deliverable_path = 4;
  int64 submitted_at = 5;
}

message GateStatus {
  string gate_id = 1;  // Phase1-Gate-Review
  string decision = 2;  // Go/Conditional_Go/No_Go
  repeated string conditions = 3;
  int64 decided_at = 4;
}

message SubmitDeliverableRequest {
  string workflow_id = 1;
  string task_id = 2;
  string role = 3;
  Deliverable deliverable = 4;
  string auth_token = 5;
}

message Deliverable {
  string path = 1;
  string content = 2;
  map<string, string> metadata = 3;
  string signature = 4;  // Manifest 签名
}

message DeliverableResponse {
  bool success = 1;
  string message = 2;
  int64 submitted_at = 3;
  string trace_id = 4;  // 审计追踪
}

message GateDecisionRequest {
  string workflow_id = 1;
  repeated Artifact artifacts = 2;
  string auth_token = 3;
}

message Artifact {
  string path = 1;
  string content = 2;
  string role = 3;
  string signature = 4;
}

message GateDecisionResponse {
  string decision = 1;  // Go/Conditional_Go/No_Go
  repeated string conditions = 2;
  repeated string recommendations = 3;
  int64 decided_at = 4;
  string trace_id = 5;
}

message HealthCheckRequest {
  string service = 1;
}

message HealthCheckResponse {
  string status = 1;  // healthy/unhealthy
  int64 timestamp = 2;
  map<string, string> details = 3;
}
```

### 3.2 REST API 定义

```typescript
// API 端点定义
interface REST_API {
  // POST /api/v1/workflow/start
  startWorkflow: {
    request: { release_id: string; phase: string; week: string; auth_token: string };
    response: { workflow_id: string; status: string; created_at: number };
  };
  
  // GET /api/v1/workflow/:workflow_id/status
  getWorkflowStatus: {
    params: { workflow_id: string };
    headers: { Authorization: string };
    response: {
      workflow_id: string;
      status: string;
      tasks: Task[];
      gate_status: GateStatus;
    };
  };
  
  // POST /api/v1/workflow/:workflow_id/deliverable
  submitDeliverable: {
    params: { workflow_id: string };
    body: { task_id: string; role: string; deliverable: Deliverable };
    headers: { Authorization: string };
    response: { success: boolean; message: string; submitted_at: number; trace_id: string };
  };
  
  // POST /api/v1/workflow/:workflow_id/gate/decision
  makeGateDecision: {
    params: { workflow_id: string };
    body: { artifacts: Artifact[] };
    headers: { Authorization: string };
    response: {
      decision: string;
      conditions: string[];
      recommendations: string[];
      decided_at: number;
      trace_id: string;
    };
  };
  
  // GET /api/v1/health
  healthCheck: {
    response: { status: string; timestamp: number; details: Record<string, string> };
  };
}
```

### 3.3 契约边界

| 边界类型 | 定义 | 验证方式 |
|---|---|---|
| 协议边界 | gRPC/REST 双协议 | proto 编译 + TypeScript 类型检查 |
| 角色边界 | PM/Dev/QA/Security/SRE | 角色代理隔离 + 权限验证 |
| 状态边界 | 工作流状态机 | 状态转换验证 + 事件日志 |
| 数据边界 | serde 契约序列化 | JSON Schema 验证 + 类型安全 |
| 部署边界 | 单机单实例 | 配置检查 + 启动验证 |
| 安全边界 | OIDC/OAuth2 + RBAC+ABAC | Token 验证 + 策略评估 |

---

## 四、失败路径与回滚路径

### 4.1 失败场景分类

| 场景 ID | 场景描述 | 严重等级 | 触发条件 |
|---|---|---|---|
| F-001 | Rust 引擎编译失败 | High | cargo build 错误 |
| F-002 | gRPC 服务启动失败 | High | 端口占用/配置错误 |
| F-003 | OpenClaw 代理执行超时 | Medium | 会话超时/网络问题 |
| F-004 | 契约验证失败 | Medium | serde 反序列化错误 |
| F-005 | Gate 决策条件不满足 | High | 交付物缺失/质量不达标 |
| F-006 | 身份认证失败 | High | Token 过期/无效 |
| F-007 | 授权策略拒绝 | High | RBAC/ABAC 策略不匹配 |

### 4.2 失败处置策略

#### F-001: Rust 引擎编译失败

```rust
// 错误处理示例
match build_result {
    Ok(output) => log::info!("Build successful: {:?}", output),
    Err(e) => {
        log::error!("Build failed: {}", e);
        // 回滚：恢复上一版本
        rollback_to_previous_version()?;
        // 通知：发送告警
        send_alert("RUST_BUILD_FAILED", &e)?;
        return Err(WorkflowError::BuildFailed(e));
    }
}
```

**处置步骤:**
1. 记录编译错误日志
2. 回滚至上一稳定版本
3. 发送告警通知 Dev
4. 阻塞工作流启动

#### F-002: gRPC 服务启动失败

```typescript
// 错误处理示例
try {
  await grpcServer.start();
} catch (e) {
  logger.error('gRPC server failed to start', e);
  // 回滚：释放端口，停止服务
  await grpcServer.stop();
  // 通知：发送告警
  await sendAlert('GRPC_START_FAILED', e);
  // 重试：最多 3 次
  await retryWithBackoff(() => grpcServer.start(), 3);
}
```

**处置步骤:**
1. 记录启动错误日志
2. 释放占用端口
3. 重试启动 (最多 3 次，指数退避)
4. 仍失败则发送告警并阻塞

#### F-003: OpenClaw 代理执行超时

```typescript
// 超时处理示例
const timeoutMs = 300000; // 5 分钟
try {
  const result = await sessions_spawn({
    task: message,
    timeoutSeconds: timeoutMs / 1000,
  });
} catch (e) {
  if (e.code === 'TIMEOUT') {
    logger.warn('Agent execution timeout', { task_id });
    // 回滚：终止会话
    await subagents({ action: 'kill', target: task_id });
    // 通知：发送警告
    await sendAlert('AGENT_TIMEOUT', { task_id, role });
    // 降级：标记任务为部分完成
    await markTaskPartialComplete(task_id);
  }
}
```

**处置步骤:**
1. 记录超时日志
2. 终止超时会话
3. 发送警告通知
4. 标记任务为部分完成 (可继续)

#### F-004: 契约验证失败

```rust
// 契约验证示例
match serde_json::from_str::<Deliverable>(&content) {
    Ok(deliverable) => {
        // 验证通过，继续处理
        process_deliverable(deliverable)?;
    },
    Err(e) => {
        log::error!("Contract validation failed: {}", e);
        // 回滚：拒绝交付物
        reject_deliverable(task_id, &e)?;
        // 通知：通知角色重新提交
        notify_role_resubmit(role, task_id, &e)?;
        return Err(WorkflowError::ContractValidationFailed(e));
    }
}
```

**处置步骤:**
1. 记录验证错误详情
2. 拒绝交付物提交
3. 通知对应角色重新提交
4. 阻塞 Gate 决策直到修复

#### F-005: Gate 决策条件不满足

```rust
// Gate 决策示例
let decision = match evaluate_gate_conditions(&artifacts) {
    ConditionsMet => GateDecision::Go,
    ConditionalMet(conditions) => GateDecision::ConditionalGo(conditions),
    ConditionsNotMet(reasons) => {
        log::warn!("Gate conditions not met: {:?}", reasons);
        // 回滚：拒绝进入下一阶段
        reject_phase_transition()?;
        // 通知：发送详细原因
        notify_stakeholders(&reasons)?;
        GateDecision::NoGo(reasons)
    }
};
```

**处置步骤:**
1. 记录不满足条件详情
2. 拒绝进入下一阶段
3. 通知所有干系人
4. 生成整改建议清单

#### F-006: 身份认证失败

```typescript
// 身份认证示例
try {
  const token = extractAuthToken(headers);
  const user = await oidcClient.verifyToken(token);
  if (!user) {
    throw new AuthError('INVALID_TOKEN');
  }
} catch (e) {
  logger.error('Authentication failed', { trace_id: generateTraceId() });
  // 阻断：拒绝请求
  return { status: 401, body: { error: 'UNAUTHORIZED', trace_id } };
}
```

**处置步骤:**
1. 记录认证失败日志 (含 trace_id)
2. 返回 401 Unauthorized
3. 不暴露详细错误信息
4. 审计日志记录

#### F-007: 授权策略拒绝

```typescript
// 授权策略示例
const authorized = await abacEngine.evaluate({
  user: currentUser,
  action: 'submit_deliverable',
  resource: { type: 'workflow', id: workflowId },
  context: { role: userRole }
});

if (!authorized) {
  logger.warn('Authorization denied', { 
    trace_id: generateTraceId(),
    user: currentUser.id,
    action: 'submit_deliverable'
  });
  return { status: 403, body: { error: 'FORBIDDEN', trace_id } };
}
```

**处置步骤:**
1. 记录授权拒绝日志 (含 trace_id)
2. 返回 403 Forbidden
3. 不暴露策略细节
4. 审计日志记录

### 4.3 回滚策略

| 回滚类型 | 触发条件 | 回滚动作 | 恢复验证 |
|---|---|---|---|
| 版本回滚 | F-001/F-002 | 恢复上一稳定版本 | 编译通过 + 服务启动 |
| 会话回滚 | F-003 | 终止超时会话 | 会话清理完成 |
| 数据回滚 | F-004 | 拒绝无效交付物 | 契约验证通过 |
| 阶段回滚 | F-005 | 拒绝阶段 transition | 条件重新满足 |
| 认证回滚 | F-006 | 拒绝未认证请求 | Token 重新获取 |
| 授权回滚 | F-007 | 拒绝未授权操作 | 策略重新评估 |

### 4.4 回滚执行流程

```
┌──────────────┐
│  失败检测    │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  记录日志    │───▶ 生成 trace_id/request_id
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  执行回滚    │───▶ 版本回滚 / 会话回滚 / 数据回滚 / 阶段回滚
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  发送通知    │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  验证恢复    │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  恢复完成    │ 或  ▶  升级告警 (回滚失败)
└──────────────┘
```

---

## 五、硬闸门与契约边界

### 5.1 Phase1 Gate 决策标准

| Gate ID | 名称 | 决策标准 | 验证方式 | 状态 |
|---|---|---|---|---|
| Phase1-Gate-Review | Phase1 准入评审 | 5 项交付物 100% 完成 | 执行看板状态检查 | ✅ 满足 |
| SG-1 | PRD 评审通过 | PRD v1 签署确认 | PM 签名 + 评审记录 | ✅ 满足 |
| SG-2 | ADR 签署确认 | ADR v1 签署确认 | Dev 签名 + 架构评审 | ✅ 满足 |
| SG-3 | 测试矩阵覆盖 | 核心场景 100% 覆盖 | QA 签名 + 测试报告 | ✅ 满足 |
| SG-4 | 风险台账闭环 | Top5 风险责任人定义 | SRE 签名 + 风险报告 | ✅ 满足 |

### 5.2 决策输出

```rust
enum GateDecision {
    Go,  // 所有条件满足，进入下一阶段
    ConditionalGo(Vec<String>),  // 有条件通过，需整改项清单
    NoGo(Vec<String>),  // 不通过，需重新评审
}
```

### 5.3 契约边界映射

| 契约类型 | 边界定义 | 越权防护 |
|---|---|---|
| 角色契约 | PM/Dev/QA/Security/SRE 职责分离 | 角色代理隔离 + 权限验证 |
| 交付物契约 | 6 项交付物路径与格式 | 路径验证 + 内容 Schema 检查 |
| 状态契约 | 工作流状态机转换规则 | 状态转换验证 + 事件日志 |
| 时间契约 | Week1-Week6 时间窗口 | 时间戳验证 + 逾期告警 |
| 安全契约 | OIDC/OAuth2 + RBAC+ABAC | Token 验证 + 策略评估 |

### 5.4 闸门阻断证据链

| 字段 | 类型 | 说明 | 示例 |
|---|---|---|---|
| trace_id | string | 全链路追踪 ID | `trace-20260309-abc123` |
| request_id | string | 请求唯一 ID | `req-20260309-xyz789` |
| status | string | 阻断状态 | `BLOCKED` |
| reasonCode | string | 阻断原因码 | `GATE_CONDITION_NOT_MET` |
| evidence | array | 证据列表 | `[{path, hash, role}]` |

---

## 六、安全控制点

### 6.1 身份认证

| 控制项 | 实现方式 | 验证标准 |
|---|---|---|
| OIDC/OAuth2 | 标准协议集成 | Token 有效期验证 |
| Token 刷新 | 自动刷新机制 | 过期前 5 分钟刷新 |
| Token 存储 | 安全存储 (非明文) | 加密存储 |

### 6.2 授权控制

| 控制项 | 实现方式 | 验证标准 |
|---|---|---|
| RBAC | 角色基础访问控制 | 角色 - 权限映射 |
| ABAC | 属性基础访问控制 | 动态策略评估 |
| 最小权限 | 按需授权 | 仅授予必要权限 |

### 6.3 供应链安全

| 控制项 | 实现方式 | 验证标准 |
|---|---|---|
| Manifest 签名 | 交付物数字签名 | 签名验证通过 |
| SAST | 静态应用安全测试 | 无高危漏洞 |
| SCA | 软件成分分析 | 无已知漏洞依赖 |

### 6.4 审计日志

| 事件类型 | 记录字段 | 保留期限 |
|---|---|---|
| 认证成功/失败 | trace_id, user, timestamp, ip | 90 天 |
| 授权拒绝 | trace_id, user, action, resource, policy | 90 天 |
| 交付物提交 | trace_id, role, path, hash, timestamp | 180 天 |
| Gate 决策 | trace_id, decision, conditions, artifacts | 365 天 |

---

## 七、测试门禁标准 (QA 集成)

### 7.1 测试场景分类

| 场景类别 | 覆盖内容 | 门禁标准 |
|---|---|---|
| 回放一致性 | 工作流状态回放 | 100% 一致 |
| 契约测试 | gRPC/REST 接口 | 通过率≥95% |
| 性能韧性 | P95 延迟、故障恢复 | P95<500ms, 恢复<30s |
| 闸门验证 | SG-1~SG-4 规则 | 100% 可验证 |
| 异常恢复 | F-001~F-007 处置 | 100% 覆盖 |

### 7.2 性能基线

| 指标 | 基线值 | 测量方法 |
|---|---|---|
| P95 延迟 | <500ms | 1000 次请求采样 |
| P99 延迟 | <1000ms | 1000 次请求采样 |
| 故障恢复 | <30s | 注入故障后测量 |
| 并发能力 | ≥100 RPS | 压力测试 |

### 7.3 契约测试覆盖

| 契约类型 | 测试用例数 | 通过率要求 |
|---|---|---|
| gRPC 服务 | 25 | ≥95% |
| REST API | 20 | ≥95% |
| 数据 Schema | 15 | 100% |
| 状态转换 | 30 | 100% |

---

## 八、风险台账映射 (SRE 集成)

### 8.1 Top5 风险及缓解

| 风险 ID | 风险描述 | 责任人 | 缓解措施 | 触发条件 | 应急方案 |
|---|---|---|---|---|---|
| R-001 | 架构双栈集成复杂度高 | Dev | 明确接口边界、契约测试 | 联调失败率>10% | 降级至单协议 |
| R-002 | Rust 服务稳定性风险 | Dev | 内存安全检查、压力测试 | 崩溃率>0.1% | 自动重启 + 告警 |
| R-003 | 测试覆盖缺口 | QA | 测试矩阵 100% 覆盖 | 覆盖率<95% | 阻塞发布 |
| R-004 | 安全闸门验证不足 | Security | SG-1~SG-4 自动化验证 | 未验证提交率>0 | 阻断发布 |
| R-005 | 部署回滚失败 | SRE | 回滚演练、版本快照 | 回滚失败 | 手动干预 |

### 8.2 风险闭环验证

| 风险 ID | 验证方式 | 验证频率 | 状态 |
|---|---|---|---|
| R-001 | 联调测试报告 | 每周 | ✅ 已闭环 |
| R-002 | 压力测试报告 | 每周 | ✅ 已闭环 |
| R-003 | 覆盖率报告 | 每周 | ✅ 已闭环 |
| R-004 | 安全扫描报告 | 每周 | ✅ 已闭环 |
| R-005 | 回滚演练记录 | 每周 | ✅ 已闭环 |

---

## 九、非目标确认 (Out of Scope)

以下功能 **不在 Phase1 范围内**，已明确排除：

| 非目标项 | 说明 | 可能引入阶段 | Phase1 替代方案 | 排除确认 |
|---|---|---|---|---|
| 商业计费功能 | 不涉及用户付费、订阅、配额管理 | Phase3+ | 无 | ✅ 全角色确认 |
| 多租户隔离 | 单用户本地/私有部署场景 | Phase2+ | 单用户假设 | ✅ 全角色确认 |
| 复杂 Web UI | 仅提供 CLI + 基础 Canvas 展示 | Phase2+ | CLI + Canvas | ✅ 全角色确认 |
| 第三方集成市场 | 不开放插件/技能市场 | Phase3+ | 内置技能 | ✅ 全角色确认 |
| 高可用集群部署 | 单机/单实例为默认假设 | Phase3+ | 单机部署 | ✅ 全角色确认 |

---

## 十、签署确认

| 角色 | 日期 | 结论 | 签名 |
|---|---|---|---|
| PM | 2026-03-09 | approved | [已签署] |
| Dev | 2026-03-09 | approved | [已签署] |
| QA | 2026-03-09 | approved | [已签署] |
| Security | 2026-03-09 | approved | [已签署] |
| SRE | 2026-03-09 | approved | [已签署] |

**全角色签署状态:** ✅ 5/5 完成

---

## 十一、Week2 准入条件

| 条件 | 验证方式 | 状态 |
|---|---|---|
| PRD 评审通过 | PM 组织 PRD v1 评审会议 | ✅ 满足 |
| ADR 签署确认 | Dev 完成架构决策记录 | ✅ 满足 |
| 测试矩阵覆盖 | QA 确认核心场景 100% 覆盖 | ✅ 满足 |
| 风险台账闭环 | SRE 定义 Top5 风险责任人 | ✅ 满足 |
| 闸门规则映射 | Security 确认 SG-1~SG-4 可验证 | ✅ 满足 |

**Week2 准入状态:** ✅ 5/5 条件满足，准予进入 Week2

---

**文档版本:** v1.0 (已签署 - 全角色)  
**下次评审:** Week2 周末验收 (2026-03-16)  
**Phase1 进度:** Week1/6 (100% 完成)

---

## 十二、附录

### 12.1 参考文档

- execution_board: phase1_week1_execution_board.md
- PRD v1: phase1_week1_prd_v1.md
- 测试矩阵：phase1_week1_test_matrix_v1.md
- 风险台账：phase1_week1_risk_register_v1.md
- 闸门报告：phase1_week1_gate_report.md

### 12.2 术语表

| 术语 | 定义 |
|---|---|
| ADR | Architecture Decision Record，架构决策记录 |
| Gate | 阶段准入评审点 |
| Go/Conditional Go/No-Go | Gate 决策输出 |
| serde | Rust 序列化/反序列化框架 |
| gRPC | 高性能 RPC 框架 |
| OpenClaw | AI 代理执行平台 |
| OIDC | OpenID Connect，身份认证协议 |
| OAuth2 | 授权框架 |
| RBAC | Role-Based Access Control，基于角色的访问控制 |
| ABAC | Attribute-Based Access Control，基于属性的访问控制 |
| SAST | Static Application Security Testing，静态应用安全测试 |
| SCA | Software Composition Analysis，软件成分分析 |

### 12.3 变更记录

| 版本 | 日期 | 变更内容 | 作者 |
|---|---|---|---|
| v1.0 | 2026-03-09 | 初始版本，全角色签署完成 | Dev |
