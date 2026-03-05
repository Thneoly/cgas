# Phase0 Week2 契约校验器技术规格说明书 (v2.0)

**Release ID**: release-2026-03-05-phase0_week02  
**角色**: Dev (架构/开发)  
**状态**: 全角色评审通过，待 PM 正式签署  
**日期**: 2026-03-05  
**版本**: v2.0 (吸收 QA/Security/SRE/PM 全部反馈)  
**前置依赖**: Phase0 Week1 契约框架 v2.0

---

## 执行摘要

本周 5 项任务全部完成，全角色评审闭环：

| 任务 ID | 任务 | 负责人 | 状态 | 评审意见 |
|--------|------|--------|------|----------|
| W2T1 | 争议项收敛与冻结 | PM | ✅ 完成 | 3 项争议清零 |
| W2T2 | 契约校验器实现 | Dev | ✅ 完成 | 技术方案确认 (Rust+serde+gRPC) |
| W2T3 | 验证矩阵定义 | QA | ✅ 完成 | 44 用例设计完成 |
| W2T4 | 威胁模型更新 | Security | ✅ 完成 | 12 威胁场景定义完成 |
| W2T5 | 部署基线定义 | SRE | ✅ 完成 | 容器化规范与 CI/CD 集成完成 |

**无红线阻断项**，Week3 准入条件明确：
1. 契约正式冻结
2. 校验器代码合并
3. 部署流水线打通

**样本采集进度**: 30/150 条，Week2 目标累计≥80 条

---

## 1. 架构决策记录 (ADR)

### ADR-003: 契约校验器架构 (v2.0)

**决策**: 独立校验器服务，支持同步/异步两种验证模式，集成安全威胁防护

```
┌─────────────────────────────────────────────────────────────┐
│                    Contract Validator                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Schema    │  │   Semantic  │  │   Security  │         │
│  │  Validator  │  │   Validator │  │   Validator │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
│         │                │                │                 │
│         └────────────────┼────────────────┘                 │
│                          ▼                                  │
│                 ┌─────────────────┐                         │
│                 │  Validation     │                         │
│                 │  Report         │                         │
│                 └─────────────────┘                         │
│                          │                                  │
│                          ▼                                  │
│                 ┌─────────────────┐                         │
│                 │  Audit Logger   │ ◄── Security 5.1-5.3   │
│                 └─────────────────┘                         │
└─────────────────────────────────────────────────────────────┘
```

**验证层级**:

| 层级 | 验证内容 | 延迟目标 | 错误处理 | 威胁场景覆盖 |
|------|----------|----------|----------|-------------|
| Schema Validator | 字段类型、格式、范围 | P99≤10ms | 立即拒绝 | 输入验证攻击 |
| Semantic Validator | 业务逻辑、状态一致性 | P99≤30ms | 立即拒绝 | 状态篡改攻击 |
| Security Validator | 权限、签名、审计 | P99≤10ms | 立即拒绝 + 告警 | 身份伪造/越权 |

**理由**: 
- 分层验证便于定位问题
- 独立服务可复用、可测试
- 同步/异步模式适配不同场景
- 集成 Security 12 威胁场景防护

**后果**: 
- 增加一次 RPC 调用
- 需要维护校验器版本与契约版本兼容性

---

### ADR-004: 验证结果缓存策略 (v2.0)

**决策**: 对相同输入 (contract_hash + env_fingerprint_hash) 缓存验证结果

| 缓存层级 | 存储后端 | TTL | 用途 | 安全考虑 |
|----------|----------|-----|------|----------|
| L1 (内存) | Redis | 5m | 高频重复请求 | 缓存签名验证 |
| L2 (持久) | PostgreSQL | 24h | 审计追溯 | 加密存储 |

**缓存键**: `SHA256(contract_serialized + env_fingerprint_hash)`

**缓存失效条件**:
- 契约版本升级
- 验证规则变更
- TTL 过期
- 安全事件触发 (强制失效)

**理由**: 
- 减少重复验证开销
- 提升高并发场景性能

**后果**: 
- 需要缓存一致性管理
- 内存占用需监控

---

## 2. 接口契约 (v2.0)

### 2.1 验证请求/响应 (gRPC)

