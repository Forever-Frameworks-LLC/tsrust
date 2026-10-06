# Historical benchmark artifacts

The initial 29 pre-Gate55 diagnostic artifact directories listed in
[the executed migration manifest](./historical-artifact-migration-2026-10-02.json) are archived in
[records.sqlite](../records.sqlite). All 4,264 original files retain exact bytes,
relative paths, sizes and SHA256 digests; 3,876 unique compressed blobs are stored.
The archive index is [historical-artifact-index-2026-10-02.jsonl](./historical-artifact-index-2026-10-02.jsonl).

The migration independently verified SQLite decoding, every original file, every
scratch-restored file, and complete path-set equality before removing loose copies.
Every original was rehashed immediately before removal. All existing top-level
raw JSON, results, workload/reference/dependency files, Gate55 evidence, benchmark
report and storage utility sources remained unchanged. No compiler or benchmark
execution occurred.

Original allocated size: 217,894,912 bytes. SQLite allocated size immediately after that migration:
59,138,048 bytes. Gross allocated storage reclaimed: 158,756,864
bytes, before this small manifest/index/documentation overhead. Original logical
size was 205,982,770 bytes; SQLite logical size is 59,138,048 bytes.

## Exact restoration and historical proof checks

From the repository root:

```sh
python3 benchmarks/storage/store.py verify benchmarks/records.sqlite
mkdir -p /tmp/tsrust-historical-proof/benchmarks
python3 benchmarks/storage/store.py restore benchmarks/records.sqlite \
  --root /tmp/tsrust-historical-proof/benchmarks --prefix runtime/diagnostics
```

This restores original `benchmarks/runtime/diagnostics/...` paths beneath the
scratch checkout. Revalidate historical proof `artifacts[].path`/`sha256` bindings
against those restored bytes; top-level proof JSON remains loose in the repository.
No historical timing or compiler runs need repeating. For one archived subtree,
replace `--prefix runtime/diagnostics` with its exact path from the migration
manifest. `store.py list` gives every archived path, kind, size and digest.

Historical scripts that hardcode the original checkout root are not automatically
portable. To restore the exact original paths in this checkout, use
`--root benchmarks` instead of the scratch destination. Restore refuses to
overwrite files. Keep historical proof/script bytes unchanged. File permissions
and timestamps are not archived; explicitly set permissions if executing a
restored historical tool is later required.

Keep backups of `benchmarks/records.sqlite`. The recorded database SHA identifies
this completed migration state; later authorized append-only imports change the
database hash while preserving every indexed original byte identity.

## Closed Gate56 artifacts

After Gate56 runtime/native/profile work completed, its three exact compiler,
source-validation and runtime artifact directories were compacted with
[compact_closed_gate.py](./compact_closed_gate.py). The actual runtime seal remains
loose and is bound by SHA256 in [the saved explicit plan](./closed-gate-56-migration/plan.json).
[The executed manifest](./closed-gate-56-migration/migration.json) and
[verified path/size/SHA256 index](./closed-gate-56-migration/verified-index.json)
record all 755 original files. All 399 top-level proof/result JSON and BENCHMARKS
files checked before and after remain byte-identical. No compiler or benchmark run
was repeated.

Selected originals occupied 171,923,231 logical bytes and 174,034,944 allocated
bytes. SQLite grew by 4,325,376 bytes to 63,463,424 bytes. Gross allocated storage
reclaimed was 169,709,568 bytes; the saved plan/index/verification/manifest evidence
occupies 614,400 allocated bytes. The prior migration manifest's database SHA
identifies its historical state; each migration manifest records its then-current DB SHA.

To inspect all archive record bindings, use `store.py list benchmarks/records.sqlite`.
To restore Gate56 source evidence into scratch, use the earlier restoration command
with this exact prefix:

```sh
python3 benchmarks/storage/store.py restore benchmarks/records.sqlite \
  --root /tmp/tsrust-historical-proof/benchmarks \
  --prefix runtime/diagnostics/2026-10-02T12-42-00-000Z-array-fill-copy-within-source-validation-artifacts
```

Restore the other two exact prefixes from the saved plan when their historical
proof consumers need them. Top-level proof JSON keeps original paths and hashes.

For each future completed gate, create an explicit JSON plan of its exact direct
`runtime/diagnostics/*-artifacts` directory names and the actual completed runtime
proof path/SHA256. Finish every native/runtime/profile/seal consumer first. Run:

```sh
python3 benchmarks/storage/compact_closed_gate.py \
  --store benchmarks/storage/store.py --root benchmarks \
  --database benchmarks/records.sqlite --plan /tmp/closed-gate-plan.json \
  --output benchmarks/storage/closed-gate-ACTUAL_GATE-migration
```

