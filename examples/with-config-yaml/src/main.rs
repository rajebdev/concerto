use scheduled::{scheduled, SchedulerBuilder};

/// Task with interval from YAML config file (using shorthand with suffix)
#[scheduled(fixed_rate = "${app.interval}")]
async fn yaml_interval_task() {
    println!("🔄 [YAML CONFIG] Task running every 7 seconds from config");
}

/// Task with cron from YAML config and Jakarta timezone
#[scheduled(cron = "${app.cron_expression}", zone = "${app.zone}")]
async fn yaml_cron_task() {
    println!("⏰ [YAML CRON] Task running with cron from YAML config (Asia/Jakarta)");
}

/// Task using minutes from nested config (using TimeUnit constant)
#[scheduled(fixed_rate = "${app.report.interval}", time_unit = scheduled::TimeUnit::Minutes)]
async fn report_task() {
    println!("📊 [REPORT] Generating report every 30 minutes");
}

/// Task with default milliseconds (no time_unit specified)
#[scheduled(fixed_rate = 1500)]
async fn default_millis_task() {
    println!("⚡ [DEFAULT-MS] Running every 1500 milliseconds (1.5 seconds)");
}

/// Task with shorthand "3s" format
#[scheduled(fixed_rate = "3s")]
async fn three_seconds_shorthand() {
    println!("🚀 [3s] Running every 3 seconds (shorthand format)");
}

/// Task with shorthand "5m" format
#[scheduled(fixed_rate = "5m")]
async fn five_minutes_shorthand() {
    println!("📍 [5m] Running every 5 minutes (shorthand format)");
}

/// Task with shorthand "30s" format
#[scheduled(fixed_rate = "30s")]
async fn thirty_seconds_shorthand() {
    println!("⏱️  [30s] Running every 30 seconds (shorthand format)");
}

/// Task with hardcoded seconds time_unit
#[scheduled(fixed_rate = 8, time_unit = scheduled::TimeUnit::Seconds)]
async fn eight_second_task() {
    println!("🕐 [8-SEC] Running every 8 seconds (using TimeUnit::Seconds)");
}

/// Task with hardcoded hours time_unit
#[scheduled(fixed_rate = 2, time_unit = scheduled::TimeUnit::Hours)]
async fn cleanup_task() {
    println!("🧹 [CLEANUP] Running cleanup every 2 hours (using TimeUnit::Hours)");
}

/// Task with days time_unit
#[scheduled(fixed_rate = 1, time_unit = scheduled::TimeUnit::Days)]
async fn daily_summary_task() {
    println!("📅 [DAILY] Daily summary task (using TimeUnit::Days)");
}

/// Task that can be enabled/disabled via YAML config
#[scheduled(fixed_rate = 4000, enabled = "${app.task_enabled}")]
async fn yaml_conditional_task() {
    println!("✅ [YAML CONDITIONAL] Running every 4000ms (controlled by config)");
}

/// Task with enabled = true literal
#[scheduled(fixed_rate = 6000, enabled = true)]
async fn always_on_task() {
    println!("🟢 [ALWAYS-ON] This task is always enabled (enabled = true)");
}

/// Task with nested config value
#[scheduled(fixed_rate = 8000, enabled = "${app.notifications.enabled}")]
async fn notification_task() {
    println!("📧 [NOTIFICATIONS] Notifications task every 8000ms");
}

/// Task with default values if config is missing
#[scheduled(fixed_rate = "${app.backup_interval:15000}")]
async fn backup_task() {
    println!("💾 [BACKUP] Using default 15000ms interval (config key not found)");
}

/// Morning task at 8 AM Jakarta time
#[scheduled(cron = "0 0 8 * * *", zone = "Asia/Jakarta")]
async fn jakarta_morning_briefing() {
    println!("☕ [MORNING] Good morning briefing at 8 AM Jakarta time");
}

/// Afternoon task at 2 PM Jakarta time
#[scheduled(cron = "0 0 14 * * *", zone = "Asia/Jakarta")]
async fn jakarta_afternoon_sync() {
    println!("🔄 [AFTERNOON] Afternoon sync at 2 PM Jakarta time");
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
