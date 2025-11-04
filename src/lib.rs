//! # Flatten Rust (Library)
//!
//! Этот крейт предоставляет основную функциональность для утилиты `flatten-rust`,
//! инструмента для "сглаживания" кодовых баз в единый markdown-файл.
//! Он включает в себя логику для обхода директорий, фильтрации файлов
//! на основе шаблонов исключений, параллельной обработки и форматирования вывода.
//!
//! ## Основные компоненты:
//!
//! - `Args`: Структура для парсинга аргументов командной строки с использованием `clap`.
//! - `run`: Асинхронная функция, являющаяся основной точкой входа в библиотеку.
//! - `FlattenConfig`: Структура для управления конфигурацией процесса "сглаживания".
//! - `config`: Модуль для управления шаблонами исключений (например, из `.gitignore`).
//! - `exclusions`: Модуль для управления логикой исключения файлов и папок.
//!
//! # Примеры
//!
//! Хотя этот крейт в основном предназначен для использования через CLI,
//! его компоненты могут быть использованы и программно.
//!
//! ```no_run
//! use flatten_rust::Args;
//! use anyhow::Result;
//! use clap::Parser;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Пример парсинга аргументов и запуска
//!     let args = Args::parse_from(["flatten-rust", "-f", ".", "-d"]);
//!     flatten_rust::run(&args).await?;
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod exclusions;

use anyhow::{Context, Result};
use clap::Parser;
use console::{style, Emoji};
use exclusions::ExclusionManager;
use indicatif::{ProgressBar, ProgressStyle};
use memmap2::MmapOptions;
use rayon::prelude::*;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

static FOLDER: Emoji<'_, '_> = Emoji("📁", "DIR");
static FILE: Emoji<'_, '_> = Emoji("📄", "FILE");
static SKIP: Emoji<'_, '_> = Emoji("⏭️", "SKIP");
static ROCKET: Emoji<'_, '_> = Emoji("🚀", "=>");
const PROGRESS_STYLE: &str =
    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})";

/// # Высокопроизводительный инструмент для "сглаживания" кодовой базы с умными исключениями
///
/// Утилита для рекурсивного обхода директорий, конкатенации текстовых файлов
/// в один Markdown-файл с сохранением структуры проекта.
#[derive(Parser, Debug)]
#[command(name = "flatten-rust")]
#[command(about = "High-performance codebase flattening tool with intelligent exclusions")]
#[command(version)]
#[command(after_help = r##"
УПРАВЛЕНИЕ ИСКЛЮЧЕНИЯМИ:
  Инструмент использует шаблоны в формате gitignore из API toptal.com для умных исключений.
  Шаблоны кэшируются в ~/.flatten/ и автоматически обновляются каждые 24 часа.

  Доступные команды для управления исключениями:
    -l, --list-templates           Показать список всех доступных шаблонов
    -e, --enable-template <TEMPLATE>  Включить определенный шаблон
    -D, --disable-template <TEMPLATE> Отключить определенный шаблон
    -u, --force-update             Принудительно обновить шаблоны из API
    
    --show-enabled                 Показать текущие включенные шаблоны

ПРИМЕРЫ:
  # Базовое использование с авто-определением
  flatten-rust -f ./project -a

  # Ручной выбор шаблонов
  flatten-rust -f ./project -e rust -e node

  # Опции производительности
  flatten-rust -f ./project -t 8 -m 50MB

  # Управление шаблонами
  flatten-rust -l
  flatten-rust -u
"##)]
pub struct Args {
    /// Базовые папки для обработки
    #[arg(long = "folders", short = 'f', num_args = 1..)]
    pub folders: Vec<PathBuf>,

