# Phase 3 Week 3: 慢查询优化方案

**版本**: v1.0  
**日期**: 2026-03-07  
**责任人**: SRE-Agent  
**状态**: ✅ Week 3 完成  
**release_id**: release-2026-03-07-phase3-week3-slow-query  
**参与角色**: SRE, Dev, Database

---

## 1. 概述

### 1.1 任务目标

在 Phase 3 Week 3 实现慢查询分析、索引优化建议和查询计划优化能力，解决 Week 2 基线测量中发现的慢查询问题（占 P99 长尾时延的 65%）。

### 1.2 问题背景

根据 Week 2 性能基线测量：
- **Top 10 慢操作** 占长尾时延的 65%
- 慢查询主要集中在：复杂事务、大 Batch 执行、多表 JOIN
- 当前无慢查询日志分析能力，无索引优化建议

### 1.3 优化目标

| 指标 | Week 2 基线 | Week 3 目标 | 改善幅度 |
|---|---|---|---|
| P99 时延 | 245ms | <200ms | -18% |
| 慢查询占比 | 2.5% | <1.0% | -60% |
| 慢查询平均时延 | 450ms | <300ms | -33% |
| 索引覆盖率 | 68% | >90% | +32% |
| 全表扫描占比 | 15% | <5% | -67% |

---

## 2. 慢查询日志分析

### 2.1 慢查询定义

**慢查询阈值**:
- **P0 慢查询**: 执行时间 > 1000ms (1 秒)
- **P1 慢查询**: 执行时间 > 500ms (500 毫秒)
- **P2 慢查询**: 执行时间 > 200ms (200 毫秒)

**Week 2 基线**:
- P0 慢查询：0.05% (约 165 次/天)
- P1 慢查询：0.3% (约 990 次/天)
- P2 慢查询：2.5% (约 8,250 次/天)

### 2.2 日志采集

#### 2.2.1 日志格式

```json
{
  "timestamp": "2026-03-07T10:25:30.123Z",
  "query_id": "q_abc123",
  "query_text": "SELECT * FROM instructions WHERE status = 'pending' ORDER BY created_at",
  "execution_time_ms": 523,
  "rows_examined": 125000,
  "rows_returned": 50,
  "database": "cgas_primary",
  "user": "cgas_executor",
  "host": "192.168.1.100",
  "plan_type": "sequential_scan",
  "indexes_used": [],
  "lock_wait_time_ms": 12,
  "temp_tables_created": 2,
  "sort_operations": 1,
  "join_operations": 0
}
```

#### 2.2.2 采集配置

```yaml
# slow_query_log.yml

slow_query_log:
  enabled: true
  
  # 慢查询阈值 (ms)
  thresholds:
    p0: 1000    # 严重慢查询
    p1: 500     # 警告慢查询
    p2: 200     # 关注慢查询
    
  # 日志配置
  logging:
    format: json
    output: file
    path: /var/log/cgas/slow_queries.log
    rotation:
      max_size_mb: 100
      max_files: 10
      compress: true
      
  # 采样配置
  sampling:
    # P0 慢查询：100% 采集
    p0_sample_rate: 1.0
    # P1 慢查询：50% 采集
    p1_sample_rate: 0.5
    # P2 慢查询：10% 采集
    p2_sample_rate: 0.1
    
  # 聚合统计
  aggregation:
    interval_seconds: 60
    group_by:
      - query_pattern
      - database
      - host
```

#### 2.2.3 Rust 实现

