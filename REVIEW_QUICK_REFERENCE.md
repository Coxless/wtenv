# Code Review - Quick Reference Guide

## 📋 三文書の構成

### 1. REVIEW_SUMMARY.md (エグゼクティブサマリー)
**読むべき人**: プロジェクトマネージャー、 lead developer
**所要時間**: 10-15分

```
- 全体的な評価スコア (60% → 95%)
- ファイル別スコア
- リスク評価
- 修正スケジュール提案
```

### 2. CODE_REVIEW.md (詳細レビュー)
**読むべき人**: 実装者
**所要時間**: 30-40分

```
- 各ファイルの良い点 / 改善提案
- HIGH/MEDIUM/LOW の問題を分類
- 詳細な説明と背景
- テスト提案を含む
```

### 3. CODE_REVIEW_SUPPLEMENTS.md (実装例)
**読むべき人**: 実装者（修正時）
**所要時間**: 対象セクションのみ

```
- 実装例とテストコード
- パフォーマンス最適化方法
- セキュリティ改善パターン
- ベンチマーク例
```

---

## 🎯 優先修正リスト

### Phase 1: Critical Fixes (必須)
| # | 問題 | ファイル | 修正難度 | 所要時間 |
|---|------|---------|--------|--------|
| 1 | JSONLパース: 1行エラーで全体失敗 | claude_task.rs | ⭐☆☆ | 1h |
| 2 | TaskManager::load() エラー通知なし | ui.rs | ⭐☆☆ | 30m |
| 3 | execute_with_notification() path未検証 | notify.rs | ⭐☆☆ | 1h |
| 4 | Python script エラー通知なし | track-progress.py | ⭐☆☆ | 1h |

**合計**: 約 4 時間

### Phase 2: Important Fixes (推奨)
| # | 問題 | ファイル | 修正難度 | 所要時間 |
|---|------|---------|--------|--------|
| 5 | TaskManager キャッシング | claude_task.rs | ⭐⭐☆ | 3h |
| 6 | Python エラーハンドリング全面改善 | track-progress.py | ⭐⭐☆ | 2h |
| 7 | is_in_worktree() prefix match 修正 | claude_task.rs | ⭐⭐☆ | 1h |
| 8 | テストカバレッジ拡充 | すべて | ⭐⭐⭐ | 4h |

**合計**: 約 10 時間

---

## 📊 問題分析マトリックス

```
          影響度
           高  │  中  │  低
─────────┼─────┼─────┼─────
確率 高  │  1  │  4  │  6
         ├─────┼─────┼─────
       中 │  2  │  5  │  7
         ├─────┼─────┼─────
       低 │  3  │  8  │  9
─────────┴─────┴─────┴─────

1 = JSONL パース (HIGH/HIGH)
2 = UI キャッシング (MEDIUM/MEDIUM)
3 = shell コマンドインジェクション (MEDIUM/LOW)
4 = エラー通知 (HIGH/MEDIUM)
5 = Python エラーハンドリング (MEDIUM/MEDIUM)
6 = is_in_worktree() (MEDIUM/LOW)
7 = テスト不足 (MEDIUM/MEDIUM)
8 = stdout/stderr 順序 (LOW/LOW)
9 = ドキュメント (LOW/LOW)
```

---

## 🔍 各ファイルの症状チェック

### src/commands/claude_task.rs
```
症状: セッションファイルが破損していると全体が読み込み失敗
原因: JSONL 1 行のパースエラーで ? 演算子が全体を終了
治療: 行単位で error handling を分離

症状: 大量セッション時に UI が遅延
原因: 毎回全ファイルを読み込むため O(n)
治療: メモリキャッシングと差分更新

症状: /a/b と /a/bc が誤検出される可能性
原因: starts_with で素朴な prefix match をしている
治療: パスコンポーネント単位の比較に変更
```

### src/commands/ui.rs
```
症状: TaskManager::load() エラーが silent に処理される
原因: unwrap_or_default() で失敗を無視している
治療: エラーを stderr に出力

症状: 小さいターミナル（< 26 行）でレイアウト崩れ
原因: 固定サイズ制約で最小ターミナルサイズが不足
治療: responsive layout または最小サイズチェック

症状: ワーカートリー名が symlink 未対応
原因: canonicalize を使わずに Path::name で抽出
治療: canonical path を優先的に使用
```

