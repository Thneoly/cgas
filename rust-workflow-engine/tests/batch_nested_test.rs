#![cfg(feature = "legacy-tests")]

// Phase 3: Batch Nested Execution Tests
// 20 test cases covering nested Batch functionality (depth 1-5)
// Reference: phase3_test_matrix_v3.md section 2.2

use workflow_engine::batch::{BatchCommand, BatchIsolationLevel};
use workflow_engine::command::Command;
use workflow_engine::context::ExecutionContext;
use workflow_engine::engine::WorkflowEngine;
use workflow_engine::result::ExecutionResult;

/// ============================================================================
/// Test Configuration
/// ============================================================================

const TEST_TIMEOUT_MS: u64 = 30_000;
const NESTED_MAX_DEPTH: u8 = 5;

/// ============================================================================
/// BN-001: Single Layer Batch Execution (Depth 1)
/// ============================================================================

#[tokio::test]
async fn test_bn_001_single_layer_batch_depth_1() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    // Create single layer batch with 3 commands
    let batch = BatchCommand {
        id: "batch_depth_1".to_string(),
        commands: vec![
            Command::create("resource_1", "value_1"),
            Command::update("resource_2", "value_2"),
            Command::delete("resource_3"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch(&mut ctx, batch).await;

    assert!(
        result.is_ok(),
        "Single layer batch should execute successfully"
    );
    assert_eq!(result.unwrap().success_count, 3);
}

/// ============================================================================
/// BN-002: Two Layer Batch Nested Execution (Depth 2)
/// ============================================================================

#[tokio::test]
async fn test_bn_002_two_layer_batch_nested_depth_2() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    // Create parent batch with nested child batch
    let child_batch = BatchCommand {
        id: "child_batch".to_string(),
        commands: vec![
            Command::create("child_resource_1", "child_value_1"),
            Command::update("child_resource_2", "child_value_2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::create("parent_resource_1", "parent_value_1"),
            Command::Batch(child_batch),
            Command::update("parent_resource_2", "parent_value_2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch(&mut ctx, parent_batch).await;

    assert!(
        result.is_ok(),
        "Two layer nested batch should execute successfully"
    );
    assert_eq!(result.unwrap().success_count, 5); // 2 parent + 2 child + 1 parent
}

/// ============================================================================
/// BN-003: Three Layer Batch Nested Execution (Depth 3)
/// ============================================================================

#[tokio::test]
async fn test_bn_003_three_layer_batch_nested_depth_3() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    // Create 3-level nested batch structure
    let grandchild_batch = BatchCommand {
        id: "grandchild_batch".to_string(),
        commands: vec![Command::create("gc_resource_1", "gc_value_1")],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let child_batch = BatchCommand {
        id: "child_batch".to_string(),
        commands: vec![
            Command::create("child_resource_1", "child_value_1"),
            Command::Batch(grandchild_batch),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::create("parent_resource_1", "parent_value_1"),
            Command::Batch(child_batch),
            Command::update("parent_resource_2", "parent_value_2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch(&mut ctx, parent_batch).await;

    assert!(
        result.is_ok(),
        "Three layer nested batch should execute successfully"
    );
    assert_eq!(result.unwrap().success_count, 5);
}

/// ============================================================================
/// BN-004: Four Layer Batch Nested Execution (Depth 4)
/// ============================================================================

#[tokio::test]
async fn test_bn_004_four_layer_batch_nested_depth_4() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    // Create 4-level nested batch structure
    let level4_batch = BatchCommand {
        id: "level4_batch".to_string(),
        commands: vec![Command::create("l4_r1", "l4_v1")],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let level3_batch = BatchCommand {
        id: "level3_batch".to_string(),
        commands: vec![
            Command::create("l3_r1", "l3_v1"),
            Command::Batch(level4_batch),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let level2_batch = BatchCommand {
        id: "level2_batch".to_string(),
        commands: vec![
            Command::create("l2_r1", "l2_v1"),
            Command::Batch(level3_batch),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let level1_batch = BatchCommand {
        id: "level1_batch".to_string(),
        commands: vec![
            Command::create("l1_r1", "l1_v1"),
            Command::Batch(level2_batch),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch(&mut ctx, level1_batch).await;

    assert!(
        result.is_ok(),
        "Four layer nested batch should execute successfully"
    );
    assert_eq!(result.unwrap().success_count, 4);
}

/// ============================================================================
/// BN-005: Five Layer Batch Nested Execution (Depth 5 - Max)
/// ============================================================================

#[tokio::test]
async fn test_bn_005_five_layer_batch_nested_depth_5_max() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    // Create 5-level nested batch structure (maximum allowed)
    let mut current_batch = BatchCommand {
        id: "level5_batch".to_string(),
        commands: vec![Command::create("l5_r1", "l5_v1")],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    // Build from inner to outer
    for level in (1..=4).rev() {
        current_batch = BatchCommand {
            id: format!("level{}_batch", level),
            commands: vec![
                Command::create(format!("l{}_r1", level), format!("l{}_v1", level)),
                Command::Batch(current_batch),
            ],
            isolation: BatchIsolationLevel::Sequential,
            max_depth: NESTED_MAX_DEPTH,
        };
    }

    let result = engine.execute_batch(&mut ctx, current_batch).await;

    assert!(
        result.is_ok(),
        "Five layer nested batch (max depth) should execute successfully"
    );
    assert_eq!(result.unwrap().success_count, 5);
}

/// ============================================================================
/// BN-006: Exceeded Nested Depth Rejection (Depth 6)
/// ============================================================================

#[tokio::test]
async fn test_bn_006_exceeded_nested_depth_rejection_depth_6() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    // Create 6-level nested batch structure (exceeds max depth of 5)
    let mut current_batch = BatchCommand {
        id: "level6_batch".to_string(),
        commands: vec![Command::create("l6_r1", "l6_v1")],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    // Build from inner to outer (6 levels)
    for level in (1..=5).rev() {
        current_batch = BatchCommand {
            id: format!("level{}_batch", level),
            commands: vec![
                Command::create(format!("l{}_r1", level), format!("l{}_v1", level)),
                Command::Batch(current_batch),
            ],
            isolation: BatchIsolationLevel::Sequential,
            max_depth: NESTED_MAX_DEPTH,
        };
    }

    let result = engine.execute_batch(&mut ctx, current_batch).await;

    assert!(result.is_err(), "Six layer nested batch should be rejected");
    assert_eq!(
        result.unwrap_err().code,
        "NESTED_DEPTH_EXCEEDED",
        "Error code should indicate depth exceeded"
    );
}

/// ============================================================================
/// BN-007: Nested Batch Independent Scope
/// ============================================================================

#[tokio::test]
async fn test_bn_007_nested_batch_independent_scope() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    // Child batch creates variable that should not leak to parent
    let child_batch = BatchCommand {
        id: "child_batch".to_string(),
        commands: vec![
            Command::set_variable("child_var", "child_value"),
            Command::create("child_resource", "child_data"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::set_variable("parent_var", "parent_value"),
            Command::Batch(child_batch),
            // Try to access child_var - should fail or return default
            Command::get_variable("child_var"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch(&mut ctx, parent_batch).await;

    assert!(
        result.is_ok(),
        "Nested batch should execute with scope isolation"
    );
    // Verify child_var is not accessible in parent scope
    let execution_result = result.unwrap();
    assert!(
        execution_result.variable_access.get("child_var").is_none()
            || execution_result.variable_access.get("child_var") == Some(&None),
        "Child variables should not leak to parent scope"
    );
}

/// ============================================================================
/// BN-008: Nested Batch Error Isolation
/// ============================================================================

#[tokio::test]
async fn test_bn_008_nested_batch_error_isolation() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    // Child batch with failing command
    let failing_child = BatchCommand {
        id: "failing_child".to_string(),
        commands: vec![
            Command::create("child_r1", "child_v1"),
            Command::fail("intentional_failure"), // This will fail
            Command::create("child_r2", "child_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let sibling_child = BatchCommand {
        id: "sibling_child".to_string(),
        commands: vec![
            Command::create("sibling_r1", "sibling_v1"),
            Command::create("sibling_r2", "sibling_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::create("parent_r1", "parent_v1"),
            Command::Batch(failing_child),
            Command::Batch(sibling_child), // Should still execute
            Command::create("parent_r2", "parent_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch(&mut ctx, parent_batch).await;

    // Parent should continue executing after failing child (depending on error handling mode)
    assert!(
        result.is_ok(),
        "Parent batch should handle child failure gracefully"
    );
}

/// ============================================================================
/// BN-009: Nested Result Aggregation
/// ============================================================================

#[tokio::test]
async fn test_bn_009_nested_result_aggregation() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    let child_batch = BatchCommand {
        id: "child_batch".to_string(),
        commands: vec![
            Command::create("child_r1", "child_v1"),
            Command::create("child_r2", "child_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::create("parent_r1", "parent_v1"),
            Command::Batch(child_batch),
            Command::create("parent_r2", "parent_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch(&mut ctx, parent_batch).await;

    assert!(result.is_ok());
    let execution_result = result.unwrap();

    // Parent should have access to all child results
    assert!(
        execution_result.has_result("child_r1"),
        "Parent should access child result child_r1"
    );
    assert!(
        execution_result.has_result("child_r2"),
        "Parent should access child result child_r2"
    );
    assert_eq!(execution_result.total_commands, 5);
}

/// ============================================================================
/// BN-010: Nested Execution Log Level Identification
/// ============================================================================

#[tokio::test]
async fn test_bn_010_nested_execution_log_level_identification() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    let child_batch = BatchCommand {
        id: "child_batch".to_string(),
        commands: vec![Command::create("child_r1", "child_v1")],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::create("parent_r1", "parent_v1"),
            Command::Batch(child_batch),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch(&mut ctx, parent_batch).await;

    assert!(result.is_ok());
    let execution_result = result.unwrap();

    // Verify log contains level identification
    let log = execution_result.execution_log;
    assert!(
        log.contains("[depth=0]"),
        "Log should contain parent level identifier"
    );
    assert!(
        log.contains("[depth=1]"),
        "Log should contain child level identifier"
    );
}

/// ============================================================================
/// BN-011: Nested Performance Overhead (Single Layer)
/// ============================================================================

#[tokio::test]
async fn test_bn_011_nested_performance_overhead_single_layer() {
    let engine = WorkflowEngine::new();

    // Baseline: non-nested batch
    let baseline_batch = BatchCommand {
        id: "baseline".to_string(),
        commands: (0..100)
            .map(|i| Command::create(format!("r_{}", i), format!("v_{}", i)))
            .collect(),
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    // Nested: single layer with same commands
    let nested_batch = BatchCommand {
        id: "nested".to_string(),
        commands: vec![Command::Batch(BatchCommand {
            id: "child".to_string(),
            commands: (0..100)
                .map(|i| Command::create(format!("r_{}", i), format!("v_{}", i)))
                .collect(),
            isolation: BatchIsolationLevel::Sequential,
            max_depth: NESTED_MAX_DEPTH,
        })],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let baseline_start = std::time::Instant::now();
    let _ = engine
        .execute_batch(&mut ExecutionContext::default(), baseline_batch)
        .await;
    let baseline_duration = baseline_start.elapsed();

    let nested_start = std::time::Instant::now();
    let _ = engine
        .execute_batch(&mut ExecutionContext::default(), nested_batch)
        .await;
    let nested_duration = nested_start.elapsed();

    let overhead =
        (nested_duration.as_millis() as f64 / baseline_duration.as_millis() as f64 - 1.0) * 100.0;

    assert!(
        overhead < 5.0,
        "Single layer nested overhead should be <5%, actual: {:.2}%",
        overhead
    );
}

/// ============================================================================
/// BN-012: Nested Performance Overhead (Five Layers)
/// ============================================================================

#[tokio::test]
async fn test_bn_012_nested_performance_overhead_five_layers() {
    let engine = WorkflowEngine::new();

    // Baseline: non-nested batch with 100 commands
    let baseline_batch = BatchCommand {
        id: "baseline".to_string(),
        commands: (0..100)
            .map(|i| Command::create(format!("r_{}", i), format!("v_{}", i)))
            .collect(),
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    // Build 5-level nested structure with 20 commands per level (total 100)
    let mut nested_batch = BatchCommand {
        id: "level5".to_string(),
        commands: (0..20)
            .map(|i| Command::create(format!("l5_r_{}", i), format!("l5_v_{}", i)))
            .collect(),
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    for level in (1..=4).rev() {
        let mut commands: Vec<Command> = (0..20)
            .map(|i| Command::create(format!("l{}_r_{}", level, i), format!("l{}_v_{}", level, i)))
            .collect();
        commands.push(Command::Batch(nested_batch));
        nested_batch = BatchCommand {
            id: format!("level{}", level),
            commands,
            isolation: BatchIsolationLevel::Sequential,
            max_depth: NESTED_MAX_DEPTH,
        };
    }

    let baseline_start = std::time::Instant::now();
    let _ = engine
        .execute_batch(&mut ExecutionContext::default(), baseline_batch)
        .await;
    let baseline_duration = baseline_start.elapsed();

    let nested_start = std::time::Instant::now();
    let _ = engine
        .execute_batch(&mut ExecutionContext::default(), nested_batch)
        .await;
    let nested_duration = nested_start.elapsed();

    let overhead =
        (nested_duration.as_millis() as f64 / baseline_duration.as_millis() as f64 - 1.0) * 100.0;

    assert!(
        overhead < 25.0,
        "Five layer nested overhead should be <25%, actual: {:.2}%",
        overhead
    );
}

/// ============================================================================
/// BN-013: Nested Batch Atomicity (atomic=true)
/// ============================================================================

#[tokio::test]
async fn test_bn_013_nested_batch_atomicity_true() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    let child_batch = BatchCommand {
        id: "child_batch".to_string(),
        commands: vec![
            Command::create("child_r1", "child_v1"),
            Command::fail("intentional_failure"), // Will fail
            Command::create("child_r2", "child_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::create("parent_r1", "parent_v1"),
            Command::Batch(child_batch),
            Command::create("parent_r2", "parent_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch_atomic(&mut ctx, parent_batch).await;

    // With atomic=true, all or nothing
    assert!(result.is_err(), "Atomic batch should fail on child failure");
    // Verify rollback occurred
    assert!(
        ctx.get_resource("parent_r1").is_none(),
        "Atomic batch should rollback parent on child failure"
    );
}

/// ============================================================================
/// BN-014: Nested Batch Non-Atomic (atomic=false)
/// ============================================================================

#[tokio::test]
async fn test_bn_014_nested_batch_non_atomic_false() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    let child_batch = BatchCommand {
        id: "child_batch".to_string(),
        commands: vec![
            Command::create("child_r1", "child_v1"),
            Command::fail("intentional_failure"),
            Command::create("child_r2", "child_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::create("parent_r1", "parent_v1"),
            Command::Batch(child_batch),
            Command::create("parent_r2", "parent_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch(&mut ctx, parent_batch).await;

    // Non-atomic allows partial success
    assert!(
        result.is_ok(),
        "Non-atomic batch should allow partial success"
    );
    assert!(
        ctx.get_resource("parent_r1").is_some(),
        "Non-atomic batch should keep successful parent commands"
    );
}

/// ============================================================================
/// BN-015: Nested Batch Replay Consistency
/// ============================================================================

#[tokio::test]
async fn test_bn_015_nested_batch_replay_consistency() {
    let engine = WorkflowEngine::new();

    let child_batch = BatchCommand {
        id: "child_batch".to_string(),
        commands: vec![
            Command::create("child_r1", "child_v1"),
            Command::update("child_r2", "child_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::create("parent_r1", "parent_v1"),
            Command::Batch(child_batch),
            Command::update("parent_r2", "parent_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    // Execute multiple times and verify consistency
    let mut results = Vec::new();
    for _ in 0..10 {
        let mut ctx = ExecutionContext::default();
        let result = engine.execute_batch(&mut ctx, parent_batch.clone()).await;
        results.push(result.is_ok());
    }

    let success_count = results.iter().filter(|&&r| r).count();
    let consistency_rate = success_count as f64 / results.len() as f64 * 100.0;

    assert!(
        consistency_rate >= 99.97,
        "Nested batch replay consistency should be >=99.97%, actual: {:.2}%",
        consistency_rate
    );
}

/// ============================================================================
/// BN-016: Nested Batch Hash Verification
/// ============================================================================

#[tokio::test]
async fn test_bn_016_nested_batch_hash_verification() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    let child_batch = BatchCommand {
        id: "child_batch".to_string(),
        commands: vec![
            Command::create("child_r1", "child_v1"),
            Command::update("child_r2", "child_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::create("parent_r1", "parent_v1"),
            Command::Batch(child_batch),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch(&mut ctx, parent_batch).await;

    assert!(result.is_ok());
    let execution_result = result.unwrap();

    // Verify hash chain is valid
    assert!(
        execution_result.verify_hash_chain(),
        "Nested batch hash chain should be valid"
    );
}

/// ============================================================================
/// BN-017: Nested Batch Concurrent Execution
/// ============================================================================

#[tokio::test]
async fn test_bn_017_nested_batch_concurrent_execution() {
    let engine = WorkflowEngine::new();

    let child_batch = BatchCommand {
        id: "child_batch".to_string(),
        commands: vec![
            Command::create("child_r1", "child_v1"),
            Command::update("child_r2", "child_v2"),
        ],
        isolation: BatchIsolationLevel::Parallel,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::create("parent_r1", "parent_v1"),
            Command::Batch(child_batch),
        ],
        isolation: BatchIsolationLevel::Parallel,
        max_depth: NESTED_MAX_DEPTH,
    };

    // Execute concurrently multiple times
    let mut handles = Vec::new();
    for _ in 0..10 {
        let engine_clone = engine.clone();
        let batch_clone = parent_batch.clone();
        let handle = tokio::spawn(async move {
            let mut ctx = ExecutionContext::default();
            engine_clone.execute_batch(&mut ctx, batch_clone).await
        });
        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;
    let success_count = results
        .iter()
        .filter(|r| r.as_ref().ok().and_then(|r| r.as_ref().ok()).is_some())
        .count();

    assert_eq!(
        success_count, 10,
        "All concurrent executions should succeed without deadlock"
    );
}

/// ============================================================================
/// BN-018: Nested Batch Rollback Verification
/// ============================================================================

#[tokio::test]
async fn test_bn_018_nested_batch_rollback_verification() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    let failing_child = BatchCommand {
        id: "failing_child".to_string(),
        commands: vec![
            Command::create("child_r1", "child_v1"),
            Command::fail("rollback_trigger"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::create("parent_r1", "parent_v1"),
            Command::Batch(failing_child),
            Command::create("parent_r2", "parent_v2"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let _ = engine.execute_batch_atomic(&mut ctx, parent_batch).await;

    // Verify all resources were rolled back
    assert!(
        ctx.get_resource("parent_r1").is_none(),
        "Parent resource should be rolled back"
    );
    assert!(
        ctx.get_resource("child_r1").is_none(),
        "Child resource should be rolled back"
    );
}

/// ============================================================================
/// BN-019: Nested Batch Hybrid Mode
/// ============================================================================

#[tokio::test]
async fn test_bn_019_nested_batch_hybrid_mode() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    let child_batch = BatchCommand {
        id: "child_batch".to_string(),
        commands: vec![
            Command::create("child_r1", "child_v1"),
            Command::create("child_r2", "child_v2"),
        ],
        isolation: BatchIsolationLevel::Parallel,
        max_depth: NESTED_MAX_DEPTH,
    };

    let parent_batch = BatchCommand {
        id: "parent_batch".to_string(),
        commands: vec![
            Command::create("parent_r1", "parent_v1"),
            Command::Batch(child_batch),
            Command::create("parent_r2", "parent_v2"),
        ],
        isolation: BatchIsolationLevel::Hybrid,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch(&mut ctx, parent_batch).await;

    assert!(
        result.is_ok(),
        "Hybrid mode nested batch should execute successfully"
    );
    assert_eq!(result.unwrap().success_count, 4);
}

/// ============================================================================
/// BN-020: Nested Batch Backward Compatibility
/// ============================================================================

#[tokio::test]
async fn test_bn_020_nested_batch_backward_compatibility() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();

    // Phase 2 style single-layer batch (no nesting)
    let phase2_batch = BatchCommand {
        id: "phase2_batch".to_string(),
        commands: vec![
            Command::create("resource_1", "value_1"),
            Command::update("resource_2", "value_2"),
            Command::delete("resource_3"),
        ],
        isolation: BatchIsolationLevel::Sequential,
        max_depth: NESTED_MAX_DEPTH,
    };

    let result = engine.execute_batch(&mut ctx, phase2_batch).await;

    assert!(
        result.is_ok(),
        "Phase 2 single-layer batch should still work"
    );
    assert_eq!(result.unwrap().success_count, 3);
    // Verify behavior matches Phase 2 expectations
}

// Note: individual batch nested tests above are declared with #[tokio::test].
// Keep them independent and avoid calling test functions from another test.
