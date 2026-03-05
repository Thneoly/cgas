/// 门禁报告生成器
/// 
/// 输出机器可读的 gate-report.json，用于 CI/CD 阻断决策

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 门禁报告主结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateReport {
    /// 报告版本
    pub report_version: String,
    
    /// 发布 ID
    pub release_id: String,
    
    /// 统计窗口（天数）
    pub window_days: u32,
    
    /// 生成时间戳
    pub generated_at: u64,
    
    /// 整体门禁状态
    pub overall_status: GateStatus,
    
    /// 各门禁检查结果
    pub gates: Vec<GateCheckResult>,
    
    /// 指标数据（原始输入）
    pub metrics: GateMetrics,
    
    /// 失败处置建议
    pub failure_actions: Vec<FailureAction>,
}

/// 门禁状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    /// 全部通过
    Pass,
    
    /// 有条件通过（有非关键失败）
    ConditionalPass,
    
    /// 失败（有关键门禁未通过）
    Fail,
}

/// 单个门禁检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCheckResult {
    /// 门禁 ID
    pub gate_id: String,
    
    /// 门禁名称
    pub gate_name: String,
    
    /// 检查状态
    pub status: CheckStatus,
    
    /// 失败原因码
    pub reason_code: Option<String>,
    
    /// 失败详细描述
    pub reason_detail: Option<String>,
    
    /// 证据引用（用于审计）
    pub evidence: HashMap<String, Value>,
    
    /// 置信度（0-1）
    pub confidence: Option<f64>,
}

/// 检查状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    /// 通过
    Pass,
    
    /// 失败
    Fail,
    
    /// 不可判定（数据不足）
    Undetermined,
}

/// 门禁指标数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateMetrics {
    /// 统计窗口天数
    pub window_days: u32,
    
    /// 全部提交请求数
    pub total_submissions: u64,
    
    /// 未经验证即提交数
    pub unverified_submissions: u64,
    
    /// 高危请求总数
    pub high_risk_requests: u64,
    
    /// 高危误放行数
    pub high_risk_false_allows: u64,
    
    /// 进入验证流程请求数
    pub verify_requests: u64,
    
    /// 验证一致请求数
    pub verify_consistent: u64,
    
    /// 验证延迟 P95（毫秒）
    pub verify_p95_ms: Option<u64>,
    
    /// 高危拦截延迟 P95（毫秒）
    pub high_risk_block_p95_ms: Option<u64>,
    
    /// 回滚总次数
    pub rollback_total: u64,
    
    /// 回滚成功次数
    pub rollback_success: u64,
    
    /// 高危策略变更总次数
    pub policy_change_total: u64,
    
    /// 完整闭环的策略变更次数
    pub policy_change_closed_loop: u64,
    
    /// 未声明能力调用次数
    pub undeclared_cap_calls: u64,
    
    /// 被拦截的未声明能力调用次数
    pub undeclared_cap_blocked: u64,
    
    /// 核心回放样本量
    pub core_replay_samples: u64,
    
    /// 高危样本量
    pub high_risk_samples: u64,
    
    /// 连续通过双周关卡次数
    pub checkpoint_passes: u32,
}

impl Default for GateMetrics {
    fn default() -> Self {
        Self {
            window_days: 14,
            total_submissions: 0,
            unverified_submissions: 0,
            high_risk_requests: 0,
            high_risk_false_allows: 0,
            verify_requests: 0,
            verify_consistent: 0,
            verify_p95_ms: None,
            high_risk_block_p95_ms: None,
            rollback_total: 0,
            rollback_success: 0,
            policy_change_total: 0,
            policy_change_closed_loop: 0,
            undeclared_cap_calls: 0,
            undeclared_cap_blocked: 0,
            core_replay_samples: 0,
            high_risk_samples: 0,
            checkpoint_passes: 0,
        }
    }
}

/// 失败处置动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureAction {
    /// 触发的门禁
    pub gate_id: String,
    
    /// 自动动作
    pub action: String,
    
    /// 恢复条件
    pub recovery_condition: String,
    
    /// 责任人
    pub owner: Option<String>,
    
    /// SLA（小时）
    pub sla_hours: Option<u32>,
}

/// 门禁报告生成器
pub struct GateReportGenerator;

