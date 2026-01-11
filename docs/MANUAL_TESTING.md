# ccmon 動作確認手順書

リリース前の手動動作確認チェックリスト。

## 前提条件

### 環境準備

```bash
# 1. ビルド
cargo build --release

# 2. パスを通す（またはエイリアス設定）
alias ccmon="./target/release/ccmon"

# 3. テスト用gitリポジトリを準備
cd /tmp
mkdir ccmon-test-repo && cd ccmon-test-repo
git init
git commit --allow-empty -m "Initial commit"
```

### 確認環境

- [ ] Linux

---

## 1. 基本コマンド

### 1.1 ヘルプ表示

```bash
ccmon --help
ccmon -h
ccmon init --help
ccmon ui --help
```

**期待結果:**
- [ ] サブコマンド一覧が表示される（init, ui）
- [ ] 各オプションの説明が表示される
- [ ] "Claude Code Monitor" が表示される

### 1.2 バージョン表示

```bash
ccmon --version
ccmon -V
```

**期待結果:**
- [ ] `ccmon 0.1.0` が表示される

---

## 2. init コマンド

### 2.1 基本的な初期化

```bash
cd /tmp/ccmon-test-repo
ccmon init
```

**期待結果:**
- [ ] `.claude/settings.json` が作成される
- [ ] `.claude/hooks/session-init.sh` が作成される
- [ ] `.claude/hooks/track-progress.py` が作成される
- [ ] `~/.claude/stop-hook-git-check.sh` が作成される
- [ ] 成功メッセージと次のステップ案内が表示される

### 2.2 作成されたファイルの確認

```bash
cat .claude/settings.json
cat .claude/hooks/session-init.sh
cat .claude/hooks/track-progress.py
cat ~/.claude/stop-hook-git-check.sh
```

**期待結果:**
- [ ] settings.json に SessionStart, PostToolUse, Stop, SessionEnd, Notification, UserPromptSubmit の hooks が定義されている
- [ ] session-init.sh が実行可能（755）
- [ ] track-progress.py が実行可能（755）
- [ ] stop-hook-git-check.sh が実行可能（755）

### 2.3 既存ファイルがある場合

```bash
ccmon init  # 2回目
```

**期待結果:**
- [ ] エラーメッセージが表示される
- [ ] `--force` オプションの案内が表示される

### 2.4 強制上書き

```bash
ccmon init -f
ccmon init --force
```

**期待結果:**
- [ ] 確認なしで上書きされる
- [ ] 成功メッセージが表示される

---

## 3. ui コマンド

### 3.1 TUI起動

```bash
ccmon ui
```

**期待結果:**
- [ ] インタラクティブTUIが起動する
- [ ] "ccmon - Claude Code Monitor" ヘッダーが表示される
- [ ] Claude Code Tasks セクションが表示される
- [ ] Task Details セクションが表示される
- [ ] フッターにアクティブ/合計タスク数が表示される

### 3.2 キーバインド確認

TUI内で以下を確認:

**期待結果:**
- [ ] `j` / `↓` で次のタスクへ移動
- [ ] `k` / `↑` で前のタスクへ移動
- [ ] `r` で手動更新
- [ ] `q` または `Esc` で終了

### 3.3 タスクがない場合

```bash
# タスクファイルを削除してから
rm -rf ~/.claude/task-progress/*.jsonl
ccmon ui
```

**期待結果:**
- [ ] "No Claude Code tasks found" メッセージが表示される
- [ ] "ccmon init" の案内が表示される

### 3.4 自動更新確認

```bash
# 別ターミナルでタスクファイルを作成しながら ui を確認
ccmon ui
# 別ターミナルで: echo '{"timestamp":"2024-01-01T00:00:00Z","session_id":"test","event":"SessionStart","cwd":"/tmp/test"}' > ~/.claude/task-progress/test-session.jsonl
```

**期待結果:**
- [ ] 1秒後に新しいタスクが表示される

---

## 4. グローバルオプション

### 4.1 詳細モード

```bash
ccmon -v init
ccmon --verbose init --force
```

**期待結果:**
- [ ] 詳細情報が追加表示される

### 4.2 サイレントモード

```bash
ccmon -q init --force
ccmon --quiet ui
```

**期待結果:**
- [ ] 成功時の出力が抑制される
- [ ] エラーは表示される

---

## 5. エラーハンドリング

### 5.1 無効なコマンド

```bash
ccmon invalid-command
```

**期待結果:**
- [ ] 適切なエラーメッセージが表示される
- [ ] 有効なコマンド一覧が表示される

---

## 6. hooks 動作確認

### 6.1 session-init.sh

```bash
cd /tmp/ccmon-test-repo
./.claude/hooks/session-init.sh
```

**期待結果:**
- [ ] "Development Context" が表示される
- [ ] ブランチ情報が表示される
- [ ] 最近のコミットが表示される

### 6.2 track-progress.py

```bash
echo '{"session_id":"test","hook_event_name":"SessionStart","cwd":"/tmp"}' | ./.claude/hooks/track-progress.py
cat ~/.claude/task-progress/test.jsonl
```

**期待結果:**
- [ ] JSONL ファイルにイベントが記録される
- [ ] タイムスタンプが含まれる

### 6.3 stop-hook-git-check.sh

```bash
# クリーンな状態で
echo '{}' | ~/.claude/stop-hook-git-check.sh
echo $?

# 変更がある状態で
echo "test" > uncommitted.txt
echo '{}' | ~/.claude/stop-hook-git-check.sh
echo $?
rm uncommitted.txt
```

**期待結果:**
- [ ] クリーン時: 終了コード 0
- [ ] 変更あり時: 終了コード 2 とメッセージ

---

## 7. パフォーマンス確認

### 7.1 起動時間

```bash
time ccmon --help
```

**期待結果:**
- [ ] 50ms未満

### 7.2 バイナリサイズ

```bash
ls -lh target/release/ccmon
```

**期待結果:**
- [ ] 5MB未満

---

## 8. クリーンアップ

```bash
# テスト用データを削除
cd /tmp
rm -rf ccmon-test-repo
rm -f ~/.claude/task-progress/test*.jsonl
```

---

## 確認完了チェックリスト

| カテゴリ | 項目数 | 完了 |
|---------|--------|------|
| 基本コマンド | 2 | [ ] |
| init | 4 | [ ] |
| ui | 4 | [ ] |
| グローバルオプション | 2 | [ ] |
| エラーハンドリング | 1 | [ ] |
| hooks 動作 | 3 | [ ] |
| パフォーマンス | 2 | [ ] |

**確認者:** _______________
**確認日:** _______________
**バージョン:** _______________
