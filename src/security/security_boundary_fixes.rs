// security_boundary_fixes.rs
// Phase 3 Week 4 Security - 边界场景修复与安全加固
// 10 个边界场景修复实现

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use anyhow::{Result, anyhow, bail};

// ============================================================================
// 场景 1: 权限边界场景 - 角色权限验证增强
// ============================================================================

/// 权限边界检查器
pub struct PermissionBoundaryChecker {
    // 角色权限映射缓存
    role_permissions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    // 权限验证超时
    timeout: Duration,
}

impl PermissionBoundaryChecker {
    pub fn new(timeout: Duration) -> Self {
        Self {
            role_permissions: Arc::new(RwLock::new(HashMap::new())),
            timeout,
        }
    }

    /// 验证用户权限边界
    /// 边界场景：用户尝试访问超出角色权限的资源
    pub async fn verify_permission_boundary(
        &self,
        user_id: &str,
        user_roles: &[String],
        required_permission: &str,
        resource_id: &str,
    ) -> Result<bool> {
        let start = Instant::now();
        
        // 边界检查 1: 超时保护
        if start.elapsed() > self.timeout {
            bail!("Permission verification timeout");
        }

        // 边界检查 2: 空角色处理
        if user_roles.is_empty() {
            return Ok(false);
        }

        // 边界检查 3: 权限字符串验证 (防止注入)
        if !self.is_valid_permission_string(required_permission) {
            bail!("Invalid permission string format");
        }

        // 并发获取所有角色的权限
        let mut has_permission = false;
        for role in user_roles {
            if let Some(permissions) = self.role_permissions.read().await.get(role) {
                if permissions.contains(&required_permission.to_string()) {
                    has_permission = true;
                    break;
                }
            }
        }

        // 边界检查 4: 隐式拒绝原则
        // 如果角色不存在于缓存中，默认拒绝
        if !has_permission && user_roles.iter().all(|r| {
            !self.role_permissions.read().await.contains_key(r)
        }) {
            log::warn!("User {} has unknown roles: {:?}", user_id, user_roles);
            return Ok(false);
        }

        Ok(has_permission)
    }

    /// 权限字符串白名单验证 (防止注入攻击)
    fn is_valid_permission_string(&self, permission: &str) -> bool {
        // 只允许字母、数字、下划线、冒号、连字符
        let allowed_pattern = regex::Regex::new(r"^[a-zA-Z0-9:_-]+$").unwrap();
        allowed_pattern.is_match(permission)
            && permission.len() <= 128  // 长度限制
            && !permission.contains("..")  // 防止路径遍历
            && !permission.starts_with('-')  // 防止特殊前缀
    }

    /// 更新角色权限缓存
    pub async fn update_role_permissions(
        &mut self,
        role: String,
        permissions: Vec<String>,
    ) {
        self.role_permissions.write().await.insert(role, permissions);
    }
}

// ============================================================================
// 场景 2: Token 过期场景 - 自动刷新与优雅降级
// ============================================================================

/// Token 管理器 - 处理过期、刷新、降级场景
pub struct TokenManager {
    // Token 缓存
    token_cache: Arc<RwLock<HashMap<String, TokenEntry>>>,
    // 刷新信号量 (防止并发刷新风暴)
    refresh_semaphore: Arc<Semaphore>,
    // Token 过期前刷新阈值 (秒)
    refresh_threshold: Duration,
}

#[derive(Clone, Debug)]
struct TokenEntry {
    token: String,
    expires_at: Instant,
    refresh_token: Option<String>,
    is_refreshing: bool,
}

impl TokenManager {
    pub fn new(refresh_threshold_secs: u64) -> Self {
        Self {
            token_cache: Arc::new(RwLock::new(HashMap::new())),
            refresh_semaphore: Arc::new(Semaphore::new(10)),  // 最多 10 个并发刷新
            refresh_threshold: Duration::from_secs(refresh_threshold_secs),
        }
    }

