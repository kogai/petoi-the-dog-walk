# 決定事項・未決定事項

決まったら「状態」を「決定」に変え、「決定内容・理由」と日付を書く。決定を覆すときは行を消さず、新しい ID を足す。
論点の詳細は、この表だけに書く（他の文書からは ID で参照する）。

| ID | 論点 | 状態 | 決め方 | 決定内容・理由 |
|---|---|---|---|---|
| D-01 | 安全上の例外（転倒しそうなとき等）を優先順位の外に置くか | 未決定 | E01 で転倒を検知できるか次第。置く場合は `arbitrate` の SAFETY 規則として最初に判定し、テストで固定する | |
| D-02 | 反射で反応させる対象（接近、持ち上げ、転倒、音など） | 未決定 | E01・E02 の結果で選べる範囲が決まる | |
| D-03 | 調停規則3「上の2つが無い場合」の定義 | 未決定 | [05](05-arbitration.md) 4節の候補から、E08 で決める | |
| D-04 | 日本語の性格文を英訳して Jev に渡すか | 未決定 | E05 で決める | |
| D-05 | 性格・スキルのファイル形式 | 仮決定 (2026-09-25) | フェーズ 1a で見直す | `personality.md`（自由文）＋ `skills.yaml`（名前・説明・トークン列）。理由: 性格は非エンジニアが書く自由文なので Markdown、スキルはコードがトークン列として読むので構造化形式 |
| D-06 | ハエ脳の実行方法（Node の worker で動かす / TypeScript へ移植 / ブラウザ） | 未決定 | [04](04-reflex-layer.md) 5節。フェーズ3の前に決める | |
| D-07 | jump_away / punch / kick / block の Bittle 側の対応 | 未決定 | フェーズ3で決める | |
| D-08 | 公式 Python API（PetoiRobot）を依存に入れるか | 決定 (2026-09-25) | — | 入れない。理由: 本体を TypeScript にした（D-11）。シリアルとトークンの送信は `serialport` で足りる |
| D-09 | 言語とツール | 覆された（D-11） | — | （当初）Python 3.11 以上、uv、ruff、mypy (strict)、pytest。理由: Jev の SDK が Python（F-J7）、Petoi の公式 API も Python（F-B6）。型検査とテストで、実機なしに振る舞いを固められる |
| D-10 | Jev SDK に非同期 API があるか。無い場合の包み方 | 解消 (2026-09-25) | — | SDK を使わず、HTTP API を `fetch`（非同期）で直接呼ぶので論点が無くなった（D-11、F-J9） |
| D-11 | 言語とツール（D-09 の置き換え） | 決定 (2026-09-25) | — | TypeScript（Node.js 22.18 以上。型を取り除いて直接実行）、pnpm、tsc (strict)、ESLint（typescript-eslint の strictTypeChecked と eslint-plugin-functional）、Biome（整形）、dependency-cruiser、Vitest、fast-check。純粋な部分は関数型で書く。理由: 人の希望（静的型検査か関数型の言語で、Python は抑える）。Jev は HTTP API で呼べる（F-J9）ので SDK が要らない。ハエ脳の公式実装が JavaScript（F-F1）で、同じ言語で動かせる。Python は使わない |
