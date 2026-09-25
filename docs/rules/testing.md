# 自動テストの方針

## 1. 原則

- テストは速く、決定的で、ネットワークと実機を使わない（既定の `pytest` 実行で）。
- 外部 I/O（Jev API、シリアル、HTTP、時計）は Protocol の後ろに置き、Fake に差し替えてテストする。
- 振る舞いを足す・変えるときは、先に失敗するテストを書く（テスト先行）。
- 「実機で動いた」は自動テストの代わりにならない。実機の確認は `experimentals/` に記録する。

## 2. テストの種類

| 種類 | 場所 | 対象 | CI で実行 |
|---|---|---|---|
| 単体 | `tests/unit/` | 純粋関数・型（`domain`、`arbiter`、`skills`、変換関数） | する |
| プロパティ | `tests/unit/`（hypothesis） | 不変条件（例: [05-arbitration.md](../design/05-arbitration.md) の P1〜P5） | する |
| 契約 | `tests/contract/` | Fake と本物の実装が同じ Protocol を満たすか。記録済み応答（fixture）を本物のパーサーに通す | する |
| 結合 | `tests/integration/` | Fake 一式でループを偽の時計で回す | する |
| Jev 実 API | `@pytest.mark.jev` | 本物の Jev を呼ぶ | しない（API キーがある手元だけ） |
| 実機 | `@pytest.mark.hardware` | 本物の Bittle を動かす | しない（人が実行し `experimentals/` に記録） |

`jev`・`hardware` マーカーのテストは、オプションを付けたときだけ実行する（`tests/conftest.py`）。
`-m` の指定では有効にならないので、別の `-m` 式を使っても誤って実 API・実機を動かさない。

```sh
uv run pytest --run-jev -m jev            # 本物の Jev（API キーが必要）
uv run pytest --run-hardware -m hardware  # 本物の Bittle（人が実行）
```

- テストのディレクトリには同じ名前のファイルがあってもよい（`--import-mode=importlib`）。
- 警告はエラーとして扱う（`filterwarnings = ["error"]`）。

## 3. Fake と fixture の決まり

- Fake は `src/petoi_walk/testing/` に置き、本物と同じ Protocol を実装する（型検査で保証する）。
- Fake の振る舞いは、実験で記録した実際の応答に合わせる。根拠の実験 ID を docstring に書く。
- 記録済み応答は `tests/fixtures/<相手>/` に置く。API キー・IP アドレスなどは含めない。
- 時計・乱数は注入する。テストの中で `time.sleep` しない。
- hypothesis は CI（`scripts/check.sh`）では `HYPOTHESIS_PROFILE=ci` で seed を固定する。手元では既定の `dev` プロファイル（ランダム）で回し、見つかった反例は `@example` で固定する。

## 4. カバレッジ

- `scripts/check.sh` がブランチカバレッジを測る（`pytest --cov`）。1ファイルだけ実行するときは測らないので、下限で落ちない。
- 全体の下限は 80%（`pyproject.toml` の `[tool.coverage.report] fail_under`）。
- `arbiter` と `domain` はブランチカバレッジ 100% を目標にする。
- 本物の I/O 実装（`SerialTransport` など）の I/O 部分は、`# pragma: no cover` ではなく契約テストでカバーする。どうしても無理な行だけ、理由をコメントして除外する。

## 5. 書き方

- テスト名は `test_<条件>_<期待する結果>`（例: `test_stale_decision_falls_back_to_reflex`）。
- 1テスト1つの振る舞い。Arrange / Act / Assert を空行で分ける。
- 失敗したテストを skip・xfail・削除して CI を通さない。仕様が変わった場合だけ、理由を PR に書いて直す。