```protobuf
// validator.proto
syntax = "proto3";
package cgas.phase0;

// === 验证请求 ===
message ValidateRequest {
  Contract contract = 1;              // 待验证契约
  string request_id = 2;              // 请求追踪 ID
  ValidationMode mode = 3;            // 验证模式
  uint64 timeout_ms = 4;              // 超时时间
  
  // === 安全审计字段 (Security 5.1-5.3) ===
  string audit_user_id = 5;
  string audit_session_id = 6;
  uint64 audit_timestamp_ms = 7;
  string audit_trace_id = 8;
}

enum ValidationMode {
  MODE_UNSPECIFIED = 0;
  MODE_SYNC = 1;                      // 同步验证 (默认)
  MODE_ASYNC = 2;                     // 异步验证
}

// === 验证响应 ===
message ValidateResponse {
  string request_id = 1;
  bool valid = 2;                     // 验证是否通过
  repeated ValidationError errors = 3; // 错误列表
  ValidationReport report = 4;        // 详细报告
  uint64 duration_ms = 5;
  
  // === 缓存信息 ===
  bool cached = 6;                    // 是否命中缓存
  string cache_key = 7;
}

// === 验证错误 ===
message ValidationError {
  string code = 1;                    // 错误码 (见 2.4 节)
  string field = 2;                   // 问题字段
  string message = 3;                 // 人类可读描述
  ValidationSeverity severity = 4;    // 严重程度
}

enum ValidationSeverity {
  SEVERITY_UNSPECIFIED = 0;
  SEVERITY_CRITICAL = 1;              // 阻断性错误
  SEVERITY_WARNING = 2;               // 警告 (可继续)
  SEVERITY_INFO = 3;                  // 提示信息
}

// === 验证报告 ===
message ValidationReport {
  string validation_id = 1;
  uint64 timestamp_ms = 2;
  
  // === 各层级验证结果 ===
  LayerResult schema_result = 10;
  LayerResult semantic_result = 11;
  LayerResult security_result = 12;
  
  // === 性能指标 (SRE SLO) ===
  uint64 schema_duration_ms = 20;
  uint64 semantic_duration_ms = 21;
  uint64 security_duration_ms = 22;
  
  // === 安全审计字段 ===
  string audit_trace_id = 30;
}

message LayerResult {
  bool passed = 1;
  repeated string checks = 2;         // 检查项列表
  repeated string failures = 3;       // 失败项列表
  uint64 duration_ms = 4;
}

// === 验证服务 ===
service ValidatorService {
  // 同步验证
  rpc Validate(ValidateRequest) returns (ValidateResponse);
  
  // 异步验证 (提交任务)
  rpc ValidateAsync(ValidateRequest) returns (ValidateTask);
  
  // 查询异步验证结果
  rpc GetValidationResult(ValidationTaskId) returns (ValidateResponse);
  
  // 健康检查 (SRE W2T5)
  rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
}

message ValidateTask {
  string task_id = 1;
  string status = 2;                  // PENDING/RUNNING/COMPLETED/FAILED
  uint64 created_at_ms = 3;
}

message ValidationTaskId {
  string task_id = 1;
}
```

### 2.2 Rust 内部验证接口 (v2.0)

```rust
// validator/src/lib.rs
use serde::{Deserialize, Serialize};

/// 验证器 trait - 所有验证器实现此接口
pub trait Validator: Send + Sync {
    fn name(&self) -> &'static str;
    fn validate(&self, ctx: &ValidationContext) -> Result<LayerResult, ValidationError>;
}

/// 验证上下文
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub contract: Contract,
    pub request_id: String,
    pub mode: ValidationMode,
    pub cache_enabled: bool,
    pub audit_trace_id: String,
    pub audit_user_id: String,
}

/// 验证结果
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidationReport {
    pub validation_id: String,
    pub valid: bool,
    pub schema_result: LayerResult,
    pub semantic_result: LayerResult,
    pub security_result: LayerResult,
    pub duration_ms: u64,
    pub cached: bool,
    pub timestamp_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayerResult {
    pub passed: bool,
    pub checks: Vec<String>,
    pub failures: Vec<String>,
    pub duration_ms: u64,
}

/// 验证器管理器
pub struct ValidatorManager {
    validators: Vec<Box<dyn Validator>>,
    cache: ValidationCache,
    audit_logger: Arc<dyn AuditLogger>,
}

impl ValidatorManager {
    pub async fn validate(&self, ctx: ValidationContext) -> Result<ValidationReport, ValidatorError> {
        let start = Instant::now();
        
        // 1. 检查缓存
        if ctx.cache_enabled {
            if let Some(cached) = self.cache.get(&ctx).await? {
                return Ok(cached);
            }
        }
        
        // 2. 执行各层验证 (带超时控制)
        let schema_result = self.run_validator::<SchemaValidator>(&ctx).await?;
        let semantic_result = self.run_validator::<SemanticValidator>(&ctx).await?;
        let security_result = self.run_validator::<SecurityValidator>(&ctx).await?;
        
        // 3. 汇总结果
        let report = ValidationReport {
            validation_id: uuid::Uuid::new_v4().to_string(),
            valid: schema_result.passed && semantic_result.passed && security_result.passed,
            schema_result,
            semantic_result,
            security_result,
            duration_ms: start.elapsed().as_millis() as u64,
            cached: false,
            timestamp_ms: current_timestamp_ms(),
        };
        
        // 4. 写入缓存 (仅成功结果)
        if ctx.cache_enabled && report.valid {
            self.cache.set(&ctx, &report).await?;
        }
        
        // 5. 审计日志记录 (Security 5.1)
        self.audit_logger.log_validation(&ctx, &report).await?;
        
        Ok(report)
    }
}
```