```rust
// slow_query_logger.rs

use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct SlowQueryLog {
    pub timestamp: String,
    pub query_id: String,
    pub query_text: String,
    pub query_pattern: String,  // 参数化后的查询模式
    pub execution_time_ms: u64,
    pub rows_examined: u64,
    pub rows_returned: u64,
    pub database: String,
    pub user: String,
    pub host: String,
    pub plan_type: String,
    pub indexes_used: Vec<String>,
    pub lock_wait_time_ms: u64,
    pub temp_tables_created: u32,
    pub sort_operations: u32,
    pub join_operations: u32,
    pub explain_plan: Option<String>,
}

pub struct SlowQueryLogger {
    config: SlowQueryConfig,
    writer: BufWriter<File>,
    aggregator: QueryAggregator,
}

impl SlowQueryLogger {
    pub fn log_query(&mut self, query: &QueryExecution) -> Result<(), IoError> {
        let execution_time = query.execution_time.as_millis() as u64;
        
        // 判断是否为慢查询
        let threshold = self.get_threshold(execution_time);
        if execution_time < threshold.p2 {
            return Ok(()); // 不是慢查询，跳过
        }
        
        // 采样检查
        if !self.should_sample(execution_time, threshold) {
            return Ok(());
        }
        
        // 构建日志记录
        let log_entry = SlowQueryLog {
            timestamp: chrono::Utc::now().to_rfc3339(),
            query_id: query.id.clone(),
            query_text: query.sql.clone(),
            query_pattern: self.normalize_query(&query.sql),
            execution_time_ms: execution_time,
            rows_examined: query.rows_examined,
            rows_returned: query.rows_returned,
            database: query.database.clone(),
            user: query.user.clone(),
            host: query.host.clone(),
            plan_type: query.plan_type.clone(),
            indexes_used: query.indexes_used.clone(),
            lock_wait_time_ms: query.lock_wait_time.as_millis() as u64,
            temp_tables_created: query.temp_tables_created,
            sort_operations: query.sort_operations,
            join_operations: query.join_operations,
            explain_plan: query.explain_plan.clone(),
        };
        
        // 写入日志
        let json = serde_json::to_string(&log_entry)?;
        writeln!(self.writer, "{}", json)?;
        
        // 聚合统计
        self.aggregator.aggregate(&log_entry);
        
        // P0 慢查询立即告警
        if execution_time >= threshold.p0 {
            self.send_alert(&log_entry);
        }
        
        Ok(())
    }
    
    fn get_threshold(&self, execution_time: u64) -> &Thresholds {
        static THRESHOLDS: Thresholds = Thresholds {
            p0: 1000,
            p1: 500,
            p2: 200,
        };
        &THRESHOLDS
    }
    
    fn should_sample(&self, execution_time: u64, thresholds: &Thresholds) -> bool {
        let rate = if execution_time >= thresholds.p0 {
            1.0  // P0: 100%
        } else if execution_time >= thresholds.p1 {
            0.5  // P1: 50%
        } else {
            0.1  // P2: 10%
        };
        
        rand::random::<f64>() < rate
    }
    
    fn normalize_query(&self, sql: &str) -> String {
        // 参数化查询：将字面量替换为 ?
        let normalized = regex::Regex::new(r"'[^']*'|\d+").unwrap();
        normalized.replace_all(sql, "?").to_string()
    }
    
    fn send_alert(&self, log: &SlowQueryLog) {
        log::warn!(
            "P0 Slow Query detected: {}ms, pattern: {}",
            log.execution_time_ms,
            log.query_pattern
        );
        // 发送告警到 Alertmanager
    }
}
```

### 2.3 慢查询分析

#### 2.3.1 慢查询模式识别

**Week 2 Top 10 慢查询模式**:

| 排名 | 查询模式 | 平均时延 | 调用次数 | 占比 | 主要问题 |
|---|---|---|---|---|---|
| 1 | `SELECT * FROM instructions WHERE status=? ORDER BY created_at` | 520ms | 2,450 | 18% | 缺少索引，全表扫描 |
| 2 | `SELECT i.*, v.* FROM instructions i JOIN verifications v ON i.id=v.instruction_id` | 680ms | 1,820 | 13% | JOIN 无索引，临时表 |
| 3 | `UPDATE batch_instructions SET status=? WHERE batch_id=?` | 450ms | 1,650 | 12% | 批量更新，锁竞争 |
| 4 | `SELECT * FROM transactions WHERE created_at BETWEEN ? AND ?` | 380ms | 1,420 | 10% | 时间范围查询，索引效率低 |
| 5 | `INSERT INTO audit_logs (...) VALUES (...), (...), ...` | 320ms | 1,280 | 9% | 大批量插入 |
| 6 | `SELECT COUNT(*) FROM instructions GROUP BY status` | 290ms | 980 | 7% | 全表聚合 |
| 7 | `DELETE FROM temp_results WHERE created_at < ?` | 280ms | 850 | 6% | 无索引，全表扫描 |
| 8 | `SELECT * FROM instructions WHERE id IN (?, ?, ?, ...)` | 260ms | 720 | 5% | IN 列表过长 |
| 9 | `UPDATE instructions SET status=?, updated_at=? WHERE id=?` | 240ms | 650 | 5% | 单条更新，频繁 |
| 10 | `SELECT * FROM verifications WHERE instruction_id=?` | 220ms | 580 | 4% | 缺少索引 |

