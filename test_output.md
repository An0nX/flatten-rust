### DIRECTORY /home/whoami/projects/flatten-rust/src FOLDER STRUCTURE ###
📁 src/
        📄 main.rs
        📄 lib.rs
### DIRECTORY /home/whoami/projects/flatten-rust/src FOLDER STRUCTURE ###

### DIRECTORY /home/whoami/projects/flatten-rust/src FLATTENED CONTENT ###
### /home/whoami/projects/flatten-rust/src/main.rs BEGIN ###
use anyhow::{Context, Result};
use clap::Parser;
use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use memmap2::MmapOptions;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

static FOLDER: Emoji<'_, '_> = Emoji("📁", "DIR");
static FILE: Emoji<'_, '_> = Emoji("📄", "FILE");
static SKIP: Emoji<'_, '_> = Emoji("⏭️", "SKIP");
static ROCKET: Emoji<'_, '_> = Emoji("🚀", "=>");

#[derive(Parser)]
#[command(name = "flatten-rust")]
#[command(about = "High-performance codebase flattening tool")]
#[command(version)]
struct Args {
    /// Base folders to process
    #[arg(long = "folders", num_args = 1..)]
    folders: Vec<PathBuf>,

    /// Folders to skip during processing (supports glob patterns)
    #[arg(long = "skip-folders", num_args = 0.., default_values = [
        "node_modules", ".git", "target", "dist", "build", "vendor", ".vscode", ".idea",
        "__pycache__", ".pytest_cache", ".mypy_cache", ".tox", "venv", ".venv", "env",
        ".next", ".nuxt", ".output", ".angular", "coverage", ".nyc_output", "htmlcov",
        ".coverage", "site-packages", "eggs", ".eggs", "pip-wheel-metadata",
        "cmake-build-debug", "cmake-build-release", ".gradle", ".idea", ".vs",
        "packages", ".pnpm-store", ".npm", ".yarn", ".yarn-integrity",
        "obj", "bin", "Debug", "Release", "x64", "x86", "out", ".next", ".nuxt"
    ])]
    skip_folders: Vec<String>,

    /// Print system instructions
    #[arg(long = "system_instructions")]
    system_instructions: bool,

    /// Output file name (default: codebase.md)
    #[arg(long = "output", default_value = "codebase.md")]
    output: PathBuf,

    /// Show skipped folders in tree structure
    #[arg(long = "show-skipped")]
    show_skipped: bool,

    /// Number of parallel file processing threads
    #[arg(long = "threads", default_value = "0")]
    threads: usize,

    /// Maximum file size to process in bytes (0 = unlimited)
    #[arg(long = "max-file-size", default_value = "104857600")]
    max_file_size: u64,

    /// Binary file extensions to skip (supports glob patterns)
    #[arg(long = "skip-extensions", num_args = 0.., default_values = [
        "exe", "dll", "so", "dylib", "bin", "img", "iso", "zip", "tar", "gz", "7z", "rar",
        "bz2", "xz", "deb", "rpm", "dmg", "pkg", "msi", "apk", "ipa", "msi",
        "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "ico", "svg", "psd", "ai",
        "mp3", "mp4", "avi", "mov", "wmv", "flv", "webm", "mkv", "wav", "flac", "ogg",
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp",
        "ttf", "otf", "woff", "woff2", "eot", "class", "jar", "war", "ear", "pyc",
        "pyo", "pyd", "egg", "whl", "so", "dll", "dylib", "a", "lib", "obj", "o"
    ])]
    skip_extensions: Vec<String>,

    /// Auto-detect project type and configure appropriate skips
    #[arg(long = "auto-detect")]
    auto_detect: bool,

    /// Include hidden files and folders
    #[arg(long = "include-hidden")]
    include_hidden: bool,

    /// Maximum directory depth (0 = unlimited)
    #[arg(long = "max-depth", default_value = "0")]
    max_depth: usize,
}

const SYSTEM_INSTRUCTIONS: &str = r##"## System Instructions for Language Model Assistance in Code Debugging
### Codebase Markdown File Structure:
- The codebase markdown file represents the actual codebase structure and content.
- It begins with a directory tree representation:
  ```
  ### DIRECTORY path/to/file/tree FOLDER STRUCTURE ###
  (file tree representation)
  ### DIRECTORY path/to/file/tree FOLDER STRUCTURE ###
  ```
- Following the directory tree, the contents of each file are displayed:
  ```
  ### path/to/file1 BEGIN ###
  (content of file1)
  ### path/to/file1 END ###
  
  ### path/to/file2 BEGIN ###
  (content of file2)
  ### path/to/file2 END ###
  ```
### Guidelines for Interaction:
- Respond to queries based on the explicit content provided within the markdown file.
- Avoid making assumptions about the code without clear evidence presented in the file content.
- When seeking specific implementation details, refer to the corresponding section in the markdown file, for example:
  ```
  ### folder1/folder2/myfile.ts BEGIN ###
  (specific implementation details)
  ### folder1/folder2/myfile.ts END ###
  ```
