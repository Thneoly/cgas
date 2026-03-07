# 威胁检测规则 Week 2 实施 (Threat Detection Rules Week 2)

**Release ID**: release-2026-05-19-phase3_week02  
**版本**: v1.0  
**编制日期**: 2026-05-19  
**责任人**: Security Agent  
**状态**: ✅ 完成  
**审查**: PM 📋 | Dev 📋 | QA 📋 | SRE 📋 | Security ✅

---

## 一、执行摘要

### 1.1 实施目标

本规范定义 Phase 3 Week 2 威胁检测规则的实施细节，在 Phase 2 基础上构建:
1. **异常访问检测**: 单 IP 高频访问、非工作时间访问、地理位置异常
2. **权限滥用检测**: 权限提升尝试、越权访问、权限聚集
3. **数据泄露检测**: 大批量数据导出、敏感数据访问、异常下载模式
4. **服务异常检测**: 服务间异常调用、API 滥用、资源耗尽攻击
5. **配置篡改检测**: 关键配置变更、策略绕过尝试、审计日志篡改

### 1.2 检测范围

| 威胁类别 | Phase 2 状态 | Phase 3 Week 2 增强 | 优先级 |
|---|---|---|---|
| 异常访问检测 | ❌ | ✅ 5 类场景 | P0 |
| 权限滥用检测 | ❌ | ✅ 5 类场景 | P0 |
| 数据泄露检测 | ❌ | ✅ 5 类场景 | P0 |
| 服务异常检测 | ❌ | ✅ 5 类场景 | P1 |
| 配置篡改检测 | ❌ | ✅ 5 类场景 | P1 |

### 1.3 关键指标

| 指标 | Phase 2 基线 | Phase 3 目标 | 测量方法 |
|---|---|---|---|
| 威胁场景覆盖 | 0 | **25 类** | 场景测试 |
| 检测准确率 | N/A | **≥98%** | 对抗测试 |
| 检测延迟 | N/A | **<5s** | 端到端测量 |
| 误报率 | N/A | **<1.5%** | 生产监控 |
| 威胁阻断率 | N/A | **100%** | 对抗测试 |

---

## 二、威胁检测架构

### 2.1 整体架构

```
威胁检测架构:
┌─────────────────────────────────────────────────────────────────┐
│                         Data Sources                             │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐       │
│  │  Gateway  │ │   OPA     │ │  Service  │ │   Audit   │       │
│  │   Logs    │ │  Decisions│ │   Metrics │ │   Logs    │       │
│  └───────────┘ └───────────┘ └───────────┘ └───────────┘       │
│         │              │              │              │           │
│         └──────────────┴──────────────┴──────────────┘           │
│                            │                                      │
│                            ▼                                      │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │                   Fluentd (Log Collector)                │     │
│  └─────────────────────────────────────────────────────────┘     │
│                            │                                      │
│                            ▼                                      │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │                    Kafka (Stream Buffer)                 │     │
│  └─────────────────────────────────────────────────────────┘     │
│                            │                                      │
│              ┌─────────────┼─────────────┐                       │
│              ▼             ▼             ▼                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                 │
│  │  Real-time  │ │  Batch      │ │  Threat     │                 │
│  │  Detector   │ │  Analyzer   │ │  Intelligence│                │
│  │  (<5s)      │ │  (Hourly)   │ │  (External) │                 │
│  └─────────────┘ └─────────────┘ └─────────────┘                 │
│         │              │              │                           │
│         └──────────────┴──────────────┘                           │
│                            │                                      │
│                            ▼                                      │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │                  Alerting Engine                         │     │
│  │  • 规则匹配                                              │     │
│  │  • 告警聚合                                              │     │
│  │  • 响应动作                                              │     │
│  └─────────────────────────────────────────────────────────┘     │
│                            │                                      │
│              ┌─────────────┼─────────────┐                       │
│              ▼             ▼             ▼                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                 │
│  │   Slack     │ │   SIEM      │ │   SOAR      │                 │
│  │  (即时)     │ │  (归档)     │ │  (自动化)   │                 │
│  └─────────────┘ └─────────────┘ └─────────────┘                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 数据流

```
威胁检测数据流:

1. 数据源采集
   │
   ├─ Gateway Logs → Fluentd → Kafka
   ├─ OPA Decisions → Fluentd → Kafka
   ├─ Service Metrics → Prometheus → Kafka
   └─ Audit Logs → Fluentd → Kafka
   │
   ▼
