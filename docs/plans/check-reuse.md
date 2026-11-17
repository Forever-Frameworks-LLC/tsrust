# Check reuse across passes and edits

Status: design, 2026-11-17. Owner goal: a much faster `tsrust dev` edit loop
with output identical to a clean build. Reuse may never let a development build
differ from a release build or from a fresh check of the same sources.

## Where Studio check time goes

Measured on Studio (101 modules, 1,264 functions, 2,638 parameter-mode
coordinates) with the release compiler at 894c595f2, one `tsrust check` of
11.7 s:

| Step | Time |
|---|---|
| Parse (cached per module) and link into one program | ~1.4 s |
| Direct analysis attempt, aborted with `recursion-inference` | 0.5 s |
| Speculative access pass 1 (1,099 parameter modes raised) | 2.2 s |
| Speculative access pass 2 (69 parameter modes raised) | 2.2 s |
| Speculative access pass 3 (no change; confirms the fixpoint) | 2.2 s |
| Final definitive analysis | 2.3 s |
| `finish_analysis` (error effects, definite assignment, layouts) | 0.5 s |

`analyze_with_recursive_fallback` first runs one definitive analysis. Any
recursive dependency group, any default parameter with an owned type, or any
generic captured value aborts it with `recursion-inference`. Large programs
always contain one of these, so they always take `analyze_recursive_with_proofs`.
That function iterates a finite access lattice over the whole program:

- the access mode of each function parameter (`modes`),
- named capture access (`capture_access`),
- access of module bindings captured by closures (`program_access`),
- callback kinds of call sites (`program_callback_sites`).

Each pass builds a fresh `Analyzer` from a clone of the program, checks every
function under the current assumptions, and raises the lattice to what it
observed. A pass that raises nothing proves a fixpoint, and one more definitive
analysis produces the checked program. Iteration starts from the bottom of the
lattice (copy types `Copy`, others `Read`) and every coordinate only rises, so
the result is the least fixpoint.

Observations:

1. In Studio only parameter modes move. Captures, module bindings and callback
   sites never change after pass 1.
2. The third speculative pass only confirms. The final definitive pass then
   repeats almost the same work.
3. At the fixpoint, the definitive pass observes exactly the lattice it was
   given (checked by instrumenting the final pass: no coordinate would rise).
4. About 80% of check time is these five whole-program analyses. Speeding up
   the remaining hotspots can save another 10-20% at most.

## Constraint: the least fixpoint, always

Recursive groups can justify themselves. A parameter that is assumed `Consume`
because the function passes it to itself is a fixpoint, but not the least one.
If a later edit removes the real consumer, seeding the lattice with the old
values would keep `Consume`. The checked program, and then the emitted Rust
(moves instead of borrows), would then differ from a clean build. Any reuse must
reach the same least fixpoint as iteration from the bottom. The rule used below:

> Seeding coordinates with values that are at most their least-fixpoint values
> converges to the same least fixpoint. A coordinate whose defining equations
> and inputs are unchanged has an unchanged least-fixpoint value, so it may be
> seeded with its previous value.

## Phase 1: the definitive pass confirms convergence

Run every iteration after the first as a definitive analysis. If its observed
lattice equals its assumptions, it is the fixpoint, and its checked program is
the result. Otherwise raise the lattice and continue. If a definitive pass fails
with a diagnostic that a speculative pass would tolerate, rerun that pass
speculatively.

- Studio: speculative 1, speculative 2, definitive (confirms): 3 analyses
  instead of 4, saving about 2.2 s.
- Obligation: for the same assumptions, a definitive and a speculative run
  observe the same lattice. Add an audit switch (`TSRUST_CHECK_AUDIT=1`) that
  runs both and compares observations at every pass. Run it over the compiler
  library tests, the CI runtime suites, Studio, Tasks and the gallery before the
  switch-over, and keep it in one CI job.
- `effects_only` and `speculative_access_run` decide what the speculative run
  tolerates. Read every use before implementing, and list the ones that can
  change an observation.

## Phase 2: propagate within a pass

