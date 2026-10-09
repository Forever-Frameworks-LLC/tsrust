# Caller-borrowing stores through record parameters (B7)

Status: gate 183 started; not accepted. B5/gate 182 is accepted at `00214a1`.
Read the completion ledger and B5 acceptance handoff before implementation.

## Current implementation progress

The latest focused checkpoint adds transitive hidden-source borrowing through locally
constructed installer closures, including three-layer calls and conditional old-owner
retention. Reusable mutable captures and explicit owning Once/Owned transfer are also
verified. Foreign captured callees, abstract environment transport and the remaining
control-flow/recursive interfaces are still open. Five installed consumers remain B5.
The sections below preserve earlier checkpoint boundaries and evidence.

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

The matched caller_borrowed_record_stores workload now retains 68 observations per
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

## Open native owner-instance obligation

The next focused witness, `effectful_installer_releases_an_independent_same_origin_factory`, is red after the direct-store checkpoint. The installer increments a separate counter and then stores its callback. Both callbacks come from independent calls to the same counter factory. Source analysis accepts old-input reuse after replacement, but generated Rust reports E0499 when the old callback is invoked while the holder remains live. This distinguishes a genuine owner-instance limitation from a callback-origin mismatch.

Do not generalize the direct-store recognizer by discarding, moving or duplicating installer effects. Do not introduce shared cells, clone environments or cast lifetimes. General transport must retain actual Rust references while distinguishing independently owned inputs that have equal native types.

A native-only owner-marker representation is the leading design to investigate. It is not implemented or accepted. Each concrete caller storage owner would supply a zero-sized type marker; borrowed parameter markers would remain generic and propagate through checked aliases and relay calls. Stored payload alternatives would retain correlated vectors of owner markers and callback types. This permits separate native lifetime parameters for equal callback types borrowed from distinct owners. Replacement reification would select the new owner alternatives using the existing all-normal-exits release proof, while ordinary calls execute installer effects unchanged. Source callable kinds and source ownership remain unchanged.

Before adopting that representation, prove a minimal Rust model with two equal-type factory instances, an opaque effectful installer, independent payload lifetimes and old-input reuse after replacement. Then add deterministic checked-owner provenance planning in an isolated module with unit witnesses for stable aliases, relay propagation and recursive convergence. Integrate payload construction and conversions, generic installer bounds, dispatch and consuming payload behavior together. A marker must identify the storage owner rather than merely the closure origin or a syntactic call site. In particular, repeated relay calls must retain caller ownership identity, and repeated execution of one source declaration must not claim runtime instances are statically distinct without a valid lifetime proof.

Acceptance must cover readonly and mutable callback inputs, direct arrays and records, nullable and plain fields, effectful arguments and installer bodies, failures, aliases, relays, branch joins and disposal. Invalid old-input access before replacement and new-input access while retained must remain source errors. Preserve the existing direct-store cases and correlated environment alternatives. The current red is retained, not ignored.

The minimal native owner-marker model now compiles and executes. Its two integration tests retain the opaque installer, check two independently mutable counter instances with equal native types, release the old instance after selecting the replacement variant, preserve effect counts, propagate the marker through a generic relay and release the new input after consuming its payload. These are representation proofs, not compiler implementation acceptance. The generated-code effectful-installer witness remains red.

## Owner provenance prototype

`borrowed_owner_provenance.rs` is deliberately test-only pending coordinated native transport integration. It computes deterministic, correlated vectors of concrete named storage owners from checked functions and retained-store summaries. Stable borrowed aliases preserve their owner, generic relay parameters propagate caller owners, recursive relay contexts converge, and repeated call sites do not fabricate distinct owners. It does not use the source callback origin as owner identity.

The prototype grounds exact named arguments only. Closure-body contexts, field/index projections, computed temporaries and unresolved formal inputs remain unsupported; they must not be silently treated as complete provenance during integration. A production plan needs explicit completeness information before it can narrow native variants. It also needs owner provenance carried through closure captures and named hidden capture parameters. Keep ordinary checked lowering for unsupported contexts until a valid general transport exists. The generated effectful-installer witness remains red.

