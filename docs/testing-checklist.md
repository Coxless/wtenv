# wtenv 動作確認チェックリスト

このドキュメントは、wtenvの新機能開発やリリース前に実施すべき標準的な動作確認項目をまとめたチェックリストです。

## 📋 目次

1. [事前準備](#事前準備)
2. [ビルド確認](#ビルド確認)
3. [コア機能の動作確認](#コア機能の動作確認)
4. [Claude Code 連携機能](#claude-code-連携機能)
5. [エラーハンドリング](#エラーハンドリング)
6. [パフォーマンス確認](#パフォーマンス確認)
7. [コード品質確認](#コード品質確認)

---

## 事前準備

### 環境確認
- [ ] Rust 1.91.0 以上がインストールされている
- [ ] Git 2.x がインストールされている
- [ ] テスト用のGitリポジトリを準備できる

### テスト環境セットアップ
```bash
# テスト用ディレクトリ作成
mkdir -p /tmp/wtenv-test
cd /tmp/wtenv-test

# Gitリポジトリ初期化
git init
git config user.name "Test User"
git config user.email "test@example.com"

# 初期コミット作成
echo "# Test Repository" > README.md
git add README.md
git commit -m "Initial commit"
```

---

## ビルド確認

### リリースビルド
- [ ] `cargo build --release` が成功する
- [ ] ビルド時に警告が出ない
- [ ] バイナリサイズが 5MB 以下（strip後）

```bash
cargo build --release
ls -lh target/release/wtenv
```

### 基本実行確認
- [ ] `wtenv --version` でバージョンが表示される
- [ ] `wtenv --help` でヘルプが表示される
- [ ] 起動時間が 50ms 以下

```bash
./target/release/wtenv --version
./target/release/wtenv --help

# 起動時間計測（Linuxの場合）
time ./target/release/wtenv --version
```

---

## コア機能の動作確認

### 1. init コマンド

#### 基本的な初期化
- [ ] `wtenv init` で `.worktree.yml` が作成される
- [ ] デフォルト設定が正しく生成される
- [ ] 既存ファイルがある場合、上書き警告が出る
- [ ] `--force` フラグで上書きできる

```bash
cd /tmp/wtenv-test
wtenv init
cat .worktree.yml

# 上書き確認
wtenv init          # 警告が出るべき
wtenv init --force  # 上書きされる
```

#### Claude Code hooks 生成（`--hooks`）
- [ ] `wtenv init --hooks` で `.claude/settings.json` が作成される
- [ ] `.claude/hooks/session-init.sh` が実行可能権限付きで作成される
- [ ] `.claude/hooks/track-progress.py` が実行可能権限付きで作成される
- [ ] `~/.claude/stop-hook-git-check.sh` が実行可能権限付きで作成される
- [ ] 全てのhookファイルがsettings.jsonに登録される

```bash
wtenv init --hooks

# ファイル確認
ls -la .claude/
ls -la .claude/hooks/
ls -la ~/.claude/

# 権限確認
test -x .claude/hooks/session-init.sh && echo "OK: executable"
test -x .claude/hooks/track-progress.py && echo "OK: executable"
test -x ~/.claude/stop-hook-git-check.sh && echo "OK: executable"

# 内容確認
cat .claude/settings.json
cat .claude/hooks/session-init.sh
cat .claude/hooks/track-progress.py
cat ~/.claude/stop-hook-git-check.sh
```

### 2. config コマンド
- [ ] `wtenv config` で現在の設定が表示される
- [ ] YAML形式で正しくフォーマットされている
- [ ] 設定ファイルが無い場合、デフォルト設定が表示される

```bash
wtenv config
```

### 3. create コマンド

#### 基本的なworktree作成
- [ ] `wtenv create <branch> <path>` でworktreeが作成される
- [ ] ブランチが自動作成される
- [ ] 設定ファイルのコピールールが適用される
- [ ] post-createコマンドが実行される

```bash
# テスト用の環境ファイル作成
echo "TEST_VAR=main" > .env
echo "version: 1
copy:
  - .env
postCreate:
  - command: echo 'Post-create executed'
    description: 'Test command'
" > .worktree.yml

# worktree作成
wtenv create feature/test ../wt-test

# 確認
cd ../wt-test
test -f .env && echo "OK: .env copied"
cat .env
```

#### オプション動作確認
- [ ] `--no-copy` でコピーがスキップされる
- [ ] `--no-post-create` でpost-createがスキップされる
- [ ] `-c <config>` で設定ファイルを指定できる

```bash
cd /tmp/wtenv-test

# コピーなし
wtenv create feature/no-copy ../wt-no-copy --no-copy
cd ../wt-no-copy
test ! -f .env && echo "OK: no copy"

# post-createなし
cd /tmp/wtenv-test
wtenv create feature/no-post ../wt-no-post --no-post-create
```

### 4. list コマンド
- [ ] `wtenv list` でworktree一覧が表示される
- [ ] mainとworktreeが区別される
- [ ] ブランチ名とパスが正しく表示される

```bash
cd /tmp/wtenv-test
wtenv list
```

### 5. status コマンド
- [ ] `wtenv status` で詳細情報が表示される
- [ ] 変更ファイル数が表示される
- [ ] コミット状態が表示される
- [ ] 最終更新日時が表示される

```bash
cd /tmp/wtenv-test
wtenv status

# worktreeでも確認
cd ../wt-test
wtenv status
```

### 6. remove コマンド
- [ ] `wtenv remove <path>` でworktreeが削除される
- [ ] 確認プロンプトが表示される
- [ ] `-f, --force` で確認なしで削除される
- [ ] 存在しないパスでエラーメッセージが出る

```bash
cd /tmp/wtenv-test

# 通常削除（確認あり）
wtenv remove ../wt-no-copy

# 強制削除
wtenv remove ../wt-no-post -f

# 存在しないパス
wtenv remove /nonexistent/path  # エラーになるべき
```

---

## Claude Code 連携機能

### 1. 生成されたhooksファイルの検証

#### settings.json の構文確認
- [ ] JSONとして正しくパースできる
- [ ] 必須フィールドが含まれている
- [ ] 全てのhookイベントが設定されている

```bash
# JSON構文チェック
python3 -m json.tool .claude/settings.json > /dev/null && echo "OK: valid JSON"

# 内容確認
cat .claude/settings.json | python3 -c "
import sys, json
data = json.load(sys.stdin)
hooks = data.get('hooks', {})
events = ['SessionStart', 'PostToolUse', 'Stop', 'SessionEnd', 'Notification']
for event in events:
    if event in hooks:
        print(f'✓ {event}')
    else:
        print(f'✗ {event} missing')
"
```

#### Bashスクリプトの構文確認
- [ ] session-init.sh が構文エラーなし
- [ ] stop-hook-git-check.sh が構文エラーなし
- [ ] 実行可能権限が付与されている

```bash
# 構文チェック
bash -n .claude/hooks/session-init.sh && echo "OK: session-init.sh"
bash -n ~/.claude/stop-hook-git-check.sh && echo "OK: stop-hook-git-check.sh"

# 実行テスト
./.claude/hooks/session-init.sh
```

#### Pythonスクリプトの構文確認
- [ ] track-progress.py が構文エラーなし
- [ ] 必要なモジュールがインポートできる

```bash
# 構文チェック
python3 -m py_compile .claude/hooks/track-progress.py && echo "OK: track-progress.py"

# インポートテスト
python3 -c "import sys; sys.path.insert(0, '.claude/hooks'); import track_progress" 2>/dev/null || \
python3 .claude/hooks/track-progress.py --help 2>&1 | head -1
```

### 2. Task Progress Tracking

#### データディレクトリの確認
- [ ] `~/.claude/task-progress/` ディレクトリが作成可能
- [ ] セッションIDごとにJSONLファイルが作成される

```bash
# ディレクトリ確認
ls -la ~/.claude/task-progress/ 2>/dev/null || echo "Directory will be created on first use"
```

#### UI表示の確認
- [ ] `wtenv ui` コマンドが起動する
- [ ] TUIが正しく表示される
- [ ] キーボード操作が機能する（q で終了など）

```bash
# UI起動テスト（すぐ終了）
echo "q" | wtenv ui 2>/dev/null || wtenv ui &
sleep 1
killall wtenv 2>/dev/null
```

---

## エラーハンドリング

### 1. 不正な入力への対応
- [ ] 不正なブランチ名でエラーメッセージが出る
- [ ] 不正なパスでエラーメッセージが出る
- [ ] 存在しないworktreeの削除で適切なエラーが出る

```bash
# 不正なブランチ名
wtenv create "invalid@branch" /tmp/test 2>&1 | grep -i error

# 不正なパス
wtenv create test /invalid/readonly/path 2>&1 | grep -i error

# 存在しないworktree
wtenv remove /nonexistent/worktree 2>&1 | grep -i error
```

### 2. Git操作のエラーハンドリング
- [ ] Gitリポジトリ外での実行で適切なエラーが出る
- [ ] 既存ブランチ名での作成で適切なエラーが出る

```bash
# リポジトリ外
cd /tmp
wtenv list 2>&1 | grep -i "not.*git.*repository" || echo "Should show git error"

# 既存ブランチ
cd /tmp/wtenv-test
git branch existing-branch
wtenv create existing-branch ../wt-existing 2>&1 | grep -i "already exists"
```

### 3. ファイル操作のエラーハンドリング
- [ ] 読み取り専用ディレクトリへの書き込みで適切なエラーが出る
- [ ] 権限のないファイルのコピーで警告が出る（処理は継続）

```bash
# 読み取り専用ディレクトリ
mkdir -p /tmp/readonly
chmod 555 /tmp/readonly
wtenv create test /tmp/readonly/wt 2>&1 | grep -i "permission denied"
chmod 755 /tmp/readonly
```

### 4. 設定ファイルのエラーハンドリング
- [ ] 不正なYAMLで適切なエラーが出る
- [ ] 不正なバージョンで適切なエラーが出る

```bash
# 不正なYAML
echo "invalid: yaml: syntax:" > .worktree.yml
wtenv config 2>&1 | grep -i "error\|parse"

# 不正なバージョン
echo "version: 999" > .worktree.yml
wtenv config 2>&1 | grep -i "version"

# 修復
wtenv init --force
```

---

## パフォーマンス確認

### 1. 起動時間
- [ ] `wtenv --version` が 50ms 以下で完了する

```bash
# 10回計測して平均を取る
for i in {1..10}; do
    time -p ./target/release/wtenv --version 2>&1 | grep real
done
```

### 2. メモリ使用量
- [ ] 通常操作で 10MB 以下のメモリ使用

```bash
# メモリ使用量確認（Linuxの場合）
/usr/bin/time -v ./target/release/wtenv list 2>&1 | grep "Maximum resident set size"
```

### 3. バイナリサイズ
- [ ] リリースビルド（strip後）が 5MB 以下

```bash
ls -lh target/release/wtenv
strip target/release/wtenv
ls -lh target/release/wtenv
```

### 4. 大量ファイル処理
- [ ] 100個のファイルコピーが 500ms 以下で完了する

```bash
# テストファイル生成
mkdir -p /tmp/wtenv-test-files
for i in {1..100}; do
    echo "test$i" > /tmp/wtenv-test-files/.env.$i
done

# 設定ファイル更新
echo "version: 1
copy:
  - /tmp/wtenv-test-files/.env.*
" > .worktree.yml

# 計測
time wtenv create perf-test ../wt-perf
wtenv remove ../wt-perf -f
```

---

## コード品質確認

### 1. Linter
- [ ] `cargo clippy` でエラーが0件
- [ ] `cargo clippy` で警告が0件（または許容範囲内）

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### 2. フォーマット
- [ ] `cargo fmt --check` でフォーマット済み

```bash
cargo fmt --check
```

### 3. テスト
- [ ] `cargo test` で全テストが成功
- [ ] テストカバレッジが十分

```bash
cargo test
cargo test -- --nocapture  # 詳細出力
```

### 4. ドキュメント
- [ ] `cargo doc` でドキュメントが生成される
- [ ] 全てのpublic関数にドキュメントコメントがある

```bash
cargo doc --no-deps --open
```

---

## 完了確認

### 全体チェック
- [ ] 全ての必須テスト項目をパスした
- [ ] 重大なバグが見つかっていない
- [ ] パフォーマンス目標を達成している
- [ ] ドキュメントが最新である

### リリース準備（該当する場合）
- [ ] CHANGELOGを更新
- [ ] バージョン番号を更新
- [ ] リリースノートを作成
- [ ] タグを作成

---

## 参考情報

### 関連ドキュメント
- [CLAUDE.md](../CLAUDE.md) - プロジェクト全体の仕様
- [README.md](../README.md) - ユーザー向けドキュメント

### トラブルシューティング
問題が発生した場合は、以下を確認してください：
1. Rustのバージョンが 1.91.0 以上
2. Gitのバージョンが 2.x 以上
3. テスト環境が正しくセットアップされている
4. ビルドキャッシュをクリア (`cargo clean`)

### フィードバック
このチェックリストに追加すべき項目や改善点があれば、Issue または PR で提案してください。
