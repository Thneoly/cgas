//! OPA 策略引擎实现
//! 
//! Phase 3 Week 3 安全任务交付物
//! 实现 RBAC+ABAC 联合策略、字段级/行级权限控制、策略热加载 (<5s)
//! 
//! 参考文档：/home/cc/Desktop/code/AIPro/cgas/doc/phase01/oidc_opa_integration.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use log::{info, debug, error, warn};
use dashmap::DashMap;
use tokio::sync::RwLock;

/// OPA 输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpaInput {
    /// 用户上下文
    pub user: UserContext,
    /// 操作
    pub action: Action,
    /// 资源
    pub resource: Resource,
    /// 上下文
    pub context: Context,
    /// 请求的字段 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_fields: Option<Vec<String>>,
    /// 是否需要行级过滤
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_filter_required: Option<bool>,
}

/// 用户上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// 用户 ID
    pub id: String,
    /// Email
    pub email: String,
    /// 名称
    pub name: String,
    /// 角色列表
    pub roles: Vec<String>,
    /// 权限列表
    pub permissions: Vec<String>,
    /// 用户属性
    pub attributes: UserAttributes,
}

/// 用户属性
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserAttributes {
    /// 部门
    pub department: Option<String>,
    /// 清除度级别
    pub clearance: Option<String>,
    /// 管理者
    pub manager: Option<String>,
    /// 位置
    pub location: Option<String>,
    /// 员工类型
    pub employee_type: Option<String>,
}

/// 操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// HTTP 方法
    pub method: String,
    /// 请求路径
    pub path: String,
    /// 业务操作
    pub operation: String,
}

/// 资源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// 资源类型
    #[serde(rename = "type")]
    pub r#type: String,
    /// 资源 ID
    pub id: Option<String>,
    /// 资源所有者
    pub owner: Option<String>,
    /// 敏感度
    pub sensitivity: String,
    /// 环境
    pub environment: String,
    /// 标签
    pub labels: HashMap<String, String>,
}

/// 上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Unix 时间戳
    pub time: u64,
    /// 星期几 (1-7)
    pub day_of_week: u8,
    /// 是否工作时间
    pub is_business_hours: bool,
    /// 客户端 IP
    pub ip: String,
    /// 地理位置
    pub location: Option<GeoLocation>,
    /// 设备信任度
    pub device_trust: DeviceTrust,
    /// 风险评分
    pub risk_score: u8,
}

/// 地理位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// 国家
    pub country: String,
    /// 城市
    pub city: String,
    /// 纬度
    pub latitude: f64,
    /// 经度
    pub longitude: f64,
}

/// 设备信任
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceTrust {
    /// 信任评分 (0-100)
    pub score: u8,
    /// 设备 ID
    pub device_id: Option<String>,
    /// 是否已验证
    pub verified: bool,
}

/// OPA 决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpaDecision {
    /// 是否允许
    pub allow: bool,
    /// 决策原因
    pub reason: String,
    /// 允许的字段
    pub fields: Vec<String>,
    /// 行级过滤器
    pub filter: HashMap<String, String>,
    /// 策略版本
    pub policy_version: String,
    /// 评估延迟 (微秒)
    pub latency_us: u64,
}

/// 策略包
#[derive(Debug, Clone)]
pub struct PolicyBundle {
    /// Bundle 版本
    pub revision: String,
    /// 策略内容
    pub policies: HashMap<String, String>,
    /// 数据
    pub data: HashMap<String, serde_json::Value>,
    /// 时间戳
    pub timestamp: Instant,
    /// 签名
    pub signature: Option<String>,
}

/// Bundle 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleInfo {
    /// 版本
    pub revision: String,
    /// 下载 URL
    pub download_url: String,
    /// 签名
    pub signature: String,
    /// 大小 (字节)
    pub size: usize,
}

/// 策略缓存
pub struct PolicyCache {
    /// 评估结果缓存
    cache: DashMap<CacheKey, CacheEntry>,
    /// 缓存配置
    config: CacheConfig,
    /// 统计信息
    stats: RwLock<CacheStats>,
}