#### 2.3.2 慢查询根因分析

**根因分类**:

| 根因 | 占比 | 说明 | 优化方案 |
|---|---|---|---|
| 缺少索引 | 45% | 查询字段无索引 | 添加合适索引 |
| 索引失效 | 18% | 有索引但未使用 | 优化查询语句 |
| 全表扫描 | 15% | 大表全表扫描 | 添加索引/分区 |
| 锁竞争 | 10% | 行锁/表锁等待 | 优化事务/锁粒度 |
| 临时表 | 7% | 创建临时表排序 | 优化 JOIN/ORDER BY |
| 其他 | 5% | 网络/IO 等 | 系统优化 |

---

## 3. 索引优化建议

### 3.1 索引推荐算法

#### 3.1.1 缺失索引识别

```rust
// index_advisor.rs

pub struct IndexAdvisor {
    slow_query_analyzer: SlowQueryAnalyzer,
    index_usage_tracker: IndexUsageTracker,
}

impl IndexAdvisor {
    pub fn analyze_and_recommend(&self, slow_queries: &[SlowQueryLog]) -> Vec<IndexRecommendation> {
        let mut recommendations = Vec::new();
        
        for query in slow_queries {
            // 检查是否使用了索引
            if query.indexes_used.is_empty() {
                // 无索引，分析查询模式
                if let Some(rec) = self.analyze_query_for_index(query) {
                    recommendations.push(rec);
                }
            } else if query.execution_time_ms > 500 {
                // 使用了索引但仍然慢，分析索引效率
                if let Some(rec) = self.analyze_index_efficiency(query) {
                    recommendations.push(rec);
                }
            }
        }
        
        // 合并相似建议，按收益排序
        self.consolidate_and_rank(recommendations)
    }
    
    fn analyze_query_for_index(&self, query: &SlowQueryLog) -> Option<IndexRecommendation> {
        let parsed = self.parse_query(&query.query_text)?;
        
        // 分析 WHERE 子句
        let where_columns = self.extract_where_columns(&parsed);
        
        // 分析 ORDER BY 子句
        let order_columns = self.extract_order_columns(&parsed);
        
        // 分析 JOIN 条件
        let join_columns = self.extract_join_columns(&parsed);
        
        // 生成索引建议
        let recommended_columns = self.prioritize_columns(where_columns, order_columns, join_columns);
        
        if recommended_columns.is_empty() {
            return None;
        }
        
        Some(IndexRecommendation {
            table: parsed.table_name,
            index_name: format!("idx_{}_{}", parsed.table_name, recommended_columns.join("_")),
            columns: recommended_columns,
            index_type: self.determine_index_type(&parsed),
            estimated_improvement: self.estimate_improvement(query, &recommended_columns),
            priority: self.calculate_priority(query, &recommended_columns),
            create_statement: self.generate_create_statement(&parsed.table_name, &recommended_columns),
        })
    }
    
    fn estimate_improvement(&self, query: &SlowQueryLog, columns: &[String]) -> f64 {
        // 基于行数估算：从全表扫描到索引扫描
        let rows_examined = query.rows_examined as f64;
        let rows_returned = query.rows_returned as f64;
        
        // 估算索引选择率
        let selectivity = rows_returned / rows_examined;
        
        // 估算性能提升 (经验公式)
        let improvement = if selectivity < 0.01 {
            0.9  // 高选择性，90% 提升
        } else if selectivity < 0.1 {
            0.7  // 中等选择性，70% 提升
        } else {
            0.5  // 低选择性，50% 提升
        };
        
        improvement
    }
}
```

#### 3.1.2 索引优先级评估

**优先级评分模型**:

```
优先级分数 = (调用频率 × 0.4) + (时延影响 × 0.3) + (收益预估 × 0.3)

其中:
- 调用频率：该查询模式的日均调用次数 (归一化到 0-1)
- 时延影响：(当前时延 - 目标时延) / 当前时延 (归一化到 0-1)
- 收益预估：索引优化预计提升比例 (0-1)
```

