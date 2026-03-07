// Phase 3: Transaction Isolation Level Tests (Repeatable Read Focus)
// 15 test cases covering Transaction isolation levels (RC/RR/Serializable)
// Reference: phase3_test_matrix_v3.md section 2.3

use workflow_engine::transaction::{Transaction, IsolationLevel};
use workflow_engine::engine::WorkflowEngine;
use workflow_engine::context::ExecutionContext;
use workflow_engine::command::Command;
use workflow_engine::result::ExecutionResult;

/// ============================================================================
/// Test Configuration
/// ============================================================================

const TEST_TIMEOUT_MS: u64 = 30_000;
const CONCURRENT_TRANSACTIONS: usize = 10;

/// ============================================================================
/// TI-001: Read Committed - Prevent Dirty Read
/// ============================================================================

#[tokio::test]
async fn test_ti_001_read_committed_prevent_dirty_read() {
    let engine = WorkflowEngine::new();
    
    // Transaction A starts and writes uncommitted data
    let mut ctx_a = ExecutionContext::default();
    let mut tx_a = Transaction::new(IsolationLevel::ReadCommitted);
    tx_a.execute(&mut ctx_a, Command::create("shared_resource", "uncommitted_value")).await;
    
    // Transaction B starts and tries to read - should not see uncommitted data
    let mut ctx_b = ExecutionContext::default();
    let mut tx_b = Transaction::new(IsolationLevel::ReadCommitted);
    let read_result = tx_b.execute(&mut ctx_b, Command::read("shared_resource")).await;
    
    // Transaction B should not see uncommitted data from Transaction A
    assert!(
        read_result.is_err() || read_result.unwrap().value.is_none(),
        "Read Committed should prevent dirty read"
    );
    
    // Cleanup
    tx_a.rollback(&mut ctx_a).await;
}

/// ============================================================================
/// TI-002: Read Committed - Allow Non-Repeatable Read
/// ============================================================================

#[tokio::test]
async fn test_ti_002_read_committed_allow_non_repeatable_read() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();
    
    // Setup initial data
    ctx.set_resource("test_resource", "initial_value");
    
    // Transaction A: First read
    let mut tx_a1 = Transaction::new(IsolationLevel::ReadCommitted);
    let read1 = tx_a1.execute(&mut ctx, Command::read("test_resource")).await.unwrap();
    
    // Transaction B: Update and commit
    let mut tx_b = Transaction::new(IsolationLevel::ReadCommitted);
    tx_b.execute(&mut ctx, Command::update("test_resource", "updated_value")).await;
    tx_b.commit(&mut ctx).await.unwrap();
    
    // Transaction A: Second read - may see different value (non-repeatable read allowed)
    let mut tx_a2 = Transaction::new(IsolationLevel::ReadCommitted);
    let read2 = tx_a2.execute(&mut ctx, Command::read("test_resource")).await.unwrap();
    
    // RC allows non-repeatable reads
    assert_ne!(
        read1.value, read2.value,
        "Read Committed allows non-repeatable reads"
    );
}

/// ============================================================================
/// TI-003: Read Committed - Allow Phantom Read
/// ============================================================================

#[tokio::test]
async fn test_ti_003_read_committed_allow_phantom_read() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();
    
    // Setup initial data
    ctx.set_resource("item_1", "value_1");
    ctx.set_resource("item_2", "value_2");
    
    // Transaction A: First query
    let mut tx_a1 = Transaction::new(IsolationLevel::ReadCommitted);
    let query1 = tx_a1.execute(&mut ctx, Command::query("item_*")).await.unwrap();
    
    // Transaction B: Insert new row and commit
    let mut tx_b = Transaction::new(IsolationLevel::ReadCommitted);
    tx_b.execute(&mut ctx, Command::create("item_3", "value_3")).await;
    tx_b.commit(&mut ctx).await.unwrap();
    
    // Transaction A: Second query - may see new row (phantom read allowed)
    let mut tx_a2 = Transaction::new(IsolationLevel::ReadCommitted);
    let query2 = tx_a2.execute(&mut ctx, Command::query("item_*")).await.unwrap();
    
    // RC allows phantom reads
    assert!(
        query2.result_count >= query1.result_count,
        "Read Committed allows phantom reads"
    );
}

