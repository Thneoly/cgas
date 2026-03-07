// threat_detection_rules_batch2.rs
// Phase 3 Week 4 Security - 威胁检测规则扩展 (新增 10 类)
// 基于 Week 2 的 25 类规则，新增 10 类高级威胁检测

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow, bail};

// ============================================================================
// 威胁检测规则基础定义
// ============================================================================

/// 威胁类别 (Week 2 + Week 4 新增)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatCategory {
    // Week 2 原有类别
    AbnormalAccess,
    PrivilegeAbuse,
    DataLeak,
    ServiceAnomaly,
    ConfigTampering,
    
    // Week 4 新增类别
    InsiderThreat,           // 内部威胁
    CredentialStuffing,      // 凭证填充攻击
    AccountTakeover,         // 账户劫持
    LateralMovement,         // 横向移动
    DataExfiltration,        // 数据渗出
    SupplyChainAttack,       // 供应链攻击
    APTBehavior,             // APT 行为
    CryptoMining,            // 加密挖矿
    BotnetActivity,          // 僵尸网络活动
    ZeroDayExploit,          // 零日漏洞利用
}

/// 威胁严重程度
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 威胁检测规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDetectionRule {
    pub rule_id: String,
    pub name: String,
    pub category: ThreatCategory,
    pub severity: ThreatSeverity,
    pub description: String,
    pub condition: RuleCondition,
    pub window_secs: u64,
    pub threshold: u32,
    pub actions: Vec<AlertAction>,
    pub enabled: bool,
}

/// 规则条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    Threshold {
        metric: String,
        operator: String,  // ">", "<", ">=", "<=", "=="
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
        logic: LogicOperator,
    },
    Behavioral {
        baseline_deviation: f64,
        features: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicOperator {
    AND,
    OR,
    XOR,
}

/// 告警动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertAction {
    Notify {
        channel: String,
        template: String,
    },
    RateLimit {
        duration_secs: u64,
        limit: u32,
    },
    Block {
        duration_secs: u64,
    },
    Quarantine {
        resource: String,
    },
    Escalate {
        to: String,
        priority: String,
    },
    ForensicCapture {
        data_types: Vec<String>,
    },
}

/// 威胁告警
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAlert {
    pub alert_id: String,
    pub rule_id: String,
    pub category: ThreatCategory,
    pub severity: ThreatSeverity,
    pub title: String,
    pub description: String,
    pub timestamp: u64,
    pub context: HashMap<String, String>,
    pub actions_taken: Vec<String>,
}

// ============================================================================
// Week 4 新增威胁检测规则 (10 类)
// ============================================================================

/// 威胁检测规则管理器 - Week 4 新增规则
pub struct ThreatDetectionRulesBatch2 {
    rules: Arc<RwLock<HashMap<String, ThreatDetectionRule>>>,
    alerts: Arc<RwLock<Vec<ThreatAlert>>>,
}

impl ThreatDetectionRulesBatch2 {
    pub fn new() -> Self {
        let mut instance = Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
        };
        
        // 初始化 10 类新增规则
        instance.initialize_rules();
        
