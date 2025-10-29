use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use tempfile::TempDir;
use walkdir::WalkDir;

fn create_large_test_structure() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Create a larger structure for benchmarking
    for i in 0..100 {
        let dir_path = temp_dir.path().join(format!("dir_{}", i));
        fs::create_dir_all(&dir_path).unwrap();

        for j in 0..10 {
            let file_path = dir_path.join(format!("file_{}.rs", j));
            fs::write(
                &file_path,
                format!("// File {} in dir {}\nfn test_{}() {{}}", j, i, j),
            )
            .unwrap();
        }
    }

    // Create some larger files
    for i in 0..10 {
        let file_path = temp_dir.path().join(format!("large_file_{}.txt", i));
        let content = "A".repeat(10000); // 10KB per file
        fs::write(&file_path, content).unwrap();
    }

    // Create some directories to skip
    fs::create_dir_all(temp_dir.path().join("node_modules")).unwrap();
    fs::create_dir_all(temp_dir.path().join("target")).unwrap();
    for i in 0..50 {
        let file_path = temp_dir
            .path()
            .join("node_modules")
            .join(format!("dep_{}.js", i));
        fs::write(&file_path, format!("// Dependency {}", i)).unwrap();
    }

    temp_dir
}

fn bench_collect_files(c: &mut Criterion) {
    let temp_dir = create_large_test_structure();

    c.bench_function("collect_files", |b| {
        b.iter(|| {
            let mut files = Vec::new();

            let walkdir = WalkDir::new(temp_dir.path())
                .follow_links(false)
                .max_depth(usize::MAX);

            for entry in walkdir.into_iter() {
                let entry = black_box(entry.unwrap());
                if entry.file_type().is_file() {
                    files.push(entry.path().to_path_buf());
                }
            }

            files
        })
    });
}

fn bench_file_reading(c: &mut Criterion) {
    let temp_dir = create_large_test_structure();

    // Collect files first
    let mut files = Vec::new();
    for entry in WalkDir::new(temp_dir.path()) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }

    c.bench_function("file_reading", |b| {
        b.iter(|| {
            for file_path in &files {
                let content = fs::read_to_string(black_box(file_path));
                let _ = black_box(content);
            }
        })
    });
}

fn bench_parallel_file_reading(c: &mut Criterion) {
    let temp_dir = create_large_test_structure();

    // Collect files first
    let mut files = Vec::new();
    for entry in WalkDir::new(temp_dir.path()) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }

    c.bench_function("parallel_file_reading", |b| {
        b.iter(|| {
            use rayon::prelude::*;

            let results: Vec<_> = files
                .par_iter()
                .map(|file_path| {
                    let content = fs::read_to_string(black_box(file_path));
                    black_box(content)
                })
                .collect();

            black_box(results);
        })
    });
}

fn bench_memory_mapping(c: &mut Criterion) {
    let temp_dir = create_large_test_structure();

    // Create a larger file for memory mapping
    let large_file_path = temp_dir.path().join("very_large.txt");
    let content = "B".repeat(1_000_000); // 1MB
    fs::write(&large_file_path, content).unwrap();

    c.bench_function("memory_mapping", |b| {
        b.iter(|| {
            use memmap2::MmapOptions;
            use std::fs::File;

            let file = File::open(&large_file_path).unwrap();
            let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

            let string_content = String::from_utf8_lossy(&mmap);
            black_box(string_content);
        })
    });
}

fn bench_skip_filtering(c: &mut Criterion) {
    let temp_dir = create_large_test_structure();

    c.bench_function("skip_filtering", |b| {
        b.iter(|| {
            let mut count = 0;

            for entry in WalkDir::new(temp_dir.path()) {
                let entry = black_box(entry.unwrap());

                if entry.file_type().is_dir() {
                    let file_name = entry.file_name().to_string_lossy();
                    if file_name == "node_modules" || file_name == "target" {
                        continue;
                    }
                }

                if entry.file_type().is_file() {
                    count += 1;
                }
            }

            black_box(count);
        })
    });
}

fn bench_string_operations(c: &mut Criterion) {
    let temp_dir = create_large_test_structure();
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, "fn main() { println!(\"Hello, world!\"); }").unwrap();

    c.bench_function("string_formatting", |b| {
        b.iter(|| {
            let content = format!(
                "### {} BEGIN ###\n{}\n### {} END ###\n",
                file_path.display(),
                "fn main() { println!(\"Hello, world!\"); }",
                file_path.display()
            );
            black_box(content);
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
    bench_string_operations
);

criterion_main!(benches);
