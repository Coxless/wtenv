use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use std::path::PathBuf;

/// ブランチ名を対話的に入力
pub fn prompt_branch_name() -> Result<String> {
    let branch: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("ブランチ名")
        .interact_text()?;

    let branch = branch.trim().to_string();
    if branch.is_empty() {
        anyhow::bail!("ブランチ名を入力してください");
    }

    Ok(branch)
}

/// worktreeパスを対話的に入力
pub fn prompt_worktree_path(default: &str) -> Result<PathBuf> {
    let path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("worktreeパス")
        .default(default.to_string())
        .interact_text()?;

    Ok(PathBuf::from(path))
}

/// 削除確認
pub fn confirm_remove(path: &std::path::Path) -> Result<bool> {
    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("本当に削除しますか？: {}", path.display()))
        .default(false)
        .interact()?;

    Ok(confirmed)
}

/// 上書き確認
pub fn confirm_overwrite(path: &std::path::Path) -> Result<bool> {
    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "既存のファイルを上書きしますか？: {}",
            path.display()
        ))
        .default(false)
        .interact()?;

    Ok(confirmed)
}

#[cfg(test)]
mod tests {
    // 対話型のテストは実行が難しいため、コンパイルチェックのみ
}
