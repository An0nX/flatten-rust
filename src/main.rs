mod config;
mod exclusions;

use anyhow::{Context, Result};
use clap::Parser;
use console::{style, Emoji};
use exclusions::ExclusionManager;
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
#[command(about = "High-performance codebase flattening tool with intelligent exclusions")]
#[command(version)]
#[command(after_help = r##"
EXCLUSION MANAGEMENT:
  The tool uses gitignore templates from toptal.com API for intelligent exclusions.
  Templates are cached in ~/.flatten/ and updated automatically every 24 hours.

  Available commands for exclusion management:
    -l, --list-templates           List all available gitignore templates
    -e, --enable-template <TEMPLATE>  Enable specific template
    -D, --disable-template <TEMPLATE>  Disable specific template
    -u, --force-update            Force update templates from API
    -c, --add-custom <PATTERN>    Add custom exclusion pattern
    -r, --remove-custom <PATTERN> Remove custom exclusion pattern
    -n, --check-internet <BOOL>   Enable/disable internet connectivity check
    --show-enabled                 Show currently enabled templates

  Common templates: rust, node, python, java, csharp, go, php, ruby, android, ios, 
                    visualstudiocode, visualstudio, eclipse, intellij, macos, linux, windows

  SHORTCUTS (Easy to remember):
    -f, --folders <FOLDERS>       Folders to process (f for folders)
    -s, --skip-folders <FOLDERS>  Folders to skip (s for skip)
    -o, --output <FILE>           Output file (o for output)
    -a, --auto-detect             Auto-detect project type (a for auto)
    -t, --threads <NUM>           Parallel threads (t for threads)
    -m, --max-file-size <SIZE>    Max file size (m for max)
    -d, --dry-run                 Dry run (d for dry)
    -S, --stats                   Show statistics (S for stats)
    -k, --show-skipped            Show skipped folders (k for keep)
    -l, --list-templates          List templates (l for list)
    -e, --enable-template <TMPL>  Enable template (e for enable)
    -D, --disable-template <TMPL> Disable template (D for disable)
    -u, --force-update            Force update (u for update)
    -n, --check-internet <BOOL>   Check internet (n for network)

EXAMPLES:
  # Basic usage with auto-detection
  flatten-rust -f ./project -a

  # Manual template selection
  flatten-rust -f ./project -e rust -e node

  # Performance options
  flatten-rust -f ./project -t 8 -m 50MB

  # Template management
  flatten-rust -l
  flatten-rust -u
  flatten-rust -n false

  # Advanced usage
  flatten-rust -f ./project -s "temp" -s "cache" -x "log" -x "tmp" -d
"##)]
struct Args {
    /// Base folders to process
    #[arg(long = "folders", short = 'f', num_args = 1..)]
    folders: Vec<PathBuf>,

    /// Folders to skip during processing (supports glob patterns)
    #[arg(long = "skip-folders", short = 's', num_args = 0.., default_values = [
        ".git", "node_modules", "target", "dist", "build"
    ])]
    skip_folders: Vec<String>,

    

    /// Output file name
    #[arg(long = "output", short = 'o', default_value = "codebase.md")]
    output: PathBuf,

    /// Show skipped folders in tree structure
    #[arg(long = "show-skipped", short = 'k')]
    show_skipped: bool,

    /// Number of parallel file processing threads
    #[arg(long = "threads", short = 't', default_value = "0")]
    threads: usize,

    /// Maximum file size to process in bytes (0 = unlimited)
    #[arg(long = "max-file-size", short = 'm', default_value = "104857600")]
    max_file_size: u64,

    /// File extension patterns to skip
    #[arg(long = "skip-extensions", short = 'x', num_args = 0.., default_values = [
        "exe", "dll", "so", "dylib", "bin", "jar", "apk", "ipa", "msi", "class", "pyc"
    ])]
    skip_extensions: Vec<String>,

    /// Auto-detect project type and configure appropriate skips
    #[arg(long = "auto-detect", short = 'a')]
    auto_detect: bool,

    /// Include hidden files and folders
    #[arg(long = "include-hidden")]
    include_hidden: bool,

    /// Maximum directory depth (0 = unlimited)
    #[arg(long = "max-depth", default_value = "0")]
    max_depth: usize,

    /// Show detailed statistics after processing
    #[arg(long = "stats", short = 'S')]
    show_stats: bool,

    /// Dry run - show what would be processed without creating output
    #[arg(long = "dry-run", short = 'd')]
    dry_run: bool,

    /// List all available gitignore templates
    #[arg(long = "list-templates", short = 'l')]
    list_templates: bool,

    /// Enable specific gitignore template
    #[arg(long = "enable-template", short = 'e', num_args = 1..)]
    enable_templates: Vec<String>,

    /// Disable specific gitignore template
    #[arg(long = "disable-template", short = 'D', num_args = 1..)]
    disable_templates: Vec<String>,

    /// Force update templates from API
    #[arg(long = "force-update", short = 'u')]
    force_update: bool,

    /// Enable/disable internet connectivity check
    #[arg(long = "check-internet", short = 'n')]
    check_internet: Option<bool>,

    /// Show enabled templates
    #[arg(long = "show-enabled")]
    show_enabled: bool,
}



