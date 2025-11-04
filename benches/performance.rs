use criterion::{criterion_group, criterion_main, Criterion};
use flatten_rust::{run, Args};
use std::fs;
use tempfile::{tempdir, TempDir};

fn create_large_test_structure(num_files: usize) -> TempDir {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();
    let src_dir = root.join("src");
    fs::create_dir(&src_dir).expect("Failed to create src dir");

    for i in 0..num_files {
        let file_path = src_dir.join(format!("file_{}.rs", i));
        fs::write(&file_path, "fn main() { /* some content */ }")
            .expect("Failed to write file");
    }
    temp_dir
}

fn bench_flatten_performance(c: &mut Criterion) {
    let temp_dir = create_large_test_structure(100);
    let test_dir_path = temp_dir.path().to_path_buf();
    let output_path = test_dir_path.join("output.md");

    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    c.bench_function("flatten_100_files", |b| {
        b.to_async(&runtime).iter(|| async {
            let args = Args {
                folders: vec![test_dir_path.clone()],
                output: output_path.clone(),
                skip_folders: vec![".git".to_string()],
                skip_extensions: vec!["log".to_string()],
                show_skipped: false,
                threads: 0,
                max_file_size: 0,
                auto_detect: false,
                include_hidden: false,
                max_depth: 0,
                show_stats: false,
                dry_run: false,
                list_templates: false,
                enable_templates: vec![],
                disable_templates: vec![],
                force_update: false,
                show_enabled: false,
            };
            run(std::hint::black_box(&args))
                .await
                .expect("Run failed");
        })
    });
}

criterion_group!(benches, bench_flatten_performance);
criterion_main!(benches);