Use a fresh output directory outside the selected trees. The command verifies the
archive, independently decodes selected blobs, restores exact scratch paths and
compares every size/SHA256 before removing selected leaves. It rehashes each leaf
immediately before removal and restores removed originals from scratch if removal
or final verification fails. An earlier failure leaves originals intact; verified
imports can safely remain in the append-only store. No wildcard gate discovery or
automatic compaction runs. Keep top-level proof/result JSON and active gate evidence
loose. Back up the database and retain each migration's plan/index/manifest.

## Closed Gate55 artifacts

After every Gate55 consumer completed, the four remaining higher-order source,
compiler, storage-support and runtime artifact directories were compacted using
[the exact saved plan](./closed-gate-55-migration/plan.json), bound to the actual
completed Gate55 runtime proof. [The executed manifest](./closed-gate-55-migration/migration.json)
and [verified original index](./closed-gate-55-migration/verified-index.json) retain
all 1,706 path/size/SHA256 identities. Originals occupied 130,066,954 logical bytes
and 134,889,472 allocated bytes. SQLite grew by 27,934,720 bytes; gross allocated
space reclaimed was 106,954,752 bytes. Migration evidence occupies another
1,249,280 allocated bytes. All 399 protected top-level proof/result JSON and
BENCHMARKS files remained byte-identical.

Immediately after Gate55 compaction, the archive contained 6,725 original records in 5,845 unique compressed blobs
and occupied 91,398,144 bytes. Across the initial historical, Gate56 and Gate55
migrations, archived originals occupied 526,819,328 allocated bytes, giving
435,421,184 bytes of gross allocated savings before migration documentation/index
overhead. No completed diagnostic artifact directory remains loose. New active
gate evidence continues to stay loose until its consumers finish.

For exact Gate55 restoration, use each explicit prefix from its saved plan with
`store.py restore`; the root/scratch/no-overwrite rules above apply unchanged.
The Gate55 migration manifest records its then-current database SHA. Earlier
manifest database hashes are valid historical state identities.

## Closed Gate57 artifacts

The three completed copying-update compiler/source/runtime artifact directories
are archived under [the exact runtime-seal-bound plan](./closed-gate-57-migration/plan.json).
[The executed manifest](./closed-gate-57-migration/migration.json) and
[verified path/size/SHA256 index](./closed-gate-57-migration/verified-index.json)
record all 791 original files. Independent decoding and exact scratch restoration
passed before each original was rehashed and removed. All 404 top-level proof/result
JSON and BENCHMARKS files and their complete path sets remained byte-identical.

Originals occupied 174,356,085 logical and 176,517,120 allocated bytes. SQLite grew
by 3,915,776 bytes; gross allocated savings were 172,601,344 bytes. Migration
evidence occupies 630,784 allocated bytes, leaving 171,970,560 bytes reclaimed
before any guide growth. No tests or compiler/native/runtime/profile runs repeated.

After Gate57, archive snapshot: **7,516 records / 6,394 unique blobs / 95,313,920 bytes**.
Combined original allocated storage was 703,336,448 bytes, so total gross allocated
savings are **608,022,528 bytes (about 580 MiB)** before evidence/docs overhead.
The Gate57 manifest records its then-current database SHA. Restore each of the three
explicit prefixes from its saved plan with `store.py restore` when historical proof
consumers need exact original paths. The scratch/no-overwrite rules above apply.

## Closed Gate58 artifacts

The three completed toSpliced compiler/source/runtime directories are archived
under [the exact completed-runtime-bound plan](./closed-gate-58-migration/plan.json).
[The executed manifest](./closed-gate-58-migration/migration.json) and
[verified original index](./closed-gate-58-migration/verified-index.json) retain all
500 original path/size/SHA256 identities. Independent decoding and exact scratch
restoration passed before per-leaf rehash/removal. All 409 protected top-level
proof/result JSON and BENCHMARKS paths/bytes remained exact.

Originals occupied 63,963,900 logical / 65,253,376 allocated bytes. SQLite grew by
2,314,240 bytes; gross allocated savings were 62,939,136 bytes, leaving 62,504,960
bytes after 434,176 bytes of migration evidence and before any guide growth. No
tests or compiler/native/runtime/profile checks repeated. Restore the exact three
prefixes from the saved plan when historical proof consumers need original paths.