impl GateReportGenerator {
    /// 生成门禁报告
    pub fn generate(metrics: GateMetrics, release_id: &str) -> GateReport {
        let mut gates = Vec::new();
        let mut failure_actions = Vec::new();
        
        // Gate 1: 未验证提交率 = 0
        let gate_unverified = Self::check_unverified_zero(&metrics);
        if gate_unverified.status == CheckStatus::Fail {
            failure_actions.push(FailureAction {
                gate_id: "gate_trust_unverified_zero".to_string(),
                action: "冻结功能发布，仅允许修复分支".to_string(),
                recovery_condition: "14 天窗口恢复达标 + 连续 2 个双周关卡通过".to_string(),
                owner: Some("Core".to_string()),
                sla_hours: Some(24),
            });
        }
        gates.push(gate_unverified);
        
        // Gate 2: 高危误放行率 <= 0.1%
        let gate_false_allow = Self::check_false_allow_rate(&metrics);
        if gate_false_allow.status == CheckStatus::Fail {
            failure_actions.push(FailureAction {
                gate_id: "gate_security_false_allow".to_string(),
                action: "阻断发布，触发安全事件单（P0/P1）".to_string(),
                recovery_condition: "误放行率与 u95 均达标".to_string(),
                owner: Some("Security".to_string()),
                sla_hours: Some(4),
            });
        }
        gates.push(gate_false_allow);
        
        // Gate 3: 治理闭环
        let gate_governance = Self::check_governance_closed_loop(&metrics);
        if gate_governance.status == CheckStatus::Fail {
            failure_actions.push(FailureAction {
                gate_id: "gate_governance_closed_loop".to_string(),
                action: "阻断策略上线".to_string(),
                recovery_condition: "补齐提案 - 测试 - 灰度 - 回滚证据链".to_string(),
                owner: Some("PM".to_string()),
                sla_hours: Some(48),
            });
        }
        gates.push(gate_governance);
        
        // Gate 4: 样本量达标
        let gate_sample = Self::check_sample_minimum(&metrics);
        if gate_sample.status == CheckStatus::Fail {
            failure_actions.push(FailureAction {
                gate_id: "gate_sample_minimum".to_string(),
                action: "阻断通过判定，只允许补样/修复".to_string(),
                recovery_condition: "样本量达标".to_string(),
                owner: Some("QA".to_string()),
                sla_hours: Some(72),
            });
        }
        gates.push(gate_sample);
        
        // 计算整体状态
        let overall_status = Self::compute_overall_status(&gates);
        
        GateReport {
            report_version: "v1.0".to_string(),
            release_id: release_id.to_string(),
            window_days: metrics.window_days,
            generated_at: Self::current_timestamp(),
            overall_status,
            gates,
            metrics,
            failure_actions,
        }
    }
    
    /// 检查未验证提交率
    fn check_unverified_zero(metrics: &GateMetrics) -> GateCheckResult {
        let status = if metrics.unverified_submissions == 0 
            && metrics.window_days >= 14 
            && metrics.checkpoint_passes >= 2 
        {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        };
        
        let mut evidence = HashMap::new();
        evidence.insert("unverified_submissions".to_string(), Value::Number(metrics.unverified_submissions.into()));
        evidence.insert("window_days".to_string(), Value::Number(metrics.window_days.into()));
        evidence.insert("checkpoint_passes".to_string(), Value::Number(metrics.checkpoint_passes.into()));
        
        GateCheckResult {
            gate_id: "gate_trust_unverified_zero".to_string(),
            gate_name: "未验证提交率红线".to_string(),
            status,
            reason_code: if status == CheckStatus::Fail { Some("unverified_nonzero".to_string()) } else { None },
            reason_detail: if status == CheckStatus::Fail {
                Some(format!(
                    "unverified_submissions={} (要求=0), window_days={}, checkpoint_passes={}",
                    metrics.unverified_submissions, metrics.window_days, metrics.checkpoint_passes
                ))
            } else {
                None
            },
            evidence,
            confidence: None,
        }
    }
    
    /// 检查高危误放行率
    fn check_false_allow_rate(metrics: &GateMetrics) -> GateCheckResult {
        let rate = if metrics.high_risk_requests > 0 {
            metrics.high_risk_false_allows as f64 / metrics.high_risk_requests as f64
        } else {
            0.0
        };
        
        let status = if metrics.high_risk_samples >= 1000 && rate <= 0.001 {
            CheckStatus::Pass
        } else if metrics.high_risk_requests == 0 {
            CheckStatus::Undetermined
        } else {
            CheckStatus::Fail
        };
        
        let mut evidence = HashMap::new();
        evidence.insert("high_risk_samples".to_string(), Value::Number(metrics.high_risk_samples.into()));
        evidence.insert("high_risk_false_allows".to_string(), Value::Number(metrics.high_risk_false_allows.into()));
        evidence.insert("high_risk_requests".to_string(), Value::Number(metrics.high_risk_requests.into()));
        evidence.insert("false_allow_rate".to_string(), Value::Number(((rate * 10000.0).round() as i64).into()));
        
        GateCheckResult {
            gate_id: "gate_security_false_allow".to_string(),
            gate_name: "高危误放行率红线".to_string(),
            status,
            reason_code: if status == CheckStatus::Fail { Some("false_allow_exceeded".to_string()) } else { None },
            reason_detail: if status == CheckStatus::Fail {
                Some(format!(
                    "false_allow_rate={:.4}% (要求<=0.1%), high_risk_samples={} (要求>=1000)",
                    rate * 100.0, metrics.high_risk_samples
                ))
            } else {
                None
            },
            evidence,
            confidence: Some(0.95),
        }
    }
    
