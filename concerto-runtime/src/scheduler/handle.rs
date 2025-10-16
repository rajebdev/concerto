use tokio_cron_scheduler::JobScheduler;

/// Handle for a running scheduler
/// Used to control and shutdown the scheduler
pub struct SchedulerHandle {
    pub(crate) cron_scheduler: JobScheduler,
    pub(crate) interval_handles: Vec<tokio::task::JoinHandle<()>>,
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
