# Exact instant workload

Uses the pinned date-fns installation from `benchmarks/calendar`:
`npm ci --prefix benchmarks/calendar --ignore-scripts`.
Run `TSRUST=/path/to/tsrust node benchmarks/temporal/run.mjs --verify`.
Both paths add 256 millisecond durations to a UTC instant, format/parse timestamps
and extract epoch milliseconds. The independent checksum is `256 * 255 / 2 = 32640`.
Node uses UTC explicitly and date-fns 4.1.0. Its Date representation supports only
milliseconds: this workload does not claim nanosecond or DST-resolution parity.
Nanosecond loss rejection and named-zone transitions have separate native witnesses.

Normal measurement retains three processes, eight warmups and 31 samples.
Timing is unmeasured. Source fingerprints, mutation guards and temporary cleanup
match the calendar runner; the shared dependency lock is included in each record.
