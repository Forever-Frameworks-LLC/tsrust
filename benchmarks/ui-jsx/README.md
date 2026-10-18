# JSX syntax witness

Run `TSRUST=/path/to/tsrust node benchmarks/ui-jsx/verify.mjs`. The runner compiles
Tsrust JSX and TypeScript JSX with an explicit local factory, then checks both
outputs against a fixed independent string oracle. It records exact source hashes
and removes temporary binaries. This is untimed syntax/lowering evidence, not UI
rendering, browser interactivity or a throughput measurement.
