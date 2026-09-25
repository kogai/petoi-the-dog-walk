# E05: 日本語の性格文が Jev にどこまで効くか

- 状態: 依頼中（TypeSafe の API キーを持っている場合のみ）
- 依頼日: 2026-09-25
- 実施日:
- 実施者:

## 目的

Jev は主な学習言語が英語で、日本語は精度が下がる（F-J8）。
日本語の性格文をそのまま渡す場合と、英訳して渡す場合で、選ばれる動作がどれだけ違うかを見る。
結果で D-04（英訳するか）を決める。実機は使わない。

## 必要なもの

- TypeSafe の API キー。シェルで `export TYPESAFE_API_KEY=...` のように設定する（**変数名は未確認**。SDK の案内と違えばそちらに従う）。
  **キーをスクリプトやファイルに書かない。** このリポジトリは公開されている
- 費用: 36 回呼び出す。入力は1回あたり数百トークン程度なので、料金（10億トークンあたり42ドル、F-J7）ではごくわずかの見込み
- Python 3.11 以上、`pip install "typesafe-sdk>=0.5.7" --extra-index-url https://pypi.typesafe.ai/`
- 所要時間の目安: 30分

## 手順

0. **実行する前に**、下の「期待する動作」の表を埋める（結果を見てから埋めると比較にならない）。
1. 下のスクリプトを `experimentals/e05_jev_ja.py` として保存する（`.gitignore` 済みでコミットされない）。
   **未実行**。SDK の書き方は公式ドキュメントの例に基づく。`system_one` の呼び方と `answers` の構造も **未確認**。
   `instruction` に `None`（JSON の null）を入れてよいかも未確認。エラーになったら、エラーをそのまま結果に貼ってもらえれば、こちらで直す。
2. `python experimentals/e05_jev_ja.py > experimentals/e05_result.txt 2>&1` を実行する。
3. 出力をそのまま「結果」に貼る。API キーが出力に含まれていないことを確認する。

### 期待する動作（実行前に依頼者が記入）

`walk` / `back` / `sit` / `hello` / `balance` のどれかを書く。指示ありの列は「人が近づいたら座って」の場合。

| 性格 | 状況 0: 知らない人が近づく（指示なし / あり） | 状況 1: 飼い主が呼ぶ（なし / あり） | 状況 2: 何も起きない（なし / あり） |
|---|---|---|---|
| shy |  /  |  /  |  /  |
| friendly |  /  |  /  |  /  |
| lazy |  /  |  /  |  /  |

```python
import json

from typesafe_sdk import Choice, Noul, TypeSafeClient

PERSONALITIES = {
    "shy": {
        "ja": "とても臆病。知らないものが近づくと、すぐに後ずさりする。",
        "en": "Very shy. When something unfamiliar approaches, it backs away immediately.",
    },
    "friendly": {
        "ja": "人懐っこい。人を見ると近寄って挨拶する。",
        "en": "Friendly. When it sees a person, it walks up and greets them.",
    },
    "lazy": {
        "ja": "怠け者。できるだけ座っていたい。",
        "en": "Lazy. Wants to stay sitting as much as possible.",
    },
}
SITUATIONS = {
    "ja": ["知らない人が近づいてきた。", "飼い主が名前を呼んだ。", "何も起きていない。"],
    "en": ["A stranger is approaching.", "The owner called its name.", "Nothing is happening."],
}
INSTRUCTIONS = {
    "ja": [None, "人が近づいたら座って。"],
    "en": [None, "Sit down when a person approaches."],
}
# 質問文と選択肢の説明も、言語ごとに用意する（ja 条件を日本語だけにするため）
QUESTIONS = {
    "en": {
        "instruction_applies": Noul(
            instructions="The user's current instruction applies to the current situation."
        ),
        "next_action": Choice(
            instructions="Which action should the robot take next?",
            criteria={
                "walk": "Walk forward",
                "back": "Step backward",
                "sit": "Sit down",
                "hello": "Greet",
                "balance": "Stand still",
            },
        ),
    },
    "ja": {
        "instruction_applies": Noul(instructions="ユーザーのその場の指示は、今の状況に当てはまる。"),
        "next_action": Choice(
            instructions="ロボットが次にとるべき動作はどれか。",
            criteria={
                "walk": "前に歩く",
                "back": "後ろに下がる",
                "sit": "座る",
                "hello": "挨拶する",
                "balance": "その場に立っている",
            },
        ),
    },
}

with TypeSafeClient() as client:
    for p_name, p in PERSONALITIES.items():
        for lang in ("ja", "en"):
            for i, situation in enumerate(SITUATIONS[lang]):
                for j, instruction in enumerate(INSTRUCTIONS[lang]):
                    state = {
                        "personality": p[lang],
                        "situation": situation,
                        "instruction": instruction,
                    }
                    r = client.system_one(state=state, questions=QUESTIONS[lang])
                    a = r.answers
                    print(json.dumps({
                        "personality": p_name,
                        "lang": lang,
                        "situation": i,
                        "instruction": j,
                        "noul": a["instruction_applies"].noul,
                        "choice": a["next_action"].choice,
                        "confidence": a["next_action"].confidence,
                        "probabilities": a["next_action"].probabilities,
                    }, ensure_ascii=False))
```

## 記録すること

- [ ] SDK のバージョン（`pip show typesafe-sdk`）
- [ ] 出力全体（36 行の JSON）
- [ ] スクリプトを直した場合は、その差分
- [ ] 直感と合わない選択があれば、どれか

## 判定基準

同じ性格・状況・指示の組（18組）で、`ja` と `en` を比べる。「期待どおり」は、実行前に埋めた表と `choice` が一致すること。

| 結果 | 設計への影響（D-04） |
|---|---|
| `ja` の期待どおりが `en` の期待どおりより 2 組以上少なくない、かつ `ja` と `en` の `choice` 一致が 15 組以上 | 日本語のまま渡す |
| `ja` の期待どおりが `en` より 3 組以上少ない | 英訳して渡す。英訳の方法（人が書く／機械翻訳）を別に決める |
| 指示ありの組（状況 0）の Noul が、`ja` で `en` より 0.2 以上低い | 少なくとも指示文は英訳する |
| 上のどれにも当てはまらない（差が小さいが一致が少ない など） | 判定保留。性格文を増やして再実験するかを相談する |

この数値は目安。結果を見て変える場合は、理由を「判定」に書く。

## 結果（実施者が記入）

### 環境

- SDK のバージョン:
- 実施日時:

### 出力

```
```

### 判定

### 気づいたこと
