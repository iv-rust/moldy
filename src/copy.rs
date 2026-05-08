use crate::error::{MoldyError, Result};
use colored::*;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn check_depth_restriction(target: &Path) -> Result<()> {
    if target.exists() {
        return Ok(());
    }

    let parent = target.parent()
        .ok_or_else(|| MoldyError::ParentDirMissing(PathBuf::from(".")))?;

    if !parent.exists() {
        return Err(MoldyError::ParentDirMissing(parent.to_path_buf()));
    }

    Ok(())
}

pub fn determine_copy_mode(target: &Path) -> CopyMode {
    if target.exists() && target.is_dir() {
        CopyMode::IntoTarget
    } else {
        CopyMode::CreateTarget
    }
}

#[derive(Debug, PartialEq)]
pub enum CopyMode {
    IntoTarget,
    CreateTarget,
}

pub fn copy_template(source: &Path, target: &Path, mode: CopyMode) -> Result<(usize, usize)> {
    match mode {
        CopyMode::IntoTarget => {
            let template_name = source.file_name()
                .ok_or_else(|| MoldyError::CopyError("Source has no file name".to_string()))?;
            let dest_dir = target.join(template_name);
            copy_directory(source, &dest_dir)
        }
        CopyMode::CreateTarget => {
            fs::create_dir_all(target)
                .map_err(|e| MoldyError::PermissionDenied(format!("Failed to create directory: {}", e)))?;
            copy_directory_contents(source, target)
        }
    }
}

fn copy_directory(source: &Path, dest: &Path) -> Result<(usize, usize)> {
    if !dest.exists() {
        fs::create_dir_all(dest)
            .map_err(|e| MoldyError::PermissionDenied(format!("Failed to create directory: {}", e)))?;
    }
    copy_directory_contents(source, dest)
}

fn copy_directory_contents(source: &Path, dest: &Path) -> Result<(usize, usize)> {
    let mut files_copied = 0;
    let mut dirs_created = 0;

    for entry in WalkDir::new(source).min_depth(1) {
        let entry = entry.map_err(|e| MoldyError::CopyError(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();
        let relative = path.strip_prefix(source)
            .map_err(|e| MoldyError::CopyError(format!("Failed to get relative path: {}", e)))?;
        let dest_path = dest.join(relative);

        let file_type = entry.file_type();

        if file_type.is_symlink() {
            let link_target = fs::read_link(path)
                .map_err(|e| MoldyError::CopyError(format!("Failed to read symlink: {}", e)))?;
            if dest_path.exists() {
                prompt_overwrite(&dest_path)?;
                fs::remove_file(&dest_path).ok();
            }
            create_symlink(&link_target, &dest_path)?;
            files_copied += 1;
        } else if file_type.is_dir() {
            if !dest_path.exists() {
                fs::create_dir_all(&dest_path)
                    .map_err(|e| MoldyError::PermissionDenied(format!("Failed to create directory: {}", e)))?;
                dirs_created += 1;
            }
        } else if file_type.is_file() {
            if dest_path.exists() {
                prompt_overwrite(&dest_path)?;
            }
            fs::copy(path, &dest_path)
                .map_err(|e| MoldyError::PermissionDenied(format!("Failed to copy file: {}", e)))?;
            files_copied += 1;
        }
    }

    Ok((files_copied, dirs_created))
}

#[cfg(unix)]
fn create_symlink(link_target: &Path, dest_path: &Path) -> Result<()> {
    std::os::unix::fs::symlink(link_target, dest_path)
        .map_err(|e| MoldyError::PermissionDenied(format!("Failed to create symlink: {}", e)))
}

#[cfg(windows)]
fn create_symlink(link_target: &Path, dest_path: &Path) -> Result<()> {
    if link_target.is_dir() {
        std::os::windows::fs::symlink_dir(link_target, dest_path)
            .map_err(|e| MoldyError::PermissionDenied(format!("Failed to create symlink: {}", e)))
    } else {
        std::os::windows::fs::symlink_file(link_target, dest_path)
            .map_err(|e| MoldyError::PermissionDenied(format!("Failed to create symlink: {}", e)))
    }
}

fn prompt_overwrite(path: &Path) -> Result<()> {
    let path_str = path.display().to_string();
    print!("{} '{}' already exists. Overwrite? [y/N] ", "!".yellow().bold(), path_str.yellow());
    io::stdout().flush()
        .map_err(|e| MoldyError::CopyError(format!("Failed to flush stdout: {}", e)))?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .map_err(|e| MoldyError::CopyError(format!("Failed to read input: {}", e)))?;

    let response = input.trim().to_lowercase();
    if response != "y" && response != "yes" {
        return Err(MoldyError::CopyError("User cancelled overwrite".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_check_depth_restriction_target_exists() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("existing");
        fs::create_dir_all(&target).unwrap();

        let result = check_depth_restriction(&target);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_depth_restriction_parent_exists() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("new_dir");

        let result = check_depth_restriction(&target);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_depth_restriction_parent_missing() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("nonexistent1").join("nonexistent2");

        let result = check_depth_restriction(&target);
        assert!(matches!(result, Err(MoldyError::ParentDirMissing(_))));
    }

    #[test]
    fn test_determine_copy_mode_target_exists() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("existing");
        fs::create_dir_all(&target).unwrap();

        let mode = determine_copy_mode(&target);
        assert_eq!(mode, CopyMode::IntoTarget);
    }

    #[test]
    fn test_determine_copy_mode_target_missing() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("new_dir");

        let mode = determine_copy_mode(&target);
        assert_eq!(mode, CopyMode::CreateTarget);
    }

    #[test]
    fn test_copy_template_into_target() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source");
        let target = temp_dir.path().join("target");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(source.join("file.txt"), "content").unwrap();

        let mode = determine_copy_mode(&target);
        let result = copy_template(&source, &target, mode).unwrap();

        assert!(target.join("source").join("file.txt").exists());
        assert_eq!(result.0, 1); // 1 file copied
    }

    #[test]
    fn test_copy_template_create_target() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source");
        let target = temp_dir.path().join("new_target");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("file.txt"), "content").unwrap();

        let mode = determine_copy_mode(&target);
        let result = copy_template(&source, &target, mode).unwrap();

        assert!(target.join("file.txt").exists());
        assert_eq!(result.0, 1); // 1 file copied
    }
}
