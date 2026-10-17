# 確認済みの事実

公式ドキュメントまたはリポジトリのコードを読んで確認した事実と、記録済みの実験結果だけを置く（[documentation.md](../rules/documentation.md)）。
設計書からは ID（例: F-B1）で参照する。出典は各節の末尾と [README の参考リンク](../../README.md#参考リンク) にある。

> 「Bittle の実機（実験で確認）」節を除く内容は、最初の README（[計画書](https://github.com/kogai/petoi-the-dog-walk/blob/5174b74/README.md)）の「4. 確認済みの事実」を移したもの。
> 計画書にあった「頭部にカメラらしきものが付いている（型番は未確認）」は、目視による観察で出典が無いため、ここには置かない（E02 で扱う）。
> 新しく事実を足すときは、出典（URL か、結果を記録した実験 ID）を必ず添える。

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

## Bittle の実機（実験で確認）

E00 の2回の実施による。回数が少ないので、値は目安。

- 共通の条件: 標準ファーム（起動の表示に `N_260806`）、USB アダプタでシリアル接続、115200 baud、脚を付けていない Bittle。
- 実施1（書き直す前の手順書。E00 の1つ目の「結果」節）: Mac の OS・CPU は記録なし。多くの回で `Low power:6V` が出ていた。
- 実施2（書き直した手順書。E00 の2つ目の「結果」節）: macOS 26.5.1（arm64）。判定に使った回では `Low power` は出ていない。

| ID | 事実 | 実施 |
|---|---|---|
| F-B9 | Mac がシリアルポートを開くたびに、起動の表示が出る。先頭に `k`、続いて `* Start *`、`Bittle`、`N_260806`、`IMU`、`Servo:`、`Skill`、`Ready!`。`Ready!` はポートを開いてから約 5.5 秒後。その約 0.1〜0.9 秒後に、何も送っていなくても `g` と `d` が出る（`Low power:6V` が続くこともある。途中で再起動したときは出なかった。F-B15）。`Ready!` より前に送ったトークンは崩れることがある（`khi` が `?hi` として返った）。ポートを開くことで再起動しているかは未確認 | 1・2 |
| F-B10 | 起動後の最初の `k` で始まるトークンには、約 30 ms で `G` が返る。続けて送った2つ目の `khi` には `G` は返らなかった。`G` の意味は未確認 | 1・2 |
| F-B11 | `k` で始まるトークンを送ると、しばらくして `k` が返る。送ってから `k` までの時間（1回の実行で1トークン、LF）: `kwkF` 375 ms、`kbalance` 844 ms、`ksit` 932 ms、`khi` 5486／5506／5503 ms（3回） | 2 |
| F-B12 | 知らないトークン `kzzz`（LF）には、約 31 ms で `?zzz`（先頭の `k` を除いた名前に `?` を付けたもの）が返り、同じ塊の中で続けて `k` も返った（1回）。改行を CRLF にすると、知っているトークンも 12〜32 ms で `?<名前>\r` と返った。再起動の無かった回では5トークンとも `k` が続き、途中で再起動した回では、どれにも `k` が続かなかった。CRLF で Bittle が動いたかは記録に使っていない（推論: 動作しない） | 1（`Low power:6V` が出ている状態） |
| F-B13 | `khi` を 2 秒あけて2回送ると、`k` は1つだけ返った。1回目の送信から 8604 ms、2回目から 6597 ms で、`khi` 1回分（F-B11）とは合わない（1回のみ） | 2 |
| F-B14 | `kwkF` を1回だけ送ると、送ってから 5 秒後には、前脚付近のサーボは動いていなかった（人が見て答えた。1回のみ）。止まった時刻は測っていない | 2 |
| F-B15 | `Low power:6V` と表示されることがある（電圧が低いという意味と読める。電圧は測っていない）。起動の途中（`IMU` の行の約 3.9 秒後）で再起動することがある（`* Start *` が2回出る）。再起動が1回だった回（3回）では、`Ready!` はポートを開いてから約 11.4 秒後。再起動が2回あった回（実施1、1回）では、16 秒までに `Ready!` は来なかった。途中で再起動した回では、`Ready!` の後に `g`・`d` は出なかった（どの回も `Ready!` より前にトークンを送っていたので、それが理由かは分からない）。`Low power` の行が無い回でも、再起動が2回あった | 1・2 |
| F-B16 | `serialport` クレート（`walk-experiments` の `e00-serial-probe`）で、macOS（arm64）の USB シリアルを開き、送受信できた。Bluetooth シリアルは試していない | 2（1 でも動いた） |

出典: E00（[experimentals/E00-connectivity.md](../../experimentals/E00-connectivity.md)）。「実施」の列は、どちらの「結果」節の記録によるか。

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
| F-F10 | 約1.2万ニューロンの部分回路は、`scripts/build_malecns_fighter.py` が maleCNS v1.0 の公開データから作る。名前の付いた感覚の群（LPLC2・LC4・LC10a・LC9・LPLC4・P1（pC1）・前脚の味覚毛・胸背と中脚の触覚毛）と運動の群（DNp09・MDN・TTMn・脚と翅の運動ニューロン）を必ず入れる。途中のニューロンには、感覚の群ごとに前向き、運動の群ごとに後ろ向きに、既定で3段まで影響を広げ、感覚と運動の組ごとにその積をその組の最大値で割り、組の間の最大を点数として付ける。点数の大きい順に、上限（既定 12,000）まで足す。使う結合はシナプス数が既定3以上のものだけ。status が Traced・Anchor のニューロンに絞るのは、感覚と運動の群を選ぶときだけで、途中のニューロンは絞っていない。部位（中心複合体など）を指定して入れる処理は無い。感覚の群を足す引数 `--probe-types` がある |
| F-F11 | `mk/flybrain.js` の冒頭のコメントは「Port of src/model.py (Shiu et al. 2024-style LIF)」と書いているが、リポジトリに `src/model.py` は無い。モデルはコンダクタンス型の LIF で、刻み 0.25ms、静止電位とリセット電位 -65mV、閾値 -50mV、興奮・抑制の反転電位 0mV / -75mV、全ニューロンに 10Hz の背景ポアソン入力がある。同じコメントで、生理のパラメーターは仮定（「physiology = assumptions」）としている |
| F-F12 | `mk/flybrain.js` の乱数は、種を指定できる mulberry32。`new FlyBrain(circuit, { seed })` で種を渡す。`flybrain.js` はブラウザの API（DOM、Web Worker、`postMessage`）を使っていない。使っているのは Web Worker の入口の `mk/flyworker.js` と、ゲームの `mk/fight.js`。Node で動かすスクリプト `scripts/js/probe_channels.mjs` があるが、その import 先 `../../app/static/mk/flybrain.js` はリポジトリに無い |
| F-F13 | 部分回路 `mk/fly_circuit.js`（12,043,072 バイト）はリポジトリに含まれる。ニューロンは 12,000、結合（`pre` の要素）は 941,464。コード・`EXPERIMENTS.md`・回路の生成スクリプト・分析スクリプトのライセンスは MIT。`fly_circuit.js` は maleCNS v1.0 から作ったもので、CC-BY 4.0（`LICENSE`、`CREDITS.md`） |

出典: [lavallee/mk-jev-fly-brain](https://github.com/lavallee/mk-jev-fly-brain)（README、[server.py](https://github.com/lavallee/mk-jev-fly-brain/blob/8217937e6e725b0ce3160c6e1ab0183db8b8aa30/server.py)）、[fight.js](https://github.com/lavallee/mk-jev-fly-brain/blob/8217937e6e725b0ce3160c6e1ab0183db8b8aa30/mk/fight.js)、[build_malecns_fighter.py](https://github.com/lavallee/mk-jev-fly-brain/blob/8217937e6e725b0ce3160c6e1ab0183db8b8aa30/scripts/build_malecns_fighter.py)（F-F8〜F-F10 はコミット `8217937` のコードで確認）、[flybrain.js](https://github.com/lavallee/mk-jev-fly-brain/blob/8217937e6e725b0ce3160c6e1ab0183db8b8aa30/mk/flybrain.js)、[flyworker.js](https://github.com/lavallee/mk-jev-fly-brain/blob/8217937e6e725b0ce3160c6e1ab0183db8b8aa30/mk/flyworker.js)、[probe_channels.mjs](https://github.com/lavallee/mk-jev-fly-brain/blob/8217937e6e725b0ce3160c6e1ab0183db8b8aa30/scripts/js/probe_channels.mjs)、[CREDITS.md](https://github.com/lavallee/mk-jev-fly-brain/blob/8217937e6e725b0ce3160c6e1ab0183db8b8aa30/CREDITS.md)（F-F11〜F-F13 もコミット `8217937` で確認。F-F13 の数は `fly_circuit.js` の配列の要素を数えた）

## ハエ脳のモデルの出どころと移植版

| ID | 事実 |
|---|---|
| F-F14 | [philshiu/Drosophila_brain_model](https://github.com/philshiu/Drosophila_brain_model) の README は、bioRxiv の論文 `10.1101/2023.05.02.539144`（Shiu et al.）に付属し、論文のデータを再現するコードを含むと書いている。ライセンスは MIT（Philip Shiu、Nico Spiller）。Python と Brian 2 で書かれ、既定の配線データは FlyWire の v630 で、v783 に切り替えられる。できることは、指定したニューロンをポアソン発火で活性化することと、結合の重みを 0 にして抑制することで、出力はスパイクの時刻と発火率。感覚入力から行動を選ぶループは無い |
| F-F15 | 同リポジトリの `model.py` のモデルは電流型の LIF（`dv/dt = (v_0 - v + g) / t_mbr`、`dg/dt = -g / tau`）。静止電位とリセット電位 -52mV、閾値 -45mV、膜の時定数 20ms、`tau` 5ms、不応期 2.2ms、シナプスの遅延 1.8ms、1シナプスあたりの重み 0.275mV（コメントでは自由パラメーター）。ポアソン入力は活性化したニューロンにだけ入り、背景入力は無い |
| F-F16 | [eonsystemspbc/fly-brain](https://github.com/eonsystemspbc/fly-brain) の README は、Shiu et al. のモデルに基づき、FlyWire の配線（約13.8万ニューロン、約500万シナプス）を使うと書いている。ライセンスは GPL-2.0。Brian 2 の CPU 版を正解として、PyTorch・Brian2CUDA・NEST GPU・GeNN・Brian2GeNN の版と、出力を比べるスクリプト（`code/compare_*.py`）がある。`data/results/nature_2026_07/no_io/timing_summary.csv` では、Brian 2 の CPU 版・1試行で、1秒分の計算に 2.909秒（`normal_sim_median_s`。I/O ありの5回の中央値）、I/O なしの1回で 2.891秒（`noio_sim_s`）かかっている。測った機械は、読んだ範囲の README には書かれていない。**第三者の測定** |
| F-F17 | [snedea/flybrain](https://github.com/snedea/flybrain) は、FlyWire FAFB v783 の 139,255 ニューロン・270万結合をブラウザの LIF で動かす、`heyseth/worm-sim` のフォーク（README）。`mk-jev-fly-brain` のコード・README・`CREDITS.md` は、このリポジトリに触れていない（コミット `8217937` で確認） |

出典: F-F14・F-F15 は `philshiu/Drosophila_brain_model` のコミット `91bdd1e`（[Readme.md](https://github.com/philshiu/Drosophila_brain_model/blob/91bdd1e7dcf193f3e7ca5a8933497fcef63b7960/Readme.md)、[model.py](https://github.com/philshiu/Drosophila_brain_model/blob/91bdd1e7dcf193f3e7ca5a8933497fcef63b7960/model.py)、`LICENSE`）、F-F16 は `eonsystemspbc/fly-brain` のコミット `a3db62f`（[README.md](https://github.com/eonsystemspbc/fly-brain/blob/a3db62f9436074e485c0278290c2164ed6150808/README.md)、`data/results/nature_2026_07/no_io/README.md`、`LICENSE`、[timing_summary.csv](https://github.com/eonsystemspbc/fly-brain/blob/a3db62f9436074e485c0278290c2164ed6150808/data/results/nature_2026_07/no_io/timing_summary.csv)）、F-F17 は `snedea/flybrain` のコミット `9191824` の [readme.md](https://github.com/snedea/flybrain/blob/9191824d17871b7851645782d53d23f213ddb938/readme.md)。論文そのものは読んでいない
