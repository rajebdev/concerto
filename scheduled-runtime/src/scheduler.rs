use crate::config::{load_toml_config, load_yaml_config, resolve_config_value};
use crate::registry::SCHEDULED_TASKS;
use crate::runnable::RunnableTask;
use crate::task::{ScheduledTask, ScheduledMethodMetadata, TimeUnit};
use config::Config;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

/// Trait for instances that have scheduled methods
/// This is automatically available for any type with `#[scheduled]` methods
pub trait ScheduledInstance: Send + Sync + 'static {
    /// Get all scheduled method metadata for this type
    fn scheduled_methods() -> Vec<ScheduledMethodMetadata>;
    
    /// Call a scheduled method by name
    fn call_scheduled_method(&self, method_name: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>>;
}

/// Type alias for method caller function
type MethodCaller = Arc<dyn Fn(&(dyn std::any::Any + Send + Sync), &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>> + Send + Sync>;

/// Wrapper for a registered instance with scheduled methods
struct RegisteredInstance {
    type_name: String,
    instance: Arc<dyn std::any::Any + Send + Sync>,
    methods: Vec<ScheduledMethodMetadata>,
    caller: MethodCaller,
}

/// Handle for a running scheduler
/// Used to control and shutdown the scheduler
pub struct SchedulerHandle {
    cron_scheduler: JobScheduler,
    interval_handles: Vec<tokio::task::JoinHandle<()>>,
}

impl SchedulerHandle {
    /// Shutdown the scheduler and all interval tasks
    pub async fn shutdown(mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Stop cron scheduler
        self.cron_scheduler.shutdown().await?;
        
        // Abort all interval tasks
        for handle in self.interval_handles {
            handle.abort();
        }
        
        Ok(())
    }
}

/// Configured scheduler ready to start
/// This struct holds all configuration and tasks but hasn't started yet
pub struct Scheduler {
    config: Arc<Config>,
    runnable_tasks: Vec<RunnableTask>,
    scheduled_tasks: Vec<ScheduledTask>,
    registered_instances: Vec<RegisteredInstance>,
}

impl Scheduler {
    /// Parse and resolve time unit configuration
    fn parse_time_unit(
        time_unit_str: &str,
    ) -> TimeUnit {
        time_unit_str.parse().unwrap_or_else(|_| {
            eprintln!(
                "    Warning: Invalid time_unit '{}', using milliseconds",
                time_unit_str
            );
            TimeUnit::Milliseconds
        })
    }

    /// Parse and resolve initial delay in milliseconds
    fn parse_initial_delay(
        initial_delay: &str,
        time_unit: TimeUnit,
    ) -> u64 {
        if let Some((value, parsed_unit)) = TimeUnit::parse_duration(initial_delay) {
            parsed_unit.to_millis(value)
        } else {
            let initial_delay_value: u64 = initial_delay.parse().unwrap_or_else(|_| {
                eprintln!(
                    "    Warning: Invalid initial_delay '{}', using 0",
                    initial_delay
                );
                0
            });
            time_unit.to_millis(initial_delay_value)
        }
    }

    /// Parse and resolve zone display string
    fn parse_zone_display(zone_str: &str) -> String {
        if zone_str.to_lowercase() == "local" {
            "Local".to_string()
        } else {
            zone_str.to_string()
        }
    }

    /// Parse interval value and return (value, time_unit, millis)
    fn parse_interval(
        interval_str: &str,
        default_time_unit: TimeUnit,
    ) -> Result<(u64, TimeUnit, u64), Box<dyn std::error::Error>> {
        let (interval_value, effective_time_unit) =
            if let Some((value, parsed_unit)) = TimeUnit::parse_duration(interval_str) {
                println!(
                    "    Parsed shorthand: '{}' -> {} {:?}",
                    interval_str, value, parsed_unit
                );
                (value, parsed_unit)
            } else {
                let value = interval_str
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid interval value: {}", interval_str))?;
                (value, default_time_unit)
            };

        let interval_millis = effective_time_unit.to_millis(interval_value);
        Ok((interval_value, effective_time_unit, interval_millis))
    }

    /// Print cron task information
    fn print_cron_info(
        cron_expr: &str,
        zone_display: &str,
        initial_delay_millis: u64,
        time_unit_str: &str,
    ) {
        println!("    Type: cron");
        println!("    Expression: {}", cron_expr);
        println!("    Zone: {} (timezone for cron evaluation)", zone_display);
        println!("    Initial delay: {}ms", initial_delay_millis);

        if time_unit_str.to_lowercase() != "milliseconds" {
            println!("    ⚠️  Warning: time_unit parameter is ignored for cron expressions");
            println!("        Cron uses absolute time (calendar-based), not intervals");
        }
    }

    /// Print interval task information
    fn print_interval_info(
        schedule_type: &str,
        interval_value: u64,
        effective_time_unit: TimeUnit,
        interval_millis: u64,
        initial_delay_millis: u64,
        zone_str: &str,
    ) {
        println!("    Type: {}", schedule_type);
        println!(
            "    Interval: {} {:?} ({}ms)",
            interval_value, effective_time_unit, interval_millis
        );
        println!("    Initial delay: {}ms", initial_delay_millis);

        if zone_str.to_lowercase() != "local" {
            println!("    ⚠️  Warning: zone parameter is ignored for interval-based tasks (fixed_rate/fixed_delay)");
            println!("        Interval tasks always use local system time");
        }
    }

    /// Start the scheduler with all registered tasks
    /// Returns a SchedulerHandle that can be used to shutdown the scheduler
    pub async fn start(self) -> Result<SchedulerHandle, Box<dyn std::error::Error>> {
        let total_tasks = self.runnable_tasks.len() + self.scheduled_tasks.len();
        let total_method_tasks: usize = self.registered_instances.iter()
            .map(|inst| inst.methods.len())
            .sum();
        println!("Starting scheduler with {} total tasks ({} from registered instances)...", 
                 total_tasks + total_method_tasks, total_method_tasks);

        let mut scheduler = JobScheduler::new().await?;
        let mut interval_handles = Vec::new();

        // Register runnable tasks (from Runnable trait implementations)
        for task in self.runnable_tasks {
            let enabled = resolve_config_value(task.enabled, &self.config)?;
            if enabled.to_lowercase() == "false" {
                println!("  [DISABLED] {} (Runnable)", task.name);
                continue;
            }

            let task_name = task.name;
            if let Err(e) = Self::register_runnable_task(
                &mut scheduler,
                &mut interval_handles,
                &self.config,
                task,
            )
            .await
            {
                eprintln!("  [ERROR] Failed to register {}: {}", task_name, e);
            }
        }

        // Register scheduled tasks (auto-discovered from #[scheduled] functions)
        for task in self.scheduled_tasks {
            let enabled = resolve_config_value(task.enabled, &self.config)?;
            if enabled.to_lowercase() == "false" {
                println!("  [DISABLED] {} (Scheduled)", task.name);
                continue;
            }

            let task_name = task.name;
            if let Err(e) = Self::register_scheduled_task(
                &mut scheduler,
                &mut interval_handles,
                &self.config,
                task,
            )
            .await
            {
                eprintln!("  [ERROR] Failed to register {}: {}", task_name, e);
            }
        }

        // Register method tasks from registered instances
        for registered_instance in self.registered_instances {
            for method_meta in &registered_instance.methods {
                let enabled = resolve_config_value(method_meta.enabled, &self.config)?;
                if enabled.to_lowercase() == "false" {
                    println!("  [DISABLED] {}::{} (Method)", 
                             registered_instance.type_name, method_meta.method_name);
                    continue;
                }

                let task_name = format!("{}::{}", registered_instance.type_name, method_meta.method_name);
                if let Err(e) = Self::register_method_task(
                    &mut scheduler,
                    &mut interval_handles,
                    &self.config,
                    &registered_instance,
                    *method_meta,
                    &task_name,
                )
                .await
                {
                    eprintln!("  [ERROR] Failed to register {}: {}", task_name, e);
                }
            }
        }

        scheduler.start().await?;
        println!("✅ Scheduler started successfully!");

        Ok(SchedulerHandle {
            cron_scheduler: scheduler,
            interval_handles,
        })
    }

    /// Register a runnable task instance
    async fn register_runnable_task(
        scheduler: &mut JobScheduler,
        interval_handles: &mut Vec<tokio::task::JoinHandle<()>>,
        config: &Arc<Config>,
        task: RunnableTask,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("  [REGISTER] {} (Runnable)", task.name);

        // Parse configuration values
        let time_unit_str = resolve_config_value(task.time_unit, config)?;
        let time_unit = Self::parse_time_unit(&time_unit_str);

        let initial_delay = resolve_config_value(task.initial_delay, config)?;
        let initial_delay_millis = Self::parse_initial_delay(&initial_delay, time_unit);

        let zone_str = resolve_config_value(task.zone, config)?;
        let zone_display = Self::parse_zone_display(&zone_str);

        match task.schedule_type {
            "cron" => {
                let cron_expr = resolve_config_value(task.schedule_value, config)?;
                Self::print_cron_info(&cron_expr, &zone_display, initial_delay_millis, &time_unit_str);

                let instance = task.instance.clone();

                let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
                    let instance = instance.clone();
                    Box::pin(async move {
                        instance.run().await;
                    })
                })?;

                scheduler.add(job).await?;
            }
            "fixed_rate" | "fixed_delay" => {
                let interval_str = resolve_config_value(task.schedule_value, config)?;
                let (interval_value, effective_time_unit, interval_millis) = 
                    Self::parse_interval(&interval_str, time_unit)?;

                Self::print_interval_info(
                    task.schedule_type,
                    interval_value,
                    effective_time_unit,
                    interval_millis,
                    initial_delay_millis,
                    &zone_str,
                );

                let instance = task.instance.clone();
                let is_fixed_delay = task.schedule_type == "fixed_delay";

                let handle = tokio::spawn(async move {
                    if initial_delay_millis > 0 {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            initial_delay_millis,
                        ))
                        .await;
                    }

                    let mut interval = tokio::time::interval(std::time::Duration::from_millis(
                        interval_millis,
                    ));

                    interval.tick().await;

                    loop {
                        if is_fixed_delay {
                            instance.run().await;
                            interval.tick().await;
                        } else {
                            interval.tick().await;
                            let instance_clone = instance.clone();
                            tokio::spawn(async move {
                                instance_clone.run().await;
                            });
                        }
                    }
                });

                interval_handles.push(handle);
                println!("    ✅ Registered as tokio::interval task");
            }
            _ => {
                return Err(format!("Unknown schedule type: {}", task.schedule_type).into());
            }
        }

        Ok(())
    }

    /// Register a scheduled task (from #[scheduled] macro)
    async fn register_scheduled_task(
        scheduler: &mut JobScheduler,
        interval_handles: &mut Vec<tokio::task::JoinHandle<()>>,
        config: &Arc<Config>,
        task: ScheduledTask,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("  [REGISTER] {} (Scheduled)", task.name);

        // Parse configuration values
        let time_unit_str = resolve_config_value(task.time_unit, config)?;
        let time_unit = Self::parse_time_unit(&time_unit_str);

        let initial_delay = resolve_config_value(task.initial_delay, config)?;
        let initial_delay_millis = Self::parse_initial_delay(&initial_delay, time_unit);

        let zone_str = resolve_config_value(task.zone, config)?;
        let zone_display = Self::parse_zone_display(&zone_str);

        match task.schedule_type {
            "cron" => {
                let cron_expr = resolve_config_value(task.schedule_value, config)?;
                Self::print_cron_info(&cron_expr, &zone_display, initial_delay_millis, &time_unit_str);

                let handler = task.handler;

                let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
                    Box::pin(async move {
                        handler();
                    })
                })?;

                scheduler.add(job).await?;
            }
            "fixed_rate" | "fixed_delay" => {
                let interval_str = resolve_config_value(task.schedule_value, config)?;
                let (interval_value, effective_time_unit, interval_millis) = 
                    Self::parse_interval(&interval_str, time_unit)?;

                Self::print_interval_info(
                    task.schedule_type,
                    interval_value,
                    effective_time_unit,
                    interval_millis,
                    initial_delay_millis,
                    &zone_str,
                );

                let handler = task.handler;
                let is_fixed_delay = task.schedule_type == "fixed_delay";

                let handle = tokio::spawn(async move {
                    if initial_delay_millis > 0 {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            initial_delay_millis,
                        ))
                        .await;
                    }

                    let mut interval = tokio::time::interval(std::time::Duration::from_millis(
                        interval_millis,
                    ));

                    interval.tick().await;

                    loop {
                        if is_fixed_delay {
                            handler();
                            interval.tick().await;
                        } else {
                            interval.tick().await;
                            let handler_clone = handler;
                            tokio::spawn(async move {
                                handler_clone();
                            });
                        }
                    }
                });

                interval_handles.push(handle);
                println!("    ✅ Registered as tokio::interval task");
            }
            _ => {
                return Err(format!("Unknown schedule type: {}", task.schedule_type).into());
            }
        }

        Ok(())
    }

    /// Register a method task from a registered instance
    async fn register_method_task(
        scheduler: &mut JobScheduler,
        interval_handles: &mut Vec<tokio::task::JoinHandle<()>>,
        config: &Arc<Config>,
        registered_instance: &RegisteredInstance,
        method_meta: ScheduledMethodMetadata,
        task_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("  [REGISTER] {} (Method)", task_name);

        // Parse configuration values
        let time_unit_str = resolve_config_value(method_meta.time_unit, config)?;
        let time_unit = Self::parse_time_unit(&time_unit_str);

        let initial_delay = resolve_config_value(method_meta.initial_delay, config)?;
        let initial_delay_millis = Self::parse_initial_delay(&initial_delay, time_unit);

        let zone_str = resolve_config_value(method_meta.zone, config)?;
        let zone_display = Self::parse_zone_display(&zone_str);

        match method_meta.schedule_type {
            "cron" => {
                let cron_expr = resolve_config_value(method_meta.schedule_value, config)?;
                Self::print_cron_info(&cron_expr, &zone_display, initial_delay_millis, &time_unit_str);

                let instance = registered_instance.instance.clone();
                let caller = registered_instance.caller.clone();
                let method_name = method_meta.method_name.to_string();

                let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
                    let instance = instance.clone();
                    let caller = caller.clone();
                    let method_name = method_name.clone();
                    Box::pin(async move {
                        let future = caller(instance.as_ref(), &method_name);
                        future.await;
                    })
                })?;

                scheduler.add(job).await?;
            }
            "fixed_rate" | "fixed_delay" => {
                let interval_str = resolve_config_value(method_meta.schedule_value, config)?;
                let (interval_value, effective_time_unit, interval_millis) = 
                    Self::parse_interval(&interval_str, time_unit)?;

                Self::print_interval_info(
                    method_meta.schedule_type,
                    interval_value,
                    effective_time_unit,
                    interval_millis,
                    initial_delay_millis,
                    &zone_str,
                );

                let instance = registered_instance.instance.clone();
                let caller = registered_instance.caller.clone();
                let method_name = method_meta.method_name.to_string();
                let is_fixed_delay = method_meta.schedule_type == "fixed_delay";

                let handle = tokio::spawn(async move {
                    if initial_delay_millis > 0 {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            initial_delay_millis,
                        ))
                        .await;
                    }

                    let mut interval = tokio::time::interval(std::time::Duration::from_millis(
                        interval_millis,
                    ));

                    interval.tick().await;

                    loop {
                        if is_fixed_delay {
                            let future = caller(instance.as_ref(), &method_name);
                            future.await;
                            interval.tick().await;
                        } else {
                            interval.tick().await;
                            let instance_clone = instance.clone();
                            let caller_clone = caller.clone();
                            let method_name_clone = method_name.clone();
                            tokio::spawn(async move {
                                let future = caller_clone(instance_clone.as_ref(), &method_name_clone);
                                future.await;
                            });
                        }
                    }
                });

                interval_handles.push(handle);
                println!("    ✅ Registered as tokio::interval task");
            }
            _ => {
                return Err(format!("Unknown schedule type: {}", method_meta.schedule_type).into());
            }
        }

        Ok(())
    }
}

/// Builder for the scheduler
pub struct SchedulerBuilder {
    config: Arc<Config>,
    runnable_tasks: Vec<RunnableTask>,
    registered_instances: Vec<RegisteredInstance>,
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
    /// use std::pin::Pin;
    /// use std::future::Future;
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
    ///     fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
    ///         Box::pin(async move {
    ///             println!("Running task");
    ///         })
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
        // Collect scheduled tasks from registry (auto-discovered #[scheduled] functions)
        let scheduled_tasks: Vec<ScheduledTask> = SCHEDULED_TASKS.iter().map(|f| f()).collect();

        let method_task_count: usize = self.registered_instances.iter()
            .map(|inst| inst.methods.len())
            .sum();

        println!(
            "Building scheduler with {} runnable tasks, {} scheduled tasks, and {} method tasks",
            self.runnable_tasks.len(),
            scheduled_tasks.len(),
            method_task_count
        );

        Scheduler {
            config: self.config,
            runnable_tasks: self.runnable_tasks,
            scheduled_tasks,
            registered_instances: self.registered_instances,
        }
    }
}
