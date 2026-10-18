# JSX syntax witness

Run `TSRUST=/path/to/tsrust node benchmarks/ui-jsx/verify.mjs`. The runner compiles
Tsrust JSX and TypeScript JSX with an explicit local factory, then checks both
outputs against a fixed independent string oracle. It records exact source hashes
and removes temporary binaries. This is untimed syntax/lowering evidence, not UI
rendering, browser interactivity or a throughput measurement.

`verify-counter.mjs <built-counter-site> --presentation-only` checks centered
button text, rounded styling and stable font/height metrics across viewport widths
in real Chromium. It skips the previously accepted input/lifetime interaction pass.
The regular invocation includes both presentation and interaction checks. Build
fresh artifacts with `tools/build-ui-web.mjs`; do not relabel an older artifact as
current. These checks report observations and source identities, not throughput.
