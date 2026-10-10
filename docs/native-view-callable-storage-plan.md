# Native views in owning callable storage

This is an implementation dependency of gate 189, not a new ownership policy.
A callback may own its environment while that environment contains a view borrowed
from another owner. Owning the callback must not imply that its payload is static.
The original owner's lifetime must follow the callback through storage and calls.
Public byte view factories remain guarded until the complete integration is ready.

## Current evidence

The private source witness in stdlib/bytes/views_stored_emit_tests.rs constructs
`Holder { read: Once<Function<Window>>, marker: int }`, where Window is projected
as the native readonly byte view. Its factory returns a callback consuming the
captured view. The caller supplies two independent inputs and selects the second.
Analysis originally rejected construction in StorageContract::environment.
The normal source construction checkpoint below turns that retained witness green.

A separate mixed record containing a direct view and an ordinary callback passes.
Its complete emitted code runs at O0/O3, preserves the selected input pointer after
an unrelated owner ends, and independently rejects selected-owner escape. Thus
ordinary record wrapping is not itself the missing dependency.

StorageContract::environment rejects closure.borrowed. Family::checked_storage
also requires lifetime_count zero. These guards protect the existing owning family
representation, whose identity and native ABI do not retain payload lifetime slots.
Removing either guard alone is unsafe and does not implement this feature.

## Implementation sequence and gates

1. Define typed scoped owning family metadata in callable_storage.rs, separate from
   the existing borrowed invocation family. Retain the original Owned/OwnedOnce
   invocation contract and finite concrete alternatives. Describe each alternative's
   carried native-view paths and lifetime slots. Do not reclassify physical shared
   or mutable captures as owned storage. Add unit tests distinguishing an owned
   view payload from an environment borrowing local callback storage. Existing
   wholly owned families must retain their current identity and ABI.
2. Add an isolated native callable provenance helper. Construction must transport
   each alternative's capture loans into family storage without retaining factory
   binding IDs. Extend parameter shape seeding and function return summaries to
   these typed slots. Call summaries must resolve the actual callee expression,
   including record fields and array elements, and join possible alternatives
   conservatively. Validate consumed invocation, replacement, moves and disposal.
   Preserve both the path of stored data and the identity of its original owner.
3. Add a dedicated native callable lifetime emitter. Render actual Rust lifetime
   parameters on the finite environment family and every containing record,
   array and Result. Variant payload types must use the right slots. Connect
   argument and return signatures to the existing named native lifetime planner.
   Keep real slices; no static lifetime, raw pointer, erased handle, hidden owner,
   clone or shared allocation may stand in for lifetime transport.
4. Turn the retained positive witness green through normal analysis and complete
   emission. Execute O0/O3 and independent owner-escape rejection. Add two distinct
   alternatives with different input origins, a field move, field replacement,
   an array element, a nullable field, returned storage and nested capture. Add
   mutable view cases: consuming a callback may transfer its carried exclusive
   view, but cannot return a borrow of storage destroyed by consuming invocation.
   Preserve ordinary local reference-capture rejection and source diagnostics.
5. Connect editor checks for the supported public syntax, including ownership
   diagnostics in imports and incomplete buffers. Record private implementation
   acceptance separately from public byte-view publication. Complete public
   factory/observer catalog entries and executable examples before lifting guards.
   Add matched subarray workload support and its independent oracle as specified
   by native-byte-views-implementation.md. Full regressions and measurements belong
   to S07 completion, not each representation increment.

## Bounded acceptance

Run the smallest failing witness after each coherent change, then the private
native-view batch, relevant callable storage tests and native editor contracts.
Analyzer dispatch changes require all ten normal-stack depth canaries and Result.
Format the entire project before source checkpoints. Preserve original failed and
successful logs byte for byte in SQLite and verify their restoration. Keep public
publication guarded while any required representation or provenance case is open.

## Carried-slot metadata checkpoint