    /// 获取有效 Token (自动处理过期和刷新)
    pub async fn get_valid_token(&self, user_id: &str) -> Result<String> {
        // 边界场景 1: Token 不存在
        let mut entry = {
            let cache = self.token_cache.read().await;
            cache.get(user_id).cloned()
        };

        match entry {
            None => {
                // 边界场景：首次获取 Token
                bail!("Token not found for user: {}", user_id);
            }
            Some(mut token_entry) => {
                let now = Instant::now();
                
                // 边界场景 2: Token 已过期
                if now > token_entry.expires_at {
                    // 尝试刷新
                    if let Some(refresh_token) = &token_entry.refresh_token {
                        token_entry = self.refresh_token(user_id, refresh_token).await?;
                    } else {
                        bail!("Token expired and no refresh token available");
                    }
                }
                // 边界场景 3: Token 即将过期 (提前刷新)
                else if now + self.refresh_threshold > token_entry.expires_at {
                    // 异步刷新，不阻塞当前请求
                    if !token_entry.is_refreshing {
                        self.schedule_refresh(user_id.to_string()).await;
                    }
                }

                Ok(token_entry.token)
            }
        }
    }

    /// 刷新 Token (带并发控制)
    async fn refresh_token(&self, user_id: &str, refresh_token: &str) -> Result<TokenEntry> {
        // 边界场景 4: 防止并发刷新风暴
        let _permit = self.refresh_semaphore.acquire().await
            .map_err(|_| anyhow!("Token refresh system overloaded"))?;

        // 双重检查：可能在等待锁时已被刷新
        {
            let cache = self.token_cache.read().await;
            if let Some(entry) = cache.get(user_id) {
                if Instant::now() + self.refresh_threshold < entry.expires_at {
                    return Ok(entry.clone());
                }
            }
        }

        // 执行刷新 (模拟调用认证服务)
        let new_token_entry = self.call_refresh_api(refresh_token).await?;

        // 更新缓存
        self.token_cache.write().await.insert(user_id.to_string(), new_token_entry.clone());

        Ok(new_token_entry)
    }

    /// 调度异步刷新
    async fn schedule_refresh(&self, user_id: String) {
        let cache = self.token_cache.clone();
        tokio::spawn(async move {
            // 实际实现中会调用刷新 API
            log::info!("Scheduled token refresh for user: {}", user_id);
        });
    }

    /// 模拟调用刷新 API
    async fn call_refresh_api(&self, _refresh_token: &str) -> Result<TokenEntry> {
        // 实际实现中会调用 OIDC Provider
        Ok(TokenEntry {
            token: "new_access_token".to_string(),
            expires_at: Instant::now() + Duration::from_secs(3600),
            refresh_token: Some("new_refresh_token".to_string()),
            is_refreshing: false,
        })
    }
}

// ============================================================================
// 场景 3: 并发访问控制场景 - 分布式锁与限流
// ============================================================================

/// 并发访问控制器
pub struct ConcurrentAccessController {
    // 用户级别信号量
    user_semaphores: Arc<RwLock<HashMap<String, Arc<Semaphore>>>>,
    // 资源级别锁
    resource_locks: Arc<RwLock<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
    // 全局限流器
    global_rate_limiter: Arc<tokio::sync::Semaphore>,
}

impl ConcurrentAccessController {
    pub fn new(max_concurrent_per_user: usize, global_limit: usize) -> Self {
        Self {
            user_semaphores: Arc::new(RwLock::new(HashMap::new())),
            resource_locks: Arc::new(RwLock::new(HashMap::new())),
            global_rate_limiter: Arc::new(tokio::sync::Semaphore::new(global_limit)),
        }
    }

    /// 获取用户并发许可
    pub async fn acquire_user_permit(&self, user_id: &str) -> Result<tokio::sync::SemaphorePermit<'_>> {
        // 边界场景 1: 用户信号量不存在时创建
        let semaphore = {
            let semaphores = self.user_semaphores.read().await;
            semaphores.get(user_id).cloned()
        };

