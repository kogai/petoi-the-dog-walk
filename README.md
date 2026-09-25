# Bittle ペットボット計画（Jev 理性層 + ハエ脳 反射層）

初代 Petoi Bittle（NyBoard）を、Mac から無線で動かす計画の README。
「理性的反応」を TypeSafe の Jev、「本能的反射」を maleCNS コネクトームのハエ脳シミュレーションが担当する。

このドキュメントは次の3種類を区別して書く。

- **確認済み**: 公式ドキュメントまたはリポジトリのコードを読んで確認した事実
- **設計案**: この計画での決め事（実装前・未実行）
- **未確認 / 実験**: 実機で試して確かめる項目

---

## 1. ゴール

- 非エンジニアが書いた自由文の「性格」と「スキル」を読ませて、Bittle の振る舞いを決める。
- 自由文による指示を、他の判断より優先する。
- 速い反応が必要な場面は、ハエ脳の反射層に任せる。
- ハエ脳は判断の中心ではない。無くても成立する構成にする。

## 2. 構成

```
 [人が書く自由文]  性格 / スキル / その場の指示
        │
        ▼
 ┌──────────────────────────────┐
 │ Mac (Python)                 │
 │  ・理性層: Jev (TypeSafe API) │──→ 次の動作を選ぶ
 │  ・反射層: ハエ脳 (後から追加)  │──→ 速い反応を出す
 │  ・優先順位: コード側で決める     │
 └──────────────┬───────────────┘
                │ 無線 (Wi-Fi 希望 / Bluetooth も可)
                ▼
      Bittle (NyBoard, ATmega328P)
       文字列トークンを受けて動く
```

- ハエ脳と Jev は PC 側で動かす。Bittle 側の MCU は ATmega328P（SRAM 2KB、Flash 32KB）で、シミュレーションは載らない。

## 3. 役割分担

| 層 | 担当 | 得意なこと | 出力 |
|---|---|---|---|
| 理性層 | Jev | 自由文（性格・スキル・指示）を読み、選択肢から選ぶ | 選択した動作、確率、確信度 |
| 反射層 | ハエ脳 | 100ms ごとの速い反応 | 動作（運動ニューロン群のうち最も強いもの） |

### 3.1 優先順位（設計案）

1. その場の自由文の指示が、今の状況に当てはまる場合は、それを最優先にする。
2. 当てはまらない間は、性格に基づく Jev の選択を使う。
3. 反射層の出力は、上の2つが無い場合のみ採用する。
4. 優先順位の判定は Jev に任せず、コード側で行う。
   （Jev の公式ドキュメントも、複数の判断を組み合わせる役割はコード側に置く考え方）

安全上の例外（転倒しそうなときなど）を優先順位の外に置くかどうかは、未決定。

## 4. 確認済みの事実

### 4.1 Bittle（初代・NyBoard）

- MCU は ATmega328P（16MHz、SRAM 2KB、Flash 32KB）。
- 基板上に 6軸 IMU（MPU6050）と赤外線受信機（VS1838B）がある。
- 拡張用 Grove ソケットが4つある（G1: I2C、G2: アナログ A2/A3、G3: D8/D9、G4: D6/D7）。
- 通信のボーレートは 115200。公式 Bluetooth ドングルがある。
- Wi-Fi は公式の ESP8266 モジュール（別売の拡張）で使える。公式サンプルは HTTP サーバーで、`http://<IP>/action?name=<動作名>` を受け、シリアルコマンドに変換して NyBoard へ渡す。
- Python API（PetoiRobot）に `sendSkillStr('ksit', 3)` や `autoConnect()` などがある。
- 頭部にカメラらしきものが付いている（型番は未確認。深掘りしない方針）。

### 4.2 動作トークン（公式 `actions.h` で確認済み）

| 動作名 | トークン |
|---|---|
| balance | `kbalance` |
| walk / forward | `kwkF` |
| forwardleft / forwardright | `kwkL` / `kwkR` |
| back | `kbk` |
| backleft / backright | `kbkL` / `kbkR` |
| trot | `ktrF` |
| crawl | `kcrF` |
| run | `krnF` |
| stepping | `kvtF` |
| sit | `ksit` |
| check | `kck` |
| buttup | `kbuttUp` |
| hello | `khi` |
| stretch | `kstr` |
| pee | `kpee` |
| pushup | `kpu` |
| lookup | `lu` |
| gyro（IMU 切替） | `g` |
| calibration | `c` |
| stop | `d` |