native_view_callable_storage.rs now derives candidate lifetime slots from checked
capture bindings. Each slot retains the capture field, its relative payload path,
Result variant guards and readonly/exclusive access. Construction-context root IDs
and source spans are not part of this layout description. Physical shared/mutable
capture storage, physical storage loans and foreign roots are excluded. Slots are
candidate metadata only; they do not certify that a family can safely store the
value, and finite alternative identity/ABI integration remains pending in step 1.

The owning-storage guard now identifies this case with native-view-callable-storage
rather than claiming the callback fails to own its environment. Existing ownership
and invocation guards remain. Focused controls cover readonly/exclusive payloads,
nested record fields, Result success guards and the retained publication guard.
The positive stored callback witness remains red and must not be counted as accepted.

## Finite family planning checkpoint

The candidate planner now retains the structural callable signature, the original
invocation mode and a sorted, deduplicated list of concrete alternatives. Each
alternative has its own carried slots; capture field numbers are not merged across
different origins. A reusable contract cannot acquire a consuming environment.
Unknown origins are diagnosed, and physical references or unresolved layouts fall
back to the existing storage checks. This is a finite layout description, not yet
a registered source storage type or an emitted native family.

callable_dispatch.rs consults this planner only for families containing a borrowed
environment. Wholly owned ordinary families keep their existing path without an
additional candidate allocation. Candidate native-view families remain rejected
with the pending lifetime-storage diagnostic. Registry identity integration, actual
callee loan transport and lifetime-bearing family emission are still required to
turn the retained positive stored-callback witness green.

## Prepared registry integration checkpoint

callable_storage::Registry now retains candidate native layouts in an optional,
separate native_view_family_registry arena. Prepared IDs cannot be used as source
StoredCallable IDs. Preparing a native alternative still returns the unsupported
storage diagnostic and does not populate the ordinary family's admitted origins.
The original structural contract and invocation mode remain part of each layout.
Repeated preparation canonicalizes equal layouts and accumulates distinct candidate
alternatives without merging their slot identities.

Closure compaction now validates prepared origins even when no ordinary family
origin was admitted. A failed mapping leaves all metadata unchanged. A successful
mapping moves each alternative's capture paths and access mode with its origin,
then reorders alternatives canonically. Ordinary programs allocate no prepared
arena. This integrates finite layout identity into the registry but does not yet
create an admitted scoped owning source type. That type, actual-callee provenance
and lifetime-bearing native emission remain prerequisites of positive acceptance.

## Actual callee transport checkpoint

Concrete expression calls now register native return summaries through an isolated
native_view_expression_calls helper. ConditionalClosure targets retain their known
origin, and specialized expression methods retain their resolved method origin.
The iterative value walker evaluates the actual callee expression before selecting
its capture field. A record field or conditional callback therefore transports the
selected environment's loans rather than looking up a factory's foreign binding.
Named calls retain their binding from the checked call target when the callee name
itself has no binding annotation. A missing returned capture slot is diagnosed
instead of silently dropping its source lifetime.

A checked-expression field control verifies source identity and missing-slot
rejection. A separately parsed source program chooses between two concrete callbacks
carrying different input views. Normal analysis retains both possible origins;
complete native emission runs at O0/O3 with both pointer identities and independently
rejects selected-owner escape. These are transport prerequisites. Prepared owning
family lookup, family variants, source storage admission and native family emission
remain pending, and the original stored-callback positive witness remains red.

## Private one-shot native family ABI proof

native_view_family_emit.rs now emits a finite enum whose variants contain the
normally emitted concrete environments with independently named view lifetimes.
Its consuming call method joins only the source lifetimes needed by each returned
payload path. The shared native lifetime planner names these output intersections;
normal function and closure emission uses the same output planning helper.

This backend is test-only until source family admission and provenance integration
are complete. It is not an emitted source storage feature. The initial proof covers
non-fallible one-shot families with scalar arguments and views carried by captures.
Borrowed invocation arguments, fallible families and reusable families still require
their planned ABI work. Invalid invocation, mismatched slot counts and missing return
sources are diagnosed before appending generated declarations.