Next integrate owner vectors with the existing whole capture-type vectors, preserving correlation across both dimensions. Wrap retained payloads in zero-sized marker types, add generic owner marker parameters to retained-input functions, and pass caller markers through ordinary calls and relays. Update family conversions, nullable conversions, native lifetime counts and replacement reification together. Reification must select by actual owner provenance as well as callback type. Consuming dispatch must move the wrapped payload rather than trying to move through a dereference. Do not declare B7 accepted from these prototype tests.

## Owner-marker transport implementation

This implementation supersedes the preceding prototype-only status. The effectful-installer red now passes. Checked owner provenance is used by native emission, and retained payloads with equal callback types from distinct named owners carry zero-sized marker types. Ordinary installer calls receive inferred marker arguments; generic relays propagate their input markers. The existing all-normal-exits release proof authorizes reconstruction of only the replacement owner variants. Installer effects remain in the ordinary function body.

Markers attach at record storage conversion, not at the original local closure declaration. This preserves local closure types and prevents a generated-type mismatch found by a forwarding-local witness. Nullable and plain fields use the same owner identity. Native payload alternatives keep correlated callback-type and owner vectors. Lifetime counting includes every tagged payload separately, including when a holder is captured by another closure; the nested-holder witness exposed and now covers this obligation.

The focused cases retain the original factory state, alternate the holder back to its first owner, call both released owners at the appropriate points, check installer effect counts and invoke a nested holder capture. No environment is cloned, no source callable kind changes and no lifetime is cast.

The current transport enables markers only for complete named-input contexts that actually collide in callback types. Unequal callback types retain the existing native representation. Direct data captures remain on the existing representation; effectful equal-type array/record captures still need singleton-family native transport. Closure-body caller contexts and function-value installer calls disable marker specialization until their provenance can be carried explicitly. Field/index arguments and unresolved inputs also remain outside the marker proof. Keep these obligations open as part of B7 rather than calling the family complete.

## Effectful named data-input replacement

The equal-type data reds exposed the same native borrow retention for array and record inputs. Nullable data stores now retain a checked singleton borrowed callback family, instead of leaving a single closure type as the final field carrier. This keeps source invocation and ownership contracts while allowing native payload alternatives for distinct caller owners. Owner marker selection also covers data-only capture vectors. Lifetime alternatives include a singleton concrete environment even when there is no abstract callback capture vector.

Mutable array replacement passes for nullable and plain fields. Readonly array replacement, an effectful named relay, nested holder capture and mutable record replacement also pass. The old input remains reusable while the holder is live, the new input stays borrowed, and installer/relay effects are asserted. The existing callback factory and local forwarding cases remain controls.

The workload's 33rd observation checks the data installer's effect count. Source contracts require both effect increments, so removing one is rejected even though the other contains identical tokens. Field/index inputs, computed temporaries, closure-body callers and installer function values remain outside marker provenance. Continue those composition requirements before B7 family acceptance.

## Conditional callback stores

Immediate conditional callback stores now preserve both capture provenance and native carrier conversion. The initial nullable witness generated incompatible union payload types for its branches. The plain-field witness exposed generic family bounds with too few native payload arguments. A sequence-valued branch exposed the same join defect after observable prefix effects.

The isolated `record_store_branches_emit.rs` helper converts each selected branch to the destination before joining it. It preserves predicate evaluation and emits sequence prefixes once, only in the selected branch. Tagged callable-store wrappers are unwrapped before conversion to the destination's generic callback field. Generic bounds expand the actual closure origins of tagged families instead of naming an invalid aggregate family type. Source retained-store summaries traverse conditional branches and the final value of sequences.

Focused cases cover nullable and plain readonly data captures, old-owner release, predicate and selected-branch effect counts and local-owner escape rejection. The selected effects total 322 across two calls, with both branches exercised. The matched workload preserves four separate observations for conditional invocation, released old-owner state, replacement invocation and effects, bringing the workload to 37 observations. Closure-body callers, installer values, field/index inputs, computed owner temporaries and broader stored-family composition remain open.

