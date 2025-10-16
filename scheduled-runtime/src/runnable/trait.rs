use std::future::Future;
use std::pin::Pin;

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
