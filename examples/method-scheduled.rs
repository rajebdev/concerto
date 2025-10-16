/// Example: Using #[scheduled] on methods within impl block
/// This demonstrates how to mark methods with #[scheduled]
/// and the scheduler will auto-discover them when you register the instance.
use scheduled::{scheduled, SchedulerBuilder};
use chrono::Local;

/// User handler with multiple scheduled methods
struct UserHandler {
    name: String,
    counter: std::sync::atomic::AtomicU32,
}

#[scheduled]
impl UserHandler {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            counter: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// This method will run every 5 seconds
    #[scheduled(fixed_rate = "5s")]
    async fn exe(&self) {
        let count = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] [{}::exe] Execution #{} - Running every 5 seconds", now, self.name, count + 1);
    }

    /// This method will run every 10 seconds
    #[scheduled(fixed_rate = "10s")]
    async fn exe2(&self) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] [{}::exe2] Running every 10 seconds", now, self.name);
    }

    /// This method will run with fixed delay of 8 seconds after previous completion
    #[scheduled(fixed_delay = "8s")]
    async fn cleanup(&self) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] [{}::cleanup] Starting cleanup...", now, self.name);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        println!("[{}] [{}::cleanup] Cleanup completed!", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"), self.name);
    }

    /// Regular method without #[scheduled] - won't be auto-executed
    /// This demonstrates that not all methods in the impl block need to be scheduled.
    #[allow(dead_code)]
    async fn regular_method(&self) {
        println!("[{}] This is a regular method, not scheduled", self.name);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Method-Level Scheduled Example ===\n");

    // Create handler instance
    let user_handler = UserHandler::new("UserHandler-1");

    // Build scheduler and register handler
    // Only registered handler will have its scheduled methods executed
    let scheduler = SchedulerBuilder::new()
        .register(user_handler)   // Register: exe(), exe2(), cleanup() will run
        .build();

    // Start the scheduler
    let _handle = scheduler.start().await?;

    println!("\n✅ Scheduler started!");
    println!("📝 Scheduled methods:");
    println!("   - UserHandler::exe()     → every 5s");
    println!("   - UserHandler::exe2()    → every 10s");
    println!("   - UserHandler::cleanup() → 8s delay after completion");
    println!("\nPress Ctrl+C to stop...\n");

    // Keep running
    tokio::signal::ctrl_c().await?;
    println!("\n🛑 Shutting down...");

    Ok(())
}
