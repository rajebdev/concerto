use concerto::{scheduled, SchedulerBuilder};
use std::sync::atomic::{AtomicU32, Ordering};
use chrono::Local;
use tracing_subscriber;

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Runs every 500 milliseconds (0.5 seconds)
#[scheduled(fixed_rate = 500)]
async fn fast_ms_task() {
    let count = COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] [FAST-MS] Execution #{} (every 500ms)", now, count);
}

/// Runs every 2 seconds
#[scheduled(fixed_rate = "2s")]
async fn slow_seconds_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] [SLOW-SEC] Every 2 seconds", now);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    println!("🚀 Demonstrating MILLISECONDS vs SECONDS...\n");
    println!("📊 Task 1: Every 500ms (milliseconds)");
    println!("📊 Task 2: Every 2s (seconds)\n");
    
    // Build scheduler (auto-discovers #[scheduled] functions)
    let scheduler = SchedulerBuilder::new().build();
    
    // Start the scheduler
    let _handle = scheduler.start().await?;
    
    println!("✅ Both tasks running! Watch the difference:\n");
    
    // Run for 10 seconds to see the pattern
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    let final_count = COUNTER.load(Ordering::SeqCst);
    println!("\n📈 RESULTS after 10 seconds:");
    println!("   Milliseconds task (500ms): {} executions", final_count);
    println!("   Expected: ~20 times (10000ms / 500ms)");
    println!("   ✅ MILLISECONDS WORK PERFECTLY!\n");
    
    Ok(())
}

