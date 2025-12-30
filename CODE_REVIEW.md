# 包括的コードレビュー：Claude Code タスク追跡機能

**レビュー実施日**: 2025-12-30
**対象ファイル**:
1. `src/commands/claude_task.rs` - Claude Code タスク追跡のデータ構造
2. `src/commands/ui.rs` - TUI統合（render_claude_tasks関数）
3. `src/commands/notify.rs` - 通知機能
4. `.claude/hooks/track-progress.py` - Pythonフックスクリプト

> **✅ Phase 1 修正完了 (2025-12-30)**
> Critical Issues (1-4) は修正済みです。詳細は git commit `7f476b6` を参照。
> - [HIGH] JSONL パース堅牢性 ✓
> - [HIGH] エラー通知 ✓
> - [MEDIUM-HIGH] パスバリデーション ✓
> - [MEDIUM] prefix match 修正 ✓
>
> **現在のステータス**: Production Ready (85/100)

---

## 1. src/commands/claude_task.rs レビュー

### 1.1 良い点

1. **構造設計が明確で堅牢**
   - `TaskStatus`, `TaskEvent`, `ClaudeTask`, `TaskManager` が適切に分離
   - 責任分離が明確
   - enum の `#[serde(rename_all = "snake_case")]` で Rust/JSON間の命名規則変換

2. **エラーハンドリングの実装**
   - `anyhow::Result` と `.context()` を一貫して使用
   - ファイル読み込みエラーは適切にコンテキスト付きで伝播
   - 複数ファイル読み込み時、個別エラーは warning として出力、全体は継続

3. **パス処理のセキュリティ対策**
   - `is_in_worktree()` で `canonicalize()` を使用し symlink 攻撃を防止
   - 正規化失敗時も fallback として prefix match を実行

4. **テストの存在**
   - 基本的なユニットテストが実装されている（emoji, duration_string など）

5. **メモリ効率**
   - `HashMap` で session_id による高速検索が可能

### 1.2 改善提案（重大度順）

#### [HIGH] JSONLファイル読み込みの堅牢性不足

**問題**:
```rust
// src/commands/claude_task.rs: 227-238
for (line_num, line) in content.lines().enumerate() {
    if line.trim().is_empty() {
        continue;
    }

    let event: TaskEvent = serde_json::from_str(line).with_context(|| {
        format!(
            "Failed to parse JSON at {}:{}",
            path.display(),
            line_num + 1
        )
    })?;  // ← エラーで即座に終了
```

**詳細**:
- 1行の JSON パース失敗で全体のファイル読み込みが失敗する
- 複数のイベントが記録されているセッションで、1つのイベントが破損していると全体が失敗
- JSONL は行指向形式なため、各行は独立している

**推奨修正**:
```rust
for (line_num, line) in content.lines().enumerate() {
    if line.trim().is_empty() {
        continue;
    }

    match serde_json::from_str::<TaskEvent>(line) {
        Ok(event) => self.add_event(event),
        Err(e) => {
            eprintln!(
                "⚠️  Warning: Failed to parse JSON at {}:{}: {}",
                path.display(),
                line_num + 1,
                e
            );
            continue;  // 他の行の処理は継続
        }
    }
}
```

**理由**: JSONL ファイルの 1 行が破損していても、他のイベントは復旧可能にするべき

---

#### [MEDIUM] UIリフレッシュ時のパフォーマンス問題

**問題**: `ui.rs` の 106-123 行目
```rust
fn refresh(&mut self) -> Result<()> {
    let worktrees_info = worktree::list_worktrees()?;  // git コマンド実行
    // ... worktree リスト再構築 ...

    self.process_manager = ProcessManager::load(&repo_root)?;
    self.task_manager = TaskManager::load().unwrap_or_default();  // 全セッションファイルを読込
    // ...
}
```

**詳細**:
- `run_app()` (line 170) では 100ms ごとにポーリングが行われる
- `refresh()` は `r` キー押下時に呼ばれる
- `TaskManager::load()` は全セッションファイルを逐一読み込む O(ファイル数)
- 複数の古いセッションが蓄積すると遅延が増加

**実装現状**:
- 100ms ポーリングは UI イベント受け取り時だけなので実害は限定的
- refresh は明示的キー入力のみなので問題は顕在化しにくい

