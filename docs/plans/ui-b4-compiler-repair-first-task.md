# Parallel task: B4 named-handler compiler repair

Status: proposed assignment, 2026-10-15; not dispatched. Planning only. This document authorizes no implementation, worktree creation, build, consumer refresh or merge.

Read [AGENTS.md](../../AGENTS.md), [development workflow](../development-workflow.md), [compiler layout](../guides/compiler-source-layout.md), [TLS status](tls-acceptance.md), [UI lifetime contract](../../experiments/ui-tsrust-owner-handles/CONTRACT.md), [H matrix](../../experiments/ui-tsrust-owner-handles/matrix.json), [R matrix](../../experiments/ui-tsrust-owner-handles/resource-matrix.json), and both [named-handler reproducer](../../experiments/ui-tsrust-owner-handles/witnesses/named-self-replacement.tsr) and [factory control](../../experiments/ui-tsrust-owner-handles/witnesses/factory-self-replacement.tsr).

## Outcome and boundaries

Repair the compiler invariant behind B4. The exact named-handler source must check, emit, compile natively and satisfy its existing assertions, just as the factory control does. Preserve valid ownership diagnostics and checked callable contracts. Then convert the UI harness from expecting this panic to requiring successful execution and mark B4 resolved with evidence.

The merged UI source is at `d012b41a06b28da2fb07b10f0f26753ed589d2c2`; its reviewed branch tip was `a4e6fadb83f87da43c5e976446befb0df0cfbaa9`. The verified S15 compiler from `af0cbba8` has SHA-256 `57fd08aad9358fa367434bb5f5f036885b0148a7c0994939addea8f230aceca3`. Both check and emit panic on the formatted reproducer in `analysis/mod.rs`, at the `assumed_modes[index]` lookup in `completed_declared_call_contract`, with length 22/index 22. Line numbers are historical, not the repair specification.

The factory control executes fresh self-replacement, stale queued delivery, unaffected FIFO work, recursive-drain rejection and root cleanup. Simpler named-handler probes passed during review: do not assume every named function fails or attribute the defect to the UI patch. Root cause remains to be established.

Do not implement identity enforcement, resource observers, rendering, JSX, HTTP/TLS features or another UI milestone. B1 and B2 remain open. B3 remains resolved by the adopted immediate-retirement contract. Fixing B4 does not make the lifetime demo fully accepted.

## Investigation and repair

1. Reproduce the exact committed failure with verified tool provenance. Preserve original source and diagnostic in evidence. The historical compiler may have been replaced; never silently repin or rebuild an old installed consumer. If no matching old executable remains, retain the existing canonical failure evidence and reproduce against the selected baseline compiler once builds are permitted.
2. Minimize the trigger without replacing the original regression. Vary named versus factory handlers, use inside/outside a retained callback, declaration/import order and repeated registrations. A simplified case that passes is a control, not a replacement reproducer.
3. Trace function identity, signature/checked-table membership and recursive access-summary construction at the failing call. Determine whether function synthesis, admission, summary refresh or index remapping violates an invariant. Inspect all producers and consumers of the affected invariant before editing.
4. Repair that invariant at its owning layer. Do not catch the panic, clamp indexes, skip analysis, supply empty/copy-mode summaries, or reject valid source merely to avoid the crash. A bounds check is useful only as a diagnostic aid; it is not acceptance. Any fallback must preserve arity, ownership modes, hidden capture arguments and convergence, with evidence.
5. Add the smallest focused compiler regression and independent controls, then batch validation after the coherent repair. Preserve the full UI source witness unchanged in behavior. Do not patch emitted Rust, add implicit clones or weaken borrow/transfer checks.

Read-only starting points are `analysis/mod.rs` access-summary setup and `completed_declared_call_contract`, named-capture planning, checked callable storage, and existing callable/closure regression helpers. They are leads, not a mandate to edit all these areas. The observed lookup alone does not establish the root cause.

## Ownership and TLS isolation

TLS acceptance was still in validation when this plan was written. Its live status and working tree can change. Re-read both at dispatch and before validation/integration.