    /// Папки для пропуска при обработке (поддерживаются glob-паттерны)
    #[arg(long = "skip-folders", short = 's', num_args = 0.., default_values = [
        ".git", "node_modules", "target", "dist", "build"
    ])]
    pub skip_folders: Vec<String>,

    /// Выходной файл
    #[arg(long = "output", short = 'o', default_value = "codebase.md")]
    pub output: PathBuf,

    /// Показывать пропущенные папки в дереве структуры
    #[arg(long = "show-skipped", short = 'k')]
    pub show_skipped: bool,

    /// Количество потоков для параллельной обработки файлов
    #[arg(long = "threads", short = 't', default_value = "0")]
    pub threads: usize,

    /// Максимальный размер файла для обработки в байтах (0 = без ограничений)
    #[arg(long = "max-file-size", short = 'm', default_value = "104857600")]
    pub max_file_size: u64,

    /// Паттерны расширений файлов для пропуска
    #[arg(long = "skip-extensions", short = 'x', num_args = 0.., default_values = [
        "exe", "dll", "so", "dylib", "bin", "jar", "apk", "ipa", "msi", "class", "pyc"
    ])]
    pub skip_extensions: Vec<String>,

    /// Автоматически определять тип проекта и настраивать соответствующие пропуски
    #[arg(long = "auto-detect", short = 'a')]
    pub auto_detect: bool,

    /// Включать скрытые файлы и папки
    #[arg(long = "include-hidden")]
    pub include_hidden: bool,

    /// Максимальная глубина обхода директорий (0 = без ограничений)
    #[arg(long = "max-depth", default_value = "0")]
    pub max_depth: usize,

    /// Показать детальную статистику после обработки
    #[arg(long = "stats", short = 'S')]
    pub show_stats: bool,

    /// Тестовый запуск - показать, что будет обработано, без создания выходного файла
    #[arg(long = "dry-run", short = 'd')]
    pub dry_run: bool,

    /// Показать список всех доступных шаблонов исключений
    #[arg(long = "list-templates", short = 'l')]
    pub list_templates: bool,

    /// Включить определенный шаблон исключений
    #[arg(long = "enable-template", short = 'e', num_args = 1..)]
    pub enable_templates: Vec<String>,

    /// Отключить определенный шаблон исключений
    #[arg(long = "disable-template", short = 'D', num_args = 1..)]
    pub disable_templates: Vec<String>,

    /// Принудительно обновить шаблоны из API
    #[arg(long = "force-update", short = 'u')]
    pub force_update: bool,

    /// Показать включенные шаблоны
    #[arg(long = "show-enabled")]
    pub show_enabled: bool,
}

/// Конфигурация процесса "сглаживания".
///
/// Содержит все параметры, необходимые для управления процессом,
/// включая правила исключений, лимиты и флаги форматирования.
#[derive(Debug)]
pub struct FlattenConfig {
    /// Менеджер для работы с шаблонами исключений.
    exclusion_manager: ExclusionManager,
    /// Набор папок для пропуска.
    skip_folders: HashSet<String>,
    /// Набор расширений файлов для пропуска.
    skip_extensions: HashSet<String>,
    /// Показывать ли пропущенные элементы в выводе.
    show_skipped: bool,
    /// Максимальный размер файла для обработки.
    max_file_size: u64,
    /// Включать ли скрытые файлы и папки.
    include_hidden: bool,
    /// Максимальная глубина рекурсии.
    max_depth: usize,
    /// Показывать ли статистику в конце.
    show_stats: bool,
    /// Выполнять ли тестовый запуск.
    dry_run: bool,
}

impl FlattenConfig {
    /// Создает новый экземпляр `FlattenConfig` на основе аргументов командной строки.
    ///
    /// Асинхронно инициализирует `ExclusionManager`, загружает и обновляет шаблоны
    /// исключений, а также обрабатывает команды управления шаблонами.
    pub async fn new(args: &Args) -> Result<Self> {
        let mut exclusion_manager = ExclusionManager::new().await?;

        if args.force_update {
            exclusion_manager.force_update_templates().await?;
        }

        if args.list_templates {
            Self::handle_list_templates(&exclusion_manager).await?;
            std::process::exit(0);
        }

        if args.show_enabled {
            Self::handle_show_enabled(&exclusion_manager);
            std::process::exit(0);
        }

        for template in &args.enable_templates {
            exclusion_manager.enable_template(template.clone());
        }

        for template in &args.disable_templates {
            exclusion_manager.disable_template(template);
        }

        if args.auto_detect && !args.folders.is_empty() {
            for folder in &args.folders {
                if folder.exists() {
                    exclusion_manager
                        .enable_templates_for_project(folder)
                        .await?;
                }
            }
        }

        let mut config = Self {
            skip_folders: args.skip_folders.iter().cloned().collect(),
            skip_extensions: args.skip_extensions.iter().cloned().collect(),
            show_skipped: args.show_skipped,
            max_file_size: args.max_file_size,
            include_hidden: args.include_hidden,
            max_depth: args.max_depth,
            show_stats: args.show_stats,
            dry_run: args.dry_run,
            exclusion_manager,
        };

        let folder_patterns = config.exclusion_manager.get_folder_patterns().await;
        let extension_patterns = config.exclusion_manager.get_extension_patterns().await;

        config.skip_folders.extend(folder_patterns);
        config.skip_extensions.extend(extension_patterns);

        Ok(config)
    }

