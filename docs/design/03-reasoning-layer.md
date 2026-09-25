# 03. 理性層（Jev）

## 1. 役割（設計案）

自由文（性格・スキル・その場の指示）と現在の状況を読み、次の動作を選ぶ。
手順の実行や優先順位の判定はしない（F-J1、F-J8）。

## 2. 入力: `state`（設計案）

`state` は JSON オブジェクトにする（F-J2）。無関係な情報は入れない（F-J8）。

```json
{
  "personality": "<人が書いた性格の文章>",
  "skills": ["<スキル名>: <人が書いた説明>", "..."],
  "situation": "<コードが作った現在の状況の説明>",
  "instruction": "<その場の指示。無ければ null>"
}
```

- `situation` はセンサー値をそのまま入れず、コード側で文章やカテゴリに変換してから入れる（数の比較が苦手なため。F-J8）。
- 日本語の文章を英訳して入れるかどうかは、実験 E05 の結果で決める（D-04）。

## 3. 質問（設計案）

1回の呼び出しにまとめる（F-J5）。

| キー | 型 | 内容 |
|---|---|---|
| `instruction_applies` | Noul | その場の指示は、今の状況に当てはまるか |
| `next_action` | Choice | 次の動作（選択肢は `Action` と、定義済みスキル） |

- Choice の選択肢の説明（`criteria`）は人が書く。
- Choice の `probabilities` は判断には使わず、ログにだけ残す（調停は `choice` と `confidence` で行う。必要になったら `ReasonedDecision` に足す）。
- Choice の答えは `Command`（単一の動作か、スキル。[02](02-architecture.md) 3節）に変換する。名前が `Action` にもスキルにも無い場合は、判断無しとして扱う（`Err` を返し、ループは止めない）。
- 指示に従った選択と、性格だけに基づく選択を、仕組みとしてどう分けるかは未決定（D-11、[05](05-arbitration.md) 4節）。

## 4. 呼び出し方（設計案。D-10）

HTTPS の API を直接呼ぶ。Python SDK（F-J7）は使わない。
リクエストとレスポンスの形は、fly-brain リポジトリの `jev.py` が呼んでいる形（F-J9）に合わせる。
**F-J9 は第三者のコードから読んだ二次情報で、公式ドキュメントでは未確認**。E05 で実際に呼んで確かめる。

```
POST https://api.typesafe.ai/v1/systemone
Authorization: Bearer <API キー>
Content-Type: application/json

{ "model": "jev-latest",
  "state": { ... },
  "questions": {
    "instruction_applies": { "type": "noul", "instructions": "..." },
    "next_action": { "type": "choice", "instructions": "...", "criteria": { "walk": "...", ... } } } }

→ { "answers": { "next_action": { "choice": "...", "probabilities": {...}, "confidence": ... }, ... },
    "model": "...", "usage": {...} }
```

`"type": "noul"` は推測（`jev.py` には choice の例しかない）。

## 5. 境界の決まり（設計案）

- Jev の API を呼ぶのは `walk-jev` クレートだけ。HTTP クライアントを依存に持つのもこのクレートと `walk-bittle` だけにする。
- 応答は serde で専用の型に変換し、値の範囲（確率が 0〜1 など）を検査してから `ReasonedDecision` を作る。形が違えば `Err`。
- それ以外のクレートは `Reasoner` trait（[02](02-architecture.md)）だけを見る。
- 単体テストでは本物の API を呼ばない。記録済みの応答（fixture）から `ReasonedDecision` への変換を検証する。
- 本物の API を呼ぶテストは、明示的に有効にしたときだけ手元で実行する（方法は開発ルールで決める）。

## 6. 未確認・実験

- E05: 日本語の性格文がどこまで効くか（英訳との比較）。あわせて、F-J9 の呼び方で実際に呼べるか、Noul の `type` の書き方を確かめる。
- HTTP API の仕様は、公式ドキュメントで確かめて F-J9 を直す。確かめられず、E05 でも呼べなかった場合は D-10 を開き直す。
