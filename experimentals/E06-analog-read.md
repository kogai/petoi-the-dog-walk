# E06: readAnalogValue(pin) が標準ファームで動くか

- 状態: 後回し（今は実施しなくてよい。手順は下書き）
- 依頼日:
- 実施日:
- 実施者:

## 目的

公式 Python API の `readAnalogValue(pin)` に当たる読み取りが、標準ファームでシリアルコマンドからできるかを確かめる（本体は TypeScript なので、Python API そのものは使わない。D-08）。Grove G2（A2/A3、F-B3）のセンサーを `sensing` に使えるかが決まる（D-02 の候補）。`readAnalogValue` は facts.md に無く **未確認**。依頼中に上げるときに判定基準を足す。

## 必要なもの

- Bittle、Grove のアナログセンサー（手持ちのもの）、PetoiRobot

## 手順（下書き）

1. センサーを G2 に接続する。
2. `readAnalogValue` で A2（または A3）を読み、センサーの状態を変えて値が変わるか記録する。

## 結果（実施者が記入）

```
```
