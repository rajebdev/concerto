use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprLit, ExprPath, ItemFn, ItemImpl, ImplItem, Lit, Meta, MetaNameValue};

/// Helper to create compile_error! token stream
fn compile_error(message: &str) -> TokenStream {
    let error = syn::Error::new(proc_macro2::Span::call_site(), message);
    error.to_compile_error().into()
}

/// Check if a value string has a time unit suffix (e.g., "5s", "10m")
fn has_time_suffix(value: &str) -> bool {
    // Check for config placeholder
    if value.starts_with("${") {
        return false;
    }
    
    let trimmed = value.trim();
    // Check if ends with valid suffixes (strict lowercase only)
    trimmed.ends_with("ms") || 
    trimmed.ends_with("s") || 
    trimmed.ends_with("m") || 
    trimmed.ends_with("h") || 
    trimmed.ends_with("d")
}

/// Validate time suffix format (must be lowercase)
/// Returns Some(error_message) if invalid, None if valid
fn validate_time_suffix(value: &str, field_name: &str, task_name: &str) -> Option<String> {
    if value.starts_with("${") {
        return None; // Skip config placeholders
    }
    
    let trimmed = value.trim();
    
    // Find where number ends
    let mut split_pos = 0;
    for (i, c) in trimmed.chars().enumerate() {
        if !c.is_ascii_digit() {
            split_pos = i;
            break;
        }
    }
    
    if split_pos > 0 && split_pos < trimmed.len() {
        let suffix = &trimmed[split_pos..];
        
        // Check if uppercase or invalid format
        if suffix.chars().any(|c| c.is_uppercase()) {
            return Some(format!(
                "Invalid time suffix '{}' in {} for task '{}'.\n\
                 Only lowercase suffixes are allowed: 's', 'm', 'h', 'ms', 'd'\n\
                 Example: Use '5s' instead of '5S' or '5Sec'",
                suffix, field_name, task_name
            ));
        }
        
        // Validate it's a recognized suffix
        match suffix {
            "ms" | "s" | "m" | "h" | "d" => {},
            _ => {
                // Try to suggest closest match
                let suggestion = match suffix.to_lowercase().as_str() {
                    "sec" | "second" | "seconds" => "Did you mean 's' (seconds)?",
                    "min" | "minute" | "minutes" => "Did you mean 'm' (minutes)?",
                    "hr" | "hour" | "hours" => "Did you mean 'h' (hours)?",
                    "day" | "days" => "Did you mean 'd' (days)?",
                    "millisecond" | "milliseconds" | "millis" => "Did you mean 'ms' (milliseconds)?",
                    _ => "Valid suffixes: 'ms', 's', 'm', 'h', 'd'",
                };
                
                return Some(format!(
                    "Invalid time suffix '{}' in {} for task '{}'.\n\
                     \n\
                     help: {}\n\
                     \n\
                     Valid time suffixes:\n\
                     - 'ms' = milliseconds\n\
                     - 's'  = seconds\n\
                     - 'm'  = minutes\n\
                     - 'h'  = hours\n\
                     - 'd'  = days\n\
                     \n\
                     Examples: '5s', '10m', '2h', '500ms', '7d'",
                    suffix, field_name, task_name, suggestion
                ));
            }
        }
    }
    None
}

/// Validate config placeholder format and detect malformed patterns
/// Returns Some(error_message) if invalid, None if valid
fn validate_config_placeholder_format(value: &str, field_name: &str, task_name: &str) -> Option<String> {
    let trimmed = value.trim();
    
    // Pattern 1: Detect incomplete placeholder like "${xxx" (missing closing brace)
    if trimmed.starts_with("${") && !trimmed.ends_with("}") {
        return Some(format!(
            "Malformed config placeholder '{}' in {} for task '{}'.\n\
             \n\
             Missing closing brace '}}'.\n\
             \n\
             Correct format:\n\
             {} = \"${{config.key:default}}\"\n\
             \n\
             Examples:\n\
             - ${{app.interval:5000}}       (with default value)\n\
             - ${{app.enabled:true}}        (boolean config)",
            value, field_name, task_name, field_name
        ));
    }
    
    // Pattern 2: Detect mix of config + suffix like "${xxx}s" or "${xxx}ms"
    if trimmed.contains("${") && trimmed.contains("}") {
        let after_closing = trimmed.find("}").map(|pos| &trimmed[pos+1..]).unwrap_or("");
        if !after_closing.is_empty() && after_closing.trim() != "" {
            return Some(format!(
                "Invalid format '{}' in {} for task '{}'.\n\
                 \n\
                 Cannot mix config placeholder with time suffix.\n\
                 \n\
                 ❌ WRONG:\n\
                 {} = \"${{app.interval}}s\"     (config + suffix)\n\
                 {} = \"${{app.interval}}ms\"    (config + suffix)\n\
                 {} = \"${{app.interval}}sec\"   (config + suffix)\n\
                 \n\
                 ✅ CORRECT - Option 1 (config with default):\n\
                 {} = \"${{app.interval:5}}\", time_unit = TimeUnit::Seconds\n\
                 \n\
                 ✅ CORRECT - Option 2 (config returns value with suffix):\n\
                 {} = \"${{app.interval:5s}}\"   (config value already has suffix)\n\
                 \n\
                 ✅ CORRECT - Option 3 (hardcoded with suffix):\n\
                 {} = \"5s\"                     (no config)",
                value, field_name, task_name,
                field_name, field_name, field_name,
                field_name,
                field_name,
                field_name
            ));
        }
    }
    
    // Pattern 3: Detect random characters after closing brace (not valid suffixes)
    if trimmed.starts_with("${") && trimmed.contains("}") {
        if let Some(pos) = trimmed.find("}") {
            let after = &trimmed[pos+1..];
            // If there's content after }, it must be empty or whitespace
            if !after.trim().is_empty() {
                // Check if it looks like someone tried to add suffix
                if after.chars().any(|c| c.is_alphabetic()) {
                    return Some(format!(
                        "Invalid format '{}' in {} for task '{}'.\n\
                         \n\
                         Extra characters '{}' found after config placeholder.\n\
                         \n\
                         Cannot append time suffix to config placeholder.\n\
                         \n\
                         ✅ CORRECT:\n\
                         {} = \"${{app.interval:5000}}\"   (milliseconds by default)\n\
                         {} = \"${{app.interval:5s}}\"     (config value has suffix)\n\
                         {} = \"${{app.interval:5}}\", time_unit = TimeUnit::Seconds",
                        value, field_name, task_name, after,
                        field_name, field_name, field_name
                    ));
                }
            }
        }
    }
    
    None
}

/// Validate that value is not negative or zero (for intervals)
/// Returns Some(error_message) if invalid, None if valid
fn validate_positive_value(value: &str, field_name: &str, task_name: &str, allow_zero: bool) -> Option<String> {
    if value.starts_with("${") {
        return None; // Skip config placeholders (will be validated at runtime)
    }
    
    let trimmed = value.trim();
    
    // Extract numeric part
    let mut num_str = String::new();
    for c in trimmed.chars() {
        if c.is_ascii_digit() {
            num_str.push(c);
        } else {
            break;
        }
    }
    
    if let Ok(num) = num_str.parse::<i64>() {
        if num < 0 {
            return Some(format!(
                "Invalid {} value '{}' for task '{}'.\n\
                 Value cannot be negative.",
                field_name, value, task_name
            ));
        }
        
        if !allow_zero && num == 0 {
            return Some(format!(
                "Invalid {} value '{}' for task '{}'.\n\
                 Interval cannot be zero (would cause infinite loop).",
                field_name, value, task_name
            ));
        }
    }
    None
}

