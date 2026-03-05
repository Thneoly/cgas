/// 确定性契约 v1.0
/// 
/// 核心原则：
/// 1. 相同输入（program + input + state_root + env_fingerprint）必须得到相同输出
/// 2. 执行阶段仅产出 state_diff，验证后提交
/// 3. 禁止隐式外部依赖（时间/随机/未声明 IO）

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 确定性契约主结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicContract {
    /// 契约版本号
    pub contract_version: String,
    
    /// 执行程序/指令序列
    pub program: String,
    
    /// 输入参数（JSON 序列化）
    pub input: Value,
    
    /// 状态树根哈希
    pub state_root: String,
    
    /// 环境指纹（捕获所有可能影响确定性的外部因素）
    pub env_fingerprint: EnvFingerprint,
}

/// 环境指纹 - 捕获所有可能影响确定性的外部因素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvFingerprint {
    /// 时间戳策略
    pub timestamp_policy: TimestampPolicy,
    
    /// 随机数策略
    pub random_policy: RandomPolicy,
    
    /// 声明的外部依赖列表
    pub external_deps: Vec<ExternalDep>,
    
    /// 环境变量哈希（仅包含白名单中的变量）
    pub env_vars_hash: String,
    
    /// 模型配置指纹（温度/top_p/max_tokens 等）
    pub model_config_hash: Option<String>,
}

/// 时间戳策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimestampPolicy {
    /// 禁止使用时间戳（完全确定性）
    Forbidden,
    
    /// 使用固定时间戳（由外部注入）
    Fixed { timestamp_ms: u64 },
    
    /// 允许使用但必须声明（用于审计）
    Injected { source: String },
}

/// 随机数策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RandomPolicy {
    /// 禁止使用随机数
    Forbidden,
    
    /// 使用固定种子
    Seeded { seed: u64 },
    
    /// 允许使用但必须声明
    Injected { source: String },
}

/// 外部依赖声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDep {
    /// 依赖类型（http_read/file_read/db_query 等）
    pub dep_type: String,
    
    /// 依赖标识（URL/路径/查询语句）
    pub identifier: String,
    
    /// 预期哈希（用于验证依赖内容未变）
    pub expected_hash: Option<String>,
    
    /// 是否只读
    pub read_only: bool,
}

/// 执行结果 - 仅包含状态差分，不直接提交
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// 执行状态
    pub status: ExecutionStatus,
    
    /// 状态差分（执行后 - 执行前）
    pub state_diff: Value,
    
    /// 执行轨迹哈希
    pub trace_hash: String,
    
    /// 结果哈希
    pub result_hash: String,
    
    /// 状态差分哈希
    pub state_diff_hash: String,
    
    /// 消耗的 Gas
    pub gas_used: u64,
    
    /// 错误码（成功时为 0）
    pub error_code: u32,
    
    /// 错误信息
    pub error_message: Option<String>,
}

/// 执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    /// 执行成功
    Success,
    
    /// 执行回滚（业务逻辑拒绝）
    Revert,
    
    /// 执行失败（系统错误）
    Failure,
}

/// 验证结果 - 独立重放验证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    /// 验证是否通过
    pub verified: bool,
    
    /// 重算的结果哈希
    pub recomputed_result_hash: String,
    
    /// 如果不一致，记录原因
    pub mismatch_reason: Option<String>,
    
    /// 验证消耗的 Gas
    pub gas_used: u64,
    
    /// 验证时间戳
    pub verified_at: u64,
}

impl DeterministicContract {
    /// 创建新契约
    pub fn new(
        program: String,
        input: Value,
        state_root: String,
        env_fingerprint: EnvFingerprint,
    ) -> Self {
        Self {
            contract_version: "v1.0".to_string(),
            program,
            input,
            state_root,
            env_fingerprint,
        }
    }
    
