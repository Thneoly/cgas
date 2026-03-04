use crate::error::EngineError;
use crate::model::{Role, RoleState, WorkflowContext};
use serde_json::Value;
use std::fs;

pub fn gate_pm_dev_qa_approved(ctx: &WorkflowContext) -> Result<(), EngineError> {
    let required = [Role::PM, Role::Dev, Role::QA];

    for role in required {
        let state = ctx
            .role_states
            .get(&role)
            .cloned()
            .unwrap_or(RoleState::Idle);
        if state != RoleState::Approved {
            return Err(EngineError::GateFailed(format!(
                "{} is not Approved",
                role.as_key()
            )));
        }
    }
    Ok(())
}

pub fn gate_security_or_exception(ctx: &WorkflowContext) -> Result<(), EngineError> {
    let security_state = ctx
        .role_states
        .get(&Role::Security)
        .cloned()
        .unwrap_or(RoleState::Idle);
    let has_exception = ctx.artifacts.contains_key("exception_approval");

    if security_state == RoleState::Approved || has_exception {
        return Ok(());
    }

    Err(EngineError::GateFailed(
        "Security not approved and exception_approval missing".to_string(),
    ))
}

pub fn gate_dispatch_audit_structured(ctx: &WorkflowContext) -> Result<(), EngineError> {
    let dispatch_events: Vec<_> = ctx
        .blackboard
        .events
        .iter()
        .filter(|event| event.event_type == "dispatch_decided")
        .collect();

    if dispatch_events.is_empty() {
        return Err(EngineError::GateFailed(
            "no dispatch_decided event found for audit".to_string(),
        ));
    }

    for event in dispatch_events {
        let audit = &event.audit;
        if !audit.is_object() {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit invalid: event_id={} audit is not object",
                event.id
            )));
        }

        let Some(rule_id) = audit.get("rule_id").and_then(Value::as_str) else {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit missing rule_id: event_id={}",
                event.id
            )));
        };
        if rule_id.trim().is_empty() {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit empty rule_id: event_id={}",
                event.id
            )));
        }

        let Some(trigger) = audit.get("trigger").and_then(Value::as_str) else {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit missing trigger: event_id={}",
                event.id
            )));
        };
        if trigger != "dispatch_decided" {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit trigger mismatch: event_id={} trigger={}",
                event.id, trigger
            )));
        }

        let Some(selected_role) = audit.get("selected_role").and_then(Value::as_str) else {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit missing selected_role: event_id={}",
                event.id
            )));
        };
        if selected_role.trim().is_empty() {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit empty selected_role: event_id={}",
                event.id
            )));
        }

        let Some(route_source) = audit.get("route_source").and_then(Value::as_str) else {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit missing route_source: event_id={}",
                event.id
            )));
        };
        if route_source.trim().is_empty() {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit empty route_source: event_id={}",
                event.id
            )));
        }

        let Some(candidates) = audit.get("candidates").and_then(Value::as_object) else {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit missing candidates: event_id={}",
                event.id
            )));
        };
        if !candidates.contains_key("artifact_next_role")
            || !candidates.contains_key("fallback_next_role")
        {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit candidates incomplete: event_id={}",
                event.id
            )));
        }

        let Some(evidence_refs) = audit.get("evidence_refs").and_then(Value::as_object) else {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit missing evidence_refs: event_id={}",
                event.id
            )));
        };

        if !evidence_refs
            .get("blackboard_event_id")
            .is_some_and(Value::is_number)
        {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit missing numeric blackboard_event_id: event_id={}",
                event.id
            )));
        }

        if !evidence_refs
            .get("artifact_type")
            .is_some_and(|v| v.as_str().is_some())
        {
            return Err(EngineError::GateFailed(format!(
                "dispatch audit missing artifact_type: event_id={}",
                event.id
            )));
        }

        if let Some(release_id) = audit.get("release_id").and_then(Value::as_str) {
            if release_id != ctx.release_id {
                return Err(EngineError::GateFailed(format!(
                    "dispatch audit release_id mismatch: event_id={} expected={} actual={}",
                    event.id, ctx.release_id, release_id
                )));
            }
        }
    }

    Ok(())
}

pub fn gate_phase2_readiness(_: &WorkflowContext) -> Result<(), EngineError> {
    let required_files = [
        "../doc/phase01/phase1_week6_gate_final_review.md",
        "../doc/phase01/phase1_week6_metrics_evidence_pack.md",
        "../doc/phase01/phase1_week6_closeout_report.md",
        "../doc/phase01/phase1_week6_gate_material_checklist.md",
        "../doc/phase01/phase1_week6_security_final_opinion.md",
    ];

    for file in required_files {
        let content = fs::read_to_string(file)
            .map_err(|e| EngineError::GateFailed(format!("phase2 readiness missing {}: {}", file, e)))?;

        if content.contains("{{") || content.contains("}}") {
            return Err(EngineError::GateFailed(format!(
                "phase2 readiness placeholder unresolved: {}",
                file
            )));
        }
    }

    let gate_final = fs::read_to_string("../doc/phase01/phase1_week6_gate_final_review.md")
        .map_err(|e| EngineError::GateFailed(format!("phase2 readiness gate final read failed: {}", e)))?;
    if !has_explicit_gate_decision(&gate_final) {
        return Err(EngineError::GateFailed(
            "phase2 readiness requires explicit Go/Conditional Go/No-Go conclusion"
                .to_string(),
        ));
    }

    Ok(())
}

fn has_explicit_gate_decision(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let patterns = [
        "decision: go",
        "decision: conditional_go",
        "decision: no_go",
        "decision: conditional go",
        "decision: no-go",
        "decision: no go",
        "phase1 gate 结论",
        "最终判定：go",
        "最终判定：conditional go",
        "最终判定：no-go",
        "gate 结论",
    ];

    patterns.iter().any(|p| lower.contains(p))
}
