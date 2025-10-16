/// Represents a scheduled task
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub name: &'static str,
    pub schedule_type: &'static str,
    pub schedule_value: &'static str,
    pub initial_delay: &'static str,
    pub enabled: &'static str,
    pub time_unit: &'static str,
    pub zone: &'static str,
    pub handler: fn(),
}

impl ScheduledTask {
    /// Create a new builder for ScheduledTask
    pub fn builder(name: &'static str, handler: fn()) -> ScheduledTaskBuilder {
        ScheduledTaskBuilder {
            name,
            schedule_type: "cron",
            schedule_value: "0 0 * * * *",
            initial_delay: "0",
            enabled: "true",
            time_unit: "seconds",
            zone: "UTC",
            handler,
        }
    }
}

/// Builder for ScheduledTask
pub struct ScheduledTaskBuilder {
    name: &'static str,
    schedule_type: &'static str,
    schedule_value: &'static str,
    initial_delay: &'static str,
    enabled: &'static str,
    time_unit: &'static str,
    zone: &'static str,
    handler: fn(),
}

impl ScheduledTaskBuilder {
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

    pub fn build(self) -> ScheduledTask {
        ScheduledTask {
            name: self.name,
            schedule_type: self.schedule_type,
            schedule_value: self.schedule_value,
            initial_delay: self.initial_delay,
            enabled: self.enabled,
            time_unit: self.time_unit,
            zone: self.zone,
            handler: self.handler,
        }
    }
}
