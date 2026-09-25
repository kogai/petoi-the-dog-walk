# 06. Bittle との通信

## 1. 確認済み

- ボーレート 115200（F-B4）。
- Wi-Fi（ESP8266）の公式サンプルは `http://<IP>/action?name=<動作名>` を受ける（F-B5）。
- Python API に `sendSkillStr()`、`autoConnect()` がある（F-B6）。

## 2. 設計案

`Transport` Protocol（[02](02-architecture.md)）の実装を3つ用意する。

| 実装 | 用途 | 依存 |
|---|---|---|
| `FakeTransport` | テスト。送ったトークンを記録するだけ | なし |
| `SerialTransport` | USB / Bluetooth。pyserial でトークンを書き込む | pyserial |
| `HttpTransport` | Wi-Fi。`/action?name=` に GET する | 標準ライブラリか httpx |

共通の決まり:

- 送る前に、トークンが許可リスト（F-T1 の表のうち `domain` で使うもの）にあるか確認する。無ければ送らない。
- 同じトークンを連続で送らない（歩行などは継続する前提。未確認 → E00）。
- 送信間隔の下限を設ける（既定値は E00 の結果で決める）。
- `close()` の前に必ず停止トークン `d` を送る。
- Mac 側に非常停止キーを用意する（押すと `d` を送ってループを抜ける）。

## 3. 未確認・実験

- E00: 各経路でトークンを送ったときの応答（エコーの有無・形式・所要時間）。`FakeTransport` の振る舞いをこれに合わせる。
- E04: Wi-Fi 経由で、決まった動作名以外（関節の直接指定など）を送れるか。
- 公式 Python API（PetoiRobot）を依存に入れるか、pyserial で直接書くか（D-08）。
