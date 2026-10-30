# Benchmark measurements

[`../measurements.sqlite`](../measurements.sqlite) records how fast each
benchmark workload runs on each engine over time. SQLite readers need no
extensions, and the Python tool uses only the standard library.

## Contents

One table, one row per measured workload and engine:

```sql
CREATE TABLE measurements (
  workload TEXT NOT NULL,   -- benchmark id from benchmarks/runtime/manifest.json
  engine   TEXT NOT NULL,   -- native, node-tsc, rust-hand, bun, deno, ...
  history  TEXT NOT NULL,   -- JSON array, oldest run first
  PRIMARY KEY (workload, engine)) WITHOUT ROWID;
```

Each `history` entry describes one run:

```json
{"at": "2026-10-30T22:11:30.207Z", "ms": 0.394, "ops": 5100000.0, "samples": 93}
```

- `at`: when the run started. Entries from the same run share it, so native and
  Node.js results are compared by matching `at`.
- `ms`: median duration of one timed sample, in milliseconds.
- `ops`: mean operations per second over the samples.
- `samples`: how many timed samples the run took (processes × samples per process).

The October 30, 2026 store was built from the previous dataset's 101 indexed
runs and the local result files of later runs. Individual sample durations,
allocation profiles, proof hashes and the legacy `records.sqlite` archive were
deliberately not kept (owner direction).

Compare runs of the same workload; the history does not record the compiler
commit or machine load, so small differences between runs are not significant.
Rates describe each complete workload, not an isolated operation.

## Use

```sh
cd benchmarks/runtime
npm run store:ingest    # add runs from benchmarks/runtime/results (skips runs already stored)
npm run checkpoint      # latest native speed relative to Node.js after tsc
```

Or query directly:

```sh
sqlite3 benchmarks/measurements.sqlite \
  "SELECT workload, json_extract(history, '$[#-1].ops') FROM measurements WHERE engine = 'native'"
```