A pass reads its assumptions only from the previous pass. Most of pass 2's
raises (69 in Studio) follow from pass 1's raises in callees that were already
checked earlier in the same pass. Publish a function's observed parameter modes
into the current assumptions as soon as its group finishes (chaotic iteration).
Every observation then comes from a state that is still at most the least
fixpoint, so the result is unchanged, and the confirming pass may come one pass
earlier. Measure how many passes Studio, Tasks and the suites need before and
after. Keep the audit from phase 1 to compare checked programs against the old
iteration.

## Phase 3: warm start across edits

`tsrust dev` keeps a project session (cached parse per module). Store the
converged lattice in that session under stable keys, then seed the next build's
iteration with the entries whose inputs did not change.

- **Keys.** Lattice vectors are indexed by function order, which changes with
  edits and with specialization discovery. Key each coordinate by the linked
  function name, the specialization's type arguments and the parameter position.
  The stable-name keys in `project/link.rs` (`name_keys`) are the starting
  point. Key module bindings and capture coordinates by declaration identity
  (module plus name), not by byte offset.
- **Dependency cone.** A function's parameter modes depend on its own body, on
  the types and modes of what it calls, and in some cases on its callers.
  `function_dependency_order` notes that record callback contracts depend on
  environments the caller builds, and specializations exist because of callers.
  The first version uses modules: reuse a function's coordinates only when its
  module and every module it imports (transitively) are unchanged, and the
  function is not specialized from, or called with a record callback by, a
  changed module. Recursive groups are reused whole or reset whole.
- **Reset to bottom** for everything else, then iterate (phase 1 and phase 2)
  as usual. Seeded values equal their least-fixpoint values and reset values
  are below them, so the result equals a clean check.
- **Expected effect.** If an edit leaves every reused coordinate correct and the
  reset ones converge in one pass, check becomes parse/link + one definitive
  analysis + finish: about 4.7 s instead of 11.7 s on Studio. Edits that reset a
  long chain of callers need more passes.
- **Proof in tests.** The audit switch also checks each warm start against a
  cold iteration from the bottom (same lattice, same checked program, same
  emitted Rust). Add edit-replay tests: apply recorded edits to Studio and the
  Tasks apps (string, JSX, new function, removed consumer in a recursive group,
  changed callback record) and compare warm and cold results after each.
- **Diagnostic gaps.** If the cone rule is wrong anywhere, the audit fails.
  Narrow the rule (for example to modules with no importers changed) rather than
  accept a difference.

## Phase 4: skip the aborted direct attempt

The direct attempt costs 0.5 s before it aborts. Skipping it is only correct if
the recursive path gives the same checked program whenever the direct attempt
would succeed. That is not established: with no assumptions, default parameters
start at `Consume`. Measure first: run both paths over every suite program that
succeeds directly and compare. If they match, take the recursive path straight
away when the session's previous check of the same entry needed it. Otherwise
leave it.

## Later: reuse checked functions, not only the lattice

Phases 1-3 still analyze every function at least once per edit. Reusing a
function's checked body needs its analysis to be a function of stable inputs.
Today the analyzer appends records, unions, closures and callable families to
arenas, numbered in discovery order, so the IDs inside a checked function depend
on everything checked before it. That needs content-addressed arena identities,
plus per-function caches of inferred results and diagnostics. It is a separate
project, built on phase 3's keys and on the emit-side stable names.

## Also in the loop

Parse and link (~1.4 s) rebuild the linked program on every edit, even though
modules are parsed from cache. `analyze_recursive_with_proofs` clones the whole
program for every pass. Emit (~1.5 s) and runtime prepare (~0.6 s) follow the
check. These are smaller targets to take on after phases 1 and 2.

## Order of work

1. Audit switch, then phase 1. Gate: audit clean on lib tests, CI suites,
   Studio, Tasks and the gallery; Studio check time measured.
2. Phase 2, under the same audit.
3. Phase 3 behind `TSRUST_CHECK_REUSE=1` until the edit-replay tests and the
   audit pass, then on by default in `tsrust dev` only. One-off `check`, `build`
   and `test` stay cold.
4. Phase 4 only if its comparison holds.
