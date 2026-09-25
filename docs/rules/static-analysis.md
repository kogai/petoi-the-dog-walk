# 静的検査

すべて `scripts/check.sh` で一度に実行でき、CI も同じスクリプトを使う。設定はワークスペースの `Cargo.toml` の `[workspace.lints]` と、`rustfmt.toml`・`clippy.toml`・`rust-toolchain.toml` に置く。

| 検査 | ツール | 失敗したら |
|---|---|---|
| 整形 | `cargo fmt --check` | `cargo fmt` で直す |
| lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings`（`--all-features` で、実 API・実機のテストもコンパイルと lint だけは通す。実行はしない） | 直す |
| 型・借用 | コンパイラ（clippy の実行に含まれる） | 直す |
| ドキュメント | `cargo doc --no-deps`（壊れたリンクをエラーにする） | 直す |
| 依存の境界 | `cargo metadata` で各クレートの依存を読み、[02-architecture.md](../design/02-architecture.md) 2節の「依存してよいもの」と照合する（`walk-core` は通常の依存を持たない、`walk-testing` は dev-dependencies からだけ、など） | 依存を外す。設計を変えるなら設計書と照合の表を直す |
| プロファイル | `panic = "abort"` が無いこと（[coding.md](coding.md) 3節） | 消す |
| カバレッジ | `cargo llvm-cov --fail-under-lines 80` | テストを足す |
| テスト | `cargo test --workspace` | 直す |

ツールチェーンのバージョンは `rust-toolchain.toml` で固定する。

## lint の方針

- rustc: `unsafe_code = "forbid"`、`missing_docs`（`walk-core`）、`unused_must_use` などを deny。
- clippy: `pedantic` を有効にする。加えて次を deny する（テストコードは除く）。
  - `unwrap_used`・`expect_used`・`panic`・`todo`・`unimplemented`・`indexing_slicing`: 失敗は `Result` で返す（[coding.md](coding.md) 3節）。
  - `wildcard_enum_match_arm`: `enum` の `match` で網羅を逃れない。
  - `print_stdout`・`print_stderr`・`dbg_macro`: 出力はログの仕組みを通す（`walk-app` と実験用の実行ファイルは除く）。
- 警告はすべてエラーとして扱う（`-D warnings`）。
- テストコードの例外（`unwrap` など）は `clippy.toml` の `allow-*-in-tests` で許可する。
- クレートごとの例外（`walk-app` と実験用の実行ファイルの print 系など）は、`lib.rs`・`main.rs` の先頭にクレート属性 `#![expect(..., reason = "...")]` で書く（Cargo では `[lints] workspace = true` とクレートごとの上書きを併用できないため）。

## 例外の書き方

- lint を外すときは、項目単位で `#[expect(clippy::<名前>, reason = "<理由>")]` と書く。`#[allow]` は使わない（`allow_attributes` を deny）。
  `expect` は、外す必要が無くなると警告になるので、不要な例外が残らない。
- クレート全体で外すときは、PR で理由を説明する。