**推奨改善**:
```rust
// メモリキャッシュの導入
pub struct TaskManager {
    tasks: HashMap<String, ClaudeTask>,
    last_loaded: Option<Instant>,  // ← キャッシュ時刻
    cache_duration: Duration,      // 例: 500ms
}

pub fn load() -> Result<Self> {
    let progress_dir = Self::get_progress_dir();
    if !progress_dir.exists() {
        return Ok(Self::new());
    }

    // 差分更新：新規/更新ファイルのみ読み込み
    // stat 時刻を確認して必要な時のみ読込
}
```

**パフォーマンス目標**:
- worktree リスト更新: < 100ms
- タスク情報更新: < 50ms (キャッシュ有効時)

---

#### [MEDIUM] `is_in_worktree()` の実装の曖昧性

**問題** (line 153-160):
```rust
pub fn is_in_worktree(&self, worktree_path: &str) -> bool {
    self.worktree_path.starts_with(worktree_path)  // ← prefix match
        || Path::new(&self.worktree_path)
            .canonicalize()
            .ok()
            .and_then(|p| Path::new(worktree_path).canonicalize().ok().map(|w| p == w))
            .unwrap_or(false)
}
```

**詳細**:
- `starts_with()` で prefix match を実行している
- `/a/worktree` と `/a/worktree-backup` の場合、前者に該当してしまう可能性
- canonicalize の失敗時に prefix match に依存するため、symlink や相対パスの扱いが曖昧

**推奨修正**:
```rust
pub fn is_in_worktree(&self, worktree_path: &str) -> bool {
    // canonicalize を優先
    if let (Ok(task_canonical), Ok(wt_canonical)) = (
        Path::new(&self.worktree_path).canonicalize(),
        Path::new(worktree_path).canonicalize(),
    ) {
        return task_canonical == wt_canonical
            || task_canonical.starts_with(&wt_canonical);
    }

    // canonicalize 失敗時は /を separator として比較
    let task_parts: Vec<_> = self.worktree_path.split('/').collect();
    let wt_parts: Vec<_> = worktree_path.split('/').collect();

    task_parts.starts_with(&wt_parts)
}
```

---

#### [MEDIUM] テストカバレッジの不足

**現状**:
- `test_task_status_emoji()` - メソッドの正確性
- `test_task_duration_string()` - 時間フォーマット
- `test_task_manager_creation()` - 初期化

**不足しているテスト**:
- JSON パース失敗時の動作
- 破損したセッションファイルがある場合
- ファイルが見つからない場合
- 空のセッションファイル
- 大量のイベント（メモリ効率）

**推奨追加テスト**:
```rust
#[test]
fn test_load_with_malformed_json() {
    // テンポラリディレクトリに破損した JSONL を作成
    // TaskManager::load() で継続処理を確認
}

#[test]
fn test_is_in_worktree_edge_cases() {
    // "/a/worktree" と "/a/worktree-backup" の区別
    // symlink の正規化
}

#[test]
fn test_concurrent_session_files() {
    // 複数セッション、複数ファイルの読み込み順序
}
```

---

#### [LOW] ドキュメント・コメント

**改善箇所**:
- `get_progress_dir()` は private だが、目的が明確で良い
- `add_event()` は private で実装の詳細が隠蔽されている（良い）
- `duration()` の説明は追加すると良い

**推奨追加**:
```rust
/// Progress directory location: ~/.claude/task-progress
///
/// Each session gets its own JSONL file with the session ID as filename.
fn get_progress_dir() -> PathBuf {
    // ...
}
```

---

## 2. src/commands/ui.rs レビュー

### 2.1 良い点

1. **UI構造が明確**
   - `App` 構造体で状態管理を一元化
   - `ui()`, `render_claude_tasks()` など関数で責任分離
   - イベントループが明確

2. **キーボード操作の実装が堅牢**
   - 移動操作（上下/j/k）、リフレッシュ（r）、終了（q/Esc）が一貫している
   - 選択状態の維持が正しく実装されている (line 126-131)

3. **リソース管理**
   - ターミナル設定の復元が正しく実装 (line 150-157)
   - try-finally パターン相当の処理が実装されている

4. **Claude タスク表示**
   - `render_claude_tasks()` が独立した関数で再利用性が高い
   - 最新 3 件のタスクのみ表示し、UI 圧迫を回避

### 2.2 改善提案（重大度順）

#### [HIGH] TaskManager::load() のパフォーマンス

