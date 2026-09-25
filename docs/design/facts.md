# 確認済みの事実

公式ドキュメントまたはリポジトリのコードを読んで確認した事実だけを置く。
設計書からは ID（例: F-B1）で参照する。出典は各節の末尾と [README の参考リンク](../../README.md#参考リンク) にある。

> この内容は、最初の README（[計画書](https://github.com/kogai/petoi-the-dog-walk/blob/5174b74/README.md)）の「4. 確認済みの事実」を移したもの。
> 計画書にあった「頭部にカメラらしきものが付いている（型番は未確認）」は、目視による観察で出典が無いため、ここには置かない（E02 で扱う）。
> 新しく事実を足すときは、出典の URL を必ず添える。

## Bittle（初代・NyBoard）

| ID | 事実 |
|---|---|
| F-B1 | MCU は ATmega328P（16MHz、SRAM 2KB、Flash 32KB） |
| F-B2 | 基板上に 6軸 IMU（MPU6050）と赤外線受信機（VS1838B）がある |
| F-B3 | 拡張用 Grove ソケットが4つある（G1: I2C、G2: アナログ A2/A3、G3: D8/D9、G4: D6/D7） |
| F-B4 | 通信のボーレートは 115200。公式 Bluetooth ドングルがある |
| F-B5 | Wi-Fi は公式の ESP8266 モジュール（別売の拡張）で使える。公式サンプルは HTTP サーバーで、`http://<IP>/action?name=<動作名>` を受け、シリアルコマンドに変換して NyBoard へ渡す |
| F-B6 | Python API（PetoiRobot）に `sendSkillStr('ksit', 3)` や `autoConnect()` などがある。立つ姿勢の例は `kup` |
| F-B8 | 公式の Wi-Fi スケッチ（`ESP8266WiFiController.ino`）の `/action` は、`name` が動作名の表（`actions.h`）にあれば対応するトークンを、**無ければ `name` の文字列をそのまま**シリアルへ送る。応答の本文は、その後 40ms 以内にシリアルから読めた文字列（`text/plain`）。表の `stop` は `d`。ほかに `/calibration?c=<関節>&o=<値>` が `c<関節> <値>` を送る。**実機に書き込まれたスケッチが、このコードと同じかは未確認**（E04） |

出典（Petoi 公式ドキュメント）:
- F-B1〜F-B3: [NyBoard V1_1 & V1_2](https://docs.petoi.com/nyboard/nyboard-v1_1-and-nyboard-v1_2.md)、[Extensible modules: Introduction](https://docs.petoi.com/extensible-modules/introduction.md)
- F-B4: [Serial Protocol](https://docs.petoi.com/apis/serial-protocol.md)
- F-B5: [WiFi module ESP8266](https://docs.petoi.com/communication-modules/wifi-esp8266.md)、[ESP8266 + Python Scripts](https://docs.petoi.com/communication-modules/wifi-esp8266/esp8266-+-python-scripts-implement-wireless-crowd-control.md)
- F-B6: [Python API](https://docs.petoi.com/apis/python-api.md)
- F-B8: [ESP8266WiFiController.ino](https://github.com/PetoiCamp/OpenCat/blob/24563ca2e0218150f6299d013cd3a20c45433467/ModuleTests/ESP8266WiFiController/ESP8266WiFiController.ino)、[actions.h](https://github.com/PetoiCamp/OpenCat/blob/24563ca2e0218150f6299d013cd3a20c45433467/ModuleTests/ESP8266WiFiController/actions.h)（コミット `24563ca` のコードで確認）

> 事実ごとの出典の対応は、計画書の時点で記録されていなかった。上は文書の範囲から当てたもの。個々の事実を設計の根拠にするときは、該当ページで確かめ直し、この対応を直す。

## 動作トークン（F-T1、公式 `actions.h` で確認済み）

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

出典: [actions.h（OpenCat）](https://raw.githubusercontent.com/PetoiCamp/OpenCat/main/ModuleTests/ESP8266WiFiController/actions.h)

## Jev（TypeSafe）

| ID | 事実 |
|---|---|
| F-J1 | Jev は System One モデルで、テキストを生成しない。状態（`state`）と型付きの質問を受け取り、型付きの答えを返す |
| F-J2 | `state` は文字列・JSON オブジェクト・配列のどれでもよい |
| F-J3 | 質問は3種類。Choice（`choice`・`probabilities`・`confidence`、選択肢は最大255個）、Score（段階評価）、Noul（yes/no の確率 `noul`、0〜1） |
| F-J4 | `instructions` と `criteria`（選択肢の説明）は、文字列・オブジェクト・配列で書ける |
| F-J5 | 複数の質問を1回の呼び出しでまとめて出せる。質問を増やしても応答時間はほとんど変わらない |
| F-J6 | 公式 cookbook（skill suggestion）に、スキルの説明文を Choice の選択肢にし、「そもそも必要か」を Noul で判定する例がある。呼び出しは 0.09〜0.31 秒だった |
| F-J7 | Python SDK は `typesafe-sdk`。`pip install "typesafe-sdk>=0.5.7" --extra-index-url https://pypi.typesafe.ai/`。アーリーアクセス。入力トークンは10億あたり42ドル |
| F-J8 | 公式が挙げる弱点（jev-1.13）: 文字どおりに解釈する／主な学習言語は英語で他言語は精度が下がる／無関係な情報が多いと精度が落ちる／矛盾・間接的な指示が苦手／数の計算・日付比較が苦手／敵対的な文章に動かされることがある／文章生成に向かない |
| F-J9 | fly-brain リポジトリの `jev.py` は、Jev を次の形で呼んでいる。`POST https://api.typesafe.ai/v1/systemone`、ヘッダー `Authorization: Bearer <API キー>`、本文 `{"model": "jev-latest", "state": ..., "questions": {<キー>: {"type": "choice", "instructions": ..., "criteria": {...}}}}`。応答から `answers.<キー>.choice`・`probabilities`・`confidence` と、`model`・`usage` を読んでいる。**第三者のコードで確認した二次情報で、公式ドキュメントでは未確認**。Noul の書き方はこのコードに無い |

出典（TypeSafe ドキュメント。F-J9 だけは第三者のコード）:
- F-J1: [Introduction](https://docs.typesafe.ai/introduction)
- F-J2: [State](https://docs.typesafe.ai/concepts/state.md)
- F-J3〜F-J5: [Choice](https://docs.typesafe.ai/primitives/choice.md)、[Introduction](https://docs.typesafe.ai/introduction)
- F-J6、F-J7（SDK）: [Skill suggestion](https://docs.typesafe.ai/cookbooks/skill_suggestion.md)
- F-J7（料金・アーリーアクセス）: [TypeSafe AI](https://typesafe.ai/)
- F-J8: [Jev 1.13 jaggedness](https://docs.typesafe.ai/model-jaggedness/jev-1.13.md)
- F-J9: [mk-jev-fly-brain の jev.py](https://github.com/lavallee/mk-jev-fly-brain/blob/8217937e6e725b0ce3160c6e1ab0183db8b8aa30/jev.py)（コミット `8217937` で確認。公式ドキュメントでの確認は未了）

## ハエ脳（mk-jev-fly-brain）

| ID | 事実 |
|---|---|
| F-F1 | maleCNS v1.0 コネクトームの約12,000ニューロンのサブグラフを、ブラウザ上（Web Worker）でスパイク単位にシミュレートする。100ms ごとに判断する |
| F-F2 | 運動ニューロン群は fwd / back / jump / punch / kick / wing の6つ。閾値を超えたもののうち、ゲイン補正後に最も強いものが動作になる。どれも超えなければ `stand` |
| F-F3 | 群ごとの閾値（`thresh`、既定は 4 SD）とゲイン（`gain`）があり、反応のしやすさが変わる |
| F-F4 | 感覚入力はゲーム状態（距離、相手の動き、被弾）から作っている。感覚のエンコードと群→動作の対応は、README 自身が「仮定」と明記している |
| F-F5 | 格闘ゲームでの強みは反応速度。Jev の応答（README では約350ms）に合わせて遅くすると、与ダメージが70%減った |
| F-F6 | hybrid モードがあり、速い局所ポリシーが動き、Jev が遅れて修正・教師役を務める |
| F-F7 | ハエ脳 vs ルールボットは、アカウントなしで動く（`python server.py`） |
| F-F8 | 出力される動きは7種類: `stand`, `walk_forward`, `walk_backward`, `jump_away`, `punch`, `kick`, `block`。運動ニューロン群との対応は fwd→walk_forward、back→walk_backward、jump→jump_away、punch→punch、kick→kick、wing→block（`fight.js` の `ACTIONS` と `POOL_ACTION`） |
| F-F9 | `server.py` は Flask で、ページの配信、TypeSafe API の中継（キーをブラウザに渡さないため）、試合記録の JSON 保存を行う。シミュレーション本体はブラウザ側で動く（`server.py` の docstring とルート定義） |

出典: [lavallee/mk-jev-fly-brain](https://github.com/lavallee/mk-jev-fly-brain)（README、[server.py](https://github.com/lavallee/mk-jev-fly-brain/blob/8217937e6e725b0ce3160c6e1ab0183db8b8aa30/server.py)）、[fight.js](https://github.com/lavallee/mk-jev-fly-brain/blob/8217937e6e725b0ce3160c6e1ab0183db8b8aa30/mk/fight.js)（F-F8、F-F9 はコミット `8217937` のコードで確認）
