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

    /// Show detailed statistics after processing
    #[arg(long = "stats")]
    show_stats: bool,

    /// Dry run - show what would be processed without creating output
    #[arg(long = "dry-run")]
    dry_run: bool,
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

#[derive(Default)]
struct ProjectDetection {
    skip_folders: HashSet<String>,
    skip_extensions: HashSet<String>,
}

struct FlattenConfig {
    skip_folders: HashSet<String>,
    skip_extensions: HashSet<String>,
    show_skipped: bool,
    max_file_size: u64,
    include_hidden: bool,
    max_depth: usize,
    show_stats: bool,
    dry_run: bool,
}

impl FlattenConfig {
    fn new(args: &Args) -> Self {
        let mut config = Self {
            skip_folders: args.skip_folders.iter().cloned().collect(),
            skip_extensions: args.skip_extensions.iter().cloned().collect(),
            show_skipped: args.show_skipped,
            max_file_size: args.max_file_size,
            include_hidden: args.include_hidden,
            max_depth: args.max_depth,
            show_stats: args.show_stats,
            dry_run: args.dry_run,
        };

        if args.auto_detect {
            config = Self::auto_detect_config(config, &args.folders);
        }

        config
    }

    fn auto_detect_config(mut config: Self, folders: &[PathBuf]) -> Self {
        for folder in folders {
            if let Ok(detected) = Self::detect_project_type(folder) {
                config.skip_folders.extend(detected.skip_folders);
                config.skip_extensions.extend(detected.skip_extensions);
            }
        }
        config
    }

    fn detect_project_type(folder: &Path) -> Result<ProjectDetection> {
        let mut detection = ProjectDetection::default();

        // Check for Rust projects
        if folder.join("Cargo.toml").exists() {
            detection
                .skip_folders
                .extend(vec!["target".to_string(), "Cargo.lock".to_string()]);
            detection
                .skip_extensions
                .extend(vec!["rlib".to_string(), "rmeta".to_string()]);
        }

        // Check for Node.js projects
        if folder.join("package.json").exists() || folder.join("package-lock.json").exists() {
            detection.skip_folders.extend(vec![
                "node_modules".to_string(),
                ".npm".to_string(),
                ".pnpm-store".to_string(),
                ".yarn".to_string(),
                ".yarn-integrity".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".next".to_string(),
                ".nuxt".to_string(),
                ".output".to_string(),
                ".angular".to_string(),
                "coverage".to_string(),
                ".nyc_output".to_string(),
            ]);
            detection.skip_extensions.extend(vec![
                "js.map".to_string(),
                "css.map".to_string(),
                "tsbuildinfo".to_string(),
            ]);
        }

        // Check for Angular projects (specific detection in addition to Node.js)
        if folder.join("angular.json").exists() || folder.join(".angular-cli.json").exists() {
            detection.skip_folders.extend(vec![
                ".angular".to_string(),
                "dist".to_string(),
                ".nuxt".to_string(),
                ".output".to_string(),
                "coverage".to_string(),
                ".coverage".to_string(),
            ]);
            detection.skip_extensions.extend(vec![
                "js.map".to_string(),
                "css.map".to_string(),
                "ngsummary.json".to_string(),
                "ngfactory".to_string(),
                "ngstyle".to_string(),
                "ngtemplate".to_string(),
            ]);
        }

        // Check for Python projects
        if folder.join("requirements.txt").exists()
            || folder.join("setup.py").exists()
            || folder.join("pyproject.toml").exists()
            || folder.join("Pipfile").exists()
        {
            detection.skip_folders.extend(vec![
                "__pycache__".to_string(),
                ".pytest_cache".to_string(),
                ".mypy_cache".to_string(),
                ".tox".to_string(),
                "venv".to_string(),
                ".venv".to_string(),
                "env".to_string(),
                ".env".to_string(),
                "site-packages".to_string(),
                "eggs".to_string(),
                ".eggs".to_string(),
                "pip-wheel-metadata".to_string(),
                ".coverage".to_string(),
                "htmlcov".to_string(),
            ]);
            detection.skip_extensions.extend(vec![
                "pyc".to_string(),
                "pyo".to_string(),
                "pyd".to_string(),
                "egg".to_string(),
                "whl".to_string(),
            ]);
        }

        // Check for Java projects
        if folder.join("pom.xml").exists() || folder.join("build.gradle").exists() {
            detection.skip_folders.extend(vec![
                "target".to_string(),
                "build".to_string(),
                ".gradle".to_string(),
                ".idea".to_string(),
                ".vs".to_string(),
                "out".to_string(),
            ]);
            detection.skip_extensions.extend(vec![
                "class".to_string(),
                "jar".to_string(),
                "war".to_string(),
                "ear".to_string(),
            ]);
        }

        // Check for C# projects
        if folder.join("*.csproj").exists()
            || folder.join("*.sln").exists()
            || folder.join("project.json").exists()
        {
            detection.skip_folders.extend(vec![
                "bin".to_string(),
                "obj".to_string(),
                "packages".to_string(),
                ".vs".to_string(),
                ".vscode".to_string(),
                "Properties".to_string(),
            ]);
            detection.skip_extensions.extend(vec![
                "exe".to_string(),
                "dll".to_string(),
                "pdb".to_string(),
                "cache".to_string(),
                "user".to_string(),
            ]);
        }

        // Check for Angular projects (already covered by Node.js, but add specific Angular exclusions)
        if folder.join("angular.json").exists() || folder.join(".angular-cli.json").exists() {
            detection.skip_folders.extend(vec![
                ".angular".to_string(),
                "dist".to_string(),
                ".nuxt".to_string(),
                ".output".to_string(),
                "coverage".to_string(),
                ".coverage".to_string(),
            ]);
            detection.skip_extensions.extend(vec![
                "js.map".to_string(),
                "css.map".to_string(),
                "ngsummary.json".to_string(),
                "ngfactory".to_string(),
                "ngstyle".to_string(),
                "ngtemplate".to_string(),
            ]);
        }

        // Check for C# projects
        if folder.join("*.csproj").exists()
            || folder.join("*.sln").exists()
            || folder.join("project.json").exists()
        {
            detection.skip_folders.extend(vec![
                "bin".to_string(),
                "obj".to_string(),
                "packages".to_string(),
                ".vs".to_string(),
                ".vscode".to_string(),
                "Properties".to_string(),
            ]);
            detection.skip_extensions.extend(vec![
                "exe".to_string(),
                "dll".to_string(),
                "pdb".to_string(),
                "cache".to_string(),
                "user".to_string(),
            ]);
        }

        // Check for Go projects
        if folder.join("go.mod").exists() || folder.join("go.sum").exists() {
            detection.skip_folders.extend(vec!["vendor".to_string()]);
        }
        if folder.join("CMakeLists.txt").exists() || folder.join("Makefile").exists() {
            detection.skip_folders.extend(vec![
                "cmake-build-debug".to_string(),
                "cmake-build-release".to_string(),
                "build".to_string(),
                "obj".to_string(),
                "bin".to_string(),
                "Debug".to_string(),
                "Release".to_string(),
                "x64".to_string(),
                "x86".to_string(),
            ]);
            detection.skip_extensions.extend(vec![
                "o".to_string(),
                "obj".to_string(),
                "exe".to_string(),
                "dll".to_string(),
                "so".to_string(),
                "dylib".to_string(),
                "a".to_string(),
                "lib".to_string(),
            ]);
        }

        // Check for Ruby projects
        if folder.join("Gemfile").exists() || folder.join("Gemfile.lock").exists() {
            detection
                .skip_folders
                .extend(vec!["vendor".to_string(), ".bundle".to_string()]);
        }

        // Check for PHP projects
        if folder.join("composer.json").exists() || folder.join("composer.lock").exists() {
            detection.skip_folders.extend(vec!["vendor".to_string()]);
        }

        Ok(detection)
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

    let mut walkdir = WalkDir::new(directory).follow_links(false);

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

    let results: Vec<(PathBuf, Result<String>)> = files
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
        .collect();

    results
}

fn collect_files(directory: &Path, config: &FlattenConfig) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let mut walkdir = WalkDir::new(directory).follow_links(false);

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
    if !config.dry_run {
        println!("Output file: {}", args.output.display());
    }
    println!();