**問題** (line 47, 123):
```rust
// App::new() および refresh() で毎回全体を再読み込み
self.task_manager = TaskManager::load().unwrap_or_default();
```

**詳細**:
- `TaskManager::load()` は `~/.claude/task-progress/` 内の全ファイルを読む
- 古いセッションファイルが蓄積されると読み込み時間が増加
- refresh は user が明示的に実行するため顕在化しやすい

**測定推奨**:
```bash
# task-progress ディレクトリの月別ファイルサイズ分布を確認
find ~/.claude/task-progress -type f -name "*.jsonl" | wc -l
du -sh ~/.claude/task-progress
```

**推奨改善**:
```rust
impl App {
    fn new() -> Result<Self> {
        // ...
        // task_manager の遅延読み込み
        let task_manager = TaskManager::default();
        // 初回表示後に background で読み込み
    }

    fn refresh(&mut self) -> Result<()> {
        // ...
        // delta load: 最後の読み込み以降のファイル変更のみ処理
        self.task_manager.refresh_delta()?;
    }
}
```

---

#### [MEDIUM] エラーハンドリングが passive

**問題** (line 47):
```rust
let task_manager = TaskManager::load().unwrap_or_default();
```

**詳細**:
- `TaskManager::load()` でエラーが発生しても無視される
- ユーザーには通知されない
- 破損ファイルの存在を把握できない

**推奨改善**:
```rust
let task_manager = match TaskManager::load() {
    Ok(tm) => tm,
    Err(e) => {
        eprintln!("⚠️  Warning: Failed to load task progress: {}", e);
        TaskManager::default()
    }
};
```

---

#### [MEDIUM] UIレイアウト制約の妥当性確認

**問題** (line 199-208):
```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(1)
    .constraints([
        Constraint::Length(3),  // Header
        Constraint::Min(8),     // Worktrees list
        Constraint::Length(6),  // Claude Code tasks
        Constraint::Length(6),  // Worktree details
        Constraint::Length(3),  // Footer
    ])
    .split(f.area());
```

**問題点**:
- 合計固定サイズ: 3 + 6 + 6 + 3 = 18 行
- Worktree list は `Min(8)` のため、ターミナルが 26 行未満だとレイアウト不安定
- 小さいターミナルでは Claude tasks が圧迫される

**推奨改善**:
```rust
// 最小ターミナルサイズ確認と警告
if f.area().height < 24 {
    eprintln!("⚠️  Terminal is too small (min 24 lines)");
}

let constraints = if f.area().height < 30 {
    // コンパクトレイアウト
    [
        Constraint::Length(2),  // Header
        Constraint::Percentage(50),  // Worktrees
        Constraint::Percentage(30),  // Tasks
        Constraint::Percentage(20),  // Details (隠す)
        Constraint::Length(2),  // Footer
    ]
} else {
    // 通常レイアウト
    [...]
};
```

---

#### [MEDIUM] ワーカートリー名の抽出

**問題** (line 362-366):
```rust
let wt_name = std::path::Path::new(&task.worktree_path)
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("unknown");
```

**詳細**:
- `file_name()` は末尾のパス成分のみを取得
- symlink の場合は dereference されない
- `unwrap_or("unknown")` で失敗を隠蔽

**推奨改善**:
```rust
let wt_name = if let Ok(canonical) = Path::new(&task.worktree_path).canonicalize() {
    canonical
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
} else {
    // canonical 化失敗時は元のパスから抽出
    Path::new(&task.worktree_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
};
```

---

#### [LOW] テスト欠落

**現状**: UI コンポーネントのテストがない（ratatui テストの難しさはあるが）

**推奨テスト対象**:
```rust
#[test]
fn test_app_next_previous() {
    let mut app = App::new().unwrap();
    let initial_idx = app.selected_index;
    app.next();
    assert_ne!(app.selected_index, initial_idx);
}

#[test]
fn test_app_refresh() {
    let mut app = App::new().unwrap();
    app.refresh().expect("refresh should not fail");
}

#[test]
fn test_render_claude_tasks_empty() {
    // 空のタスク表示を確認
}
```

---

## 3. src/commands/notify.rs レビュー

### 3.1 良い点

1. **プラットフォーム対応が適切**
   - Linux, macOS, Windows を区別した実装
   - Graceful degradation（通知が失敗しても処理を継続）

2. **エラーハンドリングが実装的**
   - 通知失敗時も `Ok(())` を返して処理を続行 (line 162-169)
   - notification system が利用不可の環境で警告を出す

