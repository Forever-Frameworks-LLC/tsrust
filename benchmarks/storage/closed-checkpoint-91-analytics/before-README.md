# Compact benchmark storage

## Current analytics policy

SQLite is the persistent analytics source of truth. Accepted raw run records must retain their original bytes and source/toolchain identities; queryable timing and allocation summaries are derived data. Closed loose records can be removed only after exact import, verification and restoration are proved. Active checks may continue using loose temporary files.

Every five compiler gates, review correctness and matched native versus Node-after-tsc performance to detect bugs and costly implementations before later features depend on them. Preserve other runtimes as context. Compare equivalent operation counts and measurement scopes, keep min/average/max throughput and individual samples, and normalize allocation metrics. A slower measurement is an investigation signal rather than proof of a compiler bug.

`BENCHMARKS.md` is an optional historical export. The future official documentation site can present database-backed charts and tables; building that site is separate work. The archive commands and migration records below describe the existing exact-byte storage implementation and its historical migrations.

`store.py` is an opt-in archive utility using Python 3.11+ standard-library
SQLite, zlib, and SHA256. Report reading optionally accepts this archive; the
benchmark harness still writes loose records. Historical diagnostic artifacts
are compacted as documented in [the archive guide](./HISTORICAL_ARCHIVE.md).
Original compressed bytes are authoritative: JSON is never parsed
or reserialized. Original relative paths, kinds, sizes, and SHA256 digests remain
indexed; identical bytes share one blob. Path bindings and blobs are append-only.

An October 2, 2026 read-only inventory measured **1,106,173,293 logical bytes**:

| Category | Original bytes | In-memory zlib level 6 bytes |
| --- | ---: | ---: |
| Runtime diagnostics | 564,360,523 | 83,954,412 |
| Runtime results | 281,174,249 | 19,766,831 |
| Approximate references | 46,011,969 | 499,273 |
| Runtime dependencies | 211,981,993 | Not measured |
| Runtime workloads | 585,098 | Not measured |

The three measured archive categories shrink from 891,546,741 to 104,220,516
bytes (88.3%). Exact file deduplication saves 64,944,390 original bytes, or another
3,629,954 bytes after compression. These are payload measurements, not measured
SQLite file sizes; SQLite pages and indexes add overhead. Dependencies are a
separate source of disk use. Completed historical/Gate55–86 migrations now store
**21,121 original records in 13,846 blobs, occupying 149,852,160 bytes**. Verified
removal reclaimed **1,894,064,128 allocated bytes (about 1,806 MiB)** before migration
evidence and documentation overhead. Actual plans, indexes and verification are in
[the initial migration](./historical-artifact-migration-2026-10-02.json),
[Gate55](./closed-gate-55-migration/migration.json),
[Gate56](./closed-gate-56-migration/migration.json),
[Gate57](./closed-gate-57-migration/migration.json),
[Gate58](./closed-gate-58-migration/migration.json),
[Gate59](./closed-gate-59-migration/migration.json),
[Gate60](./closed-gate-60-migration/migration.json),
[Gate61](./closed-gate-61-migration/migration.json),
[Gate62](./closed-gate-62-migration/migration.json),
[Gate63](./closed-gate-63-migration/migration.json), and
[Gate64](./closed-gate-64-migration/migration.json), and
[Gate65](./closed-gate-65-migration/migration.json), and
[Gate66](./closed-gate-66-migration/migration.json), and
[Gate67](./closed-gate-67-migration/migration.json), and
[Gate68](./closed-gate-68-migration/migration.json), and
[Gate69](./closed-gate-69-migration/migration.json), and
[Gate70](./closed-gate-70-migration/migration.json), and
[Gate71](./closed-gate-71-migration/migration.json), and
[Gate72](./closed-gate-72-migration/migration.json), and
[Gate73](./closed-gate-73-migration/migration.json), and
[Gate74](./closed-gate-74-migration/migration.json), and
[Gate75](./closed-gate-75-migration/migration.json), and
[Gate76](./closed-gate-76-migration/migration.json), and
[checkpoint77–81](./closed-checkpoint-81-migration/migration.json), and
[checkpoint82–86](./closed-checkpoint-86-migration/migration.json). Active gate evidence and raw
JSON links remain loose until their consumers finish.