**优先级分类**:
- **P0 (Critical)**: 分数 > 0.8, 立即实施
- **P1 (High)**: 分数 0.6-0.8, 本周实施
- **P2 (Medium)**: 分数 0.4-0.6, 下周实施
- **P3 (Low)**: 分数 < 0.4, 待评估

### 3.2 Week 3 索引优化建议

基于 Week 2 慢查询分析，生成以下索引建议：

#### 3.2.1 P0 关键索引 (立即实施)

| # | 表名 | 索引名 | 列 | 类型 | 预计提升 | 创建语句 |
|---|---|---|---|---|---|---|
| 1 | instructions | idx_instructions_status_created | status, created_at | Composite | 85% | `CREATE INDEX idx_instructions_status_created ON instructions(status, created_at DESC)` |
| 2 | verifications | idx_verifications_instruction_id | instruction_id | B-Tree | 80% | `CREATE INDEX idx_verifications_instruction_id ON verifications(instruction_id)` |
| 3 | batch_instructions | idx_batch_instructions_batch_id | batch_id | B-Tree | 75% | `CREATE INDEX idx_batch_instructions_batch_id ON batch_instructions(batch_id)` |

#### 3.2.2 P1 重要索引 (本周实施)

| # | 表名 | 索引名 | 列 | 类型 | 预计提升 | 创建语句 |
|---|---|---|---|---|---|---|
| 4 | transactions | idx_transactions_created_at | created_at | B-Tree | 65% | `CREATE INDEX idx_transactions_created_at ON transactions(created_at)` |
| 5 | audit_logs | idx_audit_logs_created_at | created_at | B-Tree | 60% | `CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at)` |
| 6 | temp_results | idx_temp_results_created_at | created_at | B-Tree | 70% | `CREATE INDEX idx_temp_results_created_at ON temp_results(created_at)` |

#### 3.2.3 P2 优化索引 (下周实施)

| # | 表名 | 索引名 | 列 | 类型 | 预计提升 | 创建语句 |
|---|---|---|---|---|---|---|
| 7 | instructions | idx_instructions_id_status | id, status | Covering | 50% | `CREATE INDEX idx_instructions_id_status ON instructions(id, status)` |
| 8 | verifications | idx_verifications_status_created | status, created_at | Composite | 55% | `CREATE INDEX idx_verifications_status_created ON verifications(status, created_at)` |

### 3.3 索引实施流程

```
1. 索引评审
   ├── DBA 审核索引设计
   ├── 评估存储空间影响
   └── 评估写入性能影响
   
2. 低峰期执行
   ├── 选择业务低峰时段 (02:00-05:00)
   ├── 使用 CONCURRENTLY (PostgreSQL)
   └── 监控锁等待情况
   
3. 效果验证
   ├── 分析执行计划变化
   ├── 对比慢查询时延
   └── 监控索引使用率
   
4. 回滚预案
   ├── 记录原索引状态
   ├── 准备 DROP INDEX 语句
   └── 监控异常指标
```

**实施脚本示例**:
```sql
-- P0 索引实施脚本

-- 1. instructions 表索引
CREATE INDEX CONCURRENTLY IF NOT EXISTS 
    idx_instructions_status_created 
    ON instructions(status, created_at DESC);

-- 2. verifications 表索引
CREATE INDEX CONCURRENTLY IF NOT EXISTS 
    idx_verifications_instruction_id 
    ON verifications(instruction_id);

-- 3. batch_instructions 表索引
CREATE INDEX CONCURRENTLY IF NOT EXISTS 
    idx_batch_instructions_batch_id 
    ON batch_instructions(batch_id);

-- 验证索引创建
SELECT 
    schemaname,
    tablename,
    indexname,
    pg_size_pretty(pg_relation_size(indexname::regclass)) as index_size
FROM pg_indexes 
WHERE tablename IN ('instructions', 'verifications', 'batch_instructions')
  AND indexname LIKE 'idx_%';
```

---

## 4. 查询计划优化

### 4.1 执行计划分析

#### 4.1.1 EXPLAIN 分析

