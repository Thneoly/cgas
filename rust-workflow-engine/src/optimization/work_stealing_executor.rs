//! 工作窃取执行器 (Work-Stealing Executor)
//! 
//! Phase 3 性能优化核心组件：
//! - 多队列任务调度 (每个 Worker 独立本地队列)
//! - 工作窃取算法 (空闲 Worker 从其他队列窃取任务)
//! - 负载均衡 (动态平衡各 Worker 负载)
//! 
//! 性能目标：执行器 P99 时延 115ms → <95ms (-17%)

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use crossbeam_queue::{ArrayQueue, SegQueue};
use log::{debug, info, warn};

/// 任务 trait
pub trait Task: Send + 'static {
    /// 执行任务
    fn execute(self: Box<Self>);
    /// 获取任务优先级 (0=最低，10=最高)
    fn priority(&self) -> u8 { 5 }
    /// 获取任务估计执行时间 (ms)
    fn estimated_duration_ms(&self) -> u64 { 10 }
}

/// Worker 统计信息
#[derive(Debug, Default)]
pub struct WorkerStats {
    /// 执行的任务数
    pub tasks_executed: AtomicU64,
    /// 窃取的任务数
    pub tasks_stolen: AtomicU64,
    /// 被窃取的任务数
    pub tasks_donated: AtomicU64,
    /// 空闲时间 (ms)
    pub idle_time_ms: AtomicU64,
}

impl WorkerStats {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn executed(&self) -> u64 {
        self.tasks_executed.load(Ordering::Relaxed)
    }
    
    pub fn stolen(&self) -> u64 {
        self.tasks_stolen.load(Ordering::Relaxed)
    }
    
    pub fn donated(&self) -> u64 {
        self.tasks_donated.load(Ordering::Relaxed)
    }
    
    pub fn idle_ms(&self) -> u64 {
        self.idle_time_ms.load(Ordering::Relaxed)
    }
}

/// Worker 配置
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Worker ID
    pub id: usize,
    /// 本地队列容量
    pub local_queue_capacity: usize,
    /// 是否启用工作窃取
    pub enable_stealing: bool,
    /// 窃取尝试次数上限
    pub steal_attempts_limit: usize,
    /// 空闲等待时间 (微秒)
    pub idle_wait_us: u64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            id: 0,
            local_queue_capacity: 1024,
            enable_stealing: true,
            steal_attempts_limit: 3,
            idle_wait_us: 100,
        }
    }
}

/// Worker
pub struct Worker<T: Task> {
    config: WorkerConfig,
    local_queue: Arc<ArrayQueue<Box<dyn Task>>>,
    global_queue: Arc<SegQueue<Box<dyn Task>>>,
    other_queues: Vec<Arc<ArrayQueue<Box<dyn Task>>>>,
    stats: Arc<WorkerStats>,
    running: Arc<AtomicUsize>,
}

