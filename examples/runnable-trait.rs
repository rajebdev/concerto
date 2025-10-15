use scheduled::{scheduled_impl, Runnable, SchedulerBuilder};
use std::pin::Pin;
use std::future::Future;

/// Example task struct
struct UserTask {
    name: String,
    counter: std::sync::atomic::AtomicU32,
}

impl UserTask {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            counter: std::sync::atomic::AtomicU32::new(0),
        }
    }
}

/// Implement Runnable with scheduling configuration
#[scheduled_impl(cron = "0 */5 * * * *")]
impl Runnable for UserTask {
    fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let count = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            println!("[UserTask] Running task '{}' - execution #{}", self.name, count + 1);
        })
    }
}

/// Another example with fixed rate
struct DatabaseCleanupTask {
    db_name: String,
}

impl DatabaseCleanupTask {
    fn new(db_name: &str) -> Self {
        Self {
            db_name: db_name.to_string(),
        }
    }
}

#[scheduled_impl(fixed_rate = "10s")]
impl Runnable for DatabaseCleanupTask {
    fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            println!("[DatabaseCleanupTask] Cleaning database: {}", self.db_name);
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            println!("[DatabaseCleanupTask] Cleanup completed for: {}", self.db_name);
        })
    }
}

/// Example with fixed delay
struct ReportGeneratorTask {
    report_type: String,
}

impl ReportGeneratorTask {
    fn new(report_type: &str) -> Self {
        Self {
            report_type: report_type.to_string(),
        }
    }
}

#[scheduled_impl(fixed_delay = "15s")]
impl Runnable for ReportGeneratorTask {
    fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            println!("[ReportGeneratorTask] Generating {} report...", self.report_type);
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            println!("[ReportGeneratorTask] Report generation completed!");
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Runnable Trait Example ===\n");

    // Create task instances
    let user_task = UserTask::new("MyUserTask");
    let cleanup_task = DatabaseCleanupTask::new("production_db");
    let report_task = ReportGeneratorTask::new("Monthly Sales");

    // Build scheduler with registered tasks
    // Only tasks registered via .runnable() will execute
    let _scheduler = SchedulerBuilder::with_toml("config/application.toml")?
        .runnable(user_task)      // Registers UserTask
        .runnable(cleanup_task)   // Registers DatabaseCleanupTask
        .runnable(report_task)    // Registers ReportGeneratorTask
        .start()                  // Starts the scheduler
        .await?;

    println!("\n✅ All tasks registered and scheduler started!");
    println!("📝 Tasks will run according to their schedules:");
    println!("   - UserTask: Every 5 minutes (cron)");
    println!("   - DatabaseCleanupTask: Every 10 seconds (fixed_rate)");
    println!("   - ReportGeneratorTask: 15 seconds after each completion (fixed_delay)");
    println!("\nPress Ctrl+C to stop...\n");

    // Keep the application running
    tokio::signal::ctrl_c().await?;
    println!("\n👋 Shutting down...");

    Ok(())
}
