# OIDC + OPA 集成规范 (OIDC + OPA Integration Specification)

**Release ID**: release-2026-05-19-phase3_week02  
**版本**: v1.0  
**编制日期**: 2026-05-19  
**责任人**: Security Agent  
**状态**: ✅ 完成  
**审查**: PM 📋 | Dev 📋 | QA 📋 | SRE 📋 | Security ✅

---

## 一、执行摘要

### 1.1 集成目标

本规范定义 Phase 3 Week 2 OIDC 身份验证与 OPA 授权策略引擎的集成方案，实现:
1. **身份 - 授权联动**: OIDC Token 声明自动映射至 OPA 输入
2. **动态策略评估**: 基于用户属性 + 环境属性的实时授权决策
3. **细粒度权限控制**: 字段级/行级权限验证
4. **策略热加载**: OPA 策略变更<5s 生效
5. **审计日志联动**: 授权决策 100% 记录审计日志

### 1.2 集成范围

| 组件 | Phase 2 状态 | Phase 3 Week 2 增强 | 优先级 |
|---|---|---|---|
| OIDC → OPA 数据流 | 基础映射 | 完整声明映射 + 缓存 | P0 |
| 策略评估延迟 | 35ms | **<15ms** | P0 |
| 动态属性支持 | 有限 | 完整运行时属性 | P0 |
| 字段级权限 | ❌ | ✅ 支持 | P1 |
| 行级权限 | ❌ | ✅ 支持 | P1 |
| 策略热加载 | ❌ | ✅ <5s 生效 | P1 |

### 1.3 关键指标

| 指标 | Phase 2 基线 | Phase 3 目标 | 优化幅度 |
|---|---|---|---|
| 策略评估延迟 | 35ms | **<15ms** | -57% |
| 策略缓存命中率 | N/A | **≥90%** | 新增能力 |
| 字段级权限覆盖 | 0% | **100%** | 新增能力 |
| 行级权限覆盖 | 0% | **100%** | 新增能力 |
| 策略热加载时间 | N/A | **<5s** | 新增能力 |
| 审计日志覆盖率 | 100% | **100%** (实时) | 保持 |

---

## 二、架构设计

### 2.1 整体架构

```
OIDC + OPA 集成架构:
┌─────────────────────────────────────────────────────────────────┐
│                         Request Flow                             │
│                                                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐          │
│  │   Client    │───▶│   Gateway   │───▶│   OIDC      │          │
│  │  (请求)     │    │  (入口层)   │    │  (身份验证) │          │
│  └─────────────┘    └─────────────┘    └─────────────┘          │
│                            │                    │                │
│                            │                    ▼                │
│                            │          ┌─────────────┐            │
│                            │          │   Token     │            │
│                            │          │  (Claims)   │            │
│                            │          └─────────────┘            │
│                            │                    │                │
│                            ▼                    ▼                │
│                     ┌─────────────────────────────┐              │
│                     │      OPA Integration        │              │
│                     │      (身份 - 授权联动)       │              │
│                     └─────────────────────────────┘              │
│                            │                                      │
│                            ▼                                      │
│                     ┌─────────────┐                               │
│                     │   OPA       │                               │
│                     │  (策略评估) │                               │
│                     └─────────────┘                               │
│                            │                                      │
│                            ▼                                      │
│                     ┌─────────────┐                               │
│                     │   Decision  │                               │
│                     │ (Allow/Deny)│                               │
│                     └─────────────┘                               │
│                            │                                      │
│                            ▼                                      │
│                     ┌─────────────┐                               │
│                     │   Backend   │                               │
│                     │  (服务执行) │                               │
│                     └─────────────┘                               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 数据流

```
OIDC → OPA 数据流:

1. 客户端请求 (携带 OIDC Token)
   │
   ▼
2. Gateway 提取 Token
   │
   ▼
3. OIDC 验证 (缓存优先)
   │   └─ 返回 Claims: {sub, roles, permissions, attributes...}
   │
   ▼