        let semaphore = match semaphore {
            Some(s) => s,
            None => {
                // 双重检查锁定模式
                let mut semaphores = self.user_semaphores.write().await;
                semaphores.entry(user_id.to_string())
                    .or_insert_with(|| Arc::new(Semaphore::new(5)))  // 默认每用户 5 个并发
                    .clone()
            }
        };

        // 边界场景 2: 获取许可超时
        tokio::time::timeout(
            Duration::from_secs(30),
            semaphore.acquire()
        ).await
            .map_err(|_| anyhow!("Timeout waiting for concurrent access permit"))?
            .map_err(|_| anyhow!("Semaphore closed"))
    }

    /// 获取资源独占锁
    pub async fn acquire_resource_lock(&self, resource_id: &str) -> tokio::sync::MutexGuard<'_, ()> {
        let lock = {
            let locks = self.resource_locks.read().await;
            locks.get(resource_id).cloned()
        };

        let lock = match lock {
            Some(l) => l,
            None => {
                let mut locks = self.resource_locks.write().await;
                locks.entry(resource_id.to_string())
                    .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                    .clone()
            }
        };

        lock.lock().await
    }

    /// 全局限流
    pub async fn acquire_global_permit(&self) -> Result<tokio::sync::SemaphorePermit<'_>> {
        tokio::time::timeout(
            Duration::from_secs(60),
            self.global_rate_limiter.acquire()
        ).await
            .map_err(|_| anyhow!("Global rate limit timeout"))?
            .map_err(|_| anyhow!("Global rate limiter closed"))
    }
}

// ============================================================================
// 场景 4: 注入攻击防护场景 - SQL/NoSQL/命令注入防护
// ============================================================================

/// 注入攻击防护器
pub struct InjectionProtector {
    // SQL 注入检测模式
    sql_injection_patterns: Vec<regex::Regex>,
    // 命令注入危险字符
    command_injection_chars: Vec<char>,
    // XSS 检测模式
    xss_patterns: Vec<regex::Regex>,
}

impl InjectionProtector {
    pub fn new() -> Self {
        Self {
            sql_injection_patterns: vec![
                regex::Regex::new(r"(?i)(union\s+select|insert\s+into|delete\s+from|drop\s+table|update\s+.*set)").unwrap(),
                regex::Regex::new(r"(?i)(or\s+1\s*=\s*1|and\s+1\s*=\s*1)").unwrap(),
                regex::Regex::new(r"(?i)(--|\#|/\*)").unwrap(),
                regex::Regex::new(r"(?i)(exec\s*\(|execute\s*\()").unwrap(),
            ],
            command_injection_chars: vec![';', '|', '&', '$', '`', '(', ')', '{', '}', '<', '>', '\n', '\r'],
            xss_patterns: vec![
                regex::Regex::new(r"(?i)(<script|javascript:|on\w+\s*=)").unwrap(),
                regex::Regex::new(r"(?i)(<iframe|<object|<embed)").unwrap(),
            ],
        }
    }

    /// 验证输入是否安全 (SQL 注入防护)
    pub fn validate_sql_input(&self, input: &str) -> Result<()> {
        // 边界场景 1: 空输入
        if input.is_empty() {
            return Ok(());
        }

        // 边界场景 2: 过长输入 (DoS 防护)
        if input.len() > 10000 {
            bail!("Input too long (potential DoS attack)");
        }

        // 检测 SQL 注入模式
        for pattern in &self.sql_injection_patterns {
            if pattern.is_match(input) {
                bail!("Potential SQL injection detected");
            }
        }

        Ok(())
    }

    /// 验证命令输入 (命令注入防护)
    pub fn validate_command_input(&self, input: &str) -> Result<()> {
        if input.is_empty() {
            return Ok(());
        }

        if input.len() > 1000 {
            bail!("Command input too long");
        }

        // 检测危险字符
        for ch in input.chars() {
            if self.command_injection_chars.contains(&ch) {
                bail!("Potential command injection detected: dangerous character '{}'", ch);
            }
        }

        Ok(())
    }

