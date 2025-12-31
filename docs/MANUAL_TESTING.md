# wtenv 動作確認手順書

リリース前の手動動作確認チェックリスト。

## 前提条件

### 環境準備

```bash
# 1. ビルド
cargo build --release

# 2. パスを通す（またはエイリアス設定）
alias wtenv="./target/release/wtenv"

# 3. テスト用gitリポジトリを準備
cd /tmp
mkdir wtenv-test-repo && cd wtenv-test-repo
git init
git commit --allow-empty -m "Initial commit"

# 4. テスト用ブランチを作成
git branch feature-a
git branch feature-b
git branch feature-c
```

### 確認環境

- [ ] Linux

---

## 1. 基本コマンド

### 1.1 ヘルプ表示

```bash
wtenv --help
wtenv -h
wtenv create --help
```

**期待結果:**
- [ ] サブコマンド一覧が表示される
- [ ] 各オプションの説明が表示される
- [ ] 日本語/英語の説明が正しく表示される

### 1.2 バージョン表示

```bash
wtenv --version
wtenv -V
```

**期待結果:**
- [ ] `wtenv 0.1.0` が表示される

---

## 2. init コマンド

### 2.1 基本的な初期化

```bash
cd /tmp/wtenv-test-repo
wtenv init
```

**期待結果:**
- [ ] `.worktree.yml` が作成される
- [ ] 成功メッセージが表示される
- [ ] 作成されたファイルが有効なYAML

### 2.2 既存ファイルがある場合

```bash
wtenv init  # 2回目
```

**期待結果:**
- [ ] 上書き確認ダイアログが表示される
- [ ] `n` で中止、`y` で上書き

### 2.3 強制上書き

```bash
wtenv init -f
wtenv init --force
```

**期待結果:**
- [ ] 確認なしで上書きされる

### 2.4 hooks付き初期化

```bash
rm -rf .worktree.yml .claude
wtenv init --hooks
```

**期待結果:**
- [ ] `.worktree.yml` が作成される
- [ ] `.claude/settings.json` が作成される
- [ ] `.claude/hooks/session-init.sh` が作成される
- [ ] `.claude/hooks/track-progress.py` が作成される
- [ ] `~/.claude/stop-hook-git-check.sh` が作成される
- [ ] 次のステップの案内が表示される

---

## 3. config コマンド

### 3.1 設定ファイル表示

```bash
wtenv config
```

**期待結果:**
- [ ] 設定ファイルのパスが表示される
- [ ] 設定内容が表示される
- [ ] バリデーション結果が表示される

### 3.2 詳細モード

```bash
wtenv config -v
```

**期待結果:**
- [ ] バージョン、コピー対象数、除外対象数、post-createコマンド数が表示される

### 3.3 設定ファイルがない場合

```bash
cd /tmp
mkdir no-config-test && cd no-config-test
git init
wtenv config
```

**期待結果:**
- [ ] 「設定ファイルが見つかりませんでした」と表示される
- [ ] `wtenv init` の案内が表示される

---

## 4. create コマンド

### 4.1 基本的なworktree作成

```bash
cd /tmp/wtenv-test-repo
wtenv create feature-a
```

**期待結果:**
- [ ] worktreeが作成される
- [ ] 成功メッセージと `cd` コマンドが表示される
- [ ] `../feature-a` ディレクトリが存在する

### 4.2 パス指定

```bash
wtenv create feature-b /tmp/custom-path
```

**期待結果:**
- [ ] 指定パスにworktreeが作成される

### 4.3 対話モード

```bash
wtenv create
```

**期待結果:**
- [ ] ブランチ名入力プロンプトが表示される
- [ ] パス確認プロンプトが表示される
- [ ] 入力に基づいてworktreeが作成される

### 4.4 ファイルコピー確認

```bash
# テスト用.envを作成
cd /tmp/wtenv-test-repo
echo "TEST_VAR=main" > .env

# 設定ファイルを更新
cat > .worktree.yml << 'EOF'
version: 1
copy:
  - .env
EOF

# 新しいworktree作成
git branch feature-copy-test
wtenv create feature-copy-test
```

**期待結果:**
- [ ] 「環境ファイルをコピー中...」が表示される
- [ ] コピー完了メッセージが表示される
- [ ] `../feature-copy-test/.env` が存在し内容が同じ

