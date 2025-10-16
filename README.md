# scheduled-rs

Spring Boot-like task scheduling for Rust with configuration support.

## Features

- 🚀 **Simple API** - Annotation-based scheduling similar to Spring Boot's `@Scheduled`
- ⏰ **Cron Support** - Full cron expression support with timezone
- 🔄 **Fixed Rate & Delay** - Schedule tasks at fixed intervals or delays
- ⚙️ **Config Integration** - Read scheduling parameters from config files using `${...}` placeholders
- 🎯 **Conditional Execution** - Enable/disable tasks via configuration
- ⏳ **Initial Delay** - Delay the first execution of a task
- 🕐 **Time Units** - Support for milliseconds, seconds, minutes, hours, and days
- 🌍 **Timezone Support** - Specify timezone for cron expressions (e.g., "Asia/Jakarta")
- 🔧 **Environment Variables** - Override config with environment variables

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
scheduled = { path = "./scheduled" }
tokio = { version = "1", features = ["full"] }
```

## Quick Start

### Option 1: Function-based Tasks (Auto-discovery)

```rust
use scheduled::{scheduled, SchedulerBuilder};

// Run every 5 minutes
#[scheduled(cron = "0 */5 * * * *")]
async fn every_five_minutes() {
    println!("Executing scheduled task!");
}

