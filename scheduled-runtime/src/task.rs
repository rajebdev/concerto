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
    /// Parse TimeUnit from string representation.
    /// Only accepts full lowercase enum names: "milliseconds", "seconds", "minutes", "hours", "days"
    /// For shorthand notations like "5s", "10m", use `parse_duration` instead.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "milliseconds" => Some(TimeUnit::Milliseconds),
            "seconds" => Some(TimeUnit::Seconds),
            "minutes" => Some(TimeUnit::Minutes),
            "hours" => Some(TimeUnit::Hours),
            "days" => Some(TimeUnit::Days),
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
