# Gate 180: fallible callback fields behind record parameters

Status: accepted. Requirement B6 of
[the compiler completion ledger](../plans/compiler-completion-plan.md); first gate of the
scoped aggregate family chosen by the owner.

## Behavior

A function that takes a record with a borrowing callback field can call a field
callback that fails. The failure propagates through the function, its relays and
its callers with a typed error, and `try`, `catch` and `finally` work inside the
function and around its calls. The error type of a handler is the union of what the
callbacks of every constructed record of that type can raise.
The [guide](https://forever-frameworks-llc.github.io/tsrust/language/borrowed-callable-fields/#handle-callback-failures) describes it.

## Implementation

- **Grounding.** The error contract of a callback is the join of its origins. For a
  parameter the origins are call arguments. For a field of a record declaration they
  are the callbacks held by its constructed instances
  (`error_effects::record_field_origins`), used by both the final annotation and the
  analysis-time annotation that types handlers.
- **Discovery.** Records are constructed in any function, so their callbacks must be
  known before a handler is typed. `Analyzer::discover_record_callbacks` checks every
  body once without recording effects and without following calls, ignoring errors.
  It runs only when the program declares a borrowing callback field and has an
  exception region.
- **Order.** A function whose handler still has an unresolved contract is postponed
  until the other functions are checked; its source is restored for the second
  attempt. After the loop, a function with a handler is checked again when records
  constructed after its check added callbacks to a field contract.
- **Native code.** No emitter change was needed: a callback stored in a field shares
  the callback error layout of its contract exactly like a callback argument.

## Limits

- The field contract belongs to the record declaration, not to one record. A handler
  must narrow alternatives that only other records of the same type can raise.
- When no record of the type is constructed anywhere, a handler that names the error
  reports `error-effect` with guidance.

## Evidence

- `fallible_record_callbacks`: native fixture (propagation, typed handler with
  `finally` over string, number and runtime alternatives, relay, failure added by
  the callee, mutable field state before a failure, a failing callback stored by a
  callee), a native runtime-error fixture, a handler declared before the functions
  that construct its records, and two diagnostics. The main fixture is
  formatter-stable and passes the pinned Node reference.
- Whole compiler crate: 2,362 pass, 0 fail, 3 intentional ignores (2,360 in the run,
  plus the two stale gate 179 expectations repaired afterwards and rerun). Whole
  editor crate: 365 pass.
- Matched `fallible_record_callbacks` workload at the end of the manifest with
  oracle, token-based source contract, registration test and hooks; native and
  Node-after-tsc agree on five untimed cases. No timing.
- The guide example runs natively and passes the Node reference.
