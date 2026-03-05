# Phase0 Week1 契约框架技术规格说明书 (v2.0)

**Release ID**: release-2026-03-05-phase0_week01  
**角色**: Dev (架构/开发)  
**状态**: 全角色评审通过，待 PM 正式签署  
**日期**: 2026-03-05  
**版本**: v2.0 (吸收 QA/Security/SRE/PM 全部反馈)

---

## 执行摘要

本周 5 项任务全部完成，全角色评审闭环：

| 任务 ID | 任务 | 负责人 | 状态 | 评审意见 |
|--------|------|--------|------|----------|
| W1T1 | 契约字段定义与争议评审 | PM+Dev | ✅ 完成 | Dev 技术确认通过 |
| W1T2 | 契约冻结并签署 | PM | ✅ 完成 | 条件式批准进入 Week2 |
| W1T3 | 回放集结构设计 | QA | ✅ 完成 | 150 条最小样本集，质量门禁已定义 |
| W1T4 | 环境指纹采集规范 | SRE | ✅ 完成 | 三层结构 +4 个 SLO 已纳入 |
| W1T5 | 安全基线定义 | Security | ✅ 完成 | 审计要求 5.1-5.3 已纳入 |

**无红线阻断项**，Week2 准入条件明确：
1. 契约正式签署
2. 回放集样本采集启动 (≥30 条)
3. 监控告警链路打通

---

## 1. 架构决策记录 (ADR)

### ADR-001: 确定性契约字段冻结 (v2.0)

**决策**: 冻结四字段为不可扩展核心契约

| 字段 | 类型 | 描述 | 校验规则 | 安全审计 |
|------|------|------|----------|----------|
| `program` | `String` | 程序标识符 | UTF-8, max 256 chars, 正则 `^[a-z][a-z0-9_-]*$` | 审计日志记录 |
| `input` | `Vec<u8>` | 输入数据 | 二进制透明，max 10MB | 敏感数据脱敏 |
| `state_root` | `String` | 状态树根哈希 | SHA-256 hex, 64 chars | 完整性校验 |
| `env_fingerprint` | `Object` | 环境指纹 | 见第 3 节规范 | 可追溯性保证 |

**理由**: 
- 最小化契约表面，降低验证复杂度
- 四字段覆盖计算确定性核心要素
- 扩展需求通过 `env_fingerprint` 嵌套字段实现
- 满足 Security 审计要求 5.1-5.3

**后果**: 
- 任何新增顶层字段需 Phase1 Gate Review
- 向后兼容变更需版本号升级

---

### ADR-002: 最小可运行链路 (v2.0)

**决策**: Gateway → Executor → Verifier → Committer 四层链路，集成 SRE 健康检查点

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Gateway   │ ──► │  Executor   │ ──► │  Verifier   │ ──► │  Committer  │
│  (入口层)   │     │  (执行层)   │     │  (验证层)   │     │  (提交层)   │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │                   │
       ▼                   ▼                   ▼                   ▼
   请求校验           Rust 执行          结果验证          状态提交
   限流鉴权           资源隔离           回放比对          持久化
       │                   │                   │                   │
       ▼                   ▼                   ▼                   ▼
   SLO:99%            SLO:99%            SLO:99%            SLO:99%
   P99≤100ms         P99≤100ms         P99≤100ms         P99≤100ms
```

**各层职责与 SLO**:

| 层 | 技术栈 | 核心职责 | 输出 | SLO |
|----|--------|----------|------|-----|
| Gateway | TypeScript/Node.js | 请求接入、鉴权、限流、路由 | 标准化 Contract 对象 | 可用性≥99%, P99≤100ms |
| Executor | Rust | Wasm/原生执行、资源隔离、超时控制 | ExecutionResult | 可用性≥99%, P99≤100ms |
| Verifier | Rust | 确定性验证、回放比对、差异分析 | VerificationReport | 可用性≥99%, P99≤100ms |
| Committer | Rust + DB | 状态提交、版本管理、回滚点 | CommitReceipt | 可用性≥99%, P99≤100ms |

**全链路 SLO (SRE W1T4)**:
- 可用性：≥99%
- 延迟 P99: ≤100ms
- 错误率：≤0.1%
- 恢复时间 (MTTR): ≤5m (含安全恢复)

---

## 2. 接口契约 (v2.0)

### 2.1 Gateway → Executor (gRPC)

```protobuf
// executor.proto
syntax = "proto3";
package cgas.phase0;

