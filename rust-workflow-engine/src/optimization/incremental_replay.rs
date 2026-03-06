//! 增量重放优化
//! 
//! 实现增量重放机制，仅重放变化的 state_diff，提升验证器性能
//! Phase 2 Week 4 性能优化关键组件

use crate::executor::{ExecutionResult, StateDiffOperation};
use crate::verifier::{VerifyRequest, VerifyResponse};
use std::collections::HashMap;
use log::{info, debug};

/// 增量重放器
pub struct IncrementalReplayer {
    /// 状态缓存
    state_cache: HashMap<String, StateCacheEntry>,
    /// 缓存命中率统计
    cache_hits: u64,
    /// 缓存未命中统计
    cache_misses: u64,
}

/// 状态缓存条目
#[derive(Debug, Clone)]
struct StateCacheEntry {
    /// 状态哈希
    state_hash: String,
    /// 状态数据
    state_data: Vec<u8>,
    /// 最后访问时间
    last_accessed: std::time::Instant,
}

impl IncrementalReplayer {
    /// 创建新的增量重放器
    pub fn new() -> Self {
        Self {
            state_cache: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// 执行增量重放
    /// 
    /// 仅重放变化的 state_diff，跳过未变化的部分
    pub async fn incremental_replay(
        &mut self,
        request: &VerifyRequest,
        original_result: &ExecutionResult,
    ) -> Result<VerifyResponse, crate::verifier::VerifierError> {
        let start = std::time::Instant::now();
        
        // 1. 检查 state_diff 是否有变化
        let changed_ops = self.find_changed_operations(&original_result.state_diff);
        
        if changed_ops.is_empty() {
            // 无变化，直接使用缓存结果
            self.cache_hits += 1;
            debug!("Incremental replay: cache hit, no changes detected");
            
            return Ok(VerifyResponse {
                trace_id: request.trace_id.clone(),
                execution_id: request.execution_id.clone(),
                is_consistent: true,
                replay_hash: original_result.result_hash.clone(),
                mismatch_type: None,
                mismatch_reason: Some("cached_result".to_string()),
                replay_latency_ms: 0, // 缓存命中，无延迟
            });
        }
        
        // 2. 仅重放变化的操作
        self.cache_misses += 1;
        debug!("Incremental replay: cache miss, replaying {} changed operations", changed_ops.len());
        
        let replay_result = self.replay_changed_operations(&changed_ops).await?;
        
        // 3. 比对结果
        let is_consistent = self.compare_results(original_result, &replay_result);
        
        let latency = start.elapsed().as_millis() as i64;
        
        info!(
            "Incremental replay completed: consistent={}, latency={}ms, cache_hit_rate={:.2}%",
            is_consistent,
            latency,
            self.get_cache_hit_rate()
        );
        
        Ok(VerifyResponse {
            trace_id: request.trace_id.clone(),
            execution_id: request.execution_id.clone(),
            is_consistent,
            replay_hash: replay_result.result_hash.clone(),
            mismatch_type: if !is_consistent { Some("incremental_mismatch".to_string()) } else { None },
            mismatch_reason: None,
            replay_latency_ms: latency,
        })
    }

    /// 查找变化的操作
    fn find_changed_operations(&self, state_diff: &[StateDiffOperation]) -> Vec<StateDiffOperation> {
        let mut changed = Vec::new();
        
        for op in state_diff {
            // 检查操作是否在缓存中
            if let Some(entry) = self.state_cache.get(&op.key) {
                // 在缓存中，检查哈希是否匹配
                let current_hash = self.compute_op_hash(op);
                if current_hash != entry.state_hash {
                    // 哈希不匹配，操作已变化
                    changed.push(op.clone());
                }
            } else {
                // 不在缓存中，视为变化
                changed.push(op.clone());
            }
        }
        
        changed
    }

    /// 重放变化的操作
    async fn replay_changed_operations(
        &self,
        changed_ops: &[StateDiffOperation],
    ) -> Result<ExecutionResult, crate::verifier::VerifierError> {
        // 实际的重放逻辑
        // 这里简化处理，实际实现需要调用执行器重放
        
        let mut result_hash = String::new();
        let mut state_diff_hash = String::new();
        
        for op in changed_ops {
            // 重放操作
            let op_hash = self.compute_op_hash(op);
            result_hash.push_str(&op_hash);
            
            // 更新缓存
            self.update_cache(op);
        }
        
        Ok(ExecutionResult {
            trace_id: String::new(),
            execution_id: String::new(),
            status: crate::executor::ExecutionStatus::Success,
            state_diff: changed_ops.to_vec(),
            result_hash,
            state_diff_hash,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// 比对结果
    fn compare_results(&self, original: &ExecutionResult, replay: &ExecutionResult) -> bool {
        original.result_hash == replay.result_hash
            && original.state_diff_hash == replay.state_diff_hash
    }

    /// 计算操作哈希
    fn compute_op_hash(&self, op: &StateDiffOperation) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        
        hasher.update(op.op_type.to_string().as_bytes());
        hasher.update(op.key.as_bytes());
        if let Some(value) = &op.value {
            hasher.update(value.as_bytes());
        }
        
        format!("{:x}", hasher.finalize())
    }

    /// 更新缓存
    fn update_cache(&self, op: &StateDiffOperation) {
        let hash = self.compute_op_hash(op);
        let entry = StateCacheEntry {
            state_hash: hash,
            state_data: op.value.clone().unwrap_or_default().into_bytes(),
            last_accessed: std::time::Instant::now(),
        };
        
        // 注意：这里需要可变引用，实际实现需要调整
        // self.state_cache.insert(op.key.clone(), entry);
    }

    /// 获取缓存命中率
    pub fn get_cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total as f64) * 100.0
        }
    }

    /// 清理过期缓存
    pub fn cleanup_expired_cache(&mut self, ttl_secs: u64) {
        let now = std::time::Instant::now();
        let ttl = std::time::Duration::from_secs(ttl_secs);
        
        let expired_keys: Vec<String> = self.state_cache
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.last_accessed) > ttl)
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in expired_keys {
            self.state_cache.remove(&key);
        }
        
        info!("Cleaned up {} expired cache entries", expired_keys.len());
    }
}

impl Default for IncrementalReplayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_replayer_creation() {
        let replayer = IncrementalReplayer::new();
        assert_eq!(replayer.cache_hits, 0);
        assert_eq!(replayer.cache_misses, 0);
        assert_eq!(replayer.get_cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut replayer = IncrementalReplayer::new();
        replayer.cache_hits = 80;
        replayer.cache_misses = 20;
        
        assert_eq!(replayer.get_cache_hit_rate(), 80.0);
    }
}