### Saved local callback families

A selected callback can now be saved in a local, invoked there, moved into another local and then stored through a mutable record parameter. Retained-borrow summaries collect every checked origin in the saved family. This keeps the new caller owner borrowed while the holder remains live and permits access to the overwritten owner after a complete replacement.

The local native family keeps one untagged payload per source origin. It does not expand a generic installer input into every concrete caller environment. At record storage, a match moves the existing selected payload into the final carrier and attaches its owner marker when required. The shared family emitter supplies the same invocation and result dispatch for local and final representations. Local definitions are emitted only when a relevant local binding exists. No initializer is replayed, payload cloned, source invocation contract changed or lifetime cast introduced.

Focused cases cover readonly and mutable families, nullable and plain fields, invocation before storage, selected-branch effects, generic callback inputs, a local move, old-owner release, new-owner rejection and callee-local escape rejection. The matched workload invokes the selected local before storage and observes that invocation in its effect total. Its readonly local family uses a matching readonly field contract; declared borrowed invocation contracts remain distinct. Closure-body callers, installer function values, more complex input places and the remaining B7 exit and composition cases still require their own acceptance.

### Forwarded installer source effects

A named installer saved as a function value must retain the same caller borrow and complete-replacement effects as a direct named call. Forwarded invocation now applies release and retention after argument checking and specialization. Source summary collection, record replacement propagation and the release replay follow checked `ForwardFunction` edges alongside named `Function` edges. Focused source witnesses cover both direct forwarded invocation and a named relay containing one. The editor rejects conflicting access to the new owner while allowing access to the released old owner.

Native acceptance remains open. Two executable witnesses expose independent issues: closure-body installers need record-view parameter and replacement-bound transport; installer function values need the forwarded callable's native signature, replacement bounds and owner markers transported together. Do not remove the conservative provenance suppression before those call interfaces are complete. Keep both executable witnesses red until their source, native, editor and matched workload obligations are implemented.

### Static installer function values

Static calls through a saved named installer now execute with independent caller owners. `native_function_signature_emit.rs` shares the named target's record-view parameters, replacement obligations, retained-input lifetimes and generic bounds with its forwarded method. Trait and typed-result adapters retain the same replacement obligations. `forwarded_owner_uses.rs` keeps abstract, stored, returned, captured and unsupported forwarding uses conservative. The finite owner and callback-capture planners follow checked static forwarding edges only after that classification. Forwarded calls preserve receiver and argument evaluation, pass owner markers and reify completely replaced record payloads after successful invocation.

Focused native witnesses cover readonly and mutable installers, nullable and plain fields, exact installer effects, independent-owner replacement and typed failure after storage. Source/editor checks continue to reject conflicting replacement-owner access. A planner control verifies that even grounded symbolic inputs cannot bypass an abstract-use guard. The shared signature extraction preserves the existing workload's emitted bytes exactly before the new scenarios are added.

The matched workload now has 41 observations, adding forwarded invocation, released old-owner state, replacement invocation and exact effects. Its installer uses the previously accepted conditional local family, so the new path also exercises local-to-record conversion through a saved function value. All workload support is current and verification remains untimed. Closure-body callers remain red. Abstract installer invocation, forwarding aliases, consuming wrappers, callback-input forwarding and the broader B7 composition cases still require complete provenance and native-interface witnesses before broader acceptance.

### Closure-body installer work in progress

After `61e3a0b0`, the kept native failure is reproduced. A new source control, `closure_body_installer_retains_new_owner_in_analysis`, shows that mutation of the newly installed owner is incorrectly accepted. This must be repaired alongside native transport; native signature work alone is insufficient.