    /// Обрабатывает команду вывода списка доступных шаблонов.
    async fn handle_list_templates(exclusion_manager: &ExclusionManager) -> Result<()> {
        let templates = exclusion_manager.get_available_templates().await;
        println!("Available exclusion templates ({} total):", templates.len());
        println!();
        let mut sorted_templates = templates;
        sorted_templates.sort();
        for chunk in sorted_templates.chunks(5) {
            println!("  {}", chunk.join(", "));
        }
        Ok(())
    }

    /// Обрабатывает команду вывода списка включенных шаблонов.
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

    /// Проверяет, следует ли пропустить данный путь (директорию).
    fn should_skip_path(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name()
           && let Some(name_str) = name.to_str() {
            if !self.include_hidden && name_str.starts_with('.') {
                return true;
            }
            return self.skip_folders.contains(name_str);
        }
        false
    }

    /// Проверяет, следует ли пропустить данный файл (по расширению).
    fn should_skip_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension()
            && let Some(ext_str) = extension.to_str() {
            return self.skip_extensions.contains(ext_str);
        }
        false
    }
}

/// Основная функция-точка входа для запуска процесса "сглаживания".
///
/// # Аргументы
/// * `args` - Ссылка на структуру `Args` с параметрами командной строки.
///
/// # Ошибки
/// Возвращает ошибку, если возникают проблемы с файловыми операциями,
/// настройкой потоков или обработкой данных.
pub async fn run(args: &Args) -> Result<()> {
    if (args.list_templates
        || args.show_enabled
        || args.force_update
        || !args.enable_templates.is_empty()
        || !args.disable_templates.is_empty())
        && args.folders.is_empty()
    {
        let _ = FlattenConfig::new(args).await?;
        return Ok(());
    }

    if args.folders.is_empty() {
        return Err(anyhow::anyhow!("Error: --folders argument is required. Use --help for more information."));
    }

    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()
            .context("Failed to configure thread pool")?;
    }

    let config = FlattenConfig::new(args).await?;

    println!("{} Starting flatten process...", ROCKET);
    println!("Processing {} folders", args.folders.len());
    if config.dry_run {
        println!("🔍 DRY RUN MODE - No output file will be created");
    } else {
        println!("Output file: {}", args.output.display());
    }
    println!();

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
    let mut any_folder_found = false;

    for base_folder in &args.folders {
        if !base_folder.exists() {
            eprintln!(
                "Warning: Folder {} does not exist, skipping",
                base_folder.display()
            );
            continue;
        }
        any_folder_found = true;

        println!("Processing folder: {}", base_folder.display());

        if let Some(ref mut output) = output_file {
            print_folder_structure(base_folder, output, &config)?;
        } else {
            println!("📁 Folder structure for {}", base_folder.display());
            let mut console_output = Vec::new();
            print_folder_structure(base_folder, &mut console_output, &config)?;
            println!("{}", String::from_utf8_lossy(&console_output));
        }

        let files = collect_files(base_folder, &config)?;
        let file_count = files.len();
        total_files.fetch_add(file_count, Ordering::Relaxed);

        if file_count == 0 {
            println!("No files found in {}", base_folder.display());
            continue;
        }

        let pb = ProgressBar::new(file_count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(PROGRESS_STYLE)
                .context("Invalid progress bar template")?
                .progress_chars("#>-"),
        );

        if let Some(ref mut output) = output_file {
            writeln!(
                output,
                "### DIRECTORY {} FLATTENED CONTENT ###",
                base_folder.display()
            )?;
        } else {
            println!("📄 Files to process from {}:", base_folder.display());
        }

        let results = process_files_parallel(files, &config, Some(pb.clone()));

        for (file_path, content_result) in results {
            if let Some(ref mut output) = output_file {
                writeln!(output, "### {} BEGIN ###", file_path.display())?;
                match content_result {
                    Ok((content, bytes_processed)) => {
                        output.write_all(content.as_bytes())?;
                        total_bytes_processed
                            .fetch_add(bytes_processed as usize, Ordering::Relaxed);
                    }
                    Err(e) => {
                        writeln!(output, "[Error reading file: {}]", e)?;
                    }
                }
                writeln!(output, "\n### {} END ###\n", file_path.display())?;
            } else {
                match content_result {
                    Ok((_, bytes_processed)) => {
                        println!("  ✅ {} ({} bytes)", file_path.display(), bytes_processed);
                        total_bytes_processed
                            .fetch_add(bytes_processed as usize, Ordering::Relaxed);
                    }
                    Err(e) => {
                        println!("  ❌ {} ({})", file_path.display(), e);
                    }
                }
            }
        }

        if let Some(ref mut output) = output_file {
            writeln!(
                output,
                "### DIRECTORY {} FLATTENED CONTENT ###",
                base_folder.display()
            )?;
        }

        pb.finish_with_message("Done");
    }

    if !any_folder_found {
        return Ok(());
    }

    println!();
    println!("{} Flatten completed successfully!", style("✓").green());
    let total = total_files.load(Ordering::Relaxed);
    println!("Total files processed: {}", total);

    if config.show_stats {
        print_stats(total, total_bytes_processed.load(Ordering::Relaxed) as u64);
    }

    if !config.dry_run {
        println!("Output written to: {}", args.output.display());
    }

    Ok(())
}