### 4.5 コピースキップ

```bash
git branch feature-no-copy
wtenv create feature-no-copy --no-copy
```

**期待結果:**
- [ ] `.env` がコピーされない

### 4.6 post-createコマンド確認

```bash
cat > .worktree.yml << 'EOF'
version: 1
postCreate:
  - command: echo "Hello from post-create"
    description: "テストメッセージ"
EOF

git branch feature-post-create
wtenv create feature-post-create
```

**期待結果:**
- [ ] post-createコマンドが実行される
- [ ] 「Hello from post-create」が出力される

### 4.7 post-createスキップ

```bash
git branch feature-no-post
wtenv create feature-no-post --no-post-create
```

**期待結果:**
- [ ] post-createコマンドが実行されない

### 4.8 設定ファイル指定

```bash
echo "version: 1" > /tmp/custom-config.yml
git branch feature-custom-config
wtenv create feature-custom-config -c /tmp/custom-config.yml
```

**期待結果:**
- [ ] 指定した設定ファイルが使用される

---

## 5. list コマンド

### 5.1 worktree一覧

```bash
wtenv list
```

**期待結果:**
- [ ] 全worktreeが一覧表示される
- [ ] パス、ブランチ名、コミットハッシュが表示される
- [ ] メインworktreeに `(main)` マークが付く

### 5.2 詳細モード

```bash
wtenv list -v
```

**期待結果:**
- [ ] より詳細な情報が表示される

### 5.3 worktreeがない場合

```bash
cd /tmp
mkdir empty-repo && cd empty-repo
git init
wtenv list
```

**期待結果:**
- [ ] 「worktreeが見つかりませんでした」または空の一覧

---

## 6. status コマンド

### 6.1 基本的な状態表示

```bash
cd /tmp/wtenv-test-repo
wtenv status
```

**期待結果:**
- [ ] 各worktreeの状態が表示される
- [ ] 変更ファイル数が表示される
- [ ] ブランチ情報が表示される

### 6.2 詳細モード

```bash
wtenv status -v
```

**期待結果:**
- [ ] より詳細な情報が表示される

---

## 7. remove コマンド

### 7.1 削除確認

```bash
# テスト用worktree作成
git branch remove-test
wtenv create remove-test

# 削除
wtenv remove ../remove-test
```

**期待結果:**
- [ ] 削除確認ダイアログが表示される
- [ ] `n` で中止、`y` で削除

### 7.2 強制削除

```bash
git branch remove-force-test
wtenv create remove-force-test
wtenv remove ../remove-force-test -f
```

**期待結果:**
- [ ] 確認なしで削除される

### 7.3 存在しないパス

```bash
wtenv remove /nonexistent/path
```

**期待結果:**
- [ ] 適切なエラーメッセージが表示される

---

## 8. ps / kill コマンド

### 8.1 プロセス一覧

```bash
wtenv ps
```

**期待結果:**
- [ ] worktree内で実行中のプロセス一覧が表示される
- [ ] プロセスがない場合は適切なメッセージ

### 8.2 フィルタ付き

```bash
wtenv ps feature-a
```

**期待結果:**
- [ ] 指定worktreeのプロセスのみ表示される

### 8.3 プロセス停止

```bash
# バックグラウンドプロセスを起動してテスト
wtenv kill <PID>
```

**期待結果:**
- [ ] 指定プロセスが停止される

### 8.4 全プロセス停止

```bash
wtenv kill --all
```

**期待結果:**
- [ ] 全プロセスが停止される

---

## 9. diff-env コマンド

### 9.1 2つのworktree比較

```bash
# 異なる.envを作成
cd /tmp/wtenv-test-repo
echo "VAR1=main" > .env

cd ../feature-a
echo "VAR1=feature" > .env
echo "VAR2=only-feature" >> .env

cd /tmp/wtenv-test-repo
wtenv diff-env . ../feature-a
```

**期待結果:**
- [ ] 差分が表示される
- [ ] 追加/削除/変更が区別される

### 9.2 全worktree比較

```bash
wtenv diff-env --all
```

**期待結果:**
- [ ] 全worktree間の環境変数差分が表示される

---

## 10. analyze コマンド

### 10.1 基本分析

```bash
wtenv analyze
```

**期待結果:**
- [ ] worktreeの分析結果が表示される

