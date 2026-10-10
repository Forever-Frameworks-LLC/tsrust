# B10 Result and match continuation

B10 is in progress, not accepted. Read AGENTS.md, development-workflow.md,
result-match-plan.md and result-match.md before continuing. One agent works in
this tree. No owner changes were present at this checkpoint.

## Completed increments

Checkpoint `1abc2696` integrated contextual Result construction and exhaustive
statement match. Checkpoint `20eac8b8` added explicit constructors and concrete
signature help. This checkpoint adds single-file pattern-binding assistance.

`Result<T, E>` uses two private nominal wrappers in the existing tagged-union
arena. Each wrapper stores only its active payload. Both arguments participate
in identity. A void payload has no field or argument. Constructors use expected
context, or explicit `Ok<T, E>` / `Err<T, E>`. User declarations and lexical
shadows win over the prelude. Generic substitutions use existing checked
specialization. No unchecked cast or implicit clone was added.

Match syntax lowers to ordinary blocks, a single owned scrutinee temporary and
branches. Trusted internal operations validate coverage, test a variant and
project its whole private record. Payload extraction first moves that record
to a hidden temporary, respecting existing owned-field rules. Coverage proves
the final unconditional arm. Owned union reads require ViewUnion in native
variant tests. Do not replace that with an owned Union test.

Editor Result specifics live in result_intelligence.rs. Explicit constructor
help parses structured type arguments; it does not split nested types on commas.
Match syntax supplies declaration spans, and checked destructuring facts supply
payload types. Unknown facts remain unknown. Arm arrows are not function arrows.

## Evidence

Result integration: nine passing tests, including formatted native witnesses,
explicit generic constructors, shadowing and source diagnostics. Earlier parser
increment: five syntax tests and ten depth canaries passed. Latest editor batch:
six tests passed, covering bindings, constructor help, contextual arrows and
destructuring. Three catalog examples pass idempotent formatting and exact native
output. Complete command records are in SQLite under result-match-prefixed paths
documented in the plan. Exact readback passed and loose copies were removed.

No full regression or performance run was performed for B10. Installed consumers
remain at B9 until family acceptance. Do not call these source checkpoints accepted
gates or claim workloads were measured.

## Next work

1. Integrate callable payload identity correctly. Result materialization currently
   runs in generic_records before analyzer callable Tables exist. Callable payload
   annotations therefore fail in result_types.rs. Do not guess callable IDs,
   erase ownership or use display spelling as checked identity. Resolve the
   phase dependency with focused failing witnesses first.
2. Implement and verify borrowed matching under existing read/mutate rules,
   including escape rejection, moved Results, replacement, early return and
   cleanup. Current matching transfers an owned scrutinee.
3. Verify project-aware binding facts and navigation, nested pattern scopes,
   incomplete editing and callback/drop stress. Single-file assistance now works.
4. Add matched TypeScript tagged-union workloads, independent oracles, source
   contracts, driver hooks, hashes and untimed checks during implementation.
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

Continue B10 from this handoff. Finish the remaining Result/match family using
focused TDD and checked ownership identities. Read AGENTS.md and the workflow
first. Keep single-file editor support and explicit constructor shadowing intact.
Do not accept B10 until callable payloads, borrowed matching, project assistance,
cleanup witnesses, benchmarks and family evidence are complete.
