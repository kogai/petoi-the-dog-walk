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

- TypeSafe の API キー（環境変数 `TYPESAFE_API_KEY` に設定する想定。**名前は未確認**。SDK の案内に従う）
- Python 3.11 以上、`pip install "typesafe-sdk>=0.5.7" --extra-index-url https://pypi.typesafe.ai/`
- 所要時間の目安: 30分

## 手順

1. 下のスクリプトを `e05_jev_ja.py` として保存する（**未実行**。SDK の書き方は公式ドキュメントの例に基づく。
   動かない場合は、エラーをそのまま結果に貼ってもらえれば、こちらで直す）。
2. `python e05_jev_ja.py > e05_result.txt` を実行する。
3. 出力をそのまま「結果」に貼る。API キーが出力に含まれていないことを確認する。

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
CRITERIA = {
    "walk": "Walk forward",
    "back": "Step backward",
    "sit": "Sit down",
    "hello": "Greet",
    "balance": "Stand still",
}

questions = {
    "instruction_applies": Noul(
        instructions="The user's current instruction applies to the current situation."
    ),
    "next_action": Choice(
        instructions="Which action should the robot take next?", criteria=CRITERIA
    ),
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
                    r = client.system_one(state=state, questions=questions)
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

同じ性格・状況・指示の組で、`ja` と `en` を比べる。

| 結果 | 設計への影響（D-04） |
|---|---|
| `choice` がほぼ一致し、`confidence` の差も小さい | 日本語のまま渡す |
| `en` のほうが直感に合う選択が明らかに多い | 英訳して渡す。英訳の方法（人が書く／機械翻訳）を別に決める |
| 指示の Noul が `ja` で低く出る | 少なくとも指示文は英訳する |

## 結果（実施者が記入）

### 環境

- SDK のバージョン:
- 実施日時:

### 出力

```
```

### 判定

### 気づいたこと
