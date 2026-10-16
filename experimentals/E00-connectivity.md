# E00: トークンを送ったときの応答の形式

> **書き直し待ち。** この手順書は [README](README.md) の「人が実行できる手順にする」より前に書いた。書き直すまで、A-4・A-5・A-6・A-8 の Bittle の動きの観察は実施せず、判定基準の連続送信・歩行の行は使わない（出力だけで判定する手順は実施してよい）。胴を持って足を浮かせる手順（A-4・A-6・A-8・B-2）は、Bittle を持つ人とキーボードを操作する人の **2人で行うときだけ** 実施する。

- 状態: 依頼中
- 依頼日: 2026-09-25
- 実施日: 2026-10-16（シリアル経路の一部。A-5・A-6 は判定できず、B は未実施）
- 実施者: kogai

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
  - [mise](https://mise.jdx.dev/)（[公式の手順](https://mise.jdx.dev/getting-started.html)）。入れ方で、シェル（zsh）で有効にするコマンドが違う。どちらも、実行した後に新しいターミナルを開く
    - Homebrew がある: `brew install mise` の後、`echo 'eval "$(mise activate zsh)"' >> ~/.zshrc`
    - Homebrew が無い: `curl https://mise.run | sh` の後、`echo 'eval "$(~/.local/bin/mise activate zsh)"' >> ~/.zshrc`（`~/.local/bin` は既定では PATH に無いため、場所を書く）
  - このリポジトリを clone し、そのフォルダで `mise trust && mise install rust` を実行しておく（Rust が `mise.toml` の版で入る。この実験に要るのは Rust だけ。初回は数分〜十数分かかる）
- ソフトウェア（Wi-Fi 経路）: `curl`（macOS に入っている）
- 所要時間の目安: 40分（ソフトウェアの準備を除く）

## 安全上の注意

- Bittle を平らな床に置き、周囲 2m 以内に物・段差が無いことを確認する。台の上では行わない。
- **歩行のトークン（`kwkF`、`forward`）を送るときは、手で胴を持って足を浮かせる。** 歩き続けるかどうかは足の動きで分かる。
- 道具（`e00-serial-probe`）は、終わるとき・エラーのとき・`Ctrl-C` を押したとき・ターミナルを閉じたときに停止トークン `d` を送る。
  `d` を送れなかったときは `!! STOP TOKEN NOT SENT` と表示する。**その表示が出たら、すぐ電源を切る。**
  `d` でシリアル経由で止まるかどうか自体が、この実験で確かめること（**未確認**）。止まらなければ電源を切る。
- **`Ctrl-Z` は使わない**（道具が一時停止し、`d` が送られないまま動き続ける）。止めるときは `Ctrl-C`。
- **`Ctrl-S` は押さない**。ターミナルの表示が止まり、道具の送信も止まる（Bittle は最後の動作を続ける）。表示が止まったら、**まず `Ctrl-C` を押し**（このとき道具は `d` を送り、それ以降のトークンは送らない）、それから `Ctrl-Q` で表示を戻す。
- 道具は、予定の終わり（`E00_ASK` を付けたときは、問いを見る 1.5 秒の後）から 3 秒たっても `d` を送っていなければ、自分で `d` を送り、それ以降のトークンは送らない。この `d` は送った瞬間には画面に出ない（終わるときに `the watchdog sent the stop token` と表示する）。**足の動きで止まったかを確かめ**、止まらなければ電源を切る。
- `Ctrl-C` と、上の 3 秒の見張りが送った `d` は、ログに `->` の行として残らない。そのため、ログに対応する `->` の無い `<-` の行が出ることがある。
- 起動時に `no independent stop path, refusing to run` と表示されたら、道具は `d` だけを送って終わる（トークンは送らない）。その回は実施できなかったとして記録する。
- `E00_ASK` を付けたときは、最後の待ち時間の後も、問いを見るための **1.5 秒間は動き続ける**。その間に止めたければ `Ctrl-C`。答えは `d` が送られた後に入力する。答えの入力に時間の制限は無い。**足が止まったのを確かめてから** Bittle を置き、答えを打つ。止まらなければ、答えずに電源を切る（`d` で止まるかは**未確認**）。
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

観察の問い（任意。道具の機能。実機ではまだ動かしていない＝**未実行**。この機能を使う手順は、後の見直しで足す）:

- コマンドの前に `E00_ASK='<問い>'` を付ける（単引用符で囲む。二重引用符だと `!` が zsh に解釈される）。問いは120文字まで。長すぎる・制御文字を含む問いは、道具がエラーで断る。`E00_ASK` は `export` しない（以降のすべての実行に問いが付いてしまう）。
- トークンを最後まで送り、最後の待ち時間が終わった瞬間に `>>> LOOK NOW: <問い>` と表示する。ベルも鳴らすが、ターミナルの設定によっては音が鳴らない（画面が光るだけ、または何も起きない）。音に頼らず、画面を見て表示を待つ。
- 表示が出たら、そのときの Bittle を見る。1.5 秒後に `d` が送られ、`>>> answer y (yes), n (no) or ? (could not tell), then press Enter:` と表示される。**この表示が出てから**、`y`（はい）・`n`（いいえ）・`?`（分からない）のどれかを入力して Enter を押す。問いより前に打った行は答えとして使われない。
- ログには `?? <問い>` と、`answer: yes`・`answer: no`・`answer: unknown`（`?` のとき）・`answer: none`（入力が終わった、または不正な答えが5回続いた）・`answer: interrupted`（答えを待つ間に `Ctrl-C`）のどれかが残る。
- 途中で `Ctrl-C` を押したときや送信に失敗したときは、問いは出ない。問いが出ている 1.5 秒の間に `Ctrl-C` を押したときは、`d` は送られるが、ログに `answer:` の行は残らない（`?? <問い>` だけになる）。

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

- ファームウェアのバージョン: 不明。接続直後の表示に `N_260806` と出る（バージョンを表す文字列かは**未確認**）
- 接続手段: USB アダプタ（シリアル）。Bittle の組み立ては途中で、脚は付けていない（首と前脚付近のサーボは付いている）
- Mac の OS・CPU: 記録なし（「道具」の行の前提にあたる macOS の版は不明。分かれば追記する）
- `rustc --version`: 記録なし（`mise.toml` の版を `mise install` で入れた）
- 実施した経路: シリアルだけ。Wi-Fi（B）は実施していない（ESP8266 のモジュール ESP-WROOM-02D は手元にあるが、準備していない）
- このリポジトリのコミット: 記録なし（2026-10-16 時点の main。`e00-serial-probe` に `E00_ASK`（#14）が入る前の版）

この結果は、[README](README.md) の「人が実行できる手順にする」より前に実施した。そのルールで外した観察（A-4・A-5・A-6・A-8 の Bittle の動き）は、記録には残すが判定には使わない。

### 出力・観察

実施者が会話に貼った出力を、エージェントが転記した。ポート名は `<PORT>` に、ホームのパスは伏せてある。「観察」「補足」は実施者の言葉、「（エージェントの読み）」はエージェントの推測で、事実ではない。

#### A-1・A-2

- A-1: ビルドは通った（エラーの報告なし）。
- A-2: Bittle をつなぐ前の `--list` には、Mac 本体のポートしか出なかった。USB でつないで電源を入れると、`usbserial` を含むポートが増えた。手順書には「先につないで電源を入れる」が書かれていない（手順書の不足）。

#### A-3 1回目（E00_SETTLE_MS 既定）
```
$ E00_LOG=experimentals/raw/e00-a3.txt ./target/release/e00-serial-probe <PORT> kbalance
  1430.3 ms  <- "\x00"
  1495.7 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  1497.4 ms  <- "\n"
  2000.3 ms  -> "kbalance"
  3350.8 ms  <- "IMU\r\n"
  3955.7 ms  <- "Servo:"
  3973.0 ms  <- "24.70\r\n"
  3990.3 ms  <- "Skill\r\n"
  5002.4 ms  -> "d"
```
観察: Bittle が動いた（首を振った）。

#### A-3 2回目（E00_SETTLE_MS=5000）
```
  1437.1 ms  <- "\x00"
  1502.5 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  1504.4 ms  <- "\n"
  3358.0 ms  <- "IMU\r\n"
  3962.6 ms  <- "Servo:"
  3980.2 ms  <- "24.70\r\n"
  3997.5 ms  <- "Skill\r\n"
  5003.5 ms  -> "kbalance"
  5514.7 ms  <- "Ready!\r\n"
  6398.6 ms  <- "k\r\n"
  8006.0 ms  -> "d"
  8018.2 ms  <- "\n\r\n"
```
観察: 首を素早く右に振った後、ゆっくり戻した（タイミングは未記録）。

#### A-3 3回目（E00_SETTLE_MS=8000）
```
  1440.6 ms  <- "\x00"
  1506.0 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  1507.9 ms  <- "\n"
  3361.6 ms  <- "IMU\r\n"
  3962.6 ms  <- "Servo:"
  3983.5 ms  <- "24.70\r\n"
  4001.0 ms  <- "Skill\r\n"
  5517.7 ms  <- "Ready!\r\n"
  5650.6 ms  <- "g\r\nd\r\n"
  8004.0 ms  -> "kbalance"
  8033.9 ms  <- "G\r\n"
  8793.7 ms  <- "k\r\n"
 11004.9 ms  -> "d"
 11016.5 ms  <- "\n\r\n"
```

#### A-4 1回目（E00_SETTLE_MS が効いていない様子: 最初の送信が 2004 ms）
```
$ E00_LOG=experimentals/raw/e00-a4.txt ./target/release/e00-serial-probe <PORT> ksit kbalance khi kwkF kbalance
  1440.7 ms  <- "\x00"
  1506.1 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  1508.0 ms  <- "\n"
  2004.6 ms  -> "ksit"
  3361.2 ms  <- "IMU\r\n"
  5008.1 ms  -> "kbalance"
  7291.8 ms  <- "\x00"
  7356.9 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  7358.8 ms  <- "\n"
  8010.3 ms  -> "khi"
  9212.2 ms  <- "IMU\r\n"
  9817.4 ms  <- "Servo:"
  9834.7 ms  <- "24.70\r\n"
  9851.9 ms  <- "Skill\r\n"
 11010.7 ms  -> "kwkF"
 11370.0 ms  <- "Ready!\r\n"
 11458.0 ms  <- "?hi\nkwkF\r\n"
 14014.4 ms  -> "kbalance"
 14308.8 ms  <- "k\r\n"
 17016.0 ms  -> "d"
 17028.7 ms  <- "\n\r\n"
```
観察: 首が動いた後に、前脚のサーボの軸（脚は未装着）が動いたように見えた（実施者の記憶。どのトークンの後かは未記録）。途中の再起動（7292 ms）の原因は不明。
補足（実施者）: 「少し回ってすぐ止まったように見えた」（kwkF の後のサーボの軸の動きについてと思われる。どの実行のことかは不明）。

#### A-4 2回目（E00_SETTLE_MS=8000 を echo で確認済み。電池は「付いていた」と実施者）
```
$ E00_LOG=experimentals/raw/e00-a4.txt ./target/release/e00-serial-probe <PORT> ksit kbalance khi kwkF kbalance
  1442.6 ms  <- "\x00"
  1508.0 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  1509.9 ms  <- "\n"
  3363.7 ms  <- "IMU\r\n"
  3968.0 ms  <- "Servo:"
  3985.5 ms  <- "24.70\r\n"
  4002.7 ms  <- "Skill\r\n"
  5520.9 ms  <- "Ready!\r\n"
  5654.0 ms  <- "g\r\nd\r\nLow power:0V\r\n"
  8005.2 ms  -> "ksit"
  8035.1 ms  <- "G\r\n"
  8937.3 ms  <- "k\r\n"
 11008.2 ms  -> "kbalance"
 11624.3 ms  <- "k\r\nLow power:0V\r\n"
 14008.9 ms  -> "khi"
 17011.7 ms  -> "kwkF"
 18782.5 ms  <- "k\r\n"
 20015.1 ms  -> "kbalance"
 20345.5 ms  <- "k\r\n"
 23016.2 ms  -> "d"
 23031.2 ms  <- "\n\r\n"
```
（エージェントの読み）"Low power:0V" が出ている。電池から電力が来ていない可能性（推測）。この回は電源の条件が不確かなので、判定には使わない。
補足（実施者）: Low power:0V は、電池の確認でボタンを長押しして電池の電源が切れたため。電池を入れ直して途中から再開した（いつ電力が戻ったかは出力からは分からない）。この回は判定に使わない。
補足（実施者）: A-4 では首と前脚付近のサーボが動いていた。トークンごとの区別など、それ以上の正確な観察は目視では不可能（手順書の改善点）。

#### A-4 3回目（判定に使う回。E00_SETTLE_MS=8000、電池の電源あり、Low power・再起動なし）
```
$ E00_LOG=experimentals/raw/e00-a4.txt ./target/release/e00-serial-probe <PORT> ksit kbalance khi kwkF kbalance
  1444.5 ms  <- "\x00"
  1509.9 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  1511.8 ms  <- "\n"
  3365.6 ms  <- "IMU\r\n"
  3970.1 ms  <- "Servo:"
  3987.6 ms  <- "24.70\r\n"
  4004.8 ms  <- "Skill\r\n"
  5518.8 ms  <- "Ready!\r\n"
  5649.8 ms  <- "g\r\nd\r\n"
  8001.1 ms  -> "ksit"
  8031.0 ms  <- "G\r\n"
  8897.9 ms  <- "k\r\n"
 11001.9 ms  -> "kbalance"
 11636.5 ms  <- "k\r\n"
 14006.1 ms  -> "khi"
 17008.3 ms  -> "kwkF"
 18990.1 ms  <- "k\r\n"
 20013.6 ms  -> "kbalance"
 20434.8 ms  <- "k\r\n"
 23018.9 ms  -> "d"
 23029.5 ms  <- "\n\r\n"
```
観察: 首と前脚付近のサーボが動いた。トークンごとの区別は目視では不可能。
（エージェントの読み）k は動作の完了で返る。khi は約5秒かかり、18990 ms の k は khi の完了（送信から約4.98秒）。kwkF（歩行）は終わらないので k を返さない。A-5 で khi 単独の k の時刻を測って確かめる。

#### A-5（変形版。khi 単独と khi×2 を比較。E00_FINAL_WAIT_MS=8000）
```
$ E00_FINAL_WAIT_MS=8000 E00_LOG=experimentals/raw/e00-a5-single.txt ./target/release/e00-serial-probe <PORT> khi
  1444.2 ms  <- "\x00"
  1509.6 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  1511.5 ms  <- "\n"
  3364.9 ms  <- "IMU\r\n"
  7302.4 ms  <- "\x00"
  7367.7 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  7369.7 ms  <- "\n"
  8001.6 ms  -> "khi"
  9223.1 ms  <- "IMU\r\n"
 13160.4 ms  <- "\x00"
 13225.9 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
 13228.0 ms  <- "\n"
 15081.5 ms  <- "IMU\r\n"
 15686.3 ms  <- "Servo:"
 15703.6 ms  <- "24.70\r\n"
 15720.8 ms  <- "Skill\r\n"
 16003.2 ms  -> "d"
```
（エージェントの読み）起動中に2回再起動した（7302 ms、13160 ms）。どちらも IMU の行の約3.9秒後で、トークンを送る前にも起きている。A-4 1回目の再起動（7292 ms）と同じ位置。khi の時間は測れなかった。

```
$ E00_WAIT_MS=200 E00_FINAL_WAIT_MS=8000 E00_LOG=experimentals/raw/e00-a5.txt ./target/release/e00-serial-probe <PORT> khi khi
  1439.1 ms  <- "\x00"
  1504.7 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  1506.6 ms  <- "\n"
  3359.9 ms  <- "IMU\r\n"
  3964.8 ms  <- "Servo:"
  3982.2 ms  <- "24.70\r\n"
  3999.2 ms  <- "Skill\r\n"
  5517.3 ms  <- "Ready!\r\n"
  6405.7 ms  <- "g\r\nd\r\n"
  8004.4 ms  -> "khi"
  8034.3 ms  <- "G\r\n"
  8206.3 ms  -> "khi"
 15215.2 ms  <- "k\r\n"
 16211.6 ms  -> "d"
 16221.5 ms  <- "\n\r\n"
```
（エージェントの読み）再起動なし。k は1つだけで、1回目の khi から 7.2 秒後。khi 単独の時間が測れていないので、やり直しか無視かは判定できない。

#### 中止（2026-10-16）
実施者の判断で E00 を中止した。理由: 手順書が「A の後に B・C を目で観察する」形で、人の注意力と記憶力で実行できる前提になっていない。この時点では A-6〜A-8 と B は未実施。

#### 再開（実施者の判断で E00 を最後まで行う）。電池は「ok」と実施者。

#### A-6
```
$ E00_FINAL_WAIT_MS=5000 E00_LOG=experimentals/raw/e00-a6.txt ./target/release/e00-serial-probe <PORT> kwkF
  1444.4 ms  <- "\x00"
  1509.7 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  1511.6 ms  <- "\n"
  3365.1 ms  <- "IMU\r\n"
  3969.9 ms  <- "Servo:"
  3987.4 ms  <- "24.70\r\n"
  4004.5 ms  <- "Skill\r\n"
  5518.8 ms  <- "Ready!\r\n"
  5650.8 ms  <- "g\r\nd\r\nLow power:6V\r\n"
  8000.1 ms  -> "kwkF"
  8029.6 ms  <- "G\r\n"
  8366.0 ms  <- "k\r\n"
 11511.6 ms  <- "Low power:6V\r\n"
 13005.2 ms  -> "d"
 13026.4 ms  <- "\n\r\n"
```
観察（実施者）: 数秒ほど首を振りながら、前脚付近のサーボも動いていた（5秒後にまだ動いていたかは不明）。
（エージェントの読み）"Low power:6V" が起動直後と歩行中に出ている。ファームが電池の電圧を低いと判断している。電池の電圧不足で歩行が止まった可能性があるので、A-6 は判定に使えない。kwkF にも 0.37 秒で k が返った（A-4 で立てた「k は完了時に返る」仮説と合わない）。
実施者の指摘: 一度に一つだけ確認するには、道具のコード変更が要る。

#### A-7
```
$ E00_LOG=experimentals/raw/e00-a7.txt ./target/release/e00-serial-probe <PORT> kzzz
  1439.8 ms  <- "\x00"
  1505.3 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  1507.3 ms  <- "\n"
  3360.6 ms  <- "IMU\r\n"
  3965.5 ms  <- "Servo:"
  3983.0 ms  <- "24.70\r\n"
  4000.3 ms  <- "Skill\r\n"
  5514.3 ms  <- "Ready!\r\n"
  5646.2 ms  <- "g\r\nd\r\nLow power:6V\r\n"
  8002.6 ms  -> "kzzz"
  8034.0 ms  <- "G\r\n?zzz\r\nk\r\n"
 11005.1 ms  -> "d"
 11016.2 ms  <- "\n\r\n"
 11506.7 ms  <- "Low power:6V\r\n"
```
（エージェントの読み）エラーの応答がある（"?zzz"。先頭の k を除いた名前に ? を付けて返す）。31 ms 以内。→ send で検出して Err を返せる。
（A-4 1回目の "?hi" も、起動中に k が欠けて未知の命令になった同じ形と読める）
（エージェントの読み）電池: Low power:6V が出たまま。出力だけで判定する手順なので、影響は小さいと見る。

#### A-8（CRLF）
```
$ E00_EOL=CRLF E00_LOG=experimentals/raw/e00-a8.txt ./target/release/e00-serial-probe <PORT> ksit kbalance khi kwkF kbalance
  1439.2 ms  <- "\x00"
  1504.6 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  1506.6 ms  <- "\n"
  3360.0 ms  <- "IMU\r\n"
  7294.1 ms  <- "\x00"
  7359.2 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  7361.4 ms  <- "\n"
  8003.4 ms  -> "ksit"
  9214.6 ms  <- "IMU\r\n"
  9819.3 ms  <- "Servo:"
  9837.1 ms  <- "24.70\r\n"
  9854.3 ms  <- "Skill\r\n"
 11008.6 ms  -> "kbalance"
 11366.5 ms  <- "Ready!\r\n"
 11462.5 ms  <- "?sit\r\nkbalance\r\r\nLow power:6V\r\n"
 14009.6 ms  -> "khi"
 14025.7 ms  <- "?hi\r\r\n"
 17012.7 ms  -> "kwkF"
 17025.1 ms  <- "?wkF\r\r\n"
 17366.6 ms  <- "Low power:6V\r\n"
 20017.9 ms  -> "kbalance"
 20030.9 ms  <- "?balance\r\r\n"
 23023.3 ms  -> "d"
 23033.1 ms  <- "\r\r\n"
```
（エージェントの読み）起動中に再起動（7294 ms。これまでと同じ位置）。起動が終わった後（Ready! の後）に送った khi・kwkF・kbalance は、すべて "?<名前>\r" の形で未知の命令として返った。CR がトークンの一部として扱われたと読める。
（エージェントの読み）LF と CRLF で違いがある。CRLF ではトークンが通らない → 改行は LF に固定する（判定基準の表は「違いがある → 改行コードを設定にする」だが、CRLF が使えないので設定にする意味が無い。設計への反映のときに扱う）。

#### A-8 2回目（CRLF。再起動なし）
```
  1443.8 ms  <- "\x00"
  1509.3 ms  <- "k\r\n\n* Start *\r\nBittle\r\nN_260806\r"
  1511.3 ms  <- "\n"
  3364.5 ms  <- "IMU\r\n"
  3969.7 ms  <- "Servo:"
  3986.6 ms  <- "24.70\r\n"
  4004.0 ms  <- "Skill\r\n"
  5521.9 ms  <- "Ready!\r\n"
  5655.8 ms  <- "g\r\nd\r\nLow power:6V\r\n"
  8002.2 ms  -> "ksit"
  8034.2 ms  <- "G\r\n?sit\r\r\nk\r\n"
 11007.9 ms  -> "kbalance"
 11023.7 ms  <- "?balance\r\r\nk\r\n"
 11515.5 ms  <- "Low power:6V\r\n"
 14012.5 ms  -> "khi"
 14028.1 ms  <- "?hi\r\r\nk\r\n"
 17014.9 ms  -> "kwkF"
 17027.0 ms  <- "?wkF\r\r\nk\r\n"
 20019.1 ms  -> "kbalance"
 20032.2 ms  <- "?balance\r\r\nk\r\n"
 23020.9 ms  -> "d"
 23033.0 ms  <- "\r\r\n"
```
（エージェントの読み）CRLF では全トークンが "?<名前>\r" で未知扱い。LF に固定する。
（エージェントの読み）未知のトークンにも "k" が続く（16 ms 程度）。k の意味は**未確認**で、読みは2通りある。(a) k で始まる命令を受け付けたしるし。(b) 既知の姿勢・動作は終わったときに k を返し、歩行（終わらない動作）と未知のトークンはすぐに返す。A-4 で立てた (b) の仮説は、ここでは取り下げない。

### 判定

判定基準の表のうち、出力だけで判定する行を使う。連続送信（A-5）と歩行（A-6）の行は、上のルールで外したので使わない。

| 観察 | 結果 | 設計への影響 |
|---|---|---|
| 応答（A-3・A-4） | k の返り方（送信からの時間）: `ksit` +897 ms、`kbalance` +635 ms／+421 ms、`khi` は 3 秒以内には無し（A-4 3回目。その後に来た k（`khi` から約 5.0 秒後）が `khi` のものかは不明。A-5 の `khi` 2回では、2回目の送信の 7.0 秒後に k が1つ）、`kwkF` +366 ms、未知のトークン（`kzzz`、CRLF の各トークン）16〜32 ms。k の意味は**未確認**（上の2通りの読み） | **保留。** 「決まった応答が返る」か「決まっていない」かは、k の意味が分かるまで選ばない。書き直した手順で、トークンごとの k の時刻を測ってから決める |
| 不正なトークン（A-7） | エラーの応答がある（`?zzz`。先頭の `k` を除いた名前に `?` を付けて返す。約 31 ms） | `send` で検出して `Err` を返す（判定基準どおり）。そのためには、送った後に少なくとも短い時間は応答を読む必要がある。上の「応答」の行が送りっぱなしになった場合とどう両立させるか（`?<名前>` だけを短い時間待つ、など）は、反映の PR で決める |
| 改行コード（A-8） | LF と CRLF で違いがある。CRLF では、どのトークンも `?<名前>\r` と返り、通らない | 判定基準の表は「改行コードを設定にする」だが、CRLF では通らないので、設定にせず LF に固定するのが妥当（設計への反映の PR で決める） |
| 道具（A-1〜A-8） | ビルドでき、実機で動いた | `SerialTransport` は `serialport` クレートで書く（判定基準どおり） |
| 経路 | Wi-Fi 経路を実施しなかった | `HttpTransport` の成否の判定方法は未決定のまま |
| 連続送信（A-5）・歩行（A-6） | 判定しない（ルールで外した。A-6 は電池の電圧が低い状態でもあった） | 書き直した手順書で再実施する |

「（全体）手順どおりに進まなかった」に当たるのは、A-5（変形版で行った）・A-6・B。そこは書き直した手順で再依頼する。A-3・A-4・A-7・A-8 の出力による判定は有効とする。一部だけ実施したので、状態は「依頼中」のままにする。

### 気づいたこと

- **どの実行でも、開始直後に起動の表示が出る**（ポートを開いたことによる再起動と推測。**未確認**）。起動の表示は `Ready!` で終わり、約 5.5 秒かかる。既定の `E00_SETTLE_MS`（2000）では足りず、起動中に送ったトークンが崩れた（`khi` → `?hi`）。8000 で足りた。
- 起動の最後に、ファームが自分で `g` と `d` を出す（何も送っていないのに表示される）。意味は**未確認**。
- 起動直後の最初の `k` トークンには、先に `G` が返る。意味は**未確認**。
- 電池の電圧が低いと `Low power:6V` と表示される。電池の電源が切れていると `Low power:0V`。
- 電池の電圧が低いとき、起動の途中（`IMU` の行の約 3.9 秒後）で再起動することが繰り返しあった（A-4 1回目、A-5 1回目、A-8 1回目）。電源の不足によると推測する（**未確認**）。
- 手順書の不足: A-2 の「先につなぐ」、待ち時間の設定、観察の手順が人の注意力・記憶力に合っていないこと（これを受けて #14〜#16 で道具とルールを直した）。