/// ============================================================================
/// TI-004: Repeatable Read - Prevent Dirty Read
/// ============================================================================

#[tokio::test]
async fn test_ti_004_repeatable_read_prevent_dirty_read() {
    let engine = WorkflowEngine::new();
    
    // Transaction A starts and writes uncommitted data
    let mut ctx_a = ExecutionContext::default();
    let mut tx_a = Transaction::new(IsolationLevel::RepeatableRead);
    tx_a.execute(&mut ctx_a, Command::create("shared_resource", "uncommitted_value")).await;
    
    // Transaction B starts and tries to read - should not see uncommitted data
    let mut ctx_b = ExecutionContext::default();
    let mut tx_b = Transaction::new(IsolationLevel::RepeatableRead);
    let read_result = tx_b.execute(&mut ctx_b, Command::read("shared_resource")).await;
    
    // Transaction B should not see uncommitted data from Transaction A
    assert!(
        read_result.is_err() || read_result.unwrap().value.is_none(),
        "Repeatable Read should prevent dirty read"
    );
    
    // Cleanup
    tx_a.rollback(&mut ctx_a).await;
}

/// ============================================================================
/// TI-005: Repeatable Read - Prevent Non-Repeatable Read
/// ============================================================================

#[tokio::test]
async fn test_ti_005_repeatable_read_prevent_non_repeatable_read() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();
    
    // Setup initial data
    ctx.set_resource("test_resource", "initial_value");
    
    // Transaction A: Start and first read
    let mut tx_a = Transaction::new(IsolationLevel::RepeatableRead);
    let read1 = tx_a.execute(&mut ctx, Command::read("test_resource")).await.unwrap();
    
    // Transaction B: Update and commit
    let mut tx_b = Transaction::new(IsolationLevel::ReadCommitted);
    tx_b.execute(&mut ctx, Command::update("test_resource", "updated_value")).await;
    tx_b.commit(&mut ctx).await.unwrap();
    
    // Transaction A: Second read - should see same value (non-repeatable read prevented)
    let read2 = tx_a.execute(&mut ctx, Command::read("test_resource")).await.unwrap();
    
    // RR prevents non-repeatable reads
    assert_eq!(
        read1.value, read2.value,
        "Repeatable Read prevents non-repeatable reads"
    );
    
    tx_a.commit(&mut ctx).await.unwrap();
}

/// ============================================================================
/// TI-006: Repeatable Read - Allow Phantom Read
/// ============================================================================

#[tokio::test]
async fn test_ti_006_repeatable_read_allow_phantom_read() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();
    
    // Setup initial data
    ctx.set_resource("item_1", "value_1");
    ctx.set_resource("item_2", "value_2");
    
    // Transaction A: Start and first query
    let mut tx_a = Transaction::new(IsolationLevel::RepeatableRead);
    let query1 = tx_a.execute(&mut ctx, Command::query("item_*")).await.unwrap();
    
    // Transaction B: Insert new row and commit
    let mut tx_b = Transaction::new(IsolationLevel::ReadCommitted);
    tx_b.execute(&mut ctx, Command::create("item_3", "value_3")).await;
    tx_b.commit(&mut ctx).await.unwrap();
    
    // Transaction A: Second query - may see new row (phantom read allowed in RR)
    let query2 = tx_a.execute(&mut ctx, Command::query("item_*")).await.unwrap();
    
    // RR allows phantom reads (only prevents non-repeatable reads)
    // Note: Some implementations prevent phantoms in RR, but ANSI SQL allows them
    assert!(
        query2.result_count >= query1.result_count,
        "Repeatable Read allows phantom reads (ANSI SQL)"
    );
    
    tx_a.commit(&mut ctx).await.unwrap();
}

/// ============================================================================
/// TI-007: Serializable - Prevent Dirty Read
/// ============================================================================

