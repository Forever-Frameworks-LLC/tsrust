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

The initial record_call_capture_release replay deliberately returned no precise
release summary when a callee created a new borrow, had defaults or exception syntax.
It currently tracks incoming callback payload tokens rather than caller-owner
provenance. Do not simply remove those guards: new borrow transport and exceptional
completion require explicit evidence. Existing boundary rejection tests include
borrowed callee inputs; convert only the forms that become genuinely supported and
retain callee-local and invalid-escape controls.

## Nullable callback installation and clearing checkpoint

Nullable callable fields now retain the concrete installed callback carrier while
preserving the source-level nullable annotation. Present payload conversion uses
the checked callback conversion; absent values carry no callback environment.
Nested local record annotations infer capture generic arguments instead of naming
signature-only parameters.

Release replay traverses nullable union members. Native emission separately proves
absence on every normal exit and reconstructs only absent variants after the call.
This ends the old Rust borrow without a lifetime cast and keeps the holder usable.
The proof handles direct clears, blocks, both branches, bare early returns and
named relays. Unknown effects, loops and recursive proof cycles stay conservative.
A released original payload alone does not prove that the replacement is absent.

Focused witnesses cover readonly and mutable installation, invocation, clearing,
input reuse while the cleared holder remains live, both relay exits, local-capture
escape rejection and an uncleared conditional path. Broader B7 acceptance and
measurements remain pending.

Nullable record field payloads also contribute concrete error origins. This keeps
nullable parameter callback bounds consistent with the installed wrapper's fallible
result, including when other callbacks in the same program introduce failures.
Typed callback throws are checked through native execution and editor catch hover.

## Nullable callback replacement checkpoint

Replacement now releases an overwritten callback's input while retaining the new
input. Release replay permits new captures only when the existing caller-owner
transport can account for them. Original incoming payload tokens still prove
whether the old environment survives any normal exit. Conditional stores that skip
replacement retain the old loan. Defaults and exception syntax remain conservative.

One source closure origin may contain different concrete callback environments at
different calls. Nullable borrowed forwarding wrappers therefore use checked
singleton source families, with finite native alternatives for whole capture
vectors. This adds no source callable kind or dynamic dispatch. Nullable replacement
metadata records the actual injected closure payload, not its abstract annotation.
The final carrier joins all genuine branch alternatives, including capture-free
callbacks.

The isolated record_callback_reification_emit module filters native alternatives
using this call's checked callback arguments and retained-capture positions. It
preserves unknown alternatives conservatively and reconstructs only admitted
payloads after a proven overwrite. Native reconstruction is distinct from source
loan release; neither substitutes for the other.

Witnesses cover different readonly/mutable environments, nested record fields,
input aliases and relays, borrowed/capture-free branches, old-input reuse and
new-input conflict rejection. More complex retained-store loops, recursive
completion and nested generic environments remain pending.

The same checked-environment reconstruction also applies to non-nullable fields.
Its focused witness previously retained the old mutable input in generated Rust;
it now releases that input while invoking the distinct replacement environment.

## Equal-type owner instances: direct store checkpoint

The new factory-instance and data-input probes expose native borrow retention even
when source release is correct. Distinct runtime owners can have the same concrete
callback or array type, so filtering native alternatives only by that type cannot
separate their lifetimes.

The isolated direct_record_store_emit module lowers a precisely recognized installer
at its call site. Its checked body must consist only of one field assignment of a
closure capturing formal inputs. Arguments must be named storage; defaults, rest,
hidden captures, fallible installers, restored captures and scoped record views
remain on the ordinary call path. Consumed non-copy arguments are excluded even
when unused, because their destruction must not move to a later scope. Recognized
construction retains the original capture access modes and checked final carrier.
Rust sees the actual field overwrite and can end the old owner's borrow. No source
ownership rule changes and no environment is cloned.

Focused native cases cover independent same-origin counter factories in nullable
and non-nullable fields, direct array inputs and plain record inputs. Controls
preserve effectful argument evaluation and installer body effects exactly once.
This is not a complete representation for all equal-type owner instances: aliases,
relays and larger effectful installers need the next native ownership work. Keep
that requirement open rather than weakening old-input reuse assertions.
