use crate::time_unit::TimeUnit;

/// Trait for extracting scheduling metadata from types annotated with #[scheduled]
pub trait ScheduledMetadata {
    fn schedule_type() -> &'static str;
    fn schedule_value() -> &'static str;
    fn initial_delay() -> &'static str;
    fn enabled() -> &'static str;
    fn time_unit() -> &'static str;
    fn zone() -> &'static str;
    
    /// Get the TimeUnit enum (used when time_unit is specified as TimeUnit::*)
    /// Returns None if time_unit was not explicitly set (uses default from string)
    fn time_unit_enum() -> Option<TimeUnit> {
        None
    }
}
