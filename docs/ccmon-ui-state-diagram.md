# ccmon UI 状態遷移図

## 概要

ccmon UI は Claude Code セッションの進捗を以下の4つの状態で管理する。

## 状態定義 (TaskStatus)

| 状態 | 表示 | 色 | 説明 |
|------|------|-----|------|
| `InProgress` | 🔵 In Progress | Blue | タスクがアクティブに実行中 |
| `Stop` | 🟡 Stop | Yellow | レスポンス完了、ユーザーアクション待ち |
| `SessionEnded` | ⚫ Session Ended | Gray | セッション終了 |
| `Error` | 🔴 Error | Red | エラー発生 |

## 状態遷移図

```
                                   ┌─────────────────────────────────────────────────────────────┐
                                   │                                                             │
                                   ▼                                                             │
┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐              ┌──────┴────────┐
│   (初期状態)     │         │   InProgress    │         │      Stop       │              │ SessionEnded  │
│   status: None   │────────▶│                 │────────▶│                 │─────────────▶│               │
│                 │  User    │  🔵 Blue        │  Stop   │  🟡 Yellow      │  SessionEnd  │  ⚫ Gray       │
└─────────────────┘  Prompt  └─────────────────┘  event  └─────────────────┘              └───────────────┘
                     Submit          │                           │
                                     │                           │
                                     │  PostToolUse              │ UserPromptSubmit
                                     │  (error)                  │ (再開)
                                     ▼                           │
                              ┌─────────────────┐                │
                              │     Error       │                │
                              │  🔴 Red         │────────────────┘
                              └─────────────────┘
                                     │
                                     │ PostToolUse (success)
                                     │ or UserPromptSubmit
                                     ▼
                              ┌─────────────────┐
                              │   InProgress    │
                              └─────────────────┘
```

## イベントと状態遷移

### track-progress.py の状態決定ロジック

| Hook Event | 発生タイミング | 設定される status |
|------------|---------------|-------------------|
| `SessionStart` | Claude Code 起動時 | `None` (状態なし) |
| `UserPromptSubmit` | ユーザーがプロンプト送信時 | `in_progress` |
| `PostToolUse` | ツール使用後 | `in_progress` (エラー時: `error`) |
| `Stop` | レスポンス完了時 | `stop` |
| `SessionEnd` | セッション終了時 | `session_ended` |
| `Notification` | 権限確認・入力待ち時 | `stop` (条件付き) or `None` |

### Rust側の状態処理 (claude_task.rs)

#### ClaudeTask::new() - 最初のイベント処理

```rust
// Line 177: status がない場合のデフォルト値
let status = event.status.unwrap_or(TaskStatus::InProgress);
```

**問題点**: SessionStart イベントは `status: None` だが、`unwrap_or()` により `InProgress` になる。

#### ClaudeTask::add_event() - 後続イベント処理

```rust
// Line 197-199: status がある場合のみ更新
if let Some(status) = event.status {
    self.status = status;
}
```

**動作**: `status: None` のイベントでは状態が変更されない（正しい動作）。

## 現在の問題

### 問題: Claude Code 起動時にすぐ InProgress になる

**原因**:
1. Claude Code 起動 → SessionStart hook 発火
2. `track-progress.py` は `status: None` を返す（正しい動作）
3. `ClaudeTask::new()` で `unwrap_or(TaskStatus::InProgress)` により **InProgress** になる

**期待される動作**:
- SessionStart 直後は「起動中」「待機中」などの中間状態
- UserPromptSubmit で初めて InProgress になるべき

## 表示フィルタリング

### active_tasks() のロジック

```rust
// Line 407-412
pub fn active_tasks(&self) -> Vec<&ClaudeTask> {
    self.all_tasks()
        .into_iter()
        .filter(|t| t.status != TaskStatus::SessionEnded && t.has_started())
        .collect()
}
```

### has_started() のロジック

```rust
// Line 278-286
pub fn has_started(&self) -> bool {
    self.events.len() > 1
        || self.events.first().is_some_and(|e| e.event != "SessionStart")
}
```

**効果**: SessionStart のみのタスクは `active_tasks()` に含まれない。

## 完全な状態遷移表

| 現在の状態 | イベント | 結果の状態 |
|-----------|---------|-----------|
| (なし) | SessionStart | `InProgress` (**問題: None であるべき**) |
| InProgress | UserPromptSubmit | InProgress |
| InProgress | PostToolUse (成功) | InProgress |
| InProgress | PostToolUse (エラー) | Error |
| InProgress | Stop | Stop |
| InProgress | SessionEnd | SessionEnded |
| Stop | UserPromptSubmit | InProgress |
| Stop | SessionEnd | SessionEnded |
| Error | UserPromptSubmit | InProgress |
| Error | PostToolUse (成功) | InProgress |
| Error | SessionEnd | SessionEnded |

## 推奨される修正

### 選択肢 1: 新しい状態 `Idle` を追加

```rust
pub enum TaskStatus {
    Idle,           // NEW: セッション開始、プロンプト待ち
    InProgress,
    Stop,
    SessionEnded,
    Error,
}
```

### 選択肢 2: ClaudeTask::new() のデフォルト値を変更

```rust
// 現在
let status = event.status.unwrap_or(TaskStatus::InProgress);

// 修正案: Stop (待機中) をデフォルトにする
let status = event.status.unwrap_or(TaskStatus::Stop);
```

### 選択肢 3: has_started() をより厳密に

現在の `has_started()` は SessionStart のみのタスクを除外しているが、
UI表示時の状態判定をさらに洗練させることも可能。

## ファイル構成

| ファイル | 役割 |
|---------|------|
| `src/commands/claude_task.rs` | TaskStatus 定義、ClaudeTask 状態管理 |
| `src/commands/ui.rs` | TUI 表示、状態に基づく色分け |
| `src/config.rs` | track-progress.py テンプレート（状態決定ロジック） |
