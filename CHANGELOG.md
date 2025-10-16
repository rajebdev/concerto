# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed - **PRODUCTION READY LOGGING**
- **Replaced all `println!` and `eprintln!` with structured logging**
  - Now uses `tracing` crate for production-grade observability
  - Library emits structured logs with contextual information
  - Zero overhead when tracing subscriber not initialized
  
- **Added comprehensive logging levels**:
  - `INFO`: Scheduler lifecycle and task registration
  - `DEBUG`: Detailed task configuration (cron expressions, intervals, time units)
  - `WARN`: Configuration warnings and fallbacks
  - `ERROR`: Task registration failures and critical errors
  
- **Structured log fields** for better observability:
  - Task names, types, and schedule types
  - Configuration values (intervals, time units, initial delays)
  - Error messages with context
  
### Added
- **LOGGING.md**: Comprehensive logging guide for library users
- **README.md Logging Section**: Quick start guide for tracing setup
- **Example with logging**: Updated `basic.rs` to demonstrate tracing setup

### Improved
- Better error messages with structured context
- Config warnings now use `tracing::warn` instead of `eprintln!`
- All internal logs follow consistent structured format

### Note for Library Users
⚠️ **You MUST initialize a tracing subscriber in your application to see logs**

```rust
// Add to your main() before creating scheduler
tracing_subscriber::fmt()
    .with_env_filter("info")
    .init();
```

See [LOGGING.md](LOGGING.md) for full documentation.

### Changed - **BREAKING**
- **Refactored SchedulerBuilder API**: Separated build and start phases
  - `SchedulerBuilder::build()` now returns `Scheduler` (not `Result`, pure setup)
  - Added `Scheduler::start()` method that returns `Result<SchedulerHandle>` (async, can error)
  - This provides clearer separation between configuration and execution phases
  
- **Removed `register_all()` method**: Now automatic
  - All `#[scheduled]` functions are auto-discovered during `build()`
  - No need to call `.register_all()` anymore
  - Simplifies API and reduces boilerplate

- **Removed deprecated `start()` method from SchedulerBuilder**
  - Old: `.runnable(task).start().await?`
  - New: `.runnable(task).build()` then `scheduler.start().await?`

### Migration Guide

**Before (Old API):**
```rust
// Function-based
let scheduler = SchedulerBuilder::new()
    .register_all()
    .build()
    .await?;

// Runnable-based
let scheduler = SchedulerBuilder::new()
    .runnable(task)
    .start()
    .await?;
```

**After (New API):**
```rust
// Function-based (auto-discovers #[scheduled] functions)
let scheduler = SchedulerBuilder::new().build();
let handle = scheduler.start().await?;

// Runnable-based (manual registration)
let scheduler = SchedulerBuilder::new()
    .runnable(task)
    .build();  // <- No ? needed
let handle = scheduler.start().await?;
```

### Benefits
- ✅ **Clearer lifecycle**: Build (setup) vs Start (execution)
- ✅ **Better error handling**: Errors only happen during `start()`, not `build()`
- ✅ **Simpler API**: Auto-discovery by default, no need for `register_all()`
- ✅ **Type safety**: `build()` cannot fail, only `start()` can
- ✅ **Consistent pattern**: Matches standard builder patterns (Rocket, Actix, etc.)

### Added
- Initial release of scheduled-rs
- `#[scheduled]` macro for annotation-based task scheduling
- Support for cron expressions
- Support for fixed rate scheduling
- Support for fixed delay scheduling
- Config placeholder support with `${...}` syntax
- Initial delay support
- Conditional execution via `enabled` parameter
- Default value support in config placeholders
- Environment variable overrides
- Integration with `tokio-cron-scheduler`
- Auto-registration using `linkme`
- Comprehensive examples
- Full documentation
- New `Scheduler` struct for pre-start configuration
- `SchedulerHandle` for managing running scheduler

### Features

#### Core Scheduling
- **Cron expressions**: Standard cron format support
- **Fixed rate**: Execute at fixed intervals
- **Fixed delay**: Wait for completion before next execution
- **Initial delay**: Delay first execution

#### Configuration
- **TOML config**: `config/application.toml` support
- **YAML config**: `config/application.yaml` support
- **Environment variables**: `APP_*` prefix support
- **Config placeholders**: `${key:default}` syntax
- **Nested keys**: Support for `app.schedule.interval`

#### Developer Experience
- **Type-safe**: Compile-time validation
- **Auto-registration**: `#[scheduled]` functions automatically discovered
- **Manual registration**: Struct-based `Runnable` tasks require explicit `.runnable()` call
- **Async-first**: Built on tokio
- **Error handling**: Graceful error handling
- **Graceful shutdown**: `handle.shutdown()` for clean termination

## [0.1.0] - 2024-XX-XX

### Added
- Initial public release


[Unreleased]: https://github.com/yourusername/scheduled-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/scheduled-rs/releases/tag/v0.1.0