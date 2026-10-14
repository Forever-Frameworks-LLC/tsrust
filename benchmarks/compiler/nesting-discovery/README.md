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


Use `--unbounded-parser` to observe the downstream pipeline beyond the inherited
parser admission bound. This opt-in test mode still runs module-state lowering
and local-function resolution, then analysis, emission and disposal. It does not
change public compiler behavior. The selected parser policy is recorded in the
JSON record and probe events. Parser-only discovery in `parser.py` is different:
its core stage intentionally skips these lowering consumers.

The `assertions`, `satisfies`, `rest-arguments`, `array-of` and `mixed-wrappers`
families generate source in linear time. Array.of/index layers keep array rank
fixed while increasing expression depth. Use the existing geometric progression
and operational process budgets; last passing sizes are not supported ceilings.

`scoped-map-branches` nests joins that renew callback-bearing Map storage;
`scoped-map-loops` nests loops that delete one callback entry and retain another.
Both construct source in linear time and exercise ownership analysis before
storage-aware emission. Their phase events distinguish checker limitations from
emitter failures; isolated checked-tree witnesses do not establish pipeline
acceptance for these families.