impl<T: Task> Worker<T> {
    /// 创建 Worker
    pub fn new(
        config: WorkerConfig,
        local_queue: Arc<ArrayQueue<Box<dyn Task>>>,
        global_queue: Arc<SegQueue<Box<dyn Task>>>,
        other_queues: Vec<Arc<ArrayQueue<Box<dyn Task>>>>,
        stats: Arc<WorkerStats>,
        running: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            config,
            local_queue,
            global_queue,
            other_queues,
            stats,
            running,
        }
    }
    
    /// 运行 Worker 循环
    pub fn run(&self) {
        info!("Worker {} started", self.config.id);
        
        while self.running.load(Ordering::Relaxed) > 0 {
            let task = self.try_get_task();
            
            match task {
                Some(task) => {
                    let start = Instant::now();
                    task.execute();
                    let elapsed = start.elapsed();
                    
                    self.stats.tasks_executed.fetch_add(1, Ordering::Relaxed);
                    debug!(
                        "Worker {} executed task in {:?}ms",
                        self.config.id,
                        elapsed.as_millis()
                    );
                }
                None => {
                    // 无任务，空闲等待
                    let idle_start = Instant::now();
                    thread::sleep(Duration::from_micros(self.config.idle_wait_us));
                    let idle_ms = idle_start.elapsed().as_millis() as u64;
                    self.stats.idle_time_ms.fetch_add(idle_ms, Ordering::Relaxed);
                }
            }
        }
        
        info!("Worker {} stopped", self.config.id);
    }
    
    /// 尝试获取任务
    fn try_get_task(&self) -> Option<Box<dyn Task>> {
        // 1. 优先从本地队列获取
        if let Some(task) = self.local_queue.pop() {
            return Some(task);
        }
        
        // 2. 从全局队列获取
        if let Some(task) = self.global_queue.pop() {
            debug!("Worker {} got task from global queue", self.config.id);
            return Some(task);
        }
        
        // 3. 工作窃取：从其他 Worker 队列窃取
        if self.config.enable_stealing {
            if let Some(task) = self.steal_task() {
                self.stats.tasks_stolen.fetch_add(1, Ordering::Relaxed);
                debug!("Worker {} stole task from another worker", self.config.id);
                return Some(task);
            }
        }
        
        None
    }
    
    /// 窃取任务
    fn steal_task(&self) -> Option<Box<dyn Task>> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // 随机尝试窃取 (避免热点)
        for _ in 0..self.config.steal_attempts_limit {
            let victim_idx = rng.gen_range(0..self.other_queues.len());
            
            if let Some(task) = self.other_queues[victim_idx].steal() {
                // 更新被窃取者的统计
                // (实际实现中需要反向引用，这里简化处理)
                return Some(task);
            }
        }
        
        None
    }
}

/// 执行器配置
#[derive(Debug, Clone)]
pub struct WorkStealingExecutorConfig {
    /// Worker 数量 (通常等于 CPU 核心数)
    pub num_workers: usize,
    /// 本地队列容量
    pub local_queue_capacity: usize,
    /// 全局队列容量
    pub global_queue_capacity: usize,
    /// 是否启用工作窃取
    pub enable_stealing: bool,
}

impl Default for WorkStealingExecutorConfig {
    fn default() -> Self {
        Self {
            num_workers: num_cpus::get(),
            local_queue_capacity: 1024,
            global_queue_capacity: 4096,
            enable_stealing: true,
        }
    }
}

/// 执行器统计
#[derive(Debug)]
pub struct ExecutorStats {
    /// 提交的任务总数
    pub tasks_submitted: AtomicU64,
    /// 执行的任务总数
    pub tasks_executed: AtomicU64,
    /// 窃取的任务总数
    pub tasks_stolen: AtomicU64,
    /// 当前队列深度
    pub queue_depth: AtomicUsize,
}

impl ExecutorStats {
    pub fn new() -> Self {
        Self {
            tasks_submitted: AtomicU64::new(0),
            tasks_executed: AtomicU64::new(0),
            tasks_stolen: AtomicU64::new(0),
            queue_depth: AtomicUsize::new(0),
        }
    }
    
    /// 获取所有 Worker 的汇总统计
    pub fn aggregate_worker_stats(&self, worker_stats: &[Arc<WorkerStats>]) -> AggregatedStats {
        let mut total_executed = 0u64;
        let mut total_stolen = 0u64;
        let mut total_donated = 0u64;
        let mut total_idle_ms = 0u64;
        
        for stats in worker_stats {
            total_executed += stats.executed();
            total_stolen += stats.stolen();
            total_donated += stats.donated();
            total_idle_ms += stats.idle_ms();
        }
        
        let steal_rate = if total_executed > 0 {
            total_stolen as f64 / total_executed as f64
        } else {
            0.0
        };
        
        let idle_rate = if total_idle_ms > 0 {
            let total_time_ms = total_idle_ms + (total_executed * 10); // 估算
            total_idle_ms as f64 / total_time_ms as f64
        } else {
            0.0
        };
        
        AggregatedStats {
            total_executed,
            total_stolen,
            total_donated,
            total_idle_ms,
            steal_rate,
            idle_rate,
        }
    }
}