### 10.2 詳細分析

```bash
wtenv analyze -d
wtenv analyze --detailed
```

**期待結果:**
- [ ] より詳細な分析結果が表示される

---

## 11. clean コマンド

### 11.1 ドライラン

```bash
wtenv clean --dry-run
```

**期待結果:**
- [ ] 削除対象が表示される
- [ ] 実際には削除されない

### 11.2 マージ済みのみ

```bash
wtenv clean --merged-only --dry-run
```

**期待結果:**
- [ ] マージ済みブランチのworktreeのみ対象

### 11.3 古いworktreeのみ

```bash
wtenv clean --stale-days 30 --dry-run
```

**期待結果:**
- [ ] 30日以上更新されていないworktreeのみ対象

### 11.4 強制削除

```bash
wtenv clean -f
```

**期待結果:**
- [ ] 確認なしで削除される

---

## 12. notify コマンド

### 12.1 成功時通知

```bash
wtenv notify "echo 'success'"
```

**期待結果:**
- [ ] コマンドが実行される
- [ ] デスクトップ通知が表示される

### 12.2 失敗時通知

```bash
wtenv notify "exit 1"
```

**期待結果:**
- [ ] エラー通知が表示される

### 12.3 作業ディレクトリ指定

```bash
wtenv notify "pwd" -d /tmp
```

**期待結果:**
- [ ] `/tmp` で実行される

---

## 13. pr コマンド

### 13.1 PR番号指定

```bash
# GitHub CLIが必要
wtenv pr 123
```

**期待結果:**
- [ ] PR情報を取得してworktreeが作成される
- [ ] または適切なエラーメッセージ

---

## 14. ui コマンド

### 14.1 TUI起動

```bash
wtenv ui
```

**期待結果:**
- [ ] インタラクティブTUIが起動する
- [ ] worktree一覧が表示される
- [ ] キー操作で移動できる
- [ ] `q` で終了できる

---

## 15. グローバルオプション

### 15.1 詳細モード

```bash
wtenv -v list
wtenv --verbose list
```

**期待結果:**
- [ ] 詳細情報が追加表示される

### 15.2 サイレントモード

```bash
wtenv -q create feature-quiet-test
wtenv --quiet list
```

**期待結果:**
- [ ] 成功時の出力が抑制される
- [ ] エラーは表示される

---

## 16. エラーハンドリング

### 16.1 gitリポジトリ外での実行

```bash
cd /tmp
mkdir not-a-repo && cd not-a-repo
wtenv list
```

**期待結果:**
- [ ] 適切なエラーメッセージが表示される

### 16.2 無効なブランチ名

```bash
wtenv create "invalid branch name with spaces"
```

**期待結果:**
- [ ] 適切なエラーメッセージが表示される

### 16.3 権限エラー

```bash
wtenv create test-branch /root/no-permission
```

**期待結果:**
- [ ] 適切なエラーメッセージが表示される

---

## 17. パフォーマンス確認

### 17.1 起動時間

```bash
time wtenv --help
```

**期待結果:**
- [ ] 50ms未満

### 17.2 バイナリサイズ

```bash
ls -lh target/release/wtenv
```

**期待結果:**
- [ ] 5MB未満

---

## 18. クリーンアップ

```bash
# テスト用データを削除
cd /tmp
rm -rf wtenv-test-repo
rm -rf feature-a feature-b feature-c
rm -rf custom-path
rm -rf empty-repo no-config-test not-a-repo
```

---

## 確認完了チェックリスト

| カテゴリ | 項目数 | 完了 |
|---------|--------|------|
| 基本コマンド | 2 | [ ] |
| init | 4 | [ ] |
| config | 3 | [ ] |
| create | 8 | [ ] |
| list | 3 | [ ] |
| status | 2 | [ ] |
| remove | 3 | [ ] |
| ps/kill | 4 | [ ] |
| diff-env | 2 | [ ] |
| analyze | 2 | [ ] |
| clean | 4 | [ ] |
| notify | 3 | [ ] |
| pr | 1 | [ ] |
| ui | 1 | [ ] |
| グローバルオプション | 2 | [ ] |
| エラーハンドリング | 3 | [ ] |
| パフォーマンス | 2 | [ ] |

**確認者:** _______________
**確認日:** _______________
**バージョン:** _______________