    /// 生成契约哈希（用于追溯和验证）
    pub fn hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.contract_version.hash(&mut hasher);
        self.program.hash(&mut hasher);
        serde_json::to_string(&self.input).unwrap_or_default().hash(&mut hasher);
        self.state_root.hash(&mut hasher);
        hasher.finish().to_string()
    }
}

impl EnvFingerprint {
    /// 创建默认环境指纹（严格模式）
    pub fn strict() -> Self {
        Self {
            timestamp_policy: TimestampPolicy::Forbidden,
            random_policy: RandomPolicy::Forbidden,
            external_deps: Vec::new(),
            env_vars_hash: "empty".to_string(),
            model_config_hash: None,
        }
    }
    
    /// 检查是否有隐式非确定性源
    pub fn has_implicit_nondeterminism(&self) -> bool {
        matches!(self.timestamp_policy, TimestampPolicy::Forbidden) ||
        matches!(self.random_policy, RandomPolicy::Forbidden)
    }
}

impl ExecutionResult {
    /// 创建成功结果
    pub fn success(
        state_diff: Value,
        trace_hash: String,
        result_hash: String,
        gas_used: u64,
    ) -> Self {
        let state_diff_hash = Self::compute_hash(&state_diff);
        Self {
            status: ExecutionStatus::Success,
            state_diff,
            trace_hash,
            result_hash,
            state_diff_hash,
            gas_used,
            error_code: 0,
            error_message: None,
        }
    }
    
    /// 创建失败结果
    pub fn failure(error_code: u32, error_message: String, gas_used: u64) -> Self {
        Self {
            status: ExecutionStatus::Failure,
            state_diff: Value::Null,
            trace_hash: String::new(),
            result_hash: String::new(),
            state_diff_hash: String::new(),
            gas_used,
            error_code,
            error_message: Some(error_message),
        }
    }
    
    fn compute_hash(value: &Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        serde_json::to_string(value).unwrap_or_default().hash(&mut hasher);
        hasher.finish().to_string()
    }
}

impl VerifyResult {
    /// 创建验证通过结果
    pub fn passed(recomputed_result_hash: String, gas_used: u64) -> Self {
        Self {
            verified: true,
            recomputed_result_hash,
            mismatch_reason: None,
            gas_used,
            verified_at: 0, // 由验证器填充
        }
    }
    
    /// 创建验证失败结果
    pub fn failed(mismatch_reason: String, recomputed_result_hash: String) -> Self {
        Self {
            verified: false,
            recomputed_result_hash,
            mismatch_reason: Some(mismatch_reason),
            gas_used: 0,
            verified_at: 0,
        }
    }
}

/// 契约校验器
pub struct ContractValidator;

impl ContractValidator {
    /// 验证契约完整性
    pub fn validate_contract(contract: &DeterministicContract) -> Result<(), String> {
        if contract.contract_version.is_empty() {
            return Err("contract_version is required".to_string());
        }
        if contract.program.is_empty() {
            return Err("program is required".to_string());
        }
        if contract.state_root.is_empty() {
            return Err("state_root is required".to_string());
        }
        Ok(())
    }
    
    /// 验证环境指纹合规性
    pub fn validate_fingerprint(fingerprint: &EnvFingerprint) -> Result<(), String> {
        // 检查是否有未声明的外部依赖
        for dep in &fingerprint.external_deps {
            if dep.dep_type.is_empty() {
                return Err("external dep dep_type is required".to_string());
            }
            if dep.identifier.is_empty() {
                return Err("external dep identifier is required".to_string());
            }
        }
        Ok(())
    }
    
    /// 验证执行结果
    pub fn validate_result(result: &ExecutionResult) -> Result<(), String> {
        match result.status {
            ExecutionStatus::Success => {
                if result.trace_hash.is_empty() {
                    return Err("trace_hash is required for success".to_string());
                }
                if result.result_hash.is_empty() {
                    return Err("result_hash is required for success".to_string());
                }
            }
            ExecutionStatus::Failure => {
                if result.error_message.is_none() {
                    return Err("error_message is required for failure".to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }
}