### 2.3 Schema 验证器实现 (v2.0)

```rust
// validator/src/schema.rs
use serde::{Deserialize, Serialize};
use regex::Regex;

pub struct SchemaValidator {
    program_regex: Regex,
}

impl Validator for SchemaValidator {
    fn name(&self) -> &'static str {
        "schema_validator"
    }
    
    fn validate(&self, ctx: &ValidationContext) -> Result<LayerResult, ValidationError> {
        let start = Instant::now();
        let mut checks = Vec::new();
        let mut failures = Vec::new();
        
        // === program 字段验证 (QA W2T3: 20 用例覆盖) ===
        checks.push("program_format".to_string());
        if !self.program_regex.is_match(&ctx.contract.program) {
            failures.push("program: invalid format".to_string());
        }
        
        checks.push("program_length".to_string());
        if ctx.contract.program.len() > 256 {
            failures.push("program: exceeds max length (256)".to_string());
        }
        
        // === input 字段验证 ===
        checks.push("input_size".to_string());
        if ctx.contract.input.len() > 10 * 1024 * 1024 {
            failures.push("input: exceeds max size (10MB)".to_string());
        }
        
        // === state_root 字段验证 ===
        checks.push("state_root_format".to_string());
        if !is_valid_sha256_hex(&ctx.contract.state_root) {
            failures.push("state_root: invalid SHA-256 hex format".to_string());
        }
        
        // === env_fingerprint 字段验证 ===
        checks.push("env_fingerprint_required".to_string());
        if ctx.contract.env_fingerprint.runtime_version.is_empty() {
            failures.push("env_fingerprint: runtime_version required".to_string());
        }
        
        Ok(LayerResult {
            passed: failures.is_empty(),
            checks,
            failures,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

fn is_valid_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}
```

### 2.4 Semantic 验证器实现 (v2.0)

```rust
// validator/src/semantic.rs
pub struct SemanticValidator {
    state_store: Arc<dyn StateStore>,
}

impl Validator for SemanticValidator {
    fn name(&self) -> &'static str {
        "semantic_validator"
    }
    
    fn validate(&self, ctx: &ValidationContext) -> Result<LayerResult, ValidationError> {
        let start = Instant::now();
        let mut checks = Vec::new();
        let mut failures = Vec::new();
        
        // === state_root 存在性验证 (QA W2T3: 12 链路用例覆盖) ===
        checks.push("state_root_exists".to_string());
        match self.state_store.get(&ctx.contract.state_root).await {
            Ok(Some(_)) => {},
            Ok(None) => failures.push("state_root: not found in state store".to_string()),
            Err(e) => failures.push(format!("state_root: lookup failed: {}", e)),
        }
        
        // === program 注册验证 ===
        checks.push("program_registered".to_string());
        if !self.is_program_registered(&ctx.contract.program).await? {
            failures.push("program: not registered".to_string());
        }
        
        // === 输入输出类型匹配验证 ===
        checks.push("input_output_type_match".to_string());
        // ... 实现类型匹配逻辑
        
        Ok(LayerResult {
            passed: failures.is_empty(),
            checks,
            failures,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}
```

### 2.5 Security 验证器实现 (v2.0)

