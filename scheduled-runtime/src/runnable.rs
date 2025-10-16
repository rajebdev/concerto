use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::task::TimeUnit;

/// Trait for schedulable tasks
/// 
/// Implement this trait on your struct to make it schedulable.
/// Use the `#[scheduled]` macro to configure the schedule.
/// 
/// # Example
/// 
/// ```rust
/// use scheduled_runtime::Runnable;
/// use std::pin::Pin;
/// use std::future::Future;
/// 
/// struct MyTask {
///     name: String,
/// }
/// 
/// // Note: #[scheduled] macro would be used in actual code
/// impl Runnable for MyTask {
///     fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
///         Box::pin(async move {
///             println!("Task {} is running", self.name);
///         })
///     }
/// }
/// 
/// # fn main() {
/// #     let task = MyTask { name: "test".to_string() };
/// #     // Task can be registered with scheduler
/// # }
/// ```
pub trait Runnable: Send + Sync {
    /// Execute the scheduled task
    fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

/// Trait for extracting scheduling metadata from types annotated with #[scheduled]
pub trait ScheduledMetadata {
    fn schedule_type() -> &'static str;
    fn schedule_value() -> &'static str;
    fn initial_delay() -> &'static str;
    fn enabled() -> &'static str;
    fn time_unit() -> &'static str;
    fn zone() -> &'static str;
    
    /// Get the TimeUnit enum (used when time_unit is specified as TimeUnit::*)
    /// Returns None if time_unit was not explicitly set (uses default from string)
    fn time_unit_enum() -> Option<TimeUnit> {
        None
    }
}

/// Internal representation of a runnable task with its schedule configuration
#[derive(Clone)]
pub struct RunnableTask {
    pub name: &'static str,
    pub schedule_type: &'static str,
    pub schedule_value: &'static str,
    pub initial_delay: &'static str,
    pub enabled: &'static str,
    pub time_unit: &'static str,
    pub zone: &'static str,
    pub instance: Arc<dyn Runnable>,
}

impl RunnableTask {
    /// Create a new builder for RunnableTask
    pub fn builder(name: &'static str, instance: Arc<dyn Runnable>) -> RunnableTaskBuilder {
        RunnableTaskBuilder {
            name,
            schedule_type: "cron",
            schedule_value: "0 0 * * * *",
            initial_delay: "0",
            enabled: "true",
            time_unit: "seconds",
            zone: "UTC",
            instance,
        }
    }
}

/// Builder for RunnableTask
pub struct RunnableTaskBuilder {
    name: &'static str,
    schedule_type: &'static str,
    schedule_value: &'static str,
    initial_delay: &'static str,
    enabled: &'static str,
    time_unit: &'static str,
    zone: &'static str,
    instance: Arc<dyn Runnable>,
}

impl RunnableTaskBuilder {
    pub fn schedule_type(mut self, schedule_type: &'static str) -> Self {
        self.schedule_type = schedule_type;
        self
    }

    pub fn schedule_value(mut self, schedule_value: &'static str) -> Self {
        self.schedule_value = schedule_value;
        self
    }

    pub fn initial_delay(mut self, initial_delay: &'static str) -> Self {
        self.initial_delay = initial_delay;
        self
    }

    pub fn enabled(mut self, enabled: &'static str) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn time_unit(mut self, time_unit: &'static str) -> Self {
        self.time_unit = time_unit;
        self
    }

    pub fn zone(mut self, zone: &'static str) -> Self {
        self.zone = zone;
        self
    }

    pub fn build(self) -> RunnableTask {
        RunnableTask {
            name: self.name,
            schedule_type: self.schedule_type,
            schedule_value: self.schedule_value,
            initial_delay: self.initial_delay,
            enabled: self.enabled,
            time_unit: self.time_unit,
            zone: self.zone,
            instance: self.instance,
        }
    }
}

/// Global distributed slice for collecting runnable tasks
/// 
/// This slice is populated by the `#[scheduled]` macro and only includes
/// tasks that are explicitly registered via `SchedulerBuilder::runnable()`
#[linkme::distributed_slice]
pub static RUNNABLE_TASKS: [fn() -> Option<RunnableTask>] = [..];