    /// 检查治理闭环
    fn check_governance_closed_loop(metrics: &GateMetrics) -> GateCheckResult {
        let rate = if metrics.policy_change_total > 0 {
            metrics.policy_change_closed_loop as f64 / metrics.policy_change_total as f64
        } else {
            1.0 // 无变更视为通过
        };
        
        let status = if metrics.policy_change_total == 0 || rate == 1.0 {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        };
        
        let mut evidence = HashMap::new();
        evidence.insert("policy_change_total".to_string(), Value::Number(metrics.policy_change_total.into()));
        evidence.insert("policy_change_closed_loop".to_string(), Value::Number(metrics.policy_change_closed_loop.into()));
        evidence.insert("closed_loop_rate".to_string(), Value::Number(((rate * 100.0) as i64).into()));
        
        GateCheckResult {
            gate_id: "gate_governance_closed_loop".to_string(),
            gate_name: "治理闭环红线".to_string(),
            status,
            reason_code: if status == CheckStatus::Fail { Some("governance_incomplete".to_string()) } else { None },
            reason_detail: if status == CheckStatus::Fail {
                Some(format!(
                    "closed_loop_rate={:.1}% (要求=100%)",
                    rate * 100.0
                ))
            } else {
                None
            },
            evidence,
            confidence: None,
        }
    }
    
    /// 检查样本量
    fn check_sample_minimum(metrics: &GateMetrics) -> GateCheckResult {
        let core_ok = metrics.core_replay_samples >= 200;
        let high_risk_ok = metrics.high_risk_samples == 0 || metrics.high_risk_samples >= 1000;
        
        let status = if core_ok && high_risk_ok {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        };
        
        let mut evidence = HashMap::new();
        evidence.insert("core_replay_samples".to_string(), Value::Number(metrics.core_replay_samples.into()));
        evidence.insert("high_risk_samples".to_string(), Value::Number(metrics.high_risk_samples.into()));
        
        GateCheckResult {
            gate_id: "gate_sample_minimum".to_string(),
            gate_name: "样本量红线".to_string(),
            status,
            reason_code: if status == CheckStatus::Fail { Some("insufficient_samples".to_string()) } else { None },
            reason_detail: if status == CheckStatus::Fail {
                Some(format!(
                    "core_replay_samples={} (要求>=200), high_risk_samples={} (要求>=1000 或=0)",
                    metrics.core_replay_samples, metrics.high_risk_samples
                ))
            } else {
                None
            },
            evidence,
            confidence: None,
        }
    }
    
    /// 计算整体状态
    fn compute_overall_status(gates: &[GateCheckResult]) -> GateStatus {
        let has_fail = gates.iter().any(|g| g.status == CheckStatus::Fail);
        let has_undetermined = gates.iter().any(|g| g.status == CheckStatus::Undetermined);
        
        if has_fail {
            GateStatus::Fail
        } else if has_undetermined {
            GateStatus::ConditionalPass
        } else {
            GateStatus::Pass
        }
    }
    
    /// 当前时间戳（秒）
    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// 保存报告到文件
    pub fn save_report(report: &GateReport, output_path: &Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(report)?;
        fs::write(output_path, json)?;
        Ok(())
    }
    
    /// 生成人类可读的 Markdown 摘要
    pub fn generate_markdown_summary(report: &GateReport) -> String {
        let mut md = String::new();
        
        md.push_str(&format!("# 门禁报告摘要\n\n"));
        md.push_str(&format!("- **发布 ID**: {}\n", report.release_id));
        md.push_str(&format!("- **生成时间**: {}\n", Self::format_timestamp(report.generated_at)));
        md.push_str(&format!("- **统计窗口**: {} 天\n", report.window_days));
        md.push_str(&format!("- **整体状态**: {:?}\n\n", report.overall_status));
        
        md.push_str("## 门禁检查结果\n\n");
        md.push_str("| 门禁 ID | 名称 | 状态 | 详情 |\n");
        md.push_str("|--------|------|------|------|\n");
        
        for gate in &report.gates {
            let status_icon = match gate.status {
                CheckStatus::Pass => "✅",
                CheckStatus::Fail => "❌",
                CheckStatus::Undetermined => "⚠️",
            };
            let detail = gate.reason_detail.as_deref().unwrap_or("-");
            md.push_str(&format!(
                "| {} | {} | {} {} |\n",
                gate.gate_id, gate.gate_name, status_icon, detail
            ));
        }
        
        if !report.failure_actions.is_empty() {
            md.push_str("\n## 失败处置\n\n");
            for action in &report.failure_actions {
                md.push_str(&format!(
                    "### {}\n- **动作**: {}\n- **恢复条件**: {}\n- **责任人**: {}\n- **SLA**: {} 小时\n\n",
                    action.gate_id,
                    action.action,
                    action.recovery_condition,
                    action.owner.as_deref().unwrap_or("未指定"),
                    action.sla_hours.map(|h| h.to_string()).unwrap_or_else(|| "N/A".to_string())
                ));
            }
        }
        
        md
    }
    
    fn format_timestamp(ts: u64) -> String {
        use std::time::{SystemTime, UNIX_EPOCH, Duration};
        let duration = Duration::from_secs(ts);
        let datetime: chrono::DateTime<chrono::Utc> = UNIX_EPOCH.checked_add(duration).unwrap().into();
        datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }
}
