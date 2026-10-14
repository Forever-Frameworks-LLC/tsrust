# Matching workload

`TSRUST=/path/to/current/tsrust node run.mjs --verify` checks matched native and
Node-after-tsc results without storing timings. Requires the repository's pinned
Node, TypeScript and Rust versions. The runner verifies formatting idempotence,
checks every output against an independent arithmetic oracle, records source and
compiler fingerprints, and removes temporary build outputs.

Each batch performs 2,048 value-producing Result matches, integer literal matches
and named-record variant matches. The TypeScript counterpart uses discriminated
unions and switches. The timing protocol is 8 warmups and 31 measured samples in
three processes per engine; run without `--verify` only at the family milestone.
Import closed JSON records into `benchmarks/records.sqlite`, verify exact bytes,
then remove the loose records. Implementation and measurement acceptance are
separate. The first accepted timing is `match-values-2026-10-14T21-27-19-471Z.json`
in SQLite: 93 samples per engine, medians 0.002167 ms native and 0.012125 ms
Node-after-tsc; every checksum is 4203521. This is a small synthetic workload,
not an application speedup claim. The local tsconfig reuses the pinned Node types.