4. 构建 OPA 输入
   │   input = {
   │     "user": {
   │       "id": claims.sub,
   │       "roles": claims.roles,
   │       "permissions": claims.permissions,
   │       "attributes": claims.custom_attributes
   │     },
   │     "action": request.method + ":" + request.path,
   │     "resource": {
   │       "type": resource_type,
   │       "id": resource_id,
   │       "owner": resource_owner,
   │       "sensitivity": resource_sensitivity
   │     },
   │     "context": {
   │       "time": current_time,
   │       "ip": client_ip,
   │       "location": geo_location,
   │       "device_trust": device_trust_score
   │     }
   │   }
   │
   ▼
5. OPA 策略评估
   │   └─ 返回 Decision: {allow: bool, reason: string, fields: [...]}
   │
   ▼
6. 授权决策执行
   │   ├─ Allow → 转发请求至后端
   │   └─ Deny → 返回 403 + 审计日志
   │
   ▼
7. 审计日志记录
   │   └─ 记录：user, action, resource, decision, timestamp
```

### 2.3 组件职责

| 组件 | 职责 | 技术选型 | 接口契约 |
|---|---|---|---|
| OIDC Middleware | Token 验证、声明提取 | Rust (自定义) | Middleware Trait |
| OPA Client | 策略评估、结果缓存 | Rust (OPA SDK) | OPA Client Trait |
| Policy Engine | 策略编译、热加载 | OPA Bundle Service | Bundle API |
| Audit Logger | 授权决策审计 | Fluentd + ES | Structured Logging |
| Attribute Store | 动态属性查询 | Redis + PostgreSQL | Query API |

---

## 三、OIDC 声明映射

### 3.1 标准声明映射

```rust
// OIDC 声明 → OPA 输入映射
#[derive(Debug, Clone, serde::Serialize)]
pub struct OpaInput {
    pub user: UserContext,
    pub action: Action,
    pub resource: Resource,
    pub context: Context,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UserContext {
    pub id: String,              // OIDC: sub
    pub email: String,           // OIDC: email
    pub name: String,            // OIDC: name
    pub roles: Vec<String>,      // OIDC: custom: roles
    pub permissions: Vec<String>, // OIDC: custom: permissions
    pub attributes: UserAttributes,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UserAttributes {
    pub department: Option<String>,      // OIDC: custom: department
    pub clearance: Option<String>,       // OIDC: custom: clearance
    pub manager: Option<String>,         // OIDC: custom: manager
    pub location: Option<String>,        // OIDC: custom: location
    pub employee_type: Option<String>,   // OIDC: custom: employee_type
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Action {
    pub method: String,          // HTTP method
    pub path: String,            // Request path
    pub operation: String,       // 业务操作 (e.g., "batch:execute")
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Resource {
    pub r#type: String,          // 资源类型 (e.g., "batch", "transaction")
    pub id: Option<String>,      // 资源 ID
    pub owner: Option<String>,   // 资源所有者
    pub sensitivity: String,     // 敏感度 (public/internal/confidential/restricted)
    pub environment: String,     // 环境 (dev/staging/prod)
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Context {
    pub time: u64,               // Unix timestamp
    pub day_of_week: u8,         // 1-7 (Monday-Sunday)
    pub is_business_hours: bool,
    pub ip: String,              // Client IP
    pub location: Option<GeoLocation>,
    pub device_trust: DeviceTrust,
    pub risk_score: u8,          // 0-100
}
```

### 3.2 自定义声明扩展

```yaml
# OIDC 自定义声明配置
custom_claims:
  # 角色声明
  roles:
    claim_path: "cgas.roles"
    type: array<string>
    default: ["viewer"]
    
  # 权限声明
  permissions:
    claim_path: "cgas.permissions"
    type: array<string>
    default: []
    
  # 用户属性
  attributes:
    department:
      claim_path: "cgas.department"
      type: string
      default: null
    clearance:
      claim_path: "cgas.clearance"
      type: string
      enum: [public, internal, confidential, restricted]
      default: public
    manager:
      claim_path: "cgas.manager"
      type: string
      default: null
    location:
      claim_path: "cgas.location"
      type: string
      default: null
      
  # 设备信任
  device_trust:
    claim_path: "cgas.device.trust_score"
    type: integer
    range: [0, 100]
    default: 50
```

### 3.3 声明验证规则

```rust
// 声明验证实现
pub struct ClaimValidator {
    schema: ClaimSchema,
}

impl ClaimValidator {
    /// 验证 OIDC 声明完整性
    pub fn validate_claims(&self, claims: &JwtClaims) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        
        // 必需声明验证
        if claims.sub.is_empty() {
            errors.push("Missing required claim: sub");
        }
        
        // 角色声明验证
        if let Some(roles) = &claims.custom.get("cgas.roles") {
            if !self.validate_roles(roles) {
                errors.push("Invalid roles format");
            }
        }
        
        // 权限声明验证
        if let Some(permissions) = &claims.custom.get("cgas.permissions") {
            if !self.validate_permissions(permissions) {
                errors.push("Invalid permissions format");
            }
        }
        
        // 清除度验证
        if let Some(clearance) = &claims.custom.get("cgas.clearance") {
            if !self.validate_clearance(clearance) {
                errors.push("Invalid clearance level");
            }
        }
        
        if errors.is_empty() {
            Ok(ValidationResult::Valid)
        } else {
            Ok(ValidationResult::Invalid { errors })
        }
    }
}
```

---

## 四、OPA 策略设计

### 4.1 策略包结构

```
opa_policies/
├── bundle.yaml                 # OPA Bundle 配置
├── manifest.json               # Bundle 清单
├── cgas/
│   ├── authz/
│   │   ├── main.rego           # 主授权策略
│   │   ├── rbac.rego           # RBAC 策略
│   │   ├── abac.rego           # ABAC 策略
│   │   ├── field_level.rego    # 字段级权限
│   │   ├── row_level.rego      # 行级权限
│   │   └── time_based.rego     # 时间衰减策略
│   ├── validation/
│   │   ├── input.rego          # 输入验证
│   │   └── output.rego         # 输出验证
│   └── test/
│       ├── authz_test.rego     # 授权测试
│       └── fixtures/           # 测试数据
└── data/
    ├── roles.yaml              # 角色定义
    ├── permissions.yaml        # 权限定义
    └── resources.yaml          # 资源定义
```

### 4.2 主授权策略

```rego
# cgas/authz/main.rego
package cgas.authz

import future.keywords.if
import future.keywords.in

# 默认拒绝
default allow = false

# 主授权决策
allow if {
    # 输入验证通过
    valid_input
    # 未吊销
    not is_revoked
    # RBAC 或 ABAC 允许
    rbac_allow
    abac_allow
    # 字段级权限验证 (如适用)
    field_level_check
    # 行级权限验证 (如适用)
    row_level_check
}

# 输入验证
valid_input if {
    # 用户 ID 存在
    input.user.id
    # 操作定义
    input.action.operation
    # 资源类型定义
    input.resource.type
}

# 吊销检查
is_revoked if {
    # 用户在吊销列表中
    revoked_users[input.user.id]
}

# 决策解释
decision := {
    "allow": allow,
    "reason": get_reason,
    "fields": allowed_fields,
    "filter": row_filter,
}

get_reason := "admin_override" if {
    "admin" in input.user.roles
}

get_reason := "rbac_allow" if {
    rbac_allow
    not "admin" in input.user.roles
}

get_reason := "abac_allow" if {
    abac_allow
    not rbac_allow
}

get_reason := "denied"
```

### 4.3 RBAC 策略

```rego
# cgas/authz/rbac.rego
package cgas.authz

# 角色权限映射
role_permissions := {
    "admin": ["*:*"],
    "developer": ["batch:*", "transaction:read", "verification:*"],
    "viewer": ["batch:read", "transaction:read", "verification:read"],
    "operator": ["batch:execute", "transaction:execute"],
}

# RBAC 允许
rbac_allow if {
    some role in input.user.roles
    some required_perm in required_permissions
    has_permission(role, required_perm)
}

# 检查角色是否有权限
has_permission(role, required_perm) if {
    some perm in role_permissions[role]
    perm_matches(perm, required_perm)
}

# 权限匹配 (支持通配符)
perm_matches(perm, required) if {
    # 完全匹配
    perm == required
}

perm_matches(perm, required) if {
    # 通配符匹配 (e.g., "batch:*" matches "batch:execute")
    startswith(perm, prefix)
    startswith(required, prefix)
    prefix := concat(":", [split(perm, ":")[0], ""])
}

# 获取所需权限
required_permissions := {
    "batch:execute": ["batch:execute"],
    "batch:read": ["batch:read"],
    "batch:delete": ["batch:delete"],
    "transaction:execute": ["transaction:execute"],
    "transaction:read": ["transaction:read"],
    "verification:execute": ["verification:execute"],
    "verification:read": ["verification:read"],
}
```

### 4.4 ABAC 策略

```rego
# cgas/authz/abac.rego
package cgas.authz

# ABAC 允许
abac_allow if {
    # 资源所有者允许访问自己的资源
    is_owner
}

abac_allow if {
    # 清除度匹配
    clearance_match
}

abac_allow if {
    # 部门匹配 (同部门可访问)
    department_match
}

abac_allow if {
    # 业务时间允许
    business_hours_allow
}

# 所有者检查
is_owner if {
    input.user.id == input.resource.owner
}

# 清除度检查
clearance_match if {
    user_clearance_level >= resource_clearance_level
}

user_clearance_level := level if {
    level := {
        "public": 1,
        "internal": 2,
        "confidential": 3,
        "restricted": 4,
    }[input.user.attributes.clearance]
}

resource_clearance_level := level if {
    level := {
        "public": 1,
        "internal": 2,
        "confidential": 3,
        "restricted": 4,
    }[input.resource.sensitivity]
}

# 部门匹配
department_match if {
    input.user.attributes.department == input.resource.labels["department"]
}

# 业务时间检查
business_hours_allow if {
    input.context.is_business_hours
}

business_hours_allow if {
    # 非业务时间需要管理员权限
    "admin" in input.user.roles
}
```

### 4.5 字段级权限策略

```rego
# cgas/authz/field_level.rego
package cgas.authz

# 字段级权限检查
field_level_check if {
    # 如果没有请求特定字段，默认通过
    not input.requested_fields
}

field_level_check if {
    # 所有请求的字段都在允许列表中
    every field in input.requested_fields {
        is_field_allowed(field)
    }
}

# 检查字段是否允许
is_field_allowed(field) if {
    # 管理员允许访问所有字段
    "admin" in input.user.roles
}

is_field_allowed(field) if {
    # Developer 允许访问的字段
    "developer" in input.user.roles
    field in developer_allowed_fields
}

is_field_allowed(field) if {
    # Viewer 允许访问的字段
    "viewer" in input.user.roles
    field in viewer_allowed_fields
}

# 允许字段定义
developer_allowed_fields := ["id", "status", "created_at", "owner", "commands", "results"]

viewer_allowed_fields := ["id", "status", "created_at", "owner"]

# 敏感字段 (仅管理员)
sensitive_fields := ["internal_config", "security_context", "encryption_keys", "audit_logs"]

# 拒绝访问敏感字段 (非管理员)
deny_field if {
    some field in input.requested_fields
    field in sensitive_fields
    not "admin" in input.user.roles
}

# 返回允许的字段
allowed_fields := field_list if {
    field_list := [field | 
        field := input.requested_fields[_]
        is_field_allowed(field)
    ]
}

allowed_fields := [] if {
    not input.requested_fields
}
```

### 4.6 行级权限策略

```rego
# cgas/authz/row_level.rego
package cgas.authz

# 行级权限检查
row_level_check if {
    # 如果没有行级过滤要求，默认通过
    not input.row_filter_required
}

row_level_check if {
    # 应用行级过滤
    row_filter_applied
}

# 行级过滤应用
row_filter_applied if {
    # 管理员无限制
    "admin" in input.user.roles
}

row_filter_applied if {
    # Developer 只能访问自己的资源
    "developer" in input.user.roles
    input.resource.owner == input.user.id
}

row_filter_applied if {
    # Viewer 只能访问自己创建的资源
    "viewer" in input.user.roles
    input.resource.owner == input.user.id
}

# 返回行级过滤器
row_filter := filter if {
    "admin" in input.user.roles
    filter := {}  # 无限制
}

row_filter := filter if {
    not "admin" in input.user.roles
    filter := {"owner": input.user.id}  # 只允许访问自己的资源
}
```

### 4.7 时间衰减策略

```rego
# cgas/authz/time_based.rego
package cgas.authz

# 权限时间衰减
permission_decay_factor := factor if {
    hours_since_grant := (time.now_ns() / 1000000000 - input.permission.granted_at) / 3600
    factor := calculate_decay(hours_since_grant)
}

calculate_decay(hours) := 1.0 if hours <= 2    # 0-2 小时：100%
calculate_decay(hours) := 0.75 if hours <= 4   # 2-4 小时：75%
calculate_decay(hours) := 0.5 if hours <= 8    # 4-8 小时：50%
calculate_decay(hours) := 0.25 if hours <= 12  # 8-12 小时：25%
calculate_decay(hours) := 0.0                  # >12 小时：0% (需续期)

# 时间衰减检查
time_decay_check if {
    # 管理员不受时间衰减影响
    "admin" in input.user.roles
}

time_decay_check if {
    # 普通用户检查时间衰减
    permission_decay_factor > 0
}

# 需要续期
needs_renewal if {
    permission_decay_factor == 0
}
```

---

## 五、策略热加载

### 5.1 Bundle Service 架构

```
OPA Bundle Service 架构:
┌─────────────────────────────────────────────────────────────────┐
│                      Policy Git Repository                       │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  opa_policies/                                          │    │
│  │    ├── bundle.yaml                                      │    │
│  │    ├── cgas/authz/*.rego                                │    │
│  │    └── data/*.yaml                                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│                            │                                     │
│                            │ (Git Push)                          │
│                            ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  CI/CD Pipeline                          │    │
│  │  • 策略语法检查 (opa check)                             │    │
│  │  • 策略测试 (opa test)                                  │    │
│  │  • Bundle 打包                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                            │                                     │
│                            │ (Bundle 推送)                        │
│                            ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              OPA Bundle Service                          │    │
│  │  • Bundle 存储                                           │    │
│  │  • 版本管理                                              │    │
│  │  • 分发服务                                              │    │
│  └─────────────────────────────────────────────────────────┘    │
│                            │                                     │
│              ┌─────────────┼─────────────┐                      │
│              ▼             ▼             ▼                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                │
│  │   OPA #1    │ │   OPA #2    │ │   OPA #3    │                │
│  │  (Gateway)  │ │  (Service)  │ │  (Service)  │                │
│  └─────────────┘ └─────────────┘ └─────────────┘                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Bundle 配置

```yaml
# bundle.yaml
revision: v1.0.0
roots:
  - cgas
  - data

metadata:
  version: 1.0.0
  timestamp: "2026-05-19T10:30:00Z"
  author: "security-agent"
  description: "Phase 3 Week 2 OPA Policies"

signing:
  algorithm: HS256
  key: "${OPA_SIGNING_KEY}"
```

### 5.3 热加载实现

```rust
// OPA Bundle Service 客户端
pub struct BundleServiceClient {
    base_url: String,
    client: reqwest::Client,
    current_revision: RwLock<String>,
}

impl BundleServiceClient {
    /// 检查 Bundle 更新
    pub async fn check_for_updates(&self) -> Result<Option<BundleInfo>> {
        let response = self.client
            .get(format!("{}/bundles/cgas/latest", self.base_url))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(Error::BundleServiceUnavailable);
        }
        
        let bundle_info: BundleInfo = response.json().await?;
        
        // 检查是否有新版本
        let current = self.current_revision.read().await;
        if bundle_info.revision != *current {
            Ok(Some(bundle_info))
        } else {
            Ok(None)
        }
    }
    
    /// 下载并加载新 Bundle
    pub async fn load_bundle(&self, bundle_info: &BundleInfo) -> Result<()> {
        // 下载 Bundle
        let bundle_data = self.download_bundle(&bundle_info.download_url).await?;
        
        // 验证签名
        self.verify_signature(&bundle_data, &bundle_info.signature).await?;
        
        // 解压 Bundle
        let policies = self.extract_bundle(&bundle_data).await?;
        
        // 热加载到 OPA
        self.opa_client.update_policies(policies).await?;
        
        // 更新当前版本
        let mut current = self.current_revision.write().await;
        *current = bundle_info.revision.clone();
        
        log::info!("Bundle loaded: {}", bundle_info.revision);
        
        Ok(())
    }
}
```

### 5.4 热加载性能指标

| 指标 | 目标值 | 测量方法 | 状态 |
|---|---|---|---|
| Bundle 下载时间 | <2s | 端到端测量 | 📋 待验证 |
| 策略编译时间 | <2s | OPA 内部指标 | 📋 待验证 |
| 策略生效延迟 | <5s | 变更→生效时间 | 📋 待验证 |
| 加载过程影响 | 零中断 | 请求成功率 | 📋 待验证 |
| 回滚时间 | <10s | 回滚操作时间 | 📋 待验证 |

---

## 六、性能优化

### 6.1 策略评估缓存

```rust
// OPA 评估缓存
pub struct PolicyCache {
    // 评估结果缓存
    cache: DashMap<CacheKey, CacheEntry>,
    
    // 缓存配置
    config: CacheConfig,
    
    // 统计信息
    stats: CacheStats,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub user_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub context_hash: u64,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub decision: bool,
    pub fields: Vec<String>,
    pub filter: HashMap<String, String>,
    pub expires_at: u64,
    pub created_at: Instant,
}

impl PolicyCache {
    /// 查询缓存
    pub async fn get(&self, key: &CacheKey) -> Option<CacheEntry> {
        if let Some(entry) = self.cache.get(key) {
            if !entry.is_expired() {
                self.stats.hits.fetch_add(1, Ordering::Relaxed);
                return Some(entry.clone());
            } else {
                self.cache.remove(key);
            }
        }
        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        None
    }
    
    /// 缓存评估结果
    pub async fn set(&self, key: CacheKey, entry: CacheEntry) {
        self.cache.insert(key, entry);
    }
}
```

### 6.2 动态属性预取

```rust
// 动态属性预取器
pub struct AttributePrefetcher {
    cache: AttributeCache,
    client: AttributeClient,
}

impl AttributePrefetcher {
    /// 预取用户属性
    pub async fn prefetch_user_attributes(&self, user_id: &str) -> Result<UserAttributes> {
        // 检查缓存
        if let Some(attrs) = self.cache.get_user(user_id).await {
            return Ok(attrs);
        }
        
        // 从 IAM 系统获取
        let attrs = self.client.get_user_attributes(user_id).await?;
        
        // 缓存 (TTL: 5min)
        self.cache.set_user(user_id, attrs.clone(), Duration::from_secs(300)).await;
        
        Ok(attrs)
    }
    
    /// 预取资源属性
    pub async fn prefetch_resource_attributes(&self, resource_id: &str) -> Result<ResourceAttributes> {
        // 检查缓存
        if let Some(attrs) = self.cache.get_resource(resource_id).await {
            return Ok(attrs);
        }
        
        // 从资源目录获取
        let attrs = self.client.get_resource_attributes(resource_id).await?;
        
        // 缓存 (TTL: 1min)
        self.cache.set_resource(resource_id, attrs.clone(), Duration::from_secs(60)).await;
        
        Ok(attrs)
    }
}
```

### 6.3 性能优化目标

| 指标 | Phase 2 基线 | Phase 3 目标 | 优化措施 |
|---|---|---|---|
| 策略评估延迟 | 35ms | **<15ms** | 缓存 + 预取 |
| 策略缓存命中率 | N/A | **≥90%** | 智能缓存 |
| 外部数据查询延迟 | 50ms | **<10ms** | 数据预取 |
| 并发评估能力 | 1000/s | **≥5000/s** | 异步评估 |
| Bundle 加载时间 | N/A | **<5s** | 增量加载 |

---

## 七、审计日志

### 7.1 审计日志格式

```json
{
  "timestamp": "2026-05-19T10:30:00.123Z",
  "event_type": "authorization_decision",
  "user": {
    "id": "user_123",
    "email": "user@example.com",
    "roles": ["developer"]
  },
  "action": {
    "method": "POST",
    "path": "/api/v1/batch/execute",
    "operation": "batch:execute"
  },
  "resource": {
    "type": "batch",
    "id": "batch_456",
    "owner": "user_123",
    "sensitivity": "internal"
  },
  "decision": {
    "allow": true,
    "reason": "rbac_allow",
    "fields": ["id", "status", "commands"],
    "policy_version": "v1.0.0"
  },
  "context": {
    "ip": "192.168.1.100",
    "location": "CN-SH",
    "device_trust": 85,
    "risk_score": 15
  },
  "latency_ms": 12,
  "opa_instance": "opa-gateway-01"
}
```

### 7.2 审计日志字段

| 字段 | 类型 | 说明 | 示例 |
|---|---|---|---|
| timestamp | Timestamp | 事件时间 | 2026-05-19T10:30:00.123Z |
| event_type | String | 事件类型 | authorization_decision |
| user.id | String | 用户 ID | user_123 |
| user.roles | Array | 用户角色 | ["developer"] |
| action.operation | String | 操作类型 | batch:execute |
| resource.type | String | 资源类型 | batch |
| resource.id | String | 资源 ID | batch_456 |
| decision.allow | Boolean | 授权结果 | true |
| decision.reason | String | 决策原因 | rbac_allow |
| decision.policy_version | String | 策略版本 | v1.0.0 |
| context.ip | String | 客户端 IP | 192.168.1.100 |
| latency_ms | Integer | 评估延迟 | 12 |
| opa_instance | String | OPA 实例 | opa-gateway-01 |

### 7.3 审计日志流

```
审计日志流:
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   OPA       │───▶│   Fluentd   │───▶│   Kafka     │
│  (决策日志) │    │  (采集器)   │    │  (消息队列) │
└─────────────┘    └─────────────┘    └─────────────┘
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    ▼                     ▼                     ▼
             ┌─────────────┐       ┌─────────────┐       ┌─────────────┐
             │  Elasticsearch│      │   SIEM      │       │  Alerting   │
             │  (存储/查询)  │      │  (安全分析) │       │  (告警)     │
             └─────────────┘       └─────────────┘       └─────────────┘
```

---

## 八、实施计划

### 8.1 Week 2 任务分解

| 任务 ID | 任务描述 | 交付物 | 优先级 | 工时 |
|---|---|---|---|---|
| T-INT-01 | OIDC 声明映射实现 | claim_mapping.rs | P0 | 3h |
| T-INT-02 | OPA 输入构建器 | opa_input_builder.rs | P0 | 3h |
| T-INT-03 | RBAC 策略实现 | rbac.rego | P0 | 2h |
| T-INT-04 | ABAC 策略实现 | abac.rego | P0 | 3h |
| T-INT-05 | 字段级权限策略 | field_level.rego | P1 | 2h |
| T-INT-06 | 行级权限策略 | row_level.rego | P1 | 2h |
| T-INT-07 | Bundle Service 配置 | bundle_service.yml | P1 | 2h |
| T-INT-08 | 审计日志集成 | audit_logger.rs | P1 | 2h |
| T-INT-09 | 性能优化 (缓存 + 预取) | policy_cache.rs | P1 | 3h |

### 8.2 验收标准

| 验收项 | 验收标准 | 验证方法 | 状态 |
|---|---|---|---|
| OIDC 声明映射 | 100% 字段映射 | 映射测试 | 📋 待验证 |
| 策略评估延迟 | P99<15ms | 性能压测 | 📋 待验证 |
| 策略缓存命中率 | ≥90% | 缓存测试 | 📋 待验证 |
| 字段级权限 | 100% 字段覆盖 | 权限测试 | 📋 待验证 |
| 行级权限 | 100% 资源覆盖 | 权限测试 | 📋 待验证 |
| 策略热加载 | <5s 生效 | 热加载测试 | 📋 待验证 |
| 审计日志覆盖 | 100% 决策记录 | 日志审计 | 📋 待验证 |

---

## 九、测试策略

### 9.1 单元测试

```rego
# cgas/test/authz_test.rego
package cgas.authz.test

import data.cgasa.authz

# 测试：管理员允许所有操作
test_admin_allow {
    allow with input as {
        "user": {"id": "admin_1", "roles": ["admin"]},
        "action": {"operation": "batch:delete"},
        "resource": {"type": "batch", "sensitivity": "restricted"}
    }
}

# 测试：Developer 允许执行 Batch
test_developer_batch_execute {
    allow with input as {
        "user": {"id": "dev_1", "roles": ["developer"]},
        "action": {"operation": "batch:execute"},
        "resource": {"type": "batch", "owner": "dev_1"}
    }
}

# 测试：Viewer 拒绝执行 Batch
test_viewer_deny_execute {
    not allow with input as {
        "user": {"id": "viewer_1", "roles": ["viewer"]},
        "action": {"operation": "batch:execute"},
        "resource": {"type": "batch"}
    }
}

# 测试：字段级权限
test_field_level_developer {
    allowed_fields with input as {
        "user": {"id": "dev_1", "roles": ["developer"]},
        "requested_fields": ["id", "status", "commands"]
    } = ["id", "status", "commands"]
}

# 测试：敏感字段拒绝
test_sensitive_field_deny {
    deny_field with input as {
        "user": {"id": "dev_1", "roles": ["developer"]},
        "requested_fields": ["security_context"]
    }
}
```

### 9.2 集成测试

| 测试场景 | 输入 | 预期输出 | 状态 |
|---|---|---|---|
| 管理员访问 | admin + 任意资源 | allow | 📋 待验证 |
| Developer 访问自己的资源 | developer + 自有资源 | allow | 📋 待验证 |
| Developer 访问他人资源 | developer + 他人资源 | deny | 📋 待验证 |
| Viewer 读取 | viewer + read 操作 | allow | 📋 待验证 |
| Viewer 执行 | viewer + execute 操作 | deny | 📋 待验证 |
| 字段级权限 | developer + 敏感字段 | deny_field | 📋 待验证 |
| 行级权限 | developer + 他人资源 | 过滤结果 | 📋 待验证 |
| 时间衰减 | 过期权限 | needs_renewal | 📋 待验证 |

---

## 十、结论

### 10.1 集成总结

Phase 3 Week 2 OIDC + OPA 集成规范实现身份验证与授权策略的无缝联动:
1. **声明映射**: OIDC Claims 自动映射至 OPA 输入
2. **动态策略**: RBAC + ABAC 组合，支持字段级/行级权限
3. **策略热加载**: Bundle Service 实现<5s 策略生效
4. **性能优化**: 缓存 + 预取，评估延迟<15ms (-57%)
5. **审计日志**: 100% 授权决策记录，实时流式审计

### 10.2 后续工作

1. **Week 2 实施**: 按 8.1 任务分解执行开发
2. **Week 3 测试**: 单元测试 + 集成测试 + 对抗测试
3. **Week 4 优化**: 基于测试结果优化性能
4. **Week 5 集成**: 与 Batch 嵌套/Transaction 隔离集成验证
5. **Week 6 Exit Gate**: 证据包整理 + 评审

---

## 签署确认

| 角色 | 日期 | 结论 | 签名 | 备注 |
|---|---|---|---|---|
| PM | 📋 | 📋 | - | 范围确认 |
| Dev | 📋 | 📋 | - | 技术可行性确认 |
| QA | 📋 | 📋 | - | 可测试性确认 |
| SRE | 📋 | 📋 | - | 运维支持确认 |
| Security | ✅ | ✅ | Security Agent | 安全合规确认 |

---

**编制人**: Security Agent  
**审查日期**: 2026-05-19  
**版本**: v1.0  
**状态**: ✅ 完成  
**下次评审**: Week 2-T3 技术评审会议

---

## 附录 A: 参考文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Phase 3 PRD v3 | phase3_prd_v3.md | Phase 3 需求基线 |
| 零信任增强方案 | zero_trust_enhancement.md | Phase 3 零信任设计 |
| OIDC 规范 | oidc_spec.md | OIDC 设计 |
| OPA 文档 | https://www.openpolicyagent.org/docs/ | OPA 参考 |

## 附录 B: 术语表

| 术语 | 定义 |
|---|---|
| OIDC | OpenID Connect，基于 OAuth2 的身份层协议 |
| OPA | Open Policy Agent，通用策略引擎 |
| RBAC | Role-Based Access Control，基于角色的访问控制 |
| ABAC | Attribute-Based Access Control，基于属性的访问控制 |
| Bundle | OPA 策略包，支持版本管理和热加载 |
| Rego | OPA 策略语言 |
