# E17: Node で動かした flybrain.js の再現性と計算時間

- 状態: 反映済み（未依頼 → 依頼中 → 実施済み → 反映済み）
- 依頼日: なし（人に依頼しない実験。[07-roadmap.md](../docs/design/07-roadmap.md) の E17 の注記）
- 実施日: 2026-10-17
- 実施者: エージェント（Claude Code のセッション）

## 目的

`mk-jev-fly-brain` の `mk/flybrain.js`（F-F11〜F-F13）を Node で動かし、次の2つを確かめる。

1. 同じ種で2回動かすと、出力が一致するか。D-06 の案 B（Rust へ移植）で、`flybrain.js` を比べる相手にして出力を1対1で比べられるかの前提になる（[04](../docs/design/04-reflex-layer.md) 5節）
2. 100ms 分のシミュレーションの計算にかかる時間。D-06 の案 A（Node の子プロセス）が計算時間の上で成り立つかの目安と、D-18 の見積もりの材料になる（04 5節・6節）

## 必要なもの

- 機材: なし（実機を使わない）
- ソフトウェア: Node（下の記録では v22.22.0）、git、coreutils（`sha256sum`、`od`）
- 人数と役目: 1人（エージェント）
- 所要時間の目安: 10分

## 安全上の注意

- 実機を動かさない。
- 外部のコードを実行する。手順1で、実行する前に確かめる。

## 手順

作業用のディレクトリ `<WORK>` で実行する。外部のリポジトリと、自分で書いたスクリプトは、別のディレクトリに置く。

計時の道具（手順4の `timing.mjs`）は JavaScript で書く。理由は [07-roadmap.md](../docs/design/07-roadmap.md) の E17 の注記。

0. 外部のリポジトリを取得する。

   ```sh
   cd <WORK>
   git clone https://github.com/lavallee/mk-jev-fly-brain.git
   git -C mk-jev-fly-brain checkout 8217937e6e725b0ce3160c6e1ab0183db8b8aa30
   R=<WORK>/mk-jev-fly-brain
   ```