### Objective:
- The primary objective is to facilitate understanding of codebase by providing accurate information and guidance strictly adhering to the content available in the markdown file."##;

struct FlattenConfig {
    skip_folders: HashSet<String>,
    skip_extensions: HashSet<String>,
    show_skipped: bool,
    max_file_size: u64,
    include_hidden: bool,
    max_depth: usize,
}

impl FlattenConfig {
    fn new(args: &Args) -> Self {
        Self {
            skip_folders: args.skip_folders.iter().cloned().collect(),
            skip_extensions: args.skip_extensions.iter().cloned().collect(),
            show_skipped: args.show_skipped,
            max_file_size: args.max_file_size,
            include_hidden: args.include_hidden,
            max_depth: args.max_depth,
        }
    }

    fn should_skip_path(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                // Skip hidden files unless explicitly included
                if !self.include_hidden && name_str.starts_with('.') {
                    return true;
                }
                return self.skip_folders.contains(name_str);
            }
        }
        false
    }

    fn should_skip_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return self.skip_extensions.contains(ext_str);
            }
        }
        false
    }
}

fn print_folder_structure<W: Write>(
    directory: &Path,
    writer: &mut W,
    config: &FlattenConfig,
) -> Result<()> {
    writer.write_all(
        format!(
            "### DIRECTORY {} FOLDER STRUCTURE ###\n",
            directory.display()
        )
        .as_bytes(),
    )?;

    let mut walkdir = WalkDir::new(directory)
        .follow_links(false);

    if config.max_depth > 0 {
        walkdir = walkdir.max_depth(config.max_depth);
    } else {
        walkdir = walkdir.max_depth(usize::MAX);
    }

    for entry in walkdir.into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            !config.should_skip_path(e.path()) || config.show_skipped
        } else {
            true
        }
    }) {
        let entry = entry?;
        let path = entry.path();
        let depth = entry.depth();

        let indent = "    ".repeat(depth);

        if entry.file_type().is_dir() {
            if config.should_skip_path(path) {
                writer.write_all(
                    format!(
                        "{}{} {}/ (skipped)\n",
                        indent,
                        SKIP,
                        path.file_name().unwrap_or_default().to_string_lossy()
                    )
                    .as_bytes(),
                )?;
            } else {
                writer.write_all(
                    format!(
                        "{}{} {}/\n",
                        indent,
                        FOLDER,
                        path.file_name().unwrap_or_default().to_string_lossy()
                    )
                    .as_bytes(),
                )?;
            }
        } else if config.should_skip_file(path) {
            writer.write_all(
                format!(
                    "{}{} {} (binary)\n",
                    indent + "    ",
                    SKIP,
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
                .as_bytes(),
            )?;
        } else {
            writer.write_all(
                format!(
                    "{}{} {}\n",
                    indent + "    ",
                    FILE,
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
                .as_bytes(),
            )?;
        }
    }

    writer.write_all(
        format!(
            "### DIRECTORY {} FOLDER STRUCTURE ###\n\n",
            directory.display()
        )
        .as_bytes(),
    )?;
    Ok(())
}

fn read_file_content_fast(path: &Path, max_size: u64) -> Result<String> {
    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let metadata = file
        .metadata()
        .with_context(|| format!("Failed to read metadata: {}", path.display()))?;
    let file_size = metadata.len();

    if max_size > 0 && file_size > max_size {
        return Ok(format!(
            "[File too large: {} bytes, max size: {} bytes]",
            file_size, max_size
        ));
    }

    // Use memory mapping for better performance
    unsafe {
        let mmap = MmapOptions::new()
            .map(&file)
            .with_context(|| format!("Failed to memory map file: {}", path.display()))?;

        // Try to decode as UTF-8, fall back to lossy conversion
        String::from_utf8(mmap.to_vec()).or_else(|_| Ok(String::from_utf8_lossy(&mmap).to_string()))
    }
}

fn process_files_parallel(
    files: Vec<PathBuf>,
    config: &FlattenConfig,
    progress_bar: Option<ProgressBar>,
) -> Vec<(PathBuf, Result<String>)> {
    let processed_count = AtomicUsize::new(0);
    let _total_files = files.len();

    files
        .into_par_iter()
        .map(|file_path| {
            let result = if config.should_skip_file(&file_path) {
                Ok(format!(
                    "[Binary file skipped: {}]",
                    file_path.file_name().unwrap_or_default().to_string_lossy()
                ))
            } else {
                read_file_content_fast(&file_path, config.max_file_size)
            };

            let count = processed_count.fetch_add(1, Ordering::Relaxed);
            if let Some(pb) = &progress_bar {
                pb.set_position(count as u64 + 1);
            }

            (file_path, result)
        })
        .collect()
}

fn collect_files(directory: &Path, config: &FlattenConfig) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let mut walkdir = WalkDir::new(directory)
        .follow_links(false);

    if config.max_depth > 0 {
        walkdir = walkdir.max_depth(config.max_depth);
    } else {
        walkdir = walkdir.max_depth(usize::MAX);
    }

    for entry in walkdir.into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            !config.should_skip_path(e.path())
        } else {
            true
        }
    }) {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }

    Ok(files)
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.system_instructions {
        println!("{}", SYSTEM_INSTRUCTIONS);
        if args.folders.is_empty() {
            return Ok(());
        }
    }

    if args.folders.is_empty() {
        eprintln!("Error: --folders argument is required");
        std::process::exit(1);
    }

    // Configure thread pool
    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()
            .with_context(|| "Failed to configure thread pool")?;
    }

    let config = FlattenConfig::new(&args);

    println!("{} Starting flatten process...", ROCKET);
    println!("Processing {} folders", args.folders.len());
    println!("Skip folders: {:?}", config.skip_folders);
    println!("Output file: {}", args.output.display());
    println!();

    // Create output file with buffered writing
    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&args.output)
        .with_context(|| format!("Failed to create output file: {}", args.output.display()))?;

    let total_files = AtomicUsize::new(0);

    for base_folder in &args.folders {
        if !base_folder.exists() {
            eprintln!(
                "Warning: Folder {} does not exist, skipping",
                base_folder.display()
            );
            continue;
        }

        println!("Processing folder: {}", base_folder.display());

        // Print folder structure
        print_folder_structure(base_folder, &mut output_file, &config)?;

        // Collect all files
        let files = collect_files(base_folder, &config)?;
        let file_count = files.len();
        total_files.fetch_add(file_count, Ordering::Relaxed);

        if file_count == 0 {
            println!("No files found in {}", base_folder.display());
            continue;
        }

        // Setup progress bar
        let pb = ProgressBar::new(file_count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        // Process files in parallel
        output_file.write_all(
            format!(
                "### DIRECTORY {} FLATTENED CONTENT ###\n",
                base_folder.display()
            )
            .as_bytes(),
        )?;

        let results = process_files_parallel(files, &config, Some(pb.clone()));

        // Write results
        for (file_path, content_result) in results {
            output_file.write_all(format!("### {} BEGIN ###\n", file_path.display()).as_bytes())?;

            match content_result {
                Ok(content) => {
                    output_file.write_all(content.as_bytes())?;
                }
                Err(e) => {
                    output_file.write_all(format!("[Error reading file: {}]\n", e).as_bytes())?;
                }
            }

            output_file
                .write_all(format!("\n### {} END ###\n\n", file_path.display()).as_bytes())?;
        }

        output_file.write_all(
            format!(
                "### DIRECTORY {} FLATTENED CONTENT ###\n",
                base_folder.display()
            )
            .as_bytes(),
        )?;

        pb.finish_with_message("Done");
        println!(
            "Processed {} files from {}",
            file_count,
            base_folder.display()
        );
    }

    let total = total_files.load(Ordering::Relaxed);
    println!();
    println!("{} Flatten completed successfully!", style("✓").green());
    println!("Total files processed: {}", total);
    println!("Output written to: {}", args.output.display());

    Ok(())
}