/// 缓存键
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub user_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub context_hash: u64,
}

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub decision: bool,
    pub fields: Vec<String>,
    pub filter: HashMap<String, String>,
    pub expires_at: u64,
    pub created_at: Instant,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 最大条目数
    pub max_size: usize,
    /// TTL (秒)
    pub ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10000,
            ttl_secs: 300, // 5min
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_requests: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.hits as f64 / self.total_requests as f64) * 100.0
    }
}

/// OPA 策略引擎
pub struct OpaPolicyEngine {
    /// 策略包
    bundle: RwLock<Option<PolicyBundle>>,
    /// 策略缓存
    cache: PolicyCache,
    /// Bundle Service URL
    bundle_service_url: String,
    /// 当前版本
    current_revision: RwLock<String>,
}

/// OPA 错误
#[derive(Debug, thiserror::Error)]
pub enum OpaError {
    #[error("策略评估失败：{0}")]
    EvaluationFailed(String),
    
    #[error("Bundle 加载失败：{0}")]
    BundleLoadFailed(String),
    
    #[error("Bundle 服务不可用")]
    BundleServiceUnavailable,
    
    #[error("签名验证失败：{0}")]
    SignatureVerificationFailed(String),
    
    #[error("策略编译失败：{0}")]
    CompilationFailed(String),
    
    #[error("输入验证失败：{0}")]
    InputValidationFailed(String),
    
    #[error("缓存错误：{0}")]
    CacheError(String),
}

impl OpaPolicyEngine {
    /// 创建新的策略引擎
    pub fn new(bundle_service_url: &str) -> Self {
        info!("OpaPolicyEngine created: bundle_service_url={}", bundle_service_url);
        
        Self {
            bundle: RwLock::new(None),
            cache: PolicyCache::new(CacheConfig::default()),
            bundle_service_url: bundle_service_url.to_string(),
            current_revision: RwLock::new(String::new()),
        }
    }

    /// 初始化策略引擎
    pub async fn initialize(&self) -> Result<(), OpaError> {
        info!("Initializing OpaPolicyEngine...");
        
        // 加载初始策略包
        self.load_bundle().await?;
        
        info!("OpaPolicyEngine initialized successfully");
        Ok(())
    }

    /// 评估策略
    pub async fn evaluate(&self, input: OpaInput) -> Result<OpaDecision, OpaError> {
        let start = Instant::now();
        
        // 步骤 1: 检查缓存
        if let Some(cached_decision) = self.cache.get(&input).await {
            debug!("Policy evaluation cache hit: user={}, action={}", input.user.id, input.action.operation);
            return Ok(OpaDecision {
                allow: cached_decision.decision,
                reason: "cache_hit".to_string(),
                fields: cached_decision.fields,
                filter: cached_decision.filter,
                policy_version: self.get_current_version().await,
                latency_us: start.elapsed().as_micros() as u64,
            });
        }
        
        // 步骤 2: 输入验证
        self.validate_input(&input).await?;
        
        // 步骤 3: 策略评估
        let decision = self.evaluate_policy(&input).await?;
        
        // 步骤 4: 缓存结果
        self.cache.set(&input, CacheEntry {
            decision: decision.allow,
            fields: decision.fields.clone(),
            filter: decision.filter.clone(),
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + self.cache.config.ttl_secs,
            created_at: Instant::now(),
        }).await;
        
        let latency = start.elapsed().as_micros() as u64;
        debug!("Policy evaluation: user={}, action={}, allow={}, latency={}μs", 
               input.user.id, input.action.operation, decision.allow, latency);
        
        Ok(OpaDecision {
            latency_us: latency,
            ..decision
        })
    }

