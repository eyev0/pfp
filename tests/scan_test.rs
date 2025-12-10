//! Integration tests for filesystem scanning.

use std::fs;
use tempfile::TempDir;

// We need to access internal types, so we use a different approach
// Integration tests test the public API behavior

/// Helper to create a test directory structure.
fn create_test_structure(base: &std::path::Path) {
    // Create project directories with markers
    let project1 = base.join("projects/rust-app");
    fs::create_dir_all(&project1).unwrap();
    fs::create_dir_all(project1.join("src")).unwrap();
    fs::write(project1.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    fs::write(project1.join("src/main.rs"), "fn main() {}").unwrap();

    let project2 = base.join("projects/go-app");
    fs::create_dir_all(&project2).unwrap();
    fs::write(project2.join("go.mod"), "module test").unwrap();

    // Create nested project
    let nested = base.join("projects/mono/packages/lib");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("package.json"), "{}").unwrap();

    // Create git repo
    let git_project = base.join("projects/git-repo");
    fs::create_dir_all(git_project.join(".git")).unwrap();
    fs::write(git_project.join("README.md"), "# Test").unwrap();

    // Create ignored directories
    let ignored = base.join("projects/rust-app/target/debug");
    fs::create_dir_all(&ignored).unwrap();
    fs::write(ignored.join("binary"), "").unwrap();

    let node_modules = base.join("projects/mono/node_modules/dep");
    fs::create_dir_all(&node_modules).unwrap();
    fs::write(node_modules.join("package.json"), "{}").unwrap();

    // Create files for file mode testing
    let scripts = base.join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(scripts.join("deploy.sh"), "#!/bin/bash").unwrap();
    fs::write(scripts.join("setup.py"), "print('setup')").unwrap();
    fs::write(scripts.join("readme.txt"), "readme").unwrap();

    // Create hidden directory (should be skipped)
    let hidden = base.join("projects/.hidden");
    fs::create_dir_all(&hidden).unwrap();
    fs::write(hidden.join("Cargo.toml"), "[package]").unwrap();
}

#[test]
fn test_directory_structure_created() {
    let temp = TempDir::new().unwrap();
    create_test_structure(temp.path());

    assert!(temp.path().join("projects/rust-app/Cargo.toml").exists());
    assert!(temp.path().join("projects/go-app/go.mod").exists());
    assert!(temp.path().join("projects/git-repo/.git").exists());
    assert!(temp.path().join("scripts/deploy.sh").exists());
}

// Note: Full integration tests for scan_paths would require
// making the function public or testing through CLI.
// These tests verify the test infrastructure works.
// 
// Ignore pattern tests are in src/fs.rs (unit tests) and test:
// - test_scan_ignores_default_patterns
// - test_scan_ignores_custom_pattern
// - test_scan_ignores_glob_pattern
// - test_scan_without_ignore_finds_all

#[test]
fn test_tempdir_cleanup() {
    let path;
    {
        let temp = TempDir::new().unwrap();
        path = temp.path().to_path_buf();
        create_test_structure(temp.path());
        assert!(path.exists());
    }
    // TempDir should clean up after going out of scope
    assert!(!path.exists());
}

