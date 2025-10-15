use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Trait for schedulable tasks
/// 
/// Implement this trait on your struct to make it schedulable.
/// Use the `#[scheduled]` macro to configure the schedule.
/// 
/// # Example
/// 
/// ```rust
/// use scheduled_runtime::Runnable;
/// 
/// struct MyTask {
///     name: String,
/// }
/// 
/// #[scheduled(cron = "0 */5 * * * *")]
/// impl Runnable for MyTask {
///     async fn run(&self) {
///         println!("Task {} is running", self.name);
///     }
/// }
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
    pub fn new(
        name: &'static str,
        schedule_type: &'static str,
        schedule_value: &'static str,
        initial_delay: &'static str,
        enabled: &'static str,
        time_unit: &'static str,
        zone: &'static str,
        instance: Arc<dyn Runnable>,
    ) -> Self {
        Self {
            name,
            schedule_type,
            schedule_value,
            initial_delay,
            enabled,
            time_unit,
            zone,
            instance,
        }
    }
}

/// Helper function to create RunnableTask from a type with ScheduledMetadata
pub fn create_runnable_task<T>(instance: T) -> RunnableTask
where
    T: Runnable + ScheduledMetadata + 'static,
{
    RunnableTask::new(
        std::any::type_name::<T>(),
        T::schedule_type(),
        T::schedule_value(),
        T::initial_delay(),
        T::enabled(),
        T::time_unit(),
        T::zone(),
        Arc::new(instance),
    )
}

/// Global distributed slice for collecting runnable tasks
/// 
/// This slice is populated by the `#[scheduled]` macro and only includes
/// tasks that are explicitly registered via `SchedulerBuilder::runnable()`
#[linkme::distributed_slice]
pub static RUNNABLE_TASKS: [fn() -> Option<RunnableTask>] = [..];
