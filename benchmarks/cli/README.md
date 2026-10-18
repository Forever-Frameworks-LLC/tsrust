# CLI process-boundary witnesses

Run `TSRUST=/path/to/tsrust node benchmarks/cli/verify-process-arguments.mjs`.
The runner compiles the paired Tsrust and TypeScript sources, executes native and
Node-after-tsc cases, and compares decoded JSON with independently supplied argument
arrays. Cases cover no arguments, empty values, spaces, Unicode, option-looking
values, quotes, newlines and 128 arguments. Temporary executable outputs are removed.
The runner records source SHA-256 fingerprints. This is untimed process-boundary
acceptance, not an argument-processing throughput benchmark.
