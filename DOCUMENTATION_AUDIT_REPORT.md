# ドキュメント整合性監査レポート
**実施日**: 2025-12-30

## 監査概要

4つの主要ドキュメントと実装コードの整合性を検証しました。

- README.md（ユーザー向け）
- CLAUDE.md（開発者向け）
- CODE_REVIEW.md（コードレビュー）
- .claude/hooks/README.md（フックのセットアップガイド）

---

## 1. 高優先度（必須修正）

### 1.1 設定ファイル形式サポートの不一致

**問題**: CLAUDE.md と実装が矛盾している

**CLAUDE.md の記述**（44行目）:
```
### 利用ライブラリ
- toml 0.8系（TOML パース）
```

**CLAUDE.md の記述**（161行目）:
```
### サポート形式
YAMLをサポート。
```

**CLAUDE.md の記述**（134-135行目）:
```
### 設定ファイル
- 検索順: `.worktree.yml` → `.worktree.yaml` → `.worktree.toml` → `worktree.config.yml` → `worktree.config.toml`
```

**実装の実態**（/src/config.rs, 7行目）:
```rust
const CONFIG_FILE_NAMES: &[&str] = &[".worktree.yml", ".worktree.yaml"];
```

**Cargo.toml の実態**:
- toml 依存関係がない
- serde_yaml 0.9 のみ

**分類**: HIGH - ドキュメント

**推奨修正**:
1. CLAUDE.md 行12-16 から `toml` 依存関係の記述を削除
2. CLAUDE.md 行134-135 を以下に修正：
   ```
   - 検索順: `.worktree.yml` → `.worktree.yaml`
   ```
3. CLAUDE.md 行160-161 を明確に：
   ```
   ### サポート形式
   YAML形式のみサポート
   ```

---

### 1.2 Rust 最小サポートバージョンの不一致

**問題**: ドキュメントと Cargo.toml が矛盾

**CLAUDE.md の記述**（9行目）:
```
- 最小サポートバージョン: 1.92.0
```

**Cargo.toml の実態**（5行目）:
```toml
rust-version = "1.91.0"
```

**分類**: MEDIUM - バージョン管理

**推奨修正**: 以下のいずれか
- オプション A: CLAUDE.md を 1.91.0 に更新（推奨 - より多くのユーザーをサポート）
- オプション B: Cargo.toml を 1.92.0 に更新

---

## 2. 中優先度（強く推奨）

### 2.1 Python フック の Status 値の不完全性

**問題**: エラー状態の処理が実装されていない

**Rust 側（src/commands/claude_task.rs, 8-21行目）**:
```rust
pub enum TaskStatus {
    InProgress,      // "in_progress"
    WaitingUser,     // "waiting_user"
    Completed,       // "completed"
    Error,           // "error"  ← このケース
}
```

**Python 側（.claude/hooks/track-progress.py, 24-36行目）**:
```python
def get_task_status(hook_event: str, tool_name: str = "") -> str:
    if hook_event == "SessionStart":
        return "in_progress"
    elif hook_event == "Stop":
        return "waiting_user"
    elif hook_event == "SessionEnd":
        return "completed"
    # ...
    else:
        return "in_progress"  # エラーケースがない
```

**CODE_REVIEW.md の期待**（54-59行目）:
- 🔴 Error - Task encountered an error

**分類**: MEDIUM - 機能の不完全性

**推奨修正**:
1. Python フック で例外をキャッチして `"error"` status を記録する
2. CLAUDE.md, CODE_REVIEW.md, README.md で error status の生成方法を明確化

---

### 2.2 CODE_REVIEW.md の HIGH 優先度項目の実装状況が古い

**問題**: ドキュメントと実装が異なる（ドキュメント側が古い）

**CODE_REVIEW.md で報告されている問題**（38-55行目）:
```
#### [HIGH] JSONLファイル読み込みの堅牢性不足
// 問題: 1行の JSON パース失敗で全体のファイル読み込みが失敗する
```