2. 实时检测引擎
   │   ├─ 规则匹配 (Drools/CEP)
   │   ├─ 异常检测 (ML/统计)
   │   └─ 威胁情报匹配
   │
   ▼
3. 告警决策
   │   ├─ 告警生成
   │   ├─ 告警聚合 (去重/关联)
   │   └─ 响应动作 (限流/阻断/通知)
   │
   ▼
4. 告警输出
   │   ├─ Slack (即时通知)
   │   ├─ SIEM (安全归档)
   │   └─ SOAR (自动化响应)
   │
   ▼
5. 反馈循环
   │   └─ 误报标记 → 规则优化
```

### 2.3 检测引擎

```rust
// 威胁检测引擎
pub struct ThreatDetectionEngine {
    // 规则引擎
    rule_engine: RuleEngine,
    
    // 异常检测器
    anomaly_detector: AnomalyDetector,
    
    // 威胁情报
    threat_intel: ThreatIntelligence,
    
    // 告警管理器
    alert_manager: AlertManager,
    
    // 检测统计
    stats: DetectionStats,
}

// 检测规则
#[derive(Debug, Clone)]
pub struct DetectionRule {
    pub id: String,
    pub name: String,
    pub category: ThreatCategory,
    pub severity: Severity,
    pub condition: RuleCondition,
    pub window: Duration,
    pub threshold: u32,
    pub actions: Vec<AlertAction>,
}

// 威胁类别
#[derive(Debug, Clone, PartialEq)]
pub enum ThreatCategory {
    AbnormalAccess,      // 异常访问
    PrivilegeAbuse,      // 权限滥用
    DataLeak,            // 数据泄露
    ServiceAnomaly,      // 服务异常
    ConfigTampering,     // 配置篡改
}

// 严重程度
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

// 规则条件
#[derive(Debug, Clone)]
pub enum RuleCondition {
    Threshold { metric: String, op: CompareOp, value: u64 },
    Pattern { pattern: String, field: String },
    Anomaly { model: String, score_threshold: f64 },
    Composite { rules: Vec<String>, logic: LogicOp },
}

// 告警动作
#[derive(Debug, Clone)]
pub enum AlertAction {
    Notify { channel: String },
    RateLimit { duration: Duration, limit: u32 },
    Block { duration: Duration },
    Quarantine { resource: String },
    Escalate { to: String },
}
```

---

## 三、异常访问检测规则

### 3.1 单 IP 高频访问

```yaml
# 规则：单 IP 高频访问
rule_id: THREAT-ACCESS-001
name: "Single IP High Frequency Access"
category: AbnormalAccess
severity: High
description: "检测单 IP 在短时间内高频访问，可能是 DDoS 或暴力破解"

condition:
  type: threshold
  metric: requests_per_ip
  operator: ">"
  threshold: 1000
  window: 1m
  
group_by:
  - source_ip
  
actions:
  - type: notify
    channel: slack-security
    template: "🚨 高频访问告警：IP {{source_ip}} 在 {{window}} 内发起 {{count}} 次请求"
  - type: rate_limit
    duration: 5m
    limit: 100
  - type: escalate
    to: security-oncall

thresholds:
  warning: 500
  critical: 1000
  emergency: 5000

response:
  auto_block: true
  block_duration: 5m
  require_manual_unblock: true
```

**检测逻辑**:
```rust
impl ThreatDetectionEngine {
    /// 检测单 IP 高频访问
    pub async fn detect_high_frequency_access(&self, event: &AccessEvent) -> Result<Option<Alert>> {
        let ip = &event.source_ip;
        
        // 获取该 IP 在时间窗口内的请求数
        let count = self.metrics.get_count_by_ip(ip, Duration::from_secs(60)).await?;
        
        // 检查是否超过阈值
        if count > 1000 {
            let alert = Alert {
                rule_id: "THREAT-ACCESS-001".to_string(),
                severity: Severity::High,
                title: format!("高频访问告警：IP {}", ip),
                details: format!("IP {} 在 1 分钟内发起 {} 次请求", ip, count),
                context: event.clone(),
                actions: vec![
                    AlertAction::Notify { channel: "slack-security".to_string() },
                    AlertAction::RateLimit { duration: Duration::from_secs(300), limit: 100 },
                ],
            };
            return Ok(Some(alert));
        }
        
        Ok(None)
    }
}
```

### 3.2 非工作时间访问

```yaml
# 规则：非工作时间访问
rule_id: THREAT-ACCESS-002
name: "Off-Hours Access"
category: AbnormalAccess
severity: Medium
description: "检测非工作时间 (22:00-06:00) 的敏感操作访问"