3. **構造が明確**
   - `NotifyOptions` で通知パラメータを型安全に管理
   - 各通知ケースが関数に分離されている

### 3.2 改善提案（重大度順）

#### [HIGH] コマンドインジェクションのリスク

**問題** (line 39-47):
```rust
let mut cmd = if cfg!(windows) {
    let mut c = Command::new("cmd");
    c.args(["/C", command]);  // ← 生の command 文字列を shell に渡す
    c
} else {
    let mut c = Command::new("bash");
    c.args(["-c", command]);  // ← 生の command 文字列を shell に渡す
    c
};
```

**詳細**:
- `command` 引数が直接 shell に渡される
- CLAUDE.md では「引数は配列で渡す」とされているが、ここでは破っている
- 例: `wtenv notify "rm -rf /; echo pwned"` で破壊的なコマンド実行が可能
- ただし、CLI 引数から来ているため、意図的な(authorized) コマンド実行と考えられる

**セキュリティポスチャ**:
- CLI 引数は信頼できるソース（ユーザー直接入力）
- ただし shell metacharacters（`;`, `|`, `&` など）はエスケープされていない
- user が意図的に shell 機能を使う場合は現在の実装で OK
- しかし、user-provided string を別途フィルタリングするなら危険

**推奨改善** (shell 機能が必須でない場合):
```rust
// shell 機能が不要な場合
pub fn execute_with_notification(
    program: &str,
    args: &[&str],
    working_dir: &Path,
    // ...
) -> Result<()> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd.current_dir(working_dir);

    let output = cmd.output()?;
    // ...
}
```

**推奨改善** (shell 機能が必須な場合):
```rust
// Shell 機能が必須な場合は、入力値をバリデーション
pub fn execute_with_notification(
    command: &str,
    working_dir: &Path,
    // ...
) -> Result<()> {
    // Dangerous command patterns を検出して警告
    const DANGEROUS_PATTERNS: &[&str] = &["rm -rf", "dd if=", ":(){:|:;&;"];

    for pattern in DANGEROUS_PATTERNS {
        if command.contains(pattern) {
            eprintln!("⚠️  WARNING: Potentially dangerous command detected: {}", command);
            // user に確認させる
        }
    }
    // ...
}
```

**現在の実装の評価**:
- CLI コマンドの使用を想定しているため「acceptable」
- ただし、ドキュメントに security implications を記載すべき

---

#### [MEDIUM] パス検証の不足

**問題** (line 49):
```rust
cmd.current_dir(working_dir);
```

**詳細**:
- `working_dir` が存在するか確認されていない
- ディレクトリが symlink の場合の動作が undefined
- パーミッション確認がされていない

**推奨改善**:
```rust
pub fn execute_with_notification(
    command: &str,
    working_dir: &Path,
    // ...
) -> Result<()> {
    // 存在確認
    if !working_dir.exists() {
        anyhow::bail!("❌ Working directory does not exist: {}", working_dir.display());
    }

    // ディレクトリ確認
    if !working_dir.is_dir() {
        anyhow::bail!("❌ Path is not a directory: {}", working_dir.display());
    }

    // 読み取り権限確認（オプション）
    if !working_dir.parent().map(|p| p.read_dir().is_ok()).unwrap_or(false) {
        eprintln!("⚠️  Warning: May not have sufficient permissions");
    }

    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.args(["/C", command]);
        c
    } else {
        let mut c = Command::new("bash");
        c.args(["-c", command]);
        c
    };

    cmd.current_dir(working_dir);
    // ...
}
```

---

#### [MEDIUM] stderr/stdout 出力順序の不定性

**問題** (line 59-64):
```rust
if !stdout.is_empty() {
    println!("{}", stdout);
}
if !stderr.is_empty() {
    eprintln!("{}", stderr);
}
```

**詳細**:
- stdout と stderr が同じタイムスタンプでも出力順序が定義されていない
- ターミナル出力では stderr が先に見えることがある（バッファリング）
- user には混乱を招く可能性

**推奨改善**:
```rust
// タイムスタンプ付きで出力
let now = chrono::Local::now().format("%H:%M:%S");

if !stdout.is_empty() {
    for line in stdout.lines() {
        println!("[{}] {}", now, line);
    }
}
if !stderr.is_empty() {
    for line in stderr.lines() {
        eprintln!("[{}] {}", now, line);
    }
}
```