### /home/whoami/projects/flatten-rust/src/main.rs END ###

### /home/whoami/projects/flatten-rust/src/lib.rs BEGIN ###
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FlattenConfig {
    pub skip_folders: HashSet<String>,
    pub skip_extensions: HashSet<String>,
    pub show_skipped: bool,
    pub max_file_size: u64,
}

impl Default for FlattenConfig {
    fn default() -> Self {
        Self {
            skip_folders: HashSet::new(),
            skip_extensions: HashSet::new(),
            show_skipped: false,
            max_file_size: 104857600, // 100MB
        }
    }
}

impl FlattenConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn should_skip_path(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                return self.skip_folders.contains(name_str);
            }
        }
        false
    }

    pub fn should_skip_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return self.skip_extensions.contains(ext_str);
            }
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

        assert!(config.should_skip_path(&PathBuf::from("/project/node_modules")));
        assert!(config.should_skip_path(&PathBuf::from("/project/.git")));
        assert!(!config.should_skip_path(&PathBuf::from("/project/src")));
    }

    #[test]
    fn test_config_skip_extensions() {
        let mut config = FlattenConfig::new();
        config.skip_extensions.insert("exe".to_string());
        config.skip_extensions.insert("bin".to_string());

        assert!(config.should_skip_file(&PathBuf::from("/project/test.exe")));
        assert!(config.should_skip_file(&PathBuf::from("/project/test.bin")));
        assert!(!config.should_skip_file(&PathBuf::from("/project/test.rs")));
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

### /home/whoami/projects/flatten-rust/src/lib.rs END ###

### DIRECTORY /home/whoami/projects/flatten-rust/src FLATTENED CONTENT ###