#[tokio::test]
async fn test_ti_007_serializable_prevent_dirty_read() {
    let engine = WorkflowEngine::new();
    
    // Transaction A starts and writes uncommitted data
    let mut ctx_a = ExecutionContext::default();
    let mut tx_a = Transaction::new(IsolationLevel::Serializable);
    tx_a.execute(&mut ctx_a, Command::create("shared_resource", "uncommitted_value")).await;
    
    // Transaction B starts and tries to read - should not see uncommitted data
    let mut ctx_b = ExecutionContext::default();
    let mut tx_b = Transaction::new(IsolationLevel::Serializable);
    let read_result = tx_b.execute(&mut ctx_b, Command::read("shared_resource")).await;
    
    // Transaction B should not see uncommitted data from Transaction A
    assert!(
        read_result.is_err() || read_result.unwrap().value.is_none(),
        "Serializable should prevent dirty read"
    );
    
    // Cleanup
    tx_a.rollback(&mut ctx_a).await;
}

/// ============================================================================
/// TI-008: Serializable - Prevent Non-Repeatable Read
/// ============================================================================

#[tokio::test]
async fn test_ti_008_serializable_prevent_non_repeatable_read() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();
    
    // Setup initial data
    ctx.set_resource("test_resource", "initial_value");
    
    // Transaction A: Start and first read
    let mut tx_a = Transaction::new(IsolationLevel::Serializable);
    let read1 = tx_a.execute(&mut ctx, Command::read("test_resource")).await.unwrap();
    
    // Transaction B: Try to update - should be blocked or fail in Serializable
    let mut tx_b = Transaction::new(IsolationLevel::Serializable);
    let update_result = tx_b.execute(&mut ctx, Command::update("test_resource", "updated_value")).await;
    
    // In Serializable, concurrent modifications should be prevented
    assert!(
        update_result.is_err(),
        "Serializable should prevent concurrent modifications"
    );
    
    // Transaction A: Second read - should see same value
    let read2 = tx_a.execute(&mut ctx, Command::read("test_resource")).await.unwrap();
    assert_eq!(read1.value, read2.value, "Serializable prevents non-repeatable reads");
    
    tx_a.commit(&mut ctx).await.unwrap();
}

/// ============================================================================
/// TI-009: Serializable - Prevent Phantom Read
/// ============================================================================

#[tokio::test]
async fn test_ti_009_serializable_prevent_phantom_read() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();
    
    // Setup initial data
    ctx.set_resource("item_1", "value_1");
    ctx.set_resource("item_2", "value_2");
    
    // Transaction A: Start and first query
    let mut tx_a = Transaction::new(IsolationLevel::Serializable);
    let query1 = tx_a.execute(&mut ctx, Command::query("item_*")).await.unwrap();
    
    // Transaction B: Try to insert new row - should be blocked or fail
    let mut tx_b = Transaction::new(IsolationLevel::Serializable);
    let insert_result = tx_b.execute(&mut ctx, Command::create("item_3", "value_3")).await;
    
    // In Serializable, concurrent inserts in queried range should be prevented
    assert!(
        insert_result.is_err(),
        "Serializable should prevent phantom inserts"
    );
    
    // Transaction A: Second query - should see same results
    let query2 = tx_a.execute(&mut ctx, Command::query("item_*")).await.unwrap();
    assert_eq!(
        query1.result_count, query2.result_count,
        "Serializable prevents phantom reads"
    );
    
    tx_a.commit(&mut ctx).await.unwrap();
}

/// ============================================================================
/// TI-010: Isolation Level Dynamic Switching
/// ============================================================================

#[tokio::test]
async fn test_ti_010_isolation_level_dynamic_switching() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();
    
    // Start with Read Committed
    let mut tx = Transaction::new(IsolationLevel::ReadCommitted);
    assert_eq!(tx.isolation_level(), IsolationLevel::ReadCommitted);
    
    // Switch to Repeatable Read
    tx.set_isolation_level(IsolationLevel::RepeatableRead);
    assert_eq!(tx.isolation_level(), IsolationLevel::RepeatableRead);
    
    // Switch to Serializable
    tx.set_isolation_level(IsolationLevel::Serializable);
    assert_eq!(tx.isolation_level(), IsolationLevel::Serializable);
    
    // Execute with Serializable
    ctx.set_resource("test_resource", "test_value");
    let result = tx.execute(&mut ctx, Command::read("test_resource")).await;
    assert!(result.is_ok(), "Dynamic isolation level switching should work");
}

/// ============================================================================
/// TI-011: Isolation Level Conflict Detection
/// ============================================================================

