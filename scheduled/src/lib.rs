//! # Scheduled - Spring Boot-like Task Scheduling for Rust
//!
//! This library provides a familiar annotation-based approach to task scheduling,
//! similar to Spring Boot's `@Scheduled` annotation.
//!
//! ## Features
//!
//! - **Cron expressions**: Schedule tasks using standard cron syntax
//! - **Fixed rate**: Execute tasks at fixed intervals
//! - **Fixed delay**: Execute tasks with fixed delay between completions
//! - **Config support**: Use placeholders like `${app.interval}` to read from config files
//! - **Initial delay**: Delay the first execution
//! - **Enable/disable**: Conditionally enable tasks via config
//!
//! ## Quick Start with SchedulerBuilder
//!
//! ```rust
//! use scheduled::{scheduled, SchedulerBuilder};
//!
//! #[scheduled(cron = "0 */5 * * * *")]
//! async fn every_five_minutes() {
//!     println!("This runs every 5 minutes");
//! }
//!
//! #[scheduled(fixed_rate = "30s")]
//! async fn every_30_seconds() {
//!     println!("This runs every 30 seconds");
//! }
//!
//! #[scheduled(fixed_rate = "${app.interval}")]
//! async fn from_config() {
//!     println!("Interval comes from config file");
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Start with YAML config
//!     let _scheduler = SchedulerBuilder::with_yaml("config/application.yaml")
//!         .await?
//!         .register_all()
//!         .await?
//!         .build()
//!         .await?;
//!     
//!     // Keep app running
//!     tokio::signal::ctrl_c().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration
//!
//! Create `config/application.toml`:
//!
//! ```toml
//! [app]
//! interval = 60
//! 
//! [app.schedule]
//! cron = "0 0 * * * *"
//! enabled = true
//! ```
//!
//! Or `config/application.yaml`:
//!
//! ```yaml
//! app:
//!   interval: 60
//!   schedule:
//!     cron: "0 0 * * * *"
//!     enabled: true
//! ```
//!
//! You can also use environment variables with `APP_` prefix:
//!
//! ```bash
//! export APP_INTERVAL=120
//! export APP_SCHEDULE_CRON="0 */10 * * * *"
//! ```

// Re-export macros
pub use scheduled_macro::{scheduled, scheduled_impl};

// Re-export core types
pub use scheduled_runtime::{
    Runnable, ScheduledMetadata, SchedulerBuilder, TimeUnit,
};

// Make scheduled_runtime available for macro expansion
pub use scheduled_runtime;

// Re-export commonly used types
pub use tokio_cron_scheduler::JobScheduler;