The private test uses parsed, formatted factories and normal native closure emission.
The additional family enum is constructed through the typed candidate layout. Both
readonly and exclusive alternatives execute at O0/O3, preserve selected pointer
identity, and exclusive writes reach the original buffer. Independent native
compilations reject owner escape, a second consuming call, and conflicting owner
mutation while an exclusive result remains live. No copy, shared owner or erased
pointer replaces the slice payload. This proves the initial backend representation,
not public API or stored-callback acceptance; the original source witness stays red.

## Result payload ABI control

The private consuming-family proof now covers `Result<Window, int>` from both
alternative factories, for readonly and exclusive views. Success preserves the
selected buffer pointer and exclusive writes reach that buffer. Error preserves
its integer payload without requiring a view result. Independent native compilation
rejects a success payload escaping its owner. All branches execute at O0 and O3.
Factory and closure identities are taken from the checked program rather than
assuming source declaration order survives specialization. This extends backend
evidence only; source storage and the public view factories remain guarded.

## Independent record fields and slot validation

The private ABI now executes callbacks returning two view fields with independent
owners. Either concrete alternative can be selected; the first field remains usable
after the second field's owner ends. Readonly and exclusive controls preserve both
pointers at O0/O3, and exclusive writes reach the surviving owner. Returning the
short-lived second field instead is independently rejected by Rust. This verifies
that output paths do not collapse distinct field lifetimes into one intersection.

A malformed candidate with the correct slot count but incorrect access previously
passed emitter validation. The emitter now compares complete capture slots against
the checked environment before writing declarations, including capture identity,
payload path, variant guards and access. The failing control and repaired results
are retained in SQLite. Source storage admission remains the next integration task;
these private ABI controls do not close gate 189 or S07.

## Scoped owning storage identity

StorageContract now distinguishes ScopedOwned from lifetime-free Owned and borrowed
invocation families. The isolated native_view_storage_registry helper validates a
finite candidate before interning an identity containing its structural contract and
canonical concrete origins. Different alternatives produce different identities;
repeated alternatives canonicalize. Invocation remains consuming or mutable according
to the original contract. Physical reference captures are not admitted by this path.
The registry stores the complete native layout separately from ordinary owning
origins, and compaction updates identities and layouts atomically after validation.

Native storage reachability now includes closure captures and stored-family
alternatives in its reverse graph, alongside records and unions. A containing record
therefore retains the fact that its callback carries views. Cycles terminate without
recursive traversal, and ordinary empty families do not acquire borrowed lifetimes.
This is a source representation prerequisite, not source construction acceptance:
normal construction remains guarded until family provenance and emitted lifetime
signatures are connected. The retained stored-callback source witness remains red.

Reachability also checks each record, union, closure and family ID against its own
range before translating it to a graph index. A retained failing control showed
that an invalid closure ID could otherwise alias a live family node. The repaired
control preserves typed identity across the combined graph.

## Scoped family provenance transport

Scoped owning parameter shapes now traverse each concrete alternative and its capture
fields. CallableAlternative is a typed lifetime-path segment, separate from user
record fields and Result record guards. Each view slot retains its own source path,
access mode and owner identity. Parameter seeding covers multiple fields and both
readonly and exclusive views.

The isolated native_view_family_provenance helper records family return summaries
from checked closure bodies. Consumed capture payloads are transported through the
actual callee's alternative and capture path; invocation arguments retain argument
provenance. Named and expression field calls consult the same summaries. A borrow of
receiver-owned storage remains explicitly diagnosed until its receiver ABI is ready.
Callable construction wraps capture loans with the concrete alternative while keeping
the original source roots. An inactive alternative does not create loans or reject a
valid construction. This is tested through checked expression transport; normal source
owning construction remains guarded until lifetime-bearing emission is connected.

Closure compaction remaps alternative IDs in binding loans, function return summaries,
prepared slot paths and Result guard paths. native_view_origin_remap keeps record
variant IDs and owner binding IDs unchanged. A missing alternative mapping leaves an
individual loan unchanged and fails before a prepared registry clone is published.
The original stored-callback source witness remains red. Gate 189/S07 are open.

