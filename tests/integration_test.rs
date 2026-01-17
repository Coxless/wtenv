use std::process::Command;

/// ccmon help command displays correctly
#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Claude Code Monitor"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("ui"));
}

/// init command help displays correctly
#[test]
fn test_init_command_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "init", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Initialize Claude Code hooks"));
}

/// ui command help displays correctly
#[test]
fn test_ui_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "ui", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Interactive TUI"));
}

/// version flag displays correctly
#[test]
fn test_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ccmon"));
}
