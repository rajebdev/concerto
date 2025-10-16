# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-10-16

### Added
- Initial release of Concerto (formerly scheduled-rs)
- `#[scheduled]` attribute macro for declarative task scheduling
- Cron expression support with timezone configuration
- Fixed rate and fixed delay scheduling
- Configuration file integration (TOML/YAML)
- Config placeholder resolution with `${key:default}` syntax
- Environment variable override support
- Conditional task execution via `enabled` parameter
- Initial delay support for delayed task start
- Time unit support (ms, s, m, h, d)
- Auto-discovery of scheduled functions via `linkme`
- Manual task registration via `Runnable` trait
- Unified `.register()` API for task registration
- Graceful shutdown support
- Structured logging with `tracing` crate
- Compile-time validation and warnings (W001, W002, W003)
- Support for scheduled methods in impl blocks
- Builder pattern for scheduler configuration

### Changed
- Renamed project from "scheduled-rs" to "Concerto"
- Refactored runtime into modular components (runnable, scheduler, config, registry)
- Simplified `Runnable::run` method to synchronous execution
- Separated scheduler build and start phases for clearer lifecycle management
- `SchedulerBuilder::build()` now returns `Scheduler` (pure setup, cannot fail)
- `Scheduler::start()` is async and returns `Result<SchedulerHandle>` (can fail)
- Removed `register_all()` method - auto-discovery is now automatic
- Replaced all `println!/eprintln!` with structured `tracing` logs

### Migration from Pre-release
```rust
// Old
let scheduler = SchedulerBuilder::new()
    .register_all()
    .build()
    .await?;

// New
let scheduler = SchedulerBuilder::new().build();
let handle = scheduler.start().await?;
```

## [Unreleased]

### Planned
- Task metrics and monitoring
- Dynamic task registration at runtime
- Task priority support
- Concurrent execution limits
- Web UI for task management
- Task history and audit logs

---

## Development History

**October 16, 2025**
- `a0a899c` - refactor: rename project from 'scheduled' to 'concerto'
- `d36ac6f` - feat(logging): implement structured logging with tracing
- `f45bfef` - feat: add chrono dependency and update logging format
- `bb6fe45` - refactor(runnable): simplify run method to synchronous execution
- `923afc6` - refactor: modularize runnable tasks and scheduler components
- `b080a7d` - refactor: scheduled library and runtime structure

**October 15, 2025**
- `6e39294` - feat: unify scheduler registration API with single .register() method
- `9f6fc2b` - feat(scheduler): add support for scheduled methods in impl blocks
- `0f01828` - feat(validation): enforce Runnable trait implementation
- `4ab8b71` - feat(warnings): add compiler warnings for misconfigurations
- `40cfc1b` - refactor: scheduler initialization and task registration
- `5900f36` - feat(config): enhance config placeholder resolution
- `9b3f0cf` - refactor(scheduled): replace `scheduled_impl` with `scheduled` macro
- `22c089d` - feat: initial scheduled task macro and runtime

[0.1.0]: https://github.com/yourusername/concerto/releases/tag/v0.1.0


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