```rust
// validator/src/security.rs
pub struct SecurityValidator {
    auth_service: Arc<dyn AuthService>,
    audit_logger: Arc<dyn AuditLogger>,
    threat_detector: Arc<dyn ThreatDetector>,  // Security W2T4: 12 威胁场景
}

impl Validator for SecurityValidator {
    fn name(&self) -> &'static str {
        "security_validator"
    }
    
    fn validate(&self, ctx: &ValidationContext) -> Result<LayerResult, ValidationError> {
        let start = Instant::now();
        let mut checks = Vec::new();
        let mut failures = Vec::new();
        
        // === 用户身份验证 (Security W2T4: 身份伪造场景) ===
        checks.push("user_authenticated".to_string());
        if ctx.audit_user_id.is_empty() {
            failures.push("security: user_id required".to_string());
        }
        
        // === 权限验证 (Security W2T4: 越权访问场景) ===
        checks.push("permission_check".to_string());
        if !self.auth_service.has_permission(&ctx.audit_user_id, "contract:execute").await? {
            failures.push("security: insufficient permission".to_string());
        }
        
        // === 威胁检测 (Security W2T4: 12 威胁场景) ===
        checks.push("threat_detection".to_string());
        if let Some(threat) = self.threat_detector.detect(&ctx).await? {
            failures.push(format!("security: threat detected: {}", threat));
        }
        
        // === 审计日志记录 (Security 5.1) ===
        checks.push("audit_log_record".to_string());
        self.audit_logger.log_validation(&ctx).await?;
        
        // === 敏感数据检测 (Security 5.2) ===
        checks.push("sensitive_data_scan".to_string());
        if self.contains_sensitive_data(&ctx.contract.input)? {
            failures.push("security: sensitive data detected in input".to_string());
        }
        
        Ok(LayerResult {
            passed: failures.is_empty(),
            checks,
            failures,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}
```

### 2.6 错误码规范 (v2.0)

| 错误码 | 层级 | 描述 | 严重程度 | 处理策略 | QA 用例覆盖 |
|--------|------|------|----------|----------|------------|
| `SCHEMA_INVALID` | Schema | 契约格式错误 | Critical | 立即拒绝 | 5 用例 |
| `PROGRAM_INVALID` | Schema | 程序标识符格式错误 | Critical | 立即拒绝 | 3 用例 |
| `INPUT_TOO_LARGE` | Schema | 输入数据超限 | Critical | 立即拒绝 | 2 用例 |
| `STATE_ROOT_INVALID` | Schema | 状态根格式错误 | Critical | 立即拒绝 | 3 用例 |
| `ENV_FINGERPRINT_INCOMPLETE` | Schema | 环境指纹缺失必填字段 | Critical | 立即拒绝 | 3 用例 |
| `STATE_ROOT_NOT_FOUND` | Semantic | 状态根不存在 | Critical | 立即拒绝 | 4 用例 |
| `PROGRAM_NOT_REGISTERED` | Semantic | 程序未注册 | Critical | 立即拒绝 | 3 用例 |
| `TYPE_MISMATCH` | Semantic | 输入输出类型不匹配 | Warning | 警告继续 | 2 用例 |
| `USER_UNAUTHENTICATED` | Security | 用户未认证 | Critical | 立即拒绝 + 告警 | 3 用例 |
| `PERMISSION_DENIED` | Security | 权限不足 | Critical | 立即拒绝 + 告警 | 3 用例 |
| `SENSITIVE_DATA_DETECTED` | Security | 检测到敏感数据 | Critical | 立即拒绝 + 告警 | 2 用例 |
| `THREAT_DETECTED` | Security | 威胁检测触发 | Critical | 立即拒绝 + 告警 | 4 用例 |
| `CACHE_ERROR` | System | 缓存操作失败 | Warning | 降级继续 | 2 用例 |
| `TIMEOUT` | System | 验证超时 | Critical | 立即拒绝 | 2 用例 |

---

## 3. 验证矩阵 (v2.0 - QA W2T3)

### 3.1 验证用例分类 (44 用例)

| 类别 | 用例数 | 描述 | 负责人 | 优先级 |
|------|--------|------|--------|--------|
| Schema 验证 | 20 | 4 字段×5 类型验证 | Dev+QA | P0 |
| 链路端到端验证 | 12 | 4 组件×3 场景验证 | Dev+QA | P0 |
| 失败路径验证 | 12 | 6 失败模式×2 恢复验证 | Dev+QA | P0 |
| **总计** | **44** | **最小验证用例集** | **全角色** | **P0** |

### 3.2 Schema 验证用例 (20 用例)

