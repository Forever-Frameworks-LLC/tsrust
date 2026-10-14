# Inferred generic calls

Run `python3 benchmarks/compiler/function-dependencies/run.py --generic-functions --generic-inferred --verify-execution --only 32 128 --output /private/tmp/inferred-generic-calls.json` for untimed validation. Add `--measure` only at the architecture milestone. The 512-function group remains registered for discovery and scaling; registration does not establish acceptance.

Native and TypeScript sources both infer type arguments and return types. Native uses `int`; TypeScript uses `number`. Each chain returns its input unchanged, with the independent execution oracle `7\n`. The shared driver preserves exact sources and hashes, tool and compiler identities, validation, and the original two-warmup/eleven-sample timing protocol. No measurements are claimed by this support change.
