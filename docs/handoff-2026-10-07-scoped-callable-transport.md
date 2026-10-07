# Scoped callable transport checkpoint

Read AGENTS.md, docs/development-workflow.md,
docs/conditional-callable-carriers-plan.md and
docs/polymorphic-callable-transport-plan.md before continuing. This checkpoint accepts
scoped traversal and nullable transport within Gate173. Full Gate173 remains open.
Accepted implementation HEAD is 70eed7c. The preceding scoped Once acceptance is ceb5dee. No owner question is pending.
Preserve all 15 currently modified owner fixtures. Use explicit Git paths.

## Current behavior and modules

Scoped monomorphic conditional families now pass reusable array traversals, reduction,
predicates/search, sort/toSorted, flatMap, Array.from and Set/Map forEach. Readonly
families use ReadCallback; mutable families use Callback. Defaults, supplied argument
counts, rest packs and failure payloads remain checked. Consuming callbacks are
rejected by reusable traversal. Receiver mutation conflicts remain errors.

callable_storage::Registry::callback_declaration resolves shared protocol metadata
without certifying ownership. The emitter explicitly selects a family's canonical
storage Result at traversal calls, avoiding ambiguity from additional callback error
bridges. Keep generic callback protocols and concrete family identities distinct.

Nullable families have generic native callable payloads and support typeof, optional
invocation and result coalescing. Readonly/mutable nullable parameter boundaries pass.
Return and throw checks inspect the nullable payload, preventing scoped escapes.
Capture loans block conflicting owner mutation while live and release at scope exit.
Existing Once retention inference remains intact; no new public wrapper is introduced.

## Acceptance evidence

Six std:testing project tests pass with formatting round trips. Their five positive
cases also pass after TypeScript compilation with Node assertions. The affected
compiler batch passes 55 tests; eight depth canaries pass 64 tests. Complete editor
verification passes 335 tests. The guide executes successfully. Matched adapters
retain 136 observations; five native/compiled-TypeScript cases, oracle, guards and
registration pass. No timing measurements run.

Original failures include array callback admission, contextual optional-call checking,
invalid native union lifetimes, typeof and nullable injection panics, and ambiguous
Result trait selection. A test incorrectly reusing a moved handle is repaired; transfer
rules remain enforced. Benchmark absence uses zero to retain the observer's existing
nonnegative domain. Exact evidence is preserved under
runtime/diagnostics/scoped-callable-transport/ in benchmarks/records.sqlite.

## Next chunks

Start general scoped aggregate storage with focused source/native lifetime witnesses.
Read docs/scoped-callable-aggregate-plan.md for concrete record, array and composition gates.
Do not map borrowed families to owned StoredCallable merely to fit record or array
layout. Native storage must carry actual environments and lifetimes. Cover reads,
mutation, moving, nullable fields, scope release, escapes and original-owner disposal.
Combining distinct carrier families through callable coalescing remains open.

Then continue polymorphic selection/replacement and point-sensitive mutable environment
restoration. Retained reds include
 generic_call_capture_effects::literal_guard_native_callable_replacement_has_common_carrier
and generic_call_validation::branch_flow::invocation_worklist::mutable_environment_sequence_tests::owned_factory_updates_are_point_sensitive_and_instances_stay_separate.
Revalidate them before implementation. Keep recursive/default/rest/failure/finally
convergence and checked lifetime boundaries in the full Gate173 audit.

Use CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target and CARGO_INCREMENTAL=0.
Pinned Node is /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. Run bounded
checks, all eight canaries after dispatch edits and the editor crate at acceptance.
Format the entire project before source checkpoints. Store verified closed evidence
in SQLite, never archive directories. Rebuild and atomically refresh consumers only
from accepted source. Preserve unmeasured workload status.

## Consumer refresh and next red witnesses

Debug compiler and LSP consumers, editor debug consumers, workspace release CLI and
installed CLI are rebuilt from 70eed7c and atomically refreshed. All six destination
hashes match their source executables. Build logs and exact consumer identities are
verified in SQLite under the same evidence prefix.

Two read-only probes establish independent record-storage blockers. An unannotated
local record reports that a named record type is required. A syntactically valid
named field declared ReadOnly<Function<int, [value: int]>> reports callback-escape:
borrowed functions currently belong in local bindings or parameters. The earlier
probe missing a field semicolon is syntax evidence only. Preserve these distinctions
when adding persistent failing tests; do not bypass owning contracts by relabeling
borrowed families. No aggregate implementation or acceptance is claimed.

Continue with docs/scoped-callable-aggregate-plan.md. Keep the three existing public
callback variants. Invocation access and escape lifetime are separate checks; unknown
retention remains conservative. Full Gate173 and the broader language plan remain open.

## Record work in progress

Two persistent source/native tests now specify named scoped readonly and mutable
record fields. Both fail with callback-escape at field declaration. Their matched
TypeScript source assertions pass. No compiler implementation has changed. The
aggregate plan identifies declaration resolution, construction specialization,
native generic layouts and aggregate escape traversal as separate responsibilities.
Run the red tests before continuing; this is a work-in-progress checkpoint, not
record acceptance. Evidence lives under runtime/diagnostics/scoped-callable-records/.

## Scoped field contract implementation checkpoint

scoped_record_fields now resolves owning versus scoped declaration contracts in a
dedicated module. Scoped contracts preserve invocation access and source parameter
labels, including nullable payloads, without certifying environment ownership.
record_layout uses this resolver but still rejects scoped field admission until
construction specialization and aggregate lifetime enforcement are integrated.
Existing storage_declaration guarantees remain unchanged.

The focused owning-field editor protocol check passes. Two module tests were red
before implementation and now pass. Three record-layout
checks and eight callable-resolution checks pass. The two source/native record
witnesses remain red; this is preparatory implementation, not feature acceptance.
Next implement concrete scoped record layout specialization without mutating shared
nominal declaration metadata, then aggregate loan/escape checks before admitting
construction. No additional public callback variant or owner decision is needed.
