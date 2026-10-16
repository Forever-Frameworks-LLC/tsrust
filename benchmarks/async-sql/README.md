# Async SQL prepared batches

Use Node v24.11.1, `npm ci --prefix benchmarks/async-sql`, and the verified compiler in `TSRUST`.

```
TSRUST_SQL_DRIVER=sqlite node benchmarks/async-sql/run.mjs --verify
```

For PostgreSQL/MySQL also set `TSRUST_SQL_URL` to a disposable database. The workload creates and drops `tsrust_s18_bench`; never target a production database. Driver SQL is explicit: PostgreSQL uses `$1` placeholders; MySQL uses `CAST(... AS SIGNED)`.

Each batch inserts 128 rows in a transaction and reads their sum. Independent oracle: `128 * 127 = 16256`. Native SQLx and Node-after-tsc pg/mysql2/node:sqlite execute matching operations. This measures the complete database-client path, including async SQLite's worker dispatch versus Node's synchronous SQLite calls. It is not an isolated scheduler comparison.

Verification uses one untimed sample per engine. Omit `--verify` for three process pairs, eight warmups and 31 samples per process, alternating engine order. Preserve original records in SQLite and remove verified loose copies. Driver versions/database images belong with run evidence. No measurements have been accepted yet.

`--cursor --verify` checks SQLite row streaming without collecting a result array.
128 ordered rows contain 2 * id; the independent checksum is 128 * 127 = 16256.
Seeding happens before the timed region. Native SQLx streams through its worker;
Node uses DatabaseSync.iterate(), so this compares the complete client paths.
The same 3-process/8-warmup/31-sample protocol is ready; verification is one untimed
batch. PostgreSQL/MySQL cursor benchmarks are explicitly unsupported until matched
Node streaming adapters exist; their correctness is covered by the driver runner.

`--jobs --verify` checks 128 application-owned SQLite jobs against a matched
Node-after-tsc workload, with one committed SQL effect per acknowledged job. It
uses the example's actual helper and pins its hash as well as both workload
sources. Timing mode retains 3 processes, 8 warmups and 31 samples. Setup and
per-batch table clearing are outside timing. This in-memory workload measures
SQL coordination, not disk durability, concurrent throughput or crash recovery;
the file-backed example acceptance covers persistence, retries and stale leases.
PostgreSQL/MySQL job dialects are unsupported. No job measurements are claimed yet.