After Gate58, archive: **8,016 records / 6,717 unique blobs / 97,628,160 bytes**. Combined
original allocated storage was 768,589,824 bytes, yielding **670,961,664 bytes
(about 640 MiB)** gross savings before evidence/docs. The Gate58 manifest records
its then-current database SHA; previous DB hashes identify historical states.

## Closed Gate59 artifacts

Gate59's three completed lastIndexOf compiler/source/runtime artifact directories
are archived under [the exact runtime-proof-bound plan](./closed-gate-59-migration/plan.json).
[The executed manifest](./closed-gate-59-migration/migration.json) and
[verified original index](./closed-gate-59-migration/verified-index.json) preserve
all 477 original path/size/SHA256 identities. Independent decoding and exact scratch
restoration passed before each selected original was rehashed and removed. All
414 protected top-level proof/result JSON and BENCHMARKS paths/bytes stayed exact.

Originals occupied 12,592,069 logical / 13,877,248 allocated bytes. SQLite grew by
1,204,224 bytes, reclaiming 12,673,024 gross allocated bytes. Migration evidence
occupies 421,888 bytes; net savings before guide growth are 12,251,136 allocated
bytes. No utility tests or compiler/native/runtime/profile checks repeated. Restore
the exact three saved plan prefixes when historical proof consumers need them.

After Gate59, the archive held **8,493 records / 7,019 unique blobs / 98,832,384 bytes**.
Combined original allocated storage was 782,467,072 bytes, yielding **683,634,688
bytes (about 652 MiB)** gross savings before evidence/docs. The Gate59 manifest
records its then-current DB SHA; each earlier DB hash identifies its historical state.

## Closed Gate60 artifacts

Gate60's four completed String.lastIndexOf compiler/source/supplement/runtime
artifact directories are archived under [the exact saved plan](./closed-gate-60-migration/plan.json),
bound to the actual completed runtime proof and acceptance certificate.
[The executed manifest](./closed-gate-60-migration/migration.json) and
[verified original index](./closed-gate-60-migration/verified-index.json) preserve
all 394 original paths, sizes and SHA256 digests, including original light checks,
native borrow supplement and failed-first-native/packaging evidence. Independent
decoding and exact scratch restoration passed before per-leaf rehash/removal. All
420 protected top-level proof/result JSON and BENCHMARKS paths/bytes stayed exact.
The actual completed acceptance certificate was verified before/after and retained
in this migration's evidence.

Originals occupied 13,428,890 logical / 14,442,496 allocated bytes. SQLite grew by
1,138,688 bytes, reclaiming 13,303,808 gross allocated bytes. Migration evidence
occupies 376,832 bytes, leaving 12,926,976 allocated bytes reclaimed before guide
growth. No utility tests or compiler/native/runtime/profile checks repeated.
Restore the four exact prefixes from the saved plan for historical proof consumers.

After Gate60, archive: **8,887 records / 7,236 unique blobs / 99,971,072 bytes**. Combined
original allocated storage was 796,909,568 bytes, yielding **696,938,496 bytes
(about 665 MiB)** gross savings before evidence/docs. The Gate60 manifest records
its then-current database SHA; earlier hashes retain their historical state identities.


## Closed Gate61 artifacts

Gate61's four completed compiler/source/supplement/runtime artifact directories
are archived under [the exact saved plan](./closed-gate-61-migration/plan.json).
[The executed migration](./closed-gate-61-migration/migration.json) retains 761
original records, including the failed native emit and corrected benchmark sources.
Independent decoding and exact scratch restoration verified every original path,
size and SHA256 before removal. All 426 protected top-level JSON/report files
remain byte-exact. The completed acceptance certificate is retained with migration
evidence. No compiler regression or benchmark measurements were repeated.

Original artifacts occupied 493,462,415 logical / 495,468,544 allocated bytes.
SQLite grew by 4,313,088 bytes, reclaiming 491,155,456 gross allocated bytes.
Restore only the four exact prefixes in the saved plan when historical consumers
need original files.

After Gate61, archive: **9,648 records / 7,598 unique blobs / 104,284,160 bytes**.
Combined gross allocated savings total **1,188,093,952 bytes (about 1,133 MiB)**
before migration evidence and documentation overhead. Each migration's database
hash identifies its own historical state.


## Closed Gate62 artifacts

Gate62's compiler, source, single observations set and runtime artifact directories
are archived under [the exact completed-gate plan](./closed-gate-62-migration/plan.json).
[The executed migration](./closed-gate-62-migration/migration.json) preserves 305
original records. Independent decoding and exact scratch restoration verified
all paths, sizes and SHA256 digests before removal. All 431 protected top-level
JSON/report files remain byte-exact; the acceptance certificate is retained.
No compiler checks or measurements repeated during compaction.

