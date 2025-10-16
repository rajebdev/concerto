# 🎻 Concerto

[![Crates.io](https://img.shields.io/crates/v/concerto)](https://crates.io/crates/concerto)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE.md)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org)

> **Orchestrate your scheduled tasks with precision timing**

Concerto is a powerful task scheduling library for Rust, inspired by Spring Boot's `@Scheduled` annotation. It brings enterprise-grade scheduling capabilities to the Rust ecosystem with a clean, declarative API.

## ✨ Features

- 🚀 **Simple API** - Declarative scheduling with `#[scheduled]` attribute macro
- ⏰ **Cron Support** - Full cron expressions with timezone support
- 🔄 **Interval Scheduling** - Fixed rate and fixed delay execution
- ⚙️ **Config Integration** - TOML/YAML configuration with `${...}` placeholders
- 🎯 **Conditional Execution** - Enable/disable tasks via configuration
- ⏳ **Initial Delay** - Delay first execution of tasks
- 🕐 **Time Units** - Support for ms, s, m, h, d
- 🌍 **Timezone Support** - Specify timezone for cron expressions
- 📊 **Structured Logging** - Production-ready logging with `tracing` crate
- 🔧 **Environment Variables** - Override config with environment variables
- ✅ **Compile-time Validation** - Catch configuration errors early

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
concerto = { git = "https://github.com/rajebdev/concerto.git", tag = "v0.1.0" }
tokio = { version = "1", features = ["full"] }
tracing-subscriber = "0.3"  # For logging
```

> **Note:** Concerto is not yet published to crates.io. Install directly from GitHub using the method above.

## 🚀 Quick Start

```rust
use concerto::{scheduled, SchedulerBuilder};

// Auto-discovered scheduled function
#[scheduled(cron = "0 */5 * * * *")]
async fn every_five_minutes() {
    println!("Executing every 5 minutes!");
}

#[scheduled(fixed_rate = "30s")]
async fn every_30_seconds() {
    println!("Executing every 30 seconds!");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    // Build and start scheduler
    let scheduler = SchedulerBuilder::new().build();
    let handle = scheduler.start().await?;
    
    // Keep running until Ctrl+C
    tokio::signal::ctrl_c().await?;
    
    // Graceful shutdown
    handle.shutdown().await?;
    Ok(())
}
```

## 📚 Documentation

### Scheduling Options

#### Cron Expression

```rust
// Basic cron (local timezone)
#[scheduled(cron = "0 */5 * * * *")]
async fn every_5_minutes() { }

// With timezone
#[scheduled(cron = "0 0 9 * * *", zone = "Asia/Jakarta")]
async fn jakarta_morning() { }

// From config
#[scheduled(cron = "${app.cron}", zone = "${app.zone:local}")]
async fn config_cron() { }
```

**Cron format:** `second minute hour day month weekday`

**Common examples:**
- `0 * * * * *` - Every minute
- `0 */5 * * * *` - Every 5 minutes
- `0 0 * * * *` - Every hour
- `0 0 0 * * *` - Daily at midnight
- `0 30 9 * * 1-5` - Weekdays at 9:30 AM

#### Fixed Rate

Execute at fixed intervals (doesn't wait for completion):

```rust
#[scheduled(fixed_rate = "30s")]
async fn every_30_seconds() { }

#[scheduled(fixed_rate = 5, time_unit = "minutes")]
async fn every_5_minutes() { }
```

**Supported time units:** `ms`, `s`, `m`, `h`, `d`

#### Fixed Delay

Execute with fixed delay between completions:

```rust
#[scheduled(fixed_delay = "30s")]
async fn after_30_seconds() { }
```

#### Initial Delay

Delay the first execution:

```rust
#[scheduled(fixed_rate = "60s", initial_delay = "10s")]
async fn delayed_start() { }
```

#### Conditional Execution

```rust
#[scheduled(fixed_rate = "30s", enabled = "${feature.enabled}")]
async fn conditional_task() { }
```

### Configuration

**TOML** (`config/application.toml`):
```toml
[app]
interval = "60"
enabled = true

[app.schedule]
cron = "0 0 * * * *"
```

**Usage:**
```rust
#[scheduled(fixed_rate = "${app.interval}", enabled = "${app.enabled}")]
async fn config_task() { }
```

**Environment variables:**
```bash
export APP_INTERVAL=120
export APP_ENABLED=true
```

### Builder API

```rust
let scheduler = SchedulerBuilder::new()              // Default config
    // OR with config file
    .with_toml("config/application.toml")            // TOML
    .with_yaml("config/application.yaml")            // YAML
    
    // Optional: register manual tasks
    .register(task)
    
    .build();                                        // Build

let handle = scheduler.start().await?;               // Start
handle.shutdown().await?;                            // Shutdown
```

### Logging

Initialize tracing subscriber to see logs:

```rust
tracing_subscriber::fmt()
    .with_env_filter("info")
    .init();
```

Control log level:
```bash
RUST_LOG=debug cargo run
```

**Log levels:**
- `INFO` - Scheduler lifecycle, task registration
- `DEBUG` - Task configuration details
- `WARN` - Configuration warnings
- `ERROR` - Registration failures

## Project Structure

```
concerto/
├── Cargo.toml                 # Workspace definition
├── README.md
├── concerto/                  # Main library
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── concerto-macro/            # Procedural macro
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── concerto-runtime/          # Runtime & scheduler
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── examples/
│   └── basic.rs              # Example usage
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

| Spring Boot | Rust Concerto |
|-------------|---------------|
| `@Scheduled(cron = "...")` | `#[scheduled(cron = "...")]` |
| `@Scheduled(fixedRate = 1000)` | `#[scheduled(fixed_rate = 1)]` |
| `@Scheduled(fixedDelay = 1000)` | `#[scheduled(fixed_delay = 1)]` |
| `@Scheduled(initialDelay = 5000)` | `#[scheduled(initial_delay = 5)]` |
| `${property.name}` | `${property.name}` |
| `application.properties` | `application.toml` |

**Key Differences:**
- Spring Boot uses milliseconds, Concerto defaults to seconds (but supports ms)
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

- [x] Cron expressions with timezone support
- [x] Fixed rate and fixed delay scheduling
- [x] Configuration file integration (TOML/YAML)
- [x] Conditional task execution
- [x] Compile-time validation
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
use concerto::{scheduled, SchedulerBuilder};
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

## �️ Architecture

```
concerto/
├── concerto/              # Main library (public API)
├── concerto-macro/        # Procedural macro (#[scheduled])
├── concerto-runtime/      # Runtime & scheduler implementation
└── examples/              # Example applications
```

## 🔄 Comparison with Spring Boot

| Spring Boot | Concerto |
|-------------|----------|
| `@Scheduled(cron = "...")` | `#[scheduled(cron = "...")]` |
| `@Scheduled(fixedRate = 1000)` | `#[scheduled(fixed_rate = "1s")]` |
| `@Scheduled(fixedDelay = 1000)` | `#[scheduled(fixed_delay = "1s")]` |
| `@Scheduled(initialDelay = 5000)` | `#[scheduled(initial_delay = "5s")]` |
| `${property.name}` | `${property.name}` |
| `application.properties` | `application.toml` |

## 🎵 Why "Concerto"?

In a musical concerto, multiple instruments play together in perfect harmony and timing. Similarly, **Concerto** orchestrates your scheduled tasks to execute at precisely the right moment.

The name reflects:
- **Precision Timing** 🎯 - Tasks execute at the perfect moment
- **Harmony** 🎶 - Multiple tasks work together seamlessly
- **Orchestration** 🎻 - Centralized control over scheduling
- **Elegance** ✨ - Clean, declarative API

## 🛣️ Roadmap

- [x] Cron expressions with timezone support
- [x] Fixed rate and fixed delay scheduling
- [x] Configuration file integration (TOML/YAML)
- [x] Conditional task execution
- [x] Compile-time validation
- [x] Structured logging with tracing
- [ ] Task metrics and monitoring
- [ ] Dynamic task registration at runtime
- [ ] Task priority support
- [ ] Concurrent execution limits

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

This project is dual-licensed under MIT OR Apache-2.0. See [LICENSE.md](LICENSE.md) for details.

---

Made with ❤️ and 🎵 for the Rust community
