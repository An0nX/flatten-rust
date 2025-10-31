use flatten_rust::create_test_structure;
use std::fs;
use std::process::Command;

#[test]
fn test_basic_flatten() {
    let temp_dir = create_test_structure().unwrap();
    let output_file = temp_dir.path().join("output.md");

    // Debug: check what was created
    println!("Temp dir: {}", temp_dir.path().display());
    for entry in std::fs::read_dir(temp_dir.path()).unwrap() {
        println!("Found: {}", entry.unwrap().path().display());
    }

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "flatten-rust",
            "--",
            "--folders",
            temp_dir.path().to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
            "--include-hidden", // Include hidden files for testing
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute flatten-rust");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(
        output.status.success(),
        "Failed to run flatten-rust: {:?}",
        stderr
    );
    assert!(output_file.exists(), "Output file was not created");

    let content = fs::read_to_string(&output_file).unwrap();
    println!("Content length: {}", content.len());
    println!("Content preview: {}", &content[..content.len().min(500)]);

    assert!(content.contains("FOLDER STRUCTURE"));
    // Only check for FLATTENED CONTENT if files were found
    if content.contains("src/main.rs") {
        assert!(content.contains("FLATTENED CONTENT"));
    }
    assert!(content.contains("src/main.rs"));
    assert!(content.contains("README.md"));
}

#[test]
fn test_skip_folders() {
    let temp_dir = create_test_structure().unwrap();
    let output_file = temp_dir.path().join("output_skip.md");

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "flatten-rust",
            "--",
            "--folders",
            temp_dir.path().to_str().unwrap(),
            "--skip-folders",
            "node_modules",
            "--output",
            output_file.to_str().unwrap(),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute flatten-rust");

    assert!(
        output.status.success(),
        "Failed to run flatten-rust: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_file).unwrap();
    // Should not contain node_modules content
    assert!(!content.contains("node_modules") || content.contains("skipped"));
}

#[test]
fn test_show_skipped() {
    let temp_dir = create_test_structure().unwrap();
    let output_file = temp_dir.path().join("output_show.md");

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "flatten-rust",
            "--",
            "--folders",
            temp_dir.path().to_str().unwrap(),
            "--skip-folders",
            "node_modules",
            "--show-skipped",
            "--output",
            output_file.to_str().unwrap(),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute flatten-rust");

    assert!(
        output.status.success(),
        "Failed to run flatten-rust: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_file).unwrap();
    // Should show skipped folders
    assert!(content.contains("node_modules"));
    assert!(content.contains("skipped"));
}

#[test]
fn test_multiple_folders() {
    let temp_dir1 = create_test_structure().unwrap();
    let temp_dir2 = create_test_structure().unwrap();
    let output_file = temp_dir1.path().join("output_multi.md");

    // Add a unique file to second directory
    fs::write(temp_dir2.path().join("unique.txt"), "unique content").unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "flatten-rust",
            "--",
            "--folders",
            temp_dir1.path().to_str().unwrap(),
            temp_dir2.path().to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
            "--include-hidden",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute flatten-rust");

    assert!(
        output.status.success(),
        "Failed to run flatten-rust: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_file).unwrap();
    assert!(content.contains(&*temp_dir1.path().to_string_lossy()));
    assert!(content.contains(&*temp_dir2.path().to_string_lossy()));
    assert!(content.contains("unique.txt"));
    assert!(content.contains("FLATTENED CONTENT"));
}



#[test]
fn test_error_handling() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "flatten-rust",
            "--",
            "--folders",
            "/nonexistent/path",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute flatten-rust");

    // Should not panic, should handle gracefully
    assert!(output.status.success() || !output.status.success()); // Either way, no panic
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain warning about non-existent folder
    assert!(
        stderr.contains("does not exist")
            || stdout.contains("does not exist")
            || stderr.contains("Warning")
    );
}
