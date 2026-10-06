# Gate 126: borrowed optional string fallbacks

Status: focused implementation, native execution, editor, documentation and matched benchmark acceptance complete. Full regressions and timing remain deferred.

## Behavior

A borrowed optional string field can select a string literal fallback without cloning: `const label = item?.label ?? "none"`. This includes loop elements and indexed nullable record array elements. Null/undefined chooses the fallback; a present empty string remains empty. The literal has static text storage. The selected value remains a readonly borrow, and source loans remain lexical. Transfer and source mutation are rejected; explicit clone() provides independent ownership.

Fresh allocated fallback owners still require a separately named stable source before borrowing. This repair does not implicitly clone, allocate a selected owner, change general union member access or change bounds behavior. See [the syntax guide](../borrowed-optional-fallback.md).

## Implementation and evidence

The analyzer previously required every non-copy fallback joined with a borrowed view to be a stable named place. String literals now qualify for this selection because their static text can safely be borrowed. Other freshly allocated expressions keep the diagnostic. Native emission already supports borrowing the static literal and required no change.

- `target/editor/borrowed-optional-fallback-red.log`: the requested source failed with borrowed-view before the repair. The fixture's expected output comes directly from `tools/node-reference/run.mjs`.
- `target/editor/borrowed-optional-fallback-accepted.log`: both focused compiler tests pass. The native witness covers present, absent, empty and Unicode labels and indexed selection. Ownership tests cover rejected transfer, rejected source mutation, rejected fresh-owner fallback and explicit independent clone.
- `target/editor/borrowed-optional-fallback-compiler.log`: adjacent destructuring (7), optional_chaining (4) and nullish_coalescing (8) pass. Its initial loan test used an unused parameter, which correctly performed no transfer; the corrected test returns the parameter and passes in the focused supplement.
- `target/editor/borrowed-optional-fallback-editor.log`: the new editor test passes for checked string hover and member documentation. The syntax guide reuses the executable witness behavior. No new built-in is added.
- `target/editor/borrowed-optional-fallback-registration-current.log`: all 18 registration/report checks pass. The independent oracle uses direct text formulas, without optional selection or cloning. Guarded sources retain every result cell, and both runtime and allocation profile preparation enforce the contract.
- `target/editor/borrowed-optional-fallback-harness.log`: 330/330 checks pass.
- `target/editor/borrowed-optional-fallback-verify.log` and `benchmarks/runtime/results/2026-10-05T01-15-28-895Z-verify.json`: all five native/Node-after-tsc correctness cases pass untimed. The UTC artifact timestamp belongs to the October 4 local work session.
- `target/editor/borrowed-optional-fallback-build-debug.log` and `borrowed-optional-fallback-build-release.log`: debug and release builds pass; both compiler/LSP consumer directories are refreshed.

Exact snapshots are retained in `archive/borrowed-optional-fallback-start`. Original failed checks remain. SQLite measurement records are unchanged; no full compiler/LSP regression, timing or allocation profiling is claimed.

Destructuring rest ownership awaits the requested owner decision. Default values remain pending; borrowed optional selection is a prerequisite repaired here. Next gate is 127.
