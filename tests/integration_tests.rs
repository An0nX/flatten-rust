use anyhow::Result;
use std::fs;
use std::process::Command;
use tempfile::{tempdir, TempDir};

fn create_test_structure() -> Result<TempDir> {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("tests"))?;
    fs::create_dir_all(root.join("node_modules"))?;

    fs::write(root.join("src/main.rs"), "fn main() {}")?;
    fs::write(root.join("tests/integration.rs"), "#[test] fn t() {}")?;
    fs::write(root.join("README.md"), "# Test Project")?;
    fs::write(root.join("test.bin"), b"\x00\x01\x02")?;
    Ok(temp_dir)
}

fn run_flatten(args: &[&str]) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_flatten-rust"))
        .args(args)
        .output()
        .expect("Failed to execute command");

    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success(),
    )
}

#[test]
fn test_basic_flatten() {
    let temp_dir = create_test_structure().expect("Failed to create test structure");
    let output_file = temp_dir.path().join("output.md");

    let args = &[
        "-f",
        temp_dir.path().to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
        "--include-hidden",
    ];

    let (stdout, stderr, success) = run_flatten(args);

    assert!(success, "Command failed. stdout: {}, stderr: {}", stdout, stderr);
    assert!(output_file.exists(), "Output file was not created");

    let content = fs::read_to_string(&output_file).expect("Could not read output file");
    assert!(content.contains("FOLDER STRUCTURE"));
    assert!(content.contains("FLATTENED CONTENT"));
    assert!(content.contains("src/main.rs"));
    assert!(content.contains("README.md"));
}

#[test]
fn test_skip_folders() {
    let temp_dir = create_test_structure().expect("Failed to create test structure");
    let output_file = temp_dir.path().join("output.md");

    let args = &[
        "-f",
        temp_dir.path().to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
        "-s",
        "node_modules",
    ];

    let (stdout, stderr, success) = run_flatten(args);
    assert!(success, "Command failed. stdout: {}, stderr: {}", stdout, stderr);
    
    let content = fs::read_to_string(&output_file).expect("Could not read output file");
    assert!(!content.contains("node_modules"));
}

#[test]
fn test_show_skipped() {
    let temp_dir = create_test_structure().expect("Failed to create test structure");
    let output_file = temp_dir.path().join("output.md");
    
    let args = &[
        "-f", temp_dir.path().to_str().unwrap(),
        "-o", output_file.to_str().unwrap(),
        "-s", "node_modules",
        "-k", // --show-skipped
    ];

    let (stdout, stderr, success) = run_flatten(args);
    assert!(success, "Command failed. stdout: {}, stderr: {}", stdout, stderr);
    
    let content = fs::read_to_string(&output_file).expect("Could not read output file");
    assert!(content.contains("node_modules/ (skipped)"));
}

#[test]
fn test_error_on_missing_folder() {
    let args = &["-f", "/non/existent/path"];
    let (stdout, stderr, success) = run_flatten(args);
    assert!(success);
    assert!(stderr.contains("does not exist, skipping"));
    assert!(!stdout.contains("Flatten completed successfully"));
}

#[test]
fn test_error_no_folder_arg() {
    let args = &["-o", "output.md"];
    let (_stdout, stderr, success) = run_flatten(args);
    assert!(!success);
    assert!(stderr.contains("Error: --folders argument is required"));
}
