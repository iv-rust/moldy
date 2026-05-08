mod config;
mod copy;
mod error;

use clap::{Parser, ValueEnum};
use colored::*;
use error::{print_error, print_success, Result};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "moldy")]
#[command(version)]
#[command(about = "A minimal templating CLI for copying template directories", long_about = None)]
struct Cli {
    target: String,

    template_key: String,
}

fn expand_path(path: &str) -> PathBuf {
    let expanded = shellexpand::tilde(path);
    PathBuf::from(expanded.as_ref())
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        print_error(&e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let target = expand_path(&cli.target);
    let template_key = &cli.template_key;

    let config = config::load_config()?;

    let (_, source_path) = config::get_template_path(&config, template_key)?;

    copy::check_depth_restriction(&target)?;

    let mode = copy::determine_copy_mode(&target);

    let (files_copied, _) = copy::copy_template(&source_path, &target, mode)
        .map_err(|e| {
            if let error::MoldyError::CopyError(ref msg) = e {
                if msg.contains("User cancelled") {
                    return e;
                }
            }
            e
        })?;

    let target_display = target.display().to_string();
    print_success(&format!(
        "Created {} from template '{}' ({} files)",
        target_display.cyan(),
        template_key.cyan(),
        files_copied
    ));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path_tilde() {
        let path = expand_path("~/projects");
        assert!(path.starts_with(dirs::home_dir().unwrap()));
    }

    #[test]
    fn test_expand_path_absolute() {
        let path = expand_path("/absolute/path");
        assert_eq!(path, PathBuf::from("/absolute/path"));
    }
}
