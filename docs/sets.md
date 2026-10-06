# Set collections

A `Set<T>` stores unique keys in insertion order. Supported keys are `int`,
`number`, `boolean`, `string`, `null`, `undefined`, enums and unions of these
values. Record, array and function identity keys are not supported yet.

Construct an empty set with an explicit key type, or infer the key type from an
array, another set or a string. A string contributes Unicode characters.
`new Set<T>(null)` and `new Set<T>(undefined)` create empty sets. Construction
copies keys explicitly and keeps the source usable.

```ts
const names = new Set<string>(["Ada", "Lin", "Ada"]);
names.add("Sam");
const count = names.size; // 3
const found = names.has("Ada"); // true
const removed = names.delete("Lin"); // true
const snapshot = names.values(); // ["Ada", "Sam"]
names.clear();
```

`size` is a readonly integer. `has` reports membership; `delete` reports whether
it removed a key. `clear` removes all keys. `add` ignores an existing key and
returns `void`; JavaScript returns the receiver. Deleting a key and adding it
again places it at the end. Number keys use SameValueZero: NaN equals NaN and
negative zero equals positive zero. Stored negative zero becomes positive zero.

`values()` and `keys()` return independent array snapshots. They explicitly
copy keys. They do not return JavaScript's live iterator objects.

`for (const key of names)` reads keys in insertion order. Iteration borrows the
set, so mutation and transfer of the set during iteration are rejected. Scalar
keys are copied; owned keys are readonly borrows. Adding an owned key transfers
that key; membership queries and deletion read their argument. Access and
ordinary bindings never implicitly clone a set.

Method arguments must match the key type; arity and unsupported key types are
static errors. An int source does not implicitly become a number source.
Compatible keys can widen into a union during explicit construction. `clone(set)`
creates an independent copy. Set algebra, live iterators,
object identity keys and constructor aliases are not supported yet.

## Key/key entry snapshots

`entries()` takes no arguments and returns an independent array of [Key, Key]
tuples in insertion order. Each tuple contains the key twice. Empty Sets return
empty arrays. Deletion and reinsertion places a key last; adding an existing key
does not change its position. NaN and zero keys follow the Set's SameValueZero
policy; negative zero is normalized to positive zero.

```tsrust
const values = new Set<string>(["b", "a", "b"]);
const pairs = values.entries();
values.clear();
console.log(pairs[0][0], pairs[0][1]); // b b
```

The method explicitly copies owned keys into both tuple positions and leaves the
Set usable. Later receiver mutation or disposal does not change the snapshot.
The return value is an array, not a live iterator. Tuple key types are retained
through generic functions, enum/union keys, temporary receivers and editor help.
Wrong argument counts or incompatible tuple result annotations are static errors.

## Borrowed callbacks

`forEach(callback)` visits each key once in insertion order. It supplies the key,
the same key again and a readonly Set. A callback may use zero through three
supplied positions, with normal default/rest parameter rules. Empty Sets make
no calls. There is no thisArg argument. The method returns void; callback return
values are evaluated and discarded after each invocation.

```tsrust
const values = new Set<int>([3, 1, 3]);
let total = 0;
values.forEach((value) => { total = total + value; });
console.log(total); // 4
```

Scalar callback arguments copy. Owned keys and the source Set borrow readonly;
no callback input implicitly clones. Mutating or transferring the receiver during
iteration is rejected. External mutable captures are allowed when they do not
conflict with source borrows. Callbacks must be reusable; a consuming callback
cannot run repeatedly. Owned callback inputs cannot be packed into an owning rest
array. Wrong argument counts and incompatible callback types/access are static
errors.

Callback errors stop iteration and propagate to surrounding try/catch. A pending
finally executes through the ordinary checked completion mechanism. Catch values
may include structured runtime errors; narrow with typeof before string-only
operations. The same checked propagation applies to Array.forEach.

## Relation predicates

`isSubsetOf(other)` checks that every receiver key occurs in other.
`isSupersetOf(other)` checks that every other key occurs in the receiver.
`isDisjointFrom(other)` checks that no key occurs in both. Each takes exactly one
Set, returns boolean and leaves both inputs unchanged. The key types may differ.
Set-like records are not accepted.

Empty sets are subsets of every set and disjoint from every set. Every set is a
superset of an empty set. A nonempty set is not disjoint from itself.

Comparisons use existing native key equality, including SameValueZero for number
keys. Numeric int/number alternatives can match; comparisons involving large
int-to-number conversion can lose precision. Enum keys retain their enum type
identity. Owned keys borrow during lookup; no key is cloned or transferred.

```tsrust
const values = new Set<int>([1, 2]); console.log(values.isSubsetOf(new Set<int>([1, 2, 3])));
```

This prints `true`.

## Independent algebra results

`union(other)`, `intersection(other)`, `difference(other)` and
`symmetricDifference(other)` return independent Sets and leave both inputs
unchanged. Each accepts one Set, possibly with a different supported key type.
Set-like records are not accepted. These explicit operations copy selected owned
keys; ordinary bindings and reads still do not clone.

Union preserves receiver order and appends new other keys. Difference preserves
receiver order while removing shared keys. Symmetric difference keeps unshared
receiver keys first, followed by unshared other keys. Intersection follows the
smaller Set's insertion order; equal sizes use receiver order. It copies matching
receiver keys so its result retains the receiver key type.

Union and symmetric difference infer a joined key type. Intersection and
difference preserve the receiver type. Key equality follows the same native
rules as relation predicates, including documented numeric precision and enum
identity limits. Empty inputs follow ordinary set algebra.

```tsrust
const values = new Set<int>([3, 1]); const result = values.union(new Set<int>([1, 2])); console.log(result.values().join(","));
```

This prints `3,1,2`. Mutating the result does not change either input.

When numeric precision loss lets an other key match multiple exact receiver keys,
intersection retains every matching receiver key. The receiver's distinct native
integer values remain distinct in its result.