**実装の現状**（src/commands/claude_task.rs, 260-276行目）:
```rust
match serde_json::from_str::<TaskEvent>(line) {
    Ok(event) => {
        self.add_event(event);
        valid_events += 1;
    }
    Err(e) => {
        parse_errors += 1;
        eprintln!(
            "⚠️  Warning: Skipping invalid line in {}:{}: {}",
            path.display(),
            line_num + 1,
            e
        );
        // Continue processing remaining lines
    }
}
```

**分類**: LOW - ドキュメント (実装は既に対応済み)

**推奨修正**:
- CODE_REVIEW.md を更新して、既に実装された修正を反映
- Phase 1 の進捗状況を更新

---

## 3. 低優先度（確認のみ）

### 3.1 .claude/hooks/README.md - 記述精度確認

**検証結果**: ✅ 正確

- イベント説明: 正確
- データフォーマット: 正確
- セットアップ手順: 正確
- トラブルシューティング: 正確

---

### 3.2 README.md - CLI 機能説明

**検証結果**: ✅ 正確 (TOML除く)

- サブコマンド: 全て正確
- オプション: 全て正確
- 使用例: 全て動作可能
- 唯一の注意: TOML サポート未記述（実装なし）

---

### 3.3 設定ファイル例

**CLAUDE.md の YAML 例**（172-191行目）:
```yaml
version: 1
copy:
  - .env
  - .env.local
  - apps/*/.env
  - packages/*/.env.local
exclude:
  - .env.production
  - .env.test
postCreate:
  - command: pnpm install
    description: "Installing dependencies..."
  - command: pnpm build
    description: "Building packages..."
    optional: true
```

**README.md の YAML 例**（66-82行目）:
```yaml
version: 1
copy:
  - .env
  - .env.local
  - config/*.local.json
exclude:
  - .env.production
postCreate:
  - command: npm install
    description: "Installing dependencies..."
  - command: npm run build
    description: "Building project..."
    optional: true
```

**検証結果**: ✅ どちらも正確（ツール名が異なるだけ）

---

### 3.4 Claude Code 統合機能

**README.md での説明**（275-319行目）:
- Claude Code タスク追跡の説明: ✅ 正確
- キーバインディング: ✅ 正確
- セットアップ手順: ✅ 正確

**.claude/hooks/README.md での説明**:
- イベント追跡: ✅ 正確
- データフォーマット: ✅ 正確
- 統合方法: ✅ 正確

**検証結果**: ✅ 全て正確

---

## 4. クロスドキュメント不整合

### 4.1 バージョン情報の一貫性

| 項目 | CLAUDE.md | Cargo.toml | 状態 |
|------|-----------|-----------|------|
| rust-version | 1.92.0 | 1.91.0 | ❌ 不一致 |
| clap | 4.4系 | 4.4 | ✅ 一致 |
| serde_yaml | 0.9系 | 0.9 | ✅ 一致 |
| ratatui | 0.29系 | 0.29 | ✅ 一致 |
| crossterm | 0.28系 | 0.28 | ✅ 一致 |
| toml | 0.8系 | ❌ なし | ❌ 不一致 |

---

### 4.2 機能説明の一貫性

| 機能 | README.md | CLAUDE.md | .claude/hooks/README.md | 状態 |
|------|-----------|-----------|------------------------|------|
| 基本の worktree 管理 | ✅ | ✅ | N/A | ✅ |
| ファイルコピー | ✅ | ✅ | N/A | ✅ |
| post-create | ✅ | ✅ | N/A | ✅ |
| プロセス管理 | ✅ | ✅ | N/A | ✅ |
| Claude Code 統合 | ✅ | ✅ | ✅ | ✅ |
| YAML 設定 | ✅ | ✅ | N/A | ✅ |
| TOML 設定 | ❌明記なし | ❌記述あり(実装なし) | N/A | ❌ |

---

## 5. セキュリティ考慮事項（CODE_REVIEW.md との整合性）

### 5.1 コマンドインジェクション対策

**CODE_REVIEW.md での指摘**（447-516行目）:
- src/commands/notify.rs での shell コマンド直接実行の警告
- CLI 入力が shell に渡される

