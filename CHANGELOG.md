# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

### Features

#### Core Scheduling
- **Cron expressions**: Standard cron format support
- **Fixed rate**: Execute at fixed intervals
- **Fixed delay**: Wait for completion before next execution
- **Initial delay**: Delay first execution

#### Configuration
- **TOML config**: `config/application.toml` support
- **Environment variables**: `APP_*` prefix support
- **Config placeholders**: `${key:default}` syntax
- **Nested keys**: Support for `app.schedule.interval`

#### Developer Experience
- **Type-safe**: Compile-time validation
- **Auto-registration**: No manual scheduler setup needed
- **Async-first**: Built on tokio
- **Error handling**: Graceful error handling

## [0.1.0] - 2024-XX-XX

### Added
- Initial public release

[Unreleased]: https://github.com/yourusername/scheduled-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/scheduled-rs/releases/tag/v0.1.0