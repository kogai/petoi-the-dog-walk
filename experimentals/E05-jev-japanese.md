# E05: 日本語の性格文が Jev にどこまで効くか

- 状態: 実施済み
- 依頼日: 2026-09-25
- 実施日: 2026-10-27
- 実施者: オーナー

## 目的

実機を使わずに、3つのことを確かめる。

1. **D-10**: Jev の HTTP API を、F-J9（第三者のコードから読んだ二次情報）の形で本当に呼べるか。Noul の質問の書き方（`"type": "noul"`、推測）が通るか
2. **D-04**: 人が日本語で書いた性格・状況・指示の文を、英訳せずに渡してよいか。
   質問文と選択肢の説明は、本番では開発者が英語で書くので、この実験でも常に英語にする（比べるのは状態の文の言語だけ）
3. **D-11**: 同じ呼び出しで「指示に従う場合の選択（`next_action`）」と「指示を無視して性格だけで選ぶ場合の選択（`personality_action`）」の2つの Choice を出したとき、後者が指示に引きずられないか（[05-arbitration.md](../docs/design/05-arbitration.md) 4節の案 (a)）

## 必要なもの

- TypeSafe の API キー。**持っていなければ、結果の「判定」に「対象外」と書いて終わってよい**
- Xcode Command Line Tools と Rust の開発環境（E00 と同じ）
- このリポジトリの clone
- 費用: 56 回呼び出す（事前確認 2 回 + 本番 54 回）。1回あたりの入力は数百トークン程度の見込みなので、料金（10億トークンあたり42ドル、F-J7）ではごくわずかの見込み（**未確認**）
- 所要時間の目安: 30分（表の記入を含む）

## 安全上の注意（API キーと結果）

- **キーをファイルやコマンドの履歴に残さない。** 次のようにして、画面に表示せずに入力する。
  ```sh
  read -rs TYPESAFE_API_KEY && export TYPESAFE_API_KEY
  ```
  （`read -rs` の後にキーを貼り付けて Enter。画面には何も出ない。）
- 実験が終わったら `unset TYPESAFE_API_KEY` を実行する。
- 道具（`e05-jev-ja`）はキーを表示しない。応答にキーが含まれていても、出力の各行で `<KEY>` に置き換える。
- キーが拒否された（HTTP 401 / 403）場合、道具はそこで止まる。
- 応答の本文には、アカウントや組織の ID、リクエスト ID、メールアドレスなどが入っているかもしれない。貼る前に確かめて伏せる（手順 5）。

## 手順

コマンドは、リポジトリのルート（`Cargo.toml` がある場所）で実行する。

0. **実行する前に**、下の「期待する動作」の表を埋める（結果を見てから埋めると比較にならない）。
1. 道具をビルドする。
   ```sh
   cargo build --release -p walk-experiments --bin e05-jev-ja
   ```
2. キーを入れる（安全上の注意を参照）。
3. 実行する。結果は `experimentals/raw/e05.jsonl`（中身はコミットされない）に、進み具合（`1/54` など）は画面に出る。
   ```sh
   ./target/release/e05-jev-ja > experimentals/raw/e05.jsonl
   ```
   道具（[`e05-jev-ja`](../crates/walk-experiments/src/bin/e05-jev-ja.rs)）は **未実行**（偽のサーバーでは動作を確かめたが、本物の API ではまだ呼んでいない）。
   - 最初に「事前確認」を2回送る（Choice だけの質問と、Noul だけの質問）。どちらかが失敗したら、本番の 54 回は送らずに止まる（料金を無駄にしないため）。
   - 本番は1件ずつ独立していて、途中で時間切れが起きても、その行に `error` が記録されて次へ進む。
4. キーが結果に含まれていないことを確かめる。出力が `0` であればよい（キーはコマンドの引数に出さず、ファイルとして渡している）。
   ```sh
   : "${TYPESAFE_API_KEY:?}" && grep -cFf <(printf '%s\n' "$TYPESAFE_API_KEY") experimentals/raw/e05.jsonl
   unset TYPESAFE_API_KEY
   ```
5. `experimentals/raw/e05.jsonl` の各行の `body` を見て、ID やメールアドレスらしい値があれば `<ID>` などに伏せてから「結果」に貼る。

出力の各行:

- 事前確認の行: `precheck`（`choice` / `noul`）・`status`・`latency_ms`・`body`
- 本番の行: `condition`（`ja-state`: 状態の文だけ日本語 / `en`: すべて英語 / `en-repeat`: `en` をもう一度）・`personality`・`situation`（0: 知らない人が近づく、1: 飼い主が呼ぶ、2: 何も起きない）・`instruction`（指示の有無）・`status`・`latency_ms`・`body`
- 失敗した行には `error`（応答が無い）か `body_error`（本文を読めない）が入る

