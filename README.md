# petoi-the-dog-walk

初代 Petoi Bittle（NyBoard）を、Mac から無線で動かす。

- **理性層**: TypeSafe の [Jev](https://docs.typesafe.ai/introduction) が、人の書いた自由文（性格・スキル・その場の指示）を読んで次の動作を選ぶ。
- **反射層**: maleCNS コネクトームの [ハエ脳シミュレーション](https://github.com/lavallee/mk-jev-fly-brain) が、100ms ごとの速い反応を出す（後から追加。無くても動く）。
- **優先順位**: その場の指示 ＞ 性格に基づく選択 ＞ 反射。判定は Jev ではなくコードで行う。

## 状態

フェーズ0（ハーネス作り）。本体の実装はまだ無い。進め方は [docs/design/07-roadmap.md](docs/design/07-roadmap.md)。

## ディレクトリ

| 場所 | 内容 |
|---|---|
| [docs/design/](docs/design/) | 設計書（概要、構成、各層、調停、通信、決定事項、確認済みの事実） |
| [docs/rules/](docs/rules/) | 開発ルール（テスト、静的検査、レビュー、Git、文書） |
| [experimentals/](experimentals/) | 人に依頼する実験の手順と結果 |
| `src/petoi_walk/` | 本体（Python） |
| `tests/` | 自動テスト |
| `scripts/check.sh` | CI と同じ検査を手元で実行する |
| `.claude/agents/` | レビュー用サブエージェントの定義 |

## 開発

必要なもの: [uv](https://docs.astral.sh/uv/)（Python 3.11 以上は uv が用意する）

```sh
uv sync
./scripts/check.sh   # ruff format / ruff check / mypy --strict / pytest
```

ルールは [docs/rules/](docs/rules/)、AI エージェント向けの要約は [CLAUDE.md](CLAUDE.md)。

## 参考リンク

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
