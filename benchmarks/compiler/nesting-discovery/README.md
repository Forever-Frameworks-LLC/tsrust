# Pipeline nesting discovery

Build the compiler unit-test executable with the shared Cargo target and
`CARGO_INCREMENTAL=0 cargo test -p tsrust-compiler --lib --no-run`. Pass the exact
executable path printed by Cargo to `run.py --probe PATH --output NEW_JSON
--seconds-per-case SECONDS --seconds-total SECONDS`.

The driver grows each input geometrically until it encounters a diagnostic,
process failure or explicit operational budget. Budgets bound this diagnostic
run; they do not set language limits. The probe runs on the ordinary Rust test
worker stack without `RUST_MIN_STACK`. It records parse, analysis, recursive
replay, emission and disposal events. A diagnostic returns normally from the test;
only the disposal-finish event establishes full pipeline completion. Peak RSS is
for each fresh probe process, including the test harness. It is not per-phase RSS.

The initial families cover grouping, blocks, binary expressions, type groups,
array literals, explicit/inferred generic chains, alternating call/closure/body nesting,
and nested arguments through declared functions, local bindings, inline callees
and stored callable fields, plus mixed grouping/record/array/call/index expressions. This is not yet complete
mixed-shape, project, editor, native-backend or resource-policy acceptance. Exact
sources, compiler-test binary hash, machine memory, stage events and original
output are retained in the JSON. Import closed records into SQLite and verify
byte-exact restoration. Existing formal benchmark protocols are unaffected.
