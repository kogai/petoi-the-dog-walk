# ドキュメント

| 場所 | 内容 |
|---|---|
| [design/](design/) | 設計書。何を作るか、どう分けるか、何が未決定か |
| [rules/](rules/) | 開発ルール。テスト方針、静的検査、レビュー、Git 運用、文書の書き方 |
| [../experimentals/](../experimentals/) | 人（実機・API キーを持つ人）に依頼する実験の手順と結果 |

## 読む順番

1. [design/01-overview.md](design/01-overview.md) — ゴールと制約
2. [design/02-architecture.md](design/02-architecture.md) — 全体構成とモジュール境界
3. [design/decisions.md](design/decisions.md) — 決定事項と未決定事項
4. [rules/README.md](rules/README.md) — 実装を始める前に守るルール

## 表記ルール（要約）

すべての文書で、記述を次の3種類に区別する。詳しくは [rules/documentation.md](rules/documentation.md)。

- **確認済み**: 公式ドキュメントまたはコードを読んで確認した事実（出典を付ける）
- **設計案**: この計画での決め事（実装前・未実行）
- **未確認 / 実験**: 実機や API で試して確かめる項目（`experimentals/` の ID を付ける）
