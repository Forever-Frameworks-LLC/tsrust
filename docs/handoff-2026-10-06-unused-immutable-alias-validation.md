# Unused immutable alias validation checkpoint

Read AGENTS.md, workflow, both generic value plans and
plans/unused-generic-call-validation-plan.md first. This implements the initial known
source-contract portion after red 0a5fd96. The complete validation plan and full
Gate173 remain open. Preserve the four owner fixture edits and stage explicit paths.

## Implementation and behavior

A dedicated generic_call_validation module runs before declarations are drained
into specialization. Original callable contracts contain type-parameter count,
required/default/optional/rest slots and source declaration spans. It checks unused
generic function bodies without instantiation, runtime execution or default effects.

Lexical binding ranges reuse the existing iterative locals traversal. Immutable alias
chains resolve at their initializer location, with narrower scopes shadowing outer
bindings. Known callback annotations are monomorphic. Synthetic forwarding wrappers
retain trusted same-span source targets; ordinary arrow aliases retain concrete
contracts. Function/closure parameters and block/catch/loop names shadow globals.
Callback parameters of named functions retain their declared cardinality.

Explicit type counts and known static value counts are checked. Literal array spreads
have static cardinality; unknown/dynamic spreads remain deferred. Unknown mutable
aliases are not assigned fabricated targets. This conservative partial implementation
must not be described as complete source validation. Remaining flow joins, closure
callback parameter contracts, selected/field targets and wider callable context
requirements need separate red controls and implementation.

## Verified evidence

Three original red cases pass: wrong generic count, explicit application on a
monomorphic alias and excess values through an immutable generic alias. Fresh callback
parameter and arrow alias reds pass. Positive alias chains, independent block scopes,
default/rest, static/dynamic spread, forward references and unused recursive aliases
pass. The paired fixture checks std:testing/native and strict TypeScript/Node.
Borrowed-result controls reject ReadOnly consumption with parameter-access.

Final bounded compiler batch passes 152 tests: 72 adapters, 17 generic value tests and
all eight depth suites (63 tests). The added paired fixture passes separately,
covering 73 adapter tests across checks. Expanded forward/self controls pass afterward.
Whole editor passes 312 tests; the new diagnostic checks exact type-argument and
related original-declaration ranges. Existing 116-observation matched workload and
all five native/Node-after-tsc cases pass untimed. No whole compiler regression or
performance timing was run; no artificial workload was added for static rejection.

Exact matched record benchmarks/runtime/results/2026-10-07T02-44-10-712Z-verify.json
and all failed/repaired tsrust-unused-alias logs are imported in benchmarks/records.sqlite,
verified byte-for-byte and removed as loose copies. SQLite quick_check passes. Earlier
red TypeScript diagnostic/source evidence remains stored. Five consumers are refreshed
atomically and SHA256-verified. No process is live. Keep the single warm target and
CARGO_INCREMENTAL=0. Prune obsolete test/native and Node scratch outputs before handoff.

## Continuation prompt

Continue Tsrust from this checkpoint. Read AGENTS.md, workflow and the physical unused
call-validation plan. Initial immutable alias/source contracts are implemented; full
validation and Gate173 remain open. Add red controls for mutable/branch alias flow,
closure callback parameters, selected/field contracts, captured wrappers and guaranteed
invalid dynamic-spread prefixes. Repair soundly without executing unused bodies,
weakening generic constraints or introducing implicit clones. Finish dedicated editor,
Node/reference and bounded acceptance support, preserve SQLite evidence, then commit.
Do not repeat timing or a full regression before the meaningful family milestone.
