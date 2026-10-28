# Direct caller-borrowed callback stores

This is the next implementation slice of B7/gate 183, following `12ffe9d`.
It is not accepted. The complete B7 scope in plans/caller-borrowed-record-stores-plan.md
remains authoritative.

## Initial red evidence

Three focused witnesses in direct_borrowed_callback_stores.rs fail with the explicit
callback-escape diagnostic in record_parameter_carriers.rs: direct readonly input,
direct mutable input with retained state and release after holder disposal, and a
stable alias passed through a reordered named relay. Each successful-source witness
first performs the shared formatter round trip. The independent TypeScript reference
in fixtures/direct-borrowed-callback-stores/reference.ts passes on Node 24.11.1.
This reference establishes observable values, not Tsrust ownership rules.

The first trial incorrectly mutated the array while its original capturing closure
was still live. That capture-conflict was correct. The corrected readonly witness
ends both the holder and original closure scope before mutating the array. Preserve
both logs; do not weaken that existing ownership rule.

## Implementation requirements

A direct store must retain the actual caller callback environment, not a reference
to the callee's parameter slot. Preserve readonly, mutable and consuming invocation
contracts and declared argument/result/error behavior. Stable aliases must resolve
to the original owner, and relays must transport that provenance. Do not introduce
implicit clones, shared cells, dynamic dispatch or a new source callable variant.

The existing borrowed-input summary describes captures of concrete closure origins
with source parameter, capture position and access mode. Direct abstract callback
inputs currently have no such concrete origin. Extend the transport representation
explicitly rather than deleting the rejection or inventing a source origin ID. Keep
complete caller input vectors correlated through the existing finite worklist.
Native replacement bounds and associated record field conversion must accept the
actual caller input environment with its source-input lifetime. The current native
replacement helper only handles Closure and BorrowedCallable payloads; accepting
Callback in the analyzer alone would leave native conversion incomplete.

Source point invocation contracts remain separate from final native carrier layouts.
Exact replacement and clearing must retain the old/new loan rules until proven;
never drop a caller borrow merely because a direct assignment has no capture list.

## Next checks

Implement readonly and mutable direct storage together with their provenance and
native conversions. Add controls rejecting independent invocation of a retained
mutable input, invalid shorter-lived owners and callee-local escape. Add consuming
input coverage, forwarded arguments, optional/default/rest call shape and failure
transport. Reuse existing callable contracts and adapters instead of fabricating a
zero-argument wrapper that loses those contracts.

Then add focused editor diagnostics, hover and signature help, a guide example and
matched workload observations with independent oracle and mutation guards. Run
bounded affected checks, required depth canaries after shared dispatch changes,
whole-project formatting and exact SQLite evidence import. No family acceptance,
complete regressions or timing is justified by these initial red witnesses alone.

## Implemented forwarding slice

`direct_callback_stores.rs` reconstructs the complete checked callback signature
and creates an ordinary forwarding closure before field assignment analysis. This
is a real checked capture, not an invented origin in a provenance summary. Argument
ownership, result types, optional parameters and rest packs use existing annotations
and adapters. The forwarding closure borrows the callback environment, so existing
caller-input summaries and native associated-field conversions can transport it.
No new source callable variant, dynamic dispatch or implicit clone is needed.

Seven direct-store checks pass across the focused batch and supplemental checks:
readonly input and release, mutable state and release, stable alias relay, required
arguments, optional/default invocation, rest invocation, and rejection of independent
mutable input access. Twelve replacement checks and seventeen existing caller-store
checks pass. Editor signature help retains the declared argument name, hover hides
internal carrier names, and independent mutable invocation reports capture-conflict.
All eight depth canaries pass on the analyzer change.

The matched workload replaces its second readonly installation with a direct store.
Its fourteen observations, denominator and independent result oracle are unchanged.
A source contract and mutation guard require the direct assignment. Consuming callback
inputs, exceptional completion and the remaining B7 forms are not accepted by this
slice. Verify those forms explicitly before family acceptance.

## Consuming stores and installer completion

The forwarding helper also handles owned callback contracts. A consuming input
resolves to an owned single-use contract; skipping owned contracts left the original
abstract parameter in the stored-value conversion and caused callable-layout.
Removing that skip lets the ordinary capture analysis transfer the environment.
Single invocation passes; repeated invocation reports partial-transfer on the record
field. The original callback is not implicitly copied.

Installer failures before and after storage pass native assertions. Mutable state
survives failure after installation, independent invocation remains rejected while
the holder retains the input, and disposal releases the input. These are installer
completion checks, not acceptance of every exception path in B7.

The new directly_stored_callback_preserves_typed_failure_and_success witness remains
red: catching the installed callback's string failure reports error-effect with an
unresolved catch contract. Investigate inferred callback error origins, retained
wrapper reachability and record field origins in error_effects.rs. Keep the witness;
do not erase the catch binding or substitute an untyped handler.

## Completed caller error origins

The earlier callback-thrown-error witness is repaired. Catch checking previously
considered the current try body before the caller had a stored checked body; an
installation before the try was therefore absent from callback-origin inference.
The isolated caller_error_origins module retains completed named-call argument types
in the checking context. Region inference includes those facts, and nested closures
inherit the outer facts. These are error-type facts, not ownership loans or runtime
storage. Checked point types and final native carrier types remain separate.

Native checks cover the original typed string failure and success path, conditional
installation of number/string failures, and a bound handler in a closure capturing
the installed holder. A negative check keeps the error-effect diagnostic when there
is no concrete callback origin. Catch hover also now refreshes from the compiler's
checked declaration type after those facts load, rather than remaining untyped.
