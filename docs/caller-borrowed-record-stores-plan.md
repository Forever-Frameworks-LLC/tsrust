# Caller-borrowing stores through record parameters (B7)

Status: gate 183 started; not accepted. B5/gate 182 is accepted at `00214a1`.
Read the completion ledger and B5 acceptance handoff before implementation.

## Required behavior

A callback installed through a mutable record parameter may retain a borrow of an
input owned by the caller, provided the caller's owner outlives the stored callback.
The caller cannot invalidate the borrowed owner while that callback remains live.
Callee locals still cannot escape. No implicit clone, shared capture cells, lifetime
casts or new callable variant is permitted.

The first witness installs a readonly callback borrowing a readonly array input,
invokes it after the installing function returns, then mutates the array after the
holder's scope ends. The existing compiler rejects the store with callback-escape.
This witness is implementation evidence to repair, not acceptance.

## Modules and invariants

- Separate eligible input provenance from callee-local storage in the replacement
  checks in record_parameter_carriers.rs. Parameter aliases and nested captures need
  actual owner provenance, not only a parameter flag.
- Transport capture obligations to caller arguments when checking named calls.
  Attach stable capture-loan identities to the stored record path and preserve them
  through relays, replacement, moves and failure paths. Reuse capture_loan_owners
  and the existing release summary machinery; do not weaken conflict checks.
- Give native replacement bounds explicit source-input lifetimes. A closure must
  borrow the caller owner through the input, not the callee's stack-local reference
  binding. Reuse native record bridges and checked concrete carrier planning.
- Keep runtime carrier shape, point-sensitive source types, error contracts and
  nominal record declarations separate. Do not revive the B5 environment-ID bugs.

## Acceptance sequence

1. Prove the focused witness red and its independent Node reference green.
2. Implement readonly input borrowing; verify a live-holder mutation/move rejection,
   callee-local rejection and release after holder destruction.
3. Cover nested records, nullable payload stores, direct and nested relay calls,
   repeated independent factory calls and multiple input environments.
4. Cover mutable captures and borrowed callback inputs under the same existing
   ownership contracts. Verify alias conflicts and once-consumption constraints.
5. Cover failure before/after stores, replacement and clearing, branch/loop stores,
   early exits, disposal and source diagnostics for genuinely invalid lifetimes.
6. Add editor facts, guide examples, formatter round trips and matched workloads
   with independent oracle/source contracts and registrations as forms land.
7. Use bounded affected checks and eight depth canaries after shared dispatch changes.
   At the complete caller-transport family milestone, run complete regressions and
   matched timing, review performance and preserve exact SQLite evidence.

## Baseline traps

The existing record_call_capture_release replay deliberately returns no precise
release summary when a callee creates a new borrow, has defaults or exception syntax.
It currently tracks incoming callback payload tokens rather than caller-owner
provenance. Do not simply remove those guards: new borrow transport and exceptional
completion require explicit evidence. Existing boundary rejection tests include
borrowed callee inputs; convert only the forms that become genuinely supported and
retain callee-local and invalid-escape controls.