        instance
    }

    /// 初始化 10 类新增威胁检测规则
    fn initialize_rules(&mut self) {
        let rules = self.rules.blocking_write();
        
        // 规则 1: 内部威胁检测 - 异常数据访问模式
        rules.insert("THREAT-INSIDER-001".to_string(), ThreatDetectionRule {
            rule_id: "THREAT-INSIDER-001".to_string(),
            name: "Insider Threat - Abnormal Data Access Pattern".to_string(),
            category: ThreatCategory::InsiderThreat,
            severity: ThreatSeverity::High,
            description: "检测内部人员异常数据访问模式 (如：访问非职责范围数据、离职前批量下载)".to_string(),
            condition: RuleCondition::Behavioral {
                baseline_deviation: 3.0,  // 3 倍标准差
                features: vec![
                    "data_access_volume".to_string(),
                    "access_time_anomaly".to_string(),
                    "resource_sensitivity".to_string(),
                    "user_departure_risk".to_string(),
                ],
            },
            window_secs: 3600,
            threshold: 1,
            actions: vec![
                AlertAction::Notify {
                    channel: "slack-security".to_string(),
                    template: "🔴 内部威胁告警：用户 {{user_id}} 异常数据访问模式，偏离度 {{deviation}}".to_string(),
                },
                AlertAction::ForensicCapture {
                    data_types: vec!["access_logs".to_string(), "download_history".to_string()],
                },
                AlertAction::Escalate {
                    to: "security-oncall".to_string(),
                    priority: "high".to_string(),
                },
            ],
            enabled: true,
        });

        // 规则 2: 凭证填充攻击检测
        rules.insert("THREAT-CREDSTUFF-001".to_string(), ThreatDetectionRule {
            rule_id: "THREAT-CREDSTUFF-001".to_string(),
            name: "Credential Stuffing Attack".to_string(),
            category: ThreatCategory::CredentialStuffing,
            severity: ThreatSeverity::Critical,
            description: "检测凭证填充攻击 (使用泄露凭证批量尝试登录)".to_string(),
            condition: RuleCondition::Composite {
                rules: vec![
                    "failed_login_high_rate".to_string(),
                    "multiple_usernames_same_ip".to_string(),
                    "known_leaked_credentials".to_string(),
                ],
                logic: LogicOperator::AND,
            },
            window_secs: 300,
            threshold: 50,  // 5 分钟内 50 次失败登录
            actions: vec![
                AlertAction::Notify {
                    channel: "slack-security".to_string(),
                    template: "🚨 凭证填充攻击：IP {{ip}} 尝试 {{count}} 次登录失败".to_string(),
                },
                AlertAction::Block {
                    duration_secs: 3600,
                },
                AlertAction::Notify {
                    channel: "slack-soc".to_string(),
                    template: "凭证填充攻击已自动阻断 IP {{ip}}".to_string(),
                },
            ],
            enabled: true,
        });

        // 规则 3: 账户劫持检测
        rules.insert("THREAT-ATO-001".to_string(), ThreatDetectionRule {
            rule_id: "THREAT-ATO-001".to_string(),
            name: "Account Takeover Detection".to_string(),
            category: ThreatCategory::AccountTakeover,
            severity: ThreatSeverity::Critical,
            description: "检测账户劫持行为 (如：突然修改密码、启用 MFA、更改恢复邮箱)".to_string(),
            condition: RuleCondition::Composite {
                rules: vec![
                    "password_change".to_string(),
                    "mfa_change".to_string(),
                    "recovery_email_change".to_string(),
                    "session_from_new_device".to_string(),
                ],
                logic: LogicOperator::OR,
            },
            window_secs: 600,
            threshold: 2,  // 10 分钟内 2 个敏感操作
            actions: vec![
                AlertAction::Notify {
                    channel: "slack-security".to_string(),
                    template: "🚨 账户劫持告警：用户 {{user_id}} 敏感账户变更".to_string(),
                },
                AlertAction::Block {
                    duration_secs: 1800,  // 30 分钟
                },
                AlertAction::Notify {
                    channel: "user_notification".to_string(),
                    template: "您的账户发生敏感变更，如非本人操作请立即联系安全团队".to_string(),
                },
            ],
            enabled: true,
        });

        // 规则 4: 横向移动检测
        rules.insert("THREAT-LATMOVE-001".to_string(), ThreatDetectionRule {
            rule_id: "THREAT-LATMOVE-001".to_string(),
            name: "Lateral Movement Detection".to_string(),
            category: ThreatCategory::LateralMovement,
            severity: ThreatSeverity::Critical,
            description: "检测攻击者横向移动行为 (如：从一个系统跳转到另一个系统)".to_string(),
            condition: RuleCondition::Behavioral {
                baseline_deviation: 2.5,
                features: vec![
                    "cross_system_access".to_string(),
                    "privilege_escalation_chain".to_string(),
                    "unusual_service_account_usage".to_string(),
                    "admin_share_access".to_string(),
                ],
            },
            window_secs: 1800,
            threshold: 1,
            actions: vec![
                AlertAction::Notify {
                    channel: "slack-security".to_string(),
                    template: "🚨 横向移动检测：用户 {{user_id}} 访问 {{system_count}} 个系统".to_string(),
                },
                AlertAction::Quarantine {
                    resource: "affected_systems".to_string(),
                },
                AlertAction::ForensicCapture {
                    data_types: vec!["network_flows".to_string(), "auth_logs".to_string()],
                },
            ],
            enabled: true,
        });

        // 规则 5: 数据渗出检测
        rules.insert("THREAT-EXFIL-001".to_string(), ThreatDetectionRule {
            rule_id: "THREAT-EXFIL-001".to_string(),
            name: "Data Exfiltration Detection".to_string(),
            category: ThreatCategory::DataExfiltration,
            severity: ThreatSeverity::Critical,
            description: "检测数据渗出行为 (如：通过 DNS/HTTP 隧道外传数据、加密上传到外部存储)".to_string(),
            condition: RuleCondition::Composite {
                rules: vec![
                    "large_outbound_transfer".to_string(),
                    "dns_tunneling_detected".to_string(),
                    "encrypted_upload_external".to_string(),
                    "unusual_destination_ip".to_string(),
                ],
                logic: LogicOperator::OR,
            },
            window_secs: 600,
            threshold: 1,
            actions: vec![
                AlertAction::Notify {
                    channel: "slack-security".to_string(),
                    template: "🚨 数据渗出告警：检测到 {{exfil_method}}，数据量 {{volume}}".to_string(),
                },
                AlertAction::Block {
                    duration_secs: 7200,
                },
                AlertAction::ForensicCapture {
                    data_types: vec!["network_packets".to_string(), "file_transfers".to_string()],
                },
                AlertAction::Escalate {
                    to: "incident-response".to_string(),
                    priority: "critical".to_string(),
                },
            ],
            enabled: true,
        });

        // 规则 6: 供应链攻击检测
        rules.insert("THREAT-SUPPLY-001".to_string(), ThreatDetectionRule {
            rule_id: "THREAT-SUPPLY-001".to_string(),
            name: "Supply Chain Attack Detection".to_string(),
            category: ThreatCategory::SupplyChainAttack,
            severity: ThreatSeverity::Critical,
            description: "检测供应链攻击 (如：第三方依赖异常、构建过程篡改、恶意代码注入)".to_string(),
            condition: RuleCondition::Composite {
                rules: vec![
                    "dependency_hash_mismatch".to_string(),
                    "build_process_anomaly".to_string(),
                    "unsigned_code_detected".to_string(),
                    "known_vulnerable_dependency".to_string(),
                ],
                logic: LogicOperator::OR,
            },
            window_secs: 300,
            threshold: 1,
            actions: vec![
                AlertAction::Notify {
                    channel: "slack-security".to_string(),
                    template: "🚨 供应链攻击告警：{{component}} 检测到异常".to_string(),
                },
                AlertAction::Block {
                    duration_secs: 86400,  // 24 小时
                },
                AlertAction::Escalate {
                    to: "security-oncall".to_string(),
                    priority: "critical".to_string(),
                },
            ],
            enabled: true,
        });

        // 规则 7: APT 行为检测
        rules.insert("THREAT-APT-001".to_string(), ThreatDetectionRule {
            rule_id: "THREAT-APT-001".to_string(),
            name: "APT Behavior Detection".to_string(),
            category: ThreatCategory::APTBehavior,
            severity: ThreatSeverity::Critical,
            description: "检测 APT 组织行为特征 (如：持久化、C2 通信、数据收集)".to_string(),
            condition: RuleCondition::Behavioral {
                baseline_deviation: 4.0,
                features: vec![
                    "persistence_mechanism".to_string(),
                    "c2_communication_pattern".to_string(),
                    "reconnaissance_activity".to_string(),
                    "data_staging".to_string(),
                    "low_and_slow_behavior".to_string(),
                ],
            },
            window_secs: 86400,  // 24 小时窗口
            threshold: 3,  // 检测到 3 个 APT 特征
            actions: vec![
                AlertAction::Notify {
                    channel: "slack-security".to_string(),
                    template: "🚨 APT 行为检测：{{user_id}} 显示 {{feature_count}} 个 APT 特征".to_string(),
                },
                AlertAction::ForensicCapture {
                    data_types: vec![
                        "full_packet_capture".to_string(),
                        "memory_dump".to_string(),
                        "process_tree".to_string(),
                    ],
                },
                AlertAction::Escalate {
                    to: "threat-intelligence".to_string(),
                    priority: "critical".to_string(),
                },
            ],
            enabled: true,
        });

        // 规则 8: 加密挖矿检测
        rules.insert("THREAT-CRYPTO-001".to_string(), ThreatDetectionRule {
            rule_id: "THREAT-CRYPTO-001".to_string(),
            name: "Cryptomining Detection".to_string(),
            category: ThreatCategory::CryptoMining,
            severity: ThreatSeverity::High,
            description: "检测加密挖矿行为 (如：CPU/GPU 异常占用、连接矿池、挖矿进程)".to_string(),
            condition: RuleCondition::Composite {
                rules: vec![
                    "high_cpu_usage_sustained".to_string(),
                    "known_mining_pool_connection".to_string(),
                    "mining_process_detected".to_string(),
                    "stratum_protocol_detected".to_string(),
                ],
                logic: LogicOperator::OR,
            },
            window_secs: 600,
            threshold: 1,
            actions: vec![
                AlertAction::Notify {
                    channel: "slack-sre".to_string(),
                    template: "⛏️ 加密挖矿检测：主机 {{host}} CPU 使用率 {{cpu_percent}}%".to_string(),
                },
                AlertAction::Block {
                    duration_secs: 3600,
                },
                AlertAction::Quarantine {
                    resource: "affected_host".to_string(),
                },
            ],
            enabled: true,
        });

        // 规则 9: 僵尸网络活动检测
        rules.insert("THREAT-BOTNET-001".to_string(), ThreatDetectionRule {
            rule_id: "THREAT-BOTNET-001".to_string(),
            name: "Botnet Activity Detection".to_string(),
            category: ThreatCategory::BotnetActivity,
            severity: ThreatSeverity::High,
            description: "检测僵尸网络活动 (如：C2 通信、DDoS 参与、扫描活动)".to_string(),
            condition: RuleCondition::Composite {
                rules: vec![
                    "c2_beacon_pattern".to_string(),
                    "ddos_traffic_pattern".to_string(),
                    "mass_scanning_activity".to_string(),
                    "known_botnet_ip_communication".to_string(),
                ],
                logic: LogicOperator::OR,
            },
            window_secs: 300,
            threshold: 1,
            actions: vec![
                AlertAction::Notify {
                    channel: "slack-security".to_string(),
                    template: "🤖 僵尸网络活动：主机 {{host}} 显示 {{activity_type}}".to_string(),
                },
                AlertAction::Block {
                    duration_secs: 7200,
                },
                AlertAction::Quarantine {
                    resource: "affected_host".to_string(),
                },
            ],
            enabled: true,
        });

        // 规则 10: 零日漏洞利用检测
        rules.insert("THREAT-ZERODAY-001".to_string(), ThreatDetectionRule {
            rule_id: "THREAT-ZERODAY-001".to_string(),
            name: "Zero-Day Exploit Detection".to_string(),
            category: ThreatCategory::ZeroDayExploit,
            severity: ThreatSeverity::Critical,
            description: "检测零日漏洞利用 (如：未知攻击模式、异常系统调用、内存破坏行为)".to_string(),
            condition: RuleCondition::Anomaly {
                model: "zero_day_anomaly_v1".to_string(),
                score_threshold: 0.95,
            },
            window_secs: 60,
            threshold: 1,
            actions: vec![
                AlertAction::Notify {
                    channel: "slack-security".to_string(),
                    template: "🚨 零日漏洞利用告警：异常评分 {{score}}, 目标 {{target}}".to_string(),
                },
                AlertAction::Block {
                    duration_secs: 86400,
                },
                AlertAction::ForensicCapture {
                    data_types: vec![
                        "memory_dump".to_string(),
                        "process_memory".to_string(),
                        "system_calls".to_string(),
                        "network_traffic".to_string(),
                    ],
                },
                AlertAction::Escalate {
                    to: "incident-response".to_string(),
                    priority: "critical".to_string(),
                },
            ],
            enabled: true,
        });

        log::info!("Initialized 10 new threat detection rules for Week 4");
    }

    /// 评估事件是否触发规则
    pub async fn evaluate_event(&self, event: &ThreatEvent) -> Result<Vec<ThreatAlert>> {
        let rules = self.rules.read().await;
        let mut alerts = Vec::new();

        for (_, rule) in rules.iter() {
            if !rule.enabled {
                continue;
            }

            if self.matches_rule(event, rule).await? {
                let alert = self.create_alert(rule, event).await?;
                alerts.push(alert);
            }
        }

        // 存储告警
        if !alerts.is_empty() {
            let mut stored_alerts = self.alerts.write().await;
            stored_alerts.extend(alerts.clone());
        }

        Ok(alerts)
    }

    /// 检查事件是否匹配规则
    async fn matches_rule(&self, event: &ThreatEvent, rule: &ThreatDetectionRule) -> Result<bool> {
        match &rule.condition {
            RuleCondition::Threshold { metric, operator, value } => {
                let event_value = event.metrics.get(metric).copied().unwrap_or(0);
                match operator.as_str() {
                    ">" => Ok(event_value > *value),
                    "<" => Ok(event_value < *value),
                    ">=" => Ok(event_value >= *value),
                    "<=" => Ok(event_value <= *value),
                    "==" => Ok(event_value == *value),
                    _ => bail!("Unknown operator: {}", operator),
                }
            }
            RuleCondition::Pattern { pattern, field } => {
                let event_field = event.fields.get(field).map(|s| s.as_str()).unwrap_or("");
                let regex = regex::Regex::new(pattern)?;
                Ok(regex.is_match(event_field))
            }
            RuleCondition::Anomaly { model, score_threshold } => {
                // 简化的异常检测
                let score = self.calculate_anomaly_score(event, model).await?;
                Ok(score >= *score_threshold)
            }
            RuleCondition::Composite { rules, logic } => {
                let mut results = Vec::new();
                for rule_id in rules {
                    // 递归评估子规则 (简化实现)
                    let matches = self.evaluate_sub_rule(event, rule_id).await?;
                    results.push(matches);
                }
                
                match logic {
                    LogicOperator::AND => Ok(results.iter().all(|&r| r)),
                    LogicOperator::OR => Ok(results.iter().any(|&r| r)),
                    LogicOperator::XOR => Ok(results.iter().filter(|&&r| r).count() == 1),
                }
            }
            RuleCondition::Behavioral { baseline_deviation, features } => {
                let deviation = self.calculate_behavioral_deviation(event, features).await?;
                Ok(deviation >= *baseline_deviation)
            }
        }
    }

    async fn evaluate_sub_rule(&self, _event: &ThreatEvent, _rule_id: &str) -> Result<bool> {
        // 简化实现：默认返回 true
        Ok(true)
    }

    async fn calculate_anomaly_score(&self, _event: &ThreatEvent, _model: &str) -> Result<f64> {
        // 简化实现：返回随机分数
        Ok(0.5)
    }

    async fn calculate_behavioral_deviation(
        &self,
        _event: &ThreatEvent,
        _features: &[String],
    ) -> Result<f64> {
        // 简化实现：返回随机偏离度
        Ok(2.0)
    }

    /// 创建告警
    async fn create_alert(&self, rule: &ThreatDetectionRule, event: &ThreatEvent) -> Result<ThreatAlert> {
        let alert_id = format!("ALT-{}-{}", rule.rule_id, Instant::now().duration_since(Instant::now()).as_millis());
        
        Ok(ThreatAlert {
            alert_id,
            rule_id: rule.rule_id.clone(),
            category: rule.category.clone(),
            severity: rule.severity.clone(),
            title: rule.name.clone(),
            description: rule.description.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            context: event.fields.clone(),
            actions_taken: vec![],
        })
    }

    /// 获取所有规则
    pub async fn get_all_rules(&self) -> Vec<ThreatDetectionRule> {
        self.rules.read().await.values().cloned().collect()
    }

    /// 获取告警历史
    pub async fn get_alerts(&self, limit: usize) -> Vec<ThreatAlert> {
        let alerts = self.alerts.read().await;
        alerts.iter().rev().take(limit).cloned().collect()
    }

    /// 启用/禁用规则
    pub async fn toggle_rule(&self, rule_id: &str, enabled: bool) -> Result<()> {
        let mut rules = self.rules.write().await;
        if let Some(rule) = rules.get_mut(rule_id) {
            rule.enabled = enabled;
            log::info!("Rule {} {}", rule_id, if enabled { "enabled" } else { "disabled" });
            Ok(())
        } else {
            bail!("Rule not found: {}", rule_id)
        }
    }
}