Original artifacts occupied 93,654,099 logical / 94,265,344 allocated bytes.
SQLite grew by 1,232,896 bytes, reclaiming 93,032,448 gross allocated bytes.
Restore only the four exact prefixes in the saved plan for historical consumers.

After Gate62, archive: **9,953 records / 7,776 unique blobs / 105,517,056 bytes**.
Combined gross allocated savings total **1,281,126,400 bytes (about 1,222 MiB)**
before migration evidence and documentation overhead. Database hashes identify
historical states, not a permanently fixed archive file.


## Closed Gate63 artifacts

Gate63's compiler, source, runtime and closure artifact directories are archived
under [the exact completed-gate plan](./closed-gate-63-migration/plan.json).
[The executed migration](./closed-gate-63-migration/migration.json) preserves
1,011 original records, including failed checks and the interrupted runtime attempt.
Independent decoding and exact scratch restoration verified original paths, sizes
and SHA256 digests before removal. All 436 protected top-level JSON/report files
remain byte-exact. The completed acceptance certificate and documentation
supplement are retained with migration evidence. No compiler checks or benchmark
measurements repeated during compaction.

Original artifacts occupied 48,270,869 logical / 50,077,696 allocated bytes.
SQLite grew by 1,867,776 bytes, reclaiming 48,209,920 gross allocated bytes.
Restore only the four exact prefixes in the saved plan for historical consumers.

After Gate63, archive: **10,964 records / 8,275 unique blobs / 107,384,832 bytes**
(about 102.4 MiB). Combined gross allocated savings total **1,329,336,320 bytes
(about 1,268 MiB)** before migration evidence and documentation overhead.
[The post-migration inventory](./closed-gate-63-migration/postmigration-stats.json)
records this database state; earlier database hashes identify historical states.

## Closed Gate64 artifacts

Gate64's compiler, source, runtime and closure artifact directories are archived
under [the exact completed-gate plan](./closed-gate-64-migration/plan.json).
[The executed migration](./closed-gate-64-migration/migration.json) preserves
568 original records, including native overflow evidence and the failed source-only
seal packaging attempt with its revised freeze. Independent decoding and exact
scratch restoration verified every original path, size and SHA256 before removal.
All 441 protected top-level JSON/report paths and bytes remain exact. The completed
acceptance certificate and postclosure README supplement are retained with migration
evidence. No tests, compiler checks or benchmark measurements repeated during compaction.

Original artifacts occupied 9,542,530 logical / 10,653,696 allocated bytes.
SQLite grew by 1,040,384 bytes, reclaiming 9,613,312 gross allocated bytes.
Migration evidence occupies 520,192 allocated bytes, leaving 9,093,120 bytes
reclaimed before guide growth. Restore only the four exact prefixes in the saved
plan when historical consumers need original files.

After Gate64, archive: **11,532 records / 8,563 unique blobs / 108,425,216 bytes**
(about 103.4 MiB). Combined gross allocated savings total **1,338,949,632 bytes
(about 1,277 MiB)** before migration evidence and documentation overhead.
[The post-migration inventory](./closed-gate-64-migration/postmigration-stats.json)
records this state. Its database SHA256 is
`0b02cef2b5e4c6d1aa9fa0b14693d92095be453e32d61db92877473ffa3b2313`;
earlier database hashes identify historical states.


## Closed Gate65 artifacts

Gate65's compiler, source, runtime and closure artifacts are archived under
[the completed-gate plan](./closed-gate-65-migration/plan.json).
[The migration](./closed-gate-65-migration/migration.json) preserves 987 files
with exact-byte import, independent decoding and scratch restoration before
removal. The complete set of 446 top-level raw JSON/report paths and their
SHA256 identities remains unchanged.

Original storage was 8,355,650 logical bytes and 10,035,200 allocated bytes.
Database growth was 1,474,560 bytes, reclaiming 8,560,640 allocated bytes gross
and 7,806,976 bytes after migration evidence.

Current archive: **12,519 records / 9,010 unique blobs / 109,899,776 bytes**
(about 104.8 MiB). Combined gross allocated savings total **1,347,510,272 bytes
(about 1,285 MiB)** before evidence and documentation overhead.
[The actual inventory](./closed-gate-65-migration/postmigration-stats.json)
retains exact counts and sizes. Current database SHA256:
`3240ba7f8ee6f07cba501b9b2b2777c57238a2603076199c36bebcb3bb9601dd`. Top-level proofs and linked raw results remain loose.