### 期待する動作（実行前に依頼者が記入）

`walk` / `back` / `sit` / `hello` / `balance` のどれかを書く。指示は「人が近づいたら座って」。

| 性格 | 状況 0: 知らない人が近づく（指示なし / あり） | 状況 1: 飼い主が呼ぶ（なし / あり） | 状況 2: 何も起きない（なし / あり） |
|---|---|---|---|
| shy |  /  |  /  |  /  |
| friendly |  /  |  /  |  /  |
| lazy |  /  |  /  |  /  |

## 記録すること

- [ ] 環境: Mac の OS と CPU、`rustc --version`、このリポジトリのコミット（`git rev-parse --short HEAD`）
- [ ] 手順 0 の「期待する動作」の表（実行前に記入したもの）
- [ ] 手順 3 の出力全体（伏せた上で。途中で止まった場合は、止まるまでの行と画面の最後の表示）
- [ ] 手順 4 の出力（`0` であること）
- [ ] 直感と合わない選択があれば、どれか

## 判定基準

用語:

- 「組」: 同じ性格・状況・指示の有無の組み合わせ（18 組）
- 「期待どおり」: `next_action` の `choice` が、手順 0 の表と一致すること
- 「自己一致」: `en` と `en-repeat` で、同じ組の `next_action` の `choice` が一致すること（Jev の答えのばらつきの目安）

3つの目的は独立に判定し、結果を併記する。

**D-10（呼べるか）**

| 結果 | 設計への影響 |
|---|---|
| 事前確認が2つとも `status` 200 で、`body.answers` に質問のキーがある | D-10 を「決定」にする（公式ドキュメントでも確かめた上で）。F-J9 と Noul の書き方を事実として記録する |
| Choice の事前確認は通り、Noul の事前確認が 400 などで失敗した | Noul の書き方が違う。`body` のエラー文を記録し、公式ドキュメントで確かめて直してから再依頼する |
| Choice の事前確認も失敗した（401 / 403 以外） | F-J9 の呼び方が違う。`body` のエラー文を記録し、D-10 を開き直す |
| 401 / 403 | キーの問題。キーを確かめて再依頼する |
| 本番の 54 行のうち 49 行以上が 200 | D-04・D-11 の判定に進む（失敗した行は除いて数える） |
| 本番の 200 が 48 行以下（5xx・429・時間切れが多い） | D-04・D-11 は判定保留。時間をおいて再依頼する |

**D-04（英訳するか）**（D-10 で判定に進めた場合だけ）

| 結果 | 設計への影響 |
|---|---|
| `ja-state` の期待どおりの数と `en` の期待どおりの数の差が 1 以下、かつ `ja-state` と `en` の一致の割合が自己一致の割合より 15 ポイント以上低くない | 日本語のまま渡す |
| `en` の期待どおりの数が `ja-state` より 2 以上多い | 状態の文を英訳して渡す。英訳の方法（人が書く／機械翻訳）を別に決める |
| 上のどちらにも当てはまらない、または自己一致が 12 組未満（ばらつきが大きすぎる） | 判定保留。性格文を増やすか、同じ組を複数回呼ぶ形で再実験するかを相談する |
| 指示ありの組の `instruction_applies` の値（`body.answers.instruction_applies` の中の数値）の平均が、`ja-state` で `en` より 0.2 以上低い | （上と併記）少なくとも指示文は英訳する。数値の形が想定と違えば、この行は判定保留 |

**D-11（指示と性格を分けられるか）**（D-10 で判定に進めた場合だけ。`en` の結果で見る）

同じ性格・状況で、指示なしの行と指示ありの行の `personality_action` の `choice` を比べる。
lazy は性格どおりでも座る（`sit`）ことが多く、引きずられたか区別できないので、shy と friendly の 6 組で見る。

| 結果 | 設計への影響 |
|---|---|
| 6 組中 5 組以上で、指示の有無で `personality_action` が変わらない | 案 (a)（2つの Choice を同じ呼び出しで出す）を採る |
| 6 組中 3 組以上で、指示ありのときだけ `personality_action` が `sit` に変わる | 案 (a) は使えない。案 (b) か (c)（指示を外して問い直す）にする |
| どちらとも言えない | 判定保留。E08 で実機の振る舞いを見て決める |

**全体**

| 結果 | 設計への影響 |
|---|---|
| API キーを持っていない（対象外） | D-04・D-10・D-11 は保留のまま。フェーズ 1b は記録済み応答（偽の応答）で進める |
| 手順どおりに進まなかった・判定できない | 再依頼する。それまで D-04・D-10・D-11 は保留のまま |

