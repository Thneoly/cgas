//! RBAC+ABAC 授权管理
//! 
//! 实现基于角色的访问控制 (RBAC) 和基于属性的访问控制 (ABAC)
//! Phase 2 Week 4 零信任架构关键组件

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use log::{info, debug, warn};

/// 权限配置
#[derive(Debug, Clone)]
pub struct PermissionConfig {
    /// 角色定义
    pub roles: HashMap<String, Role>,
    /// 权限定义
    pub permissions: HashMap<String, Permission>,
    /// 角色 - 权限映射
    pub role_permissions: HashMap<String, HashSet<String>>,
    /// 用户 - 角色映射
    pub user_roles: HashMap<String, HashSet<String>>,
    /// ABAC 策略
    pub abac_policies: Vec<AbacPolicy>,
}

impl Default for PermissionConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionConfig {
    /// 创建新的权限配置
    pub fn new() -> Self {
        let mut config = Self {
            roles: HashMap::new(),
            permissions: HashMap::new(),
            role_permissions: HashMap::new(),
            user_roles: HashMap::new(),
            abac_policies: Vec::new(),
        };
        
        // 初始化默认角色
        config.init_default_roles();
        
        config
    }
    
    /// 初始化默认角色
    fn init_default_roles(&mut self) {
        // 管理员角色
        self.roles.insert("admin".to_string(), Role {
            id: "admin".to_string(),
            name: "Administrator".to_string(),
            description: "系统管理员，拥有所有权限".to_string(),
            level: 100,
        });
        
        // 开发者角色
        self.roles.insert("developer".to_string(), Role {
            id: "developer".to_string(),
            name: "Developer".to_string(),
            description: "开发者，拥有开发和测试权限".to_string(),
            level: 50,
        });
        
        // 测试者角色
        self.roles.insert("tester".to_string(), Role {
            id: "tester".to_string(),
            name: "Tester".to_string(),
            description: "测试者，拥有测试权限".to_string(),
            level: 30,
        });
        
        // 普通用户角色
        self.roles.insert("user".to_string(), Role {
            id: "user".to_string(),
            name: "User".to_string(),
            description: "普通用户，拥有基本使用权限".to_string(),
            level: 10,
        });
        
        // 初始化默认权限
        self.init_default_permissions();
        
        // 初始化角色 - 权限映射
        self.init_default_role_permissions();
    }
    
    /// 初始化默认权限
    fn init_default_permissions(&mut self) {
        let permissions = vec![
            ("batch:execute", "执行 Batch 指令"),
            ("batch:read", "读取 Batch 结果"),
            ("transaction:execute", "执行 Transaction"),
            ("transaction:read", "读取 Transaction 状态"),
            ("admin:manage", "管理权限"),
            ("dev:deploy", "部署权限"),
            ("test:execute", "执行测试"),
        ];
        
        for (id, desc) in permissions {
            self.permissions.insert(id.to_string(), Permission {
                id: id.to_string(),
                name: id.to_string(),
                description: desc.to_string(),
                resource_type: id.split(':').next().unwrap_or("").to_string(),
                action: id.split(':').nth(1).unwrap_or("").to_string(),
            });
        }
    }
    
    /// 初始化默认角色 - 权限映射
    fn init_default_role_permissions(&mut self) {
        // 管理员拥有所有权限
        let admin_perms: HashSet<String> = self.permissions.keys().cloned().collect();
        self.role_permissions.insert("admin".to_string(), admin_perms);
        
        // 开发者权限
        self.role_permissions.insert("developer".to_string(), vec![
            "batch:execute".to_string(),
            "batch:read".to_string(),
            "transaction:execute".to_string(),
            "transaction:read".to_string(),
            "dev:deploy".to_string(),
            "test:execute".to_string(),
        ].into_iter().collect());
        
        // 测试者权限
        self.role_permissions.insert("tester".to_string(), vec![
            "batch:read".to_string(),
            "transaction:read".to_string(),
            "test:execute".to_string(),
        ].into_iter().collect());
        
        // 普通用户权限
        self.role_permissions.insert("user".to_string(), vec![
            "batch:execute".to_string(),
            "batch:read".to_string(),
            "transaction:execute".to_string(),
            "transaction:read".to_string(),
        ].into_iter().collect());
    }
}

/// 角色定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// 角色 ID
    pub id: String,
    /// 角色名称
    pub name: String,
    /// 角色描述
    pub description: String,
    /// 角色级别 (用于权限继承)
    pub level: u32,
}

/// 权限定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// 权限 ID
    pub id: String,
    /// 权限名称
    pub name: String,
    /// 权限描述
    pub description: String,
    /// 资源类型
    pub resource_type: String,
    /// 操作类型
    pub action: String,
}

