//! Integration tests for configuration loading.

use std::fs;
use tempfile::TempDir;

// Import the config module from the main crate
use pfp::config::read_config;

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

// Profile inheritance tests

#[test]
fn test_profile_inheritance_basic_config() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");

    let config = r#"{
        "profiles": {
            "base_profile": {
                "mode": "dir",
                "markers": [".git"],
                "ignore": ["node_modules", "target"],
                "depth": 10,
                "stop_on_marker": true
            },
            "child_profile": {
                "base": "base_profile",
                "depth": 5,
                "markers": ["Cargo.toml"]
            }
        },
        "include": ["/tmp"]
    }"#;

    fs::write(&config_path, config).unwrap();

    let result = read_config(config_path.to_str().unwrap());
    assert!(result.is_ok(), "Config should parse successfully");

    let config = result.unwrap();
    let child = config.profiles.get("child_profile").unwrap();

    // Child should have inherited fields from base
    assert_eq!(child.ignore, Some(vec!["node_modules".to_string(), "target".to_string()]));
    assert_eq!(child.stop_on_marker, Some(true));

    // Child should have its own overridden fields
    assert_eq!(child.depth, Some(5));
    assert_eq!(child.markers, Some(vec!["Cargo.toml".to_string()]));
}

#[test]
fn test_profile_inheritance_chain_config() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");

    let config = r#"{
        "profiles": {
            "grandparent": {
                "mode": "dir",
                "depth": 100,
                "markers": [".git"],
                "ignore": ["node_modules"]
            },
            "parent": {
                "base": "grandparent",
                "depth": 50
            },
            "child": {
                "base": "parent",
                "depth": 10,
                "traverse_hidden_dirs": true
            }
        },
        "include": ["/tmp"]
    }"#;

    fs::write(&config_path, config).unwrap();

    let result = read_config(config_path.to_str().unwrap());
    assert!(result.is_ok());

    let config = result.unwrap();
    let child = config.profiles.get("child").unwrap();

    // Should inherit from grandparent through parent chain
    assert_eq!(child.mode, Some(pfp::config::Mode::Dir));
    assert_eq!(child.markers, Some(vec![".git".to_string()]));
    assert_eq!(child.ignore, Some(vec!["node_modules".to_string()]));

    // Should have child's overrides
    assert_eq!(child.depth, Some(10));
    assert_eq!(child.traverse_hidden_dirs, Some(true));
}

#[test]
fn test_profile_inheritance_partial_override() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");

    let config = r#"{
        "profiles": {
            "base": {
                "mode": "dir",
                "markers": [".git"],
                "depth": 20,
                "ignore": ["node_modules", "target"],
                "show_hidden": false
            },
            "derived": {
                "base": "base",
                "ignore": ["custom_ignore"]
            }
        },
        "include": ["/tmp"]
    }"#;

    fs::write(&config_path, config).unwrap();

    let result = read_config(config_path.to_str().unwrap());
    assert!(result.is_ok());

    let config = result.unwrap();
    let derived = config.profiles.get("derived").unwrap();

    // Inherited fields
    assert_eq!(derived.mode, Some(pfp::config::Mode::Dir));
    assert_eq!(derived.markers, Some(vec![".git".to_string()]));
    assert_eq!(derived.depth, Some(20));
    assert_eq!(derived.show_hidden, Some(false));

    // Overridden field
    assert_eq!(derived.ignore, Some(vec!["custom_ignore".to_string()]));
}

#[test]
fn test_profile_inheritance_from_defaults() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");

    let config = r#"{
        "profiles": {
            "my_projects": {
                "base": "projects",
                "depth": 3
            }
        },
        "include": ["/tmp"]
    }"#;

    fs::write(&config_path, config).unwrap();

    let result = read_config(config_path.to_str().unwrap());
    assert!(result.is_ok());

    let config = result.unwrap();
    let my_projects = config.profiles.get("my_projects").unwrap();

    // Should inherit from built-in "projects" profile
    assert!(my_projects.markers.as_ref().unwrap().contains(&".git".to_string()));

    // Should have overridden depth
    assert_eq!(my_projects.depth, Some(3));
}

#[test]
fn test_profile_inheritance_multiple_children() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");

    let config = r#"{
        "profiles": {
            "base": {
                "mode": "dir",
                "markers": [".git"],
                "depth": 20
            },
            "child1": {
                "base": "base",
                "depth": 5
            },
            "child2": {
                "base": "base",
                "depth": 10,
                "traverse_hidden_dirs": true
            }
        },
        "include": ["/tmp"]
    }"#;

    fs::write(&config_path, config).unwrap();

    let result = read_config(config_path.to_str().unwrap());
    assert!(result.is_ok());

    let config = result.unwrap();
    let child1 = config.profiles.get("child1").unwrap();
    let child2 = config.profiles.get("child2").unwrap();

    // Both inherit from base
    assert_eq!(child1.markers, Some(vec![".git".to_string()]));
    assert_eq!(child2.markers, Some(vec![".git".to_string()]));

    // But have different overrides
    assert_eq!(child1.depth, Some(5));
    assert_eq!(child2.depth, Some(10));
    assert_eq!(child2.traverse_hidden_dirs, Some(true));
    assert!(child1.traverse_hidden_dirs.is_none());
}

#[test]
fn test_profile_inheritance_with_inline_override() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");

    let config = r#"{
        "profiles": {
            "base_profile": {
                "markers": [".git"],
                "depth": 20
            },
            "derived": {
                "base": "base_profile",
                "depth": 10
            }
        },
        "include": [
            {
                "paths": ["/tmp"],
                "profile": "derived",
                "depth": 5
            }
        ]
    }"#;

    fs::write(&config_path, config).unwrap();

    let result = read_config(config_path.to_str().unwrap());
    assert!(result.is_ok());

    let config = result.unwrap();

    // Verify the inherited profile is set up correctly
    let derived = config.profiles.get("derived").unwrap();
    assert_eq!(derived.depth, Some(10));
    assert_eq!(derived.markers, Some(vec![".git".to_string()]));

    // Inline override should take precedence when resolving
    let resolved = config.resolve_profile(&config.include[0]);
    assert_eq!(resolved.depth, 5); // Inline override
    assert_eq!(resolved.markers, vec![".git".to_string()]); // From inherited profile
}
#[test]
fn test_profile_inheritance_circular_dependency() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");

    let config = r#"{
        "profiles": {
            "a": {
                "base": "b",
                "depth": 5
            },
            "b": {
                "base": "a",
                "depth": 10
            }
        },
        "include": ["/tmp"]
    }"#;

    fs::write(&config_path, config).unwrap();

    let result = read_config(config_path.to_str().unwrap());
    // Should return error for circular dependency
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Circular"));
}

#[test]
fn test_profile_inheritance_missing_base_error() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");

    let config = r#"{
        "profiles": {
            "orphan": {
                "base": "nonexistent_profile",
                "depth": 5
            }
        },
        "include": ["/tmp"]
    }"#;

    fs::write(&config_path, config).unwrap();

    let result = read_config(config_path.to_str().unwrap());
    // Should return error when base profile doesn't exist
    assert!(result.is_err());
}

