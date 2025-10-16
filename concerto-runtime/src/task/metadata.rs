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