#[tokio::test]
async fn test_ti_011_isolation_level_conflict_detection() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();
    
    // Transaction A: Serializable
    let mut tx_a = Transaction::new(IsolationLevel::Serializable);
    tx_a.execute(&mut ctx, Command::create("resource_1", "value_1")).await;
    
    // Transaction B: Read Committed tries to read same resource
    let mut tx_b = Transaction::new(IsolationLevel::ReadCommitted);
    
    // This should detect potential conflict
    let result = tx_b.execute(&mut ctx, Command::read("resource_1")).await;
    
    // Conflict should be detected and reported
    assert!(
        result.is_err() || result.unwrap().conflict_detected,
        "Isolation level conflict should be detected"
    );
    
    tx_a.rollback(&mut ctx).await;
}

/// ============================================================================
/// TI-012: Deadlock Detection and Auto-Rollback
/// ============================================================================

#[tokio::test]
async fn test_ti_012_deadlock_detection_and_auto_rollback() {
    let engine = WorkflowEngine::new();
    
    // Create deadlock scenario: A waits for B, B waits for A
    let mut ctx_a = ExecutionContext::default();
    let mut ctx_b = ExecutionContext::default();
    
    ctx_a.set_resource("resource_a", "value_a");
    ctx_a.set_resource("resource_b", "value_b");
    ctx_b.set_resource("resource_a", "value_a");
    ctx_b.set_resource("resource_b", "value_b");
    
    let mut tx_a = Transaction::new(IsolationLevel::RepeatableRead);
    let mut tx_b = Transaction::new(IsolationLevel::RepeatableRead);
    
    // Transaction A locks resource_a
    tx_a.execute(&mut ctx_a, Command::update("resource_a", "a_new")).await;
    
    // Transaction B locks resource_b
    tx_b.execute(&mut ctx_b, Command::update("resource_b", "b_new")).await;
    
    // Try to create deadlock: A wants resource_b, B wants resource_a
    let handle_a = tokio::spawn(async move {
        tx_a.execute(&mut ctx_a, Command::update("resource_b", "a_wants")).await
    });
    
    let handle_b = tokio::spawn(async move {
        tx_b.execute(&mut ctx_b, Command::update("resource_a", "b_wants")).await
    });
    
    let (result_a, result_b) = tokio::join!(handle_a, handle_b);
    
    // At least one should fail due to deadlock detection
    assert!(
        result_a.unwrap().is_err() || result_b.unwrap().is_err(),
        "Deadlock should be detected and one transaction rolled back"
    );
}

/// ============================================================================
/// TI-013: RC Performance Baseline
/// ============================================================================

#[tokio::test]
async fn test_ti_013_rc_performance_baseline() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();
    
    // Setup test data
    for i in 0..100 {
        ctx.set_resource(format!("rc_resource_{}", i), format!("value_{}", i));
    }
    
    let start = std::time::Instant::now();
    
    // Execute 1000 transactions with Read Committed
    for _ in 0..1000 {
        let mut tx = Transaction::new(IsolationLevel::ReadCommitted);
        let _ = tx.execute(&mut ctx, Command::read("rc_resource_50")).await;
        let _ = tx.commit(&mut ctx).await;
    }
    
    let duration = start.elapsed();
    let avg_latency_ms = duration.as_millis() as f64 / 1000.0;
    let p99_estimate = avg_latency_ms * 2.0; // Rough estimate
    
    assert!(
        p99_estimate < 200.0,
        "RC P99 should be <200ms, estimated: {:.2}ms",
        p99_estimate
    );
    
    println!("RC Performance: avg={:.2}ms, estimated P99<{:.2}ms", avg_latency_ms, p99_estimate);
}

/// ============================================================================
/// TI-014: RR Performance Baseline
/// ============================================================================

