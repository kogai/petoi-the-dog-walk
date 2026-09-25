# E00: トークン送信と応答の形式

- 状態: 依頼中
- 依頼日: 2026-09-25
- 実施日:
- 実施者:

## 目的

Mac から Bittle へトークンを送ったとき、Bittle が何を返すか（エコーの有無・形式・時間）を記録する。
`FakeTransport` の振る舞いと、`SerialTransport`／`HttpTransport` の実装をこれに合わせる。
公式 Python API（PetoiRobot）を依存に入れるか（D-08）の材料にもする。

接続手段は問わない。**手持ちの経路だけ**で行う（全部を試す必要はない）。

## 必要なもの

- 機材: Bittle（標準ファーム）、手持ちの接続手段（USB アダプタ / Bluetooth ドングル / ESP8266 のどれか）
- ソフトウェア: Python 3.11 以上、`pip install pyserial`（シリアル経路の場合）、`curl`（Wi-Fi 経路の場合）
- 所要時間の目安: 30分

## 安全上の注意

- Bittle を平らな床に置く。歩行トークンを送る前に、前方 1m に物が無いことを確認する。
- 止めるときは `d` を送る。

## 手順

### A. シリアル経路（USB / Bluetooth）

1. ポート名を調べる。
   ```sh
   ls /dev/cu.*
   ```
2. pyserial 付属の端末で接続する（`<PORT>` は 1 で見つけたもの）。
   ```sh
   python -m serial.tools.miniterm <PORT> 115200 --eol LF
   ```
3. 接続直後に何か表示されたら、そのまま記録する。
4. 次のトークンを1つずつ入力し、Enter を押す。各トークンの後、動作が終わるまで待つ。
   `ksit` → `kbalance` → `khi` → `kwkF` → （2秒後）`kbalance` → `d`
5. 各トークンについて、画面の表示と、Bittle の動きを記録する。
6. `ksit` を送った直後にもう一度 `ksit` を送り、何が起きるか記録する（同じトークンの連続送信）。
7. `kwkF` を送ったあと、何も送らずに 5 秒待ったとき、歩き続けるか止まるかを記録する。
8. 存在しないトークン `kzzz` を送り、何が返るか記録する。
9. 任意: 改行を `--eol CRLF` に変えて 4 を繰り返し、違いがあれば記録する。
10. 終了は `Ctrl-]`。

任意（時間の記録）: 次のスクリプトで、送信から応答までの時間を記録できる（**未実行**。動かなければ手順 A だけでよい）。

```python
# e00_timing.py  使い方: python e00_timing.py <PORT> ksit
import sys
import time

import serial

port, token = sys.argv[1], sys.argv[2]
with serial.Serial(port, 115200, timeout=0.1) as s:
    time.sleep(2)  # 接続時のリセット待ち
    s.reset_input_buffer()
    t0 = time.monotonic()
    s.write((token + "\n").encode())
    while time.monotonic() - t0 < 5:
        line = s.readline()
        if line:
            print(f"{(time.monotonic() - t0) * 1000:7.1f} ms  {line!r}")
```

### B. Wi-Fi 経路（ESP8266）

1. Bittle の IP アドレスを確認する（公式手順どおり）。
2. 次を順に実行する（動作名はトークンではなく名前で送る。F-B5）。
   ```sh
   curl -sv "http://<IP>/action?name=sit"
   curl -sv "http://<IP>/action?name=balance"
   curl -sv "http://<IP>/action?name=forward"
   curl -sv "http://<IP>/action?name=stop"
   curl -sv "http://<IP>/action?name=zzz"
   ```
3. 各コマンドの HTTP ステータス、応答本文、Bittle の動きを記録する。

### C. 公式 Python API（任意）

PetoiRobot を使ったことがあれば、`autoConnect()` と `sendSkillStr('ksit', 3)` を試し、
インストール方法と、使ってみた感想（依存に入れてよさそうか）を書く。

## 記録すること

- [ ] 使った接続手段と、ファームのバージョン（起動時の表示などで分かれば）
- [ ] 接続直後の表示
- [ ] 各トークンへの応答（そのまま貼る）と動き
- [ ] 同じトークンを連続で送ったときの挙動（手順 A-6）
- [ ] 歩行トークンは送り続ける必要があるか（手順 A-7）
- [ ] 不正なトークンへの応答（手順 A-8 / B）
- [ ] 改行コードの違いの有無（任意）
- [ ] 応答までの時間（任意）

## 判定基準

| 結果 | 設計への影響 |
|---|---|
| トークンごとに決まった応答が返る | `SerialTransport.send` は応答を待って成否を判定する |
| 応答が無い／不定 | 送りっぱなしにして、送信間隔の下限だけで制御する |
| 歩行が継続する | 同じトークンを連続で送らない（06 の設計案どおり） |
| 歩行が止まる | 継続中は一定間隔で再送する設計に変える |

## 結果（実施者が記入）

### 環境

- ファームウェアのバージョン:
- 接続手段:
- Mac の OS / Python のバージョン:

### 出力・観察

```
```

### 判定

### 気づいたこと
