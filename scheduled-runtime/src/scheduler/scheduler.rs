use super::handle::SchedulerHandle;
use super::instance::RegisteredInstance;
use crate::config::resolve_config_value;
use crate::runnable::RunnableTask;
use crate::task::{ScheduledTask, ScheduledMethodMetadata};
use crate::time_unit::TimeUnit;
use config::Config;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

/// Configured scheduler ready to start
/// This struct holds all configuration and tasks but hasn't started yet
pub struct Scheduler {
    pub(crate) config: Arc<Config>,
    pub(crate) runnable_tasks: Vec<RunnableTask>,
    pub(crate) scheduled_tasks: Vec<ScheduledTask>,
    pub(crate) registered_instances: Vec<RegisteredInstance>,
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
                        tokio::task::spawn_blocking(move || {
                            instance.run();
                        }).await.ok();
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
                            let instance_clone = instance.clone();
                            tokio::task::spawn_blocking(move || {
                                instance_clone.run();
                            }).await.ok();
                            interval.tick().await;
                        } else {
                            interval.tick().await;
                            let instance_clone = instance.clone();
                            tokio::spawn(async move {
                                tokio::task::spawn_blocking(move || {
                                    instance_clone.run();
                                }).await.ok();
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
