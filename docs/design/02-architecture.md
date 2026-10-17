# 02. アーキテクチャ（設計案）

この文書の内容はすべて **設計案**（実装前・未実行）。言語は Rust（[D-09](decisions.md)）。

## 1. 全体像

```
 [人が書く自由文]  profiles/<名前>/  性格 / スキル / その場の指示
        │
        ▼
 ┌────────────────────────────────────────────────────────────────┐
 │ Mac (Rust)                                                     │
 │                                                                │
 │  sensing ──Observation──┬──▶ reasoner (Jev, 別スレッド) ─ReasonedDecision─┐
 │     ▲                   │                                                 ▼
 │     │                   └──▶ reflex (ハエ脳)  ──ReflexOutput──────▶ arbitrate ──Action──▶ executor
 │     │                                                              (純粋関数)              │
 │     └──────────────────────────── transport ◀──────────── Token 列 ◀─────────────────────┘
 └───────────────────────────────┬────────────────────────────────┘
                                 │ Wi-Fi (HTTP) / Bluetooth・USB (シリアル)
                                 ▼
                      Bittle (NyBoard, ATmega328P)
```

図の `Token 列` は、[ADR-0001](adr/0001-driver-layer.md)（D-16）以後は `Action` の並び。トークンへの変換は `transport` の箱の中のドライバーが行う（図そのものは変えない）。

## 2. クレートの分け方

Cargo のワークスペースにし、責務ごとにクレートを分ける。
依存の向きは Cargo.toml に書いたものしか許されないので、境界はコンパイラが守る。

| クレート | 役割 | 外部 I/O | 依存してよいもの |
|---|---|---|---|
| `walk-core` | 型（`Action`、`Command`、`Observation` など）、観測から Jev の `situation` を作る変換、調停（[05](05-arbitration.md)）、スキルの展開、ハエ脳出力の変換、境界の trait、本番でも使う「何もしない」実装（反射層なし・理性層なし） | なし | なし（`#![no_std]` + `alloc`） |
| `walk-jev` | Jev のドライバー（2.3節）: Jev への質問の組み立て、HTTP 呼び出し、応答の検査（[03](03-reasoning-layer.md)） | Jev API | `walk-core`、HTTP クライアント、serde |
| `walk-bittle` | Bittle のドライバー（2.3節）: `Token` 型と動作→トークン表、送信（[06](06-transport.md)）、センサーの読み取りと、それが使うリンク（シリアル）。Wi-Fi（HTTP）の扱いは D-17 | シリアル / HTTP | `walk-core`、serialport、HTTP クライアント |
| `walk-flybrain` | ハエ脳のドライバー（2.3節。[04](04-reflex-layer.md)、フェーズ3） | 未定 D-06 | `walk-core` |
| `walk-app` | 起動、設定と文章の読み込み、ループ、非常停止（実行ファイル） | ファイル・時計・端末 | 上のすべて |
| `walk-testing` | Fake（境界の trait と `Link` の偽物）と、テスト用の偽の時計（`dev-dependencies` 専用） | なし | `walk-core` |
| `walk-experiments` | 人が実験で使う道具（`experimentals/`）。本体ではない | 実験ごと（シリアル・HTTP） | ワークスペースのクレートには依存しない |

- `walk-testing` は `walk-core` に依存するので、`walk-core` 自身のテストでは使わない（循環になる）。`walk-core` のテストは純粋関数を直接呼ぶので Fake が要らない。Fake を使うテストは、殻のクレートの `tests/` に置く。

### 2.1 純粋な核（`walk-core`）

- `#![no_std]` にする。標準ライブラリのファイル・ネットワーク・時計・スレッドが**使えない**ので、I/O が混ざるとコンパイルが通らない。
- 依存クレートを持たない。判断のロジックはすべてここに置き、純粋関数として書く。
- 失敗は `Result` で返す。`panic!`・`unwrap` は使わない（lint で禁止する）。

### 2.2 殻（`walk-jev`、`walk-bittle`、`walk-flybrain`、`walk-app`）

- 外部 I/O はここだけで行う。外から来たデータは serde で専用の型に変換し、検査を通ったものだけを `walk-core` の型にして渡す。
- 殻は薄く保つ。条件分岐が増えてきたら、判断の部分を `walk-core` の純粋関数に移す。

### 2.3 ドライバー層（[ADR-0001](adr/0001-driver-layer.md)、D-16）

1節の図は変えない。図の各箱の中を「境界の trait（`walk-core`）・ドライバー（デバイスや API ごとの殻のクレート）・リンク（バイトの流れの通り道）」の3段に分ける。詳しい中身、テストでの差し替え方、安全の不変条件は [ADR-0001](adr/0001-driver-layer.md) の1か所に書く。要点:

- 上の層は、境界の trait だけを見る。動かす指示は意味（`Action`）で渡し、デバイスのトークンへの変換はドライバーが持つ。
- バイトの流れで話すデバイスのドライバーは、`Link` と `Clock` を受け取る。HTTP で話すドライバーの形は D-17。
- 細部の未決定は D-17。

## 3. 型と境界（スケッチ）

