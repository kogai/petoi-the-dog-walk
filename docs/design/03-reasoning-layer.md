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
- Choice の答えの名前が `Action` やスキルに無い場合は、判断無しとして扱う（例外を投げてループを止めない）。

## 4. 呼び出し方（設計案）

Python SDK は使わず、HTTPS の API を `fetch` で直接呼ぶ。
リクエストとレスポンスの形は、fly-brain リポジトリの `jev.py` で確認したもの（F-J9）。公式ドキュメントでは未確認。

設計スケッチ（未実行）:

```ts
const res = await fetch("https://api.typesafe.ai/v1/systemone", {
  method: "POST",
  headers: { Authorization: `Bearer ${apiKey}`, "Content-Type": "application/json" },
  body: JSON.stringify({
    model: "jev-latest",
    state,
    questions: {
      instruction_applies: {
        type: "noul", // 文字列 "noul" は推測（jev.py には choice の例しかない）→ E05
        instructions: "The user's current instruction applies to the current situation.",
      },
      next_action: {
        type: "choice",
        instructions: "Which action should the robot take next?",
        criteria: { walk: "Walk forward", back: "Step backward", sit: "Sit down" },
      },
    },
  }),
  signal: AbortSignal.timeout(timeoutMs),
});
// 応答: { answers: { next_action: { choice, probabilities, confidence }, ... }, model, usage }
```

## 5. 境界の決まり（設計案）

- TypeSafe の API を呼ぶのは `src/reasoning/jev.ts` の1ファイルだけにする（dependency-cruiser で検査）。
- 応答は境界でスキーマ検査し、`Result<ReasonedDecision, ReasonError>` に変換する。形が違えば `ok: false`。
- それ以外は `Reasoner` 型（[02](02-architecture.md)）だけを見る。
- 単体テストでは本物の API を呼ばない。記録済みの応答（`tests/fixtures/jev/*.json`）から `ReasonedDecision` への変換を検証する。
- 本物の API を呼ぶテストは `*.jev.test.ts` に書き、API キーがあるときだけ手元で実行する（`pnpm test:jev`）。

## 6. 未確認・実験

- E05: 日本語の性格文がどこまで効くか（英訳との比較）。あわせて、Noul の `type` 文字列と応答の形を確かめる。
- HTTP API の仕様は、公式ドキュメントで確かめ直す（この環境からは公式ドキュメントに届かなかった）。