// Run every 30 seconds
#[scheduled(fixed_rate = "30s")]
async fn every_30_seconds() {
    println!("Fixed rate task!");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build scheduler (auto-discovers #[scheduled] functions)
    let scheduler = SchedulerBuilder::new().build();
    
    // Start the scheduler
    let handle = scheduler.start().await?;
    
    // Keep running until Ctrl+C
    tokio::signal::ctrl_c().await?;
    
    // Graceful shutdown
    handle.shutdown().await?;
    Ok(())
}
```

### Option 2: Struct-based Tasks (Manual registration)

```rust
use scheduled::{scheduled, Runnable, SchedulerBuilder};
use std::pin::Pin;
use std::future::Future;

struct MyTask {
    name: String,
}

// Macro provides metadata (schedule info)
#[scheduled(fixed_rate = "5s")]
impl Runnable for MyTask {
    fn run(&self) {
        println!("Task {} is running!", self.name);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let task = MyTask { name: "MyTask".to_string() };
    
    // Build scheduler and register task manually
    let scheduler = SchedulerBuilder::new()
        .register(task)  // Manual registration with unified API
        .build();        // No ? needed - pure setup
    
    // Start the scheduler
    let handle = scheduler.start().await?;  // Errors happen here
    
    tokio::signal::ctrl_c().await?;
    handle.shutdown().await?;
    Ok(())
}
```

### Option 3: Mixed (Both patterns)

```rust
// Auto-discovered function
#[scheduled(fixed_rate = "10s")]
async fn background_task() {
    println!("Auto task");
}

// Manual struct
#[scheduled(fixed_rate = "5s")]
impl Runnable for MyTask {
    fn run(&self) {
        println!("Manual task");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let task = MyTask::new();
    
    // Combine both
    let scheduler = SchedulerBuilder::new()
        .register(task)     // Manual task with unified API
        .build();           // + auto-discovered functions
    
    let handle = scheduler.start().await?;
    tokio::signal::ctrl_c().await?;
    handle.shutdown().await?;
    Ok(())
}
```

## Configuration

Create `config/application.toml`:

```toml
[app]
interval = "60"

[app.schedule]
cron = "0 0 * * * *"
enabled = true
```

Or use environment variables:

```bash
export APP_INTERVAL=120
export APP_SCHEDULE_CRON="0 */10 * * * *"
export APP_SCHEDULE_ENABLED=false
```

## Scheduling Options

### Cron Expression

```rust
// Basic cron (uses local timezone)
#[scheduled(cron = "0 */5 * * * *")]
async fn every_5_minutes() {
    // Runs every 5 minutes
}

// Cron with specific timezone
#[scheduled(cron = "0 0 9 * * *", zone = "Asia/Jakarta")]
async fn jakarta_morning() {
    // Runs at 9:00 AM Jakarta time every day
}

// Cron with config placeholder
#[scheduled(cron = "${app.cron}", zone = "${app.zone:local}")]
async fn config_cron() {
    // Reads cron expression and timezone from config
}
```

Cron format: `second minute hour day month weekday`

**Note:** The `zone` parameter **only works with cron expressions**. It is ignored for `fixed_rate` and `fixed_delay` tasks, which always use local system time.

**Timezone examples:**
- `"Asia/Jakarta"` - Jakarta, Indonesia
- `"UTC"` - Coordinated Universal Time
- `"America/New_York"` - Eastern Time
- `"Europe/London"` - British Time
- `"local"` - System local time (default)

### Fixed Rate

Execute at fixed intervals (doesn't wait for completion):

```rust
// Default: milliseconds
#[scheduled(fixed_rate = 30000)]
async fn every_30_seconds() {
    // Runs every 30000 milliseconds (30 seconds)
}

// Using time_unit with string
#[scheduled(fixed_rate = 30, time_unit = "seconds")]
async fn every_30_seconds_v2() {
    // Runs every 30 seconds
}

// Using time_unit with enum
use scheduled::TimeUnit;

#[scheduled(fixed_rate = 5, time_unit = TimeUnit::Minutes)]
async fn every_5_minutes() {
    // Runs every 5 minutes
}

#[scheduled(fixed_rate = 1, time_unit = TimeUnit::Hours)]
async fn every_hour() {
    // Runs every hour
}

#[scheduled(fixed_rate = 1, time_unit = TimeUnit::Days)]
async fn daily_task() {
    // Runs every day
}
```

**Supported time units:**
- `milliseconds` (default), `ms`, `millis`
- `seconds`, `s`, `sec`
- `minutes`, `m`, `min`
- `hours`, `h`, `hr`
- `days`, `d`, `day`

### Fixed Delay

Execute with fixed delay between completions:

```rust
#[scheduled(fixed_delay = 30)]
async fn fixed_delay_task() {
    // Waits 30 seconds after completion
}
```

### Initial Delay

Delay the first execution:

```rust
#[scheduled(fixed_rate = 60, initial_delay = 10)]
async fn delayed_start() {
    // First execution after 10 seconds, then every 60 seconds
}
```

### Conditional Execution

Enable/disable via config or boolean literal:

```rust
// Using config placeholder
#[scheduled(fixed_rate = 30, enabled = "${feature.enabled}")]
async fn conditional_task() {
    // Only runs if feature.enabled = true in config
}

// Using boolean literal
#[scheduled(fixed_rate = 60, enabled = true)]
async fn always_enabled() {
    // Always runs
}

#[scheduled(fixed_rate = 60, enabled = false)]
async fn always_disabled() {
    // Never runs (useful for temporarily disabling tasks)
}
```

### Config Placeholders

Use `${key}` syntax to read from configuration:

```rust
// With default value
#[scheduled(cron = "${app.cron:0 * * * * *}")]
async fn with_default() {
    // Uses config value or defaults to every minute
}

// From nested config
#[scheduled(fixed_rate = "${app.schedule.interval}")]
async fn nested_config() {
    // Reads app.schedule.interval from config
}
```

## Advanced Usage

### Custom Configuration

```rust
use scheduled::{SchedulerBuilder};
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let custom_config = Config::builder()
        .add_source(config::File::with_name("my_config"))
        .build()?;

    // Build with custom config
    let scheduler = SchedulerBuilder::with_config(custom_config).build();
    
    // Start scheduler
    let handle = scheduler.start().await?;

    tokio::signal::ctrl_c().await?;
    handle.shutdown().await?;
    Ok(())
}
```

### Using TOML/YAML Config Files

```rust
use scheduled::{SchedulerBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Option 1: TOML config (panics on error - fail fast during setup)
    let scheduler = SchedulerBuilder::with_toml("config/application.toml")
        .build();
    
    // Option 2: YAML config (panics on error - fail fast during setup)
    let scheduler = SchedulerBuilder::with_yaml("config/application.yaml")
        .build();
    
    let handle = scheduler.start().await?;
    tokio::signal::ctrl_c().await?;
    handle.shutdown().await?;
    Ok(())
}
```

### Scheduler Builder API

```rust
// All available methods:
let scheduler = SchedulerBuilder::new()                             // Default config
    // OR
    .with_toml("config.toml")                                       // TOML config (no ?)
    // OR
    .with_yaml("config.yaml")                                       // YAML config (no ?)
    // OR
    .with_config(custom_config)                                     // Custom Config
    
    // Register runnable instances (optional, can chain multiple)
    .register(task1)
    .register(task2)
    .register(task3)
    
    .build();                                                       // Build (no ?)

// Start the scheduler
let handle = scheduler.start().await?;                              // Start (can error)

// Graceful shutdown
handle.shutdown().await?;
```

## Examples

### Example 1: Basic Scheduling

```rust
use scheduled::scheduled;

#[scheduled(fixed_rate = 10)]
async fn health_check() {
    println!("Health check at: {}", chrono::Local::now());
}

#[scheduled(cron = "0 0 2 * * *")]
async fn daily_cleanup() {
    println!("Running daily cleanup...");
    // Cleanup logic here
}
```

### Example 2: Config-Driven Tasks

`config/application.toml`:
```toml
[app.jobs]
health_check_interval = "30"
backup_cron = "0 0 3 * * *"
sync_enabled = true
```

`main.rs`:
```rust
#[scheduled(fixed_rate = "${app.jobs.health_check_interval}")]
async fn health_check() {
    // Runs every 30 seconds from config
}

#[scheduled(cron = "${app.jobs.backup_cron}")]
async fn backup() {
    // Runs at 3 AM daily
}

#[scheduled(fixed_rate = 60, enabled = "${app.jobs.sync_enabled}")]
async fn sync_data() {
    // Only runs if enabled in config
}
```

### Example 3: Async Operations

```rust
#[scheduled(fixed_rate = 60)]
async fn fetch_external_data() {
    match reqwest::get("https://api.example.com/data").await {
        Ok(response) => {
            println!("Fetched: {:?}", response.text().await);
        }
        Err(e) => {
            eprintln!("Error fetching data: {}", e);
        }
    }
}

#[scheduled(fixed_delay = 30)]
async fn process_queue() {
    // Process items
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    println!("Queue processed");
    // Next execution will start 30 seconds after this completes
}
```

## Compiler Warnings

This library performs compile-time validation and may emit warnings for common misconfigurations. These warnings indicate that your code will compile and run, but some parameters will be ignored.

### Warning Codes

#### **W001: time_unit Parameter Ignored (Suffix Present)**

**Cause:** Both a time suffix (`s`, `m`, `h`, etc.) and `time_unit` parameter are specified.

```rust
// ❌ This will emit W001
#[scheduled(fixed_rate = "5s", time_unit = TimeUnit::Minutes)]
//                       ^^^ has suffix    ^^^^^^^^^^^^^^^^^^^ will be ignored
async fn my_task() { }
```

**Why:** The suffix in the value takes precedence over the `time_unit` parameter.

**Fix Options:**
```rust
// ✅ Option 1: Remove suffix
#[scheduled(fixed_rate = "5", time_unit = TimeUnit::Seconds)]

// ✅ Option 2: Remove time_unit (recommended)
#[scheduled(fixed_rate = "5s")]

// ✅ Option 3: Use config with time_unit
#[scheduled(fixed_rate = "${app.interval:5}", time_unit = TimeUnit::Seconds)]
```

---

#### **W002: time_unit Parameter Ignored (Cron Expression)**

**Cause:** `time_unit` parameter is specified for a cron-based schedule.

```rust
// ❌ This will emit W002
#[scheduled(cron = "0 */5 * * * *", time_unit = TimeUnit::Seconds)]
//                                  ^^^^^^^^^^^^^^^^^^^^^^^^ ignored
async fn my_task() { }
```

**Why:** Cron expressions use absolute calendar time, not intervals. The `time_unit` parameter only applies to interval-based schedules (`fixed_rate`, `fixed_delay`).

**Fix:**
```rust
// ✅ Remove time_unit
#[scheduled(cron = "0 */5 * * * *")]
async fn my_task() { }
```

---

#### **W003: zone Parameter Ignored (Interval-Based Schedule)**

**Cause:** `zone` parameter is specified for `fixed_rate` or `fixed_delay` schedule.

```rust
// ❌ This will emit W003
#[scheduled(fixed_rate = "5s", zone = "Asia/Jakarta")]
//                             ^^^^^^^^^^^^^^^^^^^^^^ ignored
async fn my_task() { }
```

**Why:** Interval-based tasks (`fixed_rate`, `fixed_delay`) always use local system time. Timezones only apply to cron expressions which are calendar-based.

**Fix:**
```rust
// ✅ Remove zone for intervals
#[scheduled(fixed_rate = "5s")]

// ✅ Or use cron if you need timezone
#[scheduled(cron = "0 */5 * * * *", zone = "Asia/Jakarta")]
```

---

### Compile Errors vs Warnings

| Condition | Type | Reason |
|-----------|------|--------|
| Parameter ignored (suffix + time_unit) | ⚠️ **Warning W001** | Non-fatal: code works, parameter ignored |
| time_unit for cron | ⚠️ **Warning W002** | Non-fatal: code works, parameter not applicable |
| zone for interval | ⚠️ **Warning W003** | Non-fatal: code works, parameter not applicable |
| Malformed config placeholder (`"${xxx"`) | ❌ **Compile Error** | Fatal: will crash at runtime |
| Config + suffix mix (`"${app.val}s"`) | ❌ **Compile Error** | Fatal: ambiguous behavior |
| Invalid time suffix (`"5S"` uppercase) | ❌ **Compile Error** | Fatal: invalid format |
| Negative or zero interval | ❌ **Compile Error** | Fatal: would cause infinite loop |
| time_unit as config placeholder | ❌ **Compile Error** | Fatal: must be compile-time constant |

---

## Cron Expression Format

```
 ┌─────────── second (0 - 59)
 │ ┌───────── minute (0 - 59)
 │ │ ┌─────── hour (0 - 23)
 │ │ │ ┌───── day of month (1 - 31)
 │ │ │ │ ┌─── month (1 - 12)
 │ │ │ │ │ ┌─ day of week (0 - 6) (Sunday to Saturday)
 │ │ │ │ │ │
 * * * * * *
```

### Common Cron Examples

- `0 * * * * *` - Every minute
- `0 */5 * * * *` - Every 5 minutes
- `0 0 * * * *` - Every hour
- `0 0 */6 * * *` - Every 6 hours
- `0 0 0 * * *` - Daily at midnight
- `0 0 2 * * *` - Daily at 2 AM
- `0 0 0 * * 1` - Every Monday at midnight
- `0 0 0 1 * *` - First day of every month
- `0 30 9 * * 1-5` - Weekdays at 9:30 AM

## Project Structure

```
scheduled-rs/
├── Cargo.toml                 # Workspace definition
├── README.md
├── scheduled/                 # Main library
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── scheduled-macro/           # Procedural macro
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── scheduled-runtime/         # Runtime & scheduler
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── examples/
│   └── main.rs               # Example usage
└── config/
    └── application.toml      # Configuration file
```

## How It Works

1. **Macro Expansion**: The `#[scheduled]` macro generates a registration function for each annotated task
2. **Auto-Registration**: Using `linkme`, all tasks are collected into a distributed slice at compile time
3. **Runtime Resolution**: At startup, the scheduler resolves config placeholders and registers tasks
4. **Execution**: Tasks are executed by `tokio-cron-scheduler` based on their schedule type

## Performance Considerations

- Tasks run in separate tokio tasks (non-blocking)
- Fixed rate tasks don't wait for previous execution to complete
- Fixed delay tasks wait for completion before scheduling next run
- Cron tasks are scheduled efficiently using the cron parser

## Error Handling

```rust
#[scheduled(fixed_rate = 30)]
async fn safe_task() {
    if let Err(e) = risky_operation().await {
        eprintln!("Task failed: {}", e);
        // Task will continue running on schedule
    }
}

async fn risky_operation() -> Result<(), Box<dyn std::error::Error>> {
    // Your logic here
    Ok(())
}
```

## Comparison with Spring Boot

| Spring Boot | Rust scheduled-rs |
|-------------|-------------------|
| `@Scheduled(cron = "...")` | `#[scheduled(cron = "...")]` |
| `@Scheduled(fixedRate = 1000)` | `#[scheduled(fixed_rate = 1)]` |
| `@Scheduled(fixedDelay = 1000)` | `#[scheduled(fixed_delay = 1)]` |
| `@Scheduled(initialDelay = 5000)` | `#[scheduled(initial_delay = 5)]` |
| `${property.name}` | `${property.name}` |
| `application.properties` | `application.toml` |

**Key Differences:**
- Spring Boot uses milliseconds, this library uses seconds
- Spring Boot uses `@` annotations, Rust uses `#[...]` attributes
- Configuration format: properties/yaml vs TOML (but YAML supported via config crate)

## Dependencies

- `tokio` - Async runtime
- `tokio-cron-scheduler` - Cron scheduling engine
- `config` - Configuration management
- `linkme` - Distributed slice for task registration
- `once_cell` - Lazy static initialization

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT OR Apache-2.0

## Roadmap

- [ ] Support for time zones in cron expressions
- [ ] Task metrics and monitoring
- [ ] Distributed task coordination
- [ ] Web UI for task management
- [ ] Database-backed task persistence
- [ ] Task history and audit logs
- [ ] Dynamic task registration at runtime
- [ ] Task priority support
- [ ] Concurrent execution limits

## Logging

This library uses the `tracing` crate for structured logging. **You need to initialize a tracing subscriber in your application** to see logs from the scheduler.

### Quick Setup

Add to your `Cargo.toml`:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }
```

### Basic Console Logging

```rust
use scheduled::{scheduled, SchedulerBuilder};
use tracing_subscriber;

#[scheduled(fixed_rate = "5s")]
async fn my_task() {
    println!("Task running!");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize console logging with INFO level
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    let scheduler = SchedulerBuilder::new().build();
    let handle = scheduler.start().await?;
    
    tokio::signal::ctrl_c().await?;
    handle.shutdown().await?;
    Ok(())
}
```

### Production Logging (JSON format)

```rust
use tracing_subscriber::{fmt, EnvFilter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // JSON formatted logging for production
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();
    
    // ... rest of your code
}
```

### Log Levels

The library logs at different levels:

- **INFO**: Scheduler startup, task registration, and success messages
  - `Scheduler started successfully`
  - `Task registered as tokio::interval task`
  
- **DEBUG**: Detailed task configuration (intervals, cron expressions, time units)
  - `Cron task configuration`
  - `Interval-based task configuration`
  - `Task disabled, skipping registration`
  
- **WARN**: Configuration warnings and non-critical issues
  - `Invalid time_unit, using milliseconds as default`
  - `Config key not found, using default value`
  
- **ERROR**: Task registration failures and critical errors
  - `Failed to register runnable task`

### Environment Variables

Control log level via environment variable:

```bash
# Windows PowerShell
$env:RUST_LOG="debug"
cargo run

# Linux/Mac
RUST_LOG=debug cargo run
```

### Filter by Module

```rust
// Only show scheduler logs at debug level, everything else at info
tracing_subscriber::fmt()
    .with_env_filter("info,scheduled_runtime=debug")
    .init();
```

### Structured Logging Example

The library outputs structured logs that can be parsed by log aggregation tools:

```json
{
  "timestamp": "2025-10-16T10:30:00Z",
  "level": "INFO",
  "fields": {
    "message": "Registering task",
    "task_name": "my_scheduled_task",
    "task_type": "Scheduled"
  },
  "target": "scheduled_runtime::scheduler::scheduler"
}
```

### No Logging Setup?

If you don't initialize a tracing subscriber, **the library will not output any logs**. This is intentional - it gives you full control over logging in your application.

## FAQ

**Q: Can I use this in production?**
A: This is a working implementation, but consider it as a starting point. Test thoroughly before production use.

**Q: How do I stop a running task?**
A: Tasks are spawned as tokio tasks. Currently, graceful shutdown happens when the main process exits.

**Q: Can I have multiple tasks with the same schedule?**
A: Yes! Each `#[scheduled]` annotation creates an independent task.

**Q: What happens if a task panics?**
A: The task will stop, but other tasks continue running. Implement proper error handling.

**Q: Can I use blocking code in tasks?**
A: Use `tokio::task::spawn_blocking` for CPU-intensive or blocking operations.

**Q: How do I test scheduled tasks?**
A: You can call the task functions directly in tests, or use custom configs with very short intervals.

**Q: Why don't I see any logs from the scheduler?**
A: You need to initialize a `tracing-subscriber` in your application. See the [Logging](#logging) section above.

## Examples in the Wild

```rust
// Database cleanup
#[scheduled(cron = "0 0 3 * * *")]
async fn cleanup_old_records() {
    database::delete_older_than(30).await;
}

// Cache warming
#[scheduled(fixed_rate = 300, initial_delay = 10)]
async fn warm_cache() {
    cache::refresh_popular_items().await;
}

// API polling
#[scheduled(fixed_delay = "${api.poll_interval:60}")]
async fn poll_external_api() {
    match api::fetch_updates().await {
        Ok(updates) => process_updates(updates).await,
        Err(e) => log::error!("API poll failed: {}", e),
    }
}

// Metrics collection
#[scheduled(fixed_rate = 10)]
async fn collect_metrics() {
    metrics::record_system_stats().await;
}
```

## Support

For issues, questions, or contributions, please visit the GitHub repository.

---

Made with ❤️ for the Rust community