/// Scheduled task macro that supports Spring Boot-like scheduling
/// 
/// This macro works with standalone functions, Runnable trait implementations, and impl blocks with methods.
/// 
/// # Examples
/// 
/// ## Standalone Functions (Auto-registered)
/// 
/// ```rust,ignore
/// use scheduled_macro::scheduled;
/// 
/// #[scheduled(cron = "0 */5 * * * *")]
/// async fn my_cron_task() {
///     println!("Runs every 5 minutes");
/// }
/// 
/// #[scheduled(fixed_rate = "5s")]
/// async fn five_seconds_task() {
///     println!("Runs every 5 seconds");
/// }
/// ```
/// 
/// ## Methods inside impl blocks (Registered via .register())
/// 
/// ```rust,ignore
/// use scheduled_macro::scheduled;
/// use scheduled::SchedulerBuilder;
/// 
/// struct UserHandler {
///     name: String,
/// }
/// 
/// impl UserHandler {
///     #[scheduled(fixed_rate = "5s")]
///     async fn exe(&self) {
///         println!("{}: Running every 5s", self.name);
///     }
/// }
/// 
/// // Usage:
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let handler = UserHandler { name: "MyHandler".to_string() };
///     SchedulerBuilder::with_toml("config/application.toml")
///         .register(handler)
///         .build()
///         .start()
///         .await?;
///     Ok(())
/// }
/// ```
/// 
/// ## Runnable Trait Implementation (Manual registration)
/// 
/// ```rust,ignore
/// use scheduled_macro::scheduled;
/// use scheduled::{Runnable, SchedulerBuilder};
/// use std::pin::Pin;
/// use std::future::Future;
/// 
/// struct UserTask {
///     name: String,
/// }
/// 
/// #[scheduled(cron = "0 */5 * * * *")]
/// impl Runnable for UserTask {
///     fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
///         Box::pin(async move {
///             println!("Running {}", self.name);
///         })
///     }
/// }
/// 
/// // Usage:
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let user_task = UserTask { name: "MyTask".to_string() };
///     
///     SchedulerBuilder::with_toml("config/application.toml")
///         .register(user_task)
///         .start()
///         .await?;
///     Ok(())
/// }
/// ```
/// 
/// # Parameters
/// 
/// - `cron`: Cron expression for scheduling (e.g., "0 */5 * * * *")
/// - `fixed_rate`: Fixed interval between task executions
/// - `fixed_delay`: Fixed delay between task completions
/// - `time_unit`: Time unit (milliseconds, seconds, minutes, hours, days)
/// - `zone`: Timezone for cron expressions (e.g., "Asia/Jakarta", "UTC")
/// - `initial_delay`: Delay before first execution
/// - `enabled`: Enable/disable task (boolean or config placeholder)
#[proc_macro_attribute]
pub fn scheduled(args: TokenStream, input: TokenStream) -> TokenStream {
    // Try to parse as function first
    if let Ok(input_fn) = syn::parse::<ItemFn>(input.clone()) {
        return handle_scheduled_function(args, input_fn);
    }
    
    // Try to parse as impl block
    if let Ok(input_impl) = syn::parse::<ItemImpl>(input.clone()) {
        // Check if it has scheduled methods (indicated by having methods with #[scheduled])
        let has_scheduled_attrs = input_impl.items.iter().any(|item| {
            if let ImplItem::Fn(method) = item {
                method.attrs.iter().any(|attr| attr.path().is_ident("scheduled"))
            } else {
                false
            }
        });

        if has_scheduled_attrs {
            // This is an impl block with scheduled methods
            return handle_impl_with_scheduled_methods(args, input_impl);
        } else {
            // This is an impl Runnable block
            return handle_scheduled_impl(args, input_impl);
        }
    }
    
    // If neither works, provide helpful error
    compile_error("scheduled macro can only be applied to:\n  1. Async functions (for auto-registered tasks)\n  2. impl blocks with #[scheduled] methods\n  3. impl Runnable blocks (for manually registered tasks)")
}

fn handle_scheduled_function(args: TokenStream, input_fn: ItemFn) -> TokenStream {
    let attr_args = syn::parse_macro_input!(args with syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;

    let (schedule_type, schedule_value, initial_delay_str, enabled_str, time_unit_str, zone_str, time_unit_path) = 
        match parse_schedule_args(&attr_args, &fn_name.to_string()) {
            Ok(args) => args,
            Err(e) => return compile_error(&e),
        };

    // Generate unique registration function name
    let register_fn_name = syn::Ident::new(
        &format!("__register_scheduled_{}", fn_name),
        fn_name.span(),
    );
    
    // Generate dummy usage to force TimeUnit import (prevents unused import warning)
    let force_import = if let Some(ref path) = time_unit_path {
        let dummy_name = syn::Ident::new(
            &format!("__force_time_unit_import_{}", fn_name),
            fn_name.span(),
        );
        quote::quote! {
            #[allow(dead_code, non_upper_case_globals)]
            const #dummy_name: fn() -> ::scheduled::TimeUnit = || #path;
        }
    } else {
        quote::quote! {}
    };

    let expanded = quote! {
        #fn_vis #fn_sig {
            #fn_block
        }
        
        #force_import

        // Auto-registration using linkme
        #[::scheduled::scheduled_runtime::linkme::distributed_slice(::scheduled::scheduled_runtime::SCHEDULED_TASKS)]
        #[linkme(crate = ::scheduled::scheduled_runtime::linkme)]
        fn #register_fn_name() -> ::scheduled::scheduled_runtime::ScheduledTask {
            ::scheduled::scheduled_runtime::ScheduledTask {
                name: stringify!(#fn_name),
                schedule_type: #schedule_type,
                schedule_value: #schedule_value,
                initial_delay: #initial_delay_str,
                enabled: #enabled_str,
                time_unit: #time_unit_str,
                zone: #zone_str,
                handler: || {
                    ::tokio::spawn(async {
                        #fn_name().await;
                    });
                },
            }
        }
    };

    TokenStream::from(expanded)
}

