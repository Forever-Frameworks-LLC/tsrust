# Rust and Tsrust, one CPU-bound terminal program built both ways

The same command-line program written twice: `rust/` as ordinary Rust and
`tsrust/` as `.tsr` modules. Ten phases, each timed inside the program, each
printing a checksum. The two versions must print identical checksums; the
measurement script fails otherwise.

| Phase | What it exercises |
|---|---|
| sieve | 20M-element boolean array, indexed reads and writes, int arithmetic |
| nbody | array of float records, field reads and writes, `Math.sqrt` |
| matrix | 300x300 nested `number[][]` multiply |
| fib | `fib(32)`, about 7M recursive calls |
| text | 3M generated words joined into one string |
| words | `split`, a `Map<string, int>` word count, sort by count then word |
| format | template strings, `toUpperCase`, `replaceAll`, `indexOf`, `slice` |
| orders | 1M records with strings and arrays, `filter`/`map`/`reduce`, sort, `Map` |
| trees | binary trees of recursive records (`Tree \| null` fields), depth 4 to 18 |
| sets | 3M random `Set<int>` lookups and inserts |

Both use checked integer overflow (Tsrust's release profile). The Rust version
is written the way a Rust programmer would write it: `vec![false; n]`,
`HashMap::entry`, borrowed `&str` keys, sorting references.

## Running

```bash
(cd rust && cargo build --release)
(cd tsrust && tsrust build --release)
python3 measure.py rust/target/release/lang-compare tsrust/target/lang-compare 5
```

`SCALE=0.1` shrinks the element and step counts for a quick check, and
`PHASE=words` runs one phase alone (for `sample` or other profilers).

`tsrust build` without `--release` compiles at opt-level 0 (UI programs at 1);
only `--release` is comparable with `cargo build --release`. At opt-level 0 the
phases ran 4 to 28 times slower than Rust.

## Results (2026-11-18, MacBook Air M3, 8 GB, median of 5)

Runtime faults unwind by default (since 2026-11-18); these figures include it.

| Phase | Rust ms | Tsrust ms | Ratio |
|---|---:|---:|---:|
| sieve | 45.6 | 80.3 | 1.76 |
| nbody | 34.0 | 43.0 | 1.26 |
| matrix | 20.3 | 25.4 | 1.25 |
| fib | 7.1 | 7.1 | 1.00 |
| text | 114.2 | 58.6 | 0.51 |
| words | 62.7 | 116.1 | 1.85 |
| format | 136.8 | 159.9 | 1.17 |
| orders | 156.7 | 200.1 | 1.28 |
| trees | 1231.8 | 1671.9 | 1.36 |
| sets | 131.7 | 247.2 | 1.88 |
| whole process | 1.94 s | 2.67 s | 1.38 |

The first measurement (2026-11-17) had words 3.97, sets 2.64, orders 1.84 and
the whole process 1.52; the changes since are described below. History is in
`benchmarks/measurements.sqlite` under `lang-compare/*`.

Peak memory: Rust 256 MiB, Tsrust 520 MiB. Binary: 0.5 MiB and 0.8 MiB.

## Where the time goes

Profiled with `sample` per phase (`PHASE=...`, larger `SCALE`), plus small Rust
reproductions of the generated code's shape.

- **Every fallible call checks a `Result` (fib, 1.4x).** Integer overflow is a
  catchable `RangeError`, so almost every function returns
  `Result<T, error record>` and each call site matches it. A Rust reproduction
  shows the same 1.3 to 1.4x against panicking overflow. Boxing the error
  record (to return in registers) did not help, so the check itself is the
  cost. Removing it needs analysis that proves a call cannot throw.
- **Arrays are a small-vector enum (sieve 1.8x, nbody, matrix).** `int[]`,
  `boolean[]` and record arrays are `Small { [T; 4] } | Heap(Vec<T>)`. Each
  index matches the variant and converts `int` to `usize` with a checked
  conversion; because writes may alias the inline storage, LLVM cannot hoist
  the check out of the loop. A Rust reproduction of this shape costs about
  +19 ms on the sieve (37 to 74 ms with the `push`-built array).
- **No sized array constructor (sieve).** `new Array(n)`,
  `new Array(n).fill(x)` and `Array.from({ length: n }, ...)` are all
  rejected, so a large array is built by `push` in a loop: about +18 ms on the
  sieve against `vec![false; n]`.
- **`split` copies every piece; `Map.set` needs an owned key (words 4.0x).**
  Splitting an owned string allocates one string per piece, and
  `counts.set(clone(word), ...)` allocates again for every word, even when
  the key exists. The program also hashes each word twice (`get` then
  `set`). Allocation, freeing and copying are about half the phase; hashing
  about a fifth.
- **`Map`/`Set` keep insertion order through two tables (sets 2.6x).** A
  lookup hashes the key, probes a hash-to-index table, then reads the slot
  vector. `has` followed by `add` hashes twice, and number keys hash a type
  tag plus the value (int keys are first converted to float so `1` and `1.0`
  match). Hashing is 68% of the sets phase, table work 30%.
- **`filter` copies records and sort works through an index permutation
  (orders 1.8x).** `filter` clones each kept record (strings and arrays
  included); the throwing-comparator sort sorts positions, then permutes the
  records. Sorting is 60% of the phase, string hashing for the customer map
  18%.
- **Recursive records drop through a work list (trees 1.4x).** A
  `Tree | null` field is a tagged union around an optional box, and dropping a
  tree pushes every node through a heap work list (safe for deep trees) where
  Rust drops recursively.
- **Strings can be faster (text 0.5x).** Tsrust strings borrow static text
  without allocating, so pushing vocabulary words copies nothing; the Rust
  version allocates a `String` per word.

## After the first fixes (2026-11-17)

Map and Set now share one open-addressing index that stores each hash beside
its slot index, hash each key once per operation, and hash numbers without a
type tag. `m.set(clone(name), value)` with a string key clones only when the
key is new, and `for (const piece of text.split(separator))` walks the pieces
without building an array (owned text reuses one buffer). Release medians:

| Phase | Before | After | Rust |
|---|---:|---:|---:|
| words | 248.4 ms (4.0x) | 137.8 ms (2.2x) | 62.8 ms |
| sets | 332.6 ms (2.6x) | 263.4 ms (2.0x) | 133.7 ms |
| whole process | 2.94 s | 2.81 s | 1.94 s |
| peak memory | 520 MiB | 379 MiB | 256 MiB |

Other phases are unchanged within noise. Words still hashes each word twice
(`get` then `set`); sets still hashes twice for `has` then `add`.

## Second round (2026-11-17)

Shared changes first, then the orders phase:

- Map/Set remember their last lookup (hash plus slot, or a miss for scalar
  keys), so `get` then `set` and `has` then `add` hash once.
- Dead-local moves now run in functions that can throw (nearly all, because of
  checked arithmetic), see through `??` values, and allow earlier reads and
  pushes as long as the moving call is the local's last mention. `filter` joined
  the moving calls: kept elements move out of a dead source instead of being
  cloned.
- `m.set(clone(record.field), v)` also clones only new keys.
- A sort with a comparator that can throw applies its result by moving each
  element once into place (owned arrays) or reading a snapshot (number
  arrays), instead of random swaps (about 17% of the sort in a reproduction).

| Phase | First run | Now | Rust |
|---|---:|---:|---:|
| words | 248.4 ms (4.0x) | 116.4 ms (1.9x) | 62.6 ms |
| sets | 332.6 ms (2.6x) | 244.5 ms (1.9x) | 130.4 ms |
| orders | 282.4 ms (1.8x) | 212.8 ms (1.4x) | 154.8 ms |
| whole process | 2.94 s | 2.68 s | 1.94 s |
| peak memory | 520 MiB | 368 MiB | 256 MiB |

Most of what remains in orders is the comparator itself: it can throw (the
`left.id - right.id` overflow check), so each comparison returns a `Result`.

## Per-call Result cost (investigated 2026-11-17)

Reproductions of `fib(32)` in Rust, best of 7 runs:

| Shape | Time |
|---|---:|
| Plain Rust, overflow panics | 6.6 ms |
| Tsrust's generated function, copied verbatim | 9.3 ms |
| The same with `return Ok`/`return Err` instead of completion blocks | 9.3 ms |
| Hand-written `Result<i64, error record>` | 7.7 to 9.3 ms (varies with code layout) |
| Error record built in a `#[cold]` function | 8.4 ms |
| `Result<i64, marker>` with the record in a thread-local | 10.4 ms |

LLVM already removes the completion plumbing, and the error record's size or
placement does not matter. The cost is returning and checking a `Result` on
every call. Range analysis would not remove it here: `fib`'s addition,
`check`'s `1 + check(left) + check(right)` and the orders comparator's
`left.id - right.id` can all overflow. Matching plain Rust would need thrown
errors to unwind (panic with the error as payload, caught at `try`/`catch`
and at the program boundary) instead of travelling in `Result`. That changes
the exception model: unwinding has no cost until something throws, but it
needs a silent panic hook, `catch_unwind` at every catch site and every
thread or async boundary, and a target that supports unwinding.

## Possible fixes, by expected gain

1. Done: see the two rounds above.
2. A sized array constructor (`new Array<T>(n).fill(x)` or `Array.from`
   with a length), and a hoisted slice for index loops over one array
   (sieve, matrix, nbody).
4. Done (2026-11-18): runtime faults unwind instead of returning `Result`
   (calls, comparators, recursive builders); see "Fault unwinding" below.

## Fault unwinding (2026-11-18)

Runtime faults now unwind instead of travelling in every caller's `Result`
(`docs/plans/fault-unwinding.md`); explicit errors stay values. Release
medians of 5, value mode (`TSRUST_FAULT_MODE=value`) against the new default:

| Phase | Value mode | Unwind (default) | Rust |
|---|---:|---:|---:|
| fib | 9.8 ms (1.42x) | 7.1 ms (1.04x) | 6.9 ms |
| orders | 209.5 ms (1.39x) | 187.9 ms (1.26x) | 150.4 ms |
| whole process | 2.58 s | 2.53 s | 1.87 s |

The other phases are unchanged within noise. The sorted comparator and the
recursive calls no longer return a `Result`; what remains in orders is the
copy and permutation work described above.