### src/commands/notify.rs
```
症状: working_dir が存在しなくてもエラーにならない
原因: current_dir() が存在確認を行わない可能性
治療: 事前に存在確認をして Bail

症状: "rm -rf /" が実行できてしまう
原因: bash -c で生文字列が渡される
治療: 危険パターン検出または shell を使わない API

症状: stdout/stderr の出力順序が不定
原因: バッファリングの都合で順序が逆になる可能性
治療: タイムスタンプを付けるか、先に stderr で flush
```

### .claude/hooks/track-progress.py
```
症状: hook が失敗しても Claude に通知されない
原因: exception を catch して silent に処理
治療: error.log に記録 + stderr に警告

症状: エラーログが多重実行で競合する可能性
原因: file append 時のロック機構がない
治療: os.open() で排他ロック、または flock 使用

症状: tool_input から抽出したパスが symlink 未対応
原因: Path(file_path).name だけで resolve していない
治療: Path.resolve() で canonical 化

症状: ディスク full 時にエラーログが書き込み失敗
原因: OSError を catch していない
治療: write 失敗時は stderr に fallback
```

---

## ✅ 修正前チェックリスト

修正を始める前に以下を確認:

- [ ] 現在のブランチは `claude/track-code-task-progress-GSxPX` か？
- [ ] 最新の main に merge 済みか？ (rebase 必須)
- [ ] 仮想環境は activate されているか？
- [ ] `cargo test` がすべて pass するか？
- [ ] リモートに push する前に local test 済みか？

---

## 🚀 修正後の検証

各修正後に実施すべき検証:

### Phase 1 修正後
```bash
# JSONLパース改善の検証
cargo test test_load_session_with_malformed_json

# エラー通知の検証
cargo test --lib
cargo run -- ui  # 手動で task loading error を確認

# コマンド実行検証
cargo test test_execute_with_invalid_directory
```

### Phase 2 修正後
```bash
# パフォーマンステスト
cargo test bench_load_many_sessions --release

# is_in_worktree 修正の検証
cargo test test_is_in_worktree_prefix_false_positive

# テストカバレッジ
cargo tarpaulin --out Html
```

---

## 📞 Reviewers へのQ&A 想定

**Q1**: JSONLパース修正で、破損行をスキップすると、イベント順序が変わる？
```
A: 不変。破損行のイベントは捨てられるが、残りは元の順序を保つ。
   ClaudeTask::events は追加順（timestamp ではなく）を保つため OK。
```

**Q2**: TaskManager キャッシングで、cache TTL は いくつがいい？
```
A: 推奨値は 500ms（UI ポーリング間隔 100ms × 5）。
   ただし、user の refresh 頻度で調整可能。
```

**Q3**: コマンド実行のセキュリティで、shell 機能は必須か？
```
A: notify コマンドの想定用途による。
   - 単純なコマンド実行なら shell 不要 → 修正推奨
   - パイプ・リダイレクト使用なら shell 必須 → validation 強化
```

**Q4**: Pythonスクリプトで、エラーログの保持期間は？
```
A: 現在は無期限。
   大規模運用なら logrotate で 30日程度に設定推奨。
```

---

## 📚 参考資料リンク

| リソース | リンク |
|---------|-------|
| JSONL spec | https://jsonlines.org/ |
| Rust anyhow | https://docs.rs/anyhow/latest/anyhow/ |
| Python pathlib | https://docs.python.org/3/library/pathlib.html |
| chrono DateTime | https://docs.rs/chrono/latest/chrono/ |
| notify-rust | https://docs.rs/notify-rust/latest/notify_rust/ |

---

## 🎓 学習ポイント

このレビューから学べることは:

1. **JSONL はライン指向形式** → 行単位での error handling が重要
2. **Path prefix matching の落とし穴** → 常に component-wise で比較
3. **UI における blocking I/O** → キャッシングとリフレッシュ戦略が必須
4. **エラー通知の重要性** → silent failure は worst case
5. **Cross-language integration** → enum/status は single source of truth にすべき

---

**作成日**: 2025-12-30
**バージョン**: v1.0
**ステータス**: Ready for Review and Implementation