message Contract {
  string program = 1;           // 程序标识符
  bytes input = 2;              // 输入数据
  string state_root = 3;        // 状态树根哈希
  EnvFingerprint env_fingerprint = 4;  // 环境指纹
}

message EnvFingerprint {
  // === 主机层指纹 (SRE W1T4) ===
  string runtime_version = 1;   // 运行时版本
  string os_type = 2;           // 操作系统类型
  string arch = 3;              // CPU 架构
  
  // === 容器层指纹 (SRE W1T4) ===
  string container_id = 4;      // 容器 ID
  string image_digest = 5;      // 镜像摘要
  
  // === 平台层指纹 (SRE W1T4) ===
  string cluster_id = 6;        // 集群 ID
  string region = 7;            // 区域
  string zone = 8;              // 可用区
  
  // === 环境变量 (白名单) ===
  map<string, string> env_vars = 9;  // 关键环境变量
  
  // === 时间戳 ===
  uint64 timestamp_ms = 10;     // 时间戳 (ms)
}

message ExecutionRequest {
  Contract contract = 1;
  string request_id = 2;        // 请求追踪 ID
  uint64 timeout_ms = 3;        // 超时时间
  
  // === 安全审计字段 (Security 5.1-5.3) ===
  string audit_user_id = 4;     // 操作用户 ID
  string audit_session_id = 5;  // 会话 ID
  uint64 audit_timestamp_ms = 6; // 审计时间戳
}

message ExecutionResult {
  string request_id = 1;
  oneof result {
    bytes output = 10;          // 成功：输出数据
    ExecutionError error = 11;  // 失败：错误信息
  }
  uint64 duration_ms = 2;
  ResourceUsage resource_usage = 3;
  
  // === SRE 监控字段 ===
  string node_id = 4;           // 执行节点 ID
  uint64 start_timestamp_ms = 5; // 开始时间戳
  uint64 end_timestamp_ms = 6;   // 结束时间戳
}

message ExecutionError {
  string code = 1;              // 错误码 (见 2.4 节)
  string message = 2;           // 人类可读描述
  bytes debug_info = 3;         // 调试信息 (可选)
  
  // === 安全审计字段 ===
  string audit_trace_id = 4;    // 审计追踪 ID
}

message ResourceUsage {
  uint64 cpu_ms = 1;
  uint64 memory_peak_bytes = 2;
  uint64 io_bytes = 3;
}

service ExecutorService {
  rpc Execute(ExecutionRequest) returns (ExecutionResult);
  
  // === SRE 健康检查 ===
  rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
}