## Commands

These examples use disposable fixture directories. The database belongs outside
the source directory. Import paths are relative to the specified existing root.

```sh
python3 benchmarks/storage/store.py import /tmp/fixture.sqlite \
  results diagnostics --root /tmp/benchmark-fixtures
python3 benchmarks/storage/store.py verify /tmp/fixture.sqlite
python3 benchmarks/storage/store.py list /tmp/fixture.sqlite
mkdir -p /tmp/benchmark-export
python3 benchmarks/storage/store.py export /tmp/fixture.sqlite \
  results/example.json --root /tmp/benchmark-export
python3 benchmarks/storage/store.py read /tmp/fixture.sqlite results/example.json
python3 benchmarks/storage/store.py restore /tmp/fixture.sqlite \
  --root /tmp/benchmark-export --prefix diagnostics
python3 -B -m unittest discover -s benchmarks/storage -p test_store.py
```

Import accepts files and directories, commits the whole invocation atomically,
and permits idempotent reimport. Changed bytes or kinds at an existing path fail
and roll back the batch. `--kind` overrides inferred result, diagnostic, reference,
or artifact classification. Compression and decompression stream in bounded
chunks; each incoming compressed blob uses a temporary file before insertion.
`list` emits one JSON object per indexed path. `verify` checks SQLite integrity,
path safety, foreign keys, and every unique blob's decompressed size and digest.
`list --prefix PATH` selects an original subtree. `read` verifies the complete
record into a temporary file before emitting its exact bytes on stdout; this is
the narrow Python bridge used by the optional report reader.

Export writes one record at its original relative path beneath the destination
root, verifies its bytes, and atomically publishes the complete file without
overwriting existing files or symlinks. Missing subdirectories are created.
Traversal paths and source/destination symlink components are rejected. Source
symlinks are unsupported, including symlinks within the root. Export preserves
bytes and path identity; file permissions and timestamps are not archived.
`restore` verifies the store first, then exports selected original paths beneath
an existing destination root. Each file is published atomically without overwrite;
the whole tree is not one filesystem transaction. Use an empty scratch root and
discard an incomplete restoration if interrupted. Historical proof scripts remain
unchanged and can consume the restored relative tree in an isolated checkout.

## Optional report reader and raw links

`runtime/report.mjs` is optional. Its default records directory reads canonical
SQLite alongside loose JSON; custom directories can pass an explicit archive:

```js
import {updateReport} from './benchmarks/runtime/report.mjs';
updateReport({
  archive: '/tmp/fixture.sqlite',
  archivePrefix: 'runtime/results', // Original subtree; depends on import root.
  recordsDirectory: '/tmp/loose-results',
  output: '/tmp/preview/BENCHMARKS.md'
});
```

Importing with the repository's `benchmarks` directory as the source root gives
the default `runtime/results` prefix. The reader preserves original basenames,
checks original sizes and SHA256 digests, deduplicates identical loose/archive
copies, and rejects byte conflicts before replacing the report. It reads only
direct `.json` children, matching existing report discovery. Report builders and
historical approximate artifact validation receive the same parsed original data.
Archive-only reading permits the loose results directory to be absent. The
archive option requires `python3` 3.11+; default reporting does not invoke Python.

Generated raw links retain their original `.json` paths. Restore those files into
the report's relative tree before sharing an archive-only preview. For an archive
imported relative to `benchmarks`, a scratch preview can restore records with:

```sh
mkdir -p /tmp/preview/benchmarks
python3 benchmarks/storage/store.py restore /tmp/fixture.sqlite \
  --root /tmp/preview/benchmarks --prefix runtime/results
```

This supplies `/tmp/preview/benchmarks/runtime/results/*.json` for links from
`/tmp/preview/BENCHMARKS.md`. Preserve access to linked workload sources too.
Before replacing loose records, choose and prove the equivalent persistent raw
export/restore workflow. Report compatibility alone does not supply hosted raw
downloads, or make historical absolute checkout paths automatically portable.

## Migration sequence

1. Review this utility and its fixture tests. Keep all original archives.
2. Prove optional report reading, historical artifact validation, and raw links
   against restored fixtures. The report CLI and benchmark harness still use
   loose files by default; historical proof consumers may require exact relative
   tree restoration in an isolated checkout.
3. Import into a separate database, verify every blob, and compare restored
   records against their original SHA256 digests. Validate report equivalence
   before considering any archive replacement.
4. Replace loose copies only after restored hashes, report equivalence, and
   raw-record link compatibility are proven. Add derived timing/query tables only
   if needed; original bytes remain the source of truth.

This utility does not delete files, replace historical records, or reinterpret
source-only validation as measured benchmark results.


## Accepted raw analytics and restoration

`records.sqlite` preserves exact immutable original bytes. `analytics_runs`, `analytics_workloads`, `analytics_samples` and `analytics_summaries` index accepted timing records by raw SHA256 and captured source SHA256, retaining engine/process/sample boundaries. `analytics_profiles` retains raw counters; `analytics_profile_resources` exposes floating-point per-operation values. Operations come from the original captured workload, including two operations per pop/shift iteration. Summaries average each reciprocal duration. Provenance includes semantic epoch, measurement scope, source pair, toolchain and hardware; checkpoint queries never merge different runs. Correctness-only records are excluded.

Use `npm run checkpoint` from `benchmarks/runtime` for concise native/compiled-Node review, or add `-- --json` for query data. A ratio below1 indicates a slower complete workload and prompts review; it does not by itself prove a bug. Peak normalization means retained-batch peak divided by operations, not isolated-call peak. Input adaptation is separate.

Closed checkpoint86 timing/profile JSON is imported without parsing or reserializing its authoritative bytes. Derived analytics is separate. Original/restored/archive paths, sizes and SHA256 must match before any loose copy is removed. Proof references and historical links keep their original logical filenames. To restore an original link or revalidate a historical proof, export the exact file (no overwrite is permitted):

```sh
python3 benchmarks/storage/store.py export benchmarks/records.sqlite runtime/results/2026-10-03T13-42-38-756Z.json --root benchmarks
python3 benchmarks/storage/store.py export benchmarks/records.sqlite runtime/diagnostics/2026-10-03T13-46-03-175Z-native-allocations.json --root benchmarks
```

For scratch revalidation, use an existing empty directory as `--root`; exports recreate the relative tree. Optional `npm run report` reads archived runtime JSON directly, so restoration is needed only for consumers requiring loose paths. Normal runners skip Markdown generation; `--report` opts in. Existing BENCHMARKS.md and proof bytes remain unchanged. [Actual ingestion and verification](./closed-checkpoint-86-analytics/) record the closed raw migration; earlier migrations retain their original dated statistics.

Closed86 raw migration stores 281,199,604 original bytes while SQLite grows by 15,589,376 bytes, reclaiming 265,613,312 allocated bytes before evidence overhead. Current SQLite includes30,504 ordered timings,328 throughput summaries and123 resource samples. The accepted query finds native mean throughput above compiled Node on18/41 rows;23 slower rows prompt review. Both original raw files remain exactly restorable. [Current statistics](./closed-checkpoint-86-analytics/post-ingestion-stats.json) and [query verification](./closed-checkpoint-86-analytics/query-verification.json) preserve the actual results; no measurements repeat.