Uncommitted record-layout work now uses each checked closure or method’s explicit parameter binding list. Closure binding tables retain outer bindings, including outer parameter flags, so neither a zero-based arity boundary nor the parameter flag alone identifies actual closure parameters. Sixteen record-view unit checks pass. A strengthened control separately verifies an actual offset parameter and a retained outer parameter that must not receive an entry view. The native installer still has its original three errors: one missing replacement bound and two concrete-record versus record-view mismatches. Both installer controls remain red. No acceptance, consumer refresh, broad regression or timing is claimed. Closed logs named `tsrust-b7-closure-*` are verified in SQLite.

Next, represent closure parameter/capture identities explicitly in retained-borrow and replacement summaries. Carry those summaries through source invocation, then build native closure signatures and concrete owner transport from the same identities. Do not encode a closure-local binding as an ordinary named-function owner. Finish the source retained-new-owner control together with the existing released-old-owner native witness before acceptance.

### Closure-body source summaries checkpoint

The retained-new-owner control now passes. `record_callable_identity.rs` gives named functions and closures distinct identities and exposes each callable’s actual parameter binding IDs. Retained-borrow graph propagation maps callee binding IDs through written argument slots, then exports source-slot summaries. Replacement replay uses the same identity and mapping, including closure calls reached from named relays. Invocation applies release before retention. Complete replacement releases the previous owner; conditional replacement conservatively retains it. No native owner namespace is extended or fabricated.

The new editor witness agrees with compiler diagnostics. Ninety-six integration cases pass with the native closure installer witness excluded; forty borrowed unit checks and eight depth canaries pass. The strengthened source control covers direct closure invocation, a named relay containing a closure, conditional replacement, and formatted source. Whole-project formatting is stable with zero file or fingerprint changes. Native closure signatures, capture-sourced stores, abstract calls and concrete closure owner transport remain pending. Existing matched workload support is unchanged; this source-analysis checkpoint adds no native capability or measurement claim.

### Native ordinary closure installers checkpoint

The native closure-body installer witness now passes. `NativeSignaturePlan` separates signature rendering from named-function inputs. Closure signatures supply actual parameter binding IDs, nominal record views, retained-input lifetime groups and callback replacement obligations through the same renderer. Generic invocation adapters carry matching conversion bounds and neutral marker arguments; they cannot manufacture a concrete caller owner. The previous 41-observation workload emits byte-identically before extension: 345,109 CLI bytes, SHA256 `052c93aabdf8c4bc731c1e392fb2f911f8cddbf1451e78e6bfb26103b450d697`.

Native owner planning follows ordinary closure invocation contexts and maps source arguments into actual closure bindings. Concrete owners remain named caller storage. Closure-local storage is not encoded as a named function ID. Captured, escaping, polymorphic, stored and aliased installer uses retain conservative classification. Stable readonly aliases within a supported installer preserve their parameter marker. Shared replacement reification narrows stored families after successful calls, releasing the old native borrow. Checked closure release and borrow summaries are retained for emission.

Ninety-eight affected integration cases pass after resuming the failed and unrun groups. Native variants cover readonly and mutable callbacks, plain and nullable fields, readonly aliases, exact effects and typed failure after storage. Forty existing borrow units pass; the new concrete-owner/alias-guard unit passes separately after correcting its owned-transfer example. The editor witness and eight depth canaries pass. Invalid exploratory mutable local aliases were rejected by existing parameter/access rules; those failed attempts are preserved, and mutable native coverage uses direct parameters.

The matched workload now observes 45 values per iteration, adding a closure-body wrapper around the saved conditional family installer. Five cases pass on native and Node-after-tsc without timing. Oracles, contracts, guards, registrations and source hashes are current. SQLite record `runtime/results/2026-10-09T21-34-16-612Z-verify.json`, 622 exact input/text records under `runtime/source-evidence/closure-owner-transport/`, and 19 closed logs are verified by exact readback. Five disposable workload outputs are removed. B7 remains open; captured inputs, closure-local owners, abstract invocations, consuming wrappers and the remaining plan rows still need complete interfaces.

### Captured installer effects checkpoint in progress