fn handle_scheduled_impl(args: TokenStream, input_impl: ItemImpl) -> TokenStream {
    let attr_args = syn::parse_macro_input!(args with syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated);

    // Extract the type being implemented
    let impl_type = &input_impl.self_ty;
    let type_name = quote!(#impl_type).to_string().replace(" ", "");

    // ✅ STRICT VALIDATION: Check if implementing Runnable trait
    if let Some((_, trait_path, _)) = &input_impl.trait_ {
        let trait_name = quote!(#trait_path).to_string();
        
        // Check if it's implementing Runnable trait
        if !trait_name.contains("Runnable") {
            return compile_error(&format!(
                "Invalid use of #[scheduled] on impl block for type '{}'.\n\
                 \n\
                 The #[scheduled] macro on impl blocks can ONLY be used with 'impl Runnable'.\n\
                 \n\
                 ❌ WRONG:\n\
                 #[scheduled(...)]\n\
                 impl {} {{ ... }}                    // Regular impl block\n\
                 \n\
                 #[scheduled(...)]\n\
                 impl SomeTrait for {} {{ ... }}      // Other trait\n\
                 \n\
                 ✅ CORRECT:\n\
                 #[scheduled(...)]\n\
                 impl Runnable for {} {{\n\
                     fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {{\n\
                         Box::pin(async move {{\n\
                             // Your scheduled task logic\n\
                         }})\n\
                     }}\n\
                 }}\n\
                 \n\
                 💡 Alternative: Use #[scheduled] on standalone async functions for auto-registration:\n\
                 #[scheduled(...)]\n\
                 async fn my_task() {{\n\
                     // Your task logic\n\
                 }}",
                type_name, type_name, type_name, type_name
            ));
        }
    } else {
        // No trait being implemented (regular impl block)
        return compile_error(&format!(
            "Invalid use of #[scheduled] on regular impl block for type '{}'.\n\
             \n\
             The #[scheduled] macro cannot be used on regular impl blocks.\n\
             It must be used on 'impl Runnable for YourType'.\n\
             \n\
             ❌ WRONG:\n\
             #[scheduled(...)]\n\
             impl {} {{\n\
                 async fn some_method(&self) {{ ... }}\n\
             }}\n\
             \n\
             ✅ CORRECT Option 1 - Use impl Runnable:\n\
             #[scheduled(...)]\n\
             impl Runnable for {} {{\n\
                 fn run(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {{\n\
                     Box::pin(async move {{\n\
                         // Your scheduled task logic\n\
                     }})\n\
                 }}\n\
             }}\n\
             \n\
             ✅ CORRECT Option 2 - Use standalone function:\n\
             #[scheduled(...)]\n\
             async fn my_scheduled_task() {{\n\
                 // Your task logic - will be auto-registered\n\
             }}\n\
             \n\
             📖 See examples/runnable-trait.rs for complete examples.",
            type_name, type_name, type_name
        ));
    }

    let (schedule_type, schedule_value, initial_delay_str, enabled_str, time_unit_str, zone_str, time_unit_path) = 
        match parse_schedule_args(&attr_args, &type_name) {
            Ok(args) => args,
            Err(e) => return compile_error(&e),
        };
    
    // Generate time_unit_enum() implementation if TimeUnit was explicitly specified
    let time_unit_enum_impl = if let Some(ref path) = time_unit_path {
        quote::quote! {
            fn time_unit_enum() -> Option<::scheduled::TimeUnit> {
                Some(#path)
            }
            
            // Force TimeUnit import to be used (prevents unused import warning)
            #[allow(dead_code)]
            const __FORCE_TIME_UNIT_IMPORT: fn() -> ::scheduled::TimeUnit = || #path;
        }
    } else {
        quote::quote! {}
    };

    let expanded = quote! {
        #input_impl

        // Store the scheduling metadata as associated constants
        impl #impl_type {
            #[doc(hidden)]
            pub const __SCHEDULE_TYPE: &'static str = #schedule_type;
            #[doc(hidden)]
            pub const __SCHEDULE_VALUE: &'static str = #schedule_value;
            #[doc(hidden)]
            pub const __INITIAL_DELAY: &'static str = #initial_delay_str;
            #[doc(hidden)]
            pub const __ENABLED: &'static str = #enabled_str;
            #[doc(hidden)]
            pub const __TIME_UNIT: &'static str = #time_unit_str;
            #[doc(hidden)]
            pub const __ZONE: &'static str = #zone_str;
        }

        // Implementation of ScheduledMetadata trait to store schedule configuration
        impl ::scheduled::scheduled_runtime::ScheduledMetadata for #impl_type {
            fn schedule_type() -> &'static str { #schedule_type }
            fn schedule_value() -> &'static str { #schedule_value }
            fn initial_delay() -> &'static str { #initial_delay_str }
            fn enabled() -> &'static str { #enabled_str }
            fn time_unit() -> &'static str { #time_unit_str }
            fn zone() -> &'static str { #zone_str }
            
            #time_unit_enum_impl
        }

        // BONUS: Also implement ScheduledInstance for Runnable types
        // This allows using .register() for both method-based and Runnable-based tasks!
        impl ::scheduled::scheduled_runtime::ScheduledInstance for #impl_type {
            fn scheduled_methods() -> ::std::vec::Vec<::scheduled::scheduled_runtime::ScheduledMethodMetadata> {
                // Runnable types have no methods, but we create a single "run" method entry
                vec![
                    ::scheduled::scheduled_runtime::ScheduledMethodMetadata {
                        method_name: "run",
                        schedule_type: #schedule_type,
                        schedule_value: #schedule_value,
                        initial_delay: #initial_delay_str,
                        enabled: #enabled_str,
                        time_unit: #time_unit_str,
                        zone: #zone_str,
                    }
                ]
            }

            fn call_scheduled_method(&self, _method_name: &str) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()> + Send + '_>> {
                // Call the run() method from Runnable trait
                <Self as ::scheduled::scheduled_runtime::Runnable>::run(self)
            }
        }
    };

    TokenStream::from(expanded)
}

