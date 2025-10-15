use crate::config::{load_toml_config, load_yaml_config, resolve_config_value};
use crate::registry::SCHEDULED_TASKS;
use crate::runnable::{create_runnable_task, Runnable, RunnableTask, ScheduledMetadata};
use crate::task::{ScheduledTask, TimeUnit};
use config::Config;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

/// Wrapper that holds both cron scheduler and interval task handles
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

/// Builder for the scheduler
pub struct SchedulerBuilder {
    config: Arc<Config>,
    runnable_tasks: Vec<RunnableTask>,
    should_register_all: bool,
}

impl SchedulerBuilder {
    /// Create a new scheduler builder with default config (empty)
    pub fn new() -> Self {
        Self {
            config: Arc::new(Config::default()),
            runnable_tasks: Vec::new(),
            should_register_all: false,
        }
    }

    /// Create with TOML config file
    pub fn with_toml(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = load_toml_config(path)?;
        Ok(Self {
            config: Arc::new(config),
            runnable_tasks: Vec::new(),
            should_register_all: false,
        })
    }

    /// Create with YAML config file
    pub fn with_yaml(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = load_yaml_config(path)?;
        Ok(Self {
            config: Arc::new(config),
            runnable_tasks: Vec::new(),
            should_register_all: false,
        })
    }

    /// Register a runnable task instance
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let user_task = UserTask::new();
    /// 
    /// SchedulerBuilder::with_toml("config/application.toml")?
    ///     .runnable(user_task)
    ///     .start()
    ///     .await?;
    /// ```
    pub fn runnable<T>(mut self, instance: T) -> Self
    where
        T: Runnable + ScheduledMetadata + 'static,
    {
        let task = create_runnable_task(instance);
        self.runnable_tasks.push(task);
        self
    }

    /// Start the scheduler with all registered runnable tasks
    /// 
    /// This will only execute tasks that have been registered via `.runnable()`
    pub async fn start(self) -> Result<SchedulerHandle, Box<dyn std::error::Error>> {
        println!("Starting scheduler with {} registered runnable tasks...", self.runnable_tasks.len());

        let mut scheduler = JobScheduler::new().await?;
        let mut interval_handles = Vec::new();

        // Clone the tasks to avoid borrowing issues
        let tasks = self.runnable_tasks.clone();
        
        // Process all registered runnable tasks
        for task in tasks {
            // Check if task is enabled
            let enabled = resolve_config_value(task.enabled, &self.config)?;
            if enabled.to_lowercase() == "false" {
                println!("  [DISABLED] {}", task.name);
                continue;
            }

            let task_name = task.name;
            if let Err(e) = Self::register_runnable_task_internal(
                &mut scheduler, 
                &mut interval_handles,
                &self.config, 
                task
            ).await {
                eprintln!("  [ERROR] Failed to register {}: {}", task_name, e);
            }
        }

        scheduler.start().await?;
        println!("✅ Scheduler started successfully!");
        
        Ok(SchedulerHandle {
            cron_scheduler: scheduler,
            interval_handles,
        })
    }