    if config.dry_run {
        println!("🔍 DRY RUN MODE - No output file will be created");
        println!();
    }

    // Create output file with buffered writing (unless dry run)
    let mut output_file = if !config.dry_run {
        Some(
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&args.output)
                .with_context(|| {
                    format!("Failed to create output file: {}", args.output.display())
                })?,
        )
    } else {
        None
    };

    let total_files = AtomicUsize::new(0);
    let total_bytes_processed = 0u64;

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
        if let Some(ref mut output) = output_file {
            print_folder_structure(base_folder, output, &config)?;
        } else {
            // Dry run - just print structure to console
            println!("📁 Folder structure for {}", base_folder.display());
            let mut console_output = Vec::new();
            print_folder_structure(base_folder, &mut console_output, &config)?;
            println!("{}", String::from_utf8_lossy(&console_output));
        }

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
        if let Some(ref mut output) = output_file {
            output.write_all(
                format!(
                    "### DIRECTORY {} FLATTENED CONTENT ###\n",
                    base_folder.display()
                )
                .as_bytes(),
            )?;
        } else {
            println!("📄 Files to process from {}:", base_folder.display());
        }

        let results = process_files_parallel(files, &config, Some(pb.clone()));

        // Write results
        for (file_path, content_result) in results {
            if let Some(ref mut output) = output_file {
                output.write_all(format!("### {} BEGIN ###\n", file_path.display()).as_bytes())?;

                match content_result {
                    Ok(content) => {
                        output.write_all(content.as_bytes())?;
                    }
                    Err(e) => {
                        output.write_all(format!("[Error reading file: {}]\n", e).as_bytes())?;
                    }
                }

                output
                    .write_all(format!("\n### {} END ###\n\n", file_path.display()).as_bytes())?;
            } else {
                // Dry run - just show file paths
                match content_result {
                    Ok(_) => {
                        println!("  ✅ {}", file_path.display());
                    }
                    Err(e) => {
                        println!("  ❌ {} ({})", file_path.display(), e);
                    }
                }
            }
        }

        if let Some(ref mut output) = output_file {
            output.write_all(
                format!(
                    "### DIRECTORY {} FLATTENED CONTENT ###\n",
                    base_folder.display()
                )
                .as_bytes(),
            )?;
        }

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

    if config.show_stats {
        println!(
            "Total bytes processed: {} MB",
            total_bytes_processed / 1_048_576
        );
        println!(
            "Average file size: {} KB",
            if total > 0 {
                total_bytes_processed / total as u64 / 1024
            } else {
                0
            }
        );
    }

    if !config.dry_run {
        println!("Output written to: {}", args.output.display());
    } else {
        println!("Dry run completed - no files were written");
    }

    Ok(())
}