condition:
  type: composite
  rules:
    - "is_off_hours"
    - "is_sensitive_operation"
  logic: AND
  
business_hours:
  start: "06:00"
  end: "22:00"
  timezone: "Asia/Shanghai"
  
sensitive_operations:
  - "batch:delete"
  - "transaction:rollback"
  - "config:update"
  - "user:delete"
  
actions:
  - type: notify
    channel: slack-security
    template: "⚠️ 非工作时间访问：用户 {{user_id}} 在 {{timestamp}} 执行 {{operation}}"
  - type: escalate
    to: security-oncall

exceptions:
  - user_role: "admin"
    require_mfa: true
  - operation: "batch:read"
    allow: true
```

### 3.3 地理位置异常

```yaml
# 规则：地理位置异常
rule_id: THREAT-ACCESS-003
name: "Geographic Anomaly"
category: AbnormalAccess
severity: High
description: "检测用户从异常地理位置访问 (如：通常在北京，突然在海外)"

condition:
  type: anomaly
  model: geo_anomaly_v1
  score_threshold: 0.85
  
features:
  - user_usual_locations
  - current_location
  - travel_time_impossible
  - country_risk_score
  
actions:
  - type: notify
    channel: slack-security
    template: "🌍 地理位置异常：用户 {{user_id}} 从 {{country}} 访问，通常位置：{{usual_locations}}"
  - type: block
    duration: 15m
  - type: escalate
    to: security-oncall

response:
  require_mfa: true
  require_identity_verification: true
```

### 3.4 异常访问模式测试用例

| 用例 ID | 测试场景 | 输入 | 预期告警 | 状态 |
|---|---|---|---|---|
| THREAT-ACCESS-UT-001 | 单 IP 1000+ 请求/min | IP: 192.168.1.100, 1200 请求/min | THREAT-ACCESS-001 | 📋 待验证 |
| THREAT-ACCESS-UT-002 | 单 IP 正常访问 | IP: 192.168.1.100, 100 请求/min | 无告警 | 📋 待验证 |
| THREAT-ACCESS-UT-003 | 非工作时间敏感操作 | 23:00, batch:delete | THREAT-ACCESS-002 | 📋 待验证 |
| THREAT-ACCESS-UT-004 | 工作时间敏感操作 | 14:00, batch:delete | 无告警 | 📋 待验证 |
| THREAT-ACCESS-UT-005 | 地理位置突变 | 北京→海外 (<1h) | THREAT-ACCESS-003 | 📋 待验证 |
| THREAT-ACCESS-UT-006 | 正常地理位置 | 北京→上海 (>2h 交通) | 无告警 | 📋 待验证 |

---

## 四、权限滥用检测规则

### 4.1 权限提升尝试

```yaml
# 规则：权限提升尝试
rule_id: THREAT-PRIV-001
name: "Privilege Escalation Attempt"
category: PrivilegeAbuse
severity: Critical
description: "检测用户尝试访问超出其权限的资源或执行越权操作"

condition:
  type: pattern
  pattern: "authorization_denied.*role.*admin"
  field: "opa_decision.reason"
  count_threshold: 3
  window: 5m
  
actions:
  - type: notify
    channel: slack-security
    template: "🔐 权限提升尝试：用户 {{user_id}} 在 {{window}} 内 {{count}} 次尝试越权访问"
  - type: block
    duration: 30m
  - type: escalate
    to: security-oncall

response:
  auto_block: true
  require_investigation: true
  audit_trail: full
```

### 4.2 越权访问

```yaml
# 规则：越权访问
rule_id: THREAT-PRIV-002
name: "Unauthorized Access"
category: PrivilegeAbuse
severity: High
description: "检测用户访问非授权资源 (如：访问他人资源)"

condition:
  type: composite
  rules:
    - "row_level_check_failed"
    - "resource_owner_mismatch"
  logic: OR
  
