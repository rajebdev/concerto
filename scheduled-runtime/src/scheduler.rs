use crate::config::{load_toml_config, load_yaml_config, resolve_config_value};
use crate::registry::SCHEDULED_TASKS;
use crate::runnable::{create_runnable_task, Runnable, RunnableTask, ScheduledMetadata};
use crate::task::{ScheduledTask, TimeUnit};
use config::Config;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

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
}

impl Scheduler {
    /// Start the scheduler with all registered tasks
    /// Returns a SchedulerHandle that can be used to shutdown the scheduler
    pub async fn start(self) -> Result<SchedulerHandle, Box<dyn std::error::Error>> {
        let total_tasks = self.runnable_tasks.len() + self.scheduled_tasks.len();
        println!("Starting scheduler with {} total tasks...", total_tasks);

        let mut scheduler = JobScheduler::new().await?;
        let mut interval_handles = Vec::new();

        // Register runnable tasks (manual registered via .runnable())
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

        let time_unit_str = resolve_config_value(task.time_unit, config)?;
        let time_unit = TimeUnit::from_str(&time_unit_str).unwrap_or_else(|| {
            eprintln!(
                "    Warning: Invalid time_unit '{}', using milliseconds",
                time_unit_str
            );
            TimeUnit::Milliseconds
        });

        let initial_delay = resolve_config_value(task.initial_delay, config)?;
        let initial_delay_millis = if let Some((value, parsed_unit)) =
            TimeUnit::parse_duration(&initial_delay)
        {
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
        };

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

                let (interval_value, effective_time_unit) =
                    if let Some((value, parsed_unit)) = TimeUnit::parse_duration(&interval_str) {
                        println!(
                            "    Parsed shorthand: '{}' -> {} {:?}",
                            interval_str, value, parsed_unit
                        );
                        (value, parsed_unit)
                    } else {
                        let value = interval_str
                            .parse::<u64>()
                            .map_err(|_| format!("Invalid interval value: {}", interval_str))?;
                        (value, time_unit)
                    };

                let interval_millis = effective_time_unit.to_millis(interval_value);

                println!("    Type: {}", task.schedule_type);
                println!(
                    "    Interval: {} {:?} ({}ms)",
                    interval_value, effective_time_unit, interval_millis
                );
                println!("    Initial delay: {}ms", initial_delay_millis);

                if zone_str.to_lowercase() != "local" {
                    println!("    ⚠️  Warning: zone parameter is ignored for interval-based tasks (fixed_rate/fixed_delay)");
                    println!("        Interval tasks always use local system time");
                }

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

        let time_unit_str = resolve_config_value(task.time_unit, config)?;
        let time_unit = TimeUnit::from_str(&time_unit_str).unwrap_or_else(|| {
            eprintln!(
                "    Warning: Invalid time_unit '{}', using milliseconds",
                time_unit_str
            );
            TimeUnit::Milliseconds
        });

        let initial_delay = resolve_config_value(task.initial_delay, config)?;
        let initial_delay_millis = if let Some((value, parsed_unit)) =
            TimeUnit::parse_duration(&initial_delay)
        {
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
        };

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

                if time_unit_str.to_lowercase() != "milliseconds" {
                    println!("    ⚠️  Warning: time_unit parameter is ignored for cron expressions");
                    println!("        Cron uses absolute time (calendar-based), not intervals");
                }

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

                let (interval_value, effective_time_unit) =
                    if let Some((value, parsed_unit)) = TimeUnit::parse_duration(&interval_str) {
                        println!(
                            "    Parsed shorthand: '{}' -> {} {:?}",
                            interval_str, value, parsed_unit
                        );
                        (value, parsed_unit)
                    } else {
                        let value = interval_str
                            .parse::<u64>()
                            .map_err(|_| format!("Invalid interval value: {}", interval_str))?;
                        (value, time_unit)
                    };

                let interval_millis = effective_time_unit.to_millis(interval_value);

                println!("    Type: {}", task.schedule_type);
                println!(
                    "    Interval: {} {:?} ({}ms)",
                    interval_value, effective_time_unit, interval_millis
                );
                println!("    Initial delay: {}ms", initial_delay_millis);

                if zone_str.to_lowercase() != "local" {
                    println!("    ⚠️  Warning: zone parameter is ignored for interval-based tasks (fixed_rate/fixed_delay)");
                    println!("        Interval tasks always use local system time");
                }

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
}

/// Builder for the scheduler
pub struct SchedulerBuilder {
    config: Arc<Config>,
    runnable_tasks: Vec<RunnableTask>,
}

impl SchedulerBuilder {
    /// Create a new scheduler builder with default config (empty)
    pub fn new() -> Self {
        Self {
            config: Arc::new(Config::default()),
            runnable_tasks: Vec::new(),
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
        }
    }

    /// Create with custom config
    pub fn with_config(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            runnable_tasks: Vec::new(),
        }
    }

    /// Register a runnable task instance
    ///
    /// # Example
    ///
    /// ```rust
    /// let user_task = UserTask::new();
    ///
    /// let scheduler = SchedulerBuilder::with_toml("config/application.toml")?
    ///     .runnable(user_task)
    ///     .build();
    ///
    /// scheduler.start().await?;
    /// ```
    pub fn runnable<T>(mut self, instance: T) -> Self
    where
        T: Runnable + ScheduledMetadata + 'static,
    {
        let task = create_runnable_task(instance);
        self.runnable_tasks.push(task);
        self
    }

    /// Build the scheduler (does not start it yet)
    ///
    /// This will:
    /// - Collect all runnable tasks registered via `.runnable()`
    /// - Auto-discover all tasks marked with `#[scheduled]` macro
    /// - Return a `Scheduler` ready to start
    ///
    /// # Example
    ///
    /// ```rust
    /// let scheduler = SchedulerBuilder::new()
    ///     .with_toml("config.toml")?
    ///     .runnable(MyTask)
    ///     .build();  // <- No error here, just setup
    ///
    /// scheduler.start().await?;  // <- Errors happen here
    /// ```
    pub fn build(self) -> Scheduler {
        // Collect scheduled tasks from registry (auto-discovered #[scheduled] functions)
        let scheduled_tasks: Vec<ScheduledTask> = SCHEDULED_TASKS.iter().map(|f| f()).collect();

        println!(
            "Building scheduler with {} runnable tasks and {} scheduled tasks",
            self.runnable_tasks.len(),
            scheduled_tasks.len()
        );

        Scheduler {
            config: self.config,
            runnable_tasks: self.runnable_tasks,
            scheduled_tasks,
        }
    }
}
