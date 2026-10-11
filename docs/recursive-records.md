# Recursive records

A record can own another instance of its type through an array or a nullable link:

```ts
type Chain = {
  value: int;
  next?: Chain;
};
```

`next?: Chain`, `next: Chain | undefined` and `next: Chain | null` provide a
terminating link. A cycle can also pass through several record types, provided
nullable links or arrays break every otherwise infinite inline layout. A required
`next: Chain` field alone is rejected. Existing inline-depth limits still apply
between nullable or array boundaries.

A present recursive nullable link owns one heap allocation. An absent link allocates
nothing. Moving a link transfers its value without cloning it. `clone(record)` makes
an independent deep copy when every contained value supports cloning. Nonrecursive
nullable fields retain their inline representation.

Records keep ordinary field access, replacement, nullable checks, `!`, `??` and
optional chaining. A non-null assertion on an absent value reports the existing
non-null failure. Optional access to a named owner's noncopy field produces a
borrowed value; use `clone()` when an independent owning result is needed.
Recursive ownership cleanup and deep cloning use iterative traversal.

## Records containing byte views

A recursive record can contain `ByteView` or `ByteViewMut`. Every view throughout
that recursive component shares one lifetime. The structure must stay within the
lifetimes of all contributing byte owners. Removing a child does not grant early
access to owners still retained by the remaining carrier's shared lifetime.
Independent sibling fields outside the recursive component retain separate lifetimes.

Views remain noncloneable, so a record containing them cannot be deep-cloned. No
bytes are copied when constructing or moving its links. Mutable views still write
through to the original owner. Neither bytes nor views require explicit close.

The executable [linked-view example](../crates/tsrust-compiler/tests/fixtures/native-byte-views/recursive-links.tsr)
constructs two-owner readonly and mutable chains, forwards and captures a chain,
extracts its next link, observes byte `7`, writes byte `19` through an exclusive
view, and reuses the owners after the carrier scope ends.

Recursive records with stored callable fields still receive the existing
`callable-layout` diagnostic. Owning a complete linked record in a callback is
supported, as shown by the linked-view example. Stored callable fields remain part
of the pending callback-storage work.