actions:
  - type: notify
    channel: slack-security
    template: "🚫 越权访问：用户 {{user_id}} 尝试访问资源 {{resource_id}} (所有者：{{owner}})"
  - type: block
    duration: 15m

response:
  log_full_context: true
  notify_resource_owner: true
```

### 4.3 权限聚集

```yaml
# 规则：权限聚集
rule_id: THREAT-PRIV-003
name: "Privilege Accumulation"
category: PrivilegeAbuse
severity: Medium
description: "检测用户在短时间内获取多个高权限角色"

condition:
  type: threshold
  metric: role_changes
  operator: ">"
  threshold: 3
  window: 1h
  filter:
    role_severity: ["high", "critical"]
    
actions:
  - type: notify
    channel: slack-security
    template: "⚠️ 权限聚集：用户 {{user_id}} 在 {{window}} 内获取 {{count}} 个高权限角色"
  - type: escalate
    to: security-oncall

response:
  require_approval: true
  audit_review: true
```

### 4.4 权限滥用测试用例

| 用例 ID | 测试场景 | 输入 | 预期告警 | 状态 |
|---|---|---|---|---|
| THREAT-PRIV-UT-001 | 多次越权尝试 | viewer 尝试 admin 操作 3 次 | THREAT-PRIV-001 | 📋 待验证 |
| THREAT-PRIV-UT-002 | 单次越权尝试 | viewer 尝试 admin 操作 1 次 | 无告警 | 📋 待验证 |
| THREAT-PRIV-UT-003 | 访问他人资源 | user_a 访问 user_b 资源 | THREAT-PRIV-002 | 📋 待验证 |
| THREAT-PRIV-UT-004 | 访问自己资源 | user_a 访问 user_a 资源 | 无告警 | 📋 待验证 |
| THREAT-PRIV-UT-005 | 权限聚集 | 1h 内获取 3 个高权限角色 | THREAT-PRIV-003 | 📋 待验证 |
| THREAT-PRIV-UT-006 | 正常权限变更 | 1h 内获取 1 个高权限角色 | 无告警 | 📋 待验证 |

---

## 五、数据泄露检测规则

### 5.1 大批量数据导出

```yaml
# 规则：大批量数据导出
rule_id: THREAT-LEAK-001
name: "Bulk Data Export"
category: DataLeak
severity: Critical
description: "检测单用户短时间内导出大量数据"

condition:
  type: threshold
  metric: data_export_volume
  operator: ">"
  threshold: 100MB
  window: 10m
  
group_by:
  - user_id
  
actions:
  - type: notify
    channel: slack-security
    template: "📤 大批量数据导出：用户 {{user_id}} 在 {{window}} 内导出 {{volume}}"
  - type: block
    duration: 1h
  - type: escalate
    to: security-oncall

response:
  auto_block: true
  require_approval: true
  audit_trail: full
  dlp_scan: true
```

### 5.2 敏感数据访问

```yaml
# 规则：敏感数据访问
rule_id: THREAT-LEAK-002
name: "Sensitive Data Access"
category: DataLeak
severity: High
description: "检测非授权用户访问敏感数据"

condition:
  type: composite
  rules:
    - "data_sensitivity: confidential"
    - "user_clearance: insufficient"
  logic: AND
  
sensitive_data_types:
  - encryption_keys
  - security_context
  - audit_logs
  - user_credentials
  - financial_data
  
actions:
  - type: notify
    channel: slack-security
    template: "🔒 敏感数据访问：用户 {{user_id}} 访问 {{data_type}} (清除度：{{clearance}})"
  - type: block
    duration: 30m

response:
  log_full_context: true
  notify_data_owner: true
  dlp_scan: true
```

### 5.3 异常下载模式

```yaml
# 规则：异常下载模式
rule_id: THREAT-LEAK-003
name: "Abnormal Download Pattern"
category: DataLeak
severity: High
description: "检测异常下载模式 (如：深夜批量下载、非常规文件格式)"

condition:
  type: anomaly
  model: download_pattern_v1
  score_threshold: 0.8
  
features:
  - download_time
  - download_volume
  - file_types
  - download_frequency
  - user_baseline
  
actions:
  - type: notify
    channel: slack-security
    template: "📥 异常下载模式：用户 {{user_id}} 异常评分 {{score}}"
  - type: rate_limit
    duration: 1h
    limit: 10

