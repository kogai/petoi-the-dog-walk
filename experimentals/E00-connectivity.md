# E00: トークン送信と応答の形式

- 状態: 依頼中
- 依頼日: 2026-09-25
- 実施日:
- 実施者:

## 目的

Mac から Bittle へトークンを送ったとき、Bittle が何を返すか（エコーの有無・形式・時間）を記録する。
`fakeTransport` の振る舞いと、`serialTransport`／`httpTransport` の実装をこれに合わせる。
あわせて、本体で使う予定の npm パッケージ `serialport` が Mac で動くかを確かめる。

接続手段は問わない。**手持ちの経路だけ**で行う（全部を試す必要はない）。

## 必要なもの

- 機材: Bittle（標準ファーム）、手持ちの接続手段（USB アダプタ / Bluetooth ドングル / ESP8266 のどれか）
- ソフトウェア（シリアル経路）: Node.js 22.18 以上、pnpm。このリポジトリを clone して `pnpm install` しておく
- ソフトウェア（Wi-Fi 経路）: `curl`
- 所要時間の目安: 30分

## 安全上の注意

- Bittle を平らな床に置き、周囲 2m 以内に物・段差が無いことを確認する。台の上では行わない。
- 歩行トークン（`kwkF`）の観察は、**手で胴を持って足を浮かせた状態**で行ってもよい（歩き続けるかどうかは足の動きで分かる）。
- 止めるときは `d` を送る。止まらなければ電源を切る。
- 下のスクリプトは、終わるときと `Ctrl-C` を押したときに `d` を送る。それでも動き続けたら電源を切る。

## 手順

### A. シリアル経路（USB / Bluetooth）

0. Bluetooth の場合は、先に Mac とペアリングしておく（[Petoi の Bluetooth の説明](https://docs.petoi.com/)の該当ページを参照）。
1. ポート名を調べる（ポート名は機器名を含むことがあるので、結果には `<PORT>` と書く）。
   ```sh
   ls /dev/cu.*
   ```
2. 接続を確かめる。接続直後に何か表示されたら、そのまま記録する。
   ```sh
   node scripts/experiments/e00-serial-probe.ts <PORT> kbalance
   ```
   スクリプトは、送ったもの（`->`）と受け取ったもの（`<-`）を、開始からのミリ秒付きで表示する。
   最後に必ず `d` を送る。**未実行**のスクリプトなので、エラーが出たらそのまま結果に貼ればよい。
3. 基本の動作（各トークンのあと 3 秒ずつ待つ。`kwkF` は 3 秒後の `kbalance` で止める）。
   ```sh
   node scripts/experiments/e00-serial-probe.ts <PORT> ksit kbalance khi kwkF kbalance
   ```
4. 同じトークンの連続送信（0.2 秒あけて2回）。
   ```sh
   E00_WAIT_MS=200 node scripts/experiments/e00-serial-probe.ts <PORT> ksit ksit
   ```
5. 歩行は続くか（`kwkF` を送って 5 秒間なにも送らない）。
   ```sh
   E00_WAIT_MS=5000 node scripts/experiments/e00-serial-probe.ts <PORT> kwkF
   ```
6. 存在しないトークン。
   ```sh
   node scripts/experiments/e00-serial-probe.ts <PORT> kzzz
   ```
7. 任意: 改行コードを CRLF にして 3 を繰り返し、違いがあれば記録する。
   ```sh
   E00_EOL=CRLF node scripts/experiments/e00-serial-probe.ts <PORT> ksit kbalance
   ```
8. 各手順の出力をそのまま貼り、Bittle の動きを書き添える。

スクリプトが動かない場合は、Arduino IDE のシリアルモニター（115200、改行は「LF」）で同じトークンを1つずつ送り、表示と動きを記録してもよい。

### B. Wi-Fi 経路（ESP8266）

1. Bittle の IP アドレスを確認する（[WiFi module ESP8266](https://docs.petoi.com/communication-modules/wifi-esp8266.md) の手順）。結果には `<IP>` と書く。
2. 次を順に実行する（動作名はトークンではなく名前で送る。F-B5）。
   ```sh
   curl -sv "http://<IP>/action?name=sit"
   curl -sv "http://<IP>/action?name=balance"
   curl -sv "http://<IP>/action?name=forward"
   curl -sv "http://<IP>/action?name=stop"
   curl -sv "http://<IP>/action?name=zzz"
   ```
3. 各コマンドの HTTP ステータス、応答本文、Bittle の動きを記録する。

## 記録すること

- [ ] 使った接続手段と、ファームのバージョン（接続直後の表示に出ることが多い。出なければ [Serial Protocol](https://docs.petoi.com/apis/serial-protocol.md) でバージョン表示のトークンを探す。無ければ「不明」でよい）
- [ ] 接続直後の表示
- [ ] 各トークンへの応答と時刻（スクリプトの出力をそのまま貼る）と動き（手順 A-3）
- [ ] 同じトークンを連続で送ったときの挙動（手順 A-4）
- [ ] 歩行トークンは送り続ける必要があるか（手順 A-5）
- [ ] 不正なトークンへの応答（手順 A-6 / B）
- [ ] 改行コードの違いの有無（任意、手順 A-7）
- [ ] スクリプトが動いたか。動かなければエラー全文（`serialport` が Mac で使えるかの判断材料）

## 判定基準

| 観察 | 結果 | 設計への影響 |
|---|---|---|
| 応答（A-3） | トークンごとに決まった応答が返る | `serialTransport.send` は応答を待って成否を判定する |
| 応答（A-3） | 応答が無い／不定 | 送りっぱなしにして、送信間隔の下限だけで制御する |
| 連続送信（A-4） | 動作が最初からやり直しになる | 同じトークンの連続送信を必ず抑止する |
| 連続送信（A-4） | 2回目は無視される | 抑止は必須ではない（無駄な通信を減らすためだけに行う） |
| 歩行（A-5） | 歩き続ける | 同じトークンを連続で送らない（06 の設計案どおり） |
| 歩行（A-5） | 止まる | 歩行中は一定間隔で再送する設計に変える |
| 不正トークン（A-6） | エラーの応答がある | `send` で検出して `Result` の失敗として返す |
| Wi-Fi（B） | 応答本文・ステータスで成否が分かる | `httpTransport` はそれで成否を判定する |
| Wi-Fi（B） | 常に同じ応答 | `httpTransport` は HTTP エラーだけを失敗とする |
| スクリプト（A） | 動いた | `serialTransport` は `serialport` で書く |
| スクリプト（A） | `serialport` の読み込みや接続で失敗した | 原因を調べる。ネイティブモジュールの問題なら、別の方法（Web Serial など）を検討する |

## 結果（実施者が記入）

### 環境

- ファームウェアのバージョン:
- 接続手段:
- Mac の OS / Node.js のバージョン:

### 出力・観察

```
```

### 判定

### 気づいたこと
