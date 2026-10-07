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

## Verified source checkpoint after 3051c9e

The separate generic_call_linear_flow module now snapshots initializer contracts
and replaces targets after completed direct assignments. Both overwrite and saved
snapshot reds pass. Ordered flow is enabled only for straight-line bodies with
ordinary direct assignment statements and no closures, nested or expression writes.
The conservative complex-flow path remains; do not describe full flow as complete.

Bounded verification passes 82 adapter tests, 17 generic value tests and all 63 depth
canaries (162 compiler tests). The paired fixture additionally passes native
std:testing and strict TypeScript/Node. The native control uses self-assignment
followed by owned transfer, not reuse after transfer. A separate unused template
retains distinct-target source-contract coverage. Concrete replacement between
different callable environments still raises callable-origin; repairing compatible
environment transport remains required runtime work. No ownership rule changed.

Whole editor passes 315 tests. A formatting-sensitive Array.from documentation
check now canonicalizes formatting before comparing snippets; exact catalog output
and source compilation checks remain. None of the owner's fixtures was modified.
Ten unrelated fixture edits are present now, including array-parameter-pattern.tsr
and sort-search.tsr under generic-value-adapters. Preserve all unrelated dirty files
and use explicit Git paths.

The existing 116-observation matched family passes all five native/Node cases
untimed. Exact record benchmarks/runtime/results/2026-10-07T03-11-48-332Z-verify.json
and 27 closed failed/repaired logs are stored in benchmarks/records.sqlite with
byte-exact readback and passing quick_check; loose copies removed. Five executable
consumers were atomically refreshed and hash checked. Node and matched scratch were
removed, along with obsolete test executables. No timing or full compiler regression
was run. No process remains live.

### Continuation prompt

Continue Tsrust from this source checkpoint. Read AGENTS.md, development workflow,
unused-generic-call-validation-plan.md and both generic value plans. Implement the
next structured-flow checkpoint: lexical write identity, definite block assignments,
branch-local calls and both-arm replacement, loop fixed points, exceptional exits
and captured write effects. Preserve snapshot semantics and declaration provenance.
Then complete stored-field validation, compatible concrete environment replacement,
lifetime/context auditing and full Gate173 acceptance. These remain requirements.
Use TDD, native ownership/disposal assertions, strict Node references, matched support,
bounded checks, whole editor and durable SQLite evidence. Preserve unrelated fixture
edits. Keep the goal active; this checkpoint does not finish the full plan.
