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
- Choice の答えの名前が `Action` やスキルに無い場合は、判断無しとして扱う（例外を投げてループを止めない）。

設計スケッチ（未実行。SDK の書き方は公式ドキュメントの例に基づく）:

```python
from typesafe_sdk import Choice, Noul, TypeSafeClient

questions = {
    "instruction_applies": Noul(
        instructions="The user's current instruction applies to the current situation."
    ),
    "next_action": Choice(
        instructions="Which action should the robot take next?",
        criteria={"walk": "Walk forward", "back": "Step backward", "sit": "Sit down"},
    ),
}

with TypeSafeClient() as client:
    response = client.system_one(state=state, questions=questions)
```

## 4. SDK の隔離

- `typesafe-sdk` を import するのは `reasoning/jev.py` の1ファイルだけにする。
- それ以外は `Reasoner` Protocol（[02](02-architecture.md)）だけを見る。
- 単体テストでは本物の API を呼ばない。記録済みの応答（`tests/fixtures/jev/*.json`）から `ReasonedDecision` への変換を検証する。
- 本物の API を呼ぶテストは `@pytest.mark.jev` を付け、API キーがあるときだけ手元で実行する。

## 5. 未確認・実験

- E05: 日本語の性格文がどこまで効くか（英訳との比較）。
- SDK の非同期 API の有無（未確認）。無ければスレッドで包む。
