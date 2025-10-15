use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprLit, ExprPath, ItemFn, ItemImpl, Lit, Meta, MetaNameValue};

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
                return Some(format!(
                    "Invalid time suffix '{}' in {} for task '{}'.\n\
                     Valid suffixes: 'ms' (milliseconds), 's' (seconds), 'm' (minutes), 'h' (hours), 'd' (days)\n\
                     Example: '5s', '10m', '2h', '500ms'",
                    suffix, field_name, task_name
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
/// This macro works with both standalone functions and Runnable trait implementations.
/// 
/// # Examples
/// 
/// ## Standalone Functions (Auto-registered)
/// 
/// ```rust
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
/// ## Runnable Trait Implementation (Manual registration)
/// 
/// ```rust
/// use scheduled::{scheduled, Runnable};
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
/// let user_task = UserTask { name: "MyTask".to_string() };
/// 
/// SchedulerBuilder::with_toml("config/application.toml")?
///     .runnable(user_task)
///     .start()
///     .await?;
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
        return handle_scheduled_impl(args, input_impl);
    }
    
    // If neither works, provide helpful error
    compile_error("scheduled macro can only be applied to:\n  1. Async functions (for auto-registered tasks)\n  2. impl Runnable blocks (for manually registered tasks)")
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
    };

    TokenStream::from(expanded)
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
                                        _ => return Err(format!(
                                            "Invalid TimeUnit variant: TimeUnit::{}\n\
                                             Valid options: TimeUnit::Milliseconds, TimeUnit::Seconds, TimeUnit::Minutes, TimeUnit::Hours, TimeUnit::Days",
                                            last_segment.ident
                                        )),
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
            eprintln!("warning: time_unit parameter is ignored because '{}' already has a suffix in task '{}'", schedule_value_str, task_name);
            eprintln!("         = help: remove suffix (use: {} = \"{}\", time_unit = TimeUnit::...) or remove time_unit parameter", 
                schedule_type_str,
                schedule_value_str.chars().take_while(|c| c.is_ascii_digit()).collect::<String>()
            );
        }
        
        if delay_has_suffix && has_explicit_time_unit {
            eprintln!("warning: time_unit parameter is ignored for initial_delay because '{}' already has a suffix in task '{}'", initial_delay_str, task_name);
            eprintln!("         = note: initial_delay will use its own suffix, not time_unit");
        }
    }

    // Emit compile-time warnings for misused parameters
    if schedule_type_str == "cron" {
        // Warn if time_unit is specified for cron
        if let Some(ref tu) = time_unit {
            if tu.to_lowercase() != "milliseconds" {
                let display = time_unit_display.as_deref().unwrap_or(tu);
                eprintln!("warning: time_unit parameter {} is ignored for cron expressions in task '{}'", display, task_name);
                eprintln!("         = note: cron uses absolute time (calendar-based), not intervals");
            }
        }
    } else {
        // Warn if zone is specified for interval tasks
        if let Some(ref z) = zone {
            if z.to_lowercase() != "local" && !z.starts_with("${") {
                eprintln!("warning: zone parameter '{}' is ignored for interval-based tasks ({}) in task '{}'", z, schedule_type_str, task_name);
                eprintln!("         = note: interval tasks (fixed_rate/fixed_delay) always use local system time");
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