struct FlattenConfig {
    exclusion_manager: Option<ExclusionManager>,
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
    async fn new(args: &Args) -> Result<Self> {
        let mut config = Self {
            exclusion_manager: None,
            skip_folders: args.skip_folders.iter().cloned().collect(),
            skip_extensions: args.skip_extensions.iter().cloned().collect(),
            show_skipped: args.show_skipped,
            max_file_size: args.max_file_size,
            include_hidden: args.include_hidden,
            max_depth: args.max_depth,
            show_stats: args.show_stats,
            dry_run: args.dry_run,
        };

        // Initialize exclusion manager if needed
        if args.auto_detect || !args.enable_templates.is_empty() || args.force_update || 
           args.list_templates || args.show_enabled {
            
            let mut exclusion_manager = ExclusionManager::new().await?;
            
            // Handle force update
            if args.force_update {
                exclusion_manager.force_update_templates().await?;
            }
            
            // Handle internet connectivity setting
            if let Some(check_internet) = args.check_internet {
                exclusion_manager.set_check_internet(check_internet).await?;
            }
            
            // Handle template management commands
            if args.list_templates {
                Self::handle_list_templates(&exclusion_manager).await?;
                std::process::exit(0);
            }
            
            if args.show_enabled {
                Self::handle_show_enabled(&exclusion_manager);
                std::process::exit(0);
            }
            
            // Enable specific templates
            for template in &args.enable_templates {
                exclusion_manager.enable_template(template.clone());
            }
            
            // Disable specific templates
            for template in &args.disable_templates {
                exclusion_manager.disable_template(template);
            }
            
            // Auto-detect project types
            if args.auto_detect && !args.folders.is_empty() {
                for folder in &args.folders {
                    if folder.exists() {
                        exclusion_manager.enable_templates_for_project(folder).await?;
                    }
                }
            }
            
            // Update config with exclusion patterns
            if !args.enable_templates.is_empty() || args.auto_detect {
                let folder_patterns = exclusion_manager.get_folder_patterns().await;
                let extension_patterns = exclusion_manager.get_extension_patterns().await;
                
                config.skip_folders.extend(folder_patterns);
                config.skip_extensions.extend(extension_patterns);
            }
            
            config.exclusion_manager = Some(exclusion_manager);
        }

        Ok(config)
    }
    
    /// Handle list templates command
    async fn handle_list_templates(exclusion_manager: &ExclusionManager) -> Result<()> {
        let templates = exclusion_manager.get_available_templates().await;
        println!("Available gitignore templates ({} total):", templates.len());
        println!();
        
        // Group templates by category
        let mut categories: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
        
        for template in &templates {
            let category = if template.contains("studio") || template.contains("code") || 
                             template.contains("intellij") || template.contains("pycharm") ||
                             template.contains("phpstorm") || template.contains("webstorm") ||
                             template.contains("rubymine") || template.contains("clion") ||
                             template.contains("goland") || template.contains("androidstudio") {
                "IDEs"
            } else if template.contains("macos") || template.contains("linux") || template.contains("windows") ||
                      template.contains("android") || template.contains("ios") {
                "Platforms"
            } else if template.contains("visualstudio") || template.contains("eclipse") ||
                      template.contains("sublime") || template.contains("vim") ||
                      template.contains("emacs") || template.contains("atom") {
                "Editors"
            } else {
                "Languages"
            };
            
            categories.entry(category).or_default().push(template);
        }
        
        // Print templates by category
        for (category, mut templates) in categories {
            templates.sort();
            println!("{}:", category);
            for template in templates.chunks(5) {
                println!("  {}", template.join(", "));
            }
            println!();
        }
        
        Ok(())
    }
    
