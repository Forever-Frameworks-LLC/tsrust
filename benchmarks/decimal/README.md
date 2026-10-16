# Exact decimal money workload

Install with `npm ci --prefix benchmarks/decimal --ignore-scripts`.
Run `TSRUST=/path/to/tsrust node benchmarks/decimal/run.mjs --verify`.
Both engines parse decimal text, multiply price by quantity, add a fee, scale to
cents and extract exact integers. An independent integer formula verifies all
256 quantities. Node uses pinned Decimal.js 10.6.0 with precision 100 and explicit
38-digit/28-place result bounds; these bounded inputs never need 100-digit
rounding. This counterpart is a workload, not a general replacement decimal API.

Omit `--verify` only for a feature-family measurement (three processes, eight
warmups, 31 samples). Timing remains unmeasured. Native builds use the ordinary
bundled dependency CLI path. Temporary source, executables and module links are
removed after each run. Source fingerprints and mutation guards are mandatory.