## Closed Gate66 artifacts

Gate66's compiler, source, runtime and closure artifact directories are archived
under [the exact completed-gate plan](./closed-gate-66-migration/plan.json).
[The executed migration](./closed-gate-66-migration/migration.json) preserves
273 original records, including the failed full regression, failed first
parser repair and measured frame diagnosis. Independent decoding and scratch
restoration verified original paths, sizes and SHA256 before removal. All
451 protected top-level JSON/report paths and bytes remain unchanged.
The completed certificate and documentation supplement are retained with the
migration evidence. No tests or benchmark measurements repeated during compaction.

Original artifacts occupied 6,030,336 logical bytes and
6,651,904 allocated bytes. SQLite grew by
749,568 bytes, reclaiming 5,902,336
allocated bytes gross and 5,566,464 bytes after
migration evidence, before guide growth. Restore only the four exact prefixes
in the saved plan when historical consumers need original files.

Current archive: **12,792 records / 9,197 unique blobs / 110,649,344 bytes**
(about 105.5 MiB). Combined gross allocated savings total
**1,353,412,608 bytes (about 1,291 MiB)** before evidence and
documentation overhead. [The actual inventory](./closed-gate-66-migration/postmigration-stats.json)
retains exact counts and sizes. Database SHA256 at this migration:
`b0558a9579db4926827a709d806c116c23f69a897ca69279e9dade63e8705f90`. Linked raw results and top-level proofs remain loose.


## Closed Gate67 artifacts

Gate67's compiler, source, runtime and closure artifact directories are archived
under [the exact completed-gate plan](./closed-gate-67-migration/plan.json).
[The executed migration](./closed-gate-67-migration/migration.json) preserves
326 original records, including the failed full regression, failed first
parser repair and measured frame diagnosis. Independent decoding and scratch
restoration verified original paths, sizes and SHA256 before removal. All
456 protected top-level JSON/report paths and bytes remain unchanged.
The completed certificate and documentation supplement are retained with the
migration evidence. No tests or benchmark measurements repeated during compaction.

Original artifacts occupied 7,642,535 logical bytes and
8,327,168 allocated bytes. SQLite grew by
880,640 bytes, reclaiming 7,446,528
allocated bytes gross and 7,077,888 bytes after
migration evidence, before guide growth. Restore only the four exact prefixes
in the saved plan when historical consumers need original files.

Current archive: **13,118 records / 9,402 unique blobs / 111,529,984 bytes**
(about 106.4 MiB). Combined gross allocated savings total
**1,360,859,136 bytes (about 1,298 MiB)** before evidence and
documentation overhead. [The actual inventory](./closed-gate-67-migration/postmigration-stats.json)
retains exact counts and sizes. Database SHA256 at this migration:
`3247ea895facffd29c140de60dfa1b3d5e85a196759b0777a1fec13eb8ac96d2`. Linked raw results and top-level proofs remain loose.


## Closed Gate68 artifacts

Gate68's compiler, source, runtime and closure artifact directories are archived
under [the exact completed-gate plan](./closed-gate-68-migration/plan.json).
[The executed migration](./closed-gate-68-migration/migration.json) preserves
1,237 original records, including the failed focused group and exact repair, initial
integration guard failure, and compiler-seal path-alias repair. Independent decoding and scratch
restoration verified original paths, sizes and SHA256 before removal. All
461 protected top-level JSON/report paths and accepted byte identities remain unchanged.
Prior original hashes are retained from the accepted Gate67 ledger, with all file
stat identities unchanged before/after compaction; only the five new loose JSON
records and current report are freshly hashed.
The completed certificate and documentation supplement are retained with the
migration evidence. No tests or benchmark measurements repeated during compaction.

Original artifacts occupied 20,748,668 logical bytes and
22,863,872 allocated bytes. SQLite grew by
1,912,832 bytes, reclaiming 20,951,040
allocated bytes gross and 19,812,352 bytes after
migration evidence, before guide growth. Restore only the four exact prefixes
in the saved plan when historical consumers need original files.

Current archive: **14,355 records / 9,964 unique blobs / 113,442,816 bytes**
(about 108.2 MiB). Combined gross allocated savings total
**1,381,810,176 bytes (about 1,318 MiB)** before evidence and
documentation overhead. [The actual inventory](./closed-gate-68-migration/postmigration-stats.json)
retains exact counts and sizes. Database SHA256 at this migration:
`54e20a6f2dbe88ec2715f6035175b70ee03c0f38df9825a7e7848a09041a3852`. Linked raw results and top-level proofs remain loose.


## Closed Gate69 artifacts

