# E00: トークンを送ったときの応答の形式

- 状態: 依頼中
- 依頼日: 2026-09-25
- 実施日:
- 実施者: （GitHub のユーザー名）

## 目的

Mac から Bittle へトークンを送ったとき、Bittle が何を返すか（エコーの有無・形式・時間）と、
連続送信・歩行の継続がどう振る舞うかを記録する。
結果で、`SerialTransport`・`HttpTransport`・`FakeTransport` の振る舞い（[06-transport.md](../docs/design/06-transport.md) 2節）と、
`serialport` クレートを Mac で使えるか（[D-09](../docs/design/decisions.md)）が決まる。

接続手段は問わない。**手持ちの経路だけ**で行う（全部を試す必要はない）。

## 必要なもの

- 機材: Bittle（標準ファーム）、手持ちの接続手段（USB アダプタ / Bluetooth ドングル / ESP8266 のどれか）
- ソフトウェア（シリアル経路）:
  - Xcode Command Line Tools（Rust のビルドに要る。入っていなければ `xcode-select --install`）
  - [rustup](https://rustup.rs/)。入れた後は、新しいターミナルを開くか `source "$HOME/.cargo/env"` を実行する
  - このリポジトリを clone しておく（Rust のツールチェーンは初回のビルドで自動的に入る）
- ソフトウェア（Wi-Fi 経路）: `curl`（macOS に入っている）
- 所要時間の目安: 40分

## 安全上の注意

- Bittle を平らな床に置き、周囲 2m 以内に物・段差が無いことを確認する。台の上では行わない。
- **歩行のトークン（`kwkF`、`forward`）を送るときは、手で胴を持って足を浮かせる。** 歩き続けるかどうかは足の動きで分かる。
- 道具（`e00-serial-probe`）は、終わるとき・エラーのとき・`Ctrl-C` を押したとき・ターミナルを閉じたときに停止トークン `d` を送る。
  `d` を送れなかったときは `!! STOP TOKEN NOT SENT` と表示する。**その表示が出たら、すぐ電源を切る。**
  `d` でシリアル経由で止まるかどうか自体が、この実験で確かめること（**未確認**）。止まらなければ電源を切る。
- **`Ctrl-Z` は使わない**（道具が一時停止し、`d` が送られないまま動き続ける）。止めるときは `Ctrl-C`。
- Wi-Fi 経路で止めるときは、次を実行する。止まらなければ電源を切る。
  ```sh
  curl -sS --max-time 5 "http://<IP>/action?name=stop"
  ```

## 手順

コマンドは、リポジトリのルート（`Cargo.toml` がある場所）で実行する。
各手順の出力は `experimentals/raw/` に保存する（このフォルダはコミットされない）。
道具の出力は `E00_LOG=<ファイル>` を付けると、画面とファイルの両方に書かれる（`tee` は使わない。`Ctrl-C` のときに出力が失われるため）。
`experimentals/raw/` はリポジトリにある（中身はコミットされない）。

### A. シリアル経路（USB / Bluetooth）

道具: [`e00-serial-probe`](../crates/walk-experiments/src/bin/e00-serial-probe.rs)（**未実行**。偽のシリアルポートでは動作を確かめたが、実機ではまだ動かしていない）

0. Bluetooth の場合は、先に Mac とペアリングしておく（Petoi の公式ドキュメント https://docs.petoi.com/ の Bluetooth の手順に従う。該当ページはまだ確かめていない）。
1. 道具をビルドする（初回は数分かかる）。
   ```sh
   cargo build --release -p walk-experiments --bin e00-serial-probe
   ```
2. ポート名を調べる。**ポート名は結果に貼らない**（機器名や PC 名が入ることがある）。
   ```sh
   ./target/release/e00-serial-probe --list
   ```
   `/dev/cu.` で始まるものを選ぶ（`/dev/tty.` は使わない）。USB なら `usbserial` などを含む名前、Bluetooth ならペアリングした機器の名前を含むもの。
   `Bluetooth-Incoming-Port` や `debug-console` は関係が無い。それらしいポートが無ければ、手順を止めて「ポートが見つからない」と記録する。
   以下の `<PORT>` は、ここで見つけた名前に置き換える。
3. 接続の確認。接続直後に表示されるものも記録される。
   ```sh
   E00_LOG=experimentals/raw/e00-a3.txt ./target/release/e00-serial-probe <PORT> kbalance
   ```
   接続直後の表示（起動メッセージなど）が、最初の `->` の後まで続いていたら、待ち時間を延ばしてからやり直す。
   ```sh
   export E00_SETTLE_MS=5000
   ```
   （`export` した値は、同じターミナルでの以降の手順すべてに効く。）
4. 基本の動作。各トークンの後に 3 秒待つ。`kwkF` は 3 秒後の `kbalance` で止まる見込み（**未確認**）。**コマンドを実行する前から、胴を持って足を浮かせておく。**
   ```sh
   E00_LOG=experimentals/raw/e00-a4.txt ./target/release/e00-serial-probe <PORT> ksit kbalance khi kwkF kbalance
   ```
5. 同じトークンの連続送信。時間のかかる動作 `khi` を 0.2 秒あけて2回送り、2回目の後は 3 秒観察する。
   ```sh
   E00_WAIT_MS=200 E00_FINAL_WAIT_MS=3000 E00_LOG=experimentals/raw/e00-a5.txt ./target/release/e00-serial-probe <PORT> khi khi
   ```
6. 歩行は続くか。`kwkF` を送り、5 秒間なにも送らない。その後に `d` が送られる。**コマンドを実行する前から、胴を持って足を浮かせておく。**
   ```sh
   E00_FINAL_WAIT_MS=5000 E00_LOG=experimentals/raw/e00-a6.txt ./target/release/e00-serial-probe <PORT> kwkF
   ```
7. 存在しないトークン。
   ```sh
   E00_LOG=experimentals/raw/e00-a7.txt ./target/release/e00-serial-probe <PORT> kzzz
   ```
8. 任意: 改行コードを CRLF にして、手順 4 と同じトークン列を送る。**コマンドを実行する前から、胴を持って足を浮かせておく。**
   ```sh
   E00_EOL=CRLF E00_LOG=experimentals/raw/e00-a8.txt ./target/release/e00-serial-probe <PORT> ksit kbalance khi kwkF kbalance
   ```

出力の読み方: `->` は送ったもの、`<-` は受け取ったもの。左の数字は開始からのミリ秒。

### B. Wi-Fi 経路（ESP8266）（**未実行**）

1. Bittle の IP アドレスを確認する（[WiFi module ESP8266](https://docs.petoi.com/communication-modules/wifi-esp8266.md) の手順）。以下の `<IP>` を置き換える。**IP は結果に貼らない。**
2. 次を順に実行する（動作名はトークンではなく名前で送る。F-B5）。**コマンドを実行する前から、胴を持って足を浮かせておく**（`forward` を送るため）。
   各コマンドは、応答の本文と HTTP ステータスを表示する（5 秒で打ち切る）。
   ```sh
   for name in sit balance forward stop zzz; do
     echo "== $name"
     curl -sS --max-time 5 -o - -w '\nHTTP %{http_code}\n' "http://<IP>/action?name=$name"
     sleep 3
   done 2>&1 | tee experimentals/raw/e00-b.txt
   ```
3. `stop` の後も動いていたら、電源を切る。

## 記録すること

- [ ] 環境: 使った接続手段、ファームのバージョン（手順 A-3 の接続直後の表示に出るかもしれない（**未確認**）。出なければ「不明」）、Mac の OS と CPU（Apple シリコン / Intel）、`rustc --version`、このリポジトリのコミット（`git rev-parse --short HEAD`）
- [ ] 実施した経路（シリアルだけ / Wi-Fi だけ / 両方）。ポートが見つからなかった場合はそのこと
- [ ] 手順 A-1 のビルドが通ったか。通らなければエラー全文（ホームのパスを伏せる。実行時のエラー表示にも出ることがある）
- [ ] 手順 A-3〜A-8 の出力（`experimentals/raw/` のファイルの中身を、伏せた上で貼る）と、各手順での Bittle の動き
- [ ] 手順 A-3 で `E00_SETTLE_MS` を変えたか。変えたなら、その値
- [ ] 手順 B の出力と、各コマンドでの Bittle の動き
- [ ] 道具の動作で気になったこと（止まらなかった、表示がおかしい、など）

## 判定基準

| 観察 | 結果 | 設計への影響 |
|---|---|---|
| 応答（A-3・A-4） | トークンごとに決まった応答が返る | `SerialTransport::send` は応答を待って成否を判定する |
| 応答（A-3・A-4） | 応答が無い、または決まっていない | 送りっぱなしにして、送信間隔の下限だけで制御する |
| 連続送信（A-5） | 2回目で動作が最初からやり直しになる | 同じトークンの連続送信を必ず抑止する |
| 連続送信（A-5） | 2回目は無視される | 抑止は必須ではない（通信を減らすためだけに行う） |
| 連続送信（A-5） | 見分けられなかった | 安全側に倒し、同じトークンの連続送信を抑止する |
| 歩行（A-6） | 5 秒間歩き続ける | 同じトークンを連続で送らない（06 の設計案どおり） |
| 歩行（A-6） | 途中で止まる | 歩行中は一定間隔で再送する設計に変える。止まるまでの時間を記録する |
| 不正なトークン（A-7） | エラーの応答がある | `send` で検出して `Err` を返す |
| 不正なトークン（A-7） | 応答が無い、または正常時と同じ | 送る前に `Token` の許可リストで防ぐしかない（今の設計どおり）。検出はしない |
| 改行コード（A-8） | LF と CRLF で違いがある | 改行コードを設定にする |
| 改行コード（A-8） | 違いが無い、または実施しなかった | LF に固定する |
| Wi-Fi（B） | 応答本文かステータスで成否が分かる | `HttpTransport` はそれで成否を判定する |
| Wi-Fi（B） | どの名前でも同じ応答 | `HttpTransport` は HTTP のエラーと時間切れだけを失敗とする |
| 道具（A-1〜A-8） | ビルドでき、実機で動いた | `SerialTransport` は `serialport` クレートで書く |
| 道具（A-1〜A-8） | ビルドや接続に失敗した、またはポートが見つからなかった | 原因を調べて直し、再依頼する。`serialport` を使えない理由があれば D-09 の前提を見直す |
| 経路 | シリアル経路を実施しなかった | `SerialTransport` の振る舞いと、`serialport` が Mac で動くかは未確認のまま |
| 経路 | Wi-Fi 経路を実施しなかった | `HttpTransport` の成否の判定方法は未決定のまま |
| （全体） | 手順どおりに進まなかった・判定できない | 再依頼する。それまで 06 の 2節と D-09 の前提は保留のまま |

## 結果（実施者が記入）

貼る前に [public-repo.md](../docs/rules/public-repo.md) に従って伏せる（IP → `<IP>`、ポート名 → `<PORT>` など）。
コミットの前に `git diff --staged` を読み、`scripts/check-public.sh --staged` を通す。

### 環境

- ファームウェアのバージョン:
- 接続手段:
- Mac の OS・CPU:
- `rustc --version`:

### 出力・観察

```
```

### 判定

### 気づいたこと