/// Выводит статистику по завершении работы.
fn print_stats(total_files: usize, total_bytes: u64) {
    const KB: f64 = 1024.0;
    const MB: f64 = 1_048_576.0;

    let bytes_str = if total_bytes as f64 >= MB {
        format!("{:.2} MB", total_bytes as f64 / MB)
    } else if total_bytes as f64 >= KB {
        format!("{:.2} KB", total_bytes as f64 / KB)
    } else {
        format!("{} bytes", total_bytes)
    };
    println!("Total bytes processed: {}", bytes_str);

    if total_files > 0 {
        let avg_size = total_bytes / total_files as u64;
        let avg_str = if avg_size as f64 >= KB {
            format!("{:.2} KB", avg_size as f64 / KB)
        } else {
            format!("{} bytes", avg_size)
        };
        println!("Average file size: {}", avg_str);
    }
}

/// Рекурсивно собирает пути ко всем файлам в директории, учитывая конфигурацию.
fn collect_files(directory: &Path, config: &FlattenConfig) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut walkdir = WalkDir::new(directory).follow_links(false);

    if config.max_depth > 0 {
        walkdir = walkdir.max_depth(config.max_depth);
    }

    for entry in walkdir
        .into_iter()
        .filter_entry(|e| !config.should_skip_path(e.path()))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

/// Выводит в `writer` древовидную структуру директории.
fn print_folder_structure<W: Write>(
    directory: &Path,
    writer: &mut W,
    config: &FlattenConfig,
) -> Result<()> {
    writeln!(
        writer,
        "### DIRECTORY {} FOLDER STRUCTURE ###",
        directory.display()
    )?;

    let mut walkdir = WalkDir::new(directory).follow_links(false);
    if config.max_depth > 0 {
        walkdir = walkdir.max_depth(config.max_depth);
    }

    for entry in walkdir.into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            !config.should_skip_path(e.path()) || config.show_skipped
        } else {
            !config.should_skip_file(e.path())
        }
    }) {
        let entry = entry?;
        let path = entry.path();
        let depth = entry.depth();
        if depth == 0 {
            continue;
        }

        let indent = "    ".repeat(depth - 1);
        let file_name = path.file_name().unwrap_or_else(|| OsStr::new(""));

        if entry.file_type().is_dir() {
            if config.should_skip_path(path) {
                writeln!(
                    writer,
                    "{}{} {}/ (skipped)",
                    indent,
                    SKIP,
                    file_name.to_string_lossy()
                )?;
            } else {
                writeln!(
                    writer,
                    "{}{} {}/",
                    indent,
                    FOLDER,
                    file_name.to_string_lossy()
                )?;
            }
        } else {
            writeln!(
                writer,
                "{}{} {}",
                indent,
                FILE,
                file_name.to_string_lossy()
            )?;
        }
    }

    writeln!(
        writer,
        "### DIRECTORY {} FOLDER STRUCTURE ###\n",
        directory.display()
    )?;
    Ok(())
}