- One worker in a separate managed Git worktree. Start from a committed baseline containing the UI merge, preferably the latest stable committed TLS checkpoint available. Record its full commit and the exact task brief. Do not copy uncommitted main source into the worktree.
- Initial ownership: the minimal compiler analysis/callable-planning files required by the proven cause; a focused test in the matching existing compiler test target or a descriptively named new target/fixture; and `experiments/ui-tsrust-owner-handles/` for B4 conversion and evidence. Record the concrete touched-file list before source edits.
- Keep HTTP/TLS analysis, emission, runtime, native type/signature declarations, package/export support, manifests/locks, TLS fixtures, benchmarks and unrelated editor work outside ownership. A dependency reaching those files, shared interfaces or an area another worker owns requires explicit coordination before editing. Continue independent diagnosis while blocked; do not broaden ownership silently.
- Use the repository's existing single Cargo target, with `CARGO_INCREMENTAL=0`. Resolve its actual configured path at dispatch; do not invent a second target, create per-worktree caches, or download dependencies. Prefer offline builds using existing dependencies. Missing dependencies are an execution blocker, not permission to install or alter locks.
- Separate worktrees isolate source only. Acquire an explicit build window before any Cargo invocation or test process using the shared target. Confirm no TLS build/validation owns it. Cargo's lock alone does not protect benchmark timing or prevent another build replacing a binary between tests.
- Give TLS timing priority. Read-only investigation and source editing can run in parallel; defer compilation, native checks, formatting and other substantial loads during active TLS measurements. Require evidence the window is clear. Do not stop TLS processes or assume silence means approval.
- Keep the selected candidate executable unchanged throughout its validation window. Capture its source commit or exact source fingerprint, path, version and hash; verify the hash after checks. Record TLS's installed consumer hashes separately and leave them untouched. If a shared build replaces the candidate, preserve completed evidence and rerun only affected checks under a newly identified candidate.
- Do not replace `~/.cargo/bin` consumers, configured editor servers or shared release consumers. Use a candidate built in the existing target via explicit `TSRUST`/existing test-tool overrides after inspecting how each harness chooses its binary. No binary backups or snapshots. Consumer refresh and integration belong to a separately authorized serial step.
- No main-checkout edits, main SQLite writes, merge, push or PR publication. Avoid concurrent agents in the worktree. Preserve all unrelated changes.

If a build window cannot be established, finish diagnosis, proposed patch and tests that do not execute, then report validation pending. Do not mark the repair accepted or B4 resolved merely because shared-tool access is unavailable.

## Acceptance matrix

Each row has a disposition: **executed**, **analysis/emission only**, **expected rejection**, **blocked**, or **not applicable with reason**. Record exact source/tool identities and commands. Completion of the repair requires all mandatory success rows to pass; a blocker report must identify unfinished acceptance explicitly.

| ID | Case | Required result/evidence |
|---|---|---|
| C01 | Reproduction and provenance | Original B4 source/diagnostic preserved; baseline and candidate identified; TLS build/timing window established; installed consumers unchanged |
| C02 | Root-cause invariant | Explain which table/index/summary invariant failed, the producing path and why the fix restores it; cite code and reproducer rather than inferring from the panic location alone |
| C03 | Exact named-handler check | Formatted committed `named-self-replacement.tsr` passes strict project/module checking; no suppression or new arbitrary rejection |
| C04 | Exact named-handler emission/native run | Emit succeeds; unmodified generated Rust compiles and runs all existing `std:testing` assertions; expected trace matches the factory control |
| C05 | Factory control | Existing factory source still checks/emits/runs; no regression in fresh IDs, live order, stale delivery, FIFO, recursive-drain rejection or root cleanup |
| C06 | Minimal compiler regression | Small regression exercises the proven cause in the correct compiler test layer; it fails on the old behavior where reproducible and passes with the fix; original integration witness remains |
| C07 | Named-handler variations | Cover the minimal triggering arrangement and selected passing controls: inside/outside retained callback, repeated registration and a declaration/import-order variation; valid forms stay valid or retain established diagnostics |
| C08 | Access-summary correctness | Relevant read/mutate/consume and capture contracts survive the repair; summaries are not missing, shortened or defaulted to weaker modes; verify recursive/fixed-point convergence if that machinery is affected |
| C09 | Strict ownership rejections | Escaped loan, overlapping mutable access, direct registry overlap and use after transfer still fail strict check/emit with expected ownership diagnostics, not panics; retain the existing UI controls |
| C10 | Affected callable regressions | Run the focused compiler target(s) covering the repaired path and relevant existing callable/capture/recursive cases; selected targets and exclusions are justified from the actual diff |
| C11 | Editor agreement | If shared analysis affects the LSP, verify the valid named-handler case produces no internal error and invalid ownership controls remain diagnosed using existing editor test support; no unrelated editor refactor |
| C12 | Formatting and assertion sensitivity | Whole-project tokens preserved, formatting idempotent and formatted successful source executed; deliberately wrong expected state/trace fails natively |
| C13 | UI harness migration | Remove the B4 expected-panic helper/path; both named and factory witnesses now require successful native execution. Unexpected panic, rejection, timeout or signal fails the suite/demo |
| C14 | Matrix and historical reconciliation | B4 marked resolved by the identified repair; H13/R08 evidence updated and H/R reconciliation agrees. Preserve historical panic records; B1/B2 remain open and B3 remains adopted |
| C15 | Complete focused UI checks/demo | Existing focused suite passes with no silent skips. Demo executes both witnesses and still exits 2 solely for remaining B1/B2 blockers; no claim of full lifetime acceptance |
| C16 | Integration isolation and handoff | Only agreed files committed, provenance/evidence restorable, candidate/consumer identities verified, no TLS edits or consumer replacement; explain any validation still requiring serial integration |

