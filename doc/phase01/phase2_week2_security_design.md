# Phase 2 Week 2 安全设计文档

**版本**: v1.0  
**日期**: 2026-04-08  
**责任人**: Security  
**状态**: 📋 设计评审中  
**release_id**: release-2026-04-08-phase2_week02  

---

## 1. 概述

### 1.1 Week 2 安全目标

本周完成 SG-1~SG-4 Batch 扩展设计、零信任 OIDC 方案设计、扫描器优化方案设计。

| 安全任务 | 优先级 | 计划周次 | 状态 |
|---|---|---|---|
| SG-1~SG-4 Batch 扩展设计 | P0 | Week 2 | 📋 设计中 |
| 零信任 OIDC 方案设计 | P1 | Week 2 | 📋 设计中 |
| 扫描器优化方案设计 | P1 | Week 2 | 📋 设计中 |
| Batch 安全测试用例 | P1 | Week 2 | 📋 待开始 |

### 1.2 Phase 2 安全目标

| 目标 | Phase 1 基线 | Phase 2 目标 | 提升 |
|---|---|---|---|
| 未验证提交率 | 0% | 0% (红线保持) | - |
| SG-1~SG-4 验证通过率 | 100% | 100% (保持) | - |
| 扫描器误报率 | 3.2% | <2% | -37.5% |
| 高风险项 | 0 | 0 (保持) | - |
| 零信任架构 | 未接入 | OIDC/OAuth2+OPA | 新增 |

---

## 2. SG-1~SG-4 Batch 扩展设计

### 2.1 SG-1: Batch 提交路径验证

**Phase 1 功能**: 验证提交请求是否经过 Verifier

**Phase 2 Batch 扩展**:
- 验证 Batch 提交请求是否经过 Batch Verifier
- 验证 batch_hash 是否存在

**验证逻辑**:
```rust
impl PathValidator {
    pub async fn validate_batch(&self, request: &BatchCommitRequest) 
                                 -> Result<PathValidation, ValidationError> {
        // 1. 检查是否经过 Batch Verifier
        if !request.has_verifier_signature() {
            return Err(ValidationError::UnverifiedBatchPath);
        }
        
        // 2. 检查 batch_hash 是否存在
        if request.batch_hash.is_empty() {
            return Err(ValidationError::MissingBatchHash);
        }
        
        // 3. 验证 Verifier 签名
        if !self.verify_signature(&request.verifier_signature).await? {
            return Err(ValidationError::InvalidVerifierSignature);
        }
        
        Ok(PathValidation::Verified)
    }
}
```

**阻断动作**: 拒绝提交，记录审计日志

---

### 2.2 SG-2: Batch 隔离边界验证

**Phase 1 功能**: 验证执行器与验证器隔离

**Phase 2 Batch 扩展**:
- 验证 Batch 执行与验证隔离
- 验证原子性保证

**验证逻辑**:
```rust
impl IsolationValidator {
    pub async fn validate_batch_isolation(&self, request: &BatchExecuteRequest) 
                                           -> Result<IsolationValidation, ValidationError> {
        // 1. 验证 Batch 执行器与验证器隔离
        if !self.is_executor_isolated().await? {
            return Err(ValidationError::ExecutorNotIsolated);
        }
        
        // 2. 验证原子性标志
        if request.atomic {
            // 原子性模式：验证回滚机制就绪
            if !self.rollback机制_is_ready().await? {
                return Err(ValidationError::RollbackNotReady);
            }
        }
        
        Ok(IsolationValidation::Verified)
    }
}
```

**阻断动作**: 拒绝执行，告警

---

### 2.3 SG-3: Batch 哈希链验证

**Phase 1 功能**: 验证 result_hash 与 trace_hash 一致性

**Phase 2 Batch 扩展**:
- 验证 batch_hash 与子指令哈希链一致性
- 验证子指令 result_hash 完整性

**验证逻辑**:
```rust
impl HashValidator {
    pub async fn verify_batch_hash(&self, request: &BatchCommitRequest) 
                                    -> Result<bool, ValidationError> {
        // 1. 重新计算 batch_hash
        let computed_hash = compute_batch_hash(
            &request.instructions,
            &request.results,
        );
        
        // 2. 比对 hash
        if computed_hash != request.batch_hash {
            log::warn!(
                "Batch hash mismatch: computed={}, expected={}",
                computed_hash,
                request.batch_hash
            );
            return Ok(false);
        }
        
        // 3. 验证每个子指令的 result_hash
        for (i, result) in request.results.iter().enumerate() {
            if !self.verify_result_hash(result).await? {
                log::warn!("Sub-instruction {} hash verification failed", i);
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}
```

**阻断动作**: 拒绝提交，记录安全事件

---

### 2.4 SG-4: Batch 权限 + 重放检查

**Phase 1 功能**: 验证提交权限 + 检测重放攻击