/// Эффективно читает содержимое файла, используя memory-mapping.
fn read_file_content_fast(path: &Path, max_size: u64) -> Result<(String, u64)> {
    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
    let metadata = file
        .metadata()
        .with_context(|| format!("Failed to read metadata: {}", path.display()))?;
    let file_size = metadata.len();

    if max_size > 0 && file_size > max_size {
        return Ok((
            format!("[File too large: {} bytes]", file_size),
            file_size,
        ));
    }
    if file_size == 0 {
        return Ok((String::new(), 0));
    }

    // SAFETY: Memory mapping a file is safe. The file is read-only, and the lifetime
    // of the mmap is tied to the function's scope, ensuring the file handle outlives it.
    // The underlying file is not modified while the map is active.
    let mmap = unsafe {
        MmapOptions::new()
            .map(&file)
            .with_context(|| format!("Failed to memory map file: {}", path.display()))?
    };

    let content =
        String::from_utf8(mmap.to_vec()).unwrap_or_else(|_| String::from_utf8_lossy(&mmap).into());

    Ok((content, file_size))
}

/// Обрабатывает список файлов в параллельном режиме.
fn process_files_parallel(
    files: Vec<PathBuf>,
    config: &FlattenConfig,
    progress_bar: Option<ProgressBar>,
) -> Vec<(PathBuf, Result<(String, u64)>)> {
    let processed_count = AtomicUsize::new(0);

    files
        .into_par_iter()
        .map(|file_path| {
            let result = if config.should_skip_file(&file_path) {
                Ok((
                    format!("[Binary file skipped: {}]", file_path.display()),
                    0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Создает временную структуру директорий и файлов для тестов.
    fn create_test_structure() -> Result<TempDir> {
        let temp_dir = tempfile::tempdir()?;
        fs::create_dir_all(temp_dir.path().join("src"))?;
        fs::create_dir_all(temp_dir.path().join(".hidden_dir"))?;
        fs::write(
            temp_dir.path().join("src/main.rs"),
            "fn main() {}",
        )?;
        fs::write(temp_dir.path().join(".hidden_file.txt"), "hidden")?;
        fs::write(temp_dir.path().join("config.exe"), "binary")?;
        Ok(temp_dir)
    }

    #[tokio::test]
    async fn test_config_skip_path() -> Result<()> {
        let temp_dir = create_test_structure()?;
        let args = Args::parse_from([
            "flatten-rust",
            "-f",
            temp_dir.path().to_str().expect("path is utf8"),
            "-s",
            "skip_me",
        ]);
        let config = FlattenConfig::new(&args).await?;

        assert!(config.should_skip_path(Path::new("skip_me")));
        assert!(!config.should_skip_path(Path::new("src")));
        // Тест скрытых файлов
        assert!(config.should_skip_path(Path::new(".hidden_dir")));
        Ok(())
    }

    #[tokio::test]
    async fn test_config_include_hidden() -> Result<()> {
        let temp_dir = create_test_structure()?;
        let args = Args::parse_from([
            "flatten-rust",
            "-f",
            temp_dir.path().to_str().expect("path is utf8"),
            "--include-hidden",
        ]);
        let config = FlattenConfig::new(&args).await?;

        assert!(!config.should_skip_path(Path::new(".hidden_dir")));
        Ok(())
    }

    #[tokio::test]
    async fn test_config_skip_file() -> Result<()> {
        let temp_dir = create_test_structure()?;
        let args = Args::parse_from([
            "flatten-rust",
            "-f",
            temp_dir.path().to_str().expect("path is utf8"),
            "-x",
            "exe",
        ]);
        let config = FlattenConfig::new(&args).await?;

        assert!(config.should_skip_file(Path::new("some.exe")));
        assert!(!config.should_skip_file(Path::new("main.rs")));
        Ok(())
    }
}
