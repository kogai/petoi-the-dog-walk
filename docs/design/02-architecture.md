# 02. アーキテクチャ（設計案）

この文書の内容はすべて **設計案**（実装前・未実行）。

## 1. 全体像

```
 [人が書く自由文]  profiles/<名前>/  性格 / スキル / その場の指示
        │
        ▼
 ┌───────────────────────────────────────────────────────────┐
 │ Mac (Python, パッケージ petoi_walk)                        │
 │                                                           │
 │  sensing ──Observation──┬──▶ reasoning (Jev)  ─ReasonedDecision─┐
 │     ▲                   │                                       ▼
 │     │                   └──▶ reflex (ハエ脳)  ─ReflexOutput──▶ arbiter ──Action──▶ executor
 │     │                                                            (純粋関数)            │
 │     └──────────────────────────── transport ◀──────────── Token 列 ◀──────────────────┘
 └───────────────────────────────┬───────────────────────────┘
                                 │ Wi-Fi (HTTP) / Bluetooth・USB (シリアル)
                                 ▼
                      Bittle (NyBoard, ATmega328P)
```

## 2. モジュール

`src/petoi_walk/` 以下に置く。依存の向きは上から下だけ（下のモジュールは上を import しない）。

| モジュール | 役割 | 外部 I/O | テストの主な手段 |
|---|---|---|---|
| `app` | 起動、設定読込、ループの組み立て、非常停止 | あり | 結合テスト（Fake 一式） |
| `loop` | 周期実行。理性層は非同期、反射層・調停は 100ms 周期 | 時計 | 偽の時計で決定的に回す |
| `arbiter` | 優先順位の判定（[05](05-arbitration.md)） | なし | 単体 + プロパティテスト |
| `reasoning` | Jev への質問の組み立てと答えの解釈（[03](03-reasoning-layer.md)） | Jev API | 記録済み応答（fixture） |
| `reflex` | ハエ脳の出力を動作に変換（[04](04-reflex-layer.md)） | 未定 D-06 | Fake / 記録済み出力 |
| `skills` | スキル＝トークン列の定義と展開 | なし | 単体 |
| `profiles` | 性格・スキル・指示の文章を読み込む | ファイル | 単体（tmp_path） |
| `sensing` | IMU などの読み取り（範囲は E01・E02 で決まる） | Bittle | 記録済みバイト列 |
| `transport` | Bittle へトークンを送る（[06](06-transport.md)） | シリアル / HTTP | Fake + 記録済み応答 |
| `domain` | 型の定義（`Action`、`Token`、`Observation` など）と動作→トークン表 | なし | 単体 |

## 3. 境界のインターフェース（スケッチ）

外部 I/O を持つモジュールは `typing.Protocol` で境界を定義し、テストでは Fake に差し替える。
実装は Fake を先に書き、本物はその後に書く（[rules/testing.md](../rules/testing.md)）。

```python
from typing import Protocol
from dataclasses import dataclass
from enum import StrEnum


class Action(StrEnum):
    BALANCE = "balance"
    WALK = "walk"
    BACK = "back"
    SIT = "sit"
    HELLO = "hello"
    STOP = "stop"
    # ... facts.md F-T1 の動作名から、使うものだけを足す


@dataclass(frozen=True)
class Observation:
    at_ms: int              # 単調時計
    instruction: str | None # その場の指示（自由文）
    # imu などは E01 の結果で決める


@dataclass(frozen=True)
class ReasonedDecision:
    at_ms: int
    instruction_applies: float  # Noul の値 0..1
    action: Action
    confidence: float


@dataclass(frozen=True)
class ReflexOutput:
    at_ms: int
    action: Action | None       # None は stand（何もしない）


class Reasoner(Protocol):
    async def decide(self, obs: Observation) -> ReasonedDecision: ...


class Reflex(Protocol):
    def step(self, obs: Observation) -> ReflexOutput: ...


class Transport(Protocol):
    async def send(self, token: str) -> None: ...
    async def close(self) -> None: ...  # 閉じる前に必ず停止トークンを送る
```

## 4. 時間の扱い

- 反射層と調停は 100ms 周期（F-F1 に合わせる）。
- 理性層の所要時間は、cookbook の例で 0.09〜0.31 秒（F-J6）、fly-brain の README では約350ms（F-F5）。このシステムではまだ測っていない。
  100ms 周期を超えうるので、周期とは独立に非同期で回し、最新の結果だけを使う（設計案）。
- 各判断には時刻（`at_ms`）を付け、古い判断は調停で捨てる（有効期限は D-03）。
- 時計は注入する（`Clock` Protocol）。テストでは偽の時計を使い、`sleep` しない。

## 5. 失敗時の振る舞い

| 状況 | 振る舞い |
|---|---|
| Jev の呼び出し失敗・タイムアウト | その回の判断は無し扱い。調停は反射層か既定動作に落ちる |
| API キーが無い | 起動時に理性層を無効にして、反射層だけ（または既定動作だけ）で動く |
| 送信失敗 | 再試行は1回まで。続けて失敗したら停止してループを抜ける |
| Ctrl-C・例外・終了 | 必ず停止トークン（`d`、F-T1）を送ってから閉じる |

## 6. 設定と文章

- 人が書く文章は `profiles/<名前>/` に置く（例: `personality.md`、`skills.yaml`）。形式は D-05。
- 秘密情報（`TYPESAFE_API_KEY`、Bittle の IP など）は環境変数で渡す。リポジトリに入れない。
