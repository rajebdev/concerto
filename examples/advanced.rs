use scheduled::{scheduled, SchedulerBuilder};

/// Task with cron expression - runs every minute
#[scheduled(cron = "0 * * * * *")]
async fn cron_task() {
    println!("[CRON] Every minute task");
}

/// Task with fixed rate from config (using shorthand)
#[scheduled(fixed_rate = "${app.interval:10s}")]
async fn config_task() {
    println!("[CONFIG] Task with interval from config");
}

/// Task with initial delay (using shorthand "5s")
#[scheduled(fixed_rate = "5s", initial_delay = 3, time_unit = "seconds")]
async fn delayed_task() {
    println!("[DELAYED] Task with 3s initial delay, runs every 5s");
}

/// Conditional task (can be disabled via config)
#[scheduled(fixed_rate = "7s", enabled = "${app.task_enabled:true}")]
async fn conditional_task() {
    println!("[CONDITIONAL] This task can be enabled/disabled");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting advanced example...\n");
    println!("📝 Configuration:");
    println!("   - app.interval: Controls config_task interval");
    println!("   - app.task_enabled: Enable/disable conditional_task");
    println!("\n");
    
    // Build scheduler (auto-discovers #[scheduled] functions)
    let scheduler = SchedulerBuilder::new().build();
    
    // Start the scheduler
    let _handle = scheduler.start().await?;
    
    println!("\n✅ Press Ctrl+C to stop.\n");
    
    tokio::signal::ctrl_c().await?;
    
    println!("\n👋 Shutting down...");
    Ok(())
}