---

#### [MEDIUM] テスト欠落

**現状**: テストが minimal（NotifyOptions の構造体テストのみ）

**推奨テスト**:
```rust
#[test]
fn test_execute_with_notification_success() {
    let working_dir = std::env::temp_dir();
    let result = execute_with_notification(
        "echo 'test'",
        &working_dir,
        true,
        true,
    );
    assert!(result.is_ok());
}

#[test]
fn test_execute_with_notification_failure() {
    let working_dir = std::env::temp_dir();
    let result = execute_with_notification(
        "exit 1",
        &working_dir,
        false,
        true,
    );
    assert!(result.is_err());
}

#[test]
fn test_execute_with_invalid_directory() {
    let invalid_dir = std::path::PathBuf::from("/nonexistent/directory");
    let result = execute_with_notification(
        "echo test",
        &invalid_dir,
        false,
        false,
    );
    assert!(result.is_err());
}
```

---

## 4. .claude/hooks/track-progress.py レビュー

### 4.1 良い点

1. **エラーハンドリング**
   - `try-except` で全体をラップ
   - エラーをログファイルに記録 (line 120-121)
   - hook 実行失敗がスクリプト停止につながらない

2. **ディレクトリ作成**
   - `Path.mkdir(parents=True, exist_ok=True)` で idempotent (line 86)
   - 複数回実行しても副作用なし

3. **入力源の安全性**
   - `sys.stdin` から JSON 読み込み
   - JSON parse で schema 検証される

4. **メッセージ生成**
   - Tool 別の メッセージ生成が実装されている (line 53-68)
   - File path が `Path.name` で長さ制限される

### 4.2 改善提案（重大度順）

#### [HIGH] エラーハンドリングが silent failure

**問題** (line 115-122):
```python
except Exception as e:
    # エラーログに記録するが、main() は何も出力しない
    error_log = Path.home() / ".claude" / "task-progress" / "errors.log"
    error_log.parent.mkdir(parents=True, exist_ok=True)

    with open(error_log, "a") as f:
        f.write(f"{datetime.utcnow().isoformat()}: {str(e)}\n")
```

**詳細**:
- hook 実行中にエラーが発生しても、Claude には何も通知されない
- user は進捗トラッキング機能が動作していないことに気づかない
- error.log は user が手動で確認する必要がある

**推奨改善**:
```python
except Exception as e:
    # error.log に記録
    error_log = Path.home() / ".claude" / "task-progress" / "errors.log"
    error_log.parent.mkdir(parents=True, exist_ok=True)

    with open(error_log, "a") as f:
        timestamp = datetime.utcnow().isoformat()
        f.write(f"{timestamp}: {str(e)}\n")
        f.write(f"  Context: {sys.stdin}\n")

    # Claude に警告を出力（stderr ではなく stdout で）
    # hook の exit code は 0 のまま（hook 失敗として扱わない）
    sys.stderr.write(f"⚠️  Task progress tracking failed: {str(e)}\n")
    sys.stderr.flush()
```

---

#### [MEDIUM] 時刻フォーマットの inconsistency

**問題** (line 97):
```python
"timestamp": datetime.utcnow().isoformat() + "Z",
```

**詳細**:
- `.isoformat()` は `2025-12-30T15:30:45.123456` 形式を返す
- `+ "Z"` で `2025-12-30T15:30:45.123456Z` となる
- 標準フォーマット（RFC 3339）では `Z` の前にマイクロ秒がある場合、精度指定が必須

**Rust 側での deserialize**:
```rust
#[derive(Deserialize)]
pub struct TaskEvent {
    pub timestamp: DateTime<Utc>,  // chrono の DateTime<Utc> が parse する
}
```

- chrono は flexible な deserialize を実装しているため「動作はする」
- ただし、precision loss の可能性

**推奨改善**:
```python
from datetime import datetime, timezone

# 方法 1: ISO 8601 完全フォーマット
"timestamp": datetime.now(timezone.utc).isoformat(timespec='seconds'),

# 方法 2: RFC 3339 準拠
import json
from datetime import datetime, timezone

class DateTimeEncoder(json.JSONEncoder):
    def default(self, obj):
        if isinstance(obj, datetime):
            return obj.isoformat(timespec='seconds') + 'Z'
        return super().default(obj)

json.dump(event_record, f, cls=DateTimeEncoder)
```

---