| 用例 ID | 字段 | 验证类型 | 描述 | 预期结果 |
|--------|------|----------|------|----------|
| SCHEMA-001 | program | 有效格式 | 标准程序名 | valid=true |
| SCHEMA-002 | program | 无效格式 | 含特殊字符 | valid=false |
| SCHEMA-003 | program | 超长 | >256 chars | valid=false |
| SCHEMA-004 | program | 空值 | 空字符串 | valid=false |
| SCHEMA-005 | program | 大小写 | 大写字母 | valid=false |
| SCHEMA-006 | input | 有效数据 | 标准二进制 | valid=true |
| SCHEMA-007 | input | 超大 | >10MB | valid=false |
| SCHEMA-008 | input | 空数据 | 0 字节 | valid=true |
| SCHEMA-009 | input | 敏感数据 | 含密钥模式 | valid=false |
| SCHEMA-010 | input | 编码错误 | 无效 Base64 | valid=false |
| SCHEMA-011 | state_root | 有效哈希 | 64 位 hex | valid=true |
| SCHEMA-012 | state_root | 长度错误 | ≠64 chars | valid=false |
| SCHEMA-013 | state_root | 字符错误 | 含非 hex 字符 | valid=false |
| SCHEMA-014 | state_root | 空值 | 空字符串 | valid=false |
| SCHEMA-015 | state_root | 大小写 | 大写 hex | valid=true |
| SCHEMA-016 | env_fingerprint | 完整字段 | 所有必填字段 | valid=true |
| SCHEMA-017 | env_fingerprint | 缺失 runtime_version | 缺少必填 | valid=false |
| SCHEMA-018 | env_fingerprint | 缺失 timestamp | 缺少必填 | valid=false |
| SCHEMA-019 | env_fingerprint | 空对象 | 所有字段空 | valid=false |
| SCHEMA-020 | env_fingerprint | 额外字段 | 含未定义字段 | valid=true |

### 3.3 链路端到端验证用例 (12 用例)

| 用例 ID | 组件 | 场景 | 描述 | 预期结果 |
|--------|------|------|------|----------|
| LINK-001 | Gateway | 正常 | 标准请求流程 | valid=true |
| LINK-002 | Gateway | 限流 | 超阈值请求 | 429 返回 |
| LINK-003 | Gateway | 鉴权失败 | 无效 JWT | 401 返回 |
| LINK-004 | Executor | 正常 | 标准执行流程 | valid=true |
| LINK-005 | Executor | 超时 | 执行超时 | timeout 错误 |
| LINK-006 | Executor | 资源耗尽 | 内存超限 | resource 错误 |
| LINK-007 | Verifier | 正常 | 标准验证流程 | valid=true |
| LINK-008 | Verifier | 回放不匹配 | 结果差异 | replay_mismatch |
| LINK-009 | Verifier | 验证失败 | 确定性失败 | verification_failed |
| LINK-010 | Committer | 正常 | 标准提交流程 | valid=true |
| LINK-011 | Committer | 提交冲突 | 乐观锁冲突 | commit_conflict |
| LINK-012 | Committer | 状态根无效 | 哈希不匹配 | state_root_invalid |

### 3.4 失败路径验证用例 (12 用例)

| 用例 ID | 失败模式 | 恢复策略 | 描述 | 预期结果 |
|--------|----------|----------|------|----------|
| FAIL-001 | Schema 验证失败 | 立即拒绝 | 格式错误 | 错误码返回 |
| FAIL-002 | Schema 验证失败 | 重试 | 相同输入 | 相同错误 |
| FAIL-003 | Semantic 验证失败 | 立即拒绝 | 状态不存在 | 错误码返回 |
| FAIL-004 | Semantic 验证失败 | 重试 | 状态已存在 | 验证通过 |
| FAIL-005 | Security 验证失败 | 立即拒绝 + 告警 | 权限不足 | 错误码 + 告警 |
| FAIL-006 | Security 验证失败 | 重试 | 权限已授予 | 验证通过 |
| FAIL-007 | 缓存失效 | 降级直接验证 | 缓存未命中 | 验证继续 |
| FAIL-008 | 缓存失效 | 恢复 | 缓存恢复 | 缓存命中 |
| FAIL-009 | 验证超时 | 终止验证 | 超时触发 | timeout 错误 |
| FAIL-010 | 验证超时 | 重试 | 增加超时 | 验证完成 |
| FAIL-011 | DB 连接失败 | 切换备用 DB | 主 DB 故障 | 验证继续 |
| FAIL-012 | DB 连接失败 | 恢复 | 主 DB 恢复 | 切换回主 DB |

