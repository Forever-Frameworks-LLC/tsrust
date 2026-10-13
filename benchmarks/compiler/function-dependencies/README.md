# Function dependency compilation

This workload isolates a forward acyclic function chain. Every function forwards
one scalar; the final function returns it. The independent result oracle is `7`.
Compiler regression witnesses execute long native chains and separately cover
owned values, inferred returns, function values, effects and diagnostics.

Use `run.py --output /private/tmp/tsrust-function-dependencies.json` for untimed
native check/emit and TypeScript checking. `--measure` uses the manifest's fixed
two warmups and eleven fresh-process samples. Select counts/phases with `--only`
and `--phases` when resuming unrun groups. Do not overwrite an earlier record.
The output retains sources, hashes, executable identity, commands and durations;
import and verify its exact bytes in SQLite before removing the loose record.

These are frontend costs including process startup. They do not measure runtime
throughput, general recursive inference, arbitrary syntax nesting or native
backend compilation. TypeScript numbers and native integers differ semantically.
Registration alone is not measured acceptance.

`--recursive` selects `recursive-manifest.json`: an annotated cycle of the same
sizes forwards a value and counter, with the last function returning the value
at zero or starting another traversal. The entry counter is zero, so the independent
oracle remains `7`. This exercises recursive access analysis without measuring
runtime recursion. Its fixed protocol matches the acyclic workload, but its
implementation validation and measurements are recorded separately.

`--recursive --inferred` selects `inferred-recursive-manifest.json`. The first
function has the independent scalar return that seeds the native recursive
result; other functions forward it. The TypeScript counterpart retains explicit
result annotations because TypeScript does not infer these recursive results.
Control flow and the result oracle match; inference coverage does not.

The first scoped release-compiler run retains 132 durations in SQLite as
`tsrust-nesting-dependency-scaling.json`, with verified protocol and review in
`tsrust-nesting-dependency-scaling-proof.json`. All four counts and three phases
passed validation. Native check medians range from 3.147 ms at 32 functions to
27.436 ms at 2,048; emission ranges from 3.359 to 30.097 ms. Node/TypeScript check
medians range from 343.048 to 378.205 ms. Startup dominates the small sources.
There is no prior comparable scheduler baseline; these measurements do not prove
a speedup over S11 or completion of the broader nesting architecture.
