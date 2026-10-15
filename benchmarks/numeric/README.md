# Exact numeric boundaries

`TSRUST=/path/to/tsrust node benchmarks/numeric/run.mjs --verify` checks both
engines against an independent arithmetic checksum. Omit `--verify` only for a
feature-family measurement: three processes, eight warmups and 31 samples.
The Node counterpart uses bigint for signed integer representation and exact
round-trip checks. Both engines exercise successful int/number/f32 conversion
and signed-width rejection. This is a bounded conversion workload, not a claim
that all Number and bigint operations have equivalent costs. Results are not yet
measured. Source fingerprints and mutation guards are mandatory.