この数値は目安。結果を見て変える場合は、理由を「判定」に書く。

## 結果（実施者が記入）

貼る前に [public-repo.md](../docs/rules/public-repo.md) に従って確かめる（手順 4・5 を済ませること）。
コミットの前に `git diff --staged` を読み、`scripts/check-public.sh --staged` を通す。

### 環境

- Mac の OS・CPU: 記録なし
- `rustc --version`: 記録なし
- コミット: 記録なし
- 応答のモデル: `jev-1.14.0`（どの行も）

### 期待する動作（手順 0）

記入なし（実行の前に表を埋めていない）。

### 出力

実施者が会話に貼った `experimentals/raw/e05.jsonl` を、エージェントが転記した。手順 4 の出力は `0`（実施者の報告）。ID・メールアドレスらしい値は、転記した行には無かった。

```
{"body":{"answers":{"next_action":{"choice":"back","confidence":0.75,"probabilities":{"back":0.8,"balance":0.0,"hello":0.0,"sit":0.2,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":397,"output_tokens":53}},"latency_ms":234,"precheck":"choice","status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.27,"type":"noul"}},"model":"jev-1.14.0","usage":{"input_tokens":325,"output_tokens":23}},"latency_ms":139,"precheck":"noul","status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.3,"type":"noul"},"next_action":{"choice":"back","confidence":1.0,"probabilities":{"back":1.0,"balance":0.0,"hello":0.0,"sit":0.0,"walk":0.0},"type":"choice"},"personality_action":{"choice":"back","confidence":1.0,"probabilities":{"back":1.0,"balance":0.0,"hello":0.0,"sit":0.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":528,"output_tokens":124}},"condition":"ja-state","instruction":false,"latency_ms":154,"personality":"shy","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.66,"type":"noul"},"next_action":{"choice":"sit","confidence":0.55,"probabilities":{"back":0.36,"balance":0.0,"hello":0.0,"sit":0.64,"walk":0.0},"type":"choice"},"personality_action":{"choice":"back","confidence":1.0,"probabilities":{"back":1.0,"balance":0.0,"hello":0.0,"sit":0.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":540,"output_tokens":124}},"condition":"ja-state","instruction":true,"latency_ms":149,"personality":"shy","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.27,"type":"noul"},"next_action":{"choice":"back","confidence":0.82,"probabilities":{"back":0.86,"balance":0.03,"hello":0.07,"sit":0.01,"walk":0.03},"type":"choice"},"personality_action":{"choice":"back","confidence":0.97,"probabilities":{"back":0.98,"balance":0.01,"hello":0.01,"sit":0.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":527,"output_tokens":124}},"condition":"ja-state","instruction":false,"latency_ms":178,"personality":"shy","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.32,"type":"noul"},"next_action":{"choice":"sit","confidence":0.42,"probabilities":{"back":0.37,"balance":0.01,"hello":0.06,"sit":0.54,"walk":0.02},"type":"choice"},"personality_action":{"choice":"back","confidence":0.97,"probabilities":{"back":0.98,"balance":0.0,"hello":0.01,"sit":0.01,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":539,"output_tokens":124}},"condition":"ja-state","instruction":true,"latency_ms":169,"personality":"shy","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.23,"type":"noul"},"next_action":{"choice":"back","confidence":0.51,"probabilities":{"back":0.61,"balance":0.37,"hello":0.01,"sit":0.0,"walk":0.01},"type":"choice"},"personality_action":{"choice":"back","confidence":0.84,"probabilities":{"back":0.87,"balance":0.12,"hello":0.0,"sit":0.01,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":524,"output_tokens":124}},"condition":"ja-state","instruction":false,"latency_ms":183,"personality":"shy","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.31,"type":"noul"},"next_action":{"choice":"sit","confidence":0.39,"probabilities":{"back":0.32,"balance":0.17,"hello":0.0,"sit":0.51,"walk":0.0},"type":"choice"},"personality_action":{"choice":"back","confidence":0.9,"probabilities":{"back":0.92,"balance":0.06,"hello":0.0,"sit":0.02,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":536,"output_tokens":124}},"condition":"ja-state","instruction":true,"latency_ms":161,"personality":"shy","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.33,"type":"noul"},"next_action":{"choice":"hello","confidence":0.99,"probabilities":{"back":0.01,"balance":0.0,"hello":0.99,"sit":0.0,"walk":0.0},"type":"choice"},"personality_action":{"choice":"hello","confidence":1.0,"probabilities":{"back":0.0,"balance":0.0,"hello":1.0,"sit":0.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":522,"output_tokens":124}},"condition":"ja-state","instruction":false,"latency_ms":151,"personality":"friendly","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.63,"type":"noul"},"next_action":{"choice":"sit","confidence":0.73,"probabilities":{"back":0.01,"balance":0.0,"hello":0.2,"sit":0.78,"walk":0.01},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.99,"probabilities":{"back":0.0,"balance":0.0,"hello":0.99,"sit":0.0,"walk":0.01},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":534,"output_tokens":124}},"condition":"ja-state","instruction":true,"latency_ms":141,"personality":"friendly","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.33,"type":"noul"},"next_action":{"choice":"hello","confidence":0.6,"probabilities":{"back":0.0,"balance":0.01,"hello":0.68,"sit":0.02,"walk":0.29},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.84,"probabilities":{"back":0.0,"balance":0.0,"hello":0.87,"sit":0.0,"walk":0.13},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":521,"output_tokens":124}},"condition":"ja-state","instruction":false,"latency_ms":140,"personality":"friendly","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.26,"type":"noul"},"next_action":{"choice":"hello","confidence":0.37,"probabilities":{"back":0.0,"balance":0.01,"hello":0.5,"sit":0.4,"walk":0.09},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.95,"probabilities":{"back":0.0,"balance":0.0,"hello":0.96,"sit":0.0,"walk":0.04},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":533,"output_tokens":124}},"condition":"ja-state","instruction":true,"latency_ms":137,"personality":"friendly","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.24,"type":"noul"},"next_action":{"choice":"hello","confidence":0.96,"probabilities":{"back":0.0,"balance":0.01,"hello":0.97,"sit":0.0,"walk":0.02},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.99,"probabilities":{"back":0.0,"balance":0.0,"hello":0.99,"sit":0.0,"walk":0.01},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":518,"output_tokens":124}},"condition":"ja-state","instruction":false,"latency_ms":150,"personality":"friendly","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.46,"type":"noul"},"next_action":{"choice":"sit","confidence":0.34,"probabilities":{"back":0.0,"balance":0.1,"hello":0.38,"sit":0.47,"walk":0.05},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.97,"probabilities":{"back":0.0,"balance":0.0,"hello":0.98,"sit":0.0,"walk":0.02},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":530,"output_tokens":124}},"condition":"ja-state","instruction":true,"latency_ms":141,"personality":"friendly","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.27,"type":"noul"},"next_action":{"choice":"sit","confidence":0.75,"probabilities":{"back":0.1,"balance":0.07,"hello":0.03,"sit":0.8,"walk":0.0},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.92,"probabilities":{"back":0.04,"balance":0.02,"hello":0.0,"sit":0.94,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":516,"output_tokens":124}},"condition":"ja-state","instruction":false,"latency_ms":153,"personality":"lazy","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.77,"type":"noul"},"next_action":{"choice":"sit","confidence":1.0,"probabilities":{"back":0.0,"balance":0.0,"hello":0.0,"sit":1.0,"walk":0.0},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.91,"probabilities":{"back":0.04,"balance":0.02,"hello":0.0,"sit":0.93,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":528,"output_tokens":124}},"condition":"ja-state","instruction":true,"latency_ms":158,"personality":"lazy","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.28,"type":"noul"},"next_action":{"choice":"sit","confidence":0.68,"probabilities":{"back":0.0,"balance":0.05,"hello":0.18,"sit":0.74,"walk":0.03},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.97,"probabilities":{"back":0.0,"balance":0.01,"hello":0.01,"sit":0.98,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":515,"output_tokens":124}},"condition":"ja-state","instruction":false,"latency_ms":159,"personality":"lazy","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.35,"type":"noul"},"next_action":{"choice":"sit","confidence":0.85,"probabilities":{"back":0.0,"balance":0.01,"hello":0.07,"sit":0.88,"walk":0.04},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.96,"probabilities":{"back":0.01,"balance":0.01,"hello":0.01,"sit":0.97,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":527,"output_tokens":124}},"condition":"ja-state","instruction":true,"latency_ms":155,"personality":"lazy","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.29,"type":"noul"},"next_action":{"choice":"sit","confidence":1.0,"probabilities":{"back":0.0,"balance":0.0,"hello":0.0,"sit":1.0,"walk":0.0},"type":"choice"},"personality_action":{"choice":"sit","confidence":1.0,"probabilities":{"back":0.0,"balance":0.0,"hello":0.0,"sit":1.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":512,"output_tokens":124}},"condition":"ja-state","instruction":false,"latency_ms":162,"personality":"lazy","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.4,"type":"noul"},"next_action":{"choice":"sit","confidence":0.69,"probabilities":{"back":0.01,"balance":0.18,"hello":0.02,"sit":0.75,"walk":0.04},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.99,"probabilities":{"back":0.0,"balance":0.01,"hello":0.0,"sit":0.99,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":524,"output_tokens":124}},"condition":"ja-state","instruction":true,"latency_ms":145,"personality":"lazy","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.36,"type":"noul"},"next_action":{"choice":"back","confidence":1.0,"probabilities":{"back":1.0,"balance":0.0,"hello":0.0,"sit":0.0,"walk":0.0},"type":"choice"},"personality_action":{"choice":"back","confidence":1.0,"probabilities":{"back":1.0,"balance":0.0,"hello":0.0,"sit":0.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":504,"output_tokens":124}},"condition":"en","instruction":false,"latency_ms":142,"personality":"shy","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.29,"type":"noul"},"next_action":{"choice":"back","confidence":0.65,"probabilities":{"back":0.72,"balance":0.0,"hello":0.0,"sit":0.28,"walk":0.0},"type":"choice"},"personality_action":{"choice":"back","confidence":1.0,"probabilities":{"back":1.0,"balance":0.0,"hello":0.0,"sit":0.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":512,"output_tokens":124}},"condition":"en","instruction":true,"latency_ms":138,"personality":"shy","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.27,"type":"noul"},"next_action":{"choice":"back","confidence":0.9,"probabilities":{"back":0.92,"balance":0.03,"hello":0.03,"sit":0.0,"walk":0.02},"type":"choice"},"personality_action":{"choice":"back","confidence":0.99,"probabilities":{"back":0.99,"balance":0.01,"hello":0.0,"sit":0.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":505,"output_tokens":124}},"condition":"en","instruction":false,"latency_ms":137,"personality":"shy","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.29,"type":"noul"},"next_action":{"choice":"sit","confidence":0.4,"probabilities":{"back":0.41,"balance":0.01,"hello":0.02,"sit":0.52,"walk":0.04},"type":"choice"},"personality_action":{"choice":"back","confidence":0.94,"probabilities":{"back":0.95,"balance":0.0,"hello":0.01,"sit":0.02,"walk":0.02},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":513,"output_tokens":124}},"condition":"en","instruction":true,"latency_ms":184,"personality":"shy","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.18,"type":"noul"},"next_action":{"choice":"balance","confidence":0.81,"probabilities":{"back":0.14,"balance":0.85,"hello":0.0,"sit":0.01,"walk":0.0},"type":"choice"},"personality_action":{"choice":"back","confidence":0.5,"probabilities":{"back":0.6,"balance":0.37,"hello":0.0,"sit":0.03,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":503,"output_tokens":124}},"condition":"en","instruction":false,"latency_ms":146,"personality":"shy","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.17,"type":"noul"},"next_action":{"choice":"sit","confidence":0.37,"probabilities":{"back":0.21,"balance":0.28,"hello":0.0,"sit":0.5,"walk":0.01},"type":"choice"},"personality_action":{"choice":"back","confidence":0.44,"probabilities":{"back":0.55,"balance":0.4,"hello":0.0,"sit":0.04,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":511,"output_tokens":124}},"condition":"en","instruction":true,"latency_ms":148,"personality":"shy","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.43,"type":"noul"},"next_action":{"choice":"hello","confidence":0.96,"probabilities":{"back":0.0,"balance":0.0,"hello":0.97,"sit":0.0,"walk":0.03},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.97,"probabilities":{"back":0.0,"balance":0.0,"hello":0.98,"sit":0.0,"walk":0.02},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":507,"output_tokens":124}},"condition":"en","instruction":false,"latency_ms":146,"personality":"friendly","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.55,"type":"noul"},"next_action":{"choice":"sit","confidence":0.29,"probabilities":{"back":0.0,"balance":0.0,"hello":0.42,"sit":0.43,"walk":0.15},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.95,"probabilities":{"back":0.0,"balance":0.0,"hello":0.96,"sit":0.0,"walk":0.04},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":515,"output_tokens":124}},"condition":"en","instruction":true,"latency_ms":148,"personality":"friendly","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.31,"type":"noul"},"next_action":{"choice":"hello","confidence":0.44,"probabilities":{"back":0.0,"balance":0.07,"hello":0.55,"sit":0.01,"walk":0.37},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.76,"probabilities":{"back":0.0,"balance":0.01,"hello":0.81,"sit":0.0,"walk":0.17},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":508,"output_tokens":124}},"condition":"en","instruction":false,"latency_ms":136,"personality":"friendly","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.29,"type":"noul"},"next_action":{"choice":"sit","confidence":0.42,"probabilities":{"back":0.0,"balance":0.02,"hello":0.25,"sit":0.54,"walk":0.19},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.87,"probabilities":{"back":0.0,"balance":0.0,"hello":0.9,"sit":0.02,"walk":0.08},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":516,"output_tokens":124}},"condition":"en","instruction":true,"latency_ms":161,"personality":"friendly","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.24,"type":"noul"},"next_action":{"choice":"hello","confidence":0.31,"probabilities":{"back":0.0,"balance":0.24,"hello":0.45,"sit":0.0,"walk":0.31},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.75,"probabilities":{"back":0.0,"balance":0.02,"hello":0.8,"sit":0.0,"walk":0.18},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":506,"output_tokens":124}},"condition":"en","instruction":false,"latency_ms":146,"personality":"friendly","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.23,"type":"noul"},"next_action":{"choice":"sit","confidence":0.14,"probabilities":{"back":0.0,"balance":0.24,"hello":0.23,"sit":0.31,"walk":0.22},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.6,"probabilities":{"back":0.0,"balance":0.05,"hello":0.68,"sit":0.0,"walk":0.26},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":514,"output_tokens":124}},"condition":"en","instruction":true,"latency_ms":138,"personality":"friendly","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.25,"type":"noul"},"next_action":{"choice":"sit","confidence":0.79,"probabilities":{"back":0.04,"balance":0.05,"hello":0.07,"sit":0.83,"walk":0.01},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.96,"probabilities":{"back":0.01,"balance":0.01,"hello":0.01,"sit":0.97,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":502,"output_tokens":124}},"condition":"en","instruction":false,"latency_ms":156,"personality":"lazy","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.73,"type":"noul"},"next_action":{"choice":"sit","confidence":0.97,"probabilities":{"back":0.0,"balance":0.01,"hello":0.01,"sit":0.98,"walk":0.0},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.95,"probabilities":{"back":0.02,"balance":0.02,"hello":0.0,"sit":0.96,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":510,"output_tokens":124}},"condition":"en","instruction":true,"latency_ms":172,"personality":"lazy","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.3,"type":"noul"},"next_action":{"choice":"sit","confidence":0.71,"probabilities":{"back":0.0,"balance":0.06,"hello":0.11,"sit":0.77,"walk":0.06},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.99,"probabilities":{"back":0.0,"balance":0.0,"hello":0.01,"sit":0.99,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":503,"output_tokens":124}},"condition":"en","instruction":false,"latency_ms":198,"personality":"lazy","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.41,"type":"noul"},"next_action":{"choice":"sit","confidence":0.92,"probabilities":{"back":0.0,"balance":0.01,"hello":0.01,"sit":0.94,"walk":0.04},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.94,"probabilities":{"back":0.01,"balance":0.02,"hello":0.0,"sit":0.95,"walk":0.02},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":511,"output_tokens":124}},"condition":"en","instruction":true,"latency_ms":152,"personality":"lazy","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.26,"type":"noul"},"next_action":{"choice":"sit","confidence":0.99,"probabilities":{"back":0.0,"balance":0.01,"hello":0.0,"sit":0.99,"walk":0.0},"type":"choice"},"personality_action":{"choice":"sit","confidence":1.0,"probabilities":{"back":0.0,"balance":0.0,"hello":0.0,"sit":1.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":501,"output_tokens":124}},"condition":"en","instruction":false,"latency_ms":156,"personality":"lazy","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.37,"type":"noul"},"next_action":{"choice":"sit","confidence":0.94,"probabilities":{"back":0.0,"balance":0.02,"hello":0.02,"sit":0.95,"walk":0.01},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.99,"probabilities":{"back":0.0,"balance":0.01,"hello":0.0,"sit":0.99,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":509,"output_tokens":124}},"condition":"en","instruction":true,"latency_ms":147,"personality":"lazy","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.37,"type":"noul"},"next_action":{"choice":"back","confidence":1.0,"probabilities":{"back":1.0,"balance":0.0,"hello":0.0,"sit":0.0,"walk":0.0},"type":"choice"},"personality_action":{"choice":"back","confidence":1.0,"probabilities":{"back":1.0,"balance":0.0,"hello":0.0,"sit":0.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":504,"output_tokens":124}},"condition":"en-repeat","instruction":false,"latency_ms":140,"personality":"shy","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.3,"type":"noul"},"next_action":{"choice":"back","confidence":0.69,"probabilities":{"back":0.75,"balance":0.0,"hello":0.0,"sit":0.25,"walk":0.0},"type":"choice"},"personality_action":{"choice":"back","confidence":1.0,"probabilities":{"back":1.0,"balance":0.0,"hello":0.0,"sit":0.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":512,"output_tokens":124}},"condition":"en-repeat","instruction":true,"latency_ms":142,"personality":"shy","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.28,"type":"noul"},"next_action":{"choice":"back","confidence":0.91,"probabilities":{"back":0.93,"balance":0.02,"hello":0.03,"sit":0.0,"walk":0.02},"type":"choice"},"personality_action":{"choice":"back","confidence":0.99,"probabilities":{"back":0.99,"balance":0.01,"hello":0.0,"sit":0.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":505,"output_tokens":124}},"condition":"en-repeat","instruction":false,"latency_ms":155,"personality":"shy","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.31,"type":"noul"},"next_action":{"choice":"sit","confidence":0.41,"probabilities":{"back":0.4,"balance":0.02,"hello":0.02,"sit":0.53,"walk":0.03},"type":"choice"},"personality_action":{"choice":"back","confidence":0.94,"probabilities":{"back":0.95,"balance":0.0,"hello":0.01,"sit":0.02,"walk":0.02},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":513,"output_tokens":124}},"condition":"en-repeat","instruction":true,"latency_ms":145,"personality":"shy","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.2,"type":"noul"},"next_action":{"choice":"balance","confidence":0.8,"probabilities":{"back":0.15,"balance":0.84,"hello":0.0,"sit":0.01,"walk":0.0},"type":"choice"},"personality_action":{"choice":"back","confidence":0.42,"probabilities":{"back":0.54,"balance":0.43,"hello":0.0,"sit":0.03,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":503,"output_tokens":124}},"condition":"en-repeat","instruction":false,"latency_ms":151,"personality":"shy","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.16,"type":"noul"},"next_action":{"choice":"sit","confidence":0.26,"probabilities":{"back":0.2,"balance":0.39,"hello":0.0,"sit":0.41,"walk":0.0},"type":"choice"},"personality_action":{"choice":"back","confidence":0.45,"probabilities":{"back":0.56,"balance":0.4,"hello":0.0,"sit":0.04,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":511,"output_tokens":124}},"condition":"en-repeat","instruction":true,"latency_ms":586,"personality":"shy","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.45,"type":"noul"},"next_action":{"choice":"hello","confidence":0.92,"probabilities":{"back":0.0,"balance":0.0,"hello":0.94,"sit":0.0,"walk":0.06},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.97,"probabilities":{"back":0.0,"balance":0.0,"hello":0.98,"sit":0.0,"walk":0.02},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":507,"output_tokens":124}},"condition":"en-repeat","instruction":false,"latency_ms":139,"personality":"friendly","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.5,"type":"noul"},"next_action":{"choice":"sit","confidence":0.33,"probabilities":{"back":0.0,"balance":0.0,"hello":0.37,"sit":0.46,"walk":0.17},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.95,"probabilities":{"back":0.0,"balance":0.0,"hello":0.96,"sit":0.0,"walk":0.04},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":515,"output_tokens":124}},"condition":"en-repeat","instruction":true,"latency_ms":147,"personality":"friendly","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.32,"type":"noul"},"next_action":{"choice":"hello","confidence":0.41,"probabilities":{"back":0.0,"balance":0.11,"hello":0.53,"sit":0.02,"walk":0.34},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.77,"probabilities":{"back":0.0,"balance":0.01,"hello":0.82,"sit":0.0,"walk":0.17},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":508,"output_tokens":124}},"condition":"en-repeat","instruction":false,"latency_ms":143,"personality":"friendly","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.28,"type":"noul"},"next_action":{"choice":"sit","confidence":0.41,"probabilities":{"back":0.0,"balance":0.01,"hello":0.29,"sit":0.53,"walk":0.17},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.87,"probabilities":{"back":0.0,"balance":0.0,"hello":0.9,"sit":0.02,"walk":0.08},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":516,"output_tokens":124}},"condition":"en-repeat","instruction":true,"latency_ms":170,"personality":"friendly","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.27,"type":"noul"},"next_action":{"choice":"hello","confidence":0.37,"probabilities":{"back":0.0,"balance":0.2,"hello":0.5,"sit":0.0,"walk":0.3},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.79,"probabilities":{"back":0.0,"balance":0.03,"hello":0.83,"sit":0.0,"walk":0.14},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":506,"output_tokens":124}},"condition":"en-repeat","instruction":false,"latency_ms":145,"personality":"friendly","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.27,"type":"noul"},"next_action":{"choice":"sit","confidence":0.18,"probabilities":{"back":0.0,"balance":0.23,"hello":0.2,"sit":0.34,"walk":0.23},"type":"choice"},"personality_action":{"choice":"hello","confidence":0.59,"probabilities":{"back":0.0,"balance":0.05,"hello":0.67,"sit":0.0,"walk":0.28},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":514,"output_tokens":124}},"condition":"en-repeat","instruction":true,"latency_ms":155,"personality":"friendly","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.22,"type":"noul"},"next_action":{"choice":"sit","confidence":0.81,"probabilities":{"back":0.03,"balance":0.04,"hello":0.07,"sit":0.85,"walk":0.01},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.97,"probabilities":{"back":0.01,"balance":0.01,"hello":0.0,"sit":0.98,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":502,"output_tokens":124}},"condition":"en-repeat","instruction":false,"latency_ms":162,"personality":"lazy","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.73,"type":"noul"},"next_action":{"choice":"sit","confidence":0.97,"probabilities":{"back":0.0,"balance":0.01,"hello":0.01,"sit":0.98,"walk":0.0},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.97,"probabilities":{"back":0.01,"balance":0.01,"hello":0.0,"sit":0.98,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":510,"output_tokens":124}},"condition":"en-repeat","instruction":true,"latency_ms":140,"personality":"lazy","situation":0,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.3,"type":"noul"},"next_action":{"choice":"sit","confidence":0.69,"probabilities":{"back":0.0,"balance":0.06,"hello":0.12,"sit":0.75,"walk":0.07},"type":"choice"},"personality_action":{"choice":"sit","confidence":1.0,"probabilities":{"back":0.0,"balance":0.0,"hello":0.0,"sit":1.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":503,"output_tokens":124}},"condition":"en-repeat","instruction":false,"latency_ms":137,"personality":"lazy","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.39,"type":"noul"},"next_action":{"choice":"sit","confidence":0.91,"probabilities":{"back":0.0,"balance":0.01,"hello":0.02,"sit":0.93,"walk":0.04},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.97,"probabilities":{"back":0.0,"balance":0.01,"hello":0.0,"sit":0.98,"walk":0.01},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":511,"output_tokens":124}},"condition":"en-repeat","instruction":true,"latency_ms":139,"personality":"lazy","situation":1,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.25,"type":"noul"},"next_action":{"choice":"sit","confidence":0.99,"probabilities":{"back":0.0,"balance":0.01,"hello":0.0,"sit":0.99,"walk":0.0},"type":"choice"},"personality_action":{"choice":"sit","confidence":1.0,"probabilities":{"back":0.0,"balance":0.0,"hello":0.0,"sit":1.0,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":501,"output_tokens":124}},"condition":"en-repeat","instruction":false,"latency_ms":138,"personality":"lazy","situation":2,"status":200}
{"body":{"answers":{"instruction_applies":{"noul":0.33,"type":"noul"},"next_action":{"choice":"sit","confidence":0.92,"probabilities":{"back":0.0,"balance":0.02,"hello":0.03,"sit":0.94,"walk":0.01},"type":"choice"},"personality_action":{"choice":"sit","confidence":0.99,"probabilities":{"back":0.0,"balance":0.01,"hello":0.0,"sit":0.99,"walk":0.0},"type":"choice"}},"model":"jev-1.14.0","usage":{"input_tokens":509,"output_tokens":124}},"condition":"en-repeat","instruction":true,"latency_ms":147,"personality":"lazy","situation":2,"status":200}
```

