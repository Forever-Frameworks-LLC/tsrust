# Nullable and heterogeneous generic factory context audit

Read AGENTS.md, development workflow and generic-value-adapters-plan.md. Accepted HEAD is 863bb0d. Four owner fixtures remain untouched. Current uncommitted changes add nullable-factory and heterogeneous-factories native witnesses and strict TypeScript counterparts, two tests and a StoredCallable case in fresh staging.

Nullable contexts supply an owning stored family as their payload. stage_generic_expression now obtains that family's declared callback signature instead of rejecting it. Nullable fixture passes native execution with output 1,2, retained state and absent optional invocation. Its assertions explicitly narrow observed nullable values before strictEqual. The earlier assertion form exposed a separate generic strictEqual constraint problem between nullable actual and nonnullable expected; retain that defect for the broader inference audit rather than claim it repaired.

Heterogeneous selected factories remain red. One branch returns T and another T[]; both must acquire Function<int | int[], [value:int]>. Current solver constrains T from the contextual result too early and rejects the actual integer argument. Both strict TypeScript counterparts compile and execute successfully with outputs 1,2 and 1,1. Initial TypeScript reference had a known-undefined optional-call narrowing error; corrected fixture obtains absence through a declared nullable function result. Original failures and final references are stored exactly in SQLite; loose files and Node output are removed.

No acceptance yet. Next fix heterogeneous inference without accepting incompatible argument/result relations. Then nullable editor/benchmark coverage, bounded compiler/depth/whole-editor checks, records and commit. Existing five consumers and matched 88-observation family remain at accepted 863bb0d; no refresh, expanded benchmark or timing was run. All processes are terminal.

Continuation prompt: Continue Tsrust from docs/handoff-2026-10-06-generic-nullable-context-progress.md. Preserve owner fixtures. Fix retained heterogeneous result witness and generic nullable constraint defect; finish nullable context acceptance with editor, matched support and SQLite evidence. Full Gate173 remains open.

## Result context implementation progress

Heterogeneous selected factories now pass native execution. Generic inference establishes input constraints before considering contextual results, then accepts an already concrete result when the contextual union admits it. Adapter wrappers expose the requested result contract after validating the materialized source result. Existing negative control exposed an intermediate regression: comparing two declared callable result contracts omitted invocation kind. callable_compatibility now checks those kinds with can_pass_to before signature comparison. Single-use results remain rejected by reusable contexts.

A new retained nullable generic constraint witness passes: strictEqual on pop() and its known integer payload. A variable already inferred as a union admits existing member arguments, without inventing a wider union for incompatible inputs. All 37 adapter tests pass on final source. The bounded generic batch ran default and generic-function-value controls before failing on the intermediate invocation regression; finish the affected generic batch on final source, plus depth/editor checks. Original failures and repairs are preserved in SQLite. All processes are terminal. Changes are uncommitted and unaccepted; consumers and benchmark family remain at prior accepted source.

Next: audit inference ordering with zero-input/result-only and inconsistent constraints; finish generic controls, nullable/heterogeneous editor and matched coverage, records, refresh and acceptance commit. Gate173 remains open.

## Inference and editor controls

The final repaired bounded generic batch passes: generic defaults (builtins, void and scope), generic function values (17 tests), and generic value signatures. The two value_contract_tests library controls also pass, including zero-input result-only inference and inconsistent argument rejection. New editor protocol tests verify nullable results retain undefined and factory navigation, and heterogeneous results retain int | int[] in hover and signature help. Both pass. Exact evidence is imported into SQLite with readback; all processes are terminal.

Matched benchmark expansion, depth canaries and whole-editor checks remain outstanding for current source. The two editor tests are uncommitted. Do not treat existing 305-editor/88-observation evidence as current-source acceptance. No new consumer refresh or timing was run. Continue matched nullable/heterogeneous scenarios before final records and commit.