**Phase 2 Batch 扩展**:
- 验证 Batch 提交权限
- 检测 Batch 重放攻击

**验证逻辑**:
```rust
impl PermissionValidator {
    pub async fn verify_batch_permission(&self, request: &BatchCommitRequest, 
                                          user: &User) 
                                          -> Result<bool, ValidationError> {
        // 1. 验证用户 Batch 提交权限
        if !user.has_permission("batch:commit").await? {
            return Err(ValidationError::InsufficientPermission);
        }
        
        // 2. 验证 Batch 资源权限
        for instruction in &request.instructions {
            if !self.verify_instruction_permission(instruction, user).await? {
                return Err(ValidationError::InsufficientInstructionPermission);
            }
        }
        
        Ok(true)
    }
}

impl ReplayDetector {
    pub async fn is_batch_replay(&self, request: &BatchCommitRequest) 
                                  -> Result<bool, ValidationError> {
        // 1. 检查 batch_id 是否已存在
        if self.batch_exists(&request.batch_id).await? {
            log::warn!("Batch replay detected: batch_id={}", request.batch_id);
            return Ok(true);
        }
        
        // 2. 检查 batch_hash 是否已存在 (相同内容不同 ID)
        if self.batch_hash_exists(&request.batch_hash).await? {
            log::warn!("Batch hash replay detected: hash={}", request.batch_hash);
            return Ok(true);
        }
        
        Ok(false)
    }
}
```

**阻断动作**: 拒绝提交，记录安全事件，告警

---

## 3. 零信任 OIDC 方案设计

### 3.1 架构设计

```
Phase 2 零信任架构:
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │───▶│   OIDC      │───▶│   OPA       │
│  (请求)     │    │   Provider  │    │  (策略引擎) │
└─────────────┘    └─────────────┘    └─────────────┘
                        │                    │
                        ▼                    ▼
                 ┌─────────────┐    ┌─────────────┐
                 │   JWKS      │    │   RBAC+     │
                 │   (密钥)    │    │   ABAC      │
                 └─────────────┘    └─────────────┘
```

### 3.2 实施计划

| 阶段 | 内容 | 周次 | 交付物 | 状态 |
|---|---|---|---|---|
| 阶段 1 | OIDC 身份验证 | Week 2 | OIDC 方案文档 | 📋 设计中 |
| 阶段 2 | RBAC+ABAC 授权模型 | Week 3 | 授权模块 | 📋 待开始 |
| 阶段 3 | OPA 策略引擎集成 | Week 3 | 策略配置 | 📋 待开始 |
| 阶段 4 | JWKS 密钥管理 | Week 4 | 密钥轮换配置 | 📋 待开始 |
| 阶段 5 | 全链路零信任验证 | Week 4 | 零信任测试报告 | 📋 待开始 |

### 3.3 OIDC 身份验证流程

```
1. Client 请求 Token
   │
   ▼
2. OIDC Provider 验证用户身份
   │
   ▼
3. 颁发 JWT Token (含用户声明)
   │
   ▼
4. Client 携带 Token 请求 API
   │
   ▼
5. Gateway 验证 JWT 签名
   │
   ▼
6. 提取用户声明，传递给 OPA
   │
   ▼
7. OPA 执行授权策略
   │
   ▼
8. 允许/拒绝请求
```

### 3.4 JWT Token 声明

```json
{
  "sub": "user_123",
  "iss": "https://auth.example.com",
  "aud": "cgas-api",
  "exp": 1712000000,
  "iat": 1711996400,
  "roles": ["developer", "batch_user"],
  "permissions": ["batch:execute", "batch:read"],
  "client_id": "client_456"
}
```

### 3.5 OPA 策略示例

```rego
package cgas.authz

default allow = false

# 允许 admin 角色执行任何操作
allow {
    input.user.roles[_] == "admin"
}

# 允许 batch_user 角色执行 Batch
allow {
    input.user.roles[_] == "batch_user"
    input.action == "batch:execute"
    input.resource.environment == "staging"
}

# 允许 developer 角色执行指令
allow {
    input.user.roles[_] == "developer"
    input.action == "execute"
    input.resource.environment == "staging"
}

# 禁止生产环境 Batch 操作 (Phase 2 限制)
deny {
    input.action == "batch:execute"
    input.resource.environment == "production"
}
```

---

## 4. 扫描器优化方案设计

### 4.1 Phase 1 扫描器状态

| 指标 | Phase 1 值 | 分析 |
|---|---|---|
| 非确定性路径识别 | 127 路径 (100%) | ✅ 覆盖完整 |
| 误报率 | 3.2% (4/127) | ⚠️ 需优化至<2% |
| 漏报率 | 0% | ✅ 无漏报 |

### 4.2 误报根因分析

