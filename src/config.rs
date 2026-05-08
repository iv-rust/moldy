use crate::error::{MoldyError, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub templates: std::collections::HashMap<String, String>,
}

fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".config").join("moldy").join("moldy.toml")
}

pub fn load_config() -> Result<Config> {
    let config_path = get_config_path();

    if !config_path.exists() {
        return Err(MoldyError::ConfigMissing(config_path));
    }

    let contents = fs::read_to_string(&config_path)
        .map_err(|e| MoldyError::ConfigParse(format!("Failed to read config file: {}", e)))?;

    let config: Config = toml::from_str(&contents)
        .map_err(|e| MoldyError::ConfigParse(format!("TOML parse error: {}", e)))?;

    Ok(config)
}

pub fn get_template_path(config: &Config, key: &str) -> Result<(String, PathBuf)> {
    let path_str = config.templates.get(key).ok_or_else(|| {
        let available: Vec<String> = config.templates.keys().cloned().collect();
        MoldyError::TemplateKeyMissing(key.to_string(), available)
    })?;

    let path = PathBuf::from(path_str);

    if !path.exists() {
        return Err(MoldyError::TemplatePathMissing(path));
    }

    Ok((key.to_string(), path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config(dir: &TempDir, content: &str) -> PathBuf {
        let config_dir = dir.path().join(".config").join("moldy");
        fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("moldy.toml");
        fs::write(&config_path, content).unwrap();
        config_path
    }

    #[test]
    fn test_load_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_content = r#"
[templates]
react = "/home/user/templates/react"
api = "/home/user/templates/api"
"#;
        let config_path = create_test_config(&temp_dir, config_content);

        // We can't easily test the full load_config since it uses dirs::home_dir
        // But we can test parsing
        let config: Config = toml::from_str(config_content).unwrap();
        assert_eq!(config.templates.len(), 2);
        assert!(config.templates.contains_key("react"));
        assert!(config.templates.contains_key("api"));
    }

    #[test]
    fn test_get_template_path_valid() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("my-template");
        fs::create_dir_all(&template_dir).unwrap();

        let mut config = Config::default();
        config.templates.insert(
            "mytemplate".to_string(),
            template_dir.to_string_lossy().to_string(),
        );

        let (key, path) = get_template_path(&config, "mytemplate").unwrap();
        assert_eq!(key, "mytemplate");
        assert_eq!(path, template_dir);
    }

    #[test]
    fn test_get_template_path_missing_key() {
        let config = Config::default();
        let result = get_template_path(&config, "nonexistent");
        assert!(matches!(result, Err(MoldyError::TemplateKeyMissing(_, _))));
    }

    #[test]
    fn test_get_template_path_missing_source() {
        let mut config = Config::default();
        config
            .templates
            .insert("bad".to_string(), "/nonexistent/path".to_string());

        let result = get_template_path(&config, "bad");
        assert!(matches!(result, Err(MoldyError::TemplatePathMissing(_))));
    }
}
