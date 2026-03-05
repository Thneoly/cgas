use crate::model::{BlackboardEvent, BlackboardState, Role};
use serde_json::{json, Value};

pub struct BlackboardAgent;

pub struct BlackboardIngestResult {
    pub normalized_artifact: Value,
    pub blackboard_artifact: Value,
    pub next_role: Option<Role>,
}

impl BlackboardAgent {
    pub fn ingest(
        blackboard: &mut BlackboardState,
        release_id: &str,
        round: usize,
        source_role: &Role,
        raw_artifact: Value,
    ) -> BlackboardIngestResult {
        let normalized = normalize_artifact(raw_artifact, release_id, source_role);
        let decision = normalized
            .get("decision")
            .and_then(Value::as_str)
            .unwrap_or("rejected")
            .to_string();
        let summary = normalized
            .get("summary")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let next_role = normalized
            .get("next_role")
            .and_then(Value::as_str)
            .and_then(parse_next_role_flexible);

        blackboard.version += 1;
        blackboard
            .slots
            .insert(source_role.clone(), normalized.clone());
        blackboard.events.push(BlackboardEvent {
            id: blackboard.version,
            round,
            from: source_role.as_key().to_string(),
            event_type: "artifact_published".to_string(),
            decision: decision.clone(),
            summary: summary.clone(),
            next_role: next_role.as_ref().map(|r| r.as_key().to_string()),
            audit: json!({
                "rule_id": "blackboard.ingest.v1",
                "trigger": "artifact_published",
                "source_role": source_role.as_key(),
                "release_id": release_id,
                "normalized_fields": ["role", "release_id", "decision", "summary", "next_role"]
            }),
        });

        let blackboard_artifact = json!({
            "role": "Blackboard",
            "release_id": release_id,
            "decision": "approved",
            "summary": format!(
                "blackboard merged artifact from {} and broadcast feedback",
                source_role.as_key()
            ),
            "next_role": next_role.as_ref().map(|r| r.as_key()),
            "source_role": source_role.as_key(),
            "blackboard_version": blackboard.version,
            "event_type": "artifact_published",
            "normalized": normalized,
            "evidence": {
                "metric_value": blackboard.version,
                "window": format!("round-{}", round),
                "sample_size": blackboard.events.len(),
                "source": "blackboard-agent"
            }
        });

        BlackboardIngestResult {
            normalized_artifact: blackboard_artifact["normalized"].clone(),
            blackboard_artifact,
            next_role,
        }
    }

    pub fn record_dispatch(
        blackboard: &mut BlackboardState,
        round: usize,
        target: &Role,
        reason: &str,
        audit: Value,
    ) {
        blackboard.version += 1;
        blackboard.events.push(BlackboardEvent {
            id: blackboard.version,
            round,
            from: "Engine".to_string(),
            event_type: "dispatch_decided".to_string(),
            decision: "scheduled".to_string(),
            summary: reason.to_string(),
            next_role: Some(target.as_key().to_string()),
            audit,
        });
    }
}

fn normalize_artifact(mut artifact: Value, release_id: &str, source_role: &Role) -> Value {
    artifact["role"] = Value::String(source_role.as_key().to_string());
    artifact["release_id"] = Value::String(release_id.to_string());

    let decision = artifact
        .get("decision")
        .and_then(Value::as_str)
        .unwrap_or("rejected");
    let normalized_decision = if decision.eq_ignore_ascii_case("approved") {
        "approved"
    } else {
        "rejected"
    };
    artifact["decision"] = Value::String(normalized_decision.to_string());

    if artifact.get("summary").and_then(Value::as_str).is_none() {
        artifact["summary"] = Value::String("no summary provided".to_string());
    }

    if let Some(next) = artifact.get("next_role").and_then(Value::as_str) {
        if let Some(role) = parse_next_role_flexible(next) {
            artifact["next_role"] = Value::String(role.as_key().to_string());
        } else {
            artifact["next_role"] = Value::Null;
        }
    } else {
        artifact["next_role"] = Value::Null;
    }

    normalize_evidence(&mut artifact);
    normalize_skill_execution_fields(&mut artifact, source_role);

    artifact
}

