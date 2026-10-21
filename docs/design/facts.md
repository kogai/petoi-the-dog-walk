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
| F-B4 | 通信ポートの既定のボーレートは 115200。2021年1月1日から、公式の Bluetooth ドングルを同梱している（手元の機体に付いているかは未確認） |
| F-B5 | Wi-Fi は公式の ESP8266 モジュールで使える。公式サンプルは HTTP サーバーで、`http://<IP>/action?name=<動作名>` を受け、シリアルコマンドに変換して NyBoard へ渡す |
| F-B6 | Python API（PetoiRobot）に `sendSkillStr('ksit', 3)` や `autoConnect()` などがある。立つ姿勢の例は `kup` |
| F-B8 | 公式の Wi-Fi スケッチ（`ESP8266WiFiController.ino`）の `/action` は、`name` が動作名の表（`actions.h`）にあれば対応するトークンを、**無ければ `name` の文字列をそのまま**シリアルへ送る。応答の本文は、その後 40ms 以内にシリアルから読めた文字列（`text/plain`）。表の `stop` は `d`。ほかに `/calibration?c=<関節>&o=<値>` が `c<関節> <値>` を送る。**実機に書き込まれたスケッチが、このコードと同じかは未確認**（E04） |
| F-B19 | 公式のファーム（OpenCat）の `read_serial`（`src/io.h`）は、受信バッファの最初の1バイトをトークンとし、続くバイトを、最後に読んだバイトが終端（小文字のトークンなら `\n`、大文字なら `~`）になるか、バイトの来ない時間が `SERIAL_TIMEOUT`（5 ms。`src/OpenCat.h`）を超えるまで、そのトークンの引数として読む。その間は、届いているバイトをまとめて読む。**基板に書き込まれたファームが、このコードと同じかは未確認**（起動の表示の `N_260806` は、このコミットの `SOFTWARE_VERSION` と同じ） |
| F-B20 | 公式のファーム OpenCat の README は、OpenCat を「Arduino と Raspberry Pi を土台にした」フレームワークと書き、拡張の1つに「Raspberry Pi、Nvidia Jetson Nano などの AI コプロセッサー（有線・無線でつなぐ）」を挙げている。README の「Current Hardware」は Bittle X（BiBoard/ESP32）と Nybble Q で、初代 Bittle（NyBoard/ATmega328P）は販売終了だがこのリポジトリで扱うと書いている。ESP32 の BiBoard は、別のリポジトリ `OpenCatEsp32-Quadruped-Robot` で扱う |
| F-B21 | 公式の手順「Raspberry Pi serial port as an interface」は、次のように書いている。NyBoard に 2x5 のソケットをはんだ付けすると、Raspberry Pi を差せる。NyBoard の大きさに最も合うのは Pi 3A+。Bittle では、ソケットをはんだ付けすると背中のカバーを付けられなくなる。Pi を支える台（standoff）は 3D プリントできる（STL が公開されている）。ソケットに差すと、両者のシリアルは 3.3V でつながるはず（"should be"）と書いている。NyBoard のシリアルは1つだけなので、Pi のシリアルで動かすときは USB アダプタを外す（Pi を載せる前に 6ピンの USB アダプタを外す）。Pi の側では、`raspi-config` でシリアルのログインシェルを無効にし、シリアルのハードウェアを有効にする。また、GPIO 4 からリセットの信号が繰り返し送られないように、1-wire を無効にする。Pi のような上位の機械は、引数をバイナリで送るコマンドも使える（角度 65 を1バイトで送る、など）。例として Python の `ardSerial.py` を挙げ、`ardSerial.py` と `OpenCat.ino` の解析はプロトコルのすべてには対応していないと書いている。電源の注意は Nybble についてだけ（Pi の電流で激しい動きがしにくくなる）で、Bittle についての記述は無い。冒頭に「ロボットが動くのに Pi は要らない」とある。ページに日付や対象のファームの版は書かれていない |
| F-B22 | NyBoard V1_1 & V1_2 のページの「Power system」節は、Raspberry Pi は電力を多く使うので、DC-DC の TPS565201 で 5V 3A（ピークは 5A）を出すと書いている。「I2C switch」節は、スイッチを「Arduino」か「Raspberry Pi」に倒して、基板の周辺機器の I2C のマスターを切り替えられると書いている。**手元の基板の版（V1_0・V1_1・V1_2）は未確認** |
| F-B23 | 公式の「Serial Protocol」のページは、どのトークンも、続きの解析の形を決める ASCII の1文字で始まり、大文字と小文字を区別し、ふつうは小文字だと書いている。シリアルモニターで試す例に `ksit`、`m0 30`、`m0 -30`、`kbalance`、`kwkF`、`ktrL`、`d` を挙げ、`c` と `m` のコマンドはまとめて書ける（`m8 40 8 -35 0 50` など）としている |

