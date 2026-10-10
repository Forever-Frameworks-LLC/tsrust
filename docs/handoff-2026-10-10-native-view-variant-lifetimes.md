# Variant-distinct native view lifetimes

Continue from `7103fcc4`. Read AGENTS.md, docs/development-workflow.md,
docs/native-byte-views-implementation.md and the Result capture handoff first.
Gate 189 and S07 remain open; public borrowed signatures remain guarded.

The new red used a captured `Result<ByteView, ByteView>` with independent variant
lifetimes. Both payload paths have the same field spelling. Path-only lifetime keys
merged them, producing a wrong capture return lifetime and preventing a selected
variant from surviving the unused variant's shorter lifetime.

LifetimeSource now distinguishes guarded carried sources through VariantCarried;
unguarded inputs retain Carried. VariantGuard is hashable so the source identity can
remain stable through return summaries. Original source guards belong to the source
identity; current output guards still describe where the returned value lives.
Do not collapse these two concerns when projecting or wrapping values.

Parameter seeding preserves source variant guards. Call transport filters
contradictory variants before payload projection. Native input/capture slots and
output grouping distinguish path plus variant identity, and signature rendering
tracks the selected nominal variant while descending union payloads.

The formatted-source witness asserts exactly two distinct returned sources and
executes both variants at O0/O3. Each selected view keeps its original pointer and
can survive the unused variant's shorter lifetime. Independent negative Rust
compilations reject either selected view escaping its own byte owner. Existing
capture, Result, mutable write-through and ordinary function controls remain in the
combined native compilation.

181 focused checks passed: 49 private view tests, nine registered signature controls,
18 Result tests, 89 depth canaries across all ten suites and 16 native editor tests.
No failed or aborted suites. Warm CLI build was warning-free. Whole-project
formatting changed zero files and fingerprints. Original red, repair and check logs
are retained in SQLite under
`runtime/results/native-view-variant-lifetimes-2026-10-10.json`, after byte-exact
readback. No full regression or timing. Installed consumers remain f569190a.

The current acceptance map now includes consuming view/record captures, captured
Results and read-only Result observers. Remaining requirements include borrowing
receiver returns, captured owner storage, escaped/rebound environments, specialized
callbacks and stored families, recursive layouts, collection transport and exceptional
completion reconciliation. Factories/editor/benchmark acceptance remains pending.
These checks do not establish every nested variant/container composition.

## Continuation prompt

Continue gate 189 from this handoff. Finish borrowing receiver-return contracts and
captured owner storage next, with source/native red witnesses and explicit owner
escape controls. Then complete escaped/rebound environments, specialized callback
and storage families, and the remaining implementation-plan matrix. Preserve actual
slices and explicit ownership; retain guarded public factories until the full matrix
agrees. Complete editor/docs and matched benchmark support before gate acceptance.
Full regressions and timing belong to S07 completion.