## Normal native family emission

The lifetime-bearing family backend is now part of the production emitter. Normal
program emission selects it for the distinct scoped owning registry identity, while
ordinary owning and borrowed-invocation families retain their existing paths. Native
construction selects the concrete enum alternative without copying its environment.
Scoped family types render lifetime arguments in both ordinary type positions and
named input/return signatures, using the typed alternative/capture source paths.

The readonly and exclusive family controls now install checked scoped metadata and
use complete normal program emission; they no longer append a separately emitted
family declaration. Both alternatives run through a native forwarding function with
explicit named input and output lifetimes at O0/O3. Pointer identity, write-through,
owner escape, second invocation and conflicting access controls remain intact.
This connects emission to checked metadata, not source construction: the frontend
still guards normal owning construction. Nominal record generic positions, source
conversion and supported ABI validation must be connected before that guard is lifted.

## Nominal record generic positions

The isolated native_view_record_parameters index now derives generic field positions
from native-bearing instances and keeps those positions on their nominal declaration.
Record definitions, ordinary type rendering, named lifetime signatures, record arrays
and environment rendering consult the same field decision. A scoped owning instance
can therefore fill its field with a lifetime-bearing family without changing the
written owning contract or marking ordinary owning families as borrowed.

Declared-family discovery skips generic record fields. Unused symbolic lifetime-free
families no longer emit invalid native-view result signatures solely because a nominal
field was written with an owning callable annotation. Populated families and concrete
function storage remain discoverable. Discovery uses direct iteration without a
per-record temporary vector.

The normal-emission control now includes a nominal Holder record, a materialized scoped
owning field and a named native forwarding signature for that record. Both alternatives
run at O0/O3 for readonly and exclusive views. The initial record control failed on the
nominal field type and an unused symbolic family; repaired controls retain pointer,
write-through, escape, second-call and conflicting-access assertions. Normal source
construction remains guarded; the original source witness remains the next target.

## Normal source construction checkpoint

The original stored-callback source witness now passes normal analysis and complete
native emission. A returned Holder carries a distinct scoped owning callback family;
its nominal field is specialized without turning the ordinary owning family into a
lifetime-bearing type. Signature compatibility accepts owned carried-view captures,
while ordinary storage admission still rejects lifetime erasure and physical borrowed
captures. Construction interns the scoped identity and transports its capture loans.

The original two-input relay preserves the second input's pointer through a stored
Once callback at O0/O3. The unrelated first owner can end, while generated Rust
independently rejects escape beyond the selected owner. Backend eligibility is checked
before code generation: data arguments, missing returned capture lifetimes and throwing
callbacks still receive source diagnostics. These are intermediate implementation
limits, not permanent language restrictions. Public factories remain guarded.

Next extend normal source coverage to alternative families, moves, replacement,
arrays, nullable fields, nested capture and exclusive views. Preserve the existing
physical capture guards and extend each diagnosed invocation ABI rather than erasing
lifetimes to make it fit. Public observer/catalog and matched workload acceptance,
full S07 regressions and measurements remain pending.

## Source storage matrix checkpoint

Normal source analysis and complete emitted execution now cover readonly and
exclusive views in moved callback fields, saved owning callbacks, single-element
callback arrays removed with pop, present nullable callback fields, nested concrete
captures and direct record replacement. Replacement after moving the previous field
also works. The old live fields are dropped after evaluating the replacement, and
new native storage infers independent lifetimes. Already transferred fields are not
dropped again. Self-dependent replacements retain ordinary assignment behavior.

Finite alternatives now join through a scoped owning family in conditional fields
and returned records. Source families retain their concrete environments and convert
to the joined enum by moving the selected alternative, without copying captured
bytes. Readonly and exclusive returned-record alternatives execute both branches.
Owner pointer identity, write-through behavior, unrelated-owner release and selected
owner escape rejection are checked at O0/O3. Consumed callbacks remain unavailable.

