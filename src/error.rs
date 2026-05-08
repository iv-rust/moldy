use colored::*;
use std::path::PathBuf;

#[derive(Debug)]
pub enum MoldyError {
    ConfigMissing(PathBuf),
    ConfigParse(String),
    TemplateKeyMissing(String, Vec<String>),
    TemplatePathMissing(PathBuf),
    ParentDirMissing(PathBuf),
    CopyError(String),
    PermissionDenied(String),
    Usage(String),
}

impl std::fmt::Display for MoldyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoldyError::ConfigMissing(path) => {
                write!(f, "Config file not found at '{}'\nCreate one with: mkdir -p ~/.config/moldy && touch ~/.config/moldy/config.toml", path.display())
            }
            MoldyError::ConfigParse(msg) => {
                write!(f, "Failed to parse config: {}", msg)
            }
            MoldyError::TemplateKeyMissing(key, available) => {
                write!(f, "Template '{}' not found in config", key)?;
                if !available.is_empty() {
                    write!(f, "\nAvailable templates: {}", available.join(", "))?;
                }
                Ok(())
            }
            MoldyError::TemplatePathMissing(path) => {
                write!(f, "Template source path does not exist: '{}'", path.display())
            }
            MoldyError::ParentDirMissing(parent) => {
                write!(f, "Parent directory '{}' does not exist", parent.display())
            }
            MoldyError::CopyError(msg) => {
                write!(f, "Copy error: {}", msg)
            }
            MoldyError::PermissionDenied(msg) => {
                write!(f, "Permission denied: {}", msg)
            }
            MoldyError::Usage(msg) => {
                write!(f, "Usage: moldy <TARGET_DIRECTORY> <PATH_IN_CONFIG>\n\n{}", msg)
            }
        }
    }
}

impl std::error::Error for MoldyError {}

pub type Result<T> = std::result::Result<T, MoldyError>;

pub fn print_error(err: &MoldyError) {
    eprintln!("{} {}", "error:".red().bold(), err);
}

pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg.green());
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "!".yellow().bold(), msg.yellow());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_config_missing_error() {
        let err = MoldyError::ConfigMissing(PathBuf::from("/test/path"));
        let msg = err.to_string();
        assert!(msg.contains("Config file not found"));
        assert!(msg.contains("/test/path"));
    }

    #[test]
    fn test_template_key_missing_error() {
        let err = MoldyError::TemplateKeyMissing("react".to_string(), vec!["api".to_string(), "cli".to_string()]);
        let msg = err.to_string();
        assert!(msg.contains("Template 'react' not found"));
        assert!(msg.contains("api, cli"));
    }

    #[test]
    fn test_parent_dir_missing_error() {
        let err = MoldyError::ParentDirMissing(PathBuf::from("/nonexistent/parent"));
        let msg = err.to_string();
        assert!(msg.contains("Parent directory"));
        assert!(msg.contains("/nonexistent/parent"));
    }

    #[test]
    fn test_usage_error() {
        let err = MoldyError::Usage("No arguments provided".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Usage:"));
        assert!(msg.contains("No arguments provided"));
    }
}
