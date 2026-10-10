# Recursive storage that holds byte views

Date: 2026-10-10. Status: approved plan, not implemented. The owner approved parts A and B in the continuation chat.

The owner wants types such as:

```
type Chunk = {
  data: ByteView;
  next?: Chunk;
};
```

Two separate limits stop this today.

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