/// ABAC 策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacPolicy {
    /// 策略 ID
    pub id: String,
    /// 策略名称
    pub name: String,
    /// 策略描述
    pub description: String,
    /// 策略规则
    pub rule: AbacRule,
    /// 优先级 (数字越小优先级越高)
    pub priority: u32,
    /// 是否启用
    pub enabled: bool,
}

/// ABAC 规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacRule {
    /// 主体属性条件
    pub subject_conditions: Vec<AttributeCondition>,
    /// 资源属性条件
    pub resource_conditions: Vec<AttributeCondition>,
    /// 操作属性条件
    pub action_conditions: Vec<AttributeCondition>,
    /// 环境属性条件
    pub environment_conditions: Vec<AttributeCondition>,
    /// 效果 (Permit/Deny)
    pub effect: Effect,
}

/// 属性条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeCondition {
    /// 属性名称
    pub attribute: String,
    /// 操作符
    pub operator: Operator,
    /// 属性值
    pub value: serde_json::Value,
}

/// 操作符
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Operator {
    /// 等于
    Eq,
    /// 不等于
    Ne,
    /// 大于
    Gt,
    /// 大于等于
    Ge,
    /// 小于
    Lt,
    /// 小于等于
    Le,
    /// 包含
    Contains,
    /// 在列表中
    In,
    /// 正则匹配
    Regex,
}

/// 效果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Effect {
    /// 允许
    Permit,
    /// 拒绝
    Deny,
}

/// 授权请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    /// 用户 ID
    pub user_id: String,
    /// 用户属性
    pub user_attributes: HashMap<String, serde_json::Value>,
    /// 资源 ID
    pub resource_id: String,
    /// 资源属性
    pub resource_attributes: HashMap<String, serde_json::Value>,
    /// 操作
    pub action: String,
    /// 环境属性
    pub environment_attributes: HashMap<String, serde_json::Value>,
}

/// 授权响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResponse {
    /// 是否允许
    pub permitted: bool,
    /// 效果
    pub effect: Effect,
    /// 拒绝原因
    pub denial_reason: Option<String>,
    /// 匹配的策略 ID
    pub matched_policy_ids: Vec<String>,
}

/// RBAC+ABAC 授权器
pub struct RbacAbacAuthorizer {
    /// 权限配置
    config: PermissionConfig,
    /// 授权决策日志
    decision_log: Vec<AuthorizationDecision>,
}

/// 授权决策日志
#[derive(Debug, Clone)]
struct AuthorizationDecision {
    /// 时间戳
    timestamp: std::time::Instant,
    /// 请求
    request: AuthorizationRequest,
    /// 响应
    response: AuthorizationResponse,
}

impl RbacAbacAuthorizer {
    /// 创建新的授权器
    pub fn new(config: PermissionConfig) -> Self {
        info!("RbacAbacAuthorizer created");
        
        Self {
            config,
            decision_log: Vec::new(),
        }
    }

    /// 授权检查
    pub fn authorize(&mut self, request: AuthorizationRequest) -> AuthorizationResponse {
        debug!("Authorizing request: user={}, resource={}, action={}", 
               request.user_id, request.resource_id, request.action);
        
        // 1. 首先检查 ABAC 策略 (优先级高)
        let abac_result = self.check_abac_policies(&request);
        
        if let Some(response) = abac_result {
            // 记录决策日志
            self.log_decision(request.clone(), response.clone());
            return response;
        }
        
        // 2. 然后检查 RBAC 权限
        let rbac_result = self.check_rbac_permissions(&request);
        
        // 记录决策日志
        self.log_decision(request.clone(), rbac_result.clone());
        
        rbac_result
    }

    /// 检查 ABAC 策略
    fn check_abac_policies(&self, request: &AuthorizationRequest) -> Option<AuthorizationResponse> {
        // 按优先级排序策略
        let mut sorted_policies: Vec<&AbacPolicy> = self.config.abac_policies.iter()
            .filter(|p| p.enabled)
            .collect();
        sorted_policies.sort_by_key(|p| p.priority);
        
        for policy in sorted_policies {
            if self.evaluate_policy(policy, request) {
                let matched_policy_ids = vec![policy.id.clone()];
                
                return Some(AuthorizationResponse {
                    permitted: policy.rule.effect == Effect::Permit,
                    effect: policy.rule.effect.clone(),
                    denial_reason: if policy.rule.effect == Effect::Deny {
                        Some(format!("Denied by policy: {}", policy.name))
                    } else {
                        None
                    },
                    matched_policy_ids,
                });
            }
        }
        
        // 没有匹配的 ABAC 策略
        None
    }