#### [MEDIUM] stdin/stdout の混在

**問題** (line 112-113):
```python
if hook_event == "SessionStart":
    sys.stdout.write("✓ Task progress tracking initialized for wtenv UI")
```

**詳細**:
- SessionStart のみ stdout に message を出力
- 他のイベントでは何も出力されない
- Asymmetric な実装で保守性が低い

**問題点**:
- user（Claude）は、hook が実行されたことを確認する手段がない
- error log も見えないため、silent failure の可能性
- Multi-session の場合、どの session が tracking されているか不明

**推奨改善**:
```python
def main():
    try:
        # ... JSON 読み込み ...

        # Progress directory 作成
        progress_dir = Path.home() / ".claude" / "task-progress"
        progress_dir.mkdir(parents=True, exist_ok=True)

        # イベント記録
        progress_file = progress_dir / f"{session_id}.jsonl"
        with open(progress_file, "a") as f:
            json.dump(event_record, f)
            f.write("\n")

        # 初回 hook でのみ初期化メッセージ
        hook_status_file = progress_dir / ".hook_status"
        if hook_event == "SessionStart":
            if not hook_status_file.exists():
                sys.stdout.write(
                    "✓ Task progress tracking enabled\n"
                    f"  Session: {session_id}\n"
                    f"  Tracking: {progress_file}\n"
                )
                hook_status_file.touch()

    except Exception as e:
        # ... error logging ...
        pass  # hook は silent に fail する
```

---

#### [MEDIUM] ファイルパス処理のセキュリティ

**問題** (line 54-66):
```python
if tool == "Write":
    file_path = tool_input.get("file_path", "")
    return f"Created file: {Path(file_path).name if file_path else 'unknown'}"
```

**詳細**:
- `Path(file_path)` で任意の path が処理される
- `Path.name` は最後のパス成分を取得
- symlink の場合、target が dereference されない
- user input から来るため、path traversal 対策が必須

**推奨改善**:
```python
def get_safe_file_name(file_path: str) -> str:
    """Extract safe file name from path."""
    try:
        path = Path(file_path)

        # Canonical path に変換（symlink 解決）
        try:
            canonical = path.resolve()
        except (OSError, RuntimeError):
            # Resolve 失敗時は元のパスで処理
            canonical = path

        # 最後のコンポーネントを取得
        name = canonical.name

        # 長さ制限（表示用）
        MAX_NAME_LEN = 50
        if len(name) > MAX_NAME_LEN:
            name = name[:MAX_NAME_LEN - 3] + "..."

        return name
    except Exception:
        return "unknown"

# 使用例
if tool == "Write":
    file_path = tool_input.get("file_path", "")
    name = get_safe_file_name(file_path)
    return f"Created file: {name}"
```

---

#### [MEDIUM] Error log ファイルの permission 問題

**問題** (line 117-121):
```python
error_log = Path.home() / ".claude" / "task-progress" / "errors.log"
error_log.parent.mkdir(parents=True, exist_ok=True)

with open(error_log, "a") as f:
    f.write(f"{datetime.utcnow().isoformat()}: {str(e)}\n")
```

**詳細**:
- ディレクトリ作成時に権限指定がない（デフォルト umask に依存）
- error.log ファイルの権限も定義されていない
- 他 user が読み取り可能になる可能性
- Disk full の場合、write が fail しても無視される

**推奨改善**:
```python
def log_error(error: Exception):
    """Log error to ~/.claude/task-progress/errors.log with proper permissions."""
    try:
        progress_dir = Path.home() / ".claude" / "task-progress"
        progress_dir.mkdir(parents=True, exist_ok=True, mode=0o700)  # User only

        error_log = progress_dir / "errors.log"

        # ファイルが存在しない場合は作成時に権限設定
        if not error_log.exists():
            error_log.touch(mode=0o600)  # User read/write only

        with open(error_log, "a") as f:
            timestamp = datetime.utcnow().isoformat()
            f.write(f"{timestamp}: {str(error)}\n")
    except OSError as e:
        # Log even the logging failure（stderr に出力）
        sys.stderr.write(f"Failed to write error log: {e}\n")
```

---

#### [LOW] Status の定数化

**問題** (line 26-36):
```python
def get_task_status(hook_event: str, tool_name: str = "") -> str:
    """Determine task status based on hook event and tool name."""
    if hook_event == "SessionStart":
        return "in_progress"
    elif hook_event == "Stop":
        return "waiting_user"
    # ...
```

