# Recursive storage that holds byte views

Date: 2026-10-10. Status: array and nullable recursive view storage and capturing callbacks returning argument/owner views have focused acceptance; recursive callback fields and mutable-parameter replacement have focused acceptance; gate 189 has focused implementation acceptance. The owner approved parts A and B in the continuation chat.

The owner wants types such as:

```
type Chunk = {
  data: ByteView;
  next?: Chunk;
};
```

Two separate limits originally stopped this. Progress sections below record the completed increments.

## Limit 1: a record cannot contain itself directly (language rule)

`next?: Chunk` is rejected for every record, with or without views
(`recursive-layout`: "Direct record fields form an infinite layout. Use array
indirection to break the cycle."). Records are stored inline since milestone 25, so a
record that contains itself has no finite size. Recursion works today only through an
array field (`children: Chunk[]`).

### Part A: approved rule

Allow a field whose type is a nullable union (`T | undefined`, `T | null`, `name?: T`)
when `T` reaches the record that declares the field. A direct, non-nullable
self-reference stays rejected: it has no finite value.

- Meaning for the programmer: unchanged value semantics. The record owns the next
  record. Moving, cloning (a deep copy), comparing and dropping work as for any other
  nested record. Nothing is shared.
- Representation: the present variant of that union stores its payload behind one
  uniquely owned heap allocation (`Box`). The cost is one allocation for each present
  link. This is the first place where a record field allocates without an array, Map
  or string, which is why it is a language decision and not only an implementation
  detail.
- Scope: only unions on a recursive cycle are boxed. All other unions keep their
  inline layout, so existing programs and their measurements do not change.

Compiler work: `record_layout.rs` accepts the cycle through a nullable union and marks
the union variant as indirect; union construction, matching, narrowing, carrier
conversion, clone and inspection code add and remove the box at about fifty emission
sites; the formatter, editor and catalog need no change; tests cover construction,
traversal by recursion and by loop, replacement of `next`, clone independence, disposal
order and allocation counts.

## Limit 2: view lifetimes are named by path

Every byte view inside a type has its own native lifetime, named by the path to it
(`data`, `pair.left`, `items[]`). A type that contains itself has infinitely many
paths, so the emitter recursed without end. Since `cbdbd701` this is a source
diagnostic (`native-view-shape`).

### Part B: shared lifetime for a recursive record

A record on a recursive cycle that holds a view gets exactly one lifetime parameter.
Every view at every level of the structure uses it:

```rust
struct Chunk<'a> { data: ByteView<'a>, next: Option<Box<Chunk<'a>>> }
```

- Provenance paths stop at the cycle boundary: a loan of anything inside the
  structure is a loan of the whole structure.
- Consequence for programs: a chain cannot outlive the shortest-lived owner of any
  chunk in it, and an exclusive view inside the chain blocks every owner that the
  chain borrows. This is what Rust requires for the same type.
- Views outside the cycle (a sibling field that is not part of the recursion) keep
  their own lifetimes.

Compiler work: a cycle analysis for record types; a second record definition form with
a named lifetime instead of generic field parameters; the lifetime enumerations listed
in `handoff-2026-10-10-native-view-storage-complete.md` treat such a record as one
slot; loans truncate their path at the record. Part B is needed for array recursion
(`children: Chunk[]`) as well, so it can land before part A.

## Order

1. Part B with array recursion (no language change).
2. Part A under the owner-approved rule.
3. `Chunk` with `next?: Chunk` then needs both.

## Approved continuation

The owner approved nullable recursive links with unique ownership and the shared
lifetime design. This includes nullable links on cycles involving multiple record
types. Cloning remains available only when all contained types support it; this
does not make byte views clonable. Test allocation costs and long-chain behavior.

Explicit close remains the separate resource lifecycle contract: it consumes a
resource even on error, with nonthrowing scope-exit cleanup for unclosed resources.
Bytes and borrowed byte views do not require explicit close.

## Lifetime boundary groundwork

The shared iterative cycle index now identifies recursive record components through
record fields, array ranks and unions. Readonly and exclusive view reachability
propagates through the component; exclusive access wins when both occur. Acyclic
outer records retain independent slots. Incomplete type graphs do not claim a
complete lifetime boundary.

Source capture counting and native capture counting use the same index. A focused
red test first demonstrated the old 129-slot rejection; the repaired count is one
slot for a recursive component and three for an outer record containing two such
values and an independent view. Checks: 31 recursive tests, four environment tests
and two native capture tests pass, with no failures or aborted targets in the green
runs. The cycle test includes 4,096 record nodes. Exact red and green logs are in
SQLite under `tsrust-recursive-view-*.log`.

This is groundwork, not public recursive-storage acceptance. Parameter provenance,
record/runtime emission, lifetime enumeration and source/native/editor fixtures
remain required before removing the existing source diagnostic. No timing ran.

## Native recursive representation groundwork

Recursive view records now emit one named lifetime, shared through their fields,
array aliases, capture types and signature types. Recursive array support retains
its iterative drop traversal and now carries that lifetime through its drop stack.
Nonrecursive records keep their existing generic field representation.

The initial native runtime witness failed for missing lifetime parameters. Generated
record declarations and array support now execute readonly and exclusive self-recursive
and mutually recursive chains at O0 and O3. Each case has 10,001 view payloads,
checks pointer identity and exactly 10,001 destructor calls, and permits owner reuse
after disposal. Exclusive cases verify writes reach the owner. Execution uses a
128 KiB native thread stack to catch recursive cleanup. The unchanged noncloneable
record cleanup test also passes. The adjacent native-view filter passes 108 tests;
no failures or aborted targets remain in the green runs.

Exact logs are in SQLite: `tsrust-recursive-view-runtime-red.log`,
`tsrust-recursive-view-runtime-green.log`, `tsrust-recursive-view-emission.log`,
`tsrust-recursive-view-adjacent.log`, `tsrust-recursive-view-runtime-final.log` and
`tsrust-recursive-view-mutual-runtime.log`. These are private representation checks,
not public source acceptance or measurements. Source parameter seeding, whole-record
loan projections, assignment/append provenance and source/editor rejection controls
remain before removing the recursive-storage diagnostic. Nullable links are still
pending; their separate representation decision is approved.

## Recursive parameter provenance

Recursive record parameters now seed one shared carried origin. Loans explicitly
mark when their path represents a whole recursive record. Deep field/index
projections retain that origin; moving or replacing a descendant does not release
the remaining record's loan. Whole-record moves still release it. Normalization
keeps whole-record and exact-path loans distinct, and call transport preserves the
projection contract.

Three focused red projection tests and a red source parameter test preceded these
changes. Source tests now extract and forward views from recursive parameters and
preserve independent outer-record siblings. The native deep-chain witness now
executes the generated source extraction and forwarding functions for readonly and
exclusive, self-recursive and mutually recursive cases at O0/O3. Pointer identity,
write-through, exact disposal and bounded-stack cleanup still pass.

Checks: 112 adjacent native-view tests, two recursive parameter tests, the native
relay witness, and the focused editor byte-view conflict diagnostic pass. Green runs
have no failed or aborted targets. SQLite preserves `tsrust-recursive-view-projections-red.log`,
`tsrust-recursive-view-parameter-red.log`, `tsrust-recursive-view-parameter-green.log`,
`tsrust-recursive-view-projections-green.log`, `tsrust-recursive-view-source-relay.log`,
`tsrust-recursive-view-provenance-adjacent.log`, `tsrust-recursive-view-editor-diagnostics.log`
and `tsrust-recursive-view-parameter-siblings.log`.

Local recursive construction, nested assignment/append normalization and aggregate
returns remain unfinished. In particular, incoming loans from a source-created
recursive value must collapse at the recursive boundary before call transport;
relaxing call transport alone would lose the ownership guarantee. Keep the existing
aggregate-return diagnostic until construction and replacement preserve all owners.
This checkpoint does not accept gate 189, refresh consumers or claim measurements.


## Source construction and mutation through recursive arrays

Source-created recursive records now collapse carried loans at their recursive
boundary. Array elements, field replacement, append, consuming extraction and
aggregate returns retain every contributing owner. Outer sibling fields retain
independent lifetimes. Replacing a field with the same type no longer creates an
unnecessary scoped record identity that loses its cycle membership.

Plain native-view records use ordinary native record borrowing. Independent leaf
storage remains reserved for callback-bearing records, including owned callbacks
whose environments contain views. This preserves partial-move replacement while
avoiding nonexistent callback bridge types when borrowing a plain recursive record.

The public `recursive-arrays.tsr` fixture covers readonly/exclusive construction,
append, extraction, whole-record reads, identity forwarding, owning callback capture,
write-through and owner reuse after disposal. Five source rejection controls cover
owner move, write, write after extraction while the remaining root lives, exclusive
owner read and a shorter-lived nested owner. Private readonly/exclusive O0/O3 cases
also check pointer identity, independent outer siblings and native owner-escape
rejection. Editor coverage checks both the public fixture and related-location
reporting for a shorter-lived owner.

The matched `native_byte_views_recursive` workload constructs and consumes a
17-node chain of two-byte views. Its TypeScript counterpart uses
`Uint8Array.subarray`; an independent arithmetic oracle checks node count, byte sum
and retained view length. Source guards, hashes, fallible native ABI support and
registration tests are included. Timing remains pending S07 family acceptance.

Nullable recursive links and callbacks that capture views while returning view
arguments remain open. This increment does not accept gate 189 or S07.

Final checks: the original 115-test adjacent native-view batch passed. The added
whole-record borrow witness then exposed missing native bridge types; a follow-up
114-pass/one-failure run caught callback partial-move replacement during the repair.
The final affected stored-view batch passes all 16 tests, including that regression
and readonly/exclusive whole/nested borrowing. All 11 CLI byte tests, the normal-stack
concat-depth test, one focused editor test and two benchmark registration tests pass.
Whole-project formatting is unchanged and reports no fingerprint updates.

SQLite retains all 26 construction, borrowed-record, CLI/editor, registration and
formatting logs under `tsrust-recursive-view-*.log`, including original red runs.
`runtime/results/2026-10-11T00-30-34-645Z-verify.json` records agreement between native
and Node-after-tsc on five cases, with no execution samples. Exact-byte readback was
verified before deleting the loose copies. Consumers remain at `4367f4cc` until the
next planned refresh.


## Nullable-link representation checkpoint

The private `recursive_links::Index` identifies nullable record alternatives on an
inline strongly connected component. Array edges do not participate: a nullable
field whose only return path crosses an array keeps its original inline layout.
Only the record alternatives that close a cycle are indirect; unrelated alternatives
and ordinary nullable records remain unchanged. Iterative graph traversal handles a
4,096-record schema. The layout walker can account for link pointer size and cut only
planned edges; a remaining direct cycle is still rejected.

Native `RecordLinkN` support owns an `Option<Box<RecordN>>`. A present source link
will allocate exactly one box. The empty internal slot supports iterative cloning
and taking ownership without unsafe moves; it is not a new source value. Moving the
payload consumes its wrapper. Drop drains the same component work stack as recursive
arrays. Clone uses the existing iterative task engine and is emitted only for
cloneable records. A shared union used in another recursive component retains that
component's separate cleanup and clone traversal.

Private O0/O3 witnesses run self-linked and mutually linked 10,001-node chains on a
128 KiB native thread stack. They verify exactly 10,000 allocations during construction,
independent clone addresses and mutations, twice the retained allocation bytes after
cloning, 20,002 payload drops and full allocation restoration. Readonly/exclusive
view-bearing chains verify real slice lifetimes, pointer identity, write-through,
10,001 exact drops, owner reuse after the carrier scope ends and native rejection of
a local owner escaping. No clone implementation is emitted for these view chains.
The allocator observer uses the standard unsafe allocator instrumentation; generated
link ownership code contains no unsafe operations.

These are representation checks, not nullable-source acceptance. The original source
layout diagnostic remains until union construction, conversion, narrowing, matching,
clone/inspection and all move/borrow projections use the link wrapper consistently.
Enable planned indirection in `validate_inline_layout` only with that complete
emission path; replace the staging diagnostic assertion with public source/native
controls then. Disposal-order, public editor and matched-workload coverage also
remain required before accepting nullable links. No new API or timing is claimed.

Checks: five planner tests, four layout tests, three runtime tests and six public
recursive-array storage regressions pass. Whole-project formatting is unchanged.
SQLite preserves all 10 red/green/runtime/layout/formatting logs under
`tsrust-nullable-links-*.log`; exact-byte readback preceded loose-log removal.


## Nullable links through source and union emission

Nullable recursive fields now pass layout analysis with planned indirection. Native
union payload defaults use the owning link wrapper; borrowed union views expose the
logical record. Construction, widening/narrowing, consuming projection, field moves,
nullable replacement, coalescing and owned optional-chain extraction preserve the
physical representation. Record and function environment types carry real view
lifetimes through link payloads. Ordinary nullable payloads keep their existing
representation. Generic recursive record instances and multi-record nullable
alternatives have native controls.

The source matrix covers construction and consuming traversal, independent clone
mutation, mutual links, generic int/string chains, coalescing and nullable widening,
optional-chain moves of owned strings, array storage, readonly nullable inputs,
replacement and clone of nullable payloads. Readonly and exclusive view controls
exercise borrowed nullable inputs, owning union forwarding, both owner origins and
O0/O3 native execution. The public linked-view fixture and six rejection controls
cover owner moves, writes, extracted-link provenance, shorter-lived owners, exclusive
owner reads and forbidden clone. Editor checks cover both array and nullable forms.

The native allocation witness also compares exact payload disposal order against
the existing recursive-array runtime under the same deep-clone, root mutation and
partial-move sequence. Allocation restoration and small-stack checks still pass.

Checks: three source/native nullable tests, three recursive runtime tests, four
layout tests, five planner tests, 52 public tests across bytes/nullable/data unions/
nested records/recursive storage, one editor diagnostic test (both recursion forms)
and three benchmark registration tests pass. Complete compiler/editor regressions
and timing remain deferred to the S07 family milestone.

`native_byte_views_linked` is ready with matched TypeScript, an independent arithmetic
oracle, source guards/hashes and native ABI support. Native and Node-after-tsc agree
on five untimed cases in SQLite record
`runtime/results/2026-10-11T01-01-09-264Z-verify.json`; execution samples are empty.
Workload source acceptance is separate from gate 189 and measured acceptance.

Recursive stored view callbacks now have focused native coverage when links use
the same concrete callback family. Materialization reserves the identities of the
recursive component, replaces its callback fields, then retargets self/mutual links
and array leaves to that component. Nonrecursive fields and nominal source type
identity stay unchanged. Native declarations use the concrete recursive identity
and one real shared lifetime; no byte copy or shared owner is introduced.

The public `recursive-callbacks.tsr` fixture now runs. Its integration checks also
cover a mutual link, array recursion and an exclusive callback result that mutates
the original owner. Source checks retain owner-conflict, shortest-owner escape,
clone and repeated-Once rejection. A native probe verifies slice pointer identity,
short-owner rejection, and disposal of 10,000 linked callback nodes on a 64 KiB
stack at O0/O3. Nullable links keep their existing unique allocation cost.

Mixed callback factories now join one stable family per recursive callback field.
The family collects checked alternatives before emission; ordinary scoped families
keep their independent identities and lifetime slots. Links retain their concrete
record identity as factories are discovered, so joining needs no runtime chain
conversion or extra link allocation. A consumed callback field can be restored
from a different closure. Returning an extracted callback retains the shared
component lifetime and rejects even an owner retained only by a discarded sibling.
The native 10,000-node probe now alternates two factories at O0/O3.

Record-parameter transport now has focused acceptance for a uniquely resolved
concrete recursive component. Discovery collects callback families before binding
parameters; definitive checking retains real owner summaries and replaces stale
speculative callback return metadata. Native recursive records remain whole for
ordinary readonly borrowing and partial moves. Constructors checked against a
concrete parameter reuse the same component, including its mutually recursive
members. There is no added allocation or recursive value conversion.

The public parameter fixture and source variants cover owning/readonly parameters,
whole-record returns, extracted callback returns, explicit callback annotations,
inline construction, source-order permutations, mixed factories and mutual links.
Both the actual captured owner and a discarded sibling owner reject escape. The
native 10,000-node small-stack witness now passes through an owning record parameter
and proves pointer identity and both owner lifetime rejections at O0/O3.

Callback-only recursive view storage now has focused acceptance. Recursive
lifetime detection reads actual owned-family capture slots, including exclusive
access, even when the callback returns an integer. Concrete recursive layouts and
cleanup wrappers retain one shared lifetime; symbolic bases stay lifetime-free.
The public fixture covers nullable/self/mutual/array storage, parameter transport,
local replacement, exclusive writes and whole-record capture. An unused view
argument does not extend the result lifetime. The two-callback native chain witness
proves that either captured owner can limit the shared lifetime at O0/O3.

Nullable callback payloads now use the same per-field family across factories,
including absent initializers and later local replacement. Empty callback-only
results expose an independently quantified lifetime; unused inputs do not become
owners. Returned empty records can be filled with a later view without recursive
layout conversion. Public, editor and native lifetime witnesses cover these cases.

Recursive replacement through mutable parameters now has focused acceptance for
intact records. Native field families remain stable; mutation summaries retain
caller origins and enforce source-outlives-destination bounds. Relays, branches,
cleanup, function references, closures, returned installers and tracked capture
destinations transport those effects. Readonly/exclusive native execution and
source/editor owner rejections are covered. The original probe also moved the
field before passing the whole record; that use still correctly reports partial
transfer. Gate 189 scope review is complete; resource gate 190 is next. The finite-layout guard
remains for unsupported specializations; multiple incompatible components are not
resolved by choosing one arbitrarily.

SQLite preserves 24 source-emission, CLI/editor, registration and formatting
logs under `tsrust-nullable-links-*.log`, plus the exact callback probe source
`tsrust-nullable-view-callback.tsr`. Original failed witnesses remain intact. Exact
readback was verified before removing loose copies. Whole-project formatting is
unchanged; consumers remain at `4367f4cc`.
