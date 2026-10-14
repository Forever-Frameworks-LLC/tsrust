# Generic entry preparation

Run `python3 benchmarks/compiler/function-dependencies/run.py --generic-functions --generic-inferred --generic-entry spread --verify-execution --only 32 --output /private/tmp/generic-entry-spread.json` for untimed validation. Repeat with `receiver` and `optional`. Add `--measure` only at the architecture milestone. Counts 128 and 512 are registered for discovery and scaling; registration does not prove acceptance.

Native and TypeScript sources both infer generic arguments and returns. Spread uses one-element source arrays. Receiver calls copy the returned array with `slice`. Optional calls target declared functions, so every invocation is present. Every variant has the independent execution oracle `7\n` for native and Node after tsc. The shared driver retains exact sources, hashes, tool identities, all checks and the original two-warmup/eleven-sample timing protocol.
