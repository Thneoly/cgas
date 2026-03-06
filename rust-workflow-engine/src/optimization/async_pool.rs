//! 异步并发池
//! 
//! 实现异步并发池，优化执行器和验证器的并发性能
//! Phase 2 Week 4 性能优化关键组件

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use std::sync::Arc;
use log::{info, error};

/// 异步并发池配置
#[derive(Debug, Clone)]
pub struct AsyncPoolConfig {
    /// 工作线程数量
    pub worker_count: usize,
    /// 任务队列容量
    pub queue_capacity: usize,
    /// 任务超时时间 (毫秒)
    pub task_timeout_ms: u64,
}

impl Default for AsyncPoolConfig {
    fn default() -> Self {
        Self {
            worker_count: num_cpus::get() * 2, // CPU 核心数 * 2
            queue_capacity: 1000,
            task_timeout_ms: 500, // 500ms 超时
        }
    }
}

/// 异步并发池
pub struct AsyncPool<T, R> {
    /// 任务发送通道
    task_tx: mpsc::Sender<AsyncTask<T, R>>,
    /// 工作线程句柄
    workers: Vec<JoinHandle<()>>,
    /// 配置
    config: AsyncPoolConfig,
}

/// 异步任务
struct AsyncTask<T, R> {
    /// 任务 ID
    id: u64,
    /// 任务输入
    input: T,
    /// 任务响应通道
    response_tx: tokio::sync::oneshot::Sender<Result<R, AsyncTaskError>>,
}

/// 异步任务错误
#[derive(Debug, thiserror::Error)]
pub enum AsyncTaskError {
    #[error("任务超时")]
    Timeout,
    
    #[error("任务执行失败：{0}")]
    ExecutionFailed(String),
    
    #[error("通道关闭")]
    ChannelClosed,
}

impl<T, R> AsyncPool<T, R>
where
    T: Send + 'static,
    R: Send + 'static,
{
    /// 创建新的异步并发池
    pub fn new<F>(config: AsyncPoolConfig, task_fn: Arc<F>) -> Self
    where
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        let (task_tx, mut task_rx) = mpsc::channel::<AsyncTask<T, R>>(config.queue_capacity);
        let mut workers = Vec::with_capacity(config.worker_count);

        // 创建工作线程
        for worker_id in 0..config.worker_count {
            let task_rx = task_rx.resubscribe();
            let task_fn = Arc::clone(&task_fn);
            
            let handle = tokio::spawn(async move {
                info!("AsyncPool worker {} started", worker_id);
                
                while let Some(task) = task_rx.recv().await {
                    // 执行任务
                    let result = tokio::task::spawn_blocking({
                        let task_fn = Arc::clone(&task_fn);
                        move || task_fn(task.input)
                    })
                    .await;
                    
                    // 发送结果
                    let response = match result {
                        Ok(r) => Ok(r),
                        Err(e) => Err(AsyncTaskError::ExecutionFailed(e.to_string())),
                    };
                    
                    if let Err(_) = task.response_tx.send(response) {
                        error!("Failed to send task response");
                    }
                }
                
                info!("AsyncPool worker {} stopped", worker_id);
            });
            
            workers.push(handle);
        }

        Self {
            task_tx,
            workers,
            config,
        }
    }

    /// 提交任务
    pub async fn submit(&self, input: T) -> Result<R, AsyncTaskError> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        
        let task = AsyncTask {
            id: 0, // 实际使用时应生成唯一 ID
            input,
            response_tx,
        };
        
        // 发送任务
        self.task_tx.send(task).await
            .map_err(|_| AsyncTaskError::ChannelClosed)?;
        
        // 等待结果 (带超时)
        tokio::time::timeout(
            std::time::Duration::from_millis(self.config.task_timeout_ms),
            response_rx
        )
        .await
        .map_err(|_| AsyncTaskError::Timeout)?
        .map_err(|_| AsyncTaskError::ChannelClosed)?
    }

    /// 批量提交任务
    pub async fn submit_batch(&self, inputs: Vec<T>) -> Vec<Result<R, AsyncTaskError>> {
        let mut handles = Vec::with_capacity(inputs.len());
        
        for input in inputs {
            let handle = self.submit(input);
            handles.push(handle);
        }
        
        // 等待所有任务完成
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            let result = handle.await;
            results.push(result);
        }
        
        results
    }

    /// 关闭并发池
    pub async fn shutdown(self) {
        // 停止接收新任务
        drop(self.task_tx);
        
        // 等待所有工作线程完成
        for worker in self.workers {
            let _ = worker.await;
        }
        
        info!("AsyncPool shutdown complete");
    }
}

/// 执行器异步并发池
pub type ExecutorAsyncPool = AsyncPool<
    crate::executor::ExecuteRequest,
    Result<crate::executor::ExecutionResult, crate::executor::ExecutorError>,
>;

/// 验证器异步并发池
pub type VerifierAsyncPool = AsyncPool<
    crate::verifier::VerifyRequest,
    Result<crate::verifier::VerifyResponse, crate::verifier::VerifierError>,
>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_async_pool_creation() {
        let config = AsyncPoolConfig::default();
        let task_fn = Arc::new(|x: i32| x * 2);
        let pool = AsyncPool::new(config, task_fn);
        
        // 验证并发池创建成功
        assert_eq!(pool.workers.len(), num_cpus::get() * 2);
    }

    #[tokio::test]
    async fn test_async_pool_submit() {
        let config = AsyncPoolConfig {
            worker_count: 2,
            queue_capacity: 10,
            task_timeout_ms: 500,
        };
        let task_fn = Arc::new(|x: i32| x * 2);
        let pool = AsyncPool::new(config, task_fn);
        
        let result = pool.submit(5).await.unwrap();
        assert_eq!(result, 10);
    }

    #[tokio::test]
    async fn test_async_pool_batch_submit() {
        let config = AsyncPoolConfig {
            worker_count: 2,
            queue_capacity: 10,
            task_timeout_ms: 500,
        };
        let task_fn = Arc::new(|x: i32| x * 2);
        let pool = AsyncPool::new(config, task_fn);
        
        let inputs = vec![1, 2, 3, 4, 5];
        let results = pool.submit_batch(inputs).await;
        
        assert_eq!(results.len(), 5);
        assert_eq!(results[0].as_ref().unwrap(), &2);
        assert_eq!(results[1].as_ref().unwrap(), &4);
        assert_eq!(results[2].as_ref().unwrap(), &6);
        assert_eq!(results[3].as_ref().unwrap(), &8);
        assert_eq!(results[4].as_ref().unwrap(), &10);
    }
}
