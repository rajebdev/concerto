use config::{Config, File, FileFormat};
use std::path::Path;

/// Load config from a specific TOML file
pub fn load_toml_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
    let config = Config::builder()
        .add_source(File::from(path.as_ref()).format(FileFormat::Toml))
        .add_source(config::Environment::with_prefix("APP").separator("_"))
        .build()?;
    Ok(config)
}

/// Load config from a specific YAML file
pub fn load_yaml_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
    let config = Config::builder()
        .add_source(File::from(path.as_ref()).format(FileFormat::Yaml))
        .add_source(config::Environment::with_prefix("APP").separator("_"))
        .build()?;
    Ok(config)
}

/// Resolve config placeholder like ${app.interval} or ${app.interval:default}
/// 
/// # Panics
/// 
/// Panics if the config key is not found and no default value is provided.
/// 
/// # Examples
/// 
/// ```text
/// // With default value - OK even if key not found
/// ${app.interval:5s}
/// 
/// // Without default value - MUST exist in config or will panic
/// ${app.interval}
/// ```
pub fn resolve_config_value(value: &str, config: &Config) -> Result<String, Box<dyn std::error::Error>> {
    if value.starts_with("${") && value.ends_with("}") {
        let inner = &value[2..value.len() - 1];
        
        // Check if there's a default value (e.g., ${app.interval:10})
        if let Some(colon_pos) = inner.find(':') {
            let key = &inner[..colon_pos];
            let default_value = &inner[colon_pos + 1..];
            
            if let Ok(resolved) = config.get_string(key) {
                Ok(resolved)
            } else {
                eprintln!("warning: config key '{}' not found, using default value '{}'", key, default_value);
                Ok(default_value.to_string())
            }
        } else {
            // No default value - key MUST exist
            match config.get_string(inner) {
                Ok(resolved) => Ok(resolved),
                Err(_) => {
                    Err(format!(
                        "Config key '{}' not found and no default value provided.\n\
                         \n\
                         To fix this error, either:\n\
                         1. Add the key to your config file:\n\
                            [app]\n\
                            {} = \"value\"\n\
                         \n\
                         2. Or provide a default value in the placeholder:\n\
                            ${{{}:default_value}}\n\
                         \n\
                         Example: ${{{0}:5s}} for a 5 second default",
                        inner, 
                        inner.split('.').next_back().unwrap_or(inner),
                        inner
                    ).into())
                }
            }
        }
    } else {
        Ok(value.to_string())
    }
}
