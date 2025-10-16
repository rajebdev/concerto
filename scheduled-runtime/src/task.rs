/// Time unit for interval-based scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
}

impl std::str::FromStr for TimeUnit {
    type Err = String;

    /// Parse TimeUnit from string representation.
    /// Only accepts full lowercase enum names: "milliseconds", "seconds", "minutes", "hours", "days"
    /// For shorthand notations like "5s", "10m", use `parse_duration` instead.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "milliseconds" => Ok(TimeUnit::Milliseconds),
            "seconds" => Ok(TimeUnit::Seconds),
            "minutes" => Ok(TimeUnit::Minutes),
            "hours" => Ok(TimeUnit::Hours),
            "days" => Ok(TimeUnit::Days),
            _ => Err(format!("Invalid time unit: {}", s)),
        }
    }
}

impl TimeUnit {
    pub fn to_millis(&self, value: u64) -> u64 {
        match self {
            TimeUnit::Milliseconds => value,
            TimeUnit::Seconds => value * 1000,
            TimeUnit::Minutes => value * 60_000,
            TimeUnit::Hours => value * 3_600_000,
            TimeUnit::Days => value * 86_400_000,
        }
    }
    
    /// Parse a duration string like "5s", "10m", "2h", "500ms"
    /// Returns (value, TimeUnit) if successful
    /// 
    /// Strict rules:
    /// - Only lowercase suffixes are accepted: "s", "m", "h", "ms" (NOT "S", "Sec", "MIN", etc.)
    /// - Format must be: <number><suffix> (e.g., "5s", "100ms")
    /// - No spaces allowed between number and suffix
    pub fn parse_duration(s: &str) -> Option<(u64, TimeUnit)> {
        let s = s.trim();
        
        // Try to find where the number ends and unit begins
        let mut split_pos = 0;
        for (i, c) in s.chars().enumerate() {
            if !c.is_ascii_digit() {
                split_pos = i;
                break;
            }
        }
        
        if split_pos == 0 || split_pos == s.len() {
            return None;
        }
        
        let (num_str, unit_str) = s.split_at(split_pos);
        let value = num_str.parse::<u64>().ok()?;
        
        // Strict lowercase-only suffix matching
        let time_unit = match unit_str {
            "ms" => TimeUnit::Milliseconds,
            "s" => TimeUnit::Seconds,
            "m" => TimeUnit::Minutes,
            "h" => TimeUnit::Hours,
            "d" => TimeUnit::Days,
            _ => return None, // Reject uppercase or other variants
        };
        
        Some((value, time_unit))
    }
}

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

/// Metadata for a scheduled method inside an impl block
#[derive(Debug, Clone, Copy)]
pub struct ScheduledMethodMetadata {
    pub method_name: &'static str,
    pub schedule_type: &'static str,
    pub schedule_value: &'static str,
    pub initial_delay: &'static str,
    pub enabled: &'static str,
    pub time_unit: &'static str,
    pub zone: &'static str,
}