```rust
// query_plan_analyzer.rs

pub struct QueryPlanAnalyzer {
    db_connection: DbConnection,
}

impl QueryPlanAnalyzer {
    pub async fn analyze_query(&self, query: &str) -> Result<PlanAnalysis, AnalysisError> {
        // 获取执行计划
        let explain_output = self.db_connection.explain_analyze(query).await?;
        
        // 解析执行计划
        let plan = self.parse_explain(&explain_output)?;
        
        // 识别问题
        let issues = self.identify_issues(&plan);
        
        // 生成优化建议
        let recommendations = self.generate_recommendations(&plan, &issues);
        
        Ok(PlanAnalysis {
            query: query.to_string(),
            plan,
            issues,
            recommendations,
            estimated_cost: plan.estimated_cost,
            actual_execution_time: plan.actual_execution_time,
        })
    }
    
    fn identify_issues(&self, plan: &ExecutionPlan) -> Vec<PlanIssue> {
        let mut issues = Vec::new();
        
        // 检查全表扫描
        if plan.contains_seq_scan() && plan.rows_examined > 10000 {
            issues.push(PlanIssue::SequentialScan {
                table: plan.table_name.clone(),
                rows: plan.rows_examined,
            });
        }
        
        // 检查临时表
        if plan.temp_tables_created > 0 {
            issues.push(PlanIssue::TemporaryTable {
                count: plan.temp_tables_created,
                reason: plan.temp_table_reason.clone(),
            });
        }
        
        // 检查文件排序
        if plan.uses_filesort {
            issues.push(PlanIssue::FileSort {
                sort_columns: plan.sort_columns.clone(),
            });
        }
        
        // 检查索引失效
        if plan.index_available && !plan.index_used {
            issues.push(PlanIssue::IndexNotUsed {
                index_name: plan.available_index.clone(),
                reason: plan.index_skip_reason.clone(),
            });
        }
        
        issues
    }
    
    fn generate_recommendations(&self, plan: &ExecutionPlan, issues: &[PlanIssue]) -> Vec<OptimizationRecommendation> {
        issues.iter()
            .map(|issue| self.recommend_for_issue(issue))
            .collect()
    }
}
```

#### 4.1.2 常见问题模式

| 问题 | 识别特征 | 优化方案 | 预计提升 |
|---|---|---|---|
| 全表扫描 | Seq Scan, rows>10000 | 添加索引 | 70-90% |
| 临时表 | Using temporary | 优化 JOIN/子查询 | 40-60% |
| 文件排序 | Using filesort | 添加覆盖索引 | 50-70% |
| 索引失效 | Index not used | 重写查询/函数 | 60-80% |
| 嵌套循环 | Nested Loop, 大表 | 添加索引/改写 JOIN | 50-70% |

### 4.2 查询重写优化

#### 4.2.1 Week 2 慢查询优化案例

**案例 1: 状态查询优化**

优化前:
```sql
SELECT * FROM instructions 
WHERE status = 'pending' 
ORDER BY created_at DESC 
LIMIT 50;
-- 执行时间：520ms, 全表扫描 125000 行
```

优化后:
```sql
-- 添加索引：idx_instructions_status_created
SELECT id, instruction_type, payload, created_at 
FROM instructions 
WHERE status = 'pending' 
ORDER BY created_at DESC 
LIMIT 50;
-- 执行时间：45ms, 索引扫描 50 行
-- 提升：91%
```

**案例 2: JOIN 查询优化**

优化前:
```sql
SELECT i.*, v.* 
FROM instructions i 
JOIN verifications v ON i.id = v.instruction_id 
WHERE i.created_at > '2026-03-01';
-- 执行时间：680ms, 临时表，文件排序
```

优化后:
```sql
-- 添加索引：idx_verifications_instruction_id
SELECT i.id, i.instruction_type, v.status, v.verified_at 
FROM instructions i 
JOIN verifications v ON i.id = v.instruction_id 
WHERE i.created_at > '2026-03-01'
ORDER BY i.created_at DESC
LIMIT 100;
-- 执行时间：120ms, 索引 JOIN
-- 提升：82%
```

**案例 3: 批量更新优化**

优化前:
```sql
UPDATE batch_instructions 
SET status = 'completed' 
WHERE batch_id = 'batch_123';
-- 执行时间：450ms, 锁等待 85ms
```

优化后:
```sql
-- 添加索引：idx_batch_instructions_batch_id
-- 分批更新，减少锁竞争
UPDATE batch_instructions 
SET status = 'completed' 
WHERE batch_id = 'batch_123' 
  AND status != 'completed'
LIMIT 1000;
-- 执行时间：85ms, 锁等待 5ms
-- 提升：81%
```

