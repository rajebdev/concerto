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