C11 is not waived merely because no LSP file changed: the LSP uses shared compiler analysis. Use the development workflow's applicable editor agreement checks when that contract is affected. A new public built-in is not expected; if one seems necessary, stop and reassess scope rather than adding a catalog feature here.

## Validation sequence and commands

Before editing, inspect existing test support and Cargo configuration to resolve the CLI override and single target path. Do not guess test target names or let tests silently invoke the installed old compiler. Record expanded commands in evidence.

1. Reproduce the original case and add focused regression/control cases. An early check is justified to identify the cause; avoid repeated whole-suite runs while editing.
2. Complete the repair and UI test/report migration as one section. Run whole-project formatting before source checkpoints:

   ```sh
   TSRUST=<verified-formatter> node tools/format-project.mjs
   ```

   Prove token preservation, second-pass idempotence and formatted-source behavior. Use the verified installed compiler as formatter if it handles all sources; its known analysis panic does not itself invalidate its formatter. Report outside-ownership formatting requirements as checkpoint conflicts; do not commit unrelated changes.
3. During the reserved shared-target window, build only the required candidate compiler/test targets, with `CARGO_INCREMENTAL=0` and offline dependencies. Record source identity before the build. Run the smallest complete set of affected compiler/editor checks, then the original UI sources through strict check/emit/direct rustc execution.
4. Update the experiment's compiler pin only after the candidate's provenance is established. Preserve the S15 identity and original panic evidence. Run:

   ```sh
   TSRUST=<verified-candidate> node --test experiments/ui-tsrust-owner-handles/tests/*.test.mjs
   TSRUST=<verified-candidate> node experiments/ui-tsrust-owner-handles/cli.mjs demo
   ```

   Expected outcomes: focused tests exit 0, demo exits 2 for B1/B2, both B4 sources execute successfully. Direct candidate check/emit must also pass. No test should count the historical panic as a successful expected rejection after repair.
5. Repair failures and rerun affected checks only. This bounded repair does not automatically require full compiler/LSP milestones or performance benchmarks. If the cause changes a broad invariant and focused coverage is insufficient, name the additional required checks and schedule them outside TLS measurements; do not claim acceptance prematurely.

## Evidence, commit and stop

Keep a concise B4 repair report and C01–C16 matrix inside the owned experiment. Record root cause, changed invariant, actual file ownership, exact commands/results, candidate source/hash, original failure, minimized controls, ownership negatives and applicable editor evidence. Do not preselect a technical fix in the report.

Use an ignored task-local SQLite with the existing storage tooling for closed evidence. Verify byte-exact restoration, hashes and integrity before removing loose copies. No archives, source snapshots or retained obsolete native executables. Prune only task-created disposable outputs; do not prune TLS's shared target or active consumers. Main evidence integration remains a separate owner-authorized serial action.

Use Git source checkpoints before risky edits and after accepted gates, following whole-project formatting requirements. Keep validated source identity tied to the candidate binary even if the final handoff commit also contains evidence metadata. Do not call a pre-validation source checkpoint an accepted repair.

Hand off worktree path, branch/commit, root-cause explanation, exact acceptance results, source/tool provenance, isolated evidence location and any serial integration requirements. Include remaining B1/B2 status and the unchanged B3 contract. If the repair cannot be completed within scope, report the precise blocker and unfinished rows rather than substituting panic suppression.

Stop after this repair and handoff. Do not merge, refresh installed consumers, implement resource observation or identity enforcement, or start another UI/compiler milestone.