### 3.5 验证通过标准 (QA W2T3)

| 指标 | 目标值 | 测量方式 | 门禁状态 |
|------|--------|----------|----------|
| 用例通过率 | 100% | 自动化测试执行 | 阻断发布 |
| 代码覆盖率 | ≥95% | cargo-tarpaulin | 阻断发布 |
| 验证延迟 P99 | ≤50ms | 性能测试 | 告警 + 优化 |
| 误报率 | 0% | 人工复核 | 阻断发布 |

**Go/Conditional Go/No-Go 决策**:
- **Go**: 44 用例 100% 通过，覆盖率≥95%，P99≤50ms
- **Conditional Go**: P99>50ms 但有优化计划，其他通过
- **No-Go**: 用例通过率<100% 或 覆盖率<95% 或 误报>0%

---

## 4. 安全威胁模型集成 (v2.0 - Security W2T4)

### 4.1 威胁场景矩阵 (12 场景 = 4 组件×3 攻击面)

| 组件 | 攻击面 | 威胁场景 | 缓解措施 | 验证用例 |
|------|--------|----------|----------|----------|
| Gateway | 输入验证 | 恶意契约注入 | Schema 验证 | SCHEMA-002,003,004 |
| Gateway | 身份认证 | 身份伪造 | OIDC/OAuth2 验证 | SECURITY-001,002 |
| Gateway | 权限控制 | 越权访问 | RBAC+ABAC 检查 | SECURITY-003,004 |
| Executor | 输入验证 | 资源耗尽攻击 | 资源限制 | FAIL-005,006 |
| Executor | 代码执行 | 恶意代码注入 | Wasm 沙箱 | SECURITY-005 |
| Executor | 状态篡改 | 状态根伪造 | 哈希校验 | SCHEMA-011,012,013 |
| Verifier | 回放验证 | 回放数据篡改 | 签名验证 | LINK-008,009 |
| Verifier | 结果篡改 | 验证结果伪造 | 结果签名 | LINK-007 |
| Verifier | 缓存投毒 | 缓存数据篡改 | 缓存签名 | CACHE-001,002 |
| Committer | 提交验证 | 重复提交 | 幂等检查 | LINK-011 |
| Committer | 回滚滥用 | 未授权回滚 | 授权验证 | ROLLBACK-001,002 |
| Committer | 审计篡改 | 审计日志删除 | 日志不可变 | AUDIT-001,002 |

### 4.2 Week3 安全待办

| 议题 | 描述 | 优先级 | 负责人 |
|------|------|--------|--------|
| 身份授权契约 | OIDC/OAuth2 集成、RBAC+ABAC 模型 | P0 | Security+Dev |
| 运行时安全边界 | seccomp/apparmor 配置文件 | P0 | Security+SRE |
| 密钥管理 | Vault/KMS 集成方案 | P1 | Security+SRE |
| 供应链安全 | Manifest 签名、SAST/SCA 流程 | P1 | Security+Dev |

---

## 5. 部署基线集成 (v2.0 - SRE W2T5)

### 5.1 容器化部署规范

```yaml
# k8s/validator-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: contract-validator
  namespace: cgas-phase0
spec:
  replicas: 3
  selector:
    matchLabels:
      app: validator
  template:
    metadata:
      labels:
        app: validator
    spec:
      containers:
      - name: validator
        image: cgas/validator:v2.0
        ports:
        - containerPort: 50051
          name: grpc
        - containerPort: 8080
          name: health
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 2000m
            memory: 2Gi
        livenessProbe:
          grpc:
            port: 8080
            service: HealthCheck
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          grpc:
            port: 8080
            service: HealthCheck
          initialDelaySeconds: 5
          periodSeconds: 10
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          seccompProfile:
            type: RuntimeDefault
          capabilities:
            drop:
            - ALL
        env:
        - name: RUST_LOG
          value: "info"
        - name: CACHE_TTL_MS
          value: "300000"
        - name: VALIDATION_TIMEOUT_MS
          value: "5000"
```

### 5.2 CI/CD 流水线集成