    /// Create with custom config
    pub fn with_config(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            runnable_tasks: Vec::new(),
            should_register_all: false,
        }
    }

    /// Register all tasks marked with #[scheduled]
    pub fn register_all(mut self) -> Self {
        self.should_register_all = true;
        self
    }

    /// Register a single runnable task
    async fn register_runnable_task_internal(
        scheduler: &mut JobScheduler,
        interval_handles: &mut Vec<tokio::task::JoinHandle<()>>,
        config: &Arc<Config>,
        task: RunnableTask,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("  [REGISTER] {}", task.name);

        // Resolve time_unit from config (default: milliseconds)
        let time_unit_str = resolve_config_value(task.time_unit, config)?;
        let time_unit = TimeUnit::from_str(&time_unit_str)
            .unwrap_or_else(|| {
                eprintln!("    Warning: Invalid time_unit '{}', using milliseconds", time_unit_str);
                TimeUnit::Milliseconds
            });

        let initial_delay = resolve_config_value(task.initial_delay, config)?;
        
        // Try to parse as shorthand duration (e.g., "5s", "10m") or plain number
        let initial_delay_millis = if let Some((value, parsed_unit)) = TimeUnit::parse_duration(&initial_delay) {
            // Has suffix like "2s", "500ms"
            parsed_unit.to_millis(value)
        } else {
            // Plain number, use time_unit parameter
            let initial_delay_value: u64 = initial_delay
                .parse()
                .unwrap_or_else(|_| {
                    eprintln!("    Warning: Invalid initial_delay '{}', using 0", initial_delay);
                    0
                });
            time_unit.to_millis(initial_delay_value)
        };

        // Resolve zone from config
        let zone_str = resolve_config_value(task.zone, config)?;
        let zone_display = if zone_str.to_lowercase() == "local" {
            "Local".to_string()
        } else {
            zone_str.clone()
        };

        match task.schedule_type {
            "cron" => {
                let cron_expr = resolve_config_value(task.schedule_value, config)?;
                println!("    Type: cron");
                println!("    Expression: {}", cron_expr);
                println!("    Zone: {} (timezone for cron evaluation)", zone_display);
                println!("    Initial delay: {}ms", initial_delay_millis);
                
                // Warn if time_unit is specified for cron (it has no effect)
                if time_unit_str.to_lowercase() != "milliseconds" {
                    println!("    ⚠️  Warning: time_unit parameter is ignored for cron expressions");
                    println!("        Cron uses absolute time (calendar-based), not intervals");
                }

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
                
                // Try to parse as shorthand duration (e.g., "5s", "10m")
                let (interval_value, effective_time_unit) = 
                    if let Some((value, parsed_unit)) = TimeUnit::parse_duration(&interval_str) {
                        println!("    Parsed shorthand: '{}' -> {} {:?}", interval_str, value, parsed_unit);
                        (value, parsed_unit)
                    } else {
                        // Fall back to parsing as number and using time_unit parameter
                        let value = interval_str
                            .parse::<u64>()
                            .map_err(|_| format!("Invalid interval value: {}", interval_str))?;
                        (value, time_unit)
                    };
                
                // Convert interval to milliseconds using effective time_unit
                let interval_millis = effective_time_unit.to_millis(interval_value);

                println!("    Type: {}", task.schedule_type);
                println!("    Interval: {} {:?} ({}ms)", interval_value, effective_time_unit, interval_millis);
                println!("    Initial delay: {}ms", initial_delay_millis);
                
                // Warn if zone is specified for interval-based tasks (it has no effect)
                if zone_str.to_lowercase() != "local" {
                    println!("    ⚠️  Warning: zone parameter is ignored for interval-based tasks (fixed_rate/fixed_delay)");
                    println!("        Interval tasks always use local system time");
                }

                let instance = task.instance.clone();
                let is_fixed_delay = task.schedule_type == "fixed_delay";

                // Use tokio::interval for reliable interval-based execution
                // This works much better than tokio-cron-scheduler for intervals
                let handle = tokio::spawn(async move {
                    // Initial delay
                    if initial_delay_millis > 0 {
                        tokio::time::sleep(std::time::Duration::from_millis(initial_delay_millis)).await;
                    }

                    let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval_millis));
                    
                    // Skip first tick (it fires immediately)
                    interval.tick().await;
                    
                    loop {
                        if is_fixed_delay {
                            // fixed_delay: wait for task completion before scheduling next
                            instance.run().await;
                            interval.tick().await;
                        } else {
                            // fixed_rate: schedule next tick regardless of task completion
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

    /// Register a single task
    async fn register_task_internal(
        scheduler: &mut JobScheduler,
        interval_handles: &mut Vec<tokio::task::JoinHandle<()>>,
        config: &Arc<Config>,
        task: ScheduledTask,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("  [REGISTER] {}", task.name);

        // Resolve time_unit from config (default: milliseconds)
        let time_unit_str = resolve_config_value(task.time_unit, config)?;
        let time_unit = TimeUnit::from_str(&time_unit_str)
            .unwrap_or_else(|| {
                eprintln!("    Warning: Invalid time_unit '{}', using milliseconds", time_unit_str);
                TimeUnit::Milliseconds
            });

        let initial_delay = resolve_config_value(task.initial_delay, config)?;
        
        // Try to parse as shorthand duration (e.g., "5s", "10m") or plain number
        let initial_delay_millis = if let Some((value, parsed_unit)) = TimeUnit::parse_duration(&initial_delay) {
            // Has suffix like "2s", "500ms"
            parsed_unit.to_millis(value)
        } else {
            // Plain number, use time_unit parameter
            let initial_delay_value: u64 = initial_delay
                .parse()
                .unwrap_or_else(|_| {
                    eprintln!("    Warning: Invalid initial_delay '{}', using 0", initial_delay);
                    0
                });
            time_unit.to_millis(initial_delay_value)
        };

        // Resolve zone from config
        let zone_str = resolve_config_value(task.zone, config)?;
        let zone_display = if zone_str.to_lowercase() == "local" {
            "Local".to_string()
        } else {
            zone_str.clone()
        };

        match task.schedule_type {
            "cron" => {
                let cron_expr = resolve_config_value(task.schedule_value, config)?;
                println!("    Type: cron");
                println!("    Expression: {}", cron_expr);
                println!("    Zone: {} (timezone for cron evaluation)", zone_display);
                println!("    Initial delay: {}ms", initial_delay_millis);
                
                // Warn if time_unit is specified for cron (it has no effect)
                if time_unit_str.to_lowercase() != "milliseconds" {
                    println!("    ⚠️  Warning: time_unit parameter is ignored for cron expressions");
                    println!("        Cron uses absolute time (calendar-based), not intervals");
                }

                let handler = task.handler;
                
                // Note: tokio-cron-scheduler uses system local timezone by default
                // For production with explicit timezone support, consider using chrono-tz
                // Currently zone parameter is informational only
                let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
                    Box::pin(async move {
                        handler();
                    })
                })?;

                scheduler.add(job).await?;
            }
            "fixed_rate" | "fixed_delay" => {
                let interval_str = resolve_config_value(task.schedule_value, config)?;
                
                // Try to parse as shorthand duration (e.g., "5s", "10m")
                let (interval_value, effective_time_unit) = 
                    if let Some((value, parsed_unit)) = TimeUnit::parse_duration(&interval_str) {
                        println!("    Parsed shorthand: '{}' -> {} {:?}", interval_str, value, parsed_unit);
                        (value, parsed_unit)
                    } else {
                        // Fall back to parsing as number and using time_unit parameter
                        let value = interval_str
                            .parse::<u64>()
                            .map_err(|_| format!("Invalid interval value: {}", interval_str))?;
                        (value, time_unit)
                    };
                
                // Convert interval to milliseconds using effective time_unit
                let interval_millis = effective_time_unit.to_millis(interval_value);

                println!("    Type: {}", task.schedule_type);
                println!("    Interval: {} {:?} ({}ms)", interval_value, effective_time_unit, interval_millis);
                println!("    Initial delay: {}ms", initial_delay_millis);
                
                // Warn if zone is specified for interval-based tasks (it has no effect)
                if zone_str.to_lowercase() != "local" {
                    println!("    ⚠️  Warning: zone parameter is ignored for interval-based tasks (fixed_rate/fixed_delay)");
                    println!("        Interval tasks always use local system time");
                }

                let handler = task.handler;
                let is_fixed_delay = task.schedule_type == "fixed_delay";

                // Use tokio::interval for reliable interval-based execution
                let handle = tokio::spawn(async move {
                    // Initial delay
                    if initial_delay_millis > 0 {
                        tokio::time::sleep(std::time::Duration::from_millis(initial_delay_millis)).await;
                    }

                    let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval_millis));
                    
                    // Skip first tick (it fires immediately)
                    interval.tick().await;
                    
                    loop {
                        if is_fixed_delay {
                            // fixed_delay: wait for task completion before scheduling next
                            handler();
                            interval.tick().await;
                        } else {
                            // fixed_rate: schedule next tick regardless of task completion
                            interval.tick().await;
                            let handler_clone = handler.clone();
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

    /// Build and start the scheduler
    pub async fn build(self) -> Result<SchedulerHandle, Box<dyn std::error::Error>> {
        let mut scheduler = JobScheduler::new().await?;
        let mut interval_handles = Vec::new();

        // If register_all was called, register all scheduled tasks
        if self.should_register_all {
            println!("Registering {} scheduled tasks...", SCHEDULED_TASKS.len());

            for task_fn in SCHEDULED_TASKS {
                let task = task_fn();

                // Check if task is enabled
                let enabled = resolve_config_value(task.enabled, &self.config)?;
                if enabled.to_lowercase() == "false" {
                    println!("  [DISABLED] {}", task.name);
                    continue;
                }

                let task_name = task.name;
                if let Err(e) = Self::register_task_internal(
                    &mut scheduler,
                    &mut interval_handles,
                    &self.config,
                    task
                ).await {
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
}