use crate::task::ScheduledMethodMetadata;
use std::sync::Arc;

/// Trait for instances that have scheduled methods
/// This is automatically available for any type with `#[scheduled]` methods
pub trait ScheduledInstance: Send + Sync + 'static {
    /// Get all scheduled method metadata for this type
    fn scheduled_methods() -> Vec<ScheduledMethodMetadata>;
    
    /// Call a scheduled method by name
    fn call_scheduled_method(&self, method_name: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>>;
}

/// Type alias for method caller function
pub(crate) type MethodCaller = Arc<dyn Fn(&(dyn std::any::Any + Send + Sync), &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>> + Send + Sync>;

/// Wrapper for a registered instance with scheduled methods
pub(crate) struct RegisteredInstance {
    pub(crate) type_name: String,
    pub(crate) instance: Arc<dyn std::any::Any + Send + Sync>,
    pub(crate) methods: Vec<ScheduledMethodMetadata>,
    pub(crate) caller: MethodCaller,
}
