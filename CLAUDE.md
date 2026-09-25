# CLAUDE.md

Petoi Bittle を Jev（理性層）とハエ脳（反射層）で動かすプロジェクト。概要は [README.md](README.md)。

## 最初に読む

- 設計: [docs/design/](docs/design/)（特に `02-architecture.md` と `decisions.md`）
- ルール: [docs/rules/](docs/rules/)。ここに書いたことは要約で、食い違ったら rules が正しい

## 必ず守ること

- 言語は TypeScript。Python は使わない。コアは純粋関数・不変データ・`Result` 型で書き、I/O は殻に閉じ込める（`docs/rules/coding.md`）。
- 変更のたびに `scripts/check.sh` を通す（公開内容の検査 / Biome / ESLint / tsc / dependency-cruiser / Vitest）。
- 振る舞いを変えるときは、先にテストを書く。外部 I/O は関数の型の後ろに置き、Fake でテストする。
- 実機（Bittle）や Jev の API キーが要る確認は、自分で実行しない・「確認した」と書かない。
  `experimentals/TEMPLATE.md` から手順書を作り、`[実験依頼]` の PR で人に依頼する。
- 事実・設計案・未確認を区別して書く（`docs/rules/documentation.md`）。推測を事実として書かない。
- モジュール構成や境界の型を変えたら、同じ PR で `docs/design/` を直す。未決定事項を決めたら `decisions.md` を更新する。
- PR の前に `docs/rules/review.md` の必須サブエージェント（`.claude/agents/`）でレビューし、対応を PR 本文に書く。
- PR をマージしない（人が行う）。
- **公開リポジトリ**。秘密情報とローカル環境の状況（ホームのパス、IP、ポート名、機器名など）をコミットしない。
  コミット前に `git diff --staged` を読む。詳細は `docs/rules/public-repo.md`。

## よく使うコマンド

```sh
pnpm install                  # 依存を入れる
pnpm check                    # CI と同じ検査
pnpm exec vitest run tests/unit  # 単体テストだけ
pnpm test:jev                 # 本物の Jev を呼ぶテスト（API キーが必要。手元だけ）
```