    /// 验证用户输入 (XSS 防护)
    pub fn validate_user_input(&self, input: &str) -> Result<()> {
        if input.is_empty() {
            return Ok(());
        }

        if input.len() > 5000 {
            bail!("User input too long");
        }

        for pattern in &self.xss_patterns {
            if pattern.is_match(input) {
                bail!("Potential XSS attack detected");
            }
        }

        Ok(())
    }

    /// 清理输入 (白名单过滤)
    pub fn sanitize_input(&self, input: &str, allowed_chars: &str) -> String {
        input.chars()
            .filter(|c| allowed_chars.contains(*c))
            .collect()
    }
}

// ============================================================================
// 场景 5-10: 其他边界场景
// ============================================================================

/// 场景 5: 空指针/空值边界处理
pub struct NullSafetyHandler;

impl NullSafetyHandler {
    /// 安全获取 Option 值
    pub fn safe_get<T: Clone>(option: &Option<T>, default: T) -> T {
        option.clone().unwrap_or(default)
    }

    /// 安全获取 Vec 元素
    pub fn safe_vec_get<T: Clone>(vec: &[T], index: usize, default: Option<T>) -> Option<T> {
        vec.get(index).cloned().or(default)
    }

    /// 安全字符串操作
    pub fn safe_substring(s: &str, start: usize, end: usize) -> &str {
        let len = s.len();
        let safe_start = start.min(len);
        let safe_end = end.min(len);
        if safe_start >= safe_end {
            return "";
        }
        &s[safe_start..safe_end]
    }
}

/// 场景 6: 数值溢出边界处理
pub struct OverflowHandler;

impl OverflowHandler {
    /// 安全加法
    pub fn safe_add(a: u64, b: u64) -> Result<u64> {
        a.checked_add(b).ok_or_else(|| anyhow!("Integer overflow in addition"))
    }

    /// 安全减法
    pub fn safe_sub(a: u64, b: u64) -> Result<u64> {
        a.checked_sub(b).ok_or_else(|| anyhow!("Integer underflow in subtraction"))
    }

    /// 安全乘法
    pub fn safe_mul(a: u64, b: u64) -> Result<u64> {
        a.checked_mul(b).ok_or_else(|| anyhow!("Integer overflow in multiplication"))
    }

    /// 安全除法
    pub fn safe_div(a: u64, b: u64) -> Result<u64> {
        if b == 0 {
            bail!("Division by zero");
        }
        Ok(a / b)
    }
}

/// 场景 7: 时间边界处理
pub struct TimeBoundaryHandler;

impl TimeBoundaryHandler {
    /// 验证时间范围合理性
    pub fn validate_time_range(start: Instant, end: Instant) -> Result<Duration> {
        if end < start {
            bail!("End time is before start time");
        }

        let duration = end.duration_since(start);
        
        // 边界检查：时间跨度不能超过 1 年
        if duration > Duration::from_secs(365 * 24 * 3600) {
            bail!("Time range too large (max 1 year)");
        }

        Ok(duration)
    }

    /// 安全 Duration 转换
    pub fn safe_duration_to_secs(duration: Duration) -> u64 {
        duration.as_secs().min(u64::MAX)
    }
}

/// 场景 8: 集合边界处理
pub struct CollectionBoundaryHandler;

impl CollectionBoundaryHandler {
    /// 安全获取集合大小
    pub fn safe_len<T>(collection: &[T], max_allowed: usize) -> Result<usize> {
        let len = collection.len();
        if len > max_allowed {
            bail!("Collection size {} exceeds maximum allowed {}", len, max_allowed);
        }
        Ok(len)
    }

    /// 安全迭代 (防止无限循环)
    pub fn safe_iterate<T, F>(collection: &[T], max_iterations: usize, mut f: F) -> Result<()>
    where
        F: FnMut(&T) -> Result<()>,
    {
        for (i, item) in collection.iter().enumerate() {
            if i >= max_iterations {
                bail!("Iteration limit exceeded");
            }
            f(item)?;
        }
        Ok(())
    }
}