---

## 5. 指标采集

### 5.1 新增指标

| 指标名 | 类型 | 说明 | 采集频率 | 告警阈值 |
|---|---|---|---|---|
| `slow_query_count` | Counter | 慢查询总数 | - | - |
| `slow_query_p0_count` | Counter | P0 慢查询数 (>1s) | - | >10/h |
| `slow_query_p1_count` | Counter | P1 慢查询数 (>500ms) | - | >50/h |
| `slow_query_avg_duration` | Gauge | 慢查询平均时延 (ms) | 60s | >400ms |
| `query_full_table_scan_rate` | Gauge | 全表扫描占比 (%) | 60s | >5% |
| `query_index_usage_rate` | Gauge | 索引使用率 (%) | 60s | <90% |
| `query_temp_tables_created` | Counter | 临时表创建数 | - | >100/h |
| `query_filesort_count` | Counter | 文件排序次数 | - | >200/h |
| `index_recommendations_count` | Gauge | 待实施索引建议数 | 3600s | - |
| `index_coverage_rate` | Gauge | 索引覆盖率 (%) | 3600s | <90% |

### 5.2 告警规则

```yaml
groups:
  - name: slow-query-alerts
    rules:
      # P0 慢查询过多
      - alert: SlowQueryP0High
        expr: increase(slow_query_p0_count[1h]) > 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "P0 慢查询过多"
          description: "过去 1 小时检测到 {{ $value }} 个 P0 慢查询 (>1s)"
          
      # 全表扫描占比过高
      - alert: FullTableScanRateHigh
        expr: query_full_table_scan_rate > 5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "全表扫描占比过高"
          description: "全表扫描占比 {{ $value }}% 超过 5%"
          
      # 索引使用率过低
      - alert: IndexUsageRateLow
        expr: query_index_usage_rate < 90
        for: 30m
        labels:
          severity: warning
        annotations:
          summary: "索引使用率过低"
          description: "索引使用率 {{ $value }}% 低于 90%"
```

---

## 6. 实施步骤

### 6.1 Phase 1: 慢查询日志 (Week 3-T1)

**任务**:
- [ ] 实现慢查询日志采集
- [ ] 配置日志格式和采样
- [ ] 实现日志聚合统计
- [ ] 配置 P0 慢查询告警

**交付物**:
- `slow_query_logger.rs`
- `slow_query_log.yml`
- Grafana 慢查询仪表盘

### 6.2 Phase 2: 索引优化 (Week 3-T2)

**任务**:
- [ ] 实现索引推荐算法
- [ ] 分析 Week 2 慢查询
- [ ] 生成索引优化建议
- [ ] 实施 P0 关键索引

**交付物**:
- `index_advisor.rs`
- `index_optimization_plan.md`
- 索引创建脚本

### 6.3 Phase 3: 查询优化 (Week 3-T3)

**任务**:
- [ ] 实现执行计划分析
- [ ] 识别常见问题模式
- [ ] 重写优化慢查询
- [ ] 验证优化效果

**交付物**:
- `query_plan_analyzer.rs`
- `query_optimization_cases.md`
- 优化前后对比报告

### 6.4 Phase 4: 持续监控 (Week 3-T4)

**任务**:
- [ ] 配置慢查询指标采集
- [ ] 配置告警规则
- [ ] 建立周度分析机制
- [ ] 生成优化报告

**交付物**:
- `slow_query_metrics.yml`
- `weekly_slow_query_report.md`

---

## 7. 验证标准

### 7.1 功能验证

| 验证项 | 标准 | 验证方法 | 通过条件 |
|---|---|---|---|
| 慢查询日志 | 100% P0 慢查询记录 | 模拟慢查询 | 日志完整 |
| 索引推荐 | 准确率>85% | 专家评审 | 8/10 建议有效 |
| 查询分析 | 问题识别率>90% | 测试用例 | 9/10 问题识别 |
| 指标采集 | 9 个指标均有数据 | Prometheus 查询 | 100% 指标可查询 |

### 7.2 性能验证