### 判定

エージェントが出力から数えた。

- **D-10（呼べるか）**: 事前確認が2つとも `status` 200 で、`body.answers` に質問のキーがある。本番の 54 行もすべて 200。判定基準の1行目に当たる。呼び方（URL と `Authorization: Bearer`）は、公式ドキュメントの [System One](https://docs.typesafe.ai/api/system-one.md) でも確かめた（エージェント、2026-10-27）。
- **D-04（英訳するか）**: 手順 0 の表が無いので、「期待どおり」の数は数えられない。`next_action` の一致は、`ja-state` と `en` が 18 組中 15 組（83%）、自己一致（`en` と `en-repeat`）が 18 組中 18 組（100%）。一致の割合が自己一致より 17 ポイント低く、1行目の条件（15 ポイント以上低くない）を満たさない。判定基準の3行目に当たり、**判定保留**。
  - 食い違った3組: shy・状況 0・指示あり（`ja-state` は `sit`、`en` は `back`）、shy・状況 2・指示なし（`back` と `balance`）、friendly・状況 1・指示あり（`hello` と `sit`）。
  - 指示ありの組の `instruction_applies` の平均は、`ja-state` 0.46、`en` 0.37。`ja-state` の方が低くはないので、4行目（指示文を英訳する）には当たらない。
- **D-11（指示と性格を分けられるか）**: `en` の shy・friendly の 6 組すべてで、指示の有無で `personality_action` が変わらなかった（`en-repeat`・`ja-state` でも 6 組とも同じ）。判定基準の1行目に当たり、案 (a) を採る。

### 気づいたこと

- 応答時間は 136〜586 ms（586 ms は1回だけ。ほかは 234 ms 以下）。
