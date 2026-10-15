# Async SQL prepared batches

Use Node v24.11.1, `npm ci --prefix benchmarks/async-sql`, and the verified compiler in `TSRUST`.

```
TSRUST_SQL_DRIVER=sqlite node benchmarks/async-sql/run.mjs --verify
```

For PostgreSQL/MySQL also set `TSRUST_SQL_URL` to a disposable database. The workload creates and drops `tsrust_s18_bench`; never target a production database. Driver SQL is explicit: PostgreSQL uses `$1` placeholders; MySQL uses `CAST(... AS SIGNED)`.

Each batch inserts 128 rows in a transaction and reads their sum. Independent oracle: `128 * 127 = 16256`. Native SQLx and Node-after-tsc pg/mysql2/node:sqlite execute matching operations. This measures the complete database-client path, including async SQLite's worker dispatch versus Node's synchronous SQLite calls. It is not an isolated scheduler comparison.

Verification uses one untimed sample per engine. Omit `--verify` for three process pairs, eight warmups and 31 samples per process, alternating engine order. Preserve original records in SQLite and remove verified loose copies. Driver versions/database images belong with run evidence. No measurements have been accepted yet.
