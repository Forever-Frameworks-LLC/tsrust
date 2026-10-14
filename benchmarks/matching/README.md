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
separate. No timing has been accepted for this workload yet.