Gate69's compiler, source, runtime and closure artifact directories are archived
under [the exact completed-gate plan](./closed-gate-69-migration/plan.json).
[The executed migration](./closed-gate-69-migration/migration.json) preserves
710 original records, including complete conditional outputs,
source-order and eager-branch mutation evidence, and accepted actual command records. Independent decoding and scratch
restoration verified original paths, sizes and SHA256 before removal. All
466 protected top-level JSON/report paths and accepted byte identities remain unchanged.
Prior original hashes are retained from the accepted Gate68 ledger, with all file
stat identities unchanged before/after compaction; only the five new loose JSON
records and current report are freshly hashed.
The completed certificate and documentation supplement are retained with the
migration evidence. No tests or benchmark measurements repeated during compaction.

Original artifacts occupied 82,606,073 logical bytes and
83,890,176 allocated bytes. SQLite grew by
1,957,888 bytes, reclaiming 81,932,288
allocated bytes gross and 81,174,528 bytes after
migration evidence, before guide growth. Restore only the four exact prefixes
in the saved plan when historical consumers need original files.

Current archive: **15,065 records / 10,353 unique blobs / 115,400,704 bytes**
(about 110.1 MiB). Combined gross allocated savings total
**1,463,742,464 bytes (about 1,396 MiB)** before evidence and
documentation overhead. [The actual inventory](./closed-gate-69-migration/postmigration-stats.json)
retains exact counts and sizes. Database SHA256 at this migration:
`54398d8a5f853649e1c17a8c2572afadb26addd050cd33edb8e4511322d4119f`. Linked raw results and top-level proofs remain loose.


## Closed Gate70 artifacts

Gate70's compiler, source, runtime and closure artifact directories are archived
under [the exact completed-gate plan](./closed-gate-70-migration/plan.json).
[The executed migration](./closed-gate-70-migration/migration.json) preserves
646 original records, including complete String outputs,
UTF-8 padding/source-order evidence, retained failed source packaging and its repair, and accepted actual command records. Independent decoding and scratch
restoration verified original paths, sizes and SHA256 before removal. All
471 protected top-level JSON/report paths and accepted byte identities remain unchanged.
Prior original hashes are retained from the accepted Gate69 ledger, with all file
stat identities unchanged before/after compaction; only the 5 new loose JSON
records and current report are freshly hashed.
The completed certificate and documentation supplement are retained with the
migration evidence. No tests or benchmark measurements repeated during compaction.

Original artifacts occupied 13,621,874 logical bytes and
14,811,136 allocated bytes. SQLite grew by
1,490,944 bytes, reclaiming 13,320,192
allocated bytes gross and 12,599,296 bytes after
migration evidence, before guide growth. Restore only the four exact prefixes
in the saved plan when historical consumers need original files.

Current archive: **15,711 records / 10,705 unique blobs / 116,891,648 bytes**
(about 111.5 MiB). Combined gross allocated savings total
**1,477,062,656 bytes (about 1,409 MiB)** before evidence and
documentation overhead. [The actual inventory](./closed-gate-70-migration/postmigration-stats.json)
retains exact counts and sizes. Database SHA256 at this migration:
`364395f4b500e4f3693a2de2132f2f641ee576eafd9e302fc7987090793b4ae5`. Linked raw results and top-level proofs remain loose.


## Closed Gate71 artifacts

Gate71's compiler, source, runtime and closure artifact directories are archived
under [the exact completed-gate plan](./closed-gate-71-migration/plan.json).
[The executed migration](./closed-gate-71-migration/migration.json) preserves
433 original records, including complete String outputs,
UTF-8 substring clamp/swap/source-order evidence and accepted actual command records. Independent decoding and scratch
restoration verified original paths, sizes and SHA256 before removal. All
476 protected top-level JSON/report paths and accepted byte identities remain unchanged.
Prior original hashes are retained from the accepted Gate70 ledger, with all file
stat identities unchanged before/after compaction; only the 5 new loose JSON
records and current report are freshly hashed.
The completed certificate and documentation supplement are retained with the
migration evidence. No tests or benchmark measurements repeated during compaction.

Original artifacts occupied 8,511,783 logical bytes and
9,314,304 allocated bytes. SQLite grew by
1,064,960 bytes, reclaiming 8,249,344
allocated bytes gross and 7,671,808 bytes after
migration evidence, before guide growth. Restore only the four exact prefixes
in the saved plan when historical consumers need original files.

