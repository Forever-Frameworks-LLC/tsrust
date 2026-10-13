# Generic function dependencies

Run `python3 benchmarks/compiler/function-dependencies/run.py --generic-functions --output /private/tmp/generic-function-dependencies.json` for untimed validation. Add `--measure` only at the architecture milestone. Select registered counts with `--only`.

The shared driver retains matched native `int` and TypeScript `number` sources, exact hashes, tool identities, commands, validation results, two warmups and eleven samples when timing is requested. Both sources return the unchanged input through an explicit generic chain; the independent execution oracle is `7\n`.

Current acceptance covers 128 functions for checking/emission and 64 for native ownership and throw behavior. The 512-function group is registered for the remaining scaling work, not accepted or measured. Source proof cost remains high. Active analyzer depth guards and instance budgets remain separate from queued materialization.