    /// 评估策略
    fn evaluate_policy(&self, policy: &AbacPolicy, request: &AuthorizationRequest) -> bool {
        // 评估所有条件
        let subject_match = self.evaluate_conditions(&policy.rule.subject_conditions, &request.user_attributes);
        let resource_match = self.evaluate_conditions(&policy.rule.resource_conditions, &request.resource_attributes);
        let action_match = self.evaluate_conditions(&policy.rule.action_conditions, &HashMap::from([
            ("action".to_string(), serde_json::Value::String(request.action.clone())),
        ]));
        let environment_match = self.evaluate_conditions(&policy.rule.environment_conditions, &request.environment_attributes);
        
        // 所有条件都匹配
        subject_match && resource_match && action_match && environment_match
    }

    /// 评估条件
    fn evaluate_conditions(&self, conditions: &[AttributeCondition], attributes: &HashMap<String, serde_json::Value>) -> bool {
        for condition in conditions {
            if let Some(attr_value) = attributes.get(&condition.attribute) {
                if !self.evaluate_condition(condition, attr_value) {
                    return false;
                }
            } else {
                // 属性不存在
                return false;
            }
        }
        true
    }

    /// 评估单个条件
    fn evaluate_condition(&self, condition: &AttributeCondition, attr_value: &serde_json::Value) -> bool {
        match &condition.operator {
            Operator::Eq => attr_value == &condition.value,
            Operator::Ne => attr_value != &condition.value,
            Operator::Gt => self.compare_values(attr_value, &condition.value) > 0,
            Operator::Ge => self.compare_values(attr_value, &condition.value) >= 0,
            Operator::Lt => self.compare_values(attr_value, &condition.value) < 0,
            Operator::Le => self.compare_values(attr_value, &condition.value) <= 0,
            Operator::Contains => {
                if let (serde_json::Value::Array(arr), serde_json::Value::String(val)) = (attr_value, &condition.value) {
                    arr.contains(&serde_json::Value::String(val.clone()))
                } else {
                    false
                }
            }
            Operator::In => {
                if let serde_json::Value::Array(arr) = &condition.value {
                    arr.contains(attr_value)
                } else {
                    false
                }
            }
            Operator::Regex => {
                if let (serde_json::Value::String(attr_str), serde_json::Value::String(pattern)) = (attr_value, &condition.value) {
                    // 简化实现，实际应该使用 regex crate
                    attr_str.contains(&**pattern)
                } else {
                    false
                }
            }
        }
    }