Structural storage contracts, nullable injection and direct replacement epochs live
in dedicated helpers. Array pop/shift transport the element's carried loan paths;
carrier conversions preserve those paths. Native type rendering includes scoped
callback arrays. Concrete-family discovery follows reachable union storage roots,
rather than emitting lifetime-free families referenced only by symbolic generic
record fields.

This is focused implementation evidence, not public byte-view acceptance. Mixed
alternative arrays, absent nullable values, stored-family captures, field and
control-flow replacement joins, and the broader invocation/error ABI still need
coverage and implementation. Public factories, shared observer contracts, complete
IntelliSense examples and matched subarray workloads remain gated on the full matrix.

## Invocation ABI and replacement checkpoint

The family backend no longer limits stored view callbacks to scalar arguments, view
results and non-throwing bodies. `validate_abi` kept those limits only as guards; the
emitted `call` method already forwards each argument with its ordinary native type.
Now supported through normal analysis and native execution, for readonly and
exclusive views at two optimization levels:

- arguments that own data (`Owned<string>`) or borrow it (`ReadOnly<string>`), and an
  owned view argument that the callback does not return;
- optional, default and rest parameters;
- callbacks that return `int` or `void` while they own a captured view;
- throwing callbacks. A family with a throwing alternative uses the error layout
  planned for its identity, and alternatives that cannot throw are adapted to it. A
  caught failure consumes the callback and leaves the other owner usable.

Replacement now follows what Rust does with one native variable. Rust releases the
earlier owner of a reassigned local, so conflict checks stay precise. A lifetime in a
function signature is different: it is unified over every value the variable ever
holds. `retain_unified_native_view_loans` therefore keeps the replaced origins as
dormant loans (`retained: false`). They do not block their owners and they still
reach a returned value, so the function signature names every possible input.

- A replacement in the block that declares the local redeclares its native storage
  and starts a new lifetime. This now also applies to a direct field of a local
  record, including a new callback alternative and an absent value.
- A replacement in a nested block, a branch or a loop assigns to the same variable.
  The result then depends on both inputs, and generated Rust rejects a caller that
  ends either owner too early.

Still open: a new callback alternative stored into a field inside a nested block (it
needs a carrier type joined before the declaration), arrays that receive alternatives
by `push`, absent-to-present nullable transitions, captures of stored families, results
that borrow an invocation argument, and propagation of a thrown error through the
test harness relay.

## Arrays, captured families and reusable invocation checkpoint

Now supported through normal analysis and native execution, for readonly and
exclusive views at two optimization levels:

- Arrays of stored view callbacks with different alternatives: a literal with several
  closures, `push` into an empty or populated array, and elements moved in from
  another array. The array takes the family that joins every alternative. A post-pass
  (`native_view_array_carriers`) moves earlier values into that family, and `push`
  adds the origins of the new element to the array (`append_native_view_element`).
- A closure that captures an already stored scoped family, directly, through a
  record, or to build a new stored callback. Lifetime counting, environment type
  naming and closure signatures now traverse `StoredCallable` scoped families with
  one lifetime per carried slot in alternative order.
- Reusable stored callbacks. A family whose contract is not single-use is called
  through `&self` or `&mut self`; it may read a captured view, keep mutable state and
  take data arguments. A reusable family cannot hold an environment that its call
  consumes.

A lifetime-free family without environments whose contract mentions a byte view is no
longer emitted: it cannot be instantiated, and its `call` could not name a lifetime.

Safety net: a returned direct byte view without a tracked owner is now a source
diagnostic (`native-view-source`). Before, a callback that returned a view received
as an argument passed analysis and the generated Rust did not compile.

Still open, each with a pending row in `zz_matrix_probe`:

- absent-to-present transitions of a nullable callback field whose record was built
  without a view, and a conditional that selects a callback or `undefined`;
- a new callback alternative stored into a field inside a nested block or branch;
- a callback result that borrows an invocation argument (closure parameters are not
  seeded with view provenance);
- recursive view-containing storage.

The first two need the local's final storage type before its declaration is emitted
(a flow-insensitive join like the nullable record carriers of gate 182).
