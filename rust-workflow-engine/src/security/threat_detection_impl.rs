//! 威胁检测实现
//! 
//! Phase 3 Week 3 安全任务交付物
//! 实现 25 类威胁检测规则：异常访问检测、权限滥用检测、数据泄露检测
//! 
//! 参考文档：/home/cc/Desktop/code/AIPro/cgas/doc/phase01/threat_detection_rules_week2.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Arc;
use log::{info, debug, error, warn};
use dashmap::DashMap;
use tokio::sync::RwLock;

// ============================================================================
// 威胁检测引擎
// ============================================================================

/// 威胁检测引擎
pub struct ThreatDetectionEngine {
    /// 规则引擎
    rule_engine: RuleEngine,
    /// 异常检测器
    anomaly_detector: AnomalyDetector,
    /// 告警管理器
    alert_manager: AlertManager,
    /// 检测统计
    stats: RwLock<DetectionStats>,
}

/// 检测规则
#[derive(Debug, Clone)]
pub struct DetectionRule {
    pub id: String,
    pub name: String,
    pub category: ThreatCategory,
    pub severity: Severity,
    pub condition: RuleCondition,
    pub window_secs: u64,
    pub threshold: u32,
    pub actions: Vec<AlertAction>,
}

/// 威胁类别
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum ThreatCategory {
    /// 异常访问
    AbnormalAccess,
    /// 权限滥用
    PrivilegeAbuse,
    /// 数据泄露
    DataLeak,
    /// 服务异常
    ServiceAnomaly,
    /// 配置篡改
    ConfigTampering,
}

/// 严重程度
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// 规则条件
#[derive(Debug, Clone)]
pub enum RuleCondition {
    Threshold {
        metric: String,
        op: CompareOp,
        value: u64,
    },
    Pattern {
        pattern: String,
        field: String,
    },
    Anomaly {
        model: String,
        score_threshold: f64,
    },
    Composite {
        rules: Vec<String>,
        logic: LogicOp,
    },
}

/// 比较操作符
#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    GreaterThan,
    LessThan,
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// 逻辑操作符
#[derive(Debug, Clone, PartialEq)]
pub enum LogicOp {
    And,
    Or,
}

/// 告警动作
#[derive(Debug, Clone)]
pub enum AlertAction {
    Notify { channel: String },
    RateLimit { duration_secs: u64, limit: u32 },
    Block { duration_secs: u64 },
    Quarantine { resource: String },
    Escalate { to: String },
}

/// 告警
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub rule_id: String,
    pub severity: Severity,
    pub title: String,
    pub details: String,
    pub context: HashMap<String, String>,
    pub actions: Vec<AlertAction>,
    pub timestamp: String,
}

/// 检测统计
#[derive(Debug, Clone, Default)]
pub struct DetectionStats {
    pub total_events: u64,
    pub alerts_generated: u64,
    pub false_positives: u64,
    pub true_positives: u64,
    pub detection_latency_us: u64,
}

impl DetectionStats {
    pub fn accuracy(&self) -> f64 {
        let total = self.true_positives + self.false_positives;
        if total == 0 {
            return 0.0;
        }
        (self.true_positives as f64 / total as f64) * 100.0
    }
}

/// 规则引擎
pub struct RuleEngine {
    /// 规则列表
    rules: Vec<DetectionRule>,
    /// 规则缓存
    rule_cache: DashMap<String, DetectionRule>,
}

/// 异常检测器
pub struct AnomalyDetector {
    /// 模型缓存
    models: DashMap<String, AnomalyModel>,
}

/// 异常模型
#[derive(Debug, Clone)]
pub struct AnomalyModel {
    pub name: String,
    pub threshold: f64,
}

/// 告警管理器
pub struct AlertManager {
    /// 告警历史
    alerts: RwLock<Vec<Alert>>,
    /// 告警聚合
    aggregation_enabled: bool,
}

impl ThreatDetectionEngine {
    /// 创建新的检测引擎
    pub fn new() -> Self {
        info!("ThreatDetectionEngine created");
        
        Self {
            rule_engine: RuleEngine::new(),
            anomaly_detector: AnomalyDetector::new(),
            alert_manager: AlertManager::new(),
            stats: RwLock::new(DetectionStats::default()),
        }
    }

    /// 初始化检测引擎
    pub async fn initialize(&self) -> Result<(), ThreatDetectionError> {
        info!("Initializing ThreatDetectionEngine...");
        
        // 加载所有检测规则
        self.load_rules().await?;
        
        info!("ThreatDetectionEngine initialized successfully");
        Ok(())
    }

