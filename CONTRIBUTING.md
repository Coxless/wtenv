# Contributing to wtenv

wtenvへのコントリビューションをありがとうございます！

## 開発環境のセットアップ

### 必要要件

- Rust 1.92.0 以上
- Git

### セットアップ手順

```bash
# リポジトリをクローン
git clone https://github.com/USERNAME/wtenv.git
cd wtenv

# ビルド確認
cargo build

# テスト実行
cargo test
```

## コーディング規約

### フォーマット

```bash
# コードフォーマット
cargo fmt

# Lint チェック
cargo clippy
```

### コミットメッセージ

```
type: subject

body (optional)
```

**type**:
- `feat`: 新機能
- `fix`: バグ修正
- `docs`: ドキュメントのみの変更
- `refactor`: リファクタリング
- `test`: テストの追加・修正
- `chore`: ビルドプロセスや補助ツールの変更

**例**:
```
feat: worktree作成時の対話モード追加

引数なしでcreateコマンドを実行した場合、
ブランチ名とパスを対話的に入力できるようにした。
```

## プルリクエストのプロセス

1. **Issueの確認**: 作業を始める前に、関連するIssueがあるか確認してください
2. **ブランチ作成**: `feat/xxx` または `fix/xxx` の形式でブランチを作成
3. **実装**: コーディング規約に従って実装
4. **テスト**: `cargo test` が全てパスすることを確認
5. **フォーマット**: `cargo fmt` と `cargo clippy` を実行
6. **PR作成**: 変更内容を明確に記述したPRを作成

### PRチェックリスト

- [ ] `cargo test` が成功する
- [ ] `cargo fmt -- --check` が成功する
- [ ] `cargo clippy` で警告がない
- [ ] 必要に応じてドキュメントを更新した

## バグレポート

バグを見つけた場合は、以下の情報を含めてIssueを作成してください:

1. **環境情報**
   - OS（例: Ubuntu 22.04, macOS 14, Windows 11）
   - wtenv バージョン（`wtenv --version`）
   - Rust バージョン（`rustc --version`）

2. **再現手順**
   - バグを再現するための具体的な手順

3. **期待される動作**
   - 本来どのように動作すべきか

4. **実際の動作**
   - 実際に何が起きたか（エラーメッセージがあれば含める）

### バグレポートテンプレート

```markdown
## 環境
- OS:
- wtenv version:
- Rust version:

## 再現手順
1.
2.
3.

## 期待される動作


## 実際の動作

```

## 機能リクエスト

新機能のアイデアがある場合は、Issueを作成してください:

1. **目的**: この機能が解決する問題
2. **提案**: 具体的な実装案
3. **代替案**: 検討した他の方法（あれば）

## ライセンス

コントリビューションは MIT License の下でライセンスされます。
