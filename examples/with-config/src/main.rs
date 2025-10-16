use concerto::{scheduled, SchedulerBuilder, TimeUnit};
use chrono::Local;

/// Task with interval from config file (using milliseconds by default)
#[scheduled(fixed_rate = "${app.interval}")]
async fn config_interval_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 🔄 [CONFIG] Task running every 5000ms (5 seconds) from config", now);
}

/// Task with cron from config and timezone
#[scheduled(cron = "${app.cron_expression}", zone = "${app.zone}")]
async fn config_cron_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ⏰ [CRON] Task running with cron from config (Asia/Jakarta timezone)", now);
}

/// Task using seconds from config (config value should include suffix like "3s")
#[scheduled(fixed_rate = "${app.fast_interval}")]
async fn fast_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ⚡ [FAST] Running every 3 seconds", now);
}

/// Task using minutes as time unit (config returns number, time_unit is compile-time)
#[scheduled(fixed_rate = "${app.backup_interval}", time_unit = TimeUnit::Minutes)]
async fn backup_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 💾 [BACKUP] Running backup every 2 minutes", now);
}

/// Task with default milliseconds (no time_unit specified)
#[scheduled(fixed_rate = 2000)]
async fn default_millis_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ⏱️  [DEFAULT-MS] This task runs every 2000 milliseconds (2 seconds)", now);
}

/// Task with shorthand "5s" format
#[scheduled(fixed_rate = "5s")]
async fn five_seconds_shorthand() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 🚀 [5s] This task runs every 5 seconds (shorthand format)", now);
}

/// Task with shorthand "2m" format
#[scheduled(fixed_rate = "2m")]
async fn two_minutes_shorthand() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 📍 [2m] This task runs every 2 minutes (shorthand format)", now);
}

/// Task with shorthand "1h" format
#[scheduled(fixed_rate = "1h")]
async fn one_hour_shorthand() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ⏰ [1h] This task runs every 1 hour (shorthand format)", now);
}

/// Task with shorthand "500ms" format
#[scheduled(fixed_rate = "500ms")]
async fn half_second_shorthand() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ⚡ [500ms] This task runs every 500 milliseconds (shorthand format)", now);
}

/// Task with TimeUnit::Seconds constant
#[scheduled(fixed_rate = 10, time_unit = TimeUnit::Seconds)]
async fn ten_second_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 🕐 [10-SEC] This task runs every 10 seconds (using TimeUnit::Seconds)", now);
}

/// Task with TimeUnit::Hours constant
#[scheduled(fixed_rate = 1, time_unit = TimeUnit::Hours)]
async fn hourly_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ⏳ [HOURLY] This task runs every 1 hour (using TimeUnit::Hours)", now);
}

/// Task with TimeUnit::Minutes constant
#[scheduled(fixed_rate = 5, time_unit = TimeUnit::Minutes)]
async fn five_minute_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ⏱️  [5-MIN] This task runs every 5 minutes (using TimeUnit::Minutes)", now);
}

/// Task with TimeUnit::Minutes (also works without concerto:: prefix)
#[scheduled(fixed_rate = 10, time_unit = TimeUnit::Minutes)]
async fn ten_minute_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 🕐 [10-MIN] This task runs every 10 minutes (using TimeUnit::Minutes)", now);
}

/// Task with TimeUnit::Days constant
#[scheduled(fixed_rate = 1, time_unit = TimeUnit::Days)]
async fn daily_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 📅 [DAILY] This task runs every 1 day (using TimeUnit::Days)", now);
}

/// Task with TimeUnit::Days (also works!)
#[scheduled(fixed_rate = 2, time_unit = TimeUnit::Days)]
async fn every_two_days_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 📆 [2-DAYS] This task runs every 2 days (using TimeUnit::Days)", now);
}

/// Task with boolean literal enabled = true
#[scheduled(fixed_rate = 3000, enabled = true)]
async fn always_enabled_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ✅ [ALWAYS-ON] This task is always enabled (enabled = true)", now);
}

/// Task with boolean literal enabled = false (will not be registered)
#[scheduled(fixed_rate = 2000, enabled = false)]
async fn disabled_task() {
    println!("❌ [DISABLED] This should never run");
}

/// Task that can be enabled/disabled via config
#[scheduled(fixed_rate = 3000, enabled = "${app.task_enabled}")]
async fn conditional_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ✅ [CONDITIONAL] Running every 3000ms (controlled by config)", now);
}

/// Task with default values if config is missing
#[scheduled(fixed_rate = "${app.fallback_interval:10000}")]
async fn task_with_defaults() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 🔧 [DEFAULT] Using default 10000ms interval (config not found)", now);
}

/// Task with Jakarta timezone for morning schedule
#[scheduled(cron = "0 0 9 * * *", zone = "Asia/Jakarta")]
async fn jakarta_morning_task() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] 🌅 [JAKARTA] Good morning! 9 AM in Jakarta", now);
}

/// ⚠️ WARNING EXAMPLE: time_unit on cron (will show warning but still works)
#[scheduled(cron = "0 */2 * * * *", time_unit = TimeUnit::Minutes)]
async fn cron_with_ignored_time_unit() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ⚠️  [CRON-WARN] This cron runs every 2 minutes (time_unit parameter is ignored)", now);
}

/// ⚠️ WARNING EXAMPLE: zone on interval (will show warning but still works)
#[scheduled(fixed_rate = "10s", zone = "Asia/Jakarta")]
async fn interval_with_ignored_zone() {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    println!("[{}] ⚠️  [INTERVAL-WARN] Runs every 10s (zone parameter is ignored for intervals)", now);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting scheduler with TOML configuration...\n");
    println!("📁 Config file: src/config/application.toml");
    println!("📝 Configuration values:");
    println!("   - app.interval: 5000 milliseconds (5 seconds)");
    println!("   - app.time_unit: milliseconds (default)");
    println!("   - app.zone: Asia/Jakarta");
    println!("   - app.cron_expression: '0 */2 * * * *'");
    println!("   - app.task_enabled: true");
    println!("   - app.fast_interval: 3 seconds");
    println!("   - app.backup_interval: 2 minutes");
    println!("\n💡 Note: Default time unit is milliseconds if not specified");
    println!("💡 You can use shorthand format: \"5s\", \"10m\", \"2h\", \"1d\", \"500ms\"");
    println!("💡 You can use TimeUnit::Days, TimeUnit::Hours, etc. enum variants");
    println!("💡 You can also use string literals: time_unit = \"days\", \"hours\", etc.");
    println!("💡 You can use boolean literals: enabled = true or enabled = false");
    println!("\n⚠️  WARNING Examples:");
    println!("   - time_unit on cron expressions will show warning (it's ignored)");
    println!("   - zone on interval tasks will show warning (it's ignored)");
    println!("\n");

    // Build scheduler with TOML config (auto-discovers #[scheduled] functions)
    let scheduler = SchedulerBuilder::with_toml("config/application.toml").build();
    
    // Start the scheduler
    let _handle = scheduler.start().await?;

    println!("\n✅ Scheduler started! Press Ctrl+C to stop.\n");

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c");

    println!("\n👋 Shutting down...");

    Ok(())
}