Python API の例では、立つ姿勢は `kup`。

### 4.3 Jev（TypeSafe）

- Jev は System One モデルで、テキストを生成しない。状態（`state`）と型付きの質問を受け取り、型付きの答えを返す。
- `state` は文字列・JSON オブジェクト・配列のどれでもよい。
- 質問は3種類ある。
  - Choice: 選択肢から1つ選ぶ（`choice`、`probabilities`、`confidence` を返す。選択肢は最大255個）
  - Score: 段階で評価する
  - Noul: yes/no の確率（`noul`、0〜1）を返す
- `instructions` と `criteria`（選択肢の説明）は、文字列・オブジェクト・配列で書ける。人が書いた文章をここに入れられる。
- 複数の質問を1回の呼び出しでまとめて出せる。質問を増やしても応答時間はほとんど変わらない。
- 公式 cookbook（skill suggestion）に、スキルの説明文を Choice の選択肢にして、どれを使うか選ばせ、「そもそも必要か」を Noul で判定する例がある。Jev の呼び出しは 0.09〜0.31 秒だった。
- Python SDK は `typesafe-sdk`。cookbook のインストール例は、次のとおり。
  `pip install "typesafe-sdk>=0.5.7" --extra-index-url https://pypi.typesafe.ai/`
- 公式サイトは「アーリーアクセス」と案内している。入力トークンは10億あたり42ドル。

**公式が挙げている弱点（jev-1.13）**
- 文字どおりに解釈する。条件は正確に書く必要がある。
- 主な学習言語は英語。日本語などは受け付けるが、精度は下がる。
- 状態に無関係な情報が多いと精度が落ちる。
- 矛盾する指示や、間接的な指示は苦手。
- 数の計算、日付の比較は苦手。コード側で行う。
- 敵対的な文章に動かされることがある。
- 文章の生成には向かない。

### 4.4 ハエ脳（mk-jev-fly-brain）

- maleCNS v1.0 コネクトームの約12,000ニューロンのサブグラフを、ブラウザ上（Web Worker）でスパイク単位にシミュレートする。100ms ごとに判断する。
- 運動ニューロン群は fwd / back / jump / punch / kick / wing の6つ。閾値を超えたもののうち、ゲイン補正後に最も強いものが動作になる。どれも超えなければ `stand`。
- 群ごとの閾値（`thresh`、既定は 4 SD）とゲイン（`gain`）があり、これを変えると反応のしやすさが変わる。
- 感覚入力はゲーム状態（距離、相手の動き、被弾）から作っている。README も、感覚のエンコードとニューロン群→動作の対応は「仮定」と明記している。
- 格闘ゲームでのハエ脳の強みは反応速度だった。Jev の応答（README では約350ms）に合わせて遅くすると、与ダメージが70%減った。
- リポジトリには hybrid モードがあり、速い局所ポリシーが動き、Jev が遅れて修正・教師役を務める。
- ハエ脳 vs ルールボットは、アカウントなしで動く（`python server.py`）。

## 5. 設計案

### 5.1 理性層（Jev）

- `state`: 人が書いた性格の文章、スキル一覧、現在の状況、その場の指示。
- 質問（1回の呼び出しでまとめて出す）:
  - Noul: その場の指示は、今の状況に当てはまるか
  - Choice: 次の動作はどれか（選択肢は 4.2 の動作名。説明は人が書く）
- Choice の答え → 4.2 のトークンに変換して Bittle へ送る。
- スキルは「既存トークンの並び」としてコード側に持つ。Jev は、どのスキルを使うかを選ぶ役だけを担当する（Jev は手順を読んで実行できないため）。

設計スケッチ（未実行。SDK の書き方は公式ドキュメントの例に基づく）:

```python
from typesafe_sdk import Choice, Noul, TypeSafeClient

TOKENS = {"walk": "kwkF", "back": "kbk", "sit": "ksit", "hello": "khi", "balance": "kbalance"}

questions = {
    "instruction_applies": Noul(instructions="The user's current instruction applies to the current situation."),
    "next_action": Choice(
        instructions="Which action should the robot take next?",
        criteria={
            "walk": "Walk forward",
            "back": "Step backward",
            "sit": "Sit down",
            "hello": "Greet",
            "balance": "Stand still",
        },
    ),
}

with TypeSafeClient() as client:
    response = client.system_one(state=state, questions=questions)

if response.answers["instruction_applies"].noul >= THRESHOLD:  # 閾値はコード側で決める
    token = TOKENS[response.answers["next_action"].choice]
```