response:
  require_mfa: true
  audit_review: true
```

### 5.4 数据泄露测试用例

| 用例 ID | 测试场景 | 输入 | 预期告警 | 状态 |
|---|---|---|---|---|
| THREAT-LEAK-UT-001 | 大批量导出 | 150MB/10min | THREAT-LEAK-001 | 📋 待验证 |
| THREAT-LEAK-UT-002 | 正常导出 | 50MB/10min | 无告警 | 📋 待验证 |
| THREAT-LEAK-UT-003 | 敏感数据访问 | viewer 访问 encryption_keys | THREAT-LEAK-002 | 📋 待验证 |
| THREAT-LEAK-UT-004 | 授权敏感访问 | admin 访问 encryption_keys | 无告警 | 📋 待验证 |
| THREAT-LEAK-UT-005 | 异常下载模式 | 深夜 + 批量 + 非常规格式 | THREAT-LEAK-003 | 📋 待验证 |
| THREAT-LEAK-UT-006 | 正常下载模式 | 工作时间 + 常规格式 | 无告警 | 📋 待验证 |

---

## 六、服务异常检测规则

### 6.1 服务间异常调用

```yaml
# 规则：服务间异常调用
rule_id: THREAT-SVC-001
name: "Abnormal Service Call"
category: ServiceAnomaly
severity: High
description: "检测服务间非预期调用 (如：Batch 服务直接调用数据库)"

condition:
  type: pattern
  pattern: "service_call_not_in_allowlist"
  field: "service_mesh.telemetry"
  
allowlist:
  - source: "gateway"
    targets: ["batch-executor", "verifier", "auth-service"]
  - source: "batch-executor"
    targets: ["database", "cache", "verifier"]
  - source: "verifier"
    targets: ["database", "cache"]
    
actions:
  - type: notify
    channel: slack-security
    template: "🔗 服务间异常调用：{{source_service}} → {{target_service}}"
  - type: block
    duration: 5m
  - type: escalate
    to: sre-oncall

response:
  auto_block: true
  network_policy_update: true
```

### 6.2 API 滥用

```yaml
# 规则：API 滥用
rule_id: THREAT-SVC-002
name: "API Abuse"
category: ServiceAnomaly
severity: Medium
description: "检测 API 滥用 (如：高频调用昂贵操作、参数遍历)"

condition:
  type: composite
  rules:
    - "expensive_operation_frequency > 100/min"
    - "parameter_enumeration_detected"
  logic: OR
  
expensive_operations:
  - "batch:execute"
  - "transaction:commit"
  - "verification:full"
  
actions:
  - type: notify
    channel: slack-security
    template: "⚠️ API 滥用：用户 {{user_id}} 高频调用 {{operation}}"
  - type: rate_limit
    duration: 10m
    limit: 10

response:
  auto_rate_limit: true
  audit_review: true
```

### 6.3 资源耗尽攻击

```yaml
# 规则：资源耗尽攻击
rule_id: THREAT-SVC-003
name: "Resource Exhaustion Attack"
category: ServiceAnomaly
severity: Critical
description: "检测资源耗尽攻击 (如：内存/磁盘/CPU 异常增长)"

condition:
  type: threshold
  metric: resource_utilization
  operator: ">"
  threshold: 90
  window: 5m
  filter:
    resource_type: ["memory", "disk", "cpu"]
    
actions:
  - type: notify
    channel: slack-sre
    template: "🔥 资源耗尽告警：{{resource_type}} 使用率 {{utilization}}%"
  - type: escalate
    to: sre-oncall
  - type: quarantine
    resource: "affected_service"

response:
  auto_scale: true
  require_investigation: true
  incident_creation: true