出典（Petoi 公式ドキュメント）:
- F-B1〜F-B3: [NyBoard V1_1 & V1_2](https://docs.petoi.com/nyboard/nyboard-v1_1-and-nyboard-v1_2.md)（V1_1 は V1_0 からソケットとピンの定義を変えていない、とページに書かれている）、[Extensible modules: Introduction](https://docs.petoi.com/extensible-modules/introduction.md)
- F-B4: [NyBoard V1_1 & V1_2](https://docs.petoi.com/nyboard/nyboard-v1_1-and-nyboard-v1_2.md)（「Overview」節。[Serial Protocol](https://docs.petoi.com/apis/serial-protocol.md) の本文（画像・埋め込みのコードを除く）には、ボーレートも Bluetooth ドングルも書かれていない）
- F-B5: [WiFi module ESP8266](https://docs.petoi.com/communication-modules/wifi-esp8266.md)、[ESP8266 + Python Scripts](https://docs.petoi.com/communication-modules/wifi-esp8266/esp8266-+-python-scripts-implement-wireless-crowd-control.md)
- F-B6: [Python API](https://docs.petoi.com/apis/python-api.md)
- F-B23: [Serial Protocol](https://docs.petoi.com/apis/serial-protocol.md)（2026-10-17 に本文を読んだ。画像・埋め込みのコードは読んでいない）
- F-B21: [Raspberry Pi serial port as an interface](https://docs.petoi.com/apis/raspberry-pi-serial-port-as-an-interfac.md)（2026-10-17 に読んだ。`…-interface.md` の URL は、末尾の `e` が無いこの URL へ転送される）
- F-B22: [NyBoard V1_1 & V1_2](https://docs.petoi.com/nyboard/nyboard-v1_1-and-nyboard-v1_2.md)（2026-10-17 に読んだ）
- F-B19: [io.h](https://github.com/PetoiCamp/OpenCat/blob/24563ca2e0218150f6299d013cd3a20c45433467/src/io.h)、[OpenCat.h](https://github.com/PetoiCamp/OpenCat/blob/24563ca2e0218150f6299d013cd3a20c45433467/src/OpenCat.h)（コミット `24563ca` のコードで確認）
- F-B8: [ESP8266WiFiController.ino](https://github.com/PetoiCamp/OpenCat/blob/24563ca2e0218150f6299d013cd3a20c45433467/ModuleTests/ESP8266WiFiController/ESP8266WiFiController.ino)、[actions.h](https://github.com/PetoiCamp/OpenCat/blob/24563ca2e0218150f6299d013cd3a20c45433467/ModuleTests/ESP8266WiFiController/actions.h)（コミット `24563ca` のコードで確認）
- F-B20: [OpenCat の README.md](https://github.com/PetoiCamp/OpenCat/blob/24563ca2e0218150f6299d013cd3a20c45433467/README.md)（コミット `24563ca` で確認）

> 事実ごとの出典の対応は、計画書の時点で記録されていなかった。F-B1〜F-B6 は、2026-10-17 に上のページを読み直して確かめた。そのとき、F-B4 の出典を Serial Protocol から NyBoard のページに直し、F-B4 を出典の書き方（既定値、同梱の時期）に合わせた。F-B5 にあった「別売の拡張」は出典に書かれていなかったので外した。Serial Protocol のページの内容は F-B23 にした。

## Bittle の実機（実験で確認）

E00 の2回の実施による。回数が少ないので、値は目安。

- 共通の条件: 標準ファーム（起動の表示に `N_260806`）、USB アダプタでシリアル接続、115200 baud、脚を付けていない Bittle。
- 実施1（書き直す前の手順書。E00 の1つ目の「結果」節）: Mac の OS・CPU は記録なし。多くの回で `Low power:6V` が出ていた。
- 実施2（書き直した手順書。E00 の2つ目の「結果」節）: macOS 26.5.1（arm64）。判定に使った回では `Low power` は出ていない。

| ID | 事実 | 実施 |
|---|---|---|
| F-B9 | Mac がシリアルポートを開くたびに、起動の表示が出る。先頭に `k`、続いて `* Start *`、`Bittle`、`N_260806`、`IMU`、`Servo:`、`Skill`、`Ready!`。`Ready!` はポートを開いてから約 5.5 秒後。その約 0.1〜0.9 秒後に、何も送っていなくても `g` と `d` が出る（`Low power:6V` が続くこともある。途中で再起動したときは出なかった。F-B15）。`Ready!` より前に送ったトークンは崩れることがある（`khi` が `?hi` として返った。F-B18）。ポートを開くことで再起動しているかは未確認 | 1・2 |
| F-B10 | 起動後の最初の `k` で始まるトークンには、約 30 ms で `G` が返る。続けて送った2つ目の `khi` には `G` は返らなかった。`G` の意味は未確認 | 1・2 |
| F-B11 | `k` で始まるトークンを送ると、しばらくして `k` が返る。送ってから `k` までの時間（1回の実行で1トークン、LF）: `kwkF` 375 ms、`kbalance` 844 ms、`ksit` 932 ms、`khi` 5486／5506／5503 ms（3回） | 2 |
| F-B12 | 知らないトークン `kzzz`（LF）には、約 31 ms で `?zzz`（先頭の `k` を除いた名前に `?` を付けたもの）が返り、同じ塊の中で続けて `k` も返った（1回）。改行を CRLF にすると、知っているトークンも 12〜32 ms で `?<名前>\r` と返った。再起動の無かった回では5トークンとも `k` が続き、途中で再起動した回では、どれにも `k` が続かなかった。CRLF で Bittle が動いたかは記録に使っていない（推論: 動作しない） | 1（`Low power:6V` が出ている状態） |
| F-B13 | `khi` を 2 秒あけて2回送ると、`k` は1つだけ返った。1回目の送信から 8604 ms、2回目から 6597 ms で、`khi` 1回分（F-B11）とは合わない（1回のみ） | 2 |
| F-B14 | `kwkF` を1回だけ送ると、送ってから 5 秒後には、前脚付近のサーボは動いていなかった（人が見て答えた。1回のみ）。止まった時刻は測っていない | 2 |
| F-B15 | `Low power:6V` と表示されることがある（電圧が低いという意味と読める。電圧は測っていない）。起動の途中（`IMU` の行の約 3.9 秒後）で再起動することがある（`* Start *` が2回出る）。再起動が1回だった回（3回）では、`Ready!` はポートを開いてから約 11.4 秒後。再起動が2回あった回（実施1、1回）では、16 秒までに `Ready!` は来なかった。途中で再起動した回では、`Ready!` の後に `g`・`d` は出なかった（どの回も `Ready!` より前にトークンを送っていたので、それが理由かは分からない）。`Low power` の行が無い回でも、再起動が2回あった | 1・2 |
| F-B16 | `serialport` クレート（`walk-experiments` の `e00-serial-probe`）で、macOS（arm64）の USB シリアルを開き、送受信できた。Bluetooth シリアルは試していない | 2（1 でも動いた） |
| F-B17 | 道具（`e00-serial-probe`、#27 より前の版）は、停止として改行・`d`・改行を1回の書き込みで送っていた（LF なら `"\nd\n"`。道具のコード `serial_probe.rs` の `write_stop`。実施2はコミット `c940978` の版）。応答が記録された回で返ったのは、LF の回では `\n\r\n`、CRLF の回では `\r\r\n` だけで、`g`・`d` を含む応答は無かった。起動の後にファームが自分で出す `g\r\nd\r\n`（F-B9）とは違う | 1・2 |
| F-B18 | 起動の途中（`Ready!` の前）に送った `khi` と、その約 3 秒後（`Ready!` の約 0.4 秒前）に送った `kwkF` に対して、`Ready!` の後に `?hi\nkwkF\r\n` が1つだけ返った（1回のみ） | 1 |

出典: E00（[experimentals/E00-connectivity.md](../../experimentals/E00-connectivity.md)）。「実施」の列は、どちらの「結果」節の記録によるか。

## Raspberry Pi

| ID | 事実 |
|---|---|
| F-R1 | Raspberry Pi 3 Model B（3B）の CPU は 64 ビットの Arm Cortex-A53 が4コアで、各コア 1.2 GHz（最新の版の SoC は BCM2837B0。版 1.1 は BCM2837 で、販売を終えている）。RAM は 1 GB（LPDDR2）。大きさは約 85 mm × 56 mm。Raspberry Pi 3A+ は約 65 mm × 56 mm |
| F-R2 | Raspberry Pi はどの型も 5.1V の電源が要る。3B に推奨する電源は 2.5A。公式の測定では、3B の電流は、待機の平均 0.30A、負荷（Stress）の最大 1.34A・平均 0.85A（2016年2月26日時点の Raspberry Pi OS のイメージ、室温で、HDMI・USB のキーボードとマウス・無線 LAN をつないだ状態。追加の USB 機器や HAT の分は含まない） |

出典（Raspberry Pi 公式ドキュメントのソース。`raspberrypi/documentation` のコミット `ddea98e` で確認）:
- F-R1: [sbc-models.adoc](https://github.com/raspberrypi/documentation/blob/ddea98e98dafe9419feb4fd82710c2fbdef7dc44/documentation/asciidoc/computers/sbcs/sbc-models.adoc)（Raspberry Pi 3 Model B、3A+ の節）、[bcm2837.adoc](https://github.com/raspberrypi/documentation/blob/ddea98e98dafe9419feb4fd82710c2fbdef7dc44/documentation/asciidoc/computers/processors/bcm2837.adoc)
- F-R2: [power-supplies.adoc](https://github.com/raspberrypi/documentation/blob/ddea98e98dafe9419feb4fd82710c2fbdef7dc44/documentation/asciidoc/computers/raspberry-pi/power-supplies.adoc)

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

## ハエ脳（実験で確認）

E17 の2回の実施による（判定は2回目の記録。1回目も同じ判定）。条件: `mk-jev-fly-brain` のコミット `8217937` の `mk/flybrain.js` と `mk/fly_circuit.js`、Node v22.22.0、クラウドの作業用コンテナ（`lscpu` の `CPU(s)` が 4、`Model name` が `Intel(R) Xeon(R) Processor @ 2.80GHz`）。`flybrain.js` は1スレッドで計算する（下の出典のコードを読んだ範囲）。

| ID | 事実 |
|---|---|
| F-F18 | 同じ機械・同じ Node で、種を同じにすると出力が一致した。`scripts/js/probe_channels.mjs`（種 7。import 先だけを直したコピー）の2回の出力がバイト単位で一致し、`FlyBrain.step(100)` を100回呼んだ戻り値も、感覚の入力が無い場合と `loom_L`・`loom_R` に 100 Hz を入れた場合の両方で、種 7 の2回が一致した。種 8 では違った。別の機械・別の版の Node では確かめていない |
| F-F19 | `FlyBrain.step(100)`（100ms 分）1回の計算時間は、4つの計測（2つの入力 × 2回、各100回）の中央値が 48.3〜55.9 ms、最大が 82.2〜101.7 ms で、100ms 以上の窓があった（101.7 ms）。1回目の記録では、最大が 106.1〜122.9 ms だった。回路の読み込みと `FlyBrain` の構築の時間は含まない。最大が大きくなる理由は調べていない |

出典: E17（[experimentals/E17-node-flybrain.md](../../experimentals/E17-node-flybrain.md)）。1スレッドで計算することは、[flybrain.js](https://github.com/lavallee/mk-jev-fly-brain/blob/8217937e6e725b0ce3160c6e1ab0183db8b8aa30/mk/flybrain.js)（コミット `8217937`）を読んで確かめた

## 場所の記憶と帰巣の先行研究

[08](08-navigation.md) 6節の候補を比べるための事実。F-N1〜F-N9 は虫などの生物に学んだ研究、F-N10〜F-N16 はロボット工学の研究。どこまで読んだか（本文・要旨だけ・コードだけ）を、それぞれに書く。2026-10-21 に読んだ。

| ID | 事実 |
|---|---|
| F-N1 | Baddeley et al. 2012（本文を読んだ）: アリの経路の航行を「見覚えのある景色を探すこと」に置き換えるモデル。体の向きと見る向きが同じなので、見覚えのある景色は見覚えのある進む向きを示す。体を振って見回し（scan）、いちばん見覚えのある向きへ進む。見覚えは、学習中の景色すべてとの比較（perfect memory）か、学習した神経回路網（Infomax）で決める。いつ何を覚えるかの指定も、経路を中継点に分けることも要らない。同じ仕組みで、場所を探す動きも出る。試したのはシミュレーションの中だけ。物は空を背にした黒い影として描き、高解像度の2値の全周画像を区画ごとに平均した、低解像度の全周の画像を使う |
| F-N2 | Ardin et al. 2016（本文を読んだ）: ショウジョウバエの匂いの連合学習のために作られたキノコ体のスパイキングモデルが、ニューロンの数を増やし、anti-Hebbian の学習を足しただけで、アリの景色の経路の学習に使えた。視点は地面から 1cm、視野は横 296° × 縦 76°、10×36 画素（360 次元のベクトル）に縮める。実際のアリの経路から作った約 8m の経路 15 本で、経路から 20cm 以上外れた回数の平均は、でたらめに進むと 18.7、perfect memory で 1.1、キノコ体のモデルで 2.6、Infomax で 1.5。キノコ体は経路上の 80 枚以上の画像を覚えた。計算はシミュレーションの中だけ。コードは Dryad にあると書いている（ライセンスは未確認） |
| F-N3 | Sun et al. 2020（本文を読んだ）: 経路積分（中心複合体）、景色による帰巣（キノコ体）、経路追従（AOTU→BU→中心複合体の経路に当てた3層の神経回路網）の3つの仕組みを、リングアトラクターで組み合わせる虫の航行のモデル。どの仕組みを使うかは、キノコ体の出力で切り替える。行動実験のデータ（手がかりを食い違わせた実験）をシミュレーションで再現した。景色は全周の画像（300×104）を Zernike モーメントで周波数に変えて使う。コードは Python で、MIT ライセンス（`XuelongSun/InsectNavigationToolkitModelling` のコミット `470c7d7` の `LICENSE`）。ただし経路積分の部分は、Stone et al. 2017 から改変したと書かれている（README、`source/path_integration.py` と `source/insect_brain_model.py` の docstring） |
| F-N4 | Wystrach et al. 2016（本文を読んだ）: 景色の照合で向きを決める課題を、シミュレーションの中で、視野（横 36°〜360°）と解像度を変えて比べた。向きは、解像度が低く視野が広いほど安定した。視野が狭いと、学んだ経路から大きく離れたときにまったく働かなかった。画素の数は 100〜1000 ほどがいちばん良く、視野が狭いときは解像度を上げると良くなった |
| F-N5 | van Dijk et al. 2024（要旨だけ読んだ）: 虫に学んだ経路追従。行きの経路を、強く圧縮した全周の画像と、その間の位置の関係（オドメトリで測る）の組として覚える。帰りは、オドメトリと景色による帰巣を組み合わせて覚えた場所へ戻り、景色による帰巣でオドメトリの誤差がたまるのを防ぐ。覚える画像は、オドメトリの精度が許す限り間隔を空けて減らす。56g のドローンで、100m までの経路をたどれた。経路の記録は 1m あたり 20 バイト未満 |
| F-N6 | Dupeyroux et al. 2019（AntBot。要旨だけ読んだ）: 六本脚の歩行ロボットで、アリに学んだ経路積分の方法をいくつか屋外で試した。向きは空の偏光（紫外線）から得る。センサーは合わせて 14 画素で、2画素をコンパスに、12画素を optic flow に使う（距離の測り方に歩数を使ったかは、要旨には書かれていない）。帰巣の誤差は、いちばん良い場合で平均して経路全体の 0.67%（アリと同じような光の条件で） |
| F-N7 | Stone et al. 2017（要旨と、コードだけ読んだ。本文は読めなかった）: ハチの中心複合体で、偏光のコンパスのニューロンと optic flow の速さのニューロンが合流することを示し、経路積分と、今の向きと進みたい向きを比べて舵を取る回路を提案した。コード（`InsectRobotics/path-integration` のコミット `d0657cb`）にはライセンスのファイルが無い。ニューロンの数は群ごとに 2〜16 個（`central_complex.py` の定数） |
| F-N8 | Burgard, Fox, Thrun 1999（Markov 位置推定。要旨だけ読んだ。arXiv 1106.0222）: ロボットのいそうな位置すべてに確率を持ち、細かい格子で表す。最初から位置を探すことも、見失った後に立ち直ることもできる。環境の大まかなモデル（占有格子の地図など）を前提にする |
| F-N9 | OpenRatSLAM（README とコードだけ読んだ。論文は読めなかった）: 連続アトラクターの回路の動きに基づく、生物に学んだ SLAM。ループを閉じてオドメトリの誤差を直す。設定の例（車で撮ったデータセット用の `config/config_stlucia.txt.in`）には `camera_fov_deg=53` があり、全周でないカメラで使う例がある。C++ と ROS の実装で、GPL v3（`davidmball/ratslam` のコミット `9050b95` の `license.txt`） |
| F-N10 | Wisth, Camurri, Fallon 2021（VILENS。要旨だけ読んだ。arXiv 2107.07243）: 脚ロボットのオドメトリ。視覚・IMU・LiDAR・脚の4つを因子グラフで密に組み合わせ、1つのセンサーでは推定が成り立たない場面でも動くようにした。脚オドメトリのずれを減らすため、速度のずれ（bias）を状態に加えてその場で推定する。ANYmal で合計 2時間・1.8km、滑りや地面の変形のある場所で試し、ゆるく組み合わせる方法より並進の誤差を平均 62%、回転の誤差を 51% 減らした |
| F-N11 | Yang et al. 2022（Cerberus。要旨と README だけ読んだ。arXiv 2209.07654）: ステレオカメラ・IMU・関節のエンコーダー・接地センサーを使う、視覚・慣性・脚のオドメトリ。脚の幾何のパラメーターをその場で較正し、接地の外れ値を除いて、長距離・高速の歩行でずれを 1% 未満にした。大きな衝撃やカメラが隠れたときにも働いた。README は、VINS-Fusion の最適化の枠組みに接地の項を足したと書いている。リポジトリ（`ShuoYangRobotics/Cerberus` のコミット `ad56c43`）の根にライセンスのファイルは無い |
| F-N12 | Lim et al. 2021（WALK-VIO。要旨だけ読んだ。arXiv 2111.15164）: 四脚ロボットでは、歩く動きによる揺れがカメラと IMU を通して位置の推定を悪くする。歩き方・速度・制御によって揺れ方が変わるので、脚の幾何の拘束の重みを体の動きに合わせて変える VIO を提案した。評価はシミュレーションのデータセット |
| F-N13 | Campos et al. 2020（ORB-SLAM3。要旨だけ読んだ。arXiv 2007.11898）: 単眼・ステレオ・RGB-D のカメラで、視覚だけ・視覚と慣性・複数の地図の SLAM ができる。ピンホールと魚眼のレンズに対応する。見失うと新しい地図を始め、同じ場所に戻ったときに前の地図とつなげる。コード（`UZ-SLAMLab/ORB_SLAM3` のコミット `4452a3c`）は GPL v3 |
| F-N14 | Corlito et al. 2026（本文の一部（方法の選び方、IMU の比較、結論）を読んだ。arXiv 2606.19067）: 四脚ロボット（ANYmal D。GrandTour のデータセット）で、着地の衝撃・高い周波数の振動・速い回転が SLAM に与える影響を、センサーの組み合わせごとに比べた。ステレオは単眼と RGB-D より一貫して良かった。グローバルシャッターは、ローリングシャッターより動きによる追跡の失敗を減らした。視覚が中心の手法（ORB-SLAM3）では、IMU を足すとかえって頑健さが落ちることがあった。ORB-SLAM3 の単眼と IMU（中級の ADIS）の組み合わせの平均の位置の誤差（ATE）は 2.773m、高級の IMU で 0.858m、ステレオと IMU では 0.229m・0.168m。結論として、視覚が中心の手法には、IMU を組み合わせないグローバルシャッターのステレオを勧めている |
| F-N15 | Aditya et al. 2025（要旨と、本文の一部（実験の機体とセンサー）を読んだ。arXiv 2505.02272）: 安価な四脚ロボット向けに、接地を使う脚の幾何のオドメトリ・VIO・深度で安定させた視覚を組み合わせ、2D の地図を作って位置を推定し、自分で移動する。実験の機体は Unitree Go2 と MAB Silver Badger（ほかに Go1・A1・Honey Badger でも試した）。センサーは深度カメラ（Intel RealSense D435i）、機体の IMU、関節のエンコーダー |
| F-N16 | Barkley et al. 2025（要旨だけ読んだ。arXiv 2505.01931）: Petoi Bittle と Raspberry Pi Zero 2W、上から見るカメラ（overhead camera）で、GPT-4 と A* の経路計画を比べた。要旨からは、Bittle に載せたカメラで位置を推定したとは読めない |

出典:
- F-N1: [Baddeley et al. 2012, PLoS Comput Biol](https://doi.org/10.1371/journal.pcbi.1002336)（CC BY。Europe PMC の全文 PMC3252273）
- F-N2: [Ardin et al. 2016, PLoS Comput Biol](https://doi.org/10.1371/journal.pcbi.1004683)（CC BY。PMC4750948）
- F-N3: [Sun et al. 2020, eLife](https://doi.org/10.7554/eLife.54026)（CC BY。PMC7365663）、[InsectNavigationToolkitModelling](https://github.com/XuelongSun/InsectNavigationToolkitModelling/tree/470c7d783dc5678e22170f582b09a72b265071cb)
- F-N4: [Wystrach et al. 2016, J Comp Physiol A](https://doi.org/10.1007/s00359-015-1052-1)（CC BY。PMC4722065）
- F-N5: [van Dijk et al. 2024, Science Robotics](https://doi.org/10.1126/scirobotics.adk0310)（Europe PMC の要旨）
- F-N6: [Dupeyroux et al. 2019, Science Robotics](https://doi.org/10.1126/scirobotics.aau0307)（Europe PMC の要旨）
- F-N7: [Stone et al. 2017, Current Biology](https://doi.org/10.1016/j.cub.2017.08.052)（Europe PMC の要旨）、[path-integration](https://github.com/InsectRobotics/path-integration/tree/d0657cbc50de7eb437569da68ae4e30a6572d9f2)
- F-N8: [Burgard, Fox, Thrun 1999, JAIR](https://arxiv.org/abs/1106.0222)（arXiv の要旨）
- F-N9: [davidmball/ratslam](https://github.com/davidmball/ratslam/tree/9050b9531df5587e464fbbec73f9340149c4b660)（README、`license.txt`、`config/config_stlucia.txt.in`）
- F-N10: [arXiv 2107.07243](https://arxiv.org/abs/2107.07243)
- F-N11: [arXiv 2209.07654](https://arxiv.org/abs/2209.07654)、[Cerberus](https://github.com/ShuoYangRobotics/Cerberus/tree/ad56c433a9a95aa47b1a436fb92c8e14c18965bd)（README）
- F-N12: [arXiv 2111.15164](https://arxiv.org/abs/2111.15164)
- F-N13: [arXiv 2007.11898](https://arxiv.org/abs/2007.11898)、[ORB_SLAM3](https://github.com/UZ-SLAMLab/ORB_SLAM3/tree/4452a3c4ab75b1cde34e5505a36ec3f9edcdc4c4)（`LICENSE`）
- F-N14: [arXiv 2606.19067](https://arxiv.org/abs/2606.19067)（HTML 版の III-A・IV-C・V 節）
- F-N15: [arXiv 2505.02272](https://arxiv.org/abs/2505.02272)（HTML 版の IV 節）
- F-N16: [arXiv 2505.01931](https://arxiv.org/abs/2505.01931)
