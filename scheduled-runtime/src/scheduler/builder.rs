use super::instance::{RegisteredInstance, ScheduledInstance};
use super::scheduler::Scheduler;
use crate::config::{load_toml_config, load_yaml_config};
use crate::runnable::RunnableTask;
use config::Config;
use std::sync::Arc;
use tracing::info;

/// Builder for the scheduler
pub struct SchedulerBuilder {
    pub(crate) config: Arc<Config>,
    pub(crate) runnable_tasks: Vec<RunnableTask>,
    pub(crate) registered_instances: Vec<RegisteredInstance>,
}

impl Default for SchedulerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SchedulerBuilder {
    /// Create a new scheduler builder with default config (empty)
    pub fn new() -> Self {
        Self {
            config: Arc::new(Config::default()),
            runnable_tasks: Vec::new(),
            registered_instances: Vec::new(),
        }
    }

    /// Create with TOML config file
    /// 
    /// # Panics
    /// 
    /// Panics if the config file cannot be loaded or parsed.
    /// This is intentional as configuration errors should be caught early during setup.
    pub fn with_toml(path: &str) -> Self {
        let config = load_toml_config(path)
            .unwrap_or_else(|e| panic!("Failed to load TOML config from '{}': {}", path, e));
        Self {
            config: Arc::new(config),
            runnable_tasks: Vec::new(),
            registered_instances: Vec::new(),
        }
    }

    /// Create with YAML config file
    /// 
    /// # Panics
    /// 
    /// Panics if the config file cannot be loaded or parsed.
    /// This is intentional as configuration errors should be caught early during setup.
    pub fn with_yaml(path: &str) -> Self {
        let config = load_yaml_config(path)
            .unwrap_or_else(|e| panic!("Failed to load YAML config from '{}': {}", path, e));
        Self {
            config: Arc::new(config),
            runnable_tasks: Vec::new(),
            registered_instances: Vec::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            runnable_tasks: Vec::new(),
            registered_instances: Vec::new(),
        }
    }

    /// Register an instance with #[scheduled] methods
    ///
    /// This unified method can handle both:
    /// - Types implementing `Runnable` trait (with `#[scheduled]` on impl Runnable block)
    /// - Types with `#[scheduled]` methods (method-based scheduling)
    ///
    /// The Rust compiler will automatically choose the correct implementation based on the
    /// traits that your type implements.
    ///
    /// # Example with #[scheduled] methods
    ///
    /// ```rust
    /// use scheduled_runtime::SchedulerBuilder;
    /// 
    /// struct UserService {
    ///     name: String,
    /// }
    /// 
    /// impl UserService {
    ///     fn new(name: String) -> Self {
    ///         Self { name }
    ///     }
    /// }
    /// 
    /// // Note: In actual code, use #[scheduled] macro on impl block
    /// // and #[scheduled(fixed_rate = "5s")] on methods
    /// 
    /// # fn main() {
    /// let scheduler = SchedulerBuilder::new()
    ///     // .register(UserService::new("test".to_string()))
    ///     .build();
    /// # }
    /// ```
    ///
    /// # Example with Runnable trait
    ///
    /// ```rust
    /// use scheduled_runtime::{SchedulerBuilder, Runnable};
    /// 
    /// struct MyTask {
    ///     name: String,
    /// }
    /// 
    /// impl MyTask {
    ///     fn new(name: String) -> Self {
    ///         Self { name }
    ///     }
    /// }
    /// 
    /// // Note: In actual code, use #[scheduled(fixed_rate = "5s")] on impl block
    /// impl Runnable for MyTask {
    ///     fn run(&self) {
    ///         println!("Running task");
    ///     }
    /// }
    /// 
    /// # fn main() {
    /// let scheduler = SchedulerBuilder::new()
    ///     // .register(MyTask::new("test".to_string()))
    ///     .build();
    /// # }
    /// ```
    pub fn register<T>(mut self, instance: T) -> Self
    where
        T: ScheduledInstance + 'static,
    {
        let type_name = std::any::type_name::<T>().to_string();
        let methods = T::scheduled_methods();
        
        let instance_arc: Arc<T> = Arc::new(instance);
        let instance_arc_clone = instance_arc.clone();
        let caller = Arc::new(move |_any_inst: &(dyn std::any::Any + Send + Sync), method_name: &str| {
            let method_name = method_name.to_string();
            let inst_clone = instance_arc_clone.clone();
            Box::pin(async move {
                let future = inst_clone.call_scheduled_method(&method_name);
                future.await;
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>>
        });

        self.registered_instances.push(RegisteredInstance {
            type_name,
            instance: instance_arc as Arc<dyn std::any::Any + Send + Sync>,
            methods,
            caller,
        });
        
        self
    }

    /// Build the scheduler (does not start it yet)
    ///
    /// This will:
    /// - Collect all tasks registered via `.register()`
    /// - Auto-discover all tasks marked with `#[scheduled]` macro
    /// - Return a `Scheduler` ready to start
    ///
    /// # Example
    ///
    /// ```rust
    /// use scheduled_runtime::SchedulerBuilder;
    /// 
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let scheduler = SchedulerBuilder::new()
    ///     // In actual code, add .with_toml("config.toml")
    ///     // In actual code, add .register(MyTask)
    ///     .build();  // <- No error here, just setup
    /// 
    /// // scheduler.start().await?;  // <- Errors happen here
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Scheduler {
        use crate::registry::SCHEDULED_TASKS;
        use crate::task::ScheduledTask;
        
        // Collect scheduled tasks from registry (auto-discovered #[scheduled] functions)
        let scheduled_tasks: Vec<ScheduledTask> = SCHEDULED_TASKS.iter().map(|f| f()).collect();

        let method_task_count: usize = self.registered_instances.iter()
            .map(|inst| inst.methods.len())
            .sum();

        info!(
            runnable_tasks = self.runnable_tasks.len(),
            scheduled_tasks = scheduled_tasks.len(),
            method_tasks = method_task_count,
            "Building scheduler"
        );

        Scheduler {
            config: self.config,
            runnable_tasks: self.runnable_tasks,
            scheduled_tasks,
            registered_instances: self.registered_instances,
        }
    }
}