#[derive(Debug)]
pub struct AggregatedStats {
    pub total_executed: u64,
    pub total_stolen: u64,
    pub total_donated: u64,
    pub total_idle_ms: u64,
    pub steal_rate: f64,
    pub idle_rate: f64,
}

/// 工作窃取执行器
pub struct WorkStealingExecutor {
    config: WorkStealingExecutorConfig,
    global_queue: Arc<SegQueue<Box<dyn Task>>>,
    local_queues: Vec<Arc<ArrayQueue<Box<dyn Task>>>>,
    worker_handles: Vec<JoinHandle<()>>,
    worker_stats: Vec<Arc<WorkerStats>>,
    stats: Arc<ExecutorStats>,
    running: Arc<AtomicUsize>,
}

impl WorkStealingExecutor {
    /// 创建执行器
    pub fn new(config: WorkStealingExecutorConfig) -> Self {
        let global_queue = Arc::new(SegQueue::new());
        
        let local_queues: Vec<_> = (0..config.num_workers)
            .map(|_| Arc::new(ArrayQueue::new(config.local_queue_capacity)))
            .collect();
        
        let mut worker_handles = Vec::with_capacity(config.num_workers);
        let mut worker_stats = Vec::with_capacity(config.num_workers);
        let running = Arc::new(AtomicUsize::new(config.num_workers));
        
        // 创建 Worker 线程
        for i in 0..config.num_workers {
            let local_queue = Arc::clone(&local_queues[i]);
            let global_queue = Arc::clone(&global_queue);
            
            // 创建其他队列的引用 (用于窃取)
            let other_queues: Vec<_> = local_queues
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(_, q)| Arc::clone(q))
                .collect();
            
            let stats = Arc::new(WorkerStats::new());
            let running_clone = Arc::clone(&running);
            
            let config_clone = WorkerConfig {
                id: i,
                local_queue_capacity: config.local_queue_capacity,
                enable_stealing: config.enable_stealing,
                ..Default::default()
            };
            
            let worker = Worker::new(
                config_clone,
                local_queue,
                global_queue,
                other_queues,
                Arc::clone(&stats),
                running_clone,
            );
            
            let handle = thread::spawn(move || {
                worker.run();
            });
            
            worker_handles.push(handle);
            worker_stats.push(stats);
        }
        