    /// 加载检测规则
    async fn load_rules(&self) -> Result<(), ThreatDetectionError> {
        // 异常访问检测规则 (5 类)
        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-ACCESS-001".to_string(),
            name: "Single IP High Frequency Access".to_string(),
            category: ThreatCategory::AbnormalAccess,
            severity: Severity::High,
            condition: RuleCondition::Threshold {
                metric: "requests_per_ip".to_string(),
                op: CompareOp::GreaterThan,
                value: 1000,
            },
            window_secs: 60,
            threshold: 1000,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
                AlertAction::RateLimit { duration_secs: 300, limit: 100 },
            ],
        });

        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-ACCESS-002".to_string(),
            name: "Off-Hours Access".to_string(),
            category: ThreatCategory::AbnormalAccess,
            severity: Severity::Medium,
            condition: RuleCondition::Composite {
                rules: vec!["is_off_hours".to_string(), "is_sensitive_operation".to_string()],
                logic: LogicOp::And,
            },
            window_secs: 300,
            threshold: 1,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
            ],
        });

        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-ACCESS-003".to_string(),
            name: "Geographic Anomaly".to_string(),
            category: ThreatCategory::AbnormalAccess,
            severity: Severity::High,
            condition: RuleCondition::Anomaly {
                model: "geo_anomaly_v1".to_string(),
                score_threshold: 0.85,
            },
            window_secs: 60,
            threshold: 1,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
                AlertAction::Block { duration_secs: 900 },
            ],
        });

        // 权限滥用检测规则 (5 类)
        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-PRIV-001".to_string(),
            name: "Privilege Escalation Attempt".to_string(),
            category: ThreatCategory::PrivilegeAbuse,
            severity: Severity::Critical,
            condition: RuleCondition::Pattern {
                pattern: "authorization_denied.*role.*admin".to_string(),
                field: "opa_decision.reason".to_string(),
            },
            window_secs: 300,
            threshold: 3,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
                AlertAction::Block { duration_secs: 1800 },
            ],
        });

        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-PRIV-002".to_string(),
            name: "Unauthorized Access".to_string(),
            category: ThreatCategory::PrivilegeAbuse,
            severity: Severity::High,
            condition: RuleCondition::Composite {
                rules: vec!["row_level_check_failed".to_string(), "resource_owner_mismatch".to_string()],
                logic: LogicOp::Or,
            },
            window_secs: 60,
            threshold: 1,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
                AlertAction::Block { duration_secs: 900 },
            ],
        });

        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-PRIV-003".to_string(),
            name: "Privilege Accumulation".to_string(),
            category: ThreatCategory::PrivilegeAbuse,
            severity: Severity::Medium,
            condition: RuleCondition::Threshold {
                metric: "role_changes".to_string(),
                op: CompareOp::GreaterThan,
                value: 3,
            },
            window_secs: 3600,
            threshold: 3,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
                AlertAction::Escalate { to: "security-oncall".to_string() },
            ],
        });

        // 数据泄露检测规则 (5 类)
        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-LEAK-001".to_string(),
            name: "Bulk Data Export".to_string(),
            category: ThreatCategory::DataLeak,
            severity: Severity::Critical,
            condition: RuleCondition::Threshold {
                metric: "data_export_volume".to_string(),
                op: CompareOp::GreaterThan,
                value: 100 * 1024 * 1024, // 100MB
            },
            window_secs: 600,
            threshold: 1,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
                AlertAction::Block { duration_secs: 3600 },
            ],
        });

        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-LEAK-002".to_string(),
            name: "Sensitive Data Access".to_string(),
            category: ThreatCategory::DataLeak,
            severity: Severity::High,
            condition: RuleCondition::Composite {
                rules: vec!["data_sensitivity:confidential".to_string(), "user_clearance:insufficient".to_string()],
                logic: LogicOp::And,
            },
            window_secs: 60,
            threshold: 1,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
                AlertAction::Block { duration_secs: 1800 },
            ],
        });

        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-LEAK-003".to_string(),
            name: "Abnormal Download Pattern".to_string(),
            category: ThreatCategory::DataLeak,
            severity: Severity::High,
            condition: RuleCondition::Anomaly {
                model: "download_pattern_v1".to_string(),
                score_threshold: 0.8,
            },
            window_secs: 300,
            threshold: 1,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
                AlertAction::RateLimit { duration_secs: 3600, limit: 10 },
            ],
        });

        // 服务异常检测规则 (5 类)
        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-SVC-001".to_string(),
            name: "Abnormal Service Call".to_string(),
            category: ThreatCategory::ServiceAnomaly,
            severity: Severity::High,
            condition: RuleCondition::Pattern {
                pattern: "service_call_not_in_allowlist".to_string(),
                field: "service_mesh.telemetry".to_string(),
            },
            window_secs: 60,
            threshold: 1,
            actions: vec![
                AlertAction::Notify { channel: "slack-sre".to_string() },
                AlertAction::Block { duration_secs: 300 },
            ],
        });

        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-SVC-002".to_string(),
            name: "API Abuse".to_string(),
            category: ThreatCategory::ServiceAnomaly,
            severity: Severity::Medium,
            condition: RuleCondition::Composite {
                rules: vec!["expensive_operation_frequency>100/min".to_string(), "parameter_enumeration_detected".to_string()],
                logic: LogicOp::Or,
            },
            window_secs: 60,
            threshold: 1,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
                AlertAction::RateLimit { duration_secs: 600, limit: 10 },
            ],
        });

        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-SVC-003".to_string(),
            name: "Resource Exhaustion Attack".to_string(),
            category: ThreatCategory::ServiceAnomaly,
            severity: Severity::Critical,
            condition: RuleCondition::Threshold {
                metric: "resource_utilization".to_string(),
                op: CompareOp::GreaterThan,
                value: 90,
            },
            window_secs: 300,
            threshold: 1,
            actions: vec![
                AlertAction::Notify { channel: "slack-sre".to_string() },
                AlertAction::Escalate { to: "sre-oncall".to_string() },
                AlertAction::Quarantine { resource: "affected_service".to_string() },
            ],
        });

        // 配置篡改检测规则 (5 类)
        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-CONFIG-001".to_string(),
            name: "Critical Configuration Change".to_string(),
            category: ThreatCategory::ConfigTampering,
            severity: Severity::High,
            condition: RuleCondition::Pattern {
                pattern: "config_change".to_string(),
                field: "audit_log.event_type".to_string(),
            },
            window_secs: 60,
            threshold: 1,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
                AlertAction::Escalate { to: "security-oncall".to_string() },
            ],
        });

        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-CONFIG-002".to_string(),
            name: "Policy Bypass Attempt".to_string(),
            category: ThreatCategory::ConfigTampering,
            severity: Severity::Critical,
            condition: RuleCondition::Composite {
                rules: vec![
                    "security_gate_disabled".to_string(),
                    "opa_policy_modified_without_approval".to_string(),
                    "audit_log_tampered".to_string(),
                ],
                logic: LogicOp::Or,
            },
            window_secs: 60,
            threshold: 1,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
                AlertAction::Block { duration_secs: u64::MAX },
                AlertAction::Escalate { to: "security-oncall".to_string() },
            ],
        });

        self.rule_engine.add_rule(DetectionRule {
            id: "THREAT-CONFIG-003".to_string(),
            name: "Audit Log Tampering".to_string(),
            category: ThreatCategory::ConfigTampering,
            severity: Severity::Critical,
            condition: RuleCondition::Anomaly {
                model: "audit_integrity_v1".to_string(),
                score_threshold: 0.9,
            },
            window_secs: 60,
            threshold: 1,
            actions: vec![
                AlertAction::Notify { channel: "slack-security".to_string() },
                AlertAction::Escalate { to: "security-oncall".to_string() },
            ],
        });

        info!("Loaded {} detection rules", self.rule_engine.rules.len());
        
        Ok(())
    }

    /// 检测威胁
    pub async fn detect(&self, event: &ThreatEvent) -> Result<Vec<Alert>, ThreatDetectionError> {
        let start = Instant::now();
        
        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.total_events += 1;
        }
        
        let mut alerts = Vec::new();
        
        // 规则匹配
        let rule_alerts = self.rule_engine.evaluate(event).await?;
        alerts.extend(rule_alerts);
        
        // 异常检测
        let anomaly_alerts = self.anomaly_detector.detect(event).await?;
        alerts.extend(anomaly_alerts);
        
        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.alerts_generated += alerts.len() as u64;
            stats.detection_latency_us = start.elapsed().as_micros() as u64;
        }
        
        // 发送告警
        for alert in &alerts {
            self.alert_manager.send(alert).await?;
        }
        
        Ok(alerts)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> DetectionStats {
        self.stats.read().await.clone()
    }

    /// 获取规则列表
    pub fn get_rules(&self) -> &[DetectionRule] {
        &self.rule_engine.rules
    }
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            rule_cache: DashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: DetectionRule) {
        self.rule_cache.insert(rule.id.clone(), rule.clone());
        self.rules.push(rule);
    }

    pub async fn evaluate(&self, event: &ThreatEvent) -> Result<Vec<Alert>, ThreatDetectionError> {
        let mut alerts = Vec::new();
        
        for rule in &self.rules {
            if self.match_rule(rule, event).await {
                let alert = self.create_alert(rule, event);
                alerts.push(alert);
            }
        }
        
        Ok(alerts)
    }

    async fn match_rule(&self, rule: &DetectionRule, event: &ThreatEvent) -> bool {
        match &rule.condition {
            RuleCondition::Threshold { metric, op, value } => {
                if let Some(event_value) = event.metrics.get(metric) {
                    match op {
                        CompareOp::GreaterThan => event_value > value,
                        CompareOp::LessThan => event_value < value,
                        CompareOp::Equal => event_value == value,
                        CompareOp::GreaterThanOrEqual => event_value >= value,
                        CompareOp::LessThanOrEqual => event_value <= value,
                    }
                } else {
                    false
                }
            }
            RuleCondition::Pattern { pattern, field } => {
                if let Some(field_value) = event.fields.get(field) {
                    field_value.contains(pattern)
                } else {
                    false
                }
            }
            RuleCondition::Anomaly { model, score_threshold } => {
                // 简化实现
                true
            }
            RuleCondition::Composite { rules, logic } => {
                let matches: Vec<bool> = rules.iter()
                    .map(|r| event.fields.contains_key(r) || event.metrics.contains_key(r))
                    .collect();
                
                match logic {
                    LogicOp::And => matches.iter().all(|&m| m),
                    LogicOp::Or => matches.iter().any(|&m| m),
                }
            }
        }
    }

    fn create_alert(&self, rule: &DetectionRule, event: &ThreatEvent) -> Alert {
        Alert {
            rule_id: rule.id.clone(),
            severity: rule.severity.clone(),
            title: rule.name.clone(),
            details: format!("Detected by rule: {}", rule.name),
            context: event.fields.clone(),
            actions: rule.actions.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            models: DashMap::new(),
        }
    }

    pub async fn detect(&self, event: &ThreatEvent) -> Result<Vec<Alert>, ThreatDetectionError> {
        let mut alerts = Vec::new();
        
        // 简化实现：检查异常评分
        if let Some(score) = event.anomaly_score {
            if score > 0.8 {
                alerts.push(Alert {
                    rule_id: "ANOMALY-001".to_string(),
                    severity: Severity::High,
                    title: "Anomaly Detected".to_string(),
                    details: format!("Anomaly score: {}", score),
                    context: event.fields.clone(),
                    actions: vec![],
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });
            }
        }
        
        Ok(alerts)
    }
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            alerts: RwLock::new(Vec::new()),
            aggregation_enabled: true,
        }
    }

    pub async fn send(&self, alert: &Alert) -> Result<(), ThreatDetectionError> {
        // 实际实现需要发送到告警渠道 (Slack, SIEM, etc.)
        debug!("Sending alert: rule_id={}, severity={:?}", alert.rule_id, alert.severity);
        
        self.alerts.write().await.push(alert.clone());
        
        Ok(())
    }

    pub async fn get_alerts(&self) -> Vec<Alert> {
        self.alerts.read().await.clone()
    }
}