fn normalize_skill_execution_fields(artifact: &mut Value, source_role: &Role) {
    let default_skill = format!(
        "{}_baseline_skill",
        source_role.as_key().to_ascii_lowercase()
    );
    let default_risk = format!(
        "{}_risk_control_baseline",
        source_role.as_key().to_ascii_lowercase()
    );

    let skills_valid = artifact
        .get("skills_applied")
        .and_then(Value::as_array)
        .is_some_and(|arr| {
            !arr.is_empty()
                && arr
                    .iter()
                    .all(|v| v.as_str().is_some_and(|s| !s.trim().is_empty()))
        });
    if !skills_valid {
        artifact["skills_applied"] = json!([default_skill]);
    }

    let risks_valid = artifact
        .get("risk_controls")
        .and_then(Value::as_array)
        .is_some_and(|arr| {
            !arr.is_empty()
                && arr
                    .iter()
                    .all(|v| v.as_str().is_some_and(|s| !s.trim().is_empty()))
        });
    if !risks_valid {
        artifact["risk_controls"] = json!([default_risk]);
    }

    if !artifact.get("evidence_refs").is_some_and(Value::is_object) {
        artifact["evidence_refs"] = json!({
            "skill_evidence": "normalizer-default:skill-evidence",
            "risk_evidence": "normalizer-default:risk-evidence",
            "artifact_version": "normalized-v1"
        });
        return;
    }

    let skill_evidence = artifact
        .get("evidence_refs")
        .and_then(|e| e.get("skill_evidence"))
        .and_then(Value::as_str)
        .is_some_and(|s| !s.trim().is_empty());
    if !skill_evidence {
        artifact["evidence_refs"]["skill_evidence"] =
            Value::String("normalizer-default:skill-evidence".to_string());
    }

    let risk_evidence = artifact
        .get("evidence_refs")
        .and_then(|e| e.get("risk_evidence"))
        .and_then(Value::as_str)
        .is_some_and(|s| !s.trim().is_empty());
    if !risk_evidence {
        artifact["evidence_refs"]["risk_evidence"] =
            Value::String("normalizer-default:risk-evidence".to_string());
    }
}

fn normalize_evidence(artifact: &mut Value) {
    if !artifact.get("evidence").is_some_and(Value::is_object) {
        artifact["evidence"] = json!({
            "metric_value": 0,
            "window": "unknown",
            "sample_size": 0,
            "source": "normalizer-default"
        });
        return;
    }

    if let Some(window) = artifact
        .get("evidence")
        .and_then(|e| e.get("window"))
        .and_then(Value::as_str)
    {
        if window.trim().is_empty() {
            artifact["evidence"]["window"] = Value::String("unknown".to_string());
        }
    } else {
        artifact["evidence"]["window"] = Value::String("unknown".to_string());
    }

    if let Some(source) = artifact
        .get("evidence")
        .and_then(|e| e.get("source"))
        .and_then(Value::as_str)
    {
        if source.trim().is_empty() {
            artifact["evidence"]["source"] = Value::String("unknown".to_string());
        }
    } else {
        artifact["evidence"]["source"] = Value::String("unknown".to_string());
    }

    let sample_number = artifact
        .get("evidence")
        .and_then(|e| e.get("sample_size"))
        .and_then(|v| {
            if let Some(n) = v.as_u64() {
                return Some(n);
            }
            if let Some(n) = v.as_i64() {
                return (n >= 0).then_some(n as u64);
            }
            if let Some(s) = v.as_str() {
                return s.trim().parse::<u64>().ok();
            }
            None
        })
        .unwrap_or(0);
    artifact["evidence"]["sample_size"] = Value::Number(sample_number.into());

    if artifact
        .get("evidence")
        .and_then(|e| e.get("metric_value"))
        .is_none()
    {
        artifact["evidence"]["metric_value"] = Value::Number(0.into());
    }
}

fn parse_next_role_flexible(raw: &str) -> Option<Role> {
    if let Some(role) = Role::from_key(raw) {
        return Some(role);
    }

    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.contains("pm") || normalized.contains("product") {
        return Some(Role::PM);
    }
    if normalized.contains("dev")
        || normalized.contains("architect")
        || normalized.contains("engineer")
    {
        return Some(Role::Dev);
    }
    if normalized.contains("qa") || normalized.contains("test") {
        return Some(Role::QA);
    }
    if normalized.contains("security") || normalized.contains("sec") {
        return Some(Role::Security);
    }
    if normalized.contains("sre") || normalized.contains("ops") || normalized.contains("devops") {
        return Some(Role::SRE);
    }
    None
}