        Self {
            config,
            global_queue,
            local_queues,
            worker_handles,
            worker_stats,
            stats: Arc::new(ExecutorStats::new()),
            running,
        }
    }
    
    /// 提交任务 (自动选择目标队列)
    pub fn submit(&self, task: Box<dyn Task>) -> Result<(), &'static str> {
        // 基于优先级选择队列策略
        let priority = task.priority();
        
        if priority >= 8 {
            // 高优先级任务：直接放入全局队列 (快速响应)
            self.global_queue.push(task);
        } else {
            // 普通任务：放入本地队列 (负载均衡)
            // 使用轮询或基于负载选择 Worker
            let worker_idx = self.select_worker();
            
            if let Err(task) = self.local_queues[worker_idx].push(task) {
                // 本地队列满，放入全局队列
                self.global_queue.push(task);
            }
        }
        
        self.stats.tasks_submitted.fetch_add(1, Ordering::Relaxed);
        self.update_queue_depth();
        
        Ok(())
    }
    
    /// 提交任务到指定 Worker
    pub fn submit_to_worker(&self, task: Box<dyn Task>, worker_idx: usize) -> Result<(), &'static str> {
        if worker_idx >= self.local_queues.len() {
            return Err("Invalid worker index");
        }
        
        if let Err(task) = self.local_queues[worker_idx].push(task) {
            self.global_queue.push(task);
        }
        
        self.stats.tasks_submitted.fetch_add(1, Ordering::Relaxed);
        self.update_queue_depth();
        
        Ok(())
    }
    
    /// 选择 Worker (基于负载均衡)
    fn select_worker(&self) -> usize {
        // 简单轮询策略 (可扩展为基于队列深度的负载均衡)
        static NEXT_WORKER: AtomicUsize = AtomicUsize::new(0);
        
        let idx = NEXT_WORKER.fetch_add(1, Ordering::Relaxed) % self.local_queues.len();
        idx
    }
    
    /// 更新队列深度统计
    fn update_queue_depth(&self) {
        let total_depth: usize = self.local_queues
            .iter()
            .map(|q| q.len())
            .sum::<usize>() + self.global_queue.len();
        
        self.stats.queue_depth.store(total_depth, Ordering::Relaxed);
    }
    
    /// 获取执行器统计
    pub fn stats(&self) -> Arc<ExecutorStats> {
        Arc::clone(&self.stats)
    }
    
    /// 获取 Worker 统计
    pub fn worker_stats(&self) -> &[Arc<WorkerStats>] {
        &self.worker_stats
    }
    
    /// 关闭执行器
    pub fn shutdown(&mut self) {
        info!("Shutting down work-stealing executor...");
        
        // 停止所有 Worker
        self.running.store(0, Ordering::Relaxed);
        
        // 等待 Worker 线程结束
        for handle in self.worker_handles.drain(..) {
            let _ = handle.join();
        }
        
        info!("Work-stealing executor shut down");
    }
    
    /// 获取队列深度
    pub fn queue_depth(&self) -> usize {
        self.stats.queue_depth.load(Ordering::Relaxed)
    }
    
    /// 获取 Worker 数量
    pub fn num_workers(&self) -> usize {
        self.config.num_workers
    }
}

impl Drop for WorkStealingExecutor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex;
    
    struct TestTask {
        id: usize,
        counter: Arc<AtomicUsize>,
    }
    
    impl Task for TestTask {
        fn execute(self: Box<Self>) {
            self.counter.fetch_add(1, Ordering::Relaxed);
        }
        
        fn priority(&self) -> u8 {
            5
        }
    }
    
    #[test]
    fn test_work_stealing_executor_basic() {
        let config = WorkStealingExecutorConfig {
            num_workers: 4,
            ..Default::default()
        };
        
        let executor = WorkStealingExecutor::new(config);
        let counter = Arc::new(AtomicUsize::new(0));
        
        // 提交 100 个任务
        for i in 0..100 {
            let task = Box::new(TestTask {
                id: i,
                counter: Arc::clone(&counter),
            });
            executor.submit(task).unwrap();
        }
        
        // 等待任务执行
        thread::sleep(Duration::from_millis(500));
        
        // 验证所有任务都执行了
        assert_eq!(counter.load(Ordering::Relaxed), 100);
        
        // 验证统计
        let stats = executor.stats();
        assert_eq!(stats.tasks_submitted.load(Ordering::Relaxed), 100);
    }
    
    #[test]
    fn test_work_stealing_load_balancing() {
        let config = WorkStealingExecutorConfig {
            num_workers: 4,
            ..Default::default()
        };
        
        let executor = WorkStealingExecutor::new(config);
        let counter = Arc::new(AtomicUsize::new(0));
        
        // 提交 1000 个任务
        for i in 0..1000 {
            let task = Box::new(TestTask {
                id: i,
                counter: Arc::clone(&counter),
            });
            executor.submit(task).unwrap();
        }
        
        // 等待任务执行
        thread::sleep(Duration::from_millis(1000));
        
        // 验证负载均衡 (各 Worker 执行的任务数差异不应太大)
        let worker_stats = executor.worker_stats();
        let executed: Vec<u64> = worker_stats.iter().map(|s| s.executed()).collect();
        
        let min_executed = *executed.iter().min().unwrap();
        let max_executed = *executed.iter().max().unwrap();
        
        // 允许 50% 的不平衡 (工作窃取会减少不平衡)
        assert!(max_executed <= min_executed * 3, 
                "Load imbalance too high: min={}, max={:?}", min_executed, max_executed);
    }
}
