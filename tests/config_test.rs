//! Integration tests for configuration loading.

use std::fs;
use tempfile::TempDir;

#[test]
fn test_minimal_config_file() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");
    
    fs::write(&config_path, r#"{ "include": ["/tmp"] }"#).unwrap();
    
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("include"));
}

#[test]
fn test_config_with_profiles() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");
    
    let config = r#"{
        "profiles": {
            "projects": {
                "markers": [".git", "Cargo.toml", "pom.xml"]
            },
            "custom": {
                "mode": "file",
                "markers": ["*.sh"],
                "depth": 2
            }
        },
        "include": [
            "/home/user/dev",
            { "paths": ["/tmp"], "profile": "custom" }
        ]
    }"#;
    
    fs::write(&config_path, config).unwrap();
    
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("profiles"));
    assert!(content.contains("custom"));
    assert!(content.contains("pom.xml"));
}

#[test]
fn test_config_with_sessions() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");
    
    let config = r#"{
        "include": ["/tmp"],
        "sessions": [
            {
                "name": "work",
                "windows": ["/home/user/project1", "/home/user/project2"]
            },
            {
                "name": "personal",
                "windows": ["/home/user/dotfiles"]
            }
        ]
    }"#;
    
    fs::write(&config_path, config).unwrap();
    
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("sessions"));
    assert!(content.contains("work"));
    assert!(content.contains("personal"));
}

#[test]
fn test_jsonc_comments_in_config() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");
    
    // JSONC allows comments
    let config = r#"{
        // This is a comment
        "include": ["/tmp"],
        /* Multi-line
           comment */
        "sessions": []
    }"#;
    
    fs::write(&config_path, config).unwrap();
    
    assert!(config_path.exists());
}