Current archive: **16,144 records / 10,950 unique blobs / 117,956,608 bytes**
(about 112.5 MiB). Combined gross allocated savings total
**1,485,312,000 bytes (about 1,417 MiB)** before evidence and
documentation overhead. [The actual inventory](./closed-gate-71-migration/postmigration-stats.json)
retains exact counts and sizes. Database SHA256 at this migration:
`c4f67a4ff9ed626e944501cce088963a10900fd5cb3fb01a3c0f5821370a21ef`. Linked raw results and top-level proofs remain loose.


## Closed Gate72 artifacts

Gate72's compiler, source, runtime and closure artifact directories are archived
under [the exact completed-gate plan](./closed-gate-72-migration/plan.json).
[The executed migration](./closed-gate-72-migration/migration.json) preserves
558 original records, including complete primitive outputs,
String Cow copies, Boolean text/natural Boolean tokens, signed-safe integer/source-order evidence and accepted actual command records. Independent decoding and scratch
restoration verified original paths, sizes and SHA256 before removal. All
481 protected top-level JSON/report paths and accepted byte identities remain unchanged.
Prior original hashes are retained from the accepted Gate71 ledger, with all file
stat identities unchanged before/after compaction; only the 5 new loose JSON
records and current report are freshly hashed.
The completed certificate and documentation supplement are retained with the
migration evidence. No tests or benchmark measurements repeated during compaction.

Original artifacts occupied 9,441,824 logical bytes and
10,489,856 allocated bytes. SQLite grew by
1,179,648 bytes, reclaiming 9,310,208
allocated bytes gross and 8,617,984 bytes after
migration evidence, before guide growth. Restore only the four exact prefixes
in the saved plan when historical consumers need original files.

Current archive: **16,702 records / 11,255 unique blobs / 119,136,256 bytes**
(about 113.6 MiB). Combined gross allocated savings total
**1,494,622,208 bytes (about 1,425 MiB)** before evidence and
documentation overhead. [The actual inventory](./closed-gate-72-migration/postmigration-stats.json)
retains exact counts and sizes. Database SHA256 at this migration:
`2330c603b3413f828bb752201461ff7c506081e10b700a1221b2756b33985e21`. Linked raw results and top-level proofs remain loose.


## Closed Gate73 artifacts

[The executed migration](./closed-gate-73-migration/migration.json) archives 163 original integer-formatting evidence records across four finished artifact trees. Independent decoding and scratch restoration verify paths, sizes and SHA256 before removal. All 486 protected top-level JSON/report files preserve their path and stat identities; new records and the report also preserve fresh hashes. No tests or timings repeat during compaction.

SQLite grows by 544,768 bytes and reclaims 3,145,728 allocated bytes gross, before migration-evidence and documentation overhead. The archive now holds 16,865 records in 11,379 blobs, using 119,681,024 bytes. [Actual statistics](./closed-gate-73-migration/postmigration-stats.json) and the completed acceptance certificate remain durable. Linked raw results remain loose.


## Closed Gate74 artifacts

[The executed migration](./closed-gate-74-migration/migration.json) archives 343 original number-formatting evidence records across six finished artifact trees. Independent decoding and scratch restoration verify paths, sizes and SHA256 before removal. All 495 protected top-level JSON/report files preserve their path and stat identities; new records and the report also preserve fresh hashes. No tests or timings repeat during compaction.

SQLite grows by 1,347,584 bytes and reclaims 8,380,416 allocated bytes gross, before migration-evidence and documentation overhead. The archive now holds 17,208 records in 11,637 blobs, using 121,028,608 bytes. [Actual statistics](./closed-gate-74-migration/postmigration-stats.json) and the completed acceptance certificate remain durable. Linked raw results remain loose.


## Closed Gate75 artifacts

[The executed migration](./closed-gate-75-migration/migration.json) archives 372 original Number.parseFloat evidence records across four finished artifact trees. Independent decoding and scratch restoration verify every original path, size and SHA256 before removal, with each leaf rechecked immediately before unlink. All 500 protected top-level JSON/report paths and stat identities remain exact; new records and the report also preserve fresh hashes. Earlier record hashes are inherited from the bound Gate74 ledger without rehashing that corpus. No tests or timings repeat during compaction.

SQLite grows by 2,383,872 bytes and reclaims 10,444,800 allocated bytes gross, before migration-evidence and documentation overhead. The archive now holds 17,580 records in 11,876 blobs, using 123,412,480 bytes. [Actual statistics](./closed-gate-75-migration/postmigration-stats.json) bind the database SHA256 and cumulative gross savings of 1,516,593,152 bytes. The completed acceptance certificate, wrapper compatibility repair and original command evidence remain durable. Linked raw results and top-level proofs remain loose.