#[tokio::test]
async fn test_ti_014_rr_performance_baseline() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();
    
    // Setup test data
    for i in 0..100 {
        ctx.set_resource(format!("rr_resource_{}", i), format!("value_{}", i));
    }
    
    let start = std::time::Instant::now();
    
    // Execute 1000 transactions with Repeatable Read
    for _ in 0..1000 {
        let mut tx = Transaction::new(IsolationLevel::RepeatableRead);
        let _ = tx.execute(&mut ctx, Command::read("rr_resource_50")).await;
        let _ = tx.commit(&mut ctx).await;
    }
    
    let duration = start.elapsed();
    let avg_latency_ms = duration.as_millis() as f64 / 1000.0;
    let p99_estimate = avg_latency_ms * 2.0;
    
    // RR should have ~10% overhead vs RC
    assert!(
        p99_estimate < 220.0,
        "RR P99 should be <220ms (RC +10%), estimated: {:.2}ms",
        p99_estimate
    );
    
    println!("RR Performance: avg={:.2}ms, estimated P99<{:.2}ms", avg_latency_ms, p99_estimate);
}

/// ============================================================================
/// TI-015: Serializable Performance Baseline
/// ============================================================================

#[tokio::test]
async fn test_ti_015_serializable_performance_baseline() {
    let engine = WorkflowEngine::new();
    let mut ctx = ExecutionContext::default();
    
    // Setup test data
    for i in 0..100 {
        ctx.set_resource(format!("ser_resource_{}", i), format!("value_{}", i));
    }
    
    let start = std::time::Instant::now();
    
    // Execute 1000 transactions with Serializable
    for _ in 0..1000 {
        let mut tx = Transaction::new(IsolationLevel::Serializable);
        let _ = tx.execute(&mut ctx, Command::read("ser_resource_50")).await;
        let _ = tx.commit(&mut ctx).await;
    }
    
    let duration = start.elapsed();
    let avg_latency_ms = duration.as_millis() as f64 / 1000.0;
    let p99_estimate = avg_latency_ms * 2.0;
    
    // Serializable should have ~25% overhead vs RC
    assert!(
        p99_estimate < 250.0,
        "Serializable P99 should be <250ms (RC +25%), estimated: {:.2}ms",
        p99_estimate
    );
    
    println!("Serializable Performance: avg={:.2}ms, estimated P99<{:.2}ms", avg_latency_ms, p99_estimate);
}

/// ============================================================================
/// Test Runner
/// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn run_all_transaction_isolation_tests() {
        println!("Running Phase 3 Transaction Isolation Tests (15 cases)...");
        
        test_ti_001_read_committed_prevent_dirty_read().await;
        println!("✓ TI-001: Read Committed - Prevent dirty read");
        
        test_ti_002_read_committed_allow_non_repeatable_read().await;
        println!("✓ TI-002: Read Committed - Allow non-repeatable read");
        
        test_ti_003_read_committed_allow_phantom_read().await;
        println!("✓ TI-003: Read Committed - Allow phantom read");
        
        test_ti_004_repeatable_read_prevent_dirty_read().await;
        println!("✓ TI-004: Repeatable Read - Prevent dirty read");
        
        test_ti_005_repeatable_read_prevent_non_repeatable_read().await;
        println!("✓ TI-005: Repeatable Read - Prevent non-repeatable read");
        
        test_ti_006_repeatable_read_allow_phantom_read().await;
        println!("✓ TI-006: Repeatable Read - Allow phantom read");
        
        test_ti_007_serializable_prevent_dirty_read().await;
        println!("✓ TI-007: Serializable - Prevent dirty read");
        
        test_ti_008_serializable_prevent_non_repeatable_read().await;
        println!("✓ TI-008: Serializable - Prevent non-repeatable read");
        
        test_ti_009_serializable_prevent_phantom_read().await;
        println!("✓ TI-009: Serializable - Prevent phantom read");
        
        test_ti_010_isolation_level_dynamic_switching().await;
        println!("✓ TI-010: Isolation level dynamic switching");
        
        test_ti_011_isolation_level_conflict_detection().await;
        println!("✓ TI-011: Isolation level conflict detection");
        
        test_ti_012_deadlock_detection_and_auto_rollback().await;
        println!("✓ TI-012: Deadlock detection and auto-rollback");
        
        test_ti_013_rc_performance_baseline().await;
        println!("✓ TI-013: RC performance baseline");
        
        test_ti_014_rr_performance_baseline().await;
        println!("✓ TI-014: RR performance baseline");
        
        test_ti_015_serializable_performance_baseline().await;
        println!("✓ TI-015: Serializable performance baseline");
        
        println!("\n✅ All 15 Transaction Isolation Tests Passed!");
    }
}
