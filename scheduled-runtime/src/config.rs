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
pub fn resolve_config_value(value: &str, config: &Config) -> Result<String, Box<dyn std::error::Error>> {
    if value.starts_with("${") && value.ends_with("}") {
        let inner = &value[2..value.len() - 1];
        
        // Check if there's a default value (e.g., ${app.interval:10})
        if let Some(colon_pos) = inner.find(':') {
            let key = &inner[..colon_pos];
            let default_value = &inner[colon_pos + 1..];
            
            match config.get_string(key) {
                Ok(resolved) => Ok(resolved),
                Err(_) => Ok(default_value.to_string()),
            }
        } else {
            let resolved = config.get_string(inner)?;
            Ok(resolved)
        }
    } else {
        Ok(value.to_string())
    }
}
