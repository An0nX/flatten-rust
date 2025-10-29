use criterion::{criterion_group, criterion_main, Criterion};
use std::{fs, hint};
use std::path::Path;
use flatten_rust::FlattenConfig;
use tempfile::tempdir;
use rayon::prelude::*;

fn create_test_structure() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    
    // Create some test files
    for i in 0..100 {
        let file_path = src_dir.join(format!("test_{}.rs", i));
        fs::write(&file_path, format!("// Test file {}\nfn main() {{}}\n", i)).unwrap();
    }
    
    temp_dir
}

fn collect_files(directory: &Path, _config: &FlattenConfig) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let walkdir = walkdir::WalkDir::new(directory).follow_links(false);
    
    for entry in walkdir {
        let entry = hint::black_box(entry.unwrap());
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    
    files
}

fn bench_collect_files(c: &mut Criterion) {
    let temp_dir = create_test_structure();
    let config = FlattenConfig::default();
    
    c.bench_function("collect_files", |b| {
        b.iter(|| {
            let files = collect_files(temp_dir.path(), &config);
            hint::black_box(files);
        })
    });
}

fn bench_file_reading(c: &mut Criterion) {
    let temp_dir = create_test_structure();
    let files = collect_files(temp_dir.path(), &FlattenConfig::default());
    
    c.bench_function("file_reading", |b| {
        b.iter(|| {
            for file_path in &files {
                let content = fs::read_to_string(hint::black_box(file_path));
                let _ = hint::black_box(content);
            }
        })
    });
}

fn bench_parallel_file_reading(c: &mut Criterion) {
    let temp_dir = create_test_structure();
    let files = collect_files(temp_dir.path(), &FlattenConfig::default());
    
    c.bench_function("parallel_file_reading", |b| {
        b.iter(|| {
            let results: Vec<_> = files
                .par_iter()
                .map(|file_path| {
                    let content = fs::read_to_string(hint::black_box(file_path));
                    hint::black_box(content)
                })
                .collect();
            hint::black_box(results);
        })
    });
}

fn bench_memory_mapping(c: &mut Criterion) {
    let temp_dir = create_test_structure();
    let files = collect_files(temp_dir.path(), &FlattenConfig::default());
    
    c.bench_function("memory_mapping", |b| {
        b.iter(|| {
            for file_path in &files {
                if let Ok(file) = std::fs::File::open(file_path) {
                    if let Ok(map) = unsafe { memmap2::MmapOptions::new().map(&file) } {
                        let content = std::str::from_utf8(&map);
                        let _ = hint::black_box(content);
                    }
                }
            }
        })
    });
}

fn bench_skip_filtering(c: &mut Criterion) {
    let temp_dir = create_test_structure();
    let _config = FlattenConfig {
        skip_folders: vec!["target".to_string(), "node_modules".to_string()].into_iter().collect(),
        skip_extensions: vec!["exe".to_string(), "dll".to_string()].into_iter().collect(),
        ..Default::default()
    };
    
    c.bench_function("skip_filtering", |b| {
        b.iter(|| {
            let walkdir = walkdir::WalkDir::new(temp_dir.path())
                .follow_links(false)
                .max_depth(10);
            
            let mut count = 0;
            for entry in walkdir {
                let entry = hint::black_box(entry.unwrap());
                if entry.file_type().is_file() {
                    count += 1;
                }
            }
            hint::black_box(count);
        })
    });
}

fn bench_string_formatting(c: &mut Criterion) {
    let temp_dir = create_test_structure();
    let files = collect_files(temp_dir.path(), &FlattenConfig::default());
    
    c.bench_function("string_formatting", |b| {
        b.iter(|| {
            let mut string_content = String::new();
            for file_path in &files {
                string_content.push_str(&format!("File: {}\n", file_path.display()));
                if let Ok(content) = fs::read_to_string(file_path) {
                    string_content.push_str(&format!("Content: {}\n", content));
                }
            }
            hint::black_box(string_content);
        })
    });
}

criterion_group!(
    benches,
    bench_collect_files,
    bench_file_reading,
    bench_parallel_file_reading,
    bench_memory_mapping,
    bench_skip_filtering,
    bench_string_formatting
);
criterion_main!(benches);