    /// Handle show enabled templates command
    fn handle_show_enabled(exclusion_manager: &ExclusionManager) {
        let enabled = exclusion_manager.get_enabled_templates();
        if enabled.is_empty() {
            println!("No templates currently enabled.");
        } else {
            println!("Enabled templates ({}):", enabled.len());
            for template in enabled {
                println!("  - {}", template);
            }
        }
    }

    fn should_skip_path(&self, path: &Path) -> bool {
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

    fn should_skip_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension()
            && let Some(ext_str) = extension.to_str() {
            return self.skip_extensions.contains(ext_str);
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

fn read_file_content_fast(path: &Path, max_size: u64) -> Result<(String, u64)> {
    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let metadata = file
        .metadata()
        .with_context(|| format!("Failed to read metadata: {}", path.display()))?;
    let file_size = metadata.len();

    if max_size > 0 && file_size > max_size {
        return Ok((format!(
            "[File too large: {} bytes, max size: {} bytes]",
            file_size, max_size
        ), file_size));
    }

    // Use memory mapping for better performance
    unsafe {
        let mmap = MmapOptions::new()
            .map(&file)
            .with_context(|| format!("Failed to memory map file: {}", path.display()))?;

        // Try to decode as UTF-8, fall back to lossy conversion
        let content = String::from_utf8(mmap.to_vec())
            .unwrap_or_else(|_| String::from_utf8_lossy(&mmap).to_string());
        
        Ok((content, file_size))
    }
}

fn process_files_parallel(
    files: Vec<PathBuf>,
    config: &FlattenConfig,
    progress_bar: Option<ProgressBar>,
) -> Vec<(PathBuf, Result<(String, u64)>)> {
    let processed_count = AtomicUsize::new(0);
    let _total_files = files.len();

    let results: Vec<(PathBuf, Result<(String, u64)>)> = files
        .into_par_iter()
        .map(|file_path| {
            let result = if config.should_skip_file(&file_path) {
                // Для пропущенных файлов возвращаем размер 0
                Ok((format!(
                    "[Binary file skipped: {}]",
                    file_path.file_name().unwrap_or_default().to_string_lossy()
                ), 0))
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    

    // Handle template management commands that don't require folders
    if (args.list_templates || args.show_enabled || args.force_update ||
       !args.enable_templates.is_empty() || !args.disable_templates.is_empty())
        && args.folders.is_empty() {
        // Just create config to handle the commands
        let _ = FlattenConfig::new(&args).await?;
        return Ok(());
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

    let config = FlattenConfig::new(&args).await?;

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
    let total_bytes_processed = AtomicUsize::new(0);

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

        // Write results and count bytes
        for (file_path, content_result) in results {
            if let Some(ref mut output) = output_file {
                output.write_all(format!("### {} BEGIN ###\n", file_path.display()).as_bytes())?;

                match content_result {
                    Ok((content, bytes_processed)) => {
                        output.write_all(content.as_bytes())?;
                        total_bytes_processed.fetch_add(bytes_processed as usize, Ordering::Relaxed);
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
                    Ok((_, bytes_processed)) => {
                        println!("  ✅ {} ({} bytes)", file_path.display(), bytes_processed);
                        total_bytes_processed.fetch_add(bytes_processed as usize, Ordering::Relaxed);
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
    let total_bytes = total_bytes_processed.load(Ordering::Relaxed) as u64;
    println!();
    println!("{} Flatten completed successfully!", style("✓").green());
    println!("Total files processed: {}", total);

    if config.show_stats {
        // Format total bytes appropriately
        if total_bytes >= 1_048_576 {
            println!(
                "Total bytes processed: {:.2} MB",
                total_bytes as f64 / 1_048_576.0
            );
        } else if total_bytes >= 1024 {
            println!(
                "Total bytes processed: {:.2} KB",
                total_bytes as f64 / 1024.0
            );
        } else {
            println!("Total bytes processed: {} bytes", total_bytes);
        }

        // Format average file size appropriately
        if total > 0 {
            let avg_size = total_bytes / total as u64;
            if avg_size >= 1024 {
                println!(
                    "Average file size: {:.2} KB",
                    avg_size as f64 / 1024.0
                );
            } else {
                println!("Average file size: {} bytes", avg_size);
            }
        } else {
            println!("Average file size: 0 bytes");
        }
    }

    if !config.dry_run {
        println!("Output written to: {}", args.output.display());
    } else {
        println!("Dry run completed - no files were written");
    }

    Ok(())
}
