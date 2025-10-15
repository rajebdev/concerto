use crate::task::ScheduledTask;

/// Global distributed slice for collecting scheduled tasks
#[linkme::distributed_slice]
pub static SCHEDULED_TASKS: [fn() -> ScheduledTask] = [..];
