# Collection constructor sources

Run `python3 benchmarks/compiler/function-dependencies/run.py --collection-constructor set --verify-execution --only 32 --output /private/tmp/collection-constructor-set.json` for untimed validation. Repeat with `map`. Add `--measure` only at the architecture milestone. Larger registered groups remain unaccepted and unmeasured while public parser admission and scaling work continue.

Each layer constructs a Set or Map from the previous constructed collection; the innermost source holds one element or entry. Native and TypeScript execution must independently print `1\n`. The shared driver retains exact sources, hashes, compiler/tool identities and checks. Timing remains two warmups and eleven samples.
