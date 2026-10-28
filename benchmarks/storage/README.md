# Compact benchmark storage

## Current analytics policy

Public comparisons use [the benchmark-only dataset](./PUBLIC_DATA.md),
`../measurements.sqlite`. It contains no archived logs or source files. The
ignored `../records.sqlite` is a legacy mixed evidence archive with local paths;
do not publish it. The owner's October 26, 2026 direction ends routine imports
of logs, source snapshots and session receipts. The migration history below
describes existing retained evidence, not the public dataset or current log policy.

SQLite is the persistent analytics source of truth. Accepted raw run records must retain their original bytes and source/toolchain identities; queryable timing and allocation summaries are derived data. Closed loose records can be removed only after exact import, verification and restoration are proved. Active checks may continue using loose temporary files.

At completed feature-family milestones, review correctness and matched native versus Node-after-tsc performance before optimization or further dependencies. Focused implementation and benchmark readiness take priority between these milestones; gate counts do not trigger heavy runs. Preserve other runtimes as context. Compare equivalent operation counts and measurement scopes, keep min/average/max throughput and individual samples, and normalize allocation metrics. A slower measurement is an investigation signal rather than proof of a compiler bug.

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
separate source of disk use. Completed artifact migrations, closed checkpoint86/91/96 raw imports, the affected-only checkpoint96 refinement and Gate97–99 focused evidence now store
**26,083 original records in 18,390 blobs, occupying 186,728,448 bytes**. Gate98 ended with 23,976 records in 16,536 blobs and 179,654,656 bytes. Gate97 ended with 22,507 records in 15,107 blobs and 172,138,496 bytes; before that artifact-only import, checkpoint96 contained 21,130 records in 13,855 blobs and occupied 167,780,352 bytes. Verified
removal reclaimed **2,329,759,744 allocated bytes (about 2,222 MiB)** before migration
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

Checkpoint96 added 8,928 individual timing samples, 96 throughput summaries and 144 allocation profiles. The original 48-comparison baseline retains all 21 slower native results for review; all disposal checks are zero. Both raw records were restored with exact original paths, sizes and hashes before removing only their canonical loose copies. Frozen originals remain available for the pending refinement audit. See [verified import](./closed-checkpoint-96-analytics/verified-import.json) and [scoped cleanup](./closed-checkpoint-96-analytics/cleanup.json). Historical raw links remain recoverable with `store.py export` as described below.

The affected-only integer concat refinement adds 6,138 timing samples, 66 rate summaries and 99 profiles while preserving the original baseline. Its 15 comparable rows have fewer allocations and observed native mean throughput improvements; all 31 slower results in the targeted 33-row cohort remain recorded. Both new raw records were restored exactly before scoped cleanup. See [refinement verification](./closed-checkpoint-96-refinement-analytics/verified-import.json) and [cleanup](./closed-checkpoint-96-refinement-analytics/cleanup.json). The database now contains 58,218 individual timing samples and 570 allocation profiles; records remain separated by original run and compiler provenance.

Gate97 added 1,377 focused compiler, editor, root and benchmark evidence paths as artifact records, adding 4,358,144 database bytes. Each selected record was independently decoded and exported to scratch with exact original size and SHA256 verification; no files were deleted and no timing samples were added. The existing 58,218 timing samples and 570 profiles remain the last measured checkpoint history. See [artifact import verification](./gate97-artifact-import/accepted.json) and [restoration identities](./gate97-artifact-import/verified-records.json). Artifact paths were imported relative to the repository root; use that same relative path with `store.py export` and an empty scratch root.

Gate98 added 1,469 artifact paths, including exact source epochs, accepted outputs, command receipts and original failures. Independent SQLite decoding and scratch export verified every selected size and SHA256; 43,948,327 original bytes occupy 6,643,962 compressed payload bytes. No timing samples were added. After exact verification, [scoped cleanup](./gate98-artifact-import/cleanup.json) removed only the 1,464 packaged duplicate objects and their scratch exports, reclaiming 90,628,096 allocated bytes; original source workspaces and indexes remain. See [accepted import](./gate98-artifact-import/accepted.json) and [exact restoration identities](./gate98-artifact-import/verified-records.json). The [restoration map](../runtime/diagnostics/gate98-closure-evidence/restoration-map.json) maps original absolute paths and source versions to restorable repository-relative objects.

Gate99 added 2,107 artifact paths with exact source epochs, complete outputs, command receipts, original failures and final correspondence. All selected records were independently decoded and exported to scratch with exact original size and SHA256 verification; 44,361,900 original bytes occupy 6,392,564 compressed payload bytes. Final3’s complete 307-input epoch is preserved. The restoration map explicitly lists 52 intermediate historical versions recorded only by digest; those unsnapshotted bytes are not claimed restored. No timing samples were added. After exact verification, [scoped cleanup](./gate99-artifact-import/cleanup.json) removed only 2,101 packaged object duplicates and their 2,101 matching scratch exports, reclaiming 93,134,848 allocated bytes (86,061,056 net bytes after database growth). Original sources, workspaces, maps, scripts and receipts remain. See [accepted import](./gate99-artifact-import/accepted.json), [exact restoration identities](./gate99-artifact-import/verified-records.json), and [original-path restoration map](../runtime/diagnostics/gate99-closure-evidence/restoration-map.json).

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

Closed86 raw migration stores 281,199,604 original bytes while SQLite grows by 15,589,376 bytes, reclaiming 265,613,312 allocated bytes before evidence overhead. At that migration, SQLite included 30,504 ordered timings, 328 throughput summaries and 123 resource samples. The accepted query finds native mean throughput above compiled Node on18/41 rows;23 slower rows prompt review. Both original raw files remain exactly restorable. [Current statistics](./closed-checkpoint-86-analytics/post-ingestion-stats.json) and [query verification](./closed-checkpoint-86-analytics/query-verification.json) preserve the actual results; no measurements repeat.


Checkpoint91 adds 12,648 ordered timings and 204 profiles from 68 matched native/compiled-Node comparisons. Current SQLite holds **43,152 timings, 464 throughput summaries and 327 profiles**. The accepted query finds 34 native mean-throughput wins and 34 slower rows to review. Every output and final disposal check is zero; these rates cover each complete workload pipeline. No Markdown was generated.

The two original records total 172,269,980 bytes. SQLite grows by 7,700,480 bytes; verified loose removal reclaims 164,573,184 allocated bytes before evidence overhead. [Actual migration statistics](./closed-checkpoint-91-analytics/postmigration-stats.json) and [the SQLite query](./closed-checkpoint-91-analytics/checkpoint-query.json) preserve the results. Restore exact logical filenames when an older proof or link requires loose files:

```sh
python3 benchmarks/storage/store.py export benchmarks/records.sqlite runtime/results/2026-10-03T18-02-58-195Z.json --root benchmarks
python3 benchmarks/storage/store.py export benchmarks/records.sqlite runtime/diagnostics/2026-10-03T18-03-45-893Z-native-allocations.json --root benchmarks
```
