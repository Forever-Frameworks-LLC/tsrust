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

## Phase 1: definitive pass beside a confirming pass (done 2026-11-17)

The first idea was to let a definitive pass stand in for the confirming
speculative pass. A definitive run applies checks that a speculative run skips
(`effects_only` gates more than 50 checks and some behavior), so the two might
observe different lattices. Proving them equal would need an audit, and a gap
would change results.

Implemented instead: once a pass raised few coordinates (at most
`max(16, parameters / 16)`), the next speculative pass runs with the
definitive analysis under the same assumptions on a second thread. If the
speculative pass confirms the fixpoint, and appended no modes for new functions,
the concurrent result is exactly what the sequential definitive run would
compute, and it is used. Otherwise it is discarded. The pass sequence is
unchanged, so results are identical by construction. The threshold only affects
speed.

- Enabled by `tsrust dev` (`set_concurrent_check`). Library use, tests and
  batch commands stay on one thread: with the parallel test harness, a second
  analysis thread doubled the compiler library test time (62 s to 116 s).
  `TSRUST_CHECK_THREADS=1` turns it off and `=2` turns it on.
- Studio: `tsrust check` 10.9 s on one thread vs 9.1-9.9 s with the concurrent
  pass; in `tsrust dev` check is 9.6-9.7 s and an edit rebuilds in 16.5-16.8 s
  wall (21-24 s before this session's check work). Emitted Rust for Studio is
  byte-identical with the concurrent pass on and off; a library test checks the
  same and that the concurrent path ran.

## Phase 2: propagate within a pass (does not apply; measured 2026-11-17)

At a call site the analyzer already uses the callee's checked parameter modes
when the callee was checked earlier in the same pass. Assumed modes are read
only for callees that are not checked yet. Instrumenting every read on Studio:

| Pass | Coordinates read | Reads that differ from the raised lattice | Raised |
|---|---|---|---|
| 1 | 2,638 (all) | 1,099 | 1,099 |
| 2 | 2,636 | 57 | 69 |
| 3 | 2,636 | 0 | 0 |

Every pass reads nearly every coordinate. `function_dependency_order` leaves
out callees whose record parameters carry callbacks ("record callback contracts
depend on environments constructed by the caller"), so UI components with
props records are checked after their callers, and every caller sees assumed
modes. A chain of components needs one pass per level. All 69 pass-2 raises are
on functions created during the run (component instantiations and generic
specializations), whose callers ran before them.

So publishing modes earlier in a pass cannot help: when a callee finishes, its
callers have already been checked. Two variants were tried and reverted:

- Read-based convergence (stop when every value a pass read equals the raised
  lattice, since the next pass would then run identically): sound, but on
  Studio pass 2 still reads 57 stale values, so pass 3 is still needed. No gain.
- Per-pass fixed costs: cloning the program and constructing the analyzer cost
  about 0.1 s per pass; linking about 0.13 s. Not worth restructuring.

What would reduce the pass count:

- Check a record-callback callee before its modes are read (at the call, with
  the caller's environment), so callers observe checked modes and an acyclic
  chain converges in one pass. This changes the analyzer's checking order,
  which the code deliberately avoids today, and needs an audit that compares
  checked programs with the current order.
- Phase 3 (warm start across edits).

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
- **Proof in tests.** An audit switch checks each warm start against a
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

1. Phase 1 (done).
2. Phase 2, with an audit switch that compares the checked program against
   the current iteration on lib tests, CI suites, Studio, Tasks and the gallery.
3. Phase 3 behind `TSRUST_CHECK_REUSE=1` until the edit-replay tests and the
   audit pass, then on by default in `tsrust dev` only. One-off `check`, `build`
   and `test` stay cold.
4. Phase 4 only if its comparison holds.
