# CLAUDE.md

Petoi Bittle を Jev（理性層）とハエ脳（反射層）で動かすプロジェクト。概要は [README.md](README.md)。

## 最初に読む

- 設計: [docs/design/](docs/design/)（特に `02-architecture.md` と `decisions.md`）
- ルール: [docs/rules/](docs/rules/)。ここに書いたことは要約で、食い違ったら rules が正しい

## 必ず守ること

- 変更のたびに `scripts/check.sh` を通す（ruff format / ruff check / mypy --strict / pytest）。
- 振る舞いを変えるときは、先にテストを書く。外部 I/O は Protocol の後ろに置き、Fake でテストする。
- 実機（Bittle）や Jev の API キーが要る確認は、自分で実行しない・「確認した」と書かない。
  `experimentals/TEMPLATE.md` から手順書を作り、`[実験依頼]` の PR で人に依頼する。
- 事実・設計案・未確認を区別して書く（`docs/rules/documentation.md`）。推測を事実として書かない。
- モジュール構成や Protocol を変えたら、同じ PR で `docs/design/` を直す。未決定事項を決めたら `decisions.md` を更新する。
- PR の前に `docs/rules/review.md` の必須サブエージェント（`.claude/agents/`）でレビューし、対応を PR 本文に書く。
- PR をマージしない（人が行う）。
- API キー・IP アドレスをコミットしない。

## よく使うコマンド

```sh
uv sync                       # 依存を入れる
./scripts/check.sh            # CI と同じ検査
uv run pytest tests/unit      # 単体テストだけ
uv run pytest -m jev          # 本物の Jev を呼ぶテスト（API キーが必要。手元だけ）
```
