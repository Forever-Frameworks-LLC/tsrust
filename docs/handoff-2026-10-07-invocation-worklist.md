# Handoff: contextual callable effects and pending return snapshots

Read AGENTS.md, docs/development-workflow.md and
docs/captured-call-effects-plan.md and docs/callable-environments-plan.md first. Gate173 and the full callable-effects
plan remain open. This is a source checkpoint, not family acceptance. Copied
executable consumers remain at 399a1c4; the warm Cargo target contains newer work.

## Current implementation

Compiler generic-call validation tries converged contextual invocation facts
before its existing structured/conservative paths. All eight original captured
write scenarios now reject with related arity diagnostics. Opaque requests retain
fallback behavior, which still needs stronger effects for unknown regions.

The body index tracks lexical BindingId values, named/arrow bodies, parameters
and defaults. A dedicated aggregate-eligibility module caches aggregate-write
flags once per body. Contextual invocation reads the flag instead of rescanning
source. Bodies that mutate aggregate contents remain opaque because shared
storage identity is absent. No measured speed improvement is claimed.

Expression dispatch saves the callee before arguments. Normal and exceptional
states remain distinct. The finite worklist keys requests by declaration, caller
bindings and argument Values, starts new requests at bottom, and schedules reverse
dependencies. Dispatch queries/enqueues requests without recursively solving them.
Opaque dependencies propagate transitively, including cached roots.

Parameter entry installs supplied arguments and evaluates defaults left to right.
Missing and explicit undefined arguments execute defaults; uncertain undefined
selection joins both branches. Earlier parameters, captured writes and failures
survive entry. Missing required arguments retain diagnostic facts; normal arity
validation rejects the call. Rest entry packs trailing snapshots into flat element
facts. Arrays, aliases and frame projection retain these facts. Equal-length joins
merge positions; differing shapes become unknown. Constant indexing selects an
individual callback. Unknown indices and writes invalidate provenance. Shapes do
not recursively nest, and spread is not implemented by this domain.

Fixed-array forEach dispatch preserves the saved receiver, sequences callback
requests and propagates exceptional completion. Empty arrays invoke nothing.
Callbacks receive element, finite index selection and an array snapshot. Supported
console methods on the unshadowed global namespace are leaf calls: they preserve
lexical slots, return undefined and retain a conservative exceptional exit.
Unknown methods and shadowed namespaces are ineligible.

Summary outcomes now retain returned Values separately from outgoing bindings.
The iterative graph runner carries pending return snapshots through normal finally
cleanup, replaces them on overriding returns and clears them on exceptional
transfer. Worklist outcomes join return Values and provide them to callers.
Fallthrough contributes undefined. Flat returned arrays preserve element facts.

## Verified evidence

Latest checks pass 34 focused tests, 191 bounded compiler tests and the whole
329-test editor crate. All eight depth canaries ran unfiltered. The compiler batch
explicitly skips literal_guard_native_callable_replacement_has_common_carrier.
The separate conditional-arrow common-carrier red also remains open.

Return witnesses verify wrong callback arity, the snapshot before a finally
assignment, overriding return, distinct returned-array positions, throwing cleanup
canceling a return and an outer catch replacing it. Console witnesses preserve
writes, restoration and uncalled declarations. Previous invocation, alias,
rest/default, recursion, completion and ownership checks remain intact.

Whole-project formatting changes zero files and fingerprints. Exact red and
passing logs are imported into benchmarks/records.sqlite and verified by reading
back original bytes before loose copies are removed. No new benchmark observations,
timings or executable-consumer refresh is claimed. Completed compiler test binaries
are pruned after checks; reuse the one warm non-incremental Cargo target.

## Next work and acceptance gates

Named and arrow frames now reject missing captured binding identities. Named
requirements come from recursive-group capture plans; arrow requirements are
indexed from direct lexical uses and parameter defaults. Missing producer slots
or unrelated same-named slots make invocation opaque. This is necessary but not
sufficient: repeated factory instances can share source binding IDs. Implement
contextual environment identity and lifetime projection next; a body span or source
ID alone must not certify the correct escaped instance. Add paired factory witnesses
before implementing this. Preserve existing ownership rules and original
owner disposal. Do not introduce shared JavaScript capture cells or implicit clone.

Then finish opaque-region effects, shared aggregate storage, precise mutation,
spread, other collection methods and optional invocation regions. Unknown callbacks
must not become effect-free calls, and fallback must not discard known invoked
writes. Complete the retained native carrier repairs. Return snapshots alone do
not prove owned factory or recursive-group lifetime consistency.