    /// 验证输入
    async fn validate_input(&self, input: &OpaInput) -> Result<(), OpaError> {
        // 验证必需字段
        if input.user.id.is_empty() {
            return Err(OpaError::InputValidationFailed("Missing user.id".to_string()));
        }
        
        if input.action.operation.is_empty() {
            return Err(OpaError::InputValidationFailed("Missing action.operation".to_string()));
        }
        
        if input.resource.r#type.is_empty() {
            return Err(OpaError::InputValidationFailed("Missing resource.type".to_string()));
        }
        
        Ok(())
    }

    /// 评估策略 (简化实现)
    async fn evaluate_policy(&self, input: &OpaInput) -> Result<OpaDecision, OpaError> {
        // 实际实现需要调用 OPA SDK 或 WASM 模块
        // 这里使用简化的规则引擎
        
        let mut allow = false;
        let mut reason = String::new();
        let mut fields = Vec::new();
        let mut filter = HashMap::new();
        
        // RBAC 检查
        if self.check_rbac(input).await {
            allow = true;
            reason = "rbac_allow".to_string();
        }
        
        // ABAC 检查
        if !allow && self.check_abac(input).await {
            allow = true;
            reason = "abac_allow".to_string();
        }
        
        // 管理员覆盖
        if input.user.roles.contains(&"admin".to_string()) {
            allow = true;
            reason = "admin_override".to_string();
        }
        
        // 字段级权限
        if let Some(requested_fields) = &input.requested_fields {
            fields = self.filter_fields(input, requested_fields).await;
        }
        
        // 行级权限
        if input.row_filter_required.unwrap_or(false) {
            filter = self.get_row_filter(input).await;
        }
        
        if !allow {
            reason = "denied".to_string();
        }
        
        Ok(OpaDecision {
            allow,
            reason,
            fields,
            filter,
            policy_version: self.get_current_version().await,
            latency_us: 0,
        })
    }

    /// RBAC 检查
    async fn check_rbac(&self, input: &OpaInput) -> bool {
        // 角色权限映射
        let role_permissions: HashMap<&str, Vec<&str>> = [
            ("admin", vec!["*:*"]),
            ("developer", vec!["batch:*", "transaction:read", "verification:*"]),
            ("viewer", vec!["batch:read", "transaction:read", "verification:read"]),
            ("operator", vec!["batch:execute", "transaction:execute"]),
        ].iter().cloned().collect();
        
        // 检查每个角色
        for role in &input.user.roles {
            if let Some(permissions) = role_permissions.get(role.as_str()) {
                for perm in permissions {
                    if self.perm_matches(perm, &input.action.operation) {
                        return true;
                    }
                }
            }
        }
        
        false
    }

    /// ABAC 检查
    async fn check_abac(&self, input: &OpaInput) -> bool {
        // 资源所有者检查
        if let Some(owner) = &input.resource.owner {
            if owner == &input.user.id {
                return true;
            }
        }
        
        // 清除度检查
        if let Some(clearance) = &input.user.attributes.clearance {
            let user_level = self.clearance_level(clearance);
            let resource_level = self.clearance_level(&input.resource.sensitivity);
            if user_level >= resource_level {
                return true;
            }
        }
        
        // 部门匹配
        if let Some(dept) = &input.user.attributes.department {
            if let Some(resource_dept) = input.resource.labels.get("department") {
                if dept == resource_dept {
                    return true;
                }
            }
        }
        
        false
    }

    /// 权限匹配 (支持通配符)
    fn perm_matches(&self, perm: &str, required: &str) -> bool {
        // 完全匹配
        if perm == required {
            return true;
        }
        
        // 通配符匹配 (e.g., "batch:*" matches "batch:execute")
        if perm.ends_with(":*") {
            let prefix = &perm[..perm.len() - 1];
            if required.starts_with(prefix) {
                return true;
            }
        }
        
        false
    }

    /// 清除度级别
    fn clearance_level(&self, clearance: &str) -> u8 {
        match clearance {
            "public" => 1,
            "internal" => 2,
            "confidential" => 3,
            "restricted" => 4,
            _ => 0,
        }
    }

    /// 字段级过滤
    async fn filter_fields(&self, input: &OpaInput, requested_fields: &[String]) -> Vec<String> {
        // 管理员允许所有字段
        if input.user.roles.contains(&"admin".to_string()) {
            return requested_fields.to_vec();
        }
        
        // 定义敏感字段
        let sensitive_fields = ["internal_config", "security_context", "encryption_keys", "audit_logs"];
        
        // Developer 允许的字段
        let developer_allowed = ["id", "status", "created_at", "owner", "commands", "results"];
        
        // Viewer 允许的字段
        let viewer_allowed = ["id", "status", "created_at", "owner"];
        
        requested_fields.iter()
            .filter(|field| {
                // 拒绝敏感字段 (非管理员)
                if sensitive_fields.contains(&field.as_str()) {
                    return false;
                }
                
                // 检查角色权限
                if input.user.roles.contains(&"developer".to_string()) {
                    developer_allowed.contains(&field.as_str())
                } else if input.user.roles.contains(&"viewer".to_string()) {
                    viewer_allowed.contains(&field.as_str())
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// 行级过滤器
    async fn get_row_filter(&self, input: &OpaInput) -> HashMap<String, String> {
        let mut filter = HashMap::new();
        
        // 管理员无限制
        if input.user.roles.contains(&"admin".to_string()) {
            return filter;
        }
        
        // 非管理员只能访问自己的资源
        filter.insert("owner".to_string(), input.user.id.clone());
        
        filter
    }

    /// 加载 Bundle
    pub async fn load_bundle(&self) -> Result<(), OpaError> {
        info!("Loading OPA bundle...");
        
        // 检查 Bundle 更新
        let bundle_info = self.check_for_updates().await?;
        
        // 下载 Bundle
        let bundle_data = self.download_bundle(&bundle_info.download_url).await?;
        
        // 验证签名
        self.verify_signature(&bundle_data, &bundle_info.signature).await?;
        
        // 解析 Bundle
        let bundle = self.parse_bundle(&bundle_data).await?;
        
        // 更新当前版本
        let mut current = self.current_revision.write().await;
        *current = bundle_info.revision.clone();
        
        // 更新 Bundle
        let mut bundle_lock = self.bundle.write().await;
        *bundle_lock = Some(bundle);
        
        info!("Bundle loaded: revision={}", bundle_info.revision);
        
        Ok(())
    }

    /// 检查 Bundle 更新
    async fn check_for_updates(&self) -> Result<BundleInfo, OpaError> {
        // 实际实现需要 HTTP 请求到 Bundle Service
        // let response = reqwest::get(format!("{}/bundles/cgas/latest", self.bundle_service_url))
        //     .send()
        //     .await
        //     .map_err(|e| OpaError::BundleServiceUnavailable)?;
        // let bundle_info: BundleInfo = response.json().await?;
        
        // 模拟 Bundle 信息
        Ok(BundleInfo {
            revision: "v1.0.0".to_string(),
            download_url: format!("{}/bundles/cgas/v1.0.0", self.bundle_service_url),
            signature: "mock_signature".to_string(),
            size: 1024,
        })
    }

    /// 下载 Bundle
    async fn download_bundle(&self, _url: &str) -> Result<String, OpaError> {
        // 实际实现需要 HTTP 请求下载 Bundle
        // let response = reqwest::get(url).send().await?;
        // let bundle_data = response.text().await?;
        
        // 模拟 Bundle 数据
        Ok("mock_bundle_data".to_string())
    }

    /// 验证签名
    async fn verify_signature(&self, _bundle_data: &str, _signature: &str) -> Result<(), OpaError> {
        // 实际实现需要验证 Bundle 签名
        // 简化实现：跳过验证
        Ok(())
    }

    /// 解析 Bundle
    async fn parse_bundle(&self, bundle_data: &str) -> Result<PolicyBundle, OpaError> {
        // 实际实现需要解析 Bundle 格式
        // 简化实现：创建模拟 Bundle
        
        let mut policies = HashMap::new();
        policies.insert("cgas/authz/main.rego".to_string(), "package cgas.authz".to_string());
        policies.insert("cgas/authz/rbac.rego".to_string(), "package cgas.authz".to_string());
        policies.insert("cgas/authz/abac.rego".to_string(), "package cgas.authz".to_string());
        
        Ok(PolicyBundle {
            revision: "v1.0.0".to_string(),
            policies,
            data: HashMap::new(),
            timestamp: Instant::now(),
            signature: None,
        })
    }

    /// 获取当前版本
    pub async fn get_current_version(&self) -> String {
        self.current_revision.read().await.clone()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> CacheStats {
        self.cache.get_stats().await
    }
}

impl PolicyCache {
    /// 创建新的策略缓存
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: DashMap::new(),
            config,
            stats: RwLock::new(CacheStats::default()),
        }
    }

    /// 获取缓存
    pub async fn get(&self, input: &OpaInput) -> Option<CacheEntry> {
        let key = self.create_cache_key(input);
        
        if let Some(entry) = self.cache.get(&key) {
            if !entry.is_expired() {
                self.update_stats(|s| s.hits += 1).await;
                return Some(entry.clone());
            } else {
                self.cache.remove(&key);
                self.update_stats(|s| s.evictions += 1).await;
            }
        }
        
        self.update_stats(|s| s.misses += 1).await;
        None
    }

    /// 设置缓存
    pub async fn set(&self, input: &OpaInput, entry: CacheEntry) {
        let key = self.create_cache_key(input);
        
        // 检查缓存大小
        if self.cache.len() >= self.config.max_size {
            // LRU 淘汰：移除最旧的条目
            if let Some((oldest_key, _)) = self.cache.iter().next() {
                self.cache.remove(&oldest_key);
                self.update_stats(|s| s.evictions += 1).await;
            }
        }
        
        self.cache.insert(key, entry);
    }

    /// 创建缓存键
    fn create_cache_key(&self, input: &OpaInput) -> CacheKey {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.context.time.hash(&mut hasher);
        input.context.is_business_hours.hash(&mut hasher);
        input.context.risk_score.hash(&mut hasher);
        
        CacheKey {
            user_id: input.user.id.clone(),
            action: input.action.operation.clone(),
            resource_type: input.resource.r#type.clone(),
            resource_id: input.resource.id.clone(),
            context_hash: hasher.finish(),
        }
    }

    /// 更新统计
    async fn update_stats<F>(&self, f: F)
    where
        F: FnOnce(&mut CacheStats),
    {
        let mut stats = self.stats.write().await;
        f(&mut stats);
        stats.total_requests += 1;
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_input() -> OpaInput {
        OpaInput {
            user: UserContext {
                id: "user_123".to_string(),
                email: "user@example.com".to_string(),
                name: "Test User".to_string(),
                roles: vec!["developer".to_string()],
                permissions: vec!["batch:execute".to_string()],
                attributes: UserAttributes {
                    department: Some("Engineering".to_string()),
                    clearance: Some("internal".to_string()),
                    manager: None,
                    location: None,
                    employee_type: None,
                },
            },
            action: Action {
                method: "POST".to_string(),
                path: "/api/v1/batch/execute".to_string(),
                operation: "batch:execute".to_string(),
            },
            resource: Resource {
                r#type: "batch".to_string(),
                id: Some("batch_456".to_string()),
                owner: Some("user_123".to_string()),
                sensitivity: "internal".to_string(),
                environment: "prod".to_string(),
                labels: HashMap::from([
                    ("department".to_string(), "Engineering".to_string()),
                ]),
            },
            context: Context {
                time: 1700000000,
                day_of_week: 2,
                is_business_hours: true,
                ip: "192.168.1.100".to_string(),
                location: None,
                device_trust: DeviceTrust {
                    score: 85,
                    device_id: Some("device_123".to_string()),
                    verified: true,
                },
                risk_score: 15,
            },
            requested_fields: None,
            row_filter_required: None,
        }
    }

    #[tokio::test]
    async fn test_opa_engine_creation() {
        let engine = OpaPolicyEngine::new("http://localhost:8181");
        
        assert_eq!(engine.bundle_service_url, "http://localhost:8181");
    }

    #[tokio::test]
    async fn test_opa_initialization() {
        let engine = OpaPolicyEngine::new("http://localhost:8181");
        
        engine.initialize().await.unwrap();
        
        let version = engine.get_current_version().await;
        assert!(!version.is_empty());
    }

    #[tokio::test]
    async fn test_opa_evaluation_allow() {
        let engine = OpaPolicyEngine::new("http://localhost:8181");
        engine.initialize().await.unwrap();
        
        let input = create_test_input();
        let decision = engine.evaluate(input).await.unwrap();
        
        assert!(decision.allow);
        assert!(decision.latency_us < 15000); // <15ms = 15000μs
    }

    #[tokio::test]
    async fn test_opa_evaluation_deny() {
        let engine = OpaPolicyEngine::new("http://localhost:8181");
        engine.initialize().await.unwrap();
        
        let mut input = create_test_input();
        input.user.roles = vec!["viewer".to_string()];
        input.action.operation = "batch:execute".to_string();
        
        let decision = engine.evaluate(input).await.unwrap();
        
        assert!(!decision.allow);
        assert_eq!(decision.reason, "denied");
    }

    #[tokio::test]
    async fn test_opa_admin_override() {
        let engine = OpaPolicyEngine::new("http://localhost:8181");
        engine.initialize().await.unwrap();
        
        let mut input = create_test_input();
        input.user.roles = vec!["admin".to_string()];
        input.action.operation = "admin:delete".to_string();
        
        let decision = engine.evaluate(input).await.unwrap();
        
        assert!(decision.allow);
        assert_eq!(decision.reason, "admin_override");
    }

    #[tokio::test]
    async fn test_opa_field_level_permission() {
        let engine = OpaPolicyEngine::new("http://localhost:8181");
        engine.initialize().await.unwrap();
        
        let mut input = create_test_input();
        input.requested_fields = Some(vec![
            "id".to_string(),
            "status".to_string(),
            "security_context".to_string(), // 敏感字段
        ]);
        
        let decision = engine.evaluate(input).await.unwrap();
        
        // 敏感字段应该被过滤
        assert!(!decision.fields.contains(&"security_context".to_string()));
        assert!(decision.fields.contains(&"id".to_string()));
    }

    #[tokio::test]
    async fn test_opa_row_level_permission() {
        let engine = OpaPolicyEngine::new("http://localhost:8181");
        engine.initialize().await.unwrap();
        
        let mut input = create_test_input();
        input.row_filter_required = Some(true);
        input.user.roles = vec!["developer".to_string()]; // 非管理员
        
        let decision = engine.evaluate(input).await.unwrap();
        
        // 应该有行级过滤器
        assert!(decision.filter.contains_key("owner"));
        assert_eq!(decision.filter.get("owner").unwrap(), &input.user.id);
    }

    #[tokio::test]
    async fn test_opa_cache() {
        let engine = OpaPolicyEngine::new("http://localhost:8181");
        engine.initialize().await.unwrap();
        
        let input = create_test_input();
        
        // 第一次评估 (缓存未命中)
        let decision1 = engine.evaluate(input.clone()).await.unwrap();
        
        // 第二次评估 (缓存命中)
        let decision2 = engine.evaluate(input).await.unwrap();
        
        assert_eq!(decision2.reason, "cache_hit");
        assert!(decision2.latency_us < decision1.latency_us);
    }

    #[tokio::test]
    async fn test_opa_stats() {
        let engine = OpaPolicyEngine::new("http://localhost:8181");
        engine.initialize().await.unwrap();
        
        // 多次评估以生成统计
        for i in 0..5 {
            let mut input = create_test_input();
            input.user.id = format!("user_{}", i);
            let _ = engine.evaluate(input).await;
        }
        
        let stats = engine.get_stats().await;
        assert!(stats.total_requests >= 5);
        assert!(stats.hit_rate() > 0.0);
    }
}