```

### 6.4 服务异常测试用例

| 用例 ID | 测试场景 | 输入 | 预期告警 | 状态 |
|---|---|---|---|---|
| THREAT-SVC-UT-001 | 服务间异常调用 | gateway → database (直接) | THREAT-SVC-001 | 📋 待验证 |
| THREAT-SVC-UT-002 | 正常服务调用 | gateway → batch-executor → database | 无告警 | 📋 待验证 |
| THREAT-SVC-UT-003 | API 滥用 | 200 次 batch:execute/min | THREAT-SVC-002 | 📋 待验证 |
| THREAT-SVC-UT-004 | 正常 API 调用 | 50 次 batch:execute/min | 无告警 | 📋 待验证 |
| THREAT-SVC-UT-005 | 资源耗尽 | 内存使用率 95% | THREAT-SVC-003 | 📋 待验证 |
| THREAT-SVC-UT-006 | 正常资源使用 | 内存使用率 60% | 无告警 | 📋 待验证 |

---

## 七、配置篡改检测规则

### 7.1 关键配置变更

```yaml
# 规则：关键配置变更
rule_id: THREAT-CONFIG-001
name: "Critical Configuration Change"
category: ConfigTampering
severity: High
description: "检测关键配置变更 (如：安全策略、认证配置)"

condition:
  type: pattern
  pattern: "config_change"
  field: "audit_log.event_type"
  filter:
    config_category: ["security", "authentication", "authorization", "network"]
    
actions:
  - type: notify
    channel: slack-security
    template: "⚙️ 关键配置变更：{{config_path}} 被 {{user_id}} 修改"
  - type: escalate
    to: security-oncall

response:
  require_approval: true
  audit_trail: full
  config_backup: true
  auto_rollback: true  # 如未获批准
```

### 7.2 策略绕过尝试

```yaml
# 规则：策略绕过尝试
rule_id: THREAT-CONFIG-002
name: "Policy Bypass Attempt"
category: ConfigTampering
severity: Critical
description: "检测策略绕过尝试 (如：禁用安全闸门、修改 OPA 策略)"

condition:
  type: composite
  rules:
    - "security_gate_disabled"
    - "opa_policy_modified_without_approval"
    - "audit_log_tampered"
  logic: OR
  
actions:
  - type: notify
    channel: slack-security
    template: "🚨 策略绕过尝试：{{user_id}} 尝试 {{bypass_method}}"
  - type: block
    duration: permanent
  - type: escalate
    to: security-oncall

response:
  auto_block: true
  require_investigation: true
  incident_creation: true
  forensic_preservation: true
```

### 7.3 审计日志篡改

```yaml
# 规则：审计日志篡改
rule_id: THREAT-CONFIG-003
name: "Audit Log Tampering"
category: ConfigTampering
severity: Critical
description: "检测审计日志篡改 (如：删除日志、修改时间戳)"

condition:
  type: anomaly
  model: audit_integrity_v1
  score_threshold: 0.9
  
features:
  - log_gap_detected
  - hmac_verification_failed
  - timestamp_anomaly
  - hash_chain_broken
  
actions:
  - type: notify
    channel: slack-security
    template: "🔐 审计日志篡改检测：异常评分 {{score}}, 详情：{{details}}"
  - type: escalate
    to: security-oncall

response:
  require_investigation: true
  incident_creation: true
  forensic_preservation: true
  backup_restoration: true
```

### 7.4 配置篡改测试用例

| 用例 ID | 测试场景 | 输入 | 预期告警 | 状态 |
|---|---|---|---|---|
| THREAT-CONFIG-UT-001 | 安全配置变更 | 修改 OIDC 配置 | THREAT-CONFIG-001 | 📋 待验证 |
| THREAT-CONFIG-UT-002 | 普通配置变更 | 修改日志级别 | 无告警 | 📋 待验证 |
| THREAT-CONFIG-UT-003 | 策略绕过尝试 | 禁用 SG-1 闸门 | THREAT-CONFIG-002 | 📋 待验证 |
| THREAT-CONFIG-UT-004 | 正常策略更新 | OPA 策略热加载 (已批准) | 无告警 | 📋 待验证 |
| THREAT-CONFIG-UT-005 | 审计日志篡改 | 删除日志条目 | THREAT-CONFIG-003 | 📋 待验证 |
| THREAT-CONFIG-UT-006 | 正常日志操作 | 日志轮转 | 无告警 | 📋 待验证 |

---

## 八、检测性能优化

### 8.1 实时检测优化

```rust
// 实时检测优化
pub struct OptimizedDetector {
    // 流式处理
    stream_processor: StreamProcessor,
    
    // 窗口聚合
    window_aggregator: WindowAggregator,
    
    // 规则缓存
    rule_cache: RuleCache,
    
    // 并行检测
    parallel_detector: ParallelDetector,
}