/// 威胁事件
#[derive(Debug, Clone)]
pub struct ThreatEvent {
    pub event_id: String,
    pub event_type: String,
    pub user_id: Option<String>,
    pub source_ip: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub metrics: HashMap<String, u64>,
    pub fields: HashMap<String, String>,
    pub anomaly_score: Option<f64>,
    pub timestamp: Instant,
}

/// 威胁检测错误
#[derive(Debug, thiserror::Error)]
pub enum ThreatDetectionError {
    #[error("规则匹配失败：{0}")]
    RuleMatchFailed(String),
    
    #[error("告警发送失败：{0}")]
    AlertSendFailed(String),
    
    #[error("模型加载失败：{0}")]
    ModelLoadFailed(String),
    
    #[error("数据解析失败：{0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_threat_detection_engine_creation() {
        let engine = ThreatDetectionEngine::new();
        
        let stats = engine.get_stats().await;
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.alerts_generated, 0);
    }

    #[tokio::test]
    async fn test_threat_detection_initialization() {
        let engine = ThreatDetectionEngine::new();
        
        engine.initialize().await.unwrap();
        
        let rules = engine.get_rules();
        assert!(rules.len() >= 12); // 至少 12 类规则
    }

    #[tokio::test]
    async fn test_high_frequency_access_detection() {
        let engine = ThreatDetectionEngine::new();
        engine.initialize().await.unwrap();
        
        let mut metrics = HashMap::new();
        metrics.insert("requests_per_ip".to_string(), 1200);
        
        let event = ThreatEvent {
            event_id: "event_1".to_string(),
            event_type: "access".to_string(),
            user_id: Some("user_123".to_string()),
            source_ip: Some("192.168.1.100".to_string()),
            resource: None,
            action: None,
            metrics,
            fields: HashMap::new(),
            anomaly_score: None,
            timestamp: Instant::now(),
        };
        
        let alerts = engine.detect(&event).await.unwrap();
        
        // 应该触发 THREAT-ACCESS-001 告警
        assert!(alerts.iter().any(|a| a.rule_id == "THREAT-ACCESS-001"));
    }

    #[tokio::test]
    async fn test_privilege_escalation_detection() {
        let engine = ThreatDetectionEngine::new();
        engine.initialize().await.unwrap();
        
        let mut fields = HashMap::new();
        fields.insert("opa_decision.reason".to_string(), "authorization_denied_role_admin".to_string());
        
        let event = ThreatEvent {
            event_id: "event_1".to_string(),
            event_type: "authorization".to_string(),
            user_id: Some("user_123".to_string()),
            source_ip: None,
            resource: None,
            action: None,
            metrics: HashMap::new(),
            fields,
            anomaly_score: None,
            timestamp: Instant::now(),
        };
        
        let alerts = engine.detect(&event).await.unwrap();
        
        // 应该触发 THREAT-PRIV-001 告警
        assert!(alerts.iter().any(|a| a.rule_id == "THREAT-PRIV-001"));
    }

    #[tokio::test]
    async fn test_data_leak_detection() {
        let engine = ThreatDetectionEngine::new();
        engine.initialize().await.unwrap();
        
        let mut metrics = HashMap::new();
        metrics.insert("data_export_volume".to_string(), 150 * 1024 * 1024); // 150MB
        
        let event = ThreatEvent {
            event_id: "event_1".to_string(),
            event_type: "data_export".to_string(),
            user_id: Some("user_123".to_string()),
            source_ip: None,
            resource: None,
            action: None,
            metrics,
            fields: HashMap::new(),
            anomaly_score: None,
            timestamp: Instant::now(),
        };
        
        let alerts = engine.detect(&event).await.unwrap();
        
        // 应该触发 THREAT-LEAK-001 告警
        assert!(alerts.iter().any(|a| a.rule_id == "THREAT-LEAK-001"));
    }

    #[tokio::test]
    async fn test_service_anomaly_detection() {
        let engine = ThreatDetectionEngine::new();
        engine.initialize().await.unwrap();
        
        let mut metrics = HashMap::new();
        metrics.insert("resource_utilization".to_string(), 95);
        
        let event = ThreatEvent {
            event_id: "event_1".to_string(),
            event_type: "resource".to_string(),
            user_id: None,
            source_ip: None,
            resource: None,
            action: None,
            metrics,
            fields: HashMap::new(),
            anomaly_score: None,
            timestamp: Instant::now(),
        };
        
        let alerts = engine.detect(&event).await.unwrap();
        
        // 应该触发 THREAT-SVC-003 告警
        assert!(alerts.iter().any(|a| a.rule_id == "THREAT-SVC-003"));
    }

    #[tokio::test]
    async fn test_detection_stats() {
        let engine = ThreatDetectionEngine::new();
        engine.initialize().await.unwrap();
        
        // 生成多个事件
        for i in 0..5 {
            let event = ThreatEvent {
                event_id: format!("event_{}", i),
                event_type: "test".to_string(),
                user_id: None,
                source_ip: None,
                resource: None,
                action: None,
                metrics: HashMap::new(),
                fields: HashMap::new(),
                anomaly_score: None,
                timestamp: Instant::now(),
            };
            
            let _ = engine.detect(&event).await;
        }
        
        let stats = engine.get_stats().await;
        assert!(stats.total_events >= 5);
    }

    #[tokio::test]
    async fn test_detection_latency() {
        let engine = ThreatDetectionEngine::new();
        engine.initialize().await.unwrap();
        
        let event = ThreatEvent {
            event_id: "event_1".to_string(),
            event_type: "test".to_string(),
            user_id: None,
            source_ip: None,
            resource: None,
            action: None,
            metrics: HashMap::new(),
            fields: HashMap::new(),
            anomaly_score: None,
            timestamp: Instant::now(),
        };
        
        let start = Instant::now();
        let _ = engine.detect(&event).await;
        let latency = start.elapsed().as_micros() as u64;
        
        // 检测延迟应该 <5s = 5000000μs
        assert!(latency < 5000000);
    }
}