## Closed Gate76 artifacts

[The executed migration](./closed-gate-76-migration/migration.json) archives 210 original String.isWellFormed/String.toWellFormed evidence records across four finished artifact trees. Independent decoding and scratch restoration verify every original path, size and SHA256 before removal, with each leaf rechecked immediately before unlink. All 505 protected top-level JSON/report paths and stat identities remain exact; new records and the report also preserve fresh hashes. Earlier record hashes are inherited from the bound Gate75 ledger without rehashing that corpus. No tests or timings repeat during compaction.

SQLite grows by 712,704 bytes and reclaims 4,612,096 allocated bytes gross, before migration-evidence and documentation overhead. The archive now holds 17,790 records in 12,027 blobs, using 124,125,184 bytes. [Actual statistics](./closed-gate-76-migration/postmigration-stats.json) bind the database SHA256 and cumulative gross savings of 1,521,205,248 bytes. The completed acceptance certificate, unchanged observer bindings and original command evidence remain durable. Linked raw results and top-level proofs remain loose.


## Closed checkpoint77–81 artifacts

[The executed migration](./closed-checkpoint-81-migration/migration.json) archives 748 original records from 11 completed proof-derived artifact trees, including Gate77 documentation examples and the checkpoint closure. Every selected original path, size and SHA256 matches independently decoded archive bytes and scratch restoration before removal; each leaf is rechecked immediately before unlink. All 517 loose top-level JSON/report paths and stat identities remain exact. New records and the report retain fresh hashes; historical hashes inherit the bound Gate76 ledger without rehashing earlier raw records. No tests, builds or timings repeat during compaction. The unchanged compactor also validates the stored database using its existing verification routine.

SQLite grows by 2,670,592 bytes, reclaiming 16,474,112 allocated bytes gross before migration evidence and documentation overhead. The archive now holds 18,538 records in 12,674 blobs, using 126,795,776 bytes. [Actual statistics](./closed-checkpoint-81-migration/postmigration-stats.json) bind the database SHA256 and cumulative gross savings of 1,537,679,360 bytes. Linked raw results, profiles, top-level proofs and BENCHMARKS.md remain loose.


## Closed checkpoint82–86 artifacts

[The executed migration](./closed-checkpoint-86-migration/migration.json) archives 2,580 original records from 9 completed proof-derived artifact trees, including accepted implementation, regression, runtime and checkpoint closure evidence. Every selected original path, size and SHA256 matches independently decoded archive bytes and scratch restoration before removal; each leaf is rechecked immediately before unlink. All 527 loose top-level JSON/report paths and stat identities remain exact. New records and the report retain fresh hashes; historical hashes inherit the bound checkpoint81 ledger without rehashing earlier raw records. No tests, builds or timings repeat during compaction. The unchanged compactor also validates the stored database using its existing verification routine.

SQLite grows by 7,467,008 bytes, reclaiming 90,771,456 allocated bytes gross before migration evidence and documentation overhead. The archive now holds 21,118 records in 13,843 blobs, using 134,262,784 bytes. [Actual statistics](./closed-checkpoint-86-migration/postmigration-stats.json) bind the database SHA256 and cumulative gross savings of 1,628,450,816 bytes. Linked raw results, profiles, top-level proofs and BENCHMARKS.md remain loose.


## Closed checkpoint86 raw analytics

[Exact restore verification](./closed-checkpoint-86-analytics/raw-restore-verification.json) binds the original runtime and native-profile records, independently restored sizes and SHA256, and the completed certificate. Only those two closed loose files were removed after immediate hash rechecks; every other loose JSON/report path and stat identity remained exact. Original proof filenames and hashes are unchanged. Use the exact export commands in the [restoration guide](./README.md#accepted-raw-analytics-and-restoration) for historical consumers and links requiring loose paths. Optional report reading can consume archived runtime bytes directly.

[The actual query](./closed-checkpoint-86-analytics/checkpoint-query.json) compares one accepted source-matched run:18/41 native mean throughput wins and23 slower-row review signals. Derived analytics retains30,504 ordered timing samples,328 summaries and123 profiles with zero output/final disposal. Counters remain raw; per-operation views use captured workload operation counts, including two removals per pop/shift iteration. Retained-batch peak divided by operations is not an isolated-call peak. SQLite now uses149,852,160 bytes for21,121 original records in13,846 blobs plus derived tables; [statistics](./closed-checkpoint-86-analytics/post-ingestion-stats.json) record265,613,312 additional gross allocated bytes reclaimed. Older raw history was not migrated.