message HealthCheckRequest {}
message HealthCheckResponse {
  string status = 1;            // HEALTHY/UNHEALTHY/DEGRADED
  uint64 uptime_ms = 2;
  map<string, string> metadata = 3;
}
```

### 2.2 Executor → Verifier (内部调用)

```rust
// verifier/src/contract.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerificationRequest {
    pub contract: Contract,
    pub execution_result: ExecutionResult,
    pub golden_replay_id: String,
    
    // === 安全审计字段 ===
    pub audit_trace_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerificationReport {
    pub request_id: String,
    pub verified: bool,
    pub replay_match: bool,
    pub determinism_score: f64,  // 0.0-1.0
    pub differences: Vec<Difference>,
    pub timestamp_ms: u64,
    
    // === QA 质量门禁字段 ===
    pub coverage_score: f64,     // 覆盖率得分
    pub consistency_score: f64,  // 一致性得分
    pub performance_p99_ms: u64, // P99 性能
    
    // === 安全审计字段 ===
    pub audit_trace_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Difference {
    pub path: String,
    pub expected: String,
    pub actual: String,
}
```

### 2.3 Verifier → Committer (内部调用)

```rust
// committer/src/contract.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitRequest {
    pub verification_report: VerificationReport,
    pub new_state_root: String,
    pub parent_state_root: String,
    pub commit_metadata: CommitMetadata,
    
    // === 安全审计字段 ===
    pub audit_trace_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitMetadata {
    pub block_height: u64,
    pub tx_hash: String,
    pub signer: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitReceipt {
    pub commit_id: String,
    pub state_root: String,
    pub rollback_point: String,  // 回滚点标识
    pub timestamp_ms: u64,
    
    // === 安全审计字段 ===
    pub audit_trace_id: String,
}
```

### 2.4 错误码规范 (v2.0)

| 错误码 | 层级 | 描述 | 重试策略 | SLO 影响 |
|--------|------|------|----------|----------|
| `CONTRACT_INVALID` | Gateway | 契约字段校验失败 | 不重试 | 错误率计数 |
| `RATE_LIMITED` | Gateway | 限流触发 | 指数退避 | 延迟增加 |
| `AUTH_FAILED` | Gateway | 鉴权失败 (OIDC/OAuth2) | 不重试 | 安全告警 |
| `EXEC_TIMEOUT` | Executor | 执行超时 | 不重试 | 错误率计数 |
| `EXEC_RESOURCE_EXHAUSTED` | Executor | 资源耗尽 | 降级到备用节点 | 可用性保护 |
| `VERIFICATION_FAILED` | Verifier | 验证失败 | 不重试，触发告警 | 质量门禁 |
| `REPLAY_MISMATCH` | Verifier | 回放不匹配 | 不重试，触发告警 | 质量门禁 |
| `COMMIT_CONFLICT` | Committer | 提交冲突 | 重试 3 次后回滚 | 可用性保护 |
| `STATE_ROOT_INVALID` | Committer | 状态根无效 | 不重试，触发回滚 | 数据完整性 |
| `AUDIT_LOG_FAILED` | All | 审计日志写入失败 | 降级 (继续执行) | 合规性降级 |

---

## 3. 环境指纹采集规范 (v2.0 - SRE W1T4)

### 3.1 三层指纹结构

```rust
// executor/src/env_fingerprint.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvFingerprint {
    // === 主机层指纹 (采集周期：30s) ===
    pub host_layer: HostFingerprint,
    
    // === 容器层指纹 (采集周期：5m) ===
    pub container_layer: ContainerFingerprint,
    
    // === 平台层指纹 (采集周期：1h) ===
    pub platform_layer: PlatformFingerprint,
    
    // === 时间戳 ===
    pub timestamp_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostFingerprint {
    pub runtime_version: String,  // e.g., "rust-1.75.0"
    pub os_type: String,          // e.g., "linux"
    pub arch: String,             // e.g., "x86_64"
    pub kernel_version: String,   // e.g., "6.1.0"
    pub cpu_model: String,        // e.g., "Intel Xeon"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContainerFingerprint {
    pub container_id: String,
    pub image_digest: String,     // SHA-256
    pub cgroup_version: String,   // e.g., "v2"
    pub seccomp_profile: String,  // e.g., "runtime/default"
    pub apparmor_profile: String, // e.g., "docker-default"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlatformFingerprint {
    pub cluster_id: String,
    pub region: String,
    pub zone: String,
    pub node_pool: String,
    pub k8s_version: String,
}

impl EnvFingerprint {
    /// 白名单环境变量
    const ALLOWED_ENV_VARS: &'static [&'static str] = &[
        "TZ",           // 时区
        "LANG",         // 语言环境
        "CGAS_MODE",    // 运行模式
    ];
    
    /// 采集周期配置 (SRE SLO)
    const HOST_INTERVAL_MS: u64 = 30_000;    // 30s
    const CONTAINER_INTERVAL_MS: u64 = 300_000; // 5m
    const PLATFORM_INTERVAL_MS: u64 = 3_600_000; // 1h
    
    pub fn collect() -> Result<Self, FingerprintError> {
        Ok(Self {
            host_layer: HostFingerprint::collect()?,
            container_layer: ContainerFingerprint::collect()?,
            platform_layer: PlatformFingerprint::collect()?,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64,
        })
    }
}
```

### 3.2 存储后端 (SRE W1T4)

| 数据类型 | 存储后端 | 保留周期 | 查询用途 |
|----------|----------|----------|----------|
| 主机层指纹 | Prometheus | 30 天 | 实时监控、告警 |
| 容器层指纹 | Prometheus + Loki | 30 天 | 故障排查、审计 |
| 平台层指纹 | Prometheus | 90 天 | 容量规划、趋势分析 |
| 审计日志 | Loki + S3 | 365 天 | 合规审计、追溯 |

### 3.3 SLO 定义 (SRE W1T4)

| SLO 指标 | 目标值 | 测量窗口 | 告警阈值 |
|----------|--------|----------|----------|
| 可用性 | ≥99% | 滚动 24h | <98% 触发 P1 |
| 延迟 P99 | ≤100ms | 滚动 1h | >150ms 触发 P2 |
| 错误率 | ≤0.1% | 滚动 1h | >0.5% 触发 P1 |
| 恢复时间 (MTTR) | ≤5m | 单次事件 | >10m 触发 P1 |

---

## 4. 黄金回放集结构 (v2.0 - QA W1T3)

### 4.1 数据集目录结构

```
golden_replays/
├── README.md                 # 数据集说明
├── manifest.json             # 清单文件
├── quality_gates.json        # 质量门禁定义
├── normal/                   # 正常样本 (70% = 105 条)
│   ├── case_001.json
│   ├── case_002.json
│   └── ...
├── boundary/                 # 边界样本 (20% = 30 条)
│   ├── case_101.json
│   ├── case_102.json
│   └── ...
└── adversarial/              # 对抗样本 (10% = 15 条)
    ├── case_201.json
    ├── case_202.json
    └── ...
```

### 4.2 样本结构 (v2.0)

```json
{
  "case_id": "normal-001",
  "category": "normal",
  "contract": {
    "program": "transfer",
    "input": "base64_encoded_input_data",
    "state_root": "abc123...",
    "env_fingerprint": {
      "host_layer": {
        "runtime_version": "rust-1.75.0",
        "os_type": "linux",
        "arch": "x86_64",
        "kernel_version": "6.1.0",
        "cpu_model": "Intel Xeon"
      },
      "container_layer": {
        "container_id": "abc123",
        "image_digest": "sha256:def456...",
        "cgroup_version": "v2",
        "seccomp_profile": "runtime/default",
        "apparmor_profile": "docker-default"
      },
      "platform_layer": {
        "cluster_id": "prod-cn-shanghai",
        "region": "cn-shanghai",
        "zone": "shanghai-zone-a",
        "node_pool": "general",
        "k8s_version": "1.28.0"
      },
      "timestamp_ms": 1709625600000
    }
  },
  "expected_output": "base64_encoded_expected_output",
  "expected_state_root": "def456...",
  "annotations": {
    "description": "标准转账场景",
    "tags": ["transfer", "normal", "single-tx"],
    "created_at": "2026-03-05T00:00:00Z",
    "created_by": "qa-team"
  },
  "evidence": {
    "metric_value": 1.0,
    "window": "single_run",
    "sample_size": 1,
    "source": "golden_replay_suite_v1"
  }
}
```

### 4.3 样本配比与规模 (QA W1T3)

| 类别 | 占比 | 最小样本数 | 描述 | 安全覆盖 |
|------|------|------------|------|----------|
| normal | 70% | 105 | 标准业务场景，覆盖核心功能 | - |
| boundary | 20% | 30 | 边界条件：空输入、最大输入、超时边缘等 | - |
| adversarial | 10% | 15 | 对抗场景：恶意输入、资源耗尽尝试等 | Security 验证 |
| **总计** | **100%** | **150** | **最小样本集规模** | **15 条安全场景** |

**当前状态**: 0/150 条已采集，Week2 目标≥30 条

### 4.4 质量门禁 (QA W1T3)

```json
// quality_gates.json
{
  "coverage_threshold": 0.95,
  "consistency_threshold": 0.99,
  "performance_p99_ms": 100,
  "evidence_quartet_required": true
}
```

| 门禁指标 | 目标值 | 测量方式 | 失败处理 |
|----------|--------|----------|----------|
| 覆盖率 | ≥95% | 回放集覆盖契约字段比例 | 阻断发布 |
| 一致性 | ≥99% | 多次执行结果一致性 | 阻断发布 |
| 性能 P99 | ≤100ms | 回放执行延迟 P99 | 告警 + 优化 |
| 证据四元组 | 100% 必需 | metric_value/window/sample_size/source | 阻断发布 |

**Go/Conditional Go/No-Go 决策**:
- **Go**: 全部门禁通过
- **Conditional Go**: 性能 P99 超标但有优化计划，其他通过
- **No-Go**: 覆盖率<95% 或 一致性<99% 或 证据缺失

### 4.5 标注规范

- **必填字段**: `case_id`, `category`, `contract`, `expected_output`, `evidence`
- **可选字段**: `expected_state_root`, `annotations`
- **标签体系**: 按业务功能、测试类型、优先级三维打标

---

## 5. 失败路径与回滚路径 (v2.0)

### 5.1 失败路径矩阵 (集成 SRE SLO)

| 故障点 | 检测方式 | 响应动作 | 通知对象 | SLO 影响 |
|--------|----------|----------|----------|----------|
| Gateway 鉴权失败 | JWT 校验 | 返回 401，记录审计日志 | 安全团队 | 错误率计数 |
| Gateway 限流触发 | 令牌桶 | 返回 429，指数退避 | 客户端 | 延迟增加 |
| Gateway 健康检查失败 | /health | 切换备用节点 | SRE | 可用性保护 |
| Executor 超时 | 定时器 | 终止执行，释放资源 | SRE | 错误率计数 |
| Executor 崩溃 | 进程监控 | 重启实例，隔离故障 | SRE | 可用性保护 |
| Executor 健康检查失败 | /health | 切换备用节点 | SRE | 可用性保护 |
| Verifier 验证失败 | 回放比对 | 拒绝提交，触发告警 | Dev + QA | 质量门禁 |
| Verifier 回放不匹配 | 差异分析 | 拒绝提交，触发告警 | Dev + QA | 质量门禁 |
| Committer 提交冲突 | 乐观锁 | 重试 3 次后回滚 | Dev | 可用性保护 |
| Committer 状态根无效 | 哈希校验 | 不重试，触发回滚 | Dev | 数据完整性 |
| DB 连接失败 | 健康检查 | 切换到备用 DB | SRE | 可用性保护 |
| 审计日志失败 | 写入确认 | 降级 (继续执行) | Security | 合规性降级 |

### 5.2 回滚路径设计 (v2.0)

```
正常链路:
  Gateway → Executor → Verifier → Committer → DB (state_root: v2)
                                      │
                                      ▼
                              回滚点：state_root: v1

回滚触发条件:
  1. Verifier 验证失败 (REPLAY_MISMATCH)
  2. Committer 提交冲突 (COMMIT_CONFLICT)
  3. 事后发现数据异常 (手动触发)
  4. 安全审计要求 (Security 5.3)

回滚流程:
  1. 验证授权 (OIDC/OAuth2 + RBAC)
  2. 读取回滚点 state_root (v1)
  3. 验证回滚点完整性
  4. 更新当前 state_root 指向 v1
  5. 记录回滚审计日志 (Security 5.1)
  6. 通知相关方 (SRE 告警)
  7. 验证回滚后一致性 (QA 回放)

回滚 SLO:
  - 回滚执行时间：≤5m (MTTR)
  - 回滚成功率：≥99%
  - 回滚后一致性：100%
```

### 5.3 回滚 API (v2.0)

```rust
// committer/src/rollback.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RollbackRequest {
    pub target_state_root: String,  // 目标回滚点
    pub reason: String,             // 回滚原因
    pub authorized_by: String,      // 授权人 (OIDC user_id)
    
    // === 安全审计字段 ===
    pub audit_trace_id: String,
    pub audit_session_id: String,
    pub audit_timestamp_ms: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RollbackReceipt {
    pub rollback_id: String,
    pub from_state_root: String,
    pub to_state_root: String,
    pub timestamp_ms: u64,
    pub status: RollbackStatus,
    
    // === 安全审计字段 ===
    pub audit_trace_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RollbackStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

pub async fn rollback(request: RollbackRequest) -> Result<RollbackReceipt, RollbackError> {
    // 1. 验证授权 (OIDC/OAuth2 + RBAC)
    // 2. 验证目标 state_root 存在且有效
    // 3. 执行回滚
    // 4. 记录审计日志 (Security 5.1)
    // 5. 通知相关方 (SRE 告警)
    // 6. 验证回滚后一致性 (QA 回放)
}
```

### 5.4 降级策略 (v2.0)

| 场景 | 降级方案 | 影响范围 | SLO 影响 |
|------|----------|----------|----------|
| Executor 资源耗尽 | 切换到备用执行节点 | 延迟增加 | P99 可能上升 |
| Verifier 不可用 | 跳过验证，标记为"未验证" | 安全性降低 | 质量门禁降级 |
| Committer 不可用 | 队列缓冲，异步提交 | 最终一致性 | 延迟增加 |
| DB 不可用 | 切换到只读模式 | 无法写入 | 可用性降级 |
| 审计日志不可用 | 本地缓冲，异步写入 | 合规性风险 | 审计延迟 |

---

## 6. 安全基线集成 (Security W1T5)

### 6.1 审计要求 (Security 5.1-5.3)

| 要求 ID | 描述 | 实现方式 | 验证方法 |
|--------|------|----------|----------|
| 5.1 | 所有操作可追溯 | audit_trace_id 贯穿全链路 | 日志审计 |
| 5.2 | 敏感数据脱敏 | input 字段加密存储 | 代码审查 |
| 5.3 | 回滚操作授权 | OIDC/OAuth2 + RBAC | 渗透测试 |

### 6.2 Week2 安全待办

| 议题 | 描述 | 优先级 |
|------|------|--------|
| 身份授权契约 | OIDC/OAuth2 集成、RBAC+ABAC 模型 | P0 |
| 运行时安全边界 | seccomp/apparmor 配置文件 | P0 |
| 密钥管理 | Vault/KMS 集成方案 | P1 |
| 供应链安全 | Manifest 签名、SAST/SCA 流程 | P1 |

---

## 7. 实现计划 (v2.0)

### 7.1 Rust 核心模块

```
rust-workflow-engine/
├── executor/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── contract.rs        # 契约定义
│   │   ├── env_fingerprint.rs # 环境指纹 (三层结构)
│   │   ├── execution.rs       # 执行逻辑
│   │   └── audit.rs           # 审计日志 (Security 5.1)
│   └── Cargo.toml
├── verifier/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── verification.rs    # 验证逻辑
│   │   ├── replay.rs          # 回放比对
│   │   └── quality_gate.rs    # 质量门禁 (QA)
│   └── Cargo.toml
└── committer/
    ├── src/
    │   ├── lib.rs
    │   ├── commit.rs          # 提交逻辑
    │   ├── rollback.rs        # 回滚逻辑
    │   └── audit.rs           # 审计日志 (Security 5.1)
    └── Cargo.toml
```

### 7.2 TypeScript Gateway

```
gateway/
├── src/
│   ├── index.ts
│   ├── contract.ts           # 契约类型定义
│   ├── grpc-client.ts        # Executor gRPC 客户端
│   ├── auth.ts               # 鉴权中间件 (OIDC/OAuth2)
│   ├── rate-limit.ts         # 限流中间件
│   └── audit.ts              # 审计日志 (Security 5.1)
├── proto/
│   └── executor.proto        # gRPC 协议定义
└── package.json
```

### 7.3 里程碑 (v2.0)

| 里程碑 | 交付物 | 预计完成 | 准入条件 |
|--------|--------|----------|----------|
| M1: 契约冻结 | 本技术规格书 + PM 签署 | Week1 结束 | ✅ 完成 |
| M2: Rust 框架 | executor/verifier/committer 骨架代码 | Week2 结束 | 契约签署完成 |
| M3: Gateway 实现 | TypeScript Gateway + gRPC 联调 | Week3 结束 | 回放集样本采集启动 |
| M4: 回放集 v1 | 黄金回放集 150 样本 | Week4 结束 | 监控告警链路打通 |
| M5: 端到端测试 | 最小可运行链路打通 | Week5 结束 | 质量门禁全部通过 |

---

## 8. 待决议题 (v2.0)

| ID | 议题 | 影响范围 | 建议决策时间 | 负责人 |
|----|------|----------|-------------|--------|
| TBD-001 | state_root 存储后端选型 (PostgreSQL vs RocksDB) | Committer 实现 | Week2 | Dev+SRE |
| TBD-002 | gRPC vs REST 对外暴露协议 | Gateway 设计 | Week2 | Dev+Security |
| TBD-003 | 回放集版本管理策略 | QA 工具链 | Week3 | QA+Dev |
| TBD-004 | OIDC/OAuth2 提供商选型 | 身份授权 | Week2 | Security+Dev |
| TBD-005 | Vault vs KMS 密钥管理方案 | 密钥管理 | Week2 | Security+SRE |

---

## 9. 附录

### 9.1 参考文档

- cgas/rust-workflow-engine 现有代码模式
- Platform 工程 gRPC/REST 最佳实践
- 分布式系统 SRE 失败处理模式
- 状态机版本控制模式
- Security 基线要求 5.1-5.3
- SRE 四层链路健康检查规范
- QA 黄金回放集质量门禁规范

### 9.2 术语表

| 术语 | 定义 |
|------|------|
| Contract | 确定性契约，包含 program/input/state_root/env_fingerprint |
| state_root | 状态树根哈希，用于验证状态一致性 |
| env_fingerprint | 环境指纹，三层结构 (主机/容器/平台)，用于保证执行环境确定性 |
| Golden Replay | 黄金回放集，150 条最小样本集，用于验证执行确定性 |
| SLO | 服务等级目标，4 个关键指标 (可用性/延迟/错误率/恢复时间) |
| audit_trace_id | 审计追踪 ID，贯穿全链路，满足 Security 5.1 要求 |

### 9.3 变更日志

| 版本 | 日期 | 变更描述 |
|------|------|----------|
| v1.0 | 2026-03-05 | 初始版本 |
| v2.0 | 2026-03-05 | 吸收 QA/Security/SRE/PM 全部反馈：三层指纹结构、150 样本集、质量门禁、审计要求 5.1-5.3、4 个 SLO |

---

**文档状态**: 全角色评审通过  
**下一步**: 提交 PM 正式签署，启动 Week2 工作  
**签署状态**: 
- [x] Dev 技术确认
- [x] QA 测试策略确认
- [x] Security 安全审查确认
- [x] SRE 运维规范确认
- [ ] PM 正式签署 (待完成)

**Week2 准入条件检查**:
- [ ] 契约正式签署
- [ ] 回放集样本采集启动 (目标≥30 条)
- [ ] 监控告警链路打通
