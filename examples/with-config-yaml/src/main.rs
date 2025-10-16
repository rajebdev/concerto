use concerto::{scheduled, SchedulerBuilder};
use chrono::Local;

/// Task with interval from YAML config file (using shorthand with suffix)
#[scheduled(fixed_rate = "${app.interval}")]
async fn yaml_interval_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 🔄 [YAML CONFIG] Task running every 7 seconds from config", now);
}

/// Task with cron from YAML config and Jakarta timezone
#[scheduled(cron = "${app.cron_expression}", zone = "${app.zone}")]
async fn yaml_cron_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ⏰ [YAML CRON] Task running with cron from YAML config (Asia/Jakarta)", now);
}

/// Task using minutes from nested config (using TimeUnit constant)
#[scheduled(fixed_rate = "${app.report.interval}", time_unit = concerto::TimeUnit::Minutes)]
async fn report_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 📊 [REPORT] Generating report every 30 minutes", now);
}

/// Task with default milliseconds (no time_unit specified)
#[scheduled(fixed_rate = 1500)]
async fn default_millis_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ⚡ [DEFAULT-MS] Running every 1500 milliseconds (1.5 seconds)", now);
}

/// Task with shorthand "3s" format
#[scheduled(fixed_rate = "3s")]
async fn three_seconds_shorthand() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 🚀 [3s] Running every 3 seconds (shorthand format)", now);
}

/// Task with shorthand "5m" format
#[scheduled(fixed_rate = "5m")]
async fn five_minutes_shorthand() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 📍 [5m] Running every 5 minutes (shorthand format)", now);
}

/// Task with shorthand "30s" format
#[scheduled(fixed_rate = "30s")]
async fn thirty_seconds_shorthand() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ⏱️  [30s] Running every 30 seconds (shorthand format)", now);
}

/// Task with hardcoded seconds time_unit
#[scheduled(fixed_rate = 8, time_unit = concerto::TimeUnit::Seconds)]
async fn eight_second_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 🕐 [8-SEC] Running every 8 seconds (using TimeUnit::Seconds)", now);
}

/// Task with hardcoded hours time_unit
#[scheduled(fixed_rate = 2, time_unit = concerto::TimeUnit::Hours)]
async fn cleanup_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 🧹 [CLEANUP] Running cleanup every 2 hours (using TimeUnit::Hours)", now);
}

/// Task with days time_unit
#[scheduled(fixed_rate = 1, time_unit = concerto::TimeUnit::Days)]
async fn daily_summary_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 📅 [DAILY] Daily summary task (using TimeUnit::Days)", now);
}

/// Task that can be enabled/disabled via YAML config
#[scheduled(fixed_rate = 4000, enabled = "${app.task_enabled}")]
async fn yaml_conditional_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ✅ [YAML CONDITIONAL] Running every 4000ms (controlled by config)", now);
}

/// Task with enabled = true literal
#[scheduled(fixed_rate = 6000, enabled = true)]
async fn always_on_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 🟢 [ALWAYS-ON] This task is always enabled (enabled = true)", now);
}

/// Task with nested config value
#[scheduled(fixed_rate = 8000, enabled = "${app.notifications.enabled}")]
async fn notification_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 📧 [NOTIFICATIONS] Notifications task every 8000ms", now);
}

/// Task with default values if config is missing
#[scheduled(fixed_rate = "${app.backup_interval:15000}")]
async fn backup_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 💾 [BACKUP] Using default 15000ms interval (config key not found)", now);
}

/// Morning task at 8 AM Jakarta time
#[scheduled(cron = "0 0 8 * * *", zone = "Asia/Jakarta")]
async fn jakarta_morning_briefing() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ☕ [MORNING] Good morning briefing at 8 AM Jakarta time", now);
}

/// Afternoon task at 2 PM Jakarta time
#[scheduled(cron = "0 0 14 * * *", zone = "Asia/Jakarta")]
async fn jakarta_afternoon_sync() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 🔄 [AFTERNOON] Afternoon sync at 2 PM Jakarta time", now);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting scheduler with YAML configuration...\n");
    println!("📁 Config file: src/config/application.yaml");
    println!("📝 Configuration values:");
    println!("   - app.interval: 7000 milliseconds (7 seconds)");
    println!("   - app.time_unit: milliseconds (default)");
    println!("   - app.zone: Asia/Jakarta");
    println!("   - app.cron_expression: '0 */3 * * * *'");
    println!("   - app.task_enabled: true");
    println!("   - app.report.interval: 30 minutes");
    println!("   - app.notifications.enabled: true");
    println!("\n💡 Note: Default time unit is milliseconds if not specified");
    println!("💡 You can use shorthand format: \"5s\", \"10m\", \"2h\", \"1d\", \"500ms\"");
    println!("💡 You can use TimeUnit::Days, TimeUnit::Hours, etc. enum variants");
    println!("💡 You can use boolean literals: enabled = true or enabled = false");
    println!("\n");

    // Build scheduler with YAML config (auto-discovers #[scheduled] functions)
    let scheduler = SchedulerBuilder::with_yaml("config/application.yaml").build();
    
    // Start the scheduler
    let _handle = scheduler.start().await?;

    println!("\n✅ Scheduler started! Press Ctrl+C to stop.\n");

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c");

    println!("\n👋 Shutting down...");

    Ok(())
}


