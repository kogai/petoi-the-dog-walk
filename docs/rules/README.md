# 開発ルール

人にも AI エージェントにも同じルールを適用する。エージェント向けの入口は [CLAUDE.md](../../CLAUDE.md)。

| 文書 | 内容 |
|---|---|
| [coding.md](coding.md) | コードの書き方（TypeScript、関数型のコアと命令型の殻） |
| [testing.md](testing.md) | 自動テストの方針 |
| [static-analysis.md](static-analysis.md) | 静的検査（整形・lint・型・モジュールの境界） |
| [review.md](review.md) | サブエージェントによるレビューと人のレビュー |
| [git-workflow.md](git-workflow.md) | ブランチ・コミット・PR・実験依頼の流れ |
| [documentation.md](documentation.md) | 文書の書き方（確認済み／設計案／未確認の区別） |
| [public-repo.md](public-repo.md) | 公開リポジトリでの注意（コミットしてはいけないもの） |

## 変更1回ごとの最低条件（Definition of Done）

1. `scripts/check.sh` が通る（公開内容・整形・lint・型・境界・テスト）。
2. 振る舞いを変えたら、それを確かめるテストがある。
3. 設計と違うことをしたら、同じ PR で `docs/design/` を直す。
4. [review.md](review.md) の必須レビューを通し、指摘への対応を PR に書く。
5. 秘密情報・ローカル環境の状況をコミットしていない（[public-repo.md](public-repo.md)）。
6. 実機や API キーが要る確認は、自分で「確認した」と書かない。`experimentals/` に手順を書いて人に依頼する。