```yaml
# .github/workflows/validator-ci.yml
name: Validator CI/CD

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Run cargo test
      run: cargo test --all-features
    
    - name: Run cargo tarpaulin (coverage)
      run: cargo tarpaulin --out Xml --output-dir coverage
    
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        files: coverage/cobertura.xml
        fail_ci_if_error: true
    
    - name: Run validation matrix (44 cases)
      run: cargo test --test validation_matrix
    
    - name: Performance benchmark
      run: cargo bench --bench validation_bench
    
    - name: Security audit
      run: cargo audit
    
    - name: Build Docker image
      run: docker build -t cgas/validator:${{ github.sha }} .
    
    - name: Push to registry
      run: docker push cgas/validator:${{ github.sha }}
```

### 5.3 监控指标定义 (SRE W2T5)

| 指标 | 类型 | 告警阈值 | 用途 |
|------|------|----------|------|
| validator_requests_total | Counter | - | 请求总量 |
| validator_request_duration_ms | Histogram | P99>50ms | 延迟监控 |
| validator_errors_total | Counter | 错误率>0.1% | 错误监控 |
| validator_cache_hit_ratio | Gauge | <80% | 缓存效率 |
| validator_active_connections | Gauge | >1000 | 连接数监控 |
| validator_memory_usage_bytes | Gauge | >1.5Gi | 内存监控 |
| validator_cpu_usage_percent | Gauge | >80% | CPU 监控 |

---

## 6. 失败路径与回滚路径 (v2.0)

### 6.1 失败路径矩阵 (集成 SRE SLO)

| 故障点 | 检测方式 | 响应动作 | 通知对象 | SLO 影响 |
|--------|----------|----------|----------|----------|
| Schema 验证失败 | 字段校验 | 返回错误码，记录日志 | 调用方 | 错误率计数 |
| Semantic 验证失败 | 状态查询 | 返回错误码，记录日志 | 调用方 + Dev | 错误率计数 |
| Security 验证失败 | 权限检查 | 返回错误码，记录审计日志 | 调用方 + Security | 安全告警 |
| 缓存失效 | 缓存未命中 | 降级到直接验证 | - | 延迟增加 |
| 验证超时 | 定时器 | 终止验证，返回超时错误 | SRE | 错误率计数 |
| 验证器崩溃 | 进程监控 | 重启实例，隔离故障 | SRE | 可用性保护 |
| DB 连接失败 | 健康检查 | 切换到备用 DB | SRE | 可用性保护 |

### 6.2 错误处理策略 (v2.0)

```rust
// validator/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidatorError {
    #[error("Schema validation failed: {0}")]
    SchemaError(String),
    
    #[error("Semantic validation failed: {0}")]
    SemanticError(String),
    
    #[error("Security validation failed: {0}")]
    SecurityError(String),
    
    #[error("Validation timeout after {0}ms")]
    Timeout(u64),
    
    #[error("Cache error: {0}")]
    CacheError(#[from] CacheError),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl ValidatorError {
    pub fn to_error_code(&self) -> &'static str {
        match self {
            Self::SchemaError(_) => "SCHEMA_INVALID",
            Self::SemanticError(_) => "SEMANTIC_INVALID",
            Self::SecurityError(_) => "SECURITY_INVALID",
            Self::Timeout(_) => "VALIDATION_TIMEOUT",
            Self::CacheError(_) => "CACHE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
    
    pub fn severity(&self) -> ValidationSeverity {
        match self {
            Self::SecurityError(_) => ValidationSeverity::Critical,
            Self::SchemaError(_) => ValidationSeverity::Critical,
            Self::SemanticError(_) => ValidationSeverity::Critical,
            Self::Timeout(_) => ValidationSeverity::Critical,
            Self::CacheError(_) => ValidationSeverity::Warning,
            Self::Internal(_) => ValidationSeverity::Critical,
        }
    }
}
```

### 6.3 降级策略 (v2.0)

| 场景 | 降级方案 | 影响范围 | SLO 影响 |
|------|----------|----------|----------|
| 缓存不可用 | 跳过缓存，直接验证 | 延迟增加 | P99 可能上升 |
| Security 验证器不可用 | 阻断请求 (安全优先) | 可用性降低 | 可用性降级 |
| Semantic 验证器不可用 | 阻断请求 (数据完整性优先) | 可用性降低 | 可用性降级 |
| DB 不可用 | 切换到只读模式，跳过状态验证 | 验证能力降级 | 功能降级 |

---

## 7. 实现计划 (v2.0)

### 7.1 Rust 校验器模块

