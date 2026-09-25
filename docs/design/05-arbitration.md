# 05. 調停（優先順位）

## 1. 規則（設計案。[計画書](https://github.com/kogai/petoi-the-dog-walk/blob/5174b74/README.md) 3.1 から）

1. その場の自由文の指示が、今の状況に当てはまる場合は、それを最優先にする。
2. 当てはまらない間は、性格に基づく Jev の選択を使う。
3. 反射層の出力は、上の2つが無い場合のみ採用する。
4. 優先順位の判定は Jev に任せず、コード側で行う。
   （計画書では「Jev の公式ドキュメントも、複数の判断を組み合わせる役割はコード側に置く考え方」としていた。
   該当ページは記録されておらず **未確認**。規則4は、F-J8 の弱点（矛盾する指示・間接的な指示が苦手）だけでも成り立つ）

## 2. 純粋関数として定義する（設計案）

```rust
// walk-core
pub fn arbitrate(
    now: Millis,
    reasoned: Option<&ReasonedDecision>,
    reflex: Option<&ReflexOutput>,
    policy: &ArbiterPolicy, // 閾値・有効期限・D-03 の方式など
) -> Arbitration;           // 採用した Command と、採用理由（どの規則か）

pub enum Rule {
    Safety, // D-01 が決まるまで使わない（入れるかどうか自体が未決定）
    Instruction, Personality, Reflex, Default,
}
```

- I/O・時計・乱数を持たない。同じ入力には必ず同じ出力を返す。
- 時刻の差は `saturating_sub` で計算する（`at` が `now` より後でも panic しない）。
- 採用理由（`Rule`）を必ず返し、ログに残す。

## 3. 不変条件（プロパティテストで検証する）

- P1: `Rule::Safety` に当たらない限り、`reasoned` が有効で `instruction_applies >= policy.instruction_threshold` なら、規則は `Rule::Instruction`。
- P2: 規則が `Rule::Reflex` になる条件は、D-03 の方式ごとに決める。方式 (a) なら「有効な `reasoned` が無いときだけ」、方式 (b) なら「有効な `reasoned` が無いか、`confidence` が閾値未満のときだけ」。D-03 が決まるまでは、どちらの方式でも成り立つことをテストする。
- P3: 有効期限切れ（`now - at > policy.ttl`）の入力は、無いのと同じに扱う。
- P4: 入力がすべて無ければ、既定動作（`Command::Action(Action::Balance)`）を返す。
- P5: 出力の `Command` は必ずトークン列に変換できる（`Action` から `Token` への変換とスキルの展開が全域関数であることを型で保証する）。

## 4. 未決定

- D-01: 安全上の例外（転倒しそうなときなど）の扱い。[decisions.md](decisions.md) を参照。
- D-03: 規則3の「上の2つが無い場合」の定義。Jev が正常に動いていると、規則2は常に何かを選ぶので、反射層が一度も採用されない。
  候補:
  - (a) Jev の判断が有効期限切れ・失敗のときだけ
  - (b) Jev の `confidence` が閾値未満のとき
  - (c) 反射層の出力が「stand 以外」なら、有効期限内の Jev 判断より優先（計画書と逆になる）
  → 実装はまず (a) と (b) を `ArbiterPolicy` で切り替えられる形にして、実験 E08 で決める。
- D-11: 規則1（指示）と規則2（性格）の仕組みの分け方。今の質問は `next_action` が1つだけなので、指示が当てはまらないときも、
  `state` に入った指示に選択が引きずられうる（F-J8: 間接的・矛盾する指示が苦手）。そうなると2つの規則の違いは採用理由のラベルだけになる。
  候補:
  - (a) 同じ呼び出しで Choice を2つ出す（指示に従う場合の選択と、指示を無視して性格だけで選ぶ場合の選択）
  - (b) Noul で当てはまらないと出たら、指示を外した `state` でもう一度問う（呼び出しが2回になる）
  - (c) そもそも2回に分けて呼ぶ（指示あり・なし）
  → E05 で (a) の2つの Choice が区別されるかを見て決める。