    /// 比较值
    fn compare_values(&self, a: &serde_json::Value, b: &serde_json::Value) -> i32 {
        match (a, b) {
            (serde_json::Value::Number(a_num), serde_json::Value::Number(b_num)) => {
                if let (Some(a_int), Some(b_int)) = (a_num.as_i64(), b_num.as_i64()) {
                    a_int.cmp(&b_int) as i32
                } else if let (Some(a_float), Some(b_float)) = (a_num.as_f64(), b_num.as_f64()) {
                    a_float.partial_cmp(&b_float).unwrap_or(std::cmp::Ordering::Equal) as i32
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// 检查 RBAC 权限
    fn check_rbac_permissions(&self, request: &AuthorizationRequest) -> AuthorizationResponse {
        // 获取用户角色
        let user_roles = self.config.user_roles.get(&request.user_id);
        
        if user_roles.is_none() {
            return AuthorizationResponse {
                permitted: false,
                effect: Effect::Deny,
                denial_reason: Some(format!("User {} has no roles", request.user_id)),
                matched_policy_ids: vec![],
            };
        }
        
        let user_roles = user_roles.unwrap();
        
        // 检查每个角色的权限
        for role_id in user_roles {
            if let Some(permissions) = self.config.role_permissions.get(role_id) {
                // 构造权限 ID
                let permission_id = format!("{}:{}", request.resource_id, request.action);
                
                if permissions.contains(&permission_id) {
                    return AuthorizationResponse {
                        permitted: true,
                        effect: Effect::Permit,
                        denial_reason: None,
                        matched_policy_ids: vec![format!("role:{}", role_id)],
                    };
                }
            }
        }
        
        // 没有匹配的权限
        AuthorizationResponse {
            permitted: false,
            effect: Effect::Deny,
            denial_reason: Some(format!("User {} has no permission for {}:{} ", request.user_id, request.resource_id, request.action)),
            matched_policy_ids: vec![],
        }
    }

    /// 记录决策日志
    fn log_decision(&mut self, request: AuthorizationRequest, response: AuthorizationResponse) {
        self.decision_log.push(AuthorizationDecision {
            timestamp: std::time::Instant::now(),
            request,
            response,
        });
        
        // 保持日志大小合理
        if self.decision_log.len() > 1000 {
            self.decision_log.remove(0);
        }
    }

    /// 添加用户角色
    pub fn add_user_role(&mut self, user_id: &str, role_id: &str) {
        self.user_roles
            .entry(user_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(role_id.to_string());
        
        info!("Added role {} to user {}", role_id, user_id);
    }

    /// 移除用户角色
    pub fn remove_user_role(&mut self, user_id: &str, role_id: &str) {
        if let Some(roles) = self.user_roles.get_mut(user_id) {
            roles.remove(role_id);
            info!("Removed role {} from user {}", role_id, user_id);
        }
    }

    /// 添加 ABAC 策略
    pub fn add_abac_policy(&mut self, policy: AbacPolicy) {
        self.config.abac_policies.push(policy);
    }

    /// 获取授权统计信息
    pub fn get_stats(&self) -> AuthorizationStats {
        let total_decisions = self.decision_log.len();
        let permit_count = self.decision_log.iter().filter(|d| d.response.permitted).count();
        let deny_count = total_decisions - permit_count;
        
        AuthorizationStats {
            total_decisions,
            permit_count,
            deny_count,
            permit_rate: if total_decisions > 0 {
                (permit_count as f64 / total_decisions as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// 授权统计信息
#[derive(Debug, Clone)]
pub struct AuthorizationStats {
    /// 总决策数
    pub total_decisions: usize,
    /// 允许数
    pub permit_count: usize,
    /// 拒绝数
    pub deny_count: usize,
    /// 允许率
    pub permit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbac_authorization() {
        let config = PermissionConfig::new();
        let mut authorizer = RbacAbacAuthorizer::new(config);
        
        // 添加用户角色
        authorizer.add_user_role("user_1", "admin");
        
        // 创建授权请求
        let request = AuthorizationRequest {
            user_id: "user_1".to_string(),
            user_attributes: HashMap::new(),
            resource_id: "batch".to_string(),
            resource_attributes: HashMap::new(),
            action: "execute".to_string(),
            environment_attributes: HashMap::new(),
        };
        
        // 授权检查
        let response = authorizer.authorize(request);
        
        assert!(response.permitted);
        assert_eq!(response.effect, Effect::Permit);
    }

    #[test]
    fn test_rbac_deny() {
        let config = PermissionConfig::new();
        let mut authorizer = RbacAbacAuthorizer::new(config);
        
        // 添加用户角色 (tester 没有 batch:execute 权限)
        authorizer.add_user_role("user_1", "tester");
        
        // 创建授权请求
        let request = AuthorizationRequest {
            user_id: "user_1".to_string(),
            user_attributes: HashMap::new(),
            resource_id: "batch".to_string(),
            resource_attributes: HashMap::new(),
            action: "execute".to_string(),
            environment_attributes: HashMap::new(),
        };
        
        // 授权检查
        let response = authorizer.authorize(request);
        
        assert!(!response.permitted);
        assert_eq!(response.effect, Effect::Deny);
    }

    #[test]
    fn test_abac_policy() {
        let mut config = PermissionConfig::new();
        
        // 添加 ABAC 策略：只允许工作时间访问
        let policy = AbacPolicy {
            id: "work_hours_only".to_string(),
            name: "Work Hours Only".to_string(),
            description: "Only allow access during work hours".to_string(),
            rule: AbacRule {
                subject_conditions: vec![],
                resource_conditions: vec![],
                action_conditions: vec![],
                environment_conditions: vec![
                    AttributeCondition {
                        attribute: "hour".to_string(),
                        operator: Operator::Ge,
                        value: serde_json::Value::Number(9.into()),
                    },
                    AttributeCondition {
                        attribute: "hour".to_string(),
                        operator: Operator::Le,
                        value: serde_json::Value::Number(18.into()),
                    },
                ],
                effect: Effect::Permit,
            },
            priority: 1,
            enabled: true,
        };
        
        config.abac_policies.push(policy);
        
        let mut authorizer = RbacAbacAuthorizer::new(config);
        authorizer.add_user_role("user_1", "user");
        
        // 工作时间内的请求
        let mut env_attrs = HashMap::new();
        env_attrs.insert("hour".to_string(), serde_json::Value::Number(10.into()));
        
        let request = AuthorizationRequest {
            user_id: "user_1".to_string(),
            user_attributes: HashMap::new(),
            resource_id: "batch".to_string(),
            resource_attributes: HashMap::new(),
            action: "execute".to_string(),
            environment_attributes: env_attrs,
        };
        
        let response = authorizer.authorize(request);
        assert!(response.permitted);
        
        // 非工作时间的请求
        let mut env_attrs = HashMap::new();
        env_attrs.insert("hour".to_string(), serde_json::Value::Number(20.into()));
        
        let request = AuthorizationRequest {
            user_id: "user_1".to_string(),
            user_attributes: HashMap::new(),
            resource_id: "batch".to_string(),
            resource_attributes: HashMap::new(),
            action: "execute".to_string(),
            environment_attributes: env_attrs,
        };
        
        let response = authorizer.authorize(request);
        // ABAC 策略不匹配，回退到 RBAC
        // user 角色有 batch:execute 权限
        assert!(response.permitted);
    }
}
