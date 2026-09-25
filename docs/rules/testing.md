# 自動テストの方針

道具は Vitest（テスト）と fast-check（プロパティテスト）。設定は `vitest.config.ts`。

## 1. 原則

- テストは速く、決定的で、ネットワークと実機を使わない（既定の `pnpm test` で）。
- 外部 I/O（Jev API、シリアル、HTTP、時計）は関数の型の後ろに置き、Fake に差し替えてテストする。
- 振る舞いを足す・変えるときは、先に失敗するテストを書く（テスト先行）。
- 「実機で動いた」は自動テストの代わりにならない。実機の確認は `experimentals/` に記録する。

## 2. テストの種類

| 種類 | 場所・名前 | 対象 | CI で実行 |
|---|---|---|---|
| 単体 | `tests/unit/*.test.ts` | 純粋関数（`domain`、`arbiter`、`skills`、変換関数） | する |
| プロパティ | `tests/unit/*.test.ts`（fast-check） | 不変条件（例: [05-arbitration.md](../design/05-arbitration.md) の P1〜P5） | する |
| 契約 | `tests/contract/*.test.ts` | Fake と本物の実装が同じ型を満たし、同じ入力に同じように振る舞うか。記録済み応答（fixture）を本物のパーサーに通す | する |
| 結合 | `tests/integration/*.test.ts` | Fake 一式でループを偽の時計で回す | する |
| Jev 実 API | `*.jev.test.ts` | 本物の Jev を呼ぶ | しない（API キーがある手元だけ） |
| 実機 | `*.hardware.test.ts` | 本物の Bittle を動かす | しない（人が実行し `experimentals/` に記録） |

Jev 実 API と実機のテストは、ファイル名で分けた別プロジェクトにしてあり、既定では実行されない。

```sh
pnpm test            # 既定（CI と同じ範囲）
pnpm test:jev        # 本物の Jev（API キーが必要）
pnpm test:hardware   # 本物の Bittle（人が実行）
```

## 3. Fake と fixture の決まり

- Fake は `src/testing/` に置き、本物と同じ型を満たす（`satisfies Transport` などで型検査する）。
- Fake の振る舞いは、実験で記録した実際の応答に合わせる。根拠の実験 ID をコメントに書く。
- 記録済み応答は `tests/fixtures/<相手>/` に置く。API キー・IP アドレスなど、[public-repo.md](public-repo.md) の表に当たるものは含めない。
- 時計・乱数は引数で注入する。テストの中で実時間を待たない（`vi.useFakeTimers()` か、注入した偽の時計を使う）。
- fast-check は CI では seed を固定しない代わりに、失敗したときの seed と反例を出力する。見つかった反例は `examples` で固定し、回帰テストにする。

## 4. カバレッジ

- `scripts/check.sh` が `vitest run --coverage` で測る。1ファイルだけ実行するときは測らない。
- 全体の下限は分岐・行・関数とも 80%（`vitest.config.ts` の `thresholds`）。
- `arbiter` と `domain` は分岐カバレッジ 100% を目標にする。
- 本物の I/O 実装の I/O 部分は、`/* v8 ignore */` で除外せず、契約テストでカバーする。どうしても無理な行だけ、理由をコメントして除外する。

## 5. 書き方

- テスト名は「条件 → 期待する結果」が分かる文にする（例: `"古い判断は捨てて反射層に落ちる"`）。日本語でよい。
- 1テスト1つの振る舞い。準備・実行・確認を空行で分ける。
- 失敗したテストを `skip`・`todo`・削除して CI を通さない。仕様が変わった場合だけ、理由を PR に書いて直す。