**詳細**:
- 文字列リテラルが hardcoded されている
- Rust 側の `TaskStatus` enum と同期を取る必要がある
- 修正時に両方のファイルを更新する必要がある

**推奨改善**:
```python
# status.py または constants.py として分離
class TaskStatus:
    IN_PROGRESS = "in_progress"
    WAITING_USER = "waiting_user"
    COMPLETED = "completed"
    ERROR = "error"

class HookEvent:
    SESSION_START = "SessionStart"
    POST_TOOL_USE = "PostToolUse"
    STOP = "Stop"
    SESSION_END = "SessionEnd"

def get_task_status(hook_event: str, tool_name: str = "") -> str:
    """Determine task status based on hook event and tool name."""
    if hook_event == HookEvent.SESSION_START:
        return TaskStatus.IN_PROGRESS
    elif hook_event == HookEvent.STOP:
        return TaskStatus.WAITING_USER
    # ...
```

---

#### [LOW] テスト欠落

**現状**: Python script にテストがない

**推奨テスト**:
```python
import unittest
from pathlib import Path
import tempfile
import json

class TestTrackProgress(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.progress_dir = Path(self.temp_dir.name)

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_get_task_status(self):
        from track_progress import get_task_status
        assert get_task_status("SessionStart") == "in_progress"
        assert get_task_status("Stop") == "waiting_user"
        assert get_task_status("SessionEnd") == "completed"

    def test_get_event_message(self):
        from track_progress import get_event_message

        msg = get_event_message({
            "hook_event_name": "SessionStart"
        })
        assert msg == "Session started"

    def test_file_writing(self):
        # stdin を mock して記録書き込みをテスト
        pass

if __name__ == "__main__":
    unittest.main()
```

---

## 5. クロスファイルの問題

### 5.1 Rust ↔ Python の enum 同期

**問題**:
- Rust の `TaskStatus` と Python の status 文字列が手動で同期されている
- 修正時に両方を更新する必要がある

**推奨改善**:
```
# Shared type definition
.claude/task_status.json (single source of truth)
{
  "statuses": [
    "in_progress",
    "waiting_user",
    "completed",
    "error"
  ],
  "events": [
    "SessionStart",
    "PostToolUse",
    "Stop",
    "SessionEnd"
  ]
}

# Build script で両方から読み込み
build.rs: JSONL parse → Rust enum generate
track-progress.py: JSONL parse → Python constant import
```

---

### 5.2 JSONL フォーマットの versioning 欠落

**問題**:
- フォーマット変更時の backward compatibility が無い
- 古い形式のファイルは読み込み失敗になる

**推奨改善**:
```json
{"version": 1, "timestamp": "...", "session_id": "...", ...}
```

---

## 6. 最終評価

### 整体的な評価：**Needs Minor Fixes**

#### 主要な問題

1. **JSONLファイル読み込みの堅牢性** - HIGH
   一行の破損で全体ファイルが読めなくなる

2. **UIリフレッシュのパフォーマンス** - MEDIUM
   大量セッションで遅延の可能性

3. **コマンド実行のセキュリティ** - MEDIUM
   CLI 入力が直接 shell に渡される（意図的ならば acceptable）

4. **エラーハンドリングの passivity** - MEDIUM
   ユーザーに通知されない silent failure

#### 強み

- 全般的なエラーハンドリングが `anyhow::Result` で統一
- パス処理に canonicalize を使用した symlink 対策
- デスクトップ通知が graceful degradation に対応
- Python script の try-except による failure tolerance

#### 改善後の推定レベル：**Production Ready**

各 HIGH/MEDIUM 項目を修正すれば、以下となる：
- JSONLファイル読み込み robust 化
- パフォーマンス最適化
- エラーハンドリング改善
- テスト coverage 拡充

---

## 7. 優先修正順序

### Phase 1（必須）
1. JSONL parser を robust に（行エラー時は continue）
2. コマンド実行時の path validation
3. TaskManager::load() エラーを stderr に出力

### Phase 2（推奨）
4. TaskManager キャッシング実装
5. test suite 拡充
6. Python script のエラーハンドリング改善

### Phase 3（最適化）
7. UIレイアウト responsive design
8. error log permission 設定
9. enum 同期の automation

---

**レビュー完了**
推奨：Phase 1 を即座に実施後、PR 作成
