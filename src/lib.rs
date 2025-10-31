//! High-performance codebase flattening library
//!
//! This library provides functionality for flattening codebases into
//! markdown format with intelligent exclusion patterns based on gitignore templates.

pub mod config;
pub mod exclusions;

use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Configuration for codebase flattening
///
/// Controls which files and folders are processed and how they're displayed
pub struct FlattenConfig {
    /// Folders to skip during processing
    pub skip_folders: HashSet<String>,
    /// File extensions to skip
    pub skip_extensions: HashSet<String>,
    /// Whether to show skipped items in the output
    pub show_skipped: bool,
    /// Maximum file size to process in bytes
    pub max_file_size: u64,
    /// Whether to include hidden files and folders
    pub include_hidden: bool,
    /// Maximum directory traversal depth
    pub max_depth: usize,
}

impl Default for FlattenConfig {
    fn default() -> Self {
        Self {
            skip_folders: HashSet::new(),
            skip_extensions: HashSet::new(),
            show_skipped: false,
            max_file_size: 104857600, // 100MB
            include_hidden: false,
            max_depth: usize::MAX,
        }
    }
}

impl FlattenConfig {
    /// Creates a new FlattenConfig with default settings
    ///
    /// # Examples
    /// ```
    /// use flatten_rust::FlattenConfig;
    /// 
    /// let config = FlattenConfig::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    pub fn should_skip_path(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name()
            && let Some(name_str) = name.to_str() {
            // Skip hidden files unless explicitly included
            if !self.include_hidden && name_str.starts_with('.') {
                return true;
            }
            return self.skip_folders.contains(name_str);
        }
        false
    }

    pub fn should_skip_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension()
            && let Some(ext_str) = extension.to_str() {
            return self.skip_extensions.contains(ext_str);
        }
        false
    }
}

pub fn create_test_structure() -> anyhow::Result<tempfile::TempDir> {
    let temp_dir = tempfile::TempDir::new()?;

    // Create basic structure
    fs::create_dir_all(temp_dir.path().join("src"))?;
    fs::create_dir_all(temp_dir.path().join("tests"))?;
    fs::create_dir_all(temp_dir.path().join("node_modules"))?;

    // Create files
    fs::write(
        temp_dir.path().join("src/main.rs"),
        "fn main() { println!(\"Hello\"); }",
    )?;
    fs::write(
        temp_dir.path().join("src/lib.rs"),
        "pub fn hello() { \"world\" }",
    )?;
    fs::write(
        temp_dir.path().join("tests/integration.rs"),
        "#[test] fn test_integration() {}",
    )?;
    fs::write(temp_dir.path().join("README.md"), "# Test Project")?;
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )?;

    // Create a binary file
    fs::write(temp_dir.path().join("test.bin"), b"\x00\x01\x02\x03\x04")?;

    Ok(temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_config_skip_folders() {
        let mut config = FlattenConfig::new();
        config.skip_folders.insert("node_modules".to_string());
        config.skip_folders.insert(".git".to_string());

        assert!(config.should_skip_path(Path::new("/project/node_modules")));
        assert!(config.should_skip_path(Path::new("/project/.git")));
        assert!(!config.should_skip_path(Path::new("/project/src")));
    }

    #[test]
    fn test_config_skip_extensions() {
        let mut config = FlattenConfig::new();
        config.skip_extensions.insert("exe".to_string());
        config.skip_extensions.insert("bin".to_string());

        assert!(config.should_skip_file(Path::new("/project/test.exe")));
        assert!(config.should_skip_file(Path::new("/project/test.bin")));
        assert!(!config.should_skip_file(Path::new("/project/test.rs")));
    }

    #[test]
    fn test_create_test_structure() {
        let temp_dir = create_test_structure().unwrap();

        assert!(temp_dir.path().join("src").exists());
        assert!(temp_dir.path().join("tests").exists());
        assert!(temp_dir.path().join("node_modules").exists());
        assert!(temp_dir.path().join("src/main.rs").exists());
        assert!(temp_dir.path().join("README.md").exists());
    }

    #[test]
    fn test_file_content_reading() {
        let temp_dir = create_test_structure().unwrap();
        let main_rs_path = temp_dir.path().join("src/main.rs");

        let content = fs::read_to_string(&main_rs_path).unwrap();
        assert!(content.contains("fn main()"));
    }
}
