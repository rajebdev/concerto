/// Time unit for interval-based scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
}

impl TimeUnit {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "milliseconds" | "millisecond" | "millis" | "milli" | "ms" => Some(TimeUnit::Milliseconds),
            "seconds" | "second" | "s" | "sec" => Some(TimeUnit::Seconds),
            "minutes" | "minute" | "m" | "min" => Some(TimeUnit::Minutes),
            "hours" | "hour" | "h" | "hr" => Some(TimeUnit::Hours),
            "days" | "day" | "d" => Some(TimeUnit::Days),
            _ => None,
        }
    }

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
        let time_unit = Self::from_str(unit_str)?;
        
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
    pub fn new(
        name: &'static str,
        schedule_type: &'static str,
        schedule_value: &'static str,
        initial_delay: &'static str,
        enabled: &'static str,
        time_unit: &'static str,
        zone: &'static str,
        handler: fn(),
    ) -> Self {
        Self {
            name,
            schedule_type,
            schedule_value,
            initial_delay,
            enabled,
            time_unit,
            zone,
            handler,
        }
    }
}