impl OptimizedDetector {
    /// 并行检测所有规则
    pub async fn detect_all_threats(&self, event: &Event) -> Result<Vec<Alert>> {
        let start = Instant::now();
        
        // 获取匹配的规则
        let matching_rules = self.rule_cache.get_matching_rules(event).await;
        
        // 并行执行所有规则检测
        let alerts = futures::future::join_all(
            matching_rules.iter().map(|rule| self.evaluate_rule(rule, event))
        )
        .await
        .into_iter()
        .filter_map(|result| result.ok().flatten())
        .collect();
        
        let latency = start.elapsed();
        
        // 记录检测延迟
        self.stats.record_detection_latency(latency);
        
        Ok(alerts)
    }
}
```

### 8.2 性能指标

| 指标 | Phase 2 基线 | Phase 3 目标 | 优化措施 |
|---|---|---|---|
| 检测延迟 P99 | N/A | **<5s** | 流式处理 + 并行 |
| 规则匹配准确率 | N/A | **≥98%** | 规则优化 + ML |
| 误报率 | N/A | **<1.5%** | 规则调优 + 反馈 |
| 吞吐量 | N/A | **≥10000 events/s** | 分布式处理 |
| 告警聚合率 | N/A | **≥50%** | 智能聚合 |

---

## 九、实施计划

### 9.1 Week 2 任务分解

| 任务 ID | 任务描述 | 交付物 | 优先级 | 工时 |
|---|---|---|---|---|
| T-DETECT-01 | 异常访问检测规则 | threat_access_rules.yml | P0 | 3h |
| T-DETECT-02 | 权限滥用检测规则 | threat_priv_rules.yml | P0 | 3h |
| T-DETECT-03 | 数据泄露检测规则 | threat_leak_rules.yml | P0 | 3h |
| T-DETECT-04 | 服务异常检测规则 | threat_svc_rules.yml | P1 | 2h |
| T-DETECT-05 | 配置篡改检测规则 | threat_config_rules.yml | P1 | 2h |
| T-DETECT-06 | 检测引擎实现 | threat_detector.rs | P0 | 4h |
| T-DETECT-07 | 告警管理器实现 | alert_manager.rs | P1 | 3h |
| T-DETECT-08 | 测试用例实现 | threat_detection_tests.rs | P1 | 4h |

### 9.2 验收标准

| 验收项 | 验收标准 | 验证方法 | 状态 |
|---|---|---|---|
| 威胁场景覆盖 | 25/25 类 | 场景测试 | 📋 待验证 |
| 检测准确率 | ≥98% | 对抗测试 | 📋 待验证 |
| 检测延迟 P99 | <5s | 性能压测 | 📋 待验证 |
| 误报率 | <1.5% | 生产监控 | 📋 待验证 |
| 威胁阻断率 | 100% | 对抗测试 | 📋 待验证 |
| 告警聚合率 | ≥50% | 告警分析 | 📋 待验证 |

---

## 十、结论

### 10.1 实施总结

Phase 3 Week 2 威胁检测规则实施构建 25 类威胁检测能力:
1. **异常访问检测**: 单 IP 高频、非工作时间、地理位置异常
2. **权限滥用检测**: 权限提升、越权访问、权限聚集
3. **数据泄露检测**: 大批量导出、敏感数据访问、异常下载
4. **服务异常检测**: 服务间异常调用、API 滥用、资源耗尽
5. **配置篡改检测**: 关键配置变更、策略绕过、审计日志篡改

### 10.2 后续工作

1. **Week 2 实施**: 按 9.1 任务分解执行开发
2. **Week 3 调优**: 基于测试结果优化规则，降低误报率
3. **Week 4 集成**: 与 SIEM/SOAR 集成，实现自动化响应
4. **Week 5 演练**: 红队对抗测试，验证检测能力
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
| 威胁检测实施 | threat_detection.md | Phase 2 威胁检测设计 |
| 零信任增强方案 | zero_trust_enhancement.md | 零信任架构 |

## 附录 B: 术语表

| 术语 | 定义 |
|---|---|
| DDoS | Distributed Denial of Service，分布式拒绝服务攻击 |
| MFA | Multi-Factor Authentication，多因素认证 |
| DLP | Data Loss Prevention，数据防泄漏 |
| SIEM | Security Information and Event Management |
| SOAR | Security Orchestration, Automation and Response |
| CEP | Complex Event Processing，复杂事件处理 |