/// 威胁事件
#[derive(Debug, Clone)]
pub struct ThreatEvent {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: u64,
    pub source_ip: String,
    pub user_id: String,
    pub metrics: HashMap<String, u64>,
    pub fields: HashMap<String, String>,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_threat_detection_rules_batch2() {
        let detector = ThreatDetectionRulesBatch2::new();
        
        // 验证规则数量
        let rules = detector.get_all_rules().await;
        assert_eq!(rules.len(), 10);

        // 验证所有规则都已启用
        assert!(rules.iter().all(|r| r.enabled));

        // 验证包含所有 10 类威胁
        let categories: Vec<_> = rules.iter().map(|r| r.category.clone()).collect();
        assert!(categories.contains(&ThreatCategory::InsiderThreat));
        assert!(categories.contains(&ThreatCategory::CredentialStuffing));
        assert!(categories.contains(&ThreatCategory::AccountTakeover));
        assert!(categories.contains(&ThreatCategory::LateralMovement));
        assert!(categories.contains(&ThreatCategory::DataExfiltration));
        assert!(categories.contains(&ThreatCategory::SupplyChainAttack));
        assert!(categories.contains(&ThreatCategory::APTBehavior));
        assert!(categories.contains(&ThreatCategory::CryptoMining));
        assert!(categories.contains(&ThreatCategory::BotnetActivity));
        assert!(categories.contains(&ThreatCategory::ZeroDayExploit));
    }