Before acceptance, complete editor/guide and matched benchmark support, run bounded
compiler/native checks and the whole editor crate, update milestone/coverage/roadmap
records, import/read back SQLite evidence and commit. Keep unmeasured workloads
unmeasured. The generic-value-adapters family has existing untimed support; this
checkpoint adds no observations or measurement claims.

## Worktree constraints

One agent works here at a time. Preserve all fifteen protected owner fixture edits;
never stage them. They are under array-callback-errors, array-fields,
array-from-collections, comma-contexts, generic-value-adapters, interfaces,
map-for-each, map-group-by, set-algebra, sets, string-iteration and type-assertions.
Inspect Git status and use explicit Git paths only. Format the whole project before
each source checkpoint with the current verified compiler. No archives or snapshots.

Use CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target,
CARGO_INCREMENTAL=0 and offline locked Cargo. Node is
/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. Refresh executable consumers
atomically through .refresh-tmp and rename when needed; never overwrite live inodes.

## Capture availability guard

Retained failing frame witnesses exposed missing producer slots for both named
callbacks and arrows. The body index now caches required capture IDs. Named
recursive groups exclude their own member owners; a dedicated arrow requirements
module separates owned parameter/local slots from external direct uses and
defaults. Nested arrow bodies have their own requirements. Frame entry refuses
missing IDs before installing parameters. A live producer frame remains eligible;
unrelated same-spelled parameter slots do not substitute for the producer.

All 36 focused tests pass. Earlier bounded compiler checks pass 191 tests before
the arrow guard, followed by final affected compiler/canary and editor checks.
Source capture availability is not instance identity or ownership transfer. No
factory-effects acceptance, native carrier repair, timings or consumer refresh is
claimed. Exact red and corrected evidence is retained in SQLite.

Final capture-availability checks pass 76 compiler tests, including all eight
unfiltered depth canaries, and 197 editor intelligence/stdio tests. The retained
native-carrier red is explicitly excluded. Formatting changes zero files/hashes.
Closed logs are imported and read back in SQLite before deletion; completed
affected compiler test executables are pruned.

## Paired factory red and environment plan

A new unskipped focused test, separate_factory_instances_preserve_captured_callable_contracts,
now fails for both named and arrow factories: the returned selector loses its
producer environment. Each factory captures a different callable contract and
must return the correct one without merging instances. The TypeScript reference
in fixtures/callable-environments/reference.ts passes under the pinned Node.
The other 36 focused tests pass with this new red explicitly excluded.

No compiler implementation is changed in this checkpoint. The dedicated
callable-environments plan defines correlated body/environment alternatives,
finite allocation keys, live borrowing versus owned escape, callback entry and
projection, recursion convergence and lifetime/native acceptance gates. Implement
that plan next; do not weaken the missing-capture guard to make the new test pass.
No benchmark observations, timing, native Tsrust parity or editor change is claimed.

## Environment store kernel WIP

A dedicated store now interns source creation/producer-site capture facts into
stable handles. Records keep weakly joined binding state, correlated body/context
edges, explicit unknowns and change versions. Keys exclude handle edges; cyclic
records reuse addresses. Missing IDs, duplicate slots, malformed shapes and bad
edge references cannot partially mutate records. Four store controls pass, and
all 40 focused tests excluding the paired factory red pass.

No compiler dispatch or source behavior is changed by this isolated kernel. The
paired named/arrow factory red remains unskipped and unrepaired. Wire the store
into correlated Value/Binding/Element alternatives and producer call-site requests
next, including version-based reader invalidation without versioned allocation
keys. Capture ownership, strong update eligibility, lifetime/disposal and recursive
solver convergence still require implementation and tests. No native/editor parity
or measurements are claimed. Formatting changes zero files or fingerprints.

## Environment reader scheduling WIP

The environment store now records request-reader subscriptions separately from
allocation identity. A changed state or edge record queues each subscribed reader
once until the queue is drained. Unchanged and malformed merges queue nothing;
subscriptions to invalid handles are rejected. Reader notification does not add
versions or reader IDs to allocation keys. A failing stub witness preceded the
implementation, and the focused baseline passes 41 tests with the retained paired
factory red explicitly excluded. This kernel remains unused by Engine. Next connect
record reads to request subscriptions and drain changed readers into Engine's work
queue while wiring correlated value transport. No native or editor acceptance is
claimed for this isolated store change.
