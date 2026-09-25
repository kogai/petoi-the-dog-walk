# コードの書き方

言語は TypeScript（[D-11](../design/decisions.md)）。Python は使わない。
方針は「関数型のコア、命令型の殻」。判断のロジックは純粋関数にして、I/O は外側の薄い層に閉じ込める。

## 1. 関数型のコア（`src/domain`、`src/arbiter`、`src/skills` など）

- 純粋関数だけで書く。同じ入力には必ず同じ出力を返し、I/O・時計・乱数に触れない。
- データは変更しない。型は `readonly`・`ReadonlyArray` にし、更新はスプレッド構文などで新しい値を作る。
- `let`・ループ文・クラス・`this` を使わない。`map`・`filter`・`reduce`・再帰で書く。
- 失敗は例外ではなく値で返す（`Result<T, E>` 型）。`throw`・`try` を使わない。
- 選択肢はユニオン型で表し、`switch` で網羅する（漏れは lint が検出する）。
- ESLint（eslint-plugin-functional）と dependency-cruiser で機械的に検査する（[static-analysis.md](static-analysis.md)）。

## 2. 命令型の殻（`src/**/shell/`、`src/main.ts`）

- I/O（Jev の HTTP、シリアル、ファイル、時計）はここだけで行う。
- 外から来る値は `unknown` として受け、スキーマ検査してから内側の型にする。
- ライブラリが投げる例外は、殻の中で捕まえて `Result` に変換してからコアに渡す。
- 殻は薄く保つ。条件分岐が増えてきたら、判断の部分をコアの純粋関数に切り出す。

## 3. 型

- `interface` ではなく `type` を使う。
- 取り違えやすい値はブランド型で区別する（例: 許可リストのトークン `Token`、ミリ秒 `Ms`）。
- `any` を使わない。`as` は境界の直後など、理由を書ける場所に限る。
- 公開する関数には、引数と戻り値の型を書く。

## 4. 実行

- Node.js 22.18 以上で `.ts` を直接実行する（型を取り除くだけ。ビルドしない）。
  そのため `enum`・`namespace` など、型を取り除くだけでは動かない構文は使わない（`erasableSyntaxOnly`）。
- import には拡張子 `.ts` を付ける。