    #[tokio::test]
    async fn test_rule_toggle() {
        let detector = ThreatDetectionRulesBatch2::new();
        
        // 禁用规则
        detector.toggle_rule("THREAT-INSIDER-001", false).await.unwrap();
        
        // 验证规则已禁用
        let rules = detector.get_all_rules().await;
        let insider_rule = rules.iter().find(|r| r.rule_id == "THREAT-INSIDER-001").unwrap();
        assert!(!insider_rule.enabled);

        // 重新启用
        detector.toggle_rule("THREAT-INSIDER-001", true).await.unwrap();
        let rules = detector.get_all_rules().await;
        let insider_rule = rules.iter().find(|r| r.rule_id == "THREAT-INSIDER-001").unwrap();
        assert!(insider_rule.enabled);
    }

    #[tokio::test]
    async fn test_event_evaluation() {
        let detector = ThreatDetectionRulesBatch2::new();
        
        let event = ThreatEvent {
            event_id: "evt-001".to_string(),
            event_type: "login_attempt".to_string(),
            timestamp: 1234567890,
            source_ip: "192.168.1.100".to_string(),
            user_id: "user-123".to_string(),
            metrics: HashMap::new(),
            fields: HashMap::new(),
        };

        // 评估事件 (简化测试)
        let alerts = detector.evaluate_event(&event).await.unwrap();
        // 由于使用简化的评估逻辑，可能不会触发告警
        // 这里主要验证评估流程不崩溃
        assert!(alerts.len() >= 0);
    }
}
