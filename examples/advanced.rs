use scheduled::{scheduled, SchedulerBuilder};
use chrono::Local;
use tracing_subscriber;

/// Task with cron expression - runs every minute
#[scheduled(cron = " * * * * *")]
async fn cron_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] [CRON] Every minute task", now);
}

/// Task with fixed rate from config (using shorthand)
#[scheduled(fixed_rate = "${app.interval:10s}")]
async fn config_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] [CONFIG] Task with interval from config", now);
}

/// Task with initial delay (using shorthand "5s")
#[scheduled(fixed_rate = "5s", initial_delay = 3)]
async fn delayed_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] [DELAYED] Task with 3s initial delay, runs every 5s", now);
}

/// Conditional task (can be disabled via config)
#[scheduled(fixed_rate = "7s", enabled = "${app.task_enabled:true}")]
async fn conditional_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] [CONDITIONAL] This task can be enabled/disabled", now);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with DEBUG level to see detailed logs
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "debug".to_string())
        )
        .with_target(false)
        .init();

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
