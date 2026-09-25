# 静的検査

すべて `scripts/check.sh` で一度に実行でき、CI（`.github/workflows/ci.yml`）も同じスクリプトを使う。

| 検査 | ツール | 設定 | 失敗したら |
|---|---|---|---|
| 公開してよい内容か | `scripts/check-public.sh` | [public-repo.md](public-repo.md) | 伏せるか、ファイルを外す |
| ロックファイル | `pnpm install --frozen-lockfile` | `pnpm-lock.yaml` | `pnpm install` を実行してコミット |
| 整形 | Biome | `biome.json`（行長 100） | `pnpm format` で直す |
| lint | ESLint | `eslint.config.ts` | 直す。自動修正は `pnpm exec eslint --fix .` |
| 型 | `tsc` | `tsconfig.json` | 直す |
| モジュールの境界 | dependency-cruiser | `.dependency-cruiser.cjs` | 依存の向きを直す。規則を変えるなら設計書も直す |
| テスト | Vitest | [testing.md](testing.md) | 直す |

## 型検査（tsconfig.json）

`strict` に加えて、次を有効にしている。

- `noUncheckedIndexedAccess`: 配列・レコードの添字アクセスは `undefined` を含む。
- `exactOptionalPropertyTypes`: 省略可能なプロパティに `undefined` を明示的に入れられない。
- `erasableSyntaxOnly`: `enum`・`namespace` など、型を取り除くだけでは動かない構文を禁止する（Node が `.ts` を直接実行するため）。
- `noPropertyAccessFromIndexSignature`、`noImplicitReturns`、`noFallthroughCasesInSwitch`、`noUnusedLocals`、`noUnusedParameters`。

## lint（eslint.config.ts）

- typescript-eslint の `strictTypeChecked` と `stylisticTypeChecked`（型情報を使う規則）。
- `switch-exhaustiveness-check`: ユニオン型の `switch` で分岐の漏れを禁止する。
- `explicit-module-boundary-types`: 公開する関数の引数と戻り値に型を書く。
- `consistent-type-definitions: type`: `interface` ではなく `type` を使う。
- `src/` の関数型のコアには eslint-plugin-functional の規則をかける: `let`・配列やオブジェクトの書き換え・ループ文・クラス・`this`・`throw`・`try` を禁止する。
  I/O を扱う殻（`src/**/shell/**`、`src/main.ts`）は除外する。詳しくは [coding.md](coding.md)。
- `no-console`（`tests/`・`scripts/` を除く）。

## モジュールの境界（.dependency-cruiser.cjs）

- 循環 import を禁止する。
- `domain`・`arbiter`・`skills` は Node の組み込みモジュール（`node:fs` など）と I/O を持つモジュールを import できない。
- TypeSafe の API を呼ぶ `src/reasoning/jev.ts` は、`src/reasoning/` と `scripts/experiments/` 以外から import できない（それ以外は `Reasoner` 型だけを見る）。
- `src/` から `tests/` を import できない。

## 例外の書き方

- 規則を無効にするときは、行単位で `// eslint-disable-next-line <規則名> -- <理由>` と書く。ファイル単位・全体での無効化は PR で理由を説明する。
- `@ts-ignore` は禁止。`@ts-expect-error` は 10 文字以上の説明付きでだけ許す。
- `any` は使わない（`strictTypeChecked` が検出する）。外から来る値は `unknown` で受けて、境界でスキーマ検査してから内側の型にする。
- 型アサーション（`as`）は、境界でスキーマ検査した直後などに限り、理由をコメントする。レビューで確認する。

## 秘密情報

- [public-repo.md](public-repo.md) を参照。`.env` は `.gitignore` 済み。
