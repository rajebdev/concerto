use scheduled::{scheduled, Runnable, SchedulerBuilder};

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
/// This will run every 5 seconds
#[scheduled(fixed_rate = "5s")]
impl Runnable for UserTask {
    fn run(&self) {
        let count = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        println!("[UserTask] Running task '{}' - execution #{}", self.name, count + 1);
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
        println!("[DatabaseCleanupTask] Cleaning database: {}", self.db_name);
        std::thread::sleep(std::time::Duration::from_secs(2));
        println!("[DatabaseCleanupTask] Cleanup completed for: {}", self.db_name);
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
        println!("[ReportGeneratorTask] Generating {} report...", self.report_type);
        std::thread::sleep(std::time::Duration::from_secs(3));
        println!("[ReportGeneratorTask] Report generation completed!");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Runnable Trait Example ===\n");

    // Create task instances
    let user_task = UserTask::new("MyUserTask");
    let cleanup_task = DatabaseCleanupTask::new("production_db");
    let report_task = ReportGeneratorTask::new("Monthly Sales");

    // Build scheduler with registered tasks (pure setup, no errors)
    let scheduler = SchedulerBuilder::with_toml("config/application.toml")
        .register(user_task)      // Registers UserTask
        .register(cleanup_task)   // Registers DatabaseCleanupTask
        .register(report_task)    // Registers ReportGeneratorTask
        .build();                 // <- No ? needed, pure setup

    // Start the scheduler (this is where errors can happen)
    let _handle = scheduler.start().await?;

    println!("\n✅ All tasks registered and scheduler started!");
    println!("📝 Tasks will run according to their schedules:");
    println!("   - UserTask: Every 5 seconds (fixed_rate)");
    println!("   - DatabaseCleanupTask: Every 10 seconds (fixed_rate)");
    println!("   - ReportGeneratorTask: 15 seconds after each completion (fixed_delay)");
    println!("\nPress Ctrl+C to stop...\n");

    // Keep the application running
    tokio::signal::ctrl_c().await?;
    println!("\n👋 Shutting down...");

    Ok(())
}