```rust
// walk-core: 値は不変。列挙は enum で表し、match の網羅性をコンパイラが検査する。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action { Balance, Walk, Back, Sit, Hello, Stop /* 今は Bittle（F-T1）で出せるものから選ぶ。別のロボットで対応しないものは ADR-0001 S11・D-17 (5) */ }

/// Jev が選ぶもの。単一の動作か、定義済みのスキル（`Action` の並び。D-05 の見直し、ADR-0001）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command { Action(Action), Skill(SkillId) }

/// 単調時計の時刻（ミリ秒）。ただの u64 と取り違えないように型を分ける。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Millis(pub u64);

/// 時間の長さ（ミリ秒）。時刻（`Millis`）と取り違えないように型を分ける（ADR-0001 S7）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MillisSpan(pub u64);

pub struct Observation {
    pub at: Millis,
    pub instruction: Option<String>, // その場の指示（自由文）
    // imu などは E01 の結果で決める
}

pub struct ReasonedDecision {
    pub at: Millis,
    pub instruction_applies: Probability, // Noul の値。0..=1 を検査済みの型
    pub command: Command,
    pub confidence: Probability,
}

pub struct ReflexOutput {
    pub at: Millis,
    pub action: Option<Action>, // None は stand（何もしない）
}

// 境界の trait（実装は殻のクレートと walk-testing に置く）
pub trait Reasoner: Send { // 別スレッドで回すので Send
    fn decide(&mut self, obs: &Observation) -> Result<ReasonedDecision, ReasonError>;
}
pub trait Reflex {
    fn step(&mut self, obs: &Observation) -> ReflexOutput;
}
pub trait Transport {
    // 意味で受け取る。デバイスのトークンへの変換はドライバーが持つ（ADR-0001）。応答の待ち方は D-15
    fn send(&mut self, action: Action) -> Result<(), TransportError>;
    // 停止を送る（Bittle では d）。send の待ちを追い越せる（ADR-0001 S3）
    fn stop(&mut self) -> Result<(), TransportError>;
}

// ドライバー層の下側（ADR-0001）。no_std のまま書ける形にする（std::io を使わない）。
// 読み手と書き手に分けるか、Send が要るかは D-15・D-17
pub trait Link {
    // 上限の時間までに全部書けたら Ok、そうでなければ Err。中で再試行しない（ADR-0001 S4）
    fn write(&mut self, bytes: &[u8], timeout: MillisSpan) -> Result<(), LinkError>;
    // 上限の時間までに届いた分を buf に入れて、その長さを返す。何も届かなければ 0
    fn read(&mut self, buf: &mut [u8], timeout: MillisSpan) -> Result<usize, LinkError>;
}
pub trait Clock {
    fn now(&self) -> Millis; // 単調に増える
}

// walk-bittle（Bittle のドライバー）の中:
/// 表に載ったトークンだけを表す。フィールドを公開しないので、任意の文字列からは作れない。
/// 作れるのは、ドライバーの `Action` → トークン表（F-T1 のうち使うもの）からだけ。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token(&'static str);
```

## 4. 時間とスレッド

- 反射層と調停は 100ms 周期（F-F1 に合わせる）。反射層の計算が周期を超えたときの扱いは D-06 で決めた（E17 で、JS の実装に 100ms 以上の窓があった。F-F19）。
- 理性層の所要時間は、cookbook の例で 0.09〜0.31 秒（F-J6）、fly-brain の README では約350ms（F-F5）。このシステムではまだ測っていない。
  100ms 周期を超えうるので、理性層は別スレッドで回し、結果をチャネルで送る。ループは最新の結果だけを使う。
- 非同期ランタイム（tokio など）は使わない。標準のスレッドとチャネルで足りる。
- 各判断には時刻（`Millis`）を付け、古い判断は調停で捨てる（有効期限は D-03）。
- 本物の `Clock` の実装は `walk-app` だけに置き、時刻は値として `walk-core` に渡す。ドライバーが時間切れを測るときは、`walk-app` から `Clock` を受け取る（2.3節）。テストでは偽の時計を使い、実時間を待たない。

## 5. 失敗時の振る舞い

| 状況 | 振る舞い |
|---|---|
| Jev の呼び出し失敗・タイムアウト | その回の判断は無し扱い。以後の扱いは調停の規則に従う（[05](05-arbitration.md)） |
| API キーが無い | 起動時に理性層を無効にして、反射層だけ（または既定動作だけ）で動く |
| 送信失敗（書き込みの失敗） | 再試行は1回まで。続けて失敗したら停止してループを抜ける。応答の時間切れ・知らないトークンの応答では、同じトークンを送り直さない（[06](06-transport.md) 2.1節） |
| 送信失敗で止めるとき | 停止トークン自体が届かない可能性がある。届かなかったことを端末にはっきり表示し、電源を切るよう促す |
| 終了・Ctrl-C・panic | 停止（Bittle では `d`、F-T1）を送ってから閉じる。通常終了と Ctrl-C・SIGTERM では、`walk-app` が明示して `stop()` を呼ぶ（Ctrl-C・SIGTERM はシグナル処理（`ctrlc` か `signal-hook` クレート）で立てたフラグをループが見て行う）。`Drop` は panic の巻き戻しと最後の備え（ADR-0001 S1）。`panic = "abort"` や SIGKILL では送れないので、ビルド設定で巻き戻しを使い、非常時は電源を切る |

## 6. 設定と文章

- 人が書く文章は `profiles/<名前>/` に置く（例: `personality.md`、`skills.toml`）。形式は D-05。
- 秘密情報（`TYPESAFE_API_KEY`、Bittle の IP など）は環境変数で渡す。リポジトリに入れない。
