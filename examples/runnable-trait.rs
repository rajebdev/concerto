use scheduled::{scheduled, Runnable, SchedulerBuilder};
use chrono::Local;

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
#[scheduled(cron = "0 */5 * * * *")]
impl Runnable for UserTask {
    fn run(&self) {
        let count = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] [UserTask] Running task '{}' - execution #{}", now, self.name, count + 1);
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

#[scheduled(fixed_rate = "10s")]
impl Runnable for DatabaseCleanupTask {
    fn run(&self) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] [DatabaseCleanupTask] Cleaning database: {}", now, self.db_name);
        std::thread::sleep(std::time::Duration::from_secs(2));
        println!("[{}] [DatabaseCleanupTask] Cleanup completed for: {}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"), self.db_name);
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

#[scheduled(fixed_delay = "15s")]
impl Runnable for ReportGeneratorTask {
    fn run(&self) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] [ReportGeneratorTask] Generating {} report...", now, self.report_type);
        std::thread::sleep(std::time::Duration::from_secs(3));
        println!("[{}] [ReportGeneratorTask] Report generation completed!", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"));
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
    // Only tasks registered via .register() will execute
    let scheduler = SchedulerBuilder::new()
        .register(user_task)      // Registers UserTask
        .register(cleanup_task)   // Registers DatabaseCleanupTask
        .register(report_task)    // Registers ReportGeneratorTask
        .build();                 // Build the scheduler
    
    // Start the scheduler
    let _handle = scheduler.start().await?;

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