日本語の性格文を英訳して渡すかどうかは、実験で決める（5.4）。

### 5.2 反射層（ハエ脳）

- 後から追加する。Jev だけで動く状態を先に作る。
- 性格づけ: Jev の答え（Score や Noul）を、群ごとの閾値・ゲインの倍率にコード側で変換する。例: 臆病さが高いと、後退・逃避の閾値を下げる。
- ハエ脳の出力（7種類の動き）→ Bittle の対応は、次のとおり。

| ハエの動き | Bittle |
|---|---|
| stand | `kbalance`（または `kup`） |
| walk_forward | `kwkF` |
| walk_backward | `kbk` |
| jump_away | 対応スキル未確認（代替案: `kbk` か `krnF`） |
| punch / kick / block | 対応する動作が確認できていない（要決定） |

### 5.3 Bittle との通信

- 動作の送信: Wi-Fi なら `http://<IP>/action?name=...`、有線・Bluetooth なら Python API の `sendSkillStr()`。
- Mac では pyserial を使う。

### 5.4 進め方

1. Jev だけで判断する経路を作る（Mac → Jev → Bittle）。
2. 実験項目を試す（6章）。
3. 結果を見て、ハエ脳の反射層を足す。

## 6. 実験項目（未確認）

1. 標準ファームのまま、PC から IMU の値を読み出せるか。
2. カメラの認識結果を Mac へ渡せるか。
3. 反射で反応させる対象を決める（接近、持ち上げ、転倒、音など）。1と2の結果で選べる範囲が決まる。
4. Wi-Fi 経由で、決まった動作名以外（関節の直接指定など）を送れるか。
5. 日本語の性格文が Jev にどこまで効くか（英訳して渡す場合との比較）。
6. Python API の `readAnalogValue(pin)` が、標準ファームで動くか。
7. 複数の拡張モジュールを、1つのファームで同時に使えるか。
8. ハエ脳の閾値・ゲインを Jev の答えから調整したとき、反応の違いが実機で観察できるか。

## 7. 前提（検証しない）

- TypeSafe の API キーを持っているかは確認しない。
- Bittle との接続手段（Wi-Fi モジュール、Bluetooth ドングル、USB アダプタのどれを持っているか）は確認しない。
- Wi-Fi の遅延は測らない。

## 8. 参考リンク

**Petoi**
- [Serial Protocol](https://docs.petoi.com/apis/serial-protocol.md)
- [Python API](https://docs.petoi.com/apis/python-api.md)
- [NyBoard V1_1 & V1_2](https://docs.petoi.com/nyboard/nyboard-v1_1-and-nyboard-v1_2.md)
- [WiFi module ESP8266](https://docs.petoi.com/communication-modules/wifi-esp8266.md)
- [ESP8266 + Python Scripts](https://docs.petoi.com/communication-modules/wifi-esp8266/esp8266-+-python-scripts-implement-wireless-crowd-control.md)
- [Extensible modules: Introduction](https://docs.petoi.com/extensible-modules/introduction.md)
- [MU Camera](https://docs.petoi.com/extensible-modules/mu-camera.md)
- [actions.h（OpenCat）](https://raw.githubusercontent.com/PetoiCamp/OpenCat/main/ModuleTests/ESP8266WiFiController/actions.h)
- [PetoiCamp/wifi_control](https://github.com/PetoiCamp/wifi_control)

**TypeSafe（Jev）**
- [Introduction](https://docs.typesafe.ai/introduction)
- [State](https://docs.typesafe.ai/concepts/state.md)
- [Choice](https://docs.typesafe.ai/primitives/choice.md)
- [Skill suggestion](https://docs.typesafe.ai/cookbooks/skill_suggestion.md)
- [Jev 1.13 jaggedness](https://docs.typesafe.ai/model-jaggedness/jev-1.13.md)
- [TypeSafe AI](https://typesafe.ai/)

**ハエ脳**
- [lavallee/mk-jev-fly-brain](https://github.com/lavallee/mk-jev-fly-brain)
- [fight.js](https://raw.githubusercontent.com/lavallee/mk-jev-fly-brain/main/mk/fight.js)
