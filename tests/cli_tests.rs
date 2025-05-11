// Integration tests for CLI functionality
use std::env;
use std::process::Command;
use std::path::Path;

// Skip these tests when running in CI environments without API keys
#[test]
#[ignore]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kona"));
    // The version should match the version in Cargo.toml
}

#[test]
#[ignore]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check for expected help text
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("Options:"));
    
    // Check for commands
    assert!(stdout.contains("ask"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("config"));
    
    // Check for options
    assert!(stdout.contains("--debug"));
    assert!(stdout.contains("--streaming"));
    assert!(stdout.contains("--verbose"));
}

#[test]
#[ignore]
fn test_cli_init() {
    // We don't want to overwrite existing config
    let output = Command::new("cargo")
        .args(["run", "--", "init"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Config file") || stdout.contains("--force"));
}

#[test]
#[ignore]
fn test_cli_config() {
    let output = Command::new("cargo")
        .args(["run", "--", "config"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Current configuration:"));
    assert!(stdout.contains("API Key:"));
    assert!(stdout.contains("Model:"));
}