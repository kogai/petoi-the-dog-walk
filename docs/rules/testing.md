# 自動テストの方針

道具は `cargo test`（単体・結合）、proptest（プロパティテスト）、cargo-llvm-cov（カバレッジ）。

## 1. 原則

- 既定のテスト（`cargo test`）は速く、決定的で、ネットワークと実機を使わない。
- 外部 I/O（Jev API、シリアル、HTTP、時計）は trait の後ろに置き、Fake に差し替えてテストする。差し替える所は3つ: 境界の trait（Fake）、`Link`（`FakeLink`）、`Clock`（偽の時計）（[ADR-0001](../design/adr/0001-driver-layer.md) 2.5節）。
- 振る舞いを足す・変えるときは、先に失敗するテストを書く（テスト先行）。
- 「実機で動いた」は自動テストの代わりにならない。実機の確認は実験として記録する。

## 2. テストの種類

| 種類 | 場所 | 対象 | CI で実行 |
|---|---|---|---|
| 単体 | 各モジュールの `#[cfg(test)] mod tests` | 純粋関数（主に `walk-core`） | する |
| プロパティ | 同上（proptest） | 不変条件（例: [05-arbitration.md](../design/05-arbitration.md) の P1〜P5） | する |
| 契約 | 殻のクレートの `tests/` | Fake と本物の実装が同じ trait を同じように満たすか。本物のドライバーに `FakeLink` を付け、記録済み応答（fixture）を流して手順を確かめる。実機を動かすドライバーは、ADR-0001 2.6節の安全の不変条件（S1〜S11）を1つずつテストにする | する |
| 結合 | `walk-app/tests/` | Fake 一式でループを偽の時計で回す | する |
| Jev 実 API | cargo feature `live-jev` を付けたときだけコンパイルされるテスト | 本物の Jev を呼ぶ | しない（API キーがある手元だけ） |
| 実機 | cargo feature `live-hardware` を付けたときだけコンパイルされるテスト | 本物の Bittle を動かす | しない（人が実行し、実験として記録） |

実 API と実機のテストは `#[cfg(feature = "...")]` で囲むので、feature を付けない限りコンパイルすらされない。
名前の指定（フィルター）で誤って動くことはない。

```sh
cargo test --workspace                                  # 既定（CI と同じ範囲）
cargo test -p walk-jev --features live-jev              # 本物の Jev（API キーが必要）
cargo test -p walk-bittle --features live-hardware      # 本物の Bittle（人が実行）
```

## 3. Fake と fixture の決まり

- Fake は `walk-testing` クレートに置き、本物と同じ trait を実装する。`dev-dependencies` からだけ使う。`FakeLink` もここに置く。
- `FakeLink` は、決めたバイト列を、偽の時計の上の決めた時刻に返す。書かれたバイト列を記録し、テストで確かめられるようにする（例: 停止の後に何も書かれていない）。書き込みを失敗させる設定も持つ。
- Fake の振る舞いは、実験で記録した実際の応答に合わせる。根拠の実験 ID をコメントに書く。
- 記録済み応答は、そのクレートの `tests/fixtures/` に置く。API キー・IP アドレス・ポート名などは含めない。実験の手順書に記録した出力（例: E00）を使うときは、出典の実験 ID と節を書く。
- 時計・乱数は引数で渡す。テストの中で実時間を待たない（`thread::sleep` しない）。
- proptest が見つけた反例は `proptest-regressions/` に保存されるので、コミットして回帰テストにする。

## 4. カバレッジ

- CI で cargo-llvm-cov を使って測る。
- 全体の下限は行 80%（`cargo llvm-cov --fail-under-lines 80` で強制する）。
- `walk-core` は行 100% を目標にする。分岐カバレッジは nightly のツールチェーンが要るので、強制せず目標に留める。
- 本物の I/O 実装の I/O 部分も、契約テストでできるだけ通す。

## 5. 書き方

- テスト名は「条件_期待する結果」の形の英語の snake_case にする（例: `stale_decision_falls_back_to_reflex`）。
- 1テスト1つの振る舞い。準備・実行・確認を空行で分ける。
- 失敗したテストを `#[ignore]`・削除で消して CI を通さない。仕様が変わった場合だけ、理由を PR に書いて直す。
