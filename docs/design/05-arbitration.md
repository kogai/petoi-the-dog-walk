# 05. 調停（優先順位）

## 1. 規則（設計案。計画書 3.1 から）

1. その場の自由文の指示が、今の状況に当てはまる場合は、それを最優先にする。
2. 当てはまらない間は、性格に基づく Jev の選択を使う。
3. 反射層の出力は、上の2つが無い場合のみ採用する。
4. 優先順位の判定は Jev に任せず、コード側で行う。
   （計画書では「Jev の公式ドキュメントも、複数の判断を組み合わせる役割はコード側に置く考え方」としていた。
   該当ページは記録されておらず **未確認**。規則4は、F-J8 の弱点（矛盾する指示・間接的な指示が苦手）だけでも成り立つ）

## 2. 純粋関数として定義する（設計案）

```ts
export const arbitrate = (
  nowMs: number,
  reasoned: ReasonedDecision | null,
  reflex: ReflexOutput | null,
  policy: ArbiterPolicy, // 閾値・有効期限など
): Arbitration => { /* ... */ }; // 採用した Action と、採用理由（どの規則か）
```

- I/O・時計・乱数を持たない。同じ入力には必ず同じ出力を返す。
- 採用理由（`"instruction" | "personality" | "reflex" | "default" | "safety"` のユニオン型）を必ず返し、ログに残す。

## 3. 不変条件（fast-check のプロパティテストで検証する）

- P1: `reasoned` が有効で `instructionApplies >= policy.instructionThreshold` なら、規則は `instruction`。
- P2: 規則が `reflex` になるのは、有効な `reasoned` が無いときだけ。
- P3: 有効期限切れ（`nowMs - atMs > policy.ttlMs`）の入力は、無いのと同じに扱う。
- P4: 入力がすべて無ければ、既定動作（`balance`）を返す。
- P5: 出力の `Action` は必ずトークン表に存在する。

## 4. 未決定

- D-01: 安全上の例外（転倒しそうなときなど）の扱い。[decisions.md](decisions.md) を参照。
- D-03: 規則3の「上の2つが無い場合」の定義。Jev が正常に動いていると、規則2は常に何かを選ぶので、反射層が一度も採用されない。
  候補:
  - (a) Jev の判断が有効期限切れ・失敗のときだけ
  - (b) Jev の `confidence` が閾値未満のとき
  - (c) 反射層の出力が「stand 以外」なら、有効期限内の Jev 判断より優先（計画書と逆になる）
  → 実装はまず (a) + (b) を `ArbiterPolicy` で切り替えられる形にして、実験 E08 で決める。
