use scheduled::{scheduled, SchedulerBuilder};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

static COUNTER_10MS: AtomicU32 = AtomicU32::new(0);
static COUNTER_100MS: AtomicU32 = AtomicU32::new(0);
static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// VERY FAST: Runs every 10ms
#[scheduled(fixed_rate = 10)]
async fn super_fast_10ms() {
    let count = COUNTER_10MS.fetch_add(1, Ordering::SeqCst) + 1;
    let elapsed = START_TIME.get().unwrap().elapsed().as_millis();
    if count % 10 == 0 || count <= 5 {  // Print every 10th to avoid spam
        println!("[{}ms] 10ms task #{}", elapsed, count);
    }
}

/// FAST: Runs every 100ms for comparison
#[scheduled(fixed_rate = 100)]
async fn fast_100ms() {
    let count = COUNTER_100MS.fetch_add(1, Ordering::SeqCst) + 1;
    let elapsed = START_TIME.get().unwrap().elapsed().as_millis();
    println!("[{}ms] 100ms task #{}", elapsed, count);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    START_TIME.set(Instant::now()).ok();
    
    println!("=== Testing 10ms Interval ===\n");
    println!("Task 1: Every 10ms (super fast!)");
    println!("Task 2: Every 100ms (comparison)\n");
    
    let start = Instant::now();
    
    // Build scheduler (auto-discovers #[scheduled] functions)
    let scheduler = SchedulerBuilder::new().build();
    
    // Start the scheduler
    let _handle = scheduler.start().await?;
    
    let init_time = start.elapsed();
    println!("✅ Scheduler initialized in {:?}\n", init_time);
    println!("✅ Running for exactly 3 seconds...\n");
    
    // Run for exactly 3 seconds
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    let total_time = start.elapsed();
    let count_10ms = COUNTER_10MS.load(Ordering::SeqCst);
    let count_100ms = COUNTER_100MS.load(Ordering::SeqCst);
    
    println!("\n=== RESULTS ===");
    println!("Total runtime: {:?}", total_time);
    println!("Initialization overhead: {:?}", init_time);
    println!("Actual task runtime: {:?}", total_time - init_time);
    println!("\n10ms task:  {} executions (expected ~300)", count_10ms);
    println!("100ms task: {} executions (expected ~30)", count_100ms);
    
    // Calculate accuracy
    let actual_runtime_ms = (total_time - init_time).as_millis() as f64;
    let expected_10ms = (actual_runtime_ms / 10.0) as u32;
    let expected_100ms = (actual_runtime_ms / 100.0) as u32;
    let accuracy_10ms = (count_10ms as f64 / expected_10ms as f64) * 100.0;
    let accuracy_100ms = (count_100ms as f64 / expected_100ms as f64) * 100.0;
    
    println!("\nAccuracy:");
    println!("  10ms task:  {:.2}% ({} / {})", accuracy_10ms, count_10ms, expected_10ms);
    println!("  100ms task: {:.2}% ({} / {})", accuracy_100ms, count_100ms, expected_100ms);
    
    println!("\nConclusion: 10ms interval {} work!",
        if count_10ms > 250 { "DOES" } else { "may not" });
    
    Ok(())
}