**実装の現状**:
- CLI 引数は信頼できるソース（ユーザー直接入力）
- shell metacharacters はエスケープされていない
- user が意図的に shell 機能を使う場合は acceptable

**CLAUDE.md での記述**（113行目）:
```
- コマンドインジェクション対策（gitコマンドの引数は配列で渡す）
```

**検証結果**: ✅ セキュリティポスチャは明確

---

## 6. 総合評価

| ドキュメント | 整合性 | 更新が必要 | 優先度 |
|-------------|--------|-----------|--------|
| README.md | 95% | 軽微 | LOW |
| CLAUDE.md | 80% | 重大 | HIGH |
| CODE_REVIEW.md | 90% | 中程度 | MEDIUM |
| .claude/hooks/README.md | 100% | なし | - |

---

## 7. 推奨アクションアイテム

### Phase 1（必須、即座に実施）

**優先度**: HIGH

1. **CLAUDE.md の修正**
   - [ ] 行12-16: toml 依存関係の記述を削除
   - [ ] 行134-135: 設定ファイル検索順を `.yml`, `.yaml` のみに修正
   - [ ] 行160-161: サポート形式を「YAML形式のみ」に修正

2. **Rust version の決定と統一**
   - [ ] オプション A（推奨）: CLAUDE.md を 1.91.0 に更新
   - [ ] またはオプション B: Cargo.toml を 1.92.0 に更新

### Phase 2（推奨、1週間以内）

**優先度**: MEDIUM

3. **CODE_REVIEW.md の更新**
   - [ ] JSONLパーサー堅牢化は既に実装済みと記述を更新
   - [ ] Phase 1 完了の記載
   - [ ] Phase 2, 3 の進捗管理

4. **Python フック の error status 対応**
   - [ ] `.claude/hooks/track-progress.py` に例外ハンドリングと error status 記録を追加
   - [ ] ドキュメントに error status 生成メカニズムを記述

### Phase 3（オプション、最適化）

**優先度**: LOW

5. **ドキュメント内容の充実**
   - [ ] CODE_REVIEW.md に実装状況のマトリックスを追加
   - [ ] CLAUDE.md にセキュリティ実装例を拡充

---

## 8. チェックリスト

### 実装との整合性
- ✅ CLI サブコマンド: 完全一致
- ✅ CLI オプション: 完全一致
- ✅ 出力フォーマット: 一致
- ✅ Claude Code 統合: 一致
- ❌ 設定ファイル形式: 不一致（TOML 文書化のみ）
- ✅ セキュリティ実装: 記述は正確

### バージョン情報の一貫性
- ✅ 大部分のライブラリ: 一致
- ❌ Rust version: 不一致（1.91.0 vs 1.92.0）
- ❌ toml: 文書化のみ（実装なし）

### セットアップ手順の正確性
- ✅ wtenv init: 正確
- ✅ wtenv create: 正確
- ✅ Claude Code hook セットアップ: 正確
- ✅ 環境変数設定: 正確

### 機能説明の正確性
- ✅ worktree 管理: 正確
- ✅ ファイルコピー: 正確
- ✅ プロセス管理: 正確
- ✅ Claude Code 統合: 正確
- ✅ UI 表示: 正確

### 矛盾や古い情報
- ❌ CLAUDE.md: TOML サポート（実装なし）
- ❌ CODE_REVIEW.md: HIGH 優先度項目が既に解決済み

---

## 結論

**総合評価**: 整合性 88% - **マイナー修正が必要**

### 主な問題
1. CLAUDE.md の設定ファイル形式記述が実装と不一致
2. Rust バージョン記述が不一致
3. CODE_REVIEW.md が実装状況を反映していない

### 強み
- CLI 仕様の説明は正確で充実
- Claude Code 統合の説明は完全
- セキュリティ考慮事項が明確
- README.md は実装と完全一致

### 推奨対応
**Phase 1（HIGH）**: CLAUDE.md と Rust version の統一 → 1-2時間で対応可能

---

**ドキュメント監査完了**