| 误报类型 | 数量 | 根因 | 优化措施 |
|---|---|---|---|
| 并发竞争误判 | 2 | 锁粒度判断过严 | 优化锁检测算法 |
| 时序依赖误判 | 2 | 时间窗口过窄 | 放宽时间窗口阈值 |

### 4.3 扫描器优化方案

#### 4.3.1 锁检测算法优化

**Phase 1 问题**: 将所有共享资源访问判为并发竞争

**Phase 2 优化**:
```rust
// Phase 1: 简单判断
if resource.is_shared() {
    return NonDeterministic::ConcurrencyRace;
}

// Phase 2: 精细化判断
if resource.is_shared() {
    // 检查是否有锁保护
    if let Some(lock) = resource.get_lock() {
        // 检查锁粒度是否匹配
        if lock.covers_operation(&operation) {
            return NonDeterministic::None; // 有锁保护，非并发竞争
        }
    }
    return NonDeterministic::ConcurrencyRace;
}
```

**预期效果**: 误报率从 3.2% 降至 2.5%

#### 4.3.2 时间窗口阈值调整

**Phase 1 问题**: 时间窗口过窄 (1ms)，将正常时序依赖判为非确定性

**Phase 2 优化**:
```rust
// Phase 1: 固定 1ms 窗口
const TIME_WINDOW_MS: u64 = 1;

// Phase 2: 动态窗口
fn get_time_window(operation: &Operation) -> u64 {
    match operation.type {
        OperationType::Read => 5,      // 读操作 5ms 窗口
        OperationType::Write => 10,    // 写操作 10ms 窗口
        OperationType::Batch => 50,    // Batch 操作 50ms 窗口
    }
}
```

**预期效果**: 误报率从 2.5% 降至<2%

### 4.4 扫描器优化实施计划

| 周次 | 优化项 | 目标误报率 | 验证方法 | 状态 |
|---|---|---|---|---|
| Week 2 | 锁检测算法优化 | <2.5% | 回归测试 | 📋 设计中 |
| Week 3 | 时间窗口阈值调整 | <2% | 回归测试 | 📋 待开始 |
| Week 4 | 扫描器规则优化 | <2% | 对抗测试 | 📋 待开始 |

---

## 5. Batch 安全测试用例

### 5.1 安全闸门测试 (4 个)

| 用例 ID | 用例描述 | 预期结果 | 优先级 |
|---|---|---|---|
| SEC-BATCH-001 | SG-1 Batch 路径验证 | 未验证 Batch 被阻断 | P0 |
| SEC-BATCH-002 | SG-3 Batch 哈希验证 | 篡改 hash 被阻断 | P0 |
| SEC-BATCH-003 | SG-4 Batch 重放检查 | 重放 Batch 被阻断 | P0 |
| SEC-BATCH-004 | Batch 原子性违反告警 | 违反触发告警 | P0 |

### 5.2 零信任安全测试 (2 个)

| 用例 ID | 用例描述 | 预期结果 | 优先级 |
|---|---|---|---|
| SEC-OIDC-001 | OIDC Token 验证 | 无效 Token 被拒绝 | P0 |
| SEC-OIDC-002 | OPA 授权策略 | 越权请求被拒绝 | P0 |

---

## 6. 安全风险管理

### 6.1 Week 2 安全风险

| 风险 ID | 风险描述 | 影响等级 | 缓解计划 | 状态 |
|---|---|---|---|---|
| R-SEC-W2-001 | SG-1~SG-4 Batch 扩展引入漏洞 | 中 | 代码审查 + 渗透测试 | 🟡 监控中 |
| R-SEC-W2-002 | OIDC 集成配置错误 | 中 | 配置审查 + 测试环境验证 | 🟡 监控中 |
| R-SEC-W2-003 | 扫描器优化引入漏报 | 中 | 回归测试 + 对抗验证 | 🟡 监控中 |

### 6.2 安全评审计划

| 评审类型 | 频率 | 参与方 | 输出 |
|---|---|---|---|
| 安全日检 | 每日 | Security | 安全状态更新 |
| 安全周评 | 每周 | Security+Dev | 安全周报 |
| 代码审计 | Week 2/4 | Security+Dev | 审计报告 |
| 渗透测试 | Week 5 | 外部 +Security | 渗透报告 |

---

## 7. 附录

### 7.1 术语表

| 术语 | 定义 |
|---|---|
| OIDC | OpenID Connect，身份验证协议 |
| OAuth2 | 授权框架 |
| OPA | Open Policy Agent，策略引擎 |
| JWKS | JSON Web Key Set，公钥集合 |
| JWT | JSON Web Token，令牌格式 |

### 7.2 参考文档

- Phase 2 安全规划 v1
- Phase 2 Batch 设计文档
- Phase 1 Security 闸门报告

---

**文档状态**: 📋 设计评审中  
**评审计划**: Week 2-T3 安全设计评审会议  
**责任人**: Security  
**保管**: 项目文档库
