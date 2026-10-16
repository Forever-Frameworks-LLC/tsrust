# Calendar billing workload

Install with `npm ci --prefix benchmarks/calendar --ignore-scripts`.
Run `TSRUST=/path/to/tsrust node benchmarks/calendar/run.mjs --verify`.
The native calendar backend and date-fns 4.1.0 both clamp January 31 to leap-day
February, add 256 different day offsets, format/parse each date and count calendar
days. The independent integer oracle is `256 * 29 + 256 * 255 / 2 = 40064`.
Node runs with UTC explicitly selected; native calendar dates have no time zone.
Both paths validate text/date bounds and return checked results. The reference
wrapper covers this workload, not every source int or date-fns option.

Omit `--verify` only for family timing: three processes, eight warmups and 31
samples. Timing remains unmeasured. Temporary builds and module links are removed.
Fingerprints and mutation guards prevent changing the scenario silently.
