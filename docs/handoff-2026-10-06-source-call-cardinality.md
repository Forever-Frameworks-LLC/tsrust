# Source call cardinality checkpoint

Read AGENTS.md, development workflow, both generic value plans and
plans/unused-generic-call-validation-plan.md first. This extends 5efcac9 after red b5158e1.
Full source validation, Gate173 and the broad objective remain open. Preserve the
four unrelated owner fixture edits and use explicit Git paths.

## Implementation

Closure parameters now retain declared callback cardinality instead of an unknown
contract. Required/optional/default/rest slots use the same source contracts as named
function callback parameters. The helper reads parameter metadata without evaluating
defaults or materializing generic bodies.

An iterative cardinality traversal tracks known argument positions and whether the
count is exact. Nested literal spreads preserve known counts. Unknown dynamic lengths
cannot prove missing arguments, but cannot hide a guaranteed excess already established
by fixed positions. Exact nested empty spreads are zero. This remains source validation,
not runtime evaluation or a TypeScript blanket restriction on dynamic spreads.

Diagnostics state expected counts and exact/lower-bound supplied counts, with source
call and related declaration locations. Wrong generic counts also name expected and
supplied type counts. No ownership rules or callable environment identity changed.

## Verification

All three new red controls pass: closure callback excess, guaranteed excess before a
dynamic spread and guaranteed excess around nested dynamic spreads. Positive unknown
length, optional/rest callback and exact empty-spread controls pass. Paired successful
fixture passes native std:testing and strict TypeScript/Node. Independent TypeScript
rejects all three invalid cases, though its spread diagnostic is TS2556 rather than
Tsrust's provable arity diagnosis; do not claim identical spread acceptance policies.

Bounded compiler batch passes 155 tests: 75 adapter tests, 17 generic value tests and
all 63 depth canaries. Whole editor passes 313 tests, including precise count text
and source ranges. Existing 116-observation matched pair and all five native/Node-after-tsc
cases pass untimed. Runtime support is unchanged; no artificial rejection benchmark,
whole compiler regression or performance timing was added.

Exact matched record benchmarks/runtime/results/2026-10-07T02-49-56-230Z-verify.json
and all failed/repaired tsrust-call-cardinality logs plus negative TypeScript source
are imported into benchmarks/records.sqlite, verified byte-for-byte and removed as
loose copies. SQLite quick_check passes. Five consumers are atomically refreshed and
SHA256-verified. No process is live. Keep the single warm target and CARGO_INCREMENTAL=0;
prune obsolete generated executables and Node/matched scratch before handoff.

## Remaining work and continuation prompt

Continue Tsrust from this checkpoint. Implement sound mutable alias/branch flow and
selected/field callable source contracts with red witnesses. Do not invent a single
origin for unknown targets or treat one branch's contract as universal. Preserve
lexical identity, annotation-defined monomorphic contracts and ownership rules.
Finish the physical source-validation plan and remaining alias/editor/borrowed-result
and generic-field audit before accepting full Gate173. Keep tests/Node/editor/support
ready, store exact evidence in SQLite and commit bounded accepted chunks. Preserve
four owner fixtures. No routine timing/full regression before the family milestone.
