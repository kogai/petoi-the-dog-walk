# CLAUDE.md

Petoi Bittle を Jev（理性層）とハエ脳（反射層）で動かすプロジェクト。概要は [README.md](README.md)。

## 最初に読む

- 設計: [docs/design/](docs/design/)（特に `02-architecture.md` と `decisions.md`）
- ルール: [docs/rules/](docs/rules/)。ここに書いたことは要約で、食い違ったら rules が正しい

## 必ず守ること

- 言語は Rust。Python は使わない。判断のロジックは `#![no_std]` の `walk-core` に純粋関数で書き、I/O は殻のクレートに閉じ込める（`docs/rules/coding.md`）。
- `unwrap`・`expect`・`panic!`・`todo!`・`unimplemented!`・添字アクセス・`unsafe` を使わない（テストコードは除く）。失敗は `Result` で返す。非同期ランタイムは使わない（詳細は `docs/rules/coding.md` 3・4節）。
- 変更のたびに `scripts/check.sh` を通す（fmt / clippy / doc / 依存の境界 / test）。
- 振る舞いを変えるときは、先にテストを書く。外部 I/O は trait の後ろに置き、Fake でテストする。
- **1つの PR で行うことは1つにする**（`docs/rules/git-workflow.md` 3節）。ついでの修正を混ぜない。
- 実機（Bittle）や Jev の API キーが要る確認は、自分で実行しない・「確認した」と書かない。
  手順書を作り、実験1つにつき1つの `[実験依頼]` PR で人に依頼する。
- 事実・設計案・未確認を区別して書く（`docs/rules/documentation.md`）。推測を事実として書かない。
- クレート構成や境界の trait を変えたら、同じ PR で `docs/design/` を直す。未決定事項を決めたら `decisions.md` を更新する。
- PR の前に `docs/rules/review.md` の必須サブエージェント（`.claude/agents/`）でレビューし、対応を PR 本文に書く。
- PR は、必須のサブエージェントの確認レビューが PASS で、最新のコミットの CI が通っていればマージしてよい（merge commit。`docs/rules/review.md` 1節）。
- **公開リポジトリ**。秘密情報とローカル環境の状況（ホームのパス、IP、ポート名、機器名など）をコミットしない。
  コミット前に `git diff --staged` を読み、`scripts/check-public.sh` を通す（`docs/rules/public-repo.md`）。

## よく使うコマンド

```sh
./scripts/check.sh                                   # CI と同じ検査
cargo test -p walk-core                              # 核のテストだけ
```

`--features live-jev`・`--features live-hardware` を付けたテストは、本物の Jev（有料）を呼ぶか実機を動かす。
**エージェントは実行しない。** 人が手元で実行する。