Captured mutable effect state and copied scalar state can accompany retained explicit input parameters through direct ordinary closure calls. Source summaries now distinguish argument borrows from hidden captured-input borrows. Captured-input stores attach their loans to the destination record independently of the installer value, preserving conflicts after installer disposal. A named clear helper releases those loans. Native planning keeps captured retained inputs and captured record storage conservative; effect-only captures no longer block explicit-input owner replacement. B7 remains open. Captured owner markers, closure-local owners, transitive hidden capture interfaces and the remaining composition rows are not accepted by this checkpoint.

### Borrowed captured-source native signatures

Ordinary closure signatures now include stores whose source is a borrowed closure capture. Conversion bounds use the existing capture lifetime from the closure environment, while explicit parameters retain their own input lifetimes. Capture indices and argument slots remain separate. The record parameter uses its view contract, and generic invocation adapters carry the same conversion requirement. Empty retained summaries return before capture layout work. Native witnesses cover nullable fields, plain fields, installer disposal and named clearing; a local alias that transfers ownership into a shorter block remains an escape error.

This does not complete hidden owner provenance. Captured-source origins remain conservatively excluded from distinct named-owner carrier planning. Ownership-taking captures need a receiver storage lifetime. Transitive hidden sources, captured targets, callback capture vectors and the remaining B7 interfaces remain open. The matched workload adds stored-after-disposal and cleared-source observations, bringing its row to 51 values. Measurement and family acceptance remain pending.

### Independent captured installer owners

Direct ordinary installer calls now transport retained captured-input owner markers. The hidden source binding is distinct from a written argument slot. Signature bounds, named installer forwarding inside the body and post-call replacement selection carry matching markers. Captured-input origins are no longer suppressed solely for being captured; consuming wrappers, unknown captured/parameter receivers, owned captured storage and existing abstract/escape guards remain conservative.

Native witnesses cover replacing callbacks from independent captured arrays after both installer blocks end, with nullable and plain fields. Source and editor controls reject mutation of the retained replacement owner. The matched workload observes first installation, released old input and replacement input, bringing its row to 54 values. Explicit closure-local storage identities and remaining B7 interfaces stay pending.

### Closure-local owner scopes

Native owner identity now contains an explicit function or closure scope and binding ID. Named marker names remain stable; closure-local storage uses its own marker namespace. Provenance, owner type lookup, payload alternatives and replacement selection use that scope. Unknown captured bindings remain unresolved instead of becoming fabricated local owners.

The simple closure-local store already executed through direct-store lowering. Adding installer side effects produced the genuine native lifetime witness. The explicit scope fixes that witness. Nullable/plain readonly fields and nullable mutable fields execute with exact installer effects. Composing nested captured installers exposed unrelated counter loans being retained by the holder. Capture payload attachment now records direct loans on their individual capture fields; retained captured sources select their field path and preserve the conservative fallback for unknown paths. The composition case then passes without replaying initializers or relying on parser closure IDs.

The matched workload adds four observations from an effectful nested installer composition over closure-local arrays, bringing each row to 58 values. Independent oracle, source guards and matched TypeScript support remain untimed. Mutable retained captures, owned capture storage, hidden transitive interfaces and the remaining B7 rows still require acceptance.

### Retained mutable captures and explicit once transfer

Reusable mutable installers now lend captured exclusive inputs for their receiver lifetime. Source loans retain the receiver until the stored callback is cleared, preventing another mutable invocation or disposal while the callback remains live. Native signatures use the same receiver lifetime in replacement bounds. Readonly captures retain their original environment lifetime. The scoped witness invokes, clears and reinstalls twice, then mutates the source after installer disposal.

The ownership-taking witness uses an owning `Function<int>` field and `Owned<int[]>` installer parameter with `Once<Function<void, [target: Mutable<Holder>]>>`. It transfers the capture into the stored callback and invokes it twice after consuming the installer. A borrowing `Mutable<Function<int>>` field is a different contract: it cannot borrow a callee-owned parameter beyond the call. A new red witness exposed nullable injection bypassing the existing escape eligibility check; the check now examines the injected callback environment. Abstract consuming wrappers that retain borrowed inputs remain pending; this owning witness does not establish those interfaces.

