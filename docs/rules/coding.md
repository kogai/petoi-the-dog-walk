# コードの書き方

言語は Rust（[D-09](../design/decisions.md)）。Python は使わない。
方針は「純粋な核と薄い殻」。判断のロジックは純粋関数にして `walk-core` に置き、I/O は外側のクレートに閉じ込める
（クレートの分け方は [02-architecture.md](../design/02-architecture.md) 2節）。

## 1. 純粋な核（`walk-core`）

- `#![no_std]`（+ `alloc`）で書く。ファイル・ネットワーク・時計・スレッド・乱数は、そもそも使えない。
- 通常の依存（`[dependencies]`）を持たない。テスト用の `[dev-dependencies]`（proptest など）は可。
- 関数は入力だけから出力を決める。時刻などは引数で受け取る。
- 値は不変で扱う。`mut` は関数の中の局所変数に限り、共有された状態を書き換えない。内部可変性（`Cell`・`RefCell` など）は使わない。
- 選択肢は `enum` で表し、`match` で網羅する。ワイルドカード（`_ =>`）で網羅を逃れない（lint で禁止）。
- 取り違えやすい値は newtype で区別する（例: `Millis`（時刻）と `MillisSpan`（長さ）、`Probability`。ドライバーの中では `Token`）。検査が要る型は、検査する関数からしか作れないようにする。
- デバイスに依存するもの（トークンの表など）は置かない。動かす指示は意味（`Action`）で表し、デバイスへの変換はドライバーが持つ（[ADR-0001](../design/adr/0001-driver-layer.md)）。
- 境界の trait（`Transport`・`Reasoner`・`Reflex`、センサーの trait、`Link`・`Clock`）はここに置く。

## 2. 殻（`walk-jev`、`walk-bittle`、`walk-flybrain`、`walk-app`）

- I/O はここだけで行う。
- 外から来る値（Jev の応答、Bittle の応答、設定ファイル）は serde で専用の型に変換し、検査してから核の型にする。
- 殻は薄く保つ。条件分岐が増えてきたら、判断の部分を核の純粋関数に移す。

### 2.1 ドライバー（`walk-jev`、`walk-bittle`、`walk-flybrain`）

形と理由は [ADR-0001](../design/adr/0001-driver-layer.md)。書くときに守ること:

- ドライバーが持つのは、デバイスとのやり取りの手順と、デバイス固有の表（`Action` → トークンなど）だけ。判断のロジックは持たない。
- バイトの流れで話すドライバーは、`Link` と `Clock` を引数で受け取り、ジェネリクスで持つ。シリアルポートを直接開かない（開くのは `Link` の実装だけ）。
- `Link` の実装は中身の意味を知らない。読み書きとも時間の上限を持ち、中で再試行しない。
- 受け取ったバイトの解釈は純粋関数にする（置き場所は D-17）。
- 実機を動かすドライバーは、ADR-0001 2.6節の安全の不変条件（S1〜S11）を守り、それぞれを契約テストで確かめる（[testing.md](testing.md) 3節）。

### 2.2 組み立て（`walk-app`）

- 使うドライバーとリンクは、`walk-app` にある enum の値からだけ選ぶ。設定に知らない値があれば起動しない。
- 本物の `Clock` の実装はここだけに置き、ドライバーへ渡す。
- 終了と Ctrl-C・SIGTERM では、明示して `stop()` を呼ぶ（`Drop` に任せない。ADR-0001 S1）。

## 3. 失敗の扱い（全クレート）

- 失敗は `Result` で返す。エラー型は `enum` で定義し、呼び出し側が場合分けできるようにする。
- `unwrap`・`expect`・`panic!`・`todo!`・`unimplemented!`・添字アクセス `v[i]` は使わない（lint で禁止）。テストコードでは使ってよい（`clippy.toml` の `allow-*-in-tests` で許可する）。
- 実機を動かすクレートでは、panic しても停止を送る（ドライバーの `Drop` で送る。[06-transport.md](../design/06-transport.md)、ADR-0001 S1）。
  そのため、Cargo のプロファイルに `panic = "abort"` を書かない（巻き戻さないと `Drop` が走らない。検査で確かめる）。

## 4. その他

- `unsafe` は使わない（`#![forbid(unsafe_code)]`）。
- 公開する項目には doc コメントを書く（`walk-core` は lint で必須）。
- 非同期ランタイムは使わない。並行処理は標準のスレッドとチャネルで書く。
