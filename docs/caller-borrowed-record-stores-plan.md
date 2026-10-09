# Caller-borrowing stores through record parameters (B7)

Status: gate 183 started; not accepted. B5/gate 182 is accepted at `00214a1`.
Read the completion ledger and B5 acceptance handoff before implementation.

## Current implementation progress

Direct readonly and mutable array inputs, stable readonly view aliases, nested
record destinations and multiple input captures have native witnesses. Named relays
preserve provenance through reordered parameters and alias chains. Repeated stores
of one origin from different inputs use a common reborrow lifetime for the affected
carrier slot, while separate destinations retain independent lifetimes.

Borrowed readonly and mutable callback inputs can be captured by a stored wrapper.
Stable callback aliases retain their original input owner. Native generic capture
slots are independent even when callback signatures match. Local borrowing wrappers
also work; an owned escaping closure still cannot consume a borrowed callback.
Source controls cover live-holder mutation and transfer, independent mutable callback
invocation, callee-local escape and shorter-lived input storage. Editor diagnostics,
hover and parameter help have focused witnesses.

The matched caller_borrowed_record_stores workload retains eighteen observations per
iteration, with an independent arithmetic oracle, source contracts, ABI support,
runner/profile/report hooks and registration controls. Verification is untimed.
This remains progress within B7, not family acceptance or timing evidence.

Repeated replacement now accepts distinct callback input environments, including
readonly and mutable relays and multiple independent capture positions. A finite
native payload plan preserves whole input vectors and gives each environment a
separate arm without changing source origins. Capturing a replaced holder now has
readonly, mutable and nested native witnesses. Native capture storage preserves
final carrier layouts separately from checked point types. An early readonly
capture can invoke its initial callback even when later stores require a mutable
carrier; dispatch selects only the origins permitted at that source point. Singleton
concrete capture environments remain in the payload plan as well as alternatives.
Focused replacement checks pass twelve tests; this is not family acceptance.
Direct readonly and mutable callback assignment now has focused forwarding,
optional/default, rest and alias-relay witnesses. Consuming direct inputs now have native single-use and repeated-call rejection
witnesses. Installer failures before and after stores preserve the selected callback,
state and retained mutable loans. Typed failures from the installed callback itself now retain completed caller
argument origins, including conditional installation and handlers in captured closures.
Absent concrete origins remain unresolved rather than acquiring a fabricated type. Remaining acceptance includes nullable
payload stores, mutable data aliases, failure paths, exact replacement and clearing,
branch/loop stores, early exits and disposal. Recursive relay summaries still need
finite-path evidence before acceptance. Complete the required family before heavy
regression and timing; do not remove the existing conservative release guards blindly.

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
