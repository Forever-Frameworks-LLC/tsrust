# Benchmark measurements

[`../measurements.sqlite`](../measurements.sqlite) is the public comparison
dataset. SQLite readers need no extensions. The Python query tools use only the
standard library. No benchmark reruns are required to use this data.

## Contents and limits

The database retains individual measured durations, engine/process/sample
boundaries, operation counts, throughput summaries and allocation counters.
Run timestamps, semantic epochs, measurement scopes, source-pair hashes and
original acceptance-proof hashes keep comparisons attributable. A proof hash
identifies the local original; it does not include or publish that proof.

Public provenance contains only allowlisted toolchain versions, source/build
hashes and general hardware specifications such as CPU family, architecture,
logical CPU count and RAM. Unknown hardware descriptions are omitted rather
than copied. Input filenames are removed; their hashes remain. Free-form notes,
result strings, logs, source snapshots, executable paths, hostnames, usernames,
addresses, serial numbers and OS kernel build identifiers are excluded.
Historical detailed workload protocols remain in repository sources and reports.

This export covers the **70 already indexed accepted runs**, including
**86,622 timing samples and 825 allocation samples**. It does not claim that
every older file in the legacy archive has been indexed. Unindexed history
remains local until its measurement and acceptance structure can be verified.
No measurements have been discarded from that archive or retimed for this export.

The October 28, 2026 snapshot is 37,285,888 bytes. SHA256:
`9ddf5e9be9ec6daa5ce70f8f4cda36e3495d7b3459a8cc54696c4f277382624f`.
All 50 indexed execution comparisons retain their original checkpoint queries.
The latest addition contains 186 HTTP request-reader samples, preserving the
three-process, eight-warmup, 31-sample-per-process protocol for each engine.
The storage suite covers privacy exclusions, exact sample preservation,
failed-export cleanup, no-overwrite behavior and the bundled database contract.

Keep comparisons within matching workloads, source pairs, operation counts,
semantic epochs, scopes and toolchains. Hardware specifications do not establish
that separate runs used identical conditions. Samples from different runs are
never pooled by the checkpoint reader. Missing provenance is unknown, not proof
of equivalence. Rates describe each complete workload, not an isolated operation.

## Query

```sh
node benchmarks/runtime/checkpoint.mjs --database=benchmarks/measurements.sqlite
node benchmarks/runtime/checkpoint.mjs --database=benchmarks/measurements.sqlite --json
```

For SQL, use `analytics_runs`, `analytics_workloads`, `analytics_samples`,
`analytics_summaries` and `analytics_profiles`. The `analytics_profile_resources`
view normalizes counters by actual operations. Workloads are keyed by run,
workload identifier and source SHA256. Every slower result is retained.

## Refresh without exposing the archive

With Python 3.11+ and the local original archive:

```sh
python3 -B benchmarks/storage/public.py benchmarks/records.sqlite /tmp/measurements.sqlite
python3 -B -m unittest discover -s benchmarks/storage -p 'test_*.py'
```

The destination must not exist. The exporter opens the original read-only,
creates a fresh database and compares every copied measurement value against
the source before publishing the file. It validates schema, foreign keys,
integrity and allowlisted metadata. No deleted pages, compressed blobs or
record-path tables are copied. Failed exports leave no partial destination.
Review the resulting counts and checksum before replacing the public dataset.
An unrecognized measurement scope fails export and requires review; unknown
metadata fields are excluded. Version normalization keeps package versions,
not arbitrary command output. Missing fields must not be treated as equivalent
provenance when comparing runs.

`records.sqlite` is a separate, ignored legacy archive containing historical
development evidence and local paths. It is **not** a public download. Keep its
existing records locally; do not add routine command logs, source snapshots or
session receipts. Raw measured records and acceptance proofs remain local
inputs for exact verification. Existing exact-byte archive tools are unchanged.
