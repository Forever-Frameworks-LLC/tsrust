# Gate 177: unreachable catch bodies and caught-error assertions

Status: accepted. Requirements B3 and B4 of
[the compiler completion ledger](../plans/compiler-completion-plan.md).

## B3: checking handlers that cannot run

A `catch` body is checked once per failure state that reaches it. When nothing in
the `try` body can fail, no state reaches the handler and its statements were never
checked; the record gate then stopped emitting them, which hid type and name errors.

Checking the handler during the region check is not possible: a region is checked
several times while callable contracts are still provisional, and a handler that
looks unreachable in an early pass can receive a typed error later.

The analyzer now keeps, for each handler that no failure reached, a copy of the
state at the region (`Analyzer::dead_handlers`). A later check of the same region
that reaches the handler removes the entry. When analysis is definitive,
`check_dead_handlers` checks the remaining bodies on those copies, without recording
effects. The error name has the runtime error type there. The body is still not
emitted.

## B4: caught runtime error passed to `strictEqual`

The named declaration family handoff reported an internal unsupported-type
diagnostic for an unrefined caught runtime error passed to the generic assertion.
It does not reproduce at 5120aad. Runtime, string, int and mixed error alternatives
passed to `strictEqual`, compared with themselves, with a literal and through
`error.name` and `error.message` all check and run; a caught runtime error compared
with a string reports the ordinary generic `type` error. These shapes are kept as
witnesses in `unreachable_catch_bodies` (reached handlers keep their real error
type). No source change was needed.

## Evidence

- `unreachable_catch_bodies`: six rejection rows (type and name errors, unnamed
  handler, unknown field, nested region, generic function), one valid unreachable
  handler that compiles and never runs, and one fixture of reached handlers.
- Bounded compiler batch with unit tests (including the exception state tests that
  re-check regions), exception suites and all eight depth canaries: 653 pass, 0 fail.
- Whole editor crate: recorded with the commit.
- No runtime behavior changes, so no workload is added; the existing exception
  workloads are untouched.
