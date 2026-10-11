# Dev-loop performance milestone

Status: required, next implementation priority. Owner priority set 2026-10-11.
Complete this milestone before adding more S09 APIs or starting application workflows.
S09's completed evidence stays accepted; monotonic time resumes afterward. This is
part of integrated release acceptance, not an optional optimization backlog.

## Existing acceptance

Quiet default output, verbose phase timings, persistent validated native artifacts,
private executable copies, shorter supervisor waits and reuse of unchanged source
bytes are implemented. Last-good-program retention on failed builds is required.
These improvements do not constitute incremental compilation: the watcher still
scans directories and metadata, each build starts frontend checking afresh, and a
fresh source version compiles the whole generated native program.

## Required implementation order and exit evidence

1. **Filesystem events.** Replace routine full-tree polling with event-driven source
   and config watching. Handle atomic saves, renames, additions, deletions, new
   directories, rapid edit bursts and edits during a build. Recover from dropped or
   coalesced events through reconciliation; provide a tested, explicit fallback when
   native watching is unavailable. Preserve exclusions and symlink policy. Prove
   idle projects avoid periodic full-tree scans. Verify watcher cleanup on shutdown.
2. **Retained compiler state.** Keep the project graph and reusable frontend state
   alive between dev builds. Invalidate changed modules and affected dependents,
   including changed exports, removed imports and configuration changes. Compare
   diagnostics and native behavior with clean builds after edit sequences. Prove
   unrelated modules reuse state; artifact cache hits alone do not satisfy this step.
   Bound retained memory and recover from failed checks without stale diagnostics.
3. **Separate native runtime compilation.** Compile reusable runtime support
   separately from changing application code and link the required pieces. Preserve
   feature selection, native ownership/lifetime behavior, offline builds and portable
   toolchain invalidation. Prove ordinary application edits reuse runtime objects.
   Document remaining application recompilation boundaries explicitly. Do not weaken
   borrow checking or hide shared ownership to establish the boundary.
4. **Bounded caches and installed acceptance.** Define retention/eviction for old
   artifacts and runtime variants. Preserve active executable safety and concurrent
   project isolation. Verify cold start, warm restart and fresh edits through the
   installed CLI, retaining the last working program on failed or cancelled builds.

Use focused failing tests for each stage. Cover macOS and Linux watcher backends;
record which platforms actually execute acceptance tests and retain explicit limits.
Keep callbacks, default exports, recursive views and resource cleanup represented in
clean-versus-incremental equivalence checks as relevant code paths change.

## Measurement and closure

Before optimization, capture a reproducible baseline on small and larger generated
multi-module projects. Record project shape, source/toolchain identities and sample
counts. Measure cold startup, identical warm startup, leaf edit, shared-dependency
edit, configuration edit, idle CPU, retained memory and cache growth. Report stage
costs and edit-to-program-output latency with repeated samples and distribution
summaries; do not substitute identical-source cache-hit time for fresh-edit latency.

Preserve baseline and final records in SQLite and compare matching cases. Investigate
regressions, then refresh installed consumers and run lifecycle/equivalence checks.
At the completed milestone run the relevant full compiler/editor regression scope.
Implementation and measured acceptance are separate fields; neither is complete yet.
Do not close this milestone with any of the three architectural steps merely listed
as future work. Track unexpected blockers here and in the main roadmap; request an
owner decision only if the resolution materially changes the approved contract.

## Current next action

Capture the baseline and implement filesystem-event watching first. Continue through
retained frontend state and separate runtime compilation before resuming S09 APIs.