1. 実行する前に、外部のコードを確かめる。
   - `mk/flybrain.js`（約13KB）の全文を読み、ファイル・ネットワーク・プロセス・タイマーを使わないことを確かめる。
   - `mk/fly_circuit.js`（約12MB）は全文を表示しない。次のコマンドで、コメント1行と、`FLY_CIRCUIT` のオブジェクトのリテラル1つだけで、関数の呼び出し・定義に当たる形（名前の後に `(`、`=>`、バッククォート）を含まないことを確かめる。

   ```sh
   grep -c "" $R/mk/fly_circuit.js                     # 2
   sed -n 1p $R/mk/fly_circuit.js                      # // で始まるコメント
   sed -n 2p $R/mk/fly_circuit.js | head -c 28; echo   # export const FLY_CIRCUIT = {
   tail -c 3 $R/mk/fly_circuit.js | od -An -c          # }   ;  \n
   grep -o -E '[A-Za-z_$][A-Za-z0-9_$]*[[:space:]]*\(|=>|`' $R/mk/fly_circuit.js | wc -l   # 0
   ```

2. `scripts/js/probe_channels.mjs` の import 先 `../../app/static/mk/flybrain.js` はリポジトリに無い（F-F12）。import 先だけを `$R/mk/flybrain.js` に直したコピーを、別のディレクトリに作る。

   ```sh
   mkdir -p <WORK>/e17 && cd <WORK>/e17
   sed "s#from '../../app/static/mk/flybrain.js'#from '$R/mk/flybrain.js'#" \
     $R/scripts/js/probe_channels.mjs > probe_channels.mjs
   diff $R/scripts/js/probe_channels.mjs probe_channels.mjs   # 4行目だけが違う
   ```

3. `probe_channels.mjs` を2回実行し、出力を比べる。種はスクリプトの中で 7 に固定されている（コミット `8217937` のコードを読んだ）。

   ```sh
   node probe_channels.mjs $R/mk/fly_circuit.js > p1.txt
   node probe_channels.mjs $R/mk/fly_circuit.js > p2.txt
   cmp p1.txt p2.txt && echo IDENTICAL
   sha256sum p1.txt p2.txt
   ```

4. 計時のスクリプト `timing.mjs` を作り、実行する。`FlyBrain.step(100)` は 100ms 分を進めて、群ごとの発火率を返す（コミット `8217937` のコードを読んだ）。これを100回呼び、1回ごとの時間を測る。同じ設定を種 7 で2回、種 8 で1回動かし、`step` が返した値を JSON の文字列にして、100回分を比べる。感覚の入力が無い場合と、`loom_L`・`loom_R` に 100 Hz を入れた場合の2通りで行う。

   ```js
   // E17: time FlyBrain.step(100) and check that the same seed gives the same output.
   //   node timing.mjs <flybrain.js> <fly_circuit.js> [windows]
   import { pathToFileURL } from 'node:url';
   import { performance } from 'node:perf_hooks';
   const [lib, file, nArg] = process.argv.slice(2);
   const N = Number(nArg ?? 100);
   const { FlyBrain, SENSORS } = await import(pathToFileURL(lib));
   const { FLY_CIRCUIT: C } = await import(pathToFileURL(file));
   const zero = Object.fromEntries(SENSORS.map(k => [k, 0]));
   function run(seed, drive) {
       const b = new FlyBrain(C, { seed });
       b.setDrive({ ...zero, ...drive });
       const ms = [], out = [];
       for (let i = 0; i < N; i++) {
           const t0 = performance.now();
           const r = b.step(100);
           ms.push(performance.now() - t0);
           out.push(JSON.stringify(r));
       }
       return { ms, out };
   }
   const q = (a, p) => { const s = [...a].sort((x, y) => x - y); return s[Math.min(s.length - 1, Math.floor(p * s.length))]; };
   const fmt = a => `median ${q(a, 0.5).toFixed(1)} ms, p90 ${q(a, 0.9).toFixed(1)} ms, max ${Math.max(...a).toFixed(1)} ms`;
   console.log(`node ${process.version}, neurons ${C.neurons.length}, synapses ${C.pre.length}, windows ${N} x 100 ms`);
   for (const [name, drive] of [['no drive', {}], ['loom_L+loom_R 100 Hz', { loom_L: 100, loom_R: 100 }]]) {
       const a = run(7, drive), b = run(7, drive), c = run(8, drive);
       const same = a.out.every((x, i) => x === b.out[i]);
       const diff = c.out.some((x, i) => x !== a.out[i]);
       console.log(`${name}: seed 7 twice identical=${same}; seed 8 differs=${diff}`);
       console.log(`  run1 ${fmt(a.ms)} | run2 ${fmt(b.ms)} (first window ${a.ms[0].toFixed(1)} ms)`);
   }
   ```

   ```sh
   time node timing.mjs $R/mk/flybrain.js $R/mk/fly_circuit.js 100
   ```

## 記録すること

- [x] 環境（OS、`lscpu` の `CPU(s)` と `Model name` の行、Node のバージョン、外部のリポジトリのコミット）
- [x] 手順1の出力
- [x] 手順3の出力
- [x] 手順4の出力（`time` を含む）

## 判定基準

判定基準は、実行した後に書いた（人に依頼しない実験で、先に手順書を作らなかったため）。計算時間は、ふだんの値として中央値を、100ms 周期に間に合わない窓が出るかとして最大を見る。

再現性（上の行から順に当てはめ、最初に当たった行で判定する）:

| 観察 | 結果 | 設計への影響 |
|---|---|---|
| 種 7 どうしの比べ方（手順3の `cmp`、手順4の2つの入力の `seed 7 twice identical`）のどれか1つでも一致しない | 再現しない | D-06 の案 B で `flybrain.js` を相手に1対1で比べる方法は使えない。分布で比べる形を考える（04 5節） |
| 種 7 どうしはすべて一致し、手順4の2つの入力のどちらかで `seed 8 differs=false` | 種が効いていない疑い | 再現性の判定は保留。原因を調べる |
| 種 7 どうしはすべて一致し、手順4の2つの入力の両方で `seed 8 differs=true` | 同じ機械・同じ Node・同じ種なら再現する | 04 5節の「種を合わせて出力を1対1で比べる」の前提の1つが満たされる。`flybrain.js` を比べる相手の候補に残す。別の機械・別の Node での一致と、Rust と JS で浮動小数の計算の順を揃えられるかは、確かめていない |

計算時間（手順4の4つの計測 run1・run2 × 2つの入力を合わせて見る。上の行から順に当てはめる）:

| 観察 | 結果 | 設計への影響 |
|---|---|---|
| どれかの中央値が 100ms 以上 | この機械の Node でも、ふだんから実時間に間に合わない | D-06 の案 A は、計算時間の上で成り立たない見込み。案 B でも、回路を小さくするか速い実装が要る見込み |
| 中央値はすべて 100ms 未満で、どれかの最大が 100ms 以上 | ふだんは間に合うが、間に合わない窓がある | D-06 の案 A を、計算時間の上では除外しない。どの案でも、間に合わなかった窓の扱い（読み飛ばす、前の値を使う）が論点になる |
| 中央値も最大もすべて 100ms 未満 | この機械では、どの窓も実時間に間に合った | D-06 の案 A を、計算時間の上では除外しない |

どの行でも、D-18（機体に積んだボードで動かすか）については、別の機械の値なので見積もりの材料にとどまる。ボードの上の時間は、ボードの上で測る（人への依頼。D-18 の論点 (c)）。

| 観察 | 結果 | 設計への影響 |
|---|---|---|
| （全体） | 手順どおりに進まなかった | やり直す。それまで、D-06・D-18 のうち再現性と計算時間に関わる判断は保留のまま |

## 結果

同じ日に2回実施した。1回目は、記録した環境と `time` の出力が足りなかったので、手順を直して2回目を行った。判定は2回目の記録で行い、1回目は参考として残す。

### 環境（2回目）

- 機械: クラウドの作業用コンテナ。同じ時刻に他の負荷が無かったかは確かめていない
- `flybrain.js` はスレッドを使わないので、計算は1コアで行われる（コミット `8217937` のコードを読んだ範囲。コアの使用率は測っていない）

```
$ uname -sr
Linux 6.18.44-fc-v139
$ lscpu | grep -E "^CPU\(s\)|Model name"
CPU(s):                                  4
Model name:                              Intel(R) Xeon(R) Processor @ 2.80GHz
$ node --version
v22.22.0
$ git -C $R rev-parse HEAD
8217937e6e725b0ce3160c6e1ab0183db8b8aa30
```

### 出力・観察（2回目）

手順1（`mk/flybrain.js` の全文も読んだ。ファイル・ネットワーク・プロセス・タイマーは使っていない）:

最後のコマンドは、手順1と同じ正規表現を、二重引用符とエスケープの形で実行した（下は実行したとおりの字面）。

```
$ grep -c "" $R/mk/fly_circuit.js
2
$ sed -n 1p $R/mk/fly_circuit.js
// Generated by scripts/build_malecns_fighter.py from maleCNS v1.0. Do not edit.
$ sed -n 2p $R/mk/fly_circuit.js | head -c 28; echo
export const FLY_CIRCUIT = {
$ tail -c 3 $R/mk/fly_circuit.js | od -An -c
   }   ;  \n