The matched workload adds six observations: two scoped callback results, released source state, exact installer effects and two owned callback results. Each row now contains 64 values. Family acceptance and timing remain pending.

Focused acceptance for this checkpoint totals 107 affected integration tests, eight owner-provenance units, 27 capture-loan units, one editor witness and eight depth canaries. Five matched native and Node-after-tsc cases pass untimed. The final registration control requires both scoped installer calls and rejects changes to every new observation. Whole-project formatting changes one workload and its fingerprint, then zero files. SQLite preserves both correctness records (`2026-10-09T22-33-57-820Z` and `2026-10-09T22-34-26-050Z`), exact compiler/harness inputs, generated text and failed-attempt logs. Source hashes, trimmed emission and instrumented prefix are verified. Five disposable outputs totaling 1,092,118 bytes are removed. No full regression, timing or consumer refresh is claimed.

### Transitive locally constructed hidden sources

Retained-borrow summaries now propagate a callee's captured source through a direct call when the caller constructs that callee. The collector records local closure origins during its existing expression walk. Written arguments map through parameter slots; hidden sources map from the callee's inner capture binding to its outer construction binding. The mapping is rejected for foreign construction contexts rather than interpreting an unrelated binding table. No second expression walk is required.

The source red witness exposed a missing retained borrow. Restoring it exposed a separate old-owner release failure: release replay classified captures from the enclosing closure's explicit inputs as untransported because it inspected only written parameters. Eligibility now includes the callable's checked captures as well as parameters. Native signatures and concrete owner transport then work through the existing retained summary interface. No emitter change, lifetime cast or clone is needed.

Native assertions cover nullable and plain fields and three nested installer layers with exact effects. Source and editor controls retain the new input and preserve the previous input across conditional replacement. The matched workload adds four nested observations, bringing each row to 68 values. Captured callee receivers, abstract calls and the broader hidden environment interfaces remain pending; this checkpoint establishes locally constructed callees only.

Focused verification totals 110 affected integration tests, eight owner-provenance units, 27 capture-loan units, one editor witness and eight depth canaries. Registration controls and all five native/Node-after-tsc cases pass on the formatted 68-observation workload. SQLite record `runtime/results/2026-10-09T22-44-14-386Z-verify.json` and 652 exact source/text/log entries under `runtime/source-evidence/nested-hidden-installers/` preserve the source red, native red, release failure, repairs and final checks. All input hashes, trimmed emission and generated prefix match. Five disposable outputs totaling 1,126,667 bytes are removed. The shared Cargo target is 2.2 GiB with no incremental entries. No complete regression, timing or consumer refresh is claimed.

### Captured installer receiver source interface in progress

Two new witnesses expose the next boundary: an outer closure captures an installer created in another context, then calls it to store a callback borrowing that installer's data. The source mutation control was incorrectly accepted, and the executable witness failed native record-view/signature obligations. The source control now passes; the native witness remains red and is not accepted.

Retained inputs carry an explicit `Projection`: direct binding, checked capture path or opaque environment. Foreign call edges use the checked `CallTarget::Closure(binding)` receiver, not the name expression's binding (which can be absent). Source summaries prepend the callee's capture binding to the receiver projection rather than reinterpreting outer IDs. Capture loan paths select the borrowed data separately from unrelated installer effects. Unknown or cyclic paths become opaque and retain the receiver conservatively; opaque paths never become fabricated direct inputs. A focused unit checks this convergence property. `retained_capture_projection.rs` isolates the representation and checked traversal.

Native closure signatures explicitly keep projected environments pending. Do not choose the first lifetime of the captured receiver: its own borrow and its nested data captures have different lifetimes. Native acceptance still needs projected lifetime selection, projected owner keys (including distinct fields of one receiver), complete replacement/reification and matching generic conversion obligations. Existing foreign-callee provenance guards remain. The matched 68-observation workload is unchanged and emits byte-identically; no new native capability or measurement is claimed.
