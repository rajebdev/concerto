use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprLit, ExprPath, ItemFn, ItemImpl, Lit, Meta, MetaNameValue};

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
    panic!("scheduled macro can only be applied to:\n  1. Async functions (for auto-registered tasks)\n  2. impl Runnable blocks (for manually registered tasks)");
}

fn handle_scheduled_function(args: TokenStream, input_fn: ItemFn) -> TokenStream {
    let attr_args = syn::parse_macro_input!(args with syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;

    let (schedule_type, schedule_value, initial_delay_str, enabled_str, time_unit_str, zone_str) = 
        parse_schedule_args(&attr_args, &fn_name.to_string());

    // Generate unique registration function name
    let register_fn_name = syn::Ident::new(
        &format!("__register_scheduled_{}", fn_name),
        fn_name.span(),
    );

    let expanded = quote! {
        #fn_vis #fn_sig {
            #fn_block
        }

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

    let (schedule_type, schedule_value, initial_delay_str, enabled_str, time_unit_str, zone_str) = 
        parse_schedule_args(&attr_args, &type_name);

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
        }
    };

    TokenStream::from(expanded)
}

fn parse_schedule_args(
    attr_args: &syn::punctuated::Punctuated<Meta, syn::Token![,]>,
    task_name: &str,
) -> (String, String, String, String, String, String) {
    let mut schedule_type = None;
    let mut schedule_value = None;
    let mut initial_delay = None;
    let mut enabled = None;
    let mut time_unit = None;
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
                        schedule_value = Some(match value {
                            Expr::Lit(ExprLit { lit: Lit::Int(i), .. }) => i.base10_digits().to_string(),
                            Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => s.value(),
                            _ => panic!("fixed_rate must be int or string"),
                        });
                    }
                    "fixed_delay" => {
                        schedule_type = Some("fixed_delay");
                        schedule_value = Some(match value {
                            Expr::Lit(ExprLit { lit: Lit::Int(i), .. }) => i.base10_digits().to_string(),
                            Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => s.value(),
                            _ => panic!("fixed_delay must be int or string"),
                        });
                    }
                    "initial_delay" => {
                        initial_delay = Some(match value {
                            Expr::Lit(ExprLit { lit: Lit::Int(i), .. }) => i.base10_digits().to_string(),
                            Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => s.value(),
                            _ => panic!("initial_delay must be int or string"),
                        });
                    }
                    "enabled" => {
                        enabled = Some(match value {
                            Expr::Lit(ExprLit { lit: Lit::Bool(b), .. }) => b.value.to_string(),
                            Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => s.value(),
                            _ => panic!("enabled must be bool or string"),
                        });
                    }
                    "time_unit" => {
                        time_unit = Some(match value {
                            Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => s.value(),
                            Expr::Path(ExprPath { path, .. }) => {
                                // Support TimeUnit::Days, TimeUnit::Hours, etc.
                                if let Some(last_segment) = path.segments.last() {
                                    last_segment.ident.to_string().to_lowercase()
                                } else {
                                    panic!("Invalid time_unit path");
                                }
                            }
                            _ => panic!("time_unit must be a string or TimeUnit::* constant (e.g., TimeUnit::Days)"),
                        });
                    }
                    "zone" => {
                        if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = value {
                            zone = Some(s.value());
                        } else {
                            panic!("zone must be a string (e.g., 'Asia/Jakarta', 'UTC')");
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

    // Emit compile-time warnings for misused parameters
    if schedule_type_str == "cron" {
        // Warn if time_unit is specified for cron
        if let Some(ref tu) = time_unit {
            if tu.to_lowercase() != "milliseconds" {
                eprintln!(
                    "warning: time_unit parameter '{}' is ignored for cron expressions in task '{}'",
                    tu, task_name
                );
                eprintln!("         cron uses absolute time (calendar-based), not intervals");
            }
        }
    } else {
        // Warn if zone is specified for interval tasks
        if let Some(ref z) = zone {
            if z.to_lowercase() != "local" && !z.starts_with("${") {
                eprintln!(
                    "warning: zone parameter '{}' is ignored for interval-based tasks ({}) in task '{}'",
                    z, schedule_type_str, task_name
                );
                eprintln!("         interval tasks (fixed_rate/fixed_delay) always use local system time");
            }
        }
    }

    (
        schedule_type_str.to_string(),
        schedule_value_str,
        initial_delay_str,
        enabled_str,
        time_unit_str,
        zone_str,
    )
}