$ grep -o -E "[A-Za-z_\$][A-Za-z0-9_\$]*[[:space:]]*\(|=>|\`" $R/mk/fly_circuit.js | wc -l
0
```

手順2: `diff` は4行目（import の行）だけだった。

手順3（`p1.txt` の1行目は、`fly_circuit.js` の絶対パスと `rate=100 Hz` なので、貼らない）:

```
IDENTICAL
d49ed7934b6c51bd58748082ab959b5ec2e2e9b3c4d8e7485bd3bfedd8a8e38c  p1.txt
d49ed7934b6c51bd58748082ab959b5ec2e2e9b3c4d8e7485bd3bfedd8a8e38c  p2.txt
channel              fwd     back     jump    punch     kick     wing
loom                 0.0     -8.5     80.9     -1.1      8.3     16.2
object              69.0     -5.3     -0.0      0.4     -1.7     -0.1
target               0.0     -5.2      0.0     -5.7     -3.7      5.6
recede               0.0     -1.6     -0.4      0.8      0.4      8.0
p1                   0.0     -2.6      0.2     -3.1     -0.8     10.0
taste                0.0     -0.6     -0.0     -0.4     -0.4      0.2
body_touch           0.0     -8.5      0.6    -17.0      4.5     13.6
```

手順4:

```
node v22.22.0, neurons 12000, synapses 941464, windows 100 x 100 ms
no drive: seed 7 twice identical=true; seed 8 differs=true
  run1 median 52.6 ms, p90 63.9 ms, max 100.0 ms | run2 median 48.3 ms, p90 53.9 ms, max 82.2 ms (first window 79.4 ms)
loom_L+loom_R 100 Hz: seed 7 twice identical=true; seed 8 differs=true
  run1 median 55.6 ms, p90 65.8 ms, max 89.9 ms | run2 median 55.9 ms, p90 73.2 ms, max 101.7 ms (first window 79.2 ms)

real	0m33.729s
user	0m33.469s
sys	0m0.140s
```

### 1回目の出力（参考）

手順3は、`cmp` の結果が `IDENTICAL` で、`p1.txt` の2行目以降は2回目と同じだった。手順4（`time` を付けずに実行した）:

```
node v22.22.0, neurons 12000, synapses 941464, windows 100 x 100 ms
no drive: seed 7 twice identical=true; seed 8 differs=true
  run1 median 53.4 ms, p90 74.4 ms, max 122.9 ms | run2 median 48.5 ms, p90 61.5 ms, max 106.1 ms (first window 122.9 ms)
loom_L+loom_R 100 Hz: seed 7 twice identical=true; seed 8 differs=true
  run1 median 55.0 ms, p90 70.4 ms, max 111.1 ms | run2 median 57.5 ms, p90 78.9 ms, max 115.2 ms (first window 76.4 ms)
```

### 判定

- 再現性: 表の3行目。種 7 どうしはすべて一致し、種 8 は両方の入力で違った。同じ機械・同じ Node・同じ種なら再現する。
- 計算時間: 表の2行目。中央値は 48.3〜55.9 ms で、すべて 100ms 未満。最大は 82.2〜101.7 ms で、100ms 以上の窓があった（`loom` の run2 で 101.7 ms。`no drive` の run1 の 100.0 ms は小数1桁に丸めた表示なので、100ms 以上かは分からない）。1回目も同じ行に当たる（中央値 48.5〜57.5 ms、最大 106.1〜122.9 ms）。

### 気づいたこと

- 最大が 100ms を超えた理由（JIT のウォームアップ、ガベージコレクション、コンテナの他の負荷など）は調べていない。2回目では、最初の窓が最大になった計測は無かった（1回目では `no drive` の run1 の1つ）。
- 回路の読み込み（12MB の JS の import と、`FlyBrain` の構築）の時間は測っていない。
- 別の機械（Mac、D-18 の材料の Raspberry Pi）では測っていない。