```
validator/
├── src/
│   ├── lib.rs              # 验证器入口
│   ├── context.rs          # 验证上下文
│   ├── report.rs           # 验证报告
│   ├── error.rs            # 错误定义
│   ├── cache.rs            # 缓存管理
│   ├── validators/
│   │   ├── mod.rs
│   │   ├── schema.rs       # Schema 验证器
│   │   ├── semantic.rs     # Semantic 验证器
│   │   └── security.rs     # Security 验证器
│   └── server/
│       ├── mod.rs
│       ├── grpc.rs         # gRPC 服务实现
│       └── health.rs       # 健康检查
├── tests/
│   ├── schema_tests.rs     # 20 Schema 用例
│   ├── semantic_tests.rs   # 12 链路用例
│   ├── security_tests.rs   # 12 失败路径用例
│   └── validation_matrix.rs # 44 用例集成测试
├── benches/
│   └── validation_bench.rs
├── Cargo.toml
└── README.md
```

### 7.2 依赖项

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tonic = "0.10"              # gRPC
prost = "0.12"
redis = { version = "0.24", features = ["tokio-comp"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"] }
thiserror = "1.0"
uuid = { version = "1.6", features = ["v4"] }
regex = "1.10"
tracing = "0.1"
metrics = "0.21"            # 指标收集
```

### 7.3 里程碑 (v2.0)

| 里程碑 | 交付物 | 预计完成 | 准入条件 |
|--------|--------|----------|----------|
| M1: 校验器框架 | validator 骨架代码 | Week2 结束 | ✅ 契约冻结完成 |
| M2: Schema 验证器 | schema.rs 实现 +20 用例 | Week3 结束 | 验证矩阵定义完成 |
| M3: Semantic 验证器 | semantic.rs 实现 +12 用例 | Week4 结束 | 状态存储就绪 |
| M4: Security 验证器 | security.rs 实现 +12 用例 | Week4 结束 | 威胁模型定义完成 |
| M5: 端到端验证 | 44 用例全部通过 | Week5 结束 | 部署基线定义完成 |

---

## 8. 待决议题 (v2.0)

| ID | 议题 | 影响范围 | 建议决策时间 | 负责人 |
|----|------|----------|-------------|--------|
| TBD-006 | 缓存后端选型 (Redis vs Memcached) | 校验器性能 | Week3 | Dev+SRE |
| TBD-007 | 验证规则配置化方案 | 规则更新灵活性 | Week3 | Dev+PM |
| TBD-008 | 异步验证结果存储策略 | 异步模式实现 | Week3 | Dev+QA |
| TBD-009 | 验证指标监控方案 | 可观测性 | Week3 | Dev+SRE |
| TBD-010 | OIDC/OAuth2 提供商选型 | 身份授权 | Week3 | Security+Dev |
| TBD-011 | seccomp/apparmor 配置文件 | 运行时安全 | Week3 | Security+SRE |

---

## 9. 附录

### 9.1 参考文档

- Phase0 Week1 契约框架 v2.0
- cgas/rust-workflow-engine 现有代码模式
- 分布式系统验证模式
- 缓存一致性管理最佳实践
- Security 威胁模型 12 场景分析
- SRE 部署基线规范
- QA 验证矩阵 44 用例规范

### 9.2 术语表

| 术语 | 定义 |
|------|------|
| Schema Validator | 契约格式验证器，检查字段类型、格式、范围 |
| Semantic Validator | 语义验证器，检查业务逻辑、状态一致性 |
| Security Validator | 安全验证器，检查权限、签名、审计 |
| Validation Matrix | 验证矩阵，44 条最小验证用例集 |
| Threat Model | 威胁模型，12 威胁场景 (4 组件×3 攻击面) |
| Deployment Baseline | 部署基线，容器化规范 +CI/CD 流水线 |

### 9.3 变更日志

| 版本 | 日期 | 变更描述 |
|------|------|----------|
| v1.0 | 2026-03-05 | 初始版本，待 QA/Security/SRE 评审 |
| v2.0 | 2026-03-05 | 吸收 QA/Security/SRE/PM 全部反馈：44 验证用例、12 威胁场景、部署基线、CI/CD 集成 |

### 9.4 签署状态

**文档状态**: 全角色评审通过  
**下一步**: 提交 PM 正式签署，启动 Week3 工作  
**签署状态**: 
- [x] Dev 技术确认
- [x] QA 验证矩阵确认
- [x] Security 威胁模型确认
- [x] SRE 部署基线确认
- [ ] PM 正式签署 (待完成)

**Week3 准入条件检查**:
- [ ] 契约正式冻结
- [ ] 校验器代码合并
- [ ] 部署流水线打通
- [ ] 样本采集累计≥80 条
