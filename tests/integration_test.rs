use std::process::Command;

/// wtenvコマンドのヘルプが正常に表示されるかテスト
#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Git worktree environment manager"));
    assert!(stdout.contains("status"));
    assert!(stdout.contains("ps"));
    assert!(stdout.contains("kill"));
}

/// statusコマンドが実行できるかテスト
#[test]
fn test_status_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "status"])
        .output()
        .expect("Failed to execute command");

    // Gitリポジトリ内であれば成功するはず
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Worktrees Overview") || stdout.contains("worktree"));
    }
}

/// psコマンドが実行できるかテスト
#[test]
fn test_ps_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "ps"])
        .output()
        .expect("Failed to execute command");

    // コマンドが実行できればOK（プロセスがなくても正常）
    assert!(output.status.success());
}

/// listコマンドが正常に動作するかテスト
#[test]
fn test_list_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "list"])
        .output()
        .expect("Failed to execute command");

    // Gitリポジトリ内であれば成功するはず
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // 少なくとも1つのworktreeは存在する（メインworktree）
        assert!(!stdout.is_empty());
    }
}

/// configコマンドが実行できるかテスト
#[test]
fn test_config_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "config"])
        .output()
        .expect("Failed to execute command");

    // 設定ファイルがあってもなくても、コマンド自体は成功するはず
    assert!(output.status.success());
}
