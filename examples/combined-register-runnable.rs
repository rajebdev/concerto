use scheduled::{scheduled, Runnable, SchedulerBuilder};
use tokio::time::{sleep, Duration};
use chrono::Local;

// ============================================
// PART 1: Method-based scheduling with #[scheduled]
// ============================================

struct UserService {
    name: String,
}

#[scheduled]
impl UserService {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Runs every 5 seconds
    #[scheduled(fixed_rate = "5s")]
    async fn sync_users(&self) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] 🔄 [UserService::{}] Syncing users...", now, self.name);
    }

    /// Runs every 10 seconds
    #[scheduled(fixed_rate = "10s")]
    async fn cleanup_cache(&self) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] 🧹 [UserService::{}] Cleaning cache...", now, self.name);
    }
}

// ============================================
// PART 2: Trait-based scheduling with Runnable
// ============================================

struct DatabaseBackup {
    db_name: String,
}

impl DatabaseBackup {
    fn new(db_name: &str) -> Self {
        Self {
            db_name: db_name.to_string(),
        }
    }
}

#[scheduled(fixed_rate = "15s")]
impl Runnable for DatabaseBackup {
    fn run(&self) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] 💾 [DatabaseBackup] Backing up database: {}", now, self.db_name);
        std::thread::sleep(std::time::Duration::from_millis(500));
        println!("[{}] ✅ [DatabaseBackup] Backup completed for: {}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"), self.db_name);
    }
}

struct EmailSender {
    queue_size: usize,
}

impl EmailSender {
    fn new(queue_size: usize) -> Self {
        Self { queue_size }
    }
}

#[scheduled(fixed_delay = "20s")]
impl Runnable for EmailSender {
    fn run(&self) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] 📧 [EmailSender] Processing {} emails...", now, self.queue_size);
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("[{}] ✅ [EmailSender] Emails sent!", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"));
    }
}

// ============================================
// MAIN: Combining both approaches
// ============================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting combined scheduler example...\n");

    // Create instances
    let user_service = UserService::new("Production");
    let user_service_staging = UserService::new("Staging");
    let db_backup = DatabaseBackup::new("main_db");
    let email_sender = EmailSender::new(50);

    // Build scheduler with UNIFIED .register() for BOTH approaches
    let scheduler = SchedulerBuilder::new()
        // PART 1: Register instance with #[scheduled] methods
        .register(user_service)
        .register(user_service_staging)
        // PART 2: Register Runnable trait implementations (now also works with .register()!)
        .register(db_backup)
        .register(email_sender)
        .build();

    // Start the scheduler
    let _handle = scheduler.start().await?;
    
    println!("✅ Scheduler started with combined tasks!\n");
    println!("📋 Expected schedule:");
    println!("   • UserService::sync_users()    → every 5s  (method-based with #[scheduled])");
    println!("   • UserService::cleanup_cache() → every 10s (method-based with #[scheduled])");
    println!("   • DatabaseBackup               → every 15s (trait-based with fixed_rate)");
    println!("   • EmailSender                  → 20s delay after completion (trait-based with fixed_delay)");
    println!("\n⏳ Running for 60 seconds...\n");

    // Keep running
    sleep(Duration::from_secs(60)).await;

    println!("\n🛑 Shutting down...");
    Ok(())
}
