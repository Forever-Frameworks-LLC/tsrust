# B10 Result and match continuation

B10 is in progress, not accepted. Read AGENTS.md, development-workflow.md,
result-match-plan.md and result-match.md before continuing. One agent works in
this tree. No owner changes were present at this checkpoint.

## Completed increments

Checkpoint `1abc2696` integrated contextual Result construction and exhaustive
statement match. Checkpoint `20eac8b8` added explicit constructors and concrete
signature help. Checkpoint `3ef7515d` added single-file pattern-binding assistance;
`d5d70c6c` integrated owned callable payloads. This checkpoint adds readonly matching,
match formatting and the first matched Result workload.

Checkpoint `9966563a` saves that readonly increment. A subsequent editor repair
renders inferred and nested Result types through their public names, including
`Ok(payload)` and `Err(error)` in narrowing notes, instead of private record names.
The focused `intelligence` filter `result_` passes 36 tests after this repair.
Project formatting changes no files or fingerprints. This is presentation only;
mutable matching was still pending at that checkpoint.

The current increment implements exclusive matching on Mutable Result parameters
and nested mutable payloads. Hidden match holders use a distinct compiler-only
name. Checked bindings retain both a scoped storage borrow and its exclusive mode;
native local emission borrows with `&mut` and preserves the union parameter's
existing borrowed-payload representation. Scalars still copy. Pattern bindings
remain non-reassignable; mutation of their record fields and array contents changes
the owner. Reusable owned callable payloads retain their state; Once calls through
the borrow are rejected. An active alias blocks direct access to its owner, and
borrowed payloads cannot escape directly or through owning closures.

The active exclusive alias list is scoped and empty for ordinary functions, so
readonly access does not scan all bindings. Capture context projection preserves
exclusive provenance. A native closure signature defect found by the arrow witness
is repaired: mutable union parameters now receive a mutable local parameter slot,
consistent with named functions and specialized methods.

Current evidence: sixteen Result tests, six syntax tests, ten depth canaries,
37 affected editor tests, five adjacent capture-context tests and six executable
catalog examples pass. The expanded matched workload passes five cases on native
and Node-after-tsc, without timing. Exact records are retained in SQLite under
`runtime/results/result-match-mutable-2026-10-10-*.json` and
`runtime/results/2026-10-10T12-17-07-686Z-verify.json`. Original checks precede
the final callable witness and capture-loan correction; the final Result record
supplements those checks. B10 remains open and installed consumers remain at B9.

The project assistance increment is also complete. ProjectIndex refreshes match
bindings after checked module facts arrive. Compiler project metadata recognizes
Result through a shared result_identity helper and presents public Result/variant
types while retaining each payload's nominal source identity. This fixes inferred
Result hover and imported aliases such as User imported as Person. Single-file
presentation and match analysis use the same identity helper.

An unfinished match with a complete pattern header recovers its binding's lexical
scope and navigation. It does not guess payload types from source spelling or stale
facts. Nested and sibling arms keep their bindings separate, including completion
after the match. The affected batch passes sixteen Result tests, two project metadata
tests, 62 project editor tests and 37 single-file editor tests. An extended completion
scope witness passes separately. Exact readback of
`runtime/results/result-match-project-2026-10-10-checks.json` passed in SQLite;
the loose copy is removed. Project formatting changes zero files/fingerprints.

`Result<T, E>` uses two private nominal wrappers in the existing tagged-union
arena. Each wrapper stores only its active payload. Both arguments participate
in identity. A void payload has no field or argument. Constructors use expected
context, or explicit `Ok<T, E>` / `Err<T, E>`. User declarations and lexical
shadows win over the prelude. Generic substitutions use existing checked
specialization. No unchecked cast or implicit clone was added.

Match syntax lowers to ordinary blocks, a single owned scrutinee temporary and
branches. Trusted internal operations validate coverage, test a variant and
project its whole private record. Owned extraction first moves that record
to a hidden temporary; readonly extraction lends it and its payload under existing
borrowed-place rules. Coverage proves
the final unconditional arm. Owned union reads require ViewUnion in native
variant tests. Do not replace that with an owned Union test.

Editor Result specifics live in result_intelligence.rs. Explicit constructor
help parses structured type arguments; it does not split nested types on commas.
Match syntax supplies declaration spans, and checked destructuring facts supply
payload types. Unknown facts remain unknown. Arm arrows are not function arrows.

## Evidence

Result integration: fourteen passing tests, including formatted native witnesses,
explicit generic constructors, callable storage, shadowing and source diagnostics.
The readonly compiler increment passes ten depth canaries. Formatter checks pass
six match syntax tests, 27 formatting tests and the lexable fixture corpus.
Four focused match editor tests pass, including readonly payload assistance.
Five catalog examples pass idempotent formatting and exact native output. The
independent Node callable reference passes. Complete command records are in SQLite under result-match-prefixed paths
documented in the plan. Exact readback passed and loose copies were removed.

No full regression or performance run was performed for B10. Installed consumers
remain at B9 until family acceptance. Do not call these source checkpoints accepted
gates or claim workloads were measured.

The new `result_match` workload passes untimed native and Node-after-tsc verification
on five cases each. Its independent arithmetic oracle, source guards, ABI registration,
runner/profile hooks and report category are complete. The manifest row is appended;
it remains unmeasured and not implementation-accepted. Formatter fingerprints are
updated only after token-preservation proof. Both original verification records are
retained in SQLite. Temporary workload build outputs are removed after import.

## Next work

1. Owned callable payload integration is now implemented. The analyzer creates
   callable Tables before initial materialization and lends the same registry to
   all retained materialization. Result keys use canonical storage types. Explicit
   generic wrappers check their arguments through the callable resolver. Abstract
   template storage checks contracts; only concrete specializations register
   environments. Native witnesses cover captured state, Once, forwarding and both
   variants. Keep context-free inline callback inference as an explicit limitation;
   callers can supply the callable type argument. Continue with the following work.
2. Readonly and mutable matching now work with checked scoped borrows. Extend
   replacement and cleanup witnesses, including early return and failures. Preserve
   scalar copying, exclusive owner conflicts and no implicit ownership transfer.
3. Project-aware binding facts, navigation, nested scopes and incomplete binding
   assistance now work. Complete callback/drop stress, including failure paths.
4. Extend the registered matched Result workload for remaining forms as they land.
   Retain its independent oracle, source contracts, hooks and untimed checks.
5. Finish the family milestone: full compiler/LSP checks with explicit abort
   detection, required matched measurements and review, acceptance records,
   roadmap, SQLite import/readback, checkpoint and five-consumer refresh.

Match expressions, guards, nested destructuring patterns, `?` shorthand and D2
general discriminated unions remain separate work. User chose Rust-style match
and expects its foundation to be reusable for D2.

## Environment

Use Node 24.11.1 first on PATH, `/opt/homebrew/bin/python3`,
`CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target` and
`CARGO_INCREMENTAL=0`. Keep normal worker stacks. Run the project formatter with
the verified warm-target compiler before source checkpoints. Use explicit Git
paths. Preserve completed evidence and rerun only affected checks.

## Continuation prompt

Continue B10 from this handoff. Owned callable payloads are implemented; finish
the remaining Result/match family using
focused TDD and checked ownership identities. Read AGENTS.md and the workflow
first. Keep single-file editor support and explicit constructor shadowing intact.
Do not accept B10 until cleanup witnesses, benchmarks and family evidence are complete.