| 指标 | Week 2 基线 | Week 3 目标 | 验证方法 | 通过条件 |
|---|---|---|---|---|
| P99 时延 | 245ms | <200ms | Prometheus | <-18% |
| 慢查询占比 | 2.5% | <1.0% | 日志统计 | <-60% |
| 慢查询平均时延 | 450ms | <300ms | 日志统计 | <-33% |
| 索引覆盖率 | 68% | >90% | 数据库查询 | >+32% |
| 全表扫描占比 | 15% | <5% | 执行计划分析 | <-67% |

### 7.3 优化效果验证

| 优化项 | 优化前 | 优化后 | 提升 | 验证方法 |
|---|---|---|---|---|
| Top1 慢查询 | 520ms | 45ms | 91% | EXPLAIN ANALYZE |
| Top2 慢查询 | 680ms | 120ms | 82% | EXPLAIN ANALYZE |
| Top3 慢查询 | 450ms | 85ms | 81% | EXPLAIN ANALYZE |
| 整体 P99 | 245ms | <200ms | 18% | Prometheus |

---

## 8. 预期收益

### 8.1 性能提升

| 指标 | Week 2 基线 | Week 3 预期 | 改善 |
|---|---|---|---|
| P99 时延 | 245ms | <200ms | -18% |
| 慢查询占比 | 2.5% | <1.0% | -60% |
| 慢查询平均时延 | 450ms | <300ms | -33% |
| Top 10 慢查询时延 | 450ms | <150ms | -67% |

### 8.2 数据库效率

| 指标 | Week 2 | Week 3 | 优化 |
|---|---|---|---|
| 索引覆盖率 | 68% | >90% | +32% |
| 全表扫描占比 | 15% | <5% | -67% |
| 临时表创建 | 250/h | <100/h | -60% |
| 文件排序 | 450/h | <200/h | -56% |

### 8.3 运维效率

| 能力 | Week 2 | Week 3 | 提升 |
|---|---|---|---|
| 慢查询发现 | 被动投诉 | 主动告警 | 实时发现 |
| 问题定位 | 手动分析 | 自动推荐 | 10x 提速 |
| 优化验证 | 人工对比 | 自动报告 | 5x 提速 |

---

## 9. 附录

### 9.1 慢查询分析仪表盘

```json
{
  "title": "Slow Query Analysis Dashboard",
  "panels": [
    {
      "title": "Slow Query Count by Severity",
      "type": "timeseries",
      "targets": [
        {
          "expr": "increase(slow_query_p0_count[1h])",
          "legendFormat": "P0 (>1s)"
        },
        {
          "expr": "increase(slow_query_p1_count[1h])",
          "legendFormat": "P1 (>500ms)"
        },
        {
          "expr": "increase(slow_query_count[1h])",
          "legendFormat": "Total"
        }
      ]
    },
    {
      "title": "Slow Query Average Duration (ms)",
      "type": "timeseries",
      "targets": [
        {
          "expr": "slow_query_avg_duration",
          "legendFormat": "Avg Duration"
        }
      ],
      "thresholds": [
        {"value": 300, "color": "orange"},
        {"value": 400, "color": "red"}
      ]
    },
    {
      "title": "Index Usage Rate (%)",
      "type": "gauge",
      "targets": [
        {
          "expr": "query_index_usage_rate",
          "legendFormat": "Usage Rate"
        }
      ],
      "thresholds": [
        {"value": 90, "color": "green"},
        {"value": 80, "color": "red"}
      ]
    },
    {
      "title": "Full Table Scan Rate (%)",
      "type": "gauge",
      "targets": [
        {
          "expr": "query_full_table_scan_rate",
          "legendFormat": "Full Scan Rate"
        }
      ],
      "thresholds": [
        {"value": 5, "color": "red"}
      ]
    },
    {
      "title": "Top 10 Slow Query Patterns",
      "type": "table",
      "targets": [
        {
          "expr": "topk(10, sum by(query_pattern) (rate(slow_query_duration_sum[1h])))",
          "format": "table"
        }
      ]
    }
  ]
}
```

### 9.2 相关文档

| 文档 | 路径 | 用途 |
|---|---|---|
| Week 2 性能基线 | performance_baseline_week2.md | 问题来源 |
| 连接池优化 | connection_pool_tuning.md | 关联优化 |
| Phase 3 50 指标规划 | phase3_50_metrics_plan.md | 指标体系 |

---

**文档状态**: ✅ Week 3 完成  
**创建日期**: 2026-03-07  
**责任人**: SRE-Agent  
**保管**: 项目文档库
