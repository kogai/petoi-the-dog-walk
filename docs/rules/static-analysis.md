# 静的検査

すべて `scripts/check.sh` で一度に実行でき、CI（`.github/workflows/ci.yml`）も同じスクリプトを使う。
設定は `pyproject.toml` に集める。

| 検査 | ツール | 設定 | 失敗したら |
|---|---|---|---|
| フォーマット | `ruff format` | 行長 100 | `uv run ruff format` で直す |
| lint | `ruff check` | 下の規則セット | 直す。自動修正は `uv run ruff check --fix` |
| 型 | `mypy --strict` | `src/` と `tests/` | 直す |
| テスト | `pytest` | [testing.md](testing.md) | 直す |
| ロックファイル | `uv lock --check` | `uv.lock` | `uv lock` を実行してコミット |

## ruff の規則セット

`E`, `W`, `F`（基本）, `I`（import 順）, `B`（バグになりやすい書き方）, `UP`（新しい構文）,
`SIM`（簡略化）, `RUF`, `PT`（pytest）, `ASYNC`（非同期の誤用）, `S`（セキュリティ。テストでは `S101` を除外）,
`N`（命名）, `ANN`（型注釈）, `TID`（import 制限）。

- `TID251` で `typesafe_sdk` の import を `reasoning/jev.py` 以外で禁止する（[03](../design/03-reasoning-layer.md) 4節）。本体を実装するときに設定を足す。

## 例外の書き方

- 規則を無効にするときは、行単位で `# noqa: <コード>  # <理由>` と書く。ファイル単位・全体での無効化は PR で理由を説明する。
- `type: ignore` は `# type: ignore[<コード>]  # <理由>` の形だけ許す（`warn_unused_ignores` で余分なものは検出される）。
- `Any` は外部ライブラリの境界だけで使い、すぐに自前の型へ変換する。

## 秘密情報

- API キー・IP アドレス・シリアル番号をコミットしない。`.env` は `.gitignore` 済み。
- `ruff` の `S105`/`S106`（ハードコードされたパスワード）で一部を検出する。