fn handle_impl_with_scheduled_methods(_args: TokenStream, mut input_impl: ItemImpl) -> TokenStream {
    let impl_type = &input_impl.self_ty;

    // Collect all scheduled methods and their metadata
    let mut scheduled_methods = Vec::new();
    let mut method_calls = Vec::new();

    for item in &mut input_impl.items {
        if let ImplItem::Fn(method) = item {
            // Find #[scheduled] attribute
            let scheduled_attr_idx = method.attrs.iter().position(|attr| attr.path().is_ident("scheduled"));
            
            if let Some(idx) = scheduled_attr_idx {
                let attr = method.attrs.remove(idx);
                let method_name = &method.sig.ident;
                let method_name_str = method_name.to_string();

                // Parse the attribute arguments
                let attr_meta: Meta = attr.parse_args().expect("Failed to parse scheduled attribute");
                let mut attr_args = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::new();
                
                // Handle the meta
                match attr_meta {
                    Meta::List(list) => {
                        // Parse the nested metas
                        attr_args = list.parse_args_with(
                            syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
                        ).expect("Failed to parse attribute arguments");
                    }
                    Meta::NameValue(nv) => {
                        attr_args.push(Meta::NameValue(nv));
                    }
                    Meta::Path(_) => {
                        // No arguments
                    }
                }

                let (schedule_type, schedule_value, initial_delay_str, enabled_str, time_unit_str, zone_str, _time_unit_path) = 
                    match parse_schedule_args(&attr_args, &method_name_str) {
                        Ok(args) => args,
                        Err(e) => return compile_error(&e),
                    };

                scheduled_methods.push(quote! {
                    ::scheduled::scheduled_runtime::ScheduledMethodMetadata {
                        method_name: stringify!(#method_name),
                        schedule_type: #schedule_type,
                        schedule_value: #schedule_value,
                        initial_delay: #initial_delay_str,
                        enabled: #enabled_str,
                        time_unit: #time_unit_str,
                        zone: #zone_str,
                    }
                });

                method_calls.push(quote! {
                    stringify!(#method_name) => ::std::boxed::Box::pin(self.#method_name())
                });
            }
        }
    }

    let expanded = quote! {
        #input_impl

        impl ::scheduled::scheduled_runtime::ScheduledInstance for #impl_type {
            fn scheduled_methods() -> ::std::vec::Vec<::scheduled::scheduled_runtime::ScheduledMethodMetadata> {
                vec![
                    #(#scheduled_methods),*
                ]
            }

            fn call_scheduled_method(&self, method_name: &str) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()> + Send + '_>> {
                match method_name {
                    #(#method_calls,)*
                    _ => ::std::boxed::Box::pin(async {}),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Internal macro to mark scheduled methods
#[proc_macro_attribute]
#[doc(hidden)]
pub fn __scheduled_method(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Just pass through the method as-is, this is a marker attribute
    input
}

fn parse_schedule_args(
    attr_args: &syn::punctuated::Punctuated<Meta, syn::Token![,]>,
    task_name: &str,
) -> Result<(String, String, String, String, String, String, Option<proc_macro2::TokenStream>), String> {
    let mut schedule_type = None;
    let mut schedule_value = None;
    let mut initial_delay = None;
    let mut enabled = None;
    let mut time_unit = None;
    let mut time_unit_path: Option<proc_macro2::TokenStream> = None; // Store the actual TimeUnit:: path
    let mut time_unit_display: Option<String> = None; // Store display string for warnings (e.g., "TimeUnit::Minutes")
    let mut zone = None;

    // Parse macro arguments using syn 2.0 API
    for arg in attr_args {
        match arg {
            Meta::NameValue(MetaNameValue { path, value, .. }) => {
                let path_str = path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default();

                match path_str.as_str() {
                    "cron" => {
                        if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = value {
                            schedule_type = Some("cron");
                            schedule_value = Some(s.value());
                        }
                    }
                    "fixed_rate" => {
                        schedule_type = Some("fixed_rate");
                        let value_str = match value {
                            Expr::Lit(ExprLit { lit: Lit::Int(i), .. }) => i.base10_digits().to_string(),
                            Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => s.value(),
                            _ => return Err("fixed_rate must be int or string".to_string()),
                        };
                        
                        // Validate config placeholder format
                        if let Some(err) = validate_config_placeholder_format(&value_str, "fixed_rate", task_name) {
                            return Err(err);
                        }
                        
                        schedule_value = Some(value_str);
                    }
                    "fixed_delay" => {
                        schedule_type = Some("fixed_delay");
                        let value_str = match value {
                            Expr::Lit(ExprLit { lit: Lit::Int(i), .. }) => i.base10_digits().to_string(),
                            Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => s.value(),
                            _ => return Err("fixed_delay must be int or string".to_string()),
                        };
                        
                        // Validate config placeholder format
                        if let Some(err) = validate_config_placeholder_format(&value_str, "fixed_delay", task_name) {
                            return Err(err);
                        }
                        
                        schedule_value = Some(value_str);
                    }
                    "initial_delay" => {
                        let value_str = match value {
                            Expr::Lit(ExprLit { lit: Lit::Int(i), .. }) => i.base10_digits().to_string(),
                            Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => s.value(),
                            _ => return Err("initial_delay must be int or string".to_string()),
                        };
                        
                        // Validate config placeholder format
                        if let Some(err) = validate_config_placeholder_format(&value_str, "initial_delay", task_name) {
                            return Err(err);
                        }
                        
                        initial_delay = Some(value_str);
                    }
                    "enabled" => {
                        enabled = Some(match value {
                            Expr::Lit(ExprLit { lit: Lit::Bool(b), .. }) => b.value.to_string(),
                            Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => s.value(),
                            _ => return Err("enabled must be bool or string".to_string()),
                        });
                    }
                    "time_unit" => {
                        time_unit = Some(match value {
                            Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => {
                                let value_str = s.value();
                                
                                // REJECT config placeholder for time_unit
                                if value_str.starts_with("${") {
                                    return Err(format!(
                                        "time_unit cannot use config placeholders like '{}'.\n\
                                         \n\
                                         ❌ WRONG:\n\
                                         time_unit = \"${{app.time_unit}}\"    (config not allowed)\n\
                                         \n\
                                         time_unit must be a compile-time constant TimeUnit enum.\n\
                                         \n\
                                         ✅ CORRECT - Use TimeUnit enum:\n\
                                         time_unit = TimeUnit::Seconds\n\
                                         time_unit = TimeUnit::Minutes\n\
                                         time_unit = TimeUnit::Hours\n\
                                         \n\
                                         ✅ BETTER - Use shorthand in the interval value:\n\
                                         fixed_rate = \"5s\"              (5 seconds)\n\
                                         fixed_rate = \"10m\"             (10 minutes)\n\
                                         fixed_rate = \"${{app.interval:5s}}\"  (config with suffix)\n\
                                         \n\
                                         Valid TimeUnit options:\n\
                                         - TimeUnit::Milliseconds\n\
                                         - TimeUnit::Seconds\n\
                                         - TimeUnit::Minutes\n\
                                         - TimeUnit::Hours\n\
                                         - TimeUnit::Days",
                                        value_str
                                    ));
                                }
                                
                                // REJECT any string literal for time_unit
                                return Err(format!(
                                    "time_unit does not accept string values like '{}'. Use TimeUnit enum instead:\n\
                                     Example: time_unit = TimeUnit::Seconds\n\
                                     Valid options: TimeUnit::Milliseconds, TimeUnit::Seconds, TimeUnit::Minutes, TimeUnit::Hours, TimeUnit::Days",
                                    value_str
                                ));
                            }
                            Expr::Path(ExprPath { path, .. }) => {
                                // Support TimeUnit::Days, TimeUnit::Hours, etc.
                                if let Some(last_segment) = path.segments.last() {
                                    let unit = last_segment.ident.to_string().to_lowercase();
                                    // Validate it's a valid TimeUnit variant
                                    match unit.as_str() {
                                        "milliseconds" | "seconds" | "minutes" | "hours" | "days" => {
                                            // Store the user's path (forces import in scope)
                                            time_unit_path = Some(quote::quote! { #path });
                                            // Store display string for warning (e.g., "TimeUnit::Minutes")
                                            // Remove spaces from token stream string representation
                                            let display_str = quote::quote! { #path }.to_string().replace(" ", "");
                                            time_unit_display = Some(display_str);
                                            unit
                                        },
                                        _ => {
                                            // Provide smart suggestions for common typos
                                            let suggestion = match unit.as_str() {
                                                "millisecond" | "millis" | "ms" => "Did you mean TimeUnit::Milliseconds?",
                                                "second" | "sec" | "s" => "Did you mean TimeUnit::Seconds?",
                                                "minute" | "min" | "m" => "Did you mean TimeUnit::Minutes?",
                                                "hour" | "hr" | "h" => "Did you mean TimeUnit::Hours?",
                                                "day" | "d" => "Did you mean TimeUnit::Days?",
                                                _ => "",
                                            };
                                            
                                            let help_text = if !suggestion.is_empty() {
                                                format!("\n   = help: {}", suggestion)
                                            } else {
                                                String::new()
                                            };
                                            
                                            return Err(format!(
                                                "Invalid TimeUnit variant: TimeUnit::{}{}\n\
                                                 \n\
                                                 Valid TimeUnit options:\n\
                                                 - TimeUnit::Milliseconds\n\
                                                 - TimeUnit::Seconds\n\
                                                 - TimeUnit::Minutes\n\
                                                 - TimeUnit::Hours\n\
                                                 - TimeUnit::Days",
                                                last_segment.ident, help_text
                                            ));
                                        }
                                    }
                                } else {
                                    return Err("Invalid time_unit path".to_string());
                                }
                            }
                            _ => return Err("time_unit must be a TimeUnit enum (e.g., TimeUnit::Seconds, TimeUnit::Minutes)".to_string()),
                        });
                    }
                    "zone" => {
                        if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = value {
                            zone = Some(s.value());
                        } else {
                            return Err("zone must be a string (e.g., 'Asia/Jakarta', 'UTC')".to_string());
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let schedule_type_str = schedule_type.expect("Must specify schedule type (cron, fixed_rate, or fixed_delay)");
    let schedule_value_str = schedule_value.expect("Must specify schedule value");

    let initial_delay_str = initial_delay.unwrap_or_else(|| "0".to_string());
    let enabled_str = enabled.unwrap_or_else(|| "true".to_string());
    let time_unit_str = time_unit.clone().unwrap_or_else(|| "milliseconds".to_string());
    let zone_str = zone.clone().unwrap_or_else(|| "local".to_string());

    // ========== COMPILE-TIME VALIDATIONS ==========
    
    // Rule 12: Validate time suffix format (must be lowercase)
    if schedule_type_str != "cron" {
        if let Some(err) = validate_time_suffix(&schedule_value_str, schedule_type_str, task_name) {
            return Err(err);
        }
        if let Some(err) = validate_time_suffix(&initial_delay_str, "initial_delay", task_name) {
            return Err(err);
        }
    }
    
    // Rule 10: Validate positive values
    if schedule_type_str != "cron" {
        if let Some(err) = validate_positive_value(&schedule_value_str, schedule_type_str, task_name, false) {
            return Err(err); // interval cannot be zero
        }
        if let Some(err) = validate_positive_value(&initial_delay_str, "initial_delay", task_name, true) {
            return Err(err); // delay can be zero
        }
    }
    
    // Rule 1, 3, 7: Warn if both suffix and time_unit are specified
    if schedule_type_str != "cron" {
        let value_has_suffix = has_time_suffix(&schedule_value_str);
        let delay_has_suffix = has_time_suffix(&initial_delay_str);
        let has_explicit_time_unit = time_unit.is_some() && time_unit_str.to_lowercase() != "milliseconds";
        
        if value_has_suffix && has_explicit_time_unit {
            let numeric_value = schedule_value_str.chars().take_while(|c| c.is_ascii_digit()).collect::<String>();
            eprintln!("\nwarning[W001]: time_unit parameter is ignored because '{}' already has a suffix", schedule_value_str);
            eprintln!("  --> task '{}'", task_name);
            eprintln!("   |");
            eprintln!("   = help: remove suffix OR remove time_unit parameter");
            eprintln!("   = note: suffix in value takes precedence over time_unit");
            eprintln!();
            eprintln!("   Fix options:");
            eprintln!("     1. Keep suffix:      {} = \"{}\"", schedule_type_str, schedule_value_str);
            eprintln!("     2. Use time_unit:    {} = \"{}\", time_unit = TimeUnit::...", schedule_type_str, numeric_value);
            eprintln!();
        }
        
        if delay_has_suffix && has_explicit_time_unit {
            eprintln!("\nwarning[W001]: time_unit parameter is ignored for initial_delay because '{}' already has a suffix", initial_delay_str);
            eprintln!("  --> task '{}'", task_name);
            eprintln!("   |");
            eprintln!("   = note: initial_delay will use its own suffix, not time_unit");
            eprintln!();
        }
    }

    // Emit compile-time warnings for misused parameters
    if schedule_type_str == "cron" {
        // Warn if time_unit is specified for cron
        if let Some(ref tu) = time_unit {
            if tu.to_lowercase() != "milliseconds" {
                let display = time_unit_display.as_deref().unwrap_or(tu);
                eprintln!("\nwarning[W002]: time_unit parameter {} is ignored for cron expressions", display);
                eprintln!("  --> task '{}'", task_name);
                eprintln!("   |");
                eprintln!("   = note: cron uses absolute calendar time, not intervals");
                eprintln!("   = help: remove time_unit parameter (it has no effect on cron schedules)");
                eprintln!();
            }
        }
    } else {
        // Warn if zone is specified for interval tasks
        if let Some(ref z) = zone {
            if z.to_lowercase() != "local" && !z.starts_with("${") {
                eprintln!("\nwarning[W003]: zone parameter '{}' is ignored for interval-based tasks ({})", z, schedule_type_str);
                eprintln!("  --> task '{}'", task_name);
                eprintln!("   |");
                eprintln!("   = note: interval tasks (fixed_rate/fixed_delay) always use local system time");
                eprintln!("   = help: use cron expression if you need timezone support");
                eprintln!();
            }
        }
    }

    Ok((
        schedule_type_str.to_string(),
        schedule_value_str,
        initial_delay_str,
        enabled_str,
        time_unit_str,
        zone_str,
        time_unit_path,
    ))
}