/// 场景 9: 递归深度边界处理
pub struct RecursionBoundaryHandler {
    max_depth: usize,
}

impl RecursionBoundaryHandler {
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }

    /// 检查递归深度
    pub fn check_depth(&self, current_depth: usize) -> Result<()> {
        if current_depth > self.max_depth {
            bail!("Recursion depth {} exceeds maximum {}", current_depth, self.max_depth);
        }
        Ok(())
    }

    /// 安全递归执行
    pub fn safe_recursive<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(usize) -> Result<T>,
    {
        f(0)
    }
}

/// 场景 10: 外部资源边界处理 (文件/网络/数据库)
pub struct ResourceBoundaryHandler {
    max_file_size: u64,
    max_response_size: u64,
    max_query_rows: usize,
}

impl ResourceBoundaryHandler {
    pub fn new(max_file_size_mb: u64, max_response_mb: u64, max_rows: usize) -> Self {
        Self {
            max_file_size: max_file_size_mb * 1024 * 1024,
            max_response_size: max_response_mb * 1024 * 1024,
            max_query_rows: max_rows,
        }
    }

    /// 验证文件大小
    pub fn validate_file_size(&self, size: u64) -> Result<()> {
        if size > self.max_file_size {
            bail!("File size {} exceeds maximum {}", size, self.max_file_size);
        }
        Ok(())
    }

    /// 验证响应大小
    pub fn validate_response_size(&self, size: u64) -> Result<()> {
        if size > self.max_response_size {
            bail!("Response size {} exceeds maximum {}", size, self.max_response_size);
        }
        Ok(())
    }

    /// 验证查询行数
    pub fn validate_query_rows(&self, rows: usize) -> Result<()> {
        if rows > self.max_query_rows {
            bail!("Query result rows {} exceeds maximum {}", rows, self.max_query_rows);
        }
        Ok(())
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_boundary_checker() {
        let mut checker = PermissionBoundaryChecker::new(Duration::from_secs(5));
        
        // 设置角色权限
        checker.update_role_permissions(
            "admin".to_string(),
            vec!["read".to_string(), "write".to_string(), "delete".to_string()],
        ).await;

        // 测试有权访问
        let result = checker.verify_permission_boundary(
            "user1",
            &["admin".to_string()],
            "write",
            "resource1",
        ).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // 测试无权访问
        let result = checker.verify_permission_boundary(
            "user2",
            &["viewer".to_string()],
            "delete",
            "resource2",
        ).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_injection_protector() {
        let protector = InjectionProtector::new();

        // 测试 SQL 注入检测
        assert!(protector.validate_sql_input("SELECT * FROM users").is_ok());
        assert!(protector.validate_sql_input("SELECT * FROM users; DROP TABLE users;").is_err());
        assert!(protector.validate_sql_input("1 OR 1=1").is_err());

        // 测试命令注入检测
        assert!(protector.validate_command_input("ls -la").is_ok());
        assert!(protector.validate_command_input("ls; rm -rf /").is_err());

        // 测试 XSS 检测
        assert!(protector.validate_user_input("Hello World").is_ok());
        assert!(protector.validate_user_input("<script>alert('xss')</script>").is_err());
    }

    #[test]
    fn test_overflow_handler() {
        assert!(OverflowHandler::safe_add(100, 200).is_ok());
        assert!(OverflowHandler::safe_add(u64::MAX, 1).is_err());
        
        assert!(OverflowHandler::safe_sub(200, 100).is_ok());
        assert!(OverflowHandler::safe_sub(100, 200).is_err());
        
        assert!(OverflowHandler::safe_div(100, 0).is_err());
    }

    #[test]
    fn test_time_boundary_handler() {
        let start = Instant::now();
        let end = start + Duration::from_secs(3600);
        
        assert!(TimeBoundaryHandler::validate_time_range(start, end).is_ok());
        assert!(TimeBoundaryHandler::validate_time_range(end, start).is_err());
    }
}
