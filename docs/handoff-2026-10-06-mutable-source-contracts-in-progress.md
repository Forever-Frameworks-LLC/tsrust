# Mutable source contracts: work in progress

HEAD is a1a8bab, which preserves the four original red witnesses. The current
implementation is uncommitted and has not received gate acceptance.

The source validator now collects candidate callable contracts for mutable aliases
and selected expressions. It rejects argument or generic counts only when every
known candidate has the same cardinality. Diagnostics retain alternative declaration
locations. This is a conservative common-contract proof, not ordered assignment
flow; differing or unknown candidates defer validation. Full Gate173 remains open.

A mechanical typo in the write index integration caused a compiler build failure.
The typo was repaired. The direct selected-call red now passes. The bounded compiler
run passes 78 generic adapter tests and 17 generic function value tests. The new
editor diagnostic test passes and verifies both declaration locations. No whole
editor run, depth canaries, consumer refresh, matched runtime verification, or gate
acceptance has occurred in this continuation.

Current implementation files are generic_call_validation.rs and the new
 generic_call_targets.rs under crates/tsrust-compiler/src. Tests changed in
crates/tsrust-compiler/tests/generic_value_adapters.rs and
crates/tsrust-lsp/tests/intelligence.rs. Preserve all eight dirty fixture files;
none belongs to this implementation. Inspect Git status before staging explicit
paths.

Active evidence logs use /private/tmp/tsrust-mutable-alias-*.log. Import closed logs
into benchmarks/records.sqlite, verify byte-exact readback, then remove loose copies.
The direct-repaired.log contains the build failure; direct-repaired-2.log contains
the passing repair. Do not overwrite or discard failed evidence.

Next: test valid loop, closure-write and shadowing cases; audit candidate joins for
false rejection and missing write forms. Implement precise assignment flow and
stored field contracts rather than treating the conservative join as completion.
Then finish paired source witnesses, whole editor checks, applicable depth canaries,
documentation, matched untimed verification, SQLite evidence and explicit-path
checkpoint. Follow docs/unused-generic-call-validation-plan.md and the development
workflow. Reuse the warm target with incremental compilation disabled.

## Straight-line ordering continuation

A new red witness showed that a later assignment to a different callable shape
hid an invalid earlier call. Source ordering now excludes later writes only for
straight-line bodies with direct statement assignments and no closures or nested
write sources. Complex flow retains the conservative whole-function candidates.
This avoids treating loop iterations or hoisted capture writes as source ordered.
The original red and three loop/closure/expression-write controls pass, as does a
shadowing control. The final guarded focused run passes four tests. Full ordered
flow, definite overwrites, branch joins and stored fields remain incomplete.

Additional evidence is flow-red.log, flow-repaired.log (a borrow-check build
failure), flow-repaired-2.log and flow-guarded.log under the same temporary prefix.
No acceptance or consumer refresh is implied by these focused results.
