# Array API audit

Status: 43 members reviewed: 40 receiver APIs and three static Array members.
Retain familiar global Array static operations and receiver methods. Keep the
native remove extension and void-returning mutators explicitly documented; do not
add imported aliases or silently convert native ownership into shared references.

## Shared behavior and ownership

Arrays are dense with a checked element type, including declared union elements;
they do not imply JavaScript sparse slots or arbitrary dynamically typed mixtures.
Signatures below use Element/Result placeholders for checked inferred types.
Callback Result means the callback result type, not necessarily the Result enum.
Indices/counts use strict int arguments. Wrong counts/types/access are static
errors. Invalid mandatory bounds and empty seedless reductions terminate; optional
at/find/pop/shift operations use undefined as documented. Checked callback errors
propagate through catch/finally. No allocation Result contract is introduced.

Copy-producing APIs make independent values and require cloneable owned contents.
Views do not become cloneable. Mutators require exclusive receiver access; ordinary
traversal borrows elements and prevents overlapping mutation/transfer. Temporary
receivers can transfer ownership where the operation explicitly supports it.
Every row below retains its current receiver/static placement.

## Receiver members

| Symbol | Signature | Behavior, defaults, errors and ownership |
|---|---|---|
| `Array.toString` | `toString(): string` | Borrow; comma-joined element text, empty gives empty string; no arguments. |
| `Array.join` | `join(separator?: string): string` | Borrow; separator defaults comma; nested array textual joins, absent elements empty; independent text. |
| `Array.length` | `length: int` | Readonly dense element count; no mutation. |
| `Array.push` | `push(value: Element): void` | Exclusive receiver; transfers one element; returns void. |
| `Array.remove` | `remove(index: int): Element` | Exclusive receiver; required zero-based int index; out-of-range fails; transfers removed owner and shifts later elements. |
| `Array.includes` | `includes(value: Primitive, fromIndex?: int): boolean` | Read-only primitive search, SameValueZero (NaN matches); optional relative fromIndex defaults zero; primitive union elements supported, query itself must be narrowed. |
| `Array.indexOf` | `indexOf(value: Primitive, fromIndex?: int): int` | Read-only primitive strict equality search; NaN never matches; optional relative fromIndex defaults zero; result index or -1. |
| `Array.lastIndexOf` | `lastIndexOf(value: Primitive, fromIndex?: int): int` | Same strict equality, backward search; fromIndex defaults last; index or -1. |
| `Array.slice` | `slice(start?: int, end?: int): Element[]` | Borrows source, copies selected descendants; defaults zero/length, relative negative bounds clamp; reversed range empty. |
| `Array.reverse` | `reverse(): void` | Exclusive receiver, rearranges existing owners, returns void. |
| `Array.some` | `some(callback: (value: Element, index: int, source: Element[]) => boolean): boolean` | Borrowed traversal forward until true; empty false; predicate must return boolean. |
| `Array.every` | `every(callback: (value: Element, index: int, source: Element[]) => boolean): boolean` | Borrowed traversal forward until false; empty true; predicate must return boolean. |
| `Array.findIndex` | `findIndex(callback: (value: Element, index: int, source: Element[]) => boolean): int` | Borrowed traversal forward to first true predicate; index or -1. |
| `Array.findLastIndex` | `findLastIndex(callback: (value: Element, index: int, source: Element[]) => boolean): int` | Borrowed traversal backward to first true predicate; original index or -1. |
| `Array.filter` | `filter(callback: (value: Element, index: int, source: Element[]) => boolean): Element[]` | Borrows source; clones retained owned descendants into independent result in encounter order. |
| `Array.map` | `map(callback: (value: Element, index: int, source: Element[]) => Result): Result[]` | Borrows source; callback result owners transfer into independent output array; same visit order. |
| `Array.forEach` | `forEach(callback: (value: Element, index: int, source: Element[]) => Result): void` | Borrows source; visits in order, discards callback results with ordinary disposal; void. |
| `Array.reduce` | `reduce(callback: (accumulator: Accumulator, value: Element, index: int, source: Element[]) => Accumulator, initialValue?: Accumulator): Accumulator` | Left-to-right; explicit seed visits all elements, omission uses first; empty seedless fails. Accumulator access follows callback contract; independent results transfer to next step. |
| `Array.reduceRight` | `reduceRight(callback: (accumulator: Accumulator, value: Element, index: int, source: Element[]) => Accumulator, initialValue?: Accumulator): Accumulator` | Same accumulator contract right-to-left; omission uses last; empty seedless fails. |
| `Array.concat` | `concat(...values: Element \| Element[]): Element[]` | Independent copied receiver plus same-type elements/arrays in argument order; no coercion or recursive flattening. |
| `Array.fill` | `fill(value: Element, start?: int, end?: int): void` | Mutates relative clamped range (default zero/length); owned fill value transfers and clones into independent positions; void. |
| `Array.copyWithin` | `copyWithin(target: int, start: int, end?: int): void` | Mutates target range using overlap-safe snapshot; relative clamped offsets, end defaults length; deeply copies owned elements; void. |
| `Array.with` | `with(index: int, value: Element): Element[]` | Borrows/copies source, transfers replacement; negative index from end; out-of-range fails; independent result. |
| `Array.toReversed` | `toReversed(): Element[]` | Borrowed source, independent reversed copy. |
| `Array.toSpliced` | `toSpliced(start: int, deleteCount?: int, ...items: Element): Element[]` | Borrowed source; independent copied result, inserted items transfer; relative start clamps, omitted deleteCount removes suffix; source unchanged. |
| `Array.pop` | `pop(): Element \| undefined` | Exclusive receiver; removes/transfers last owner; empty undefined. |
| `Array.shift` | `shift(): Element \| undefined` | Exclusive receiver; removes/transfers first owner, shifts remaining elements; empty undefined. |
| `Array.at` | `at(index: int): Element \| undefined` | Required signed int; negative from end; out-of-range undefined. Named owned element borrows readonly; copyable element copies; consumed temporary may transfer selected owner. Declared nullable elements preserve null. |
| `Array.find` | `find(callback: (value: Element, index: int, source: Element[]) => boolean): Element \| undefined` | Forward predicate traversal; first matching element or undefined; named owned result borrows readonly, consumed temporary can transfer selected owner. |
| `Array.findLast` | `findLast(callback: (value: Element, index: int, source: Element[]) => boolean): Element \| undefined` | Same result contract, backward predicate traversal; original indices. |
| `Array.unshift` | `unshift(...values: Element[]): int` | Transfers zero or more elements to front in argument order; returns new int length; empty call leaves content unchanged. |
| `Array.splice` | `splice(start?: int, deleteCount?: int, ...items: Element[]): Element[]` | Exclusive receiver; transfers removed owners into result and inserted owners into receiver. No arguments empty/no mutation; relative start clamps; omitted deleteCount removes suffix. |
| `Array.sort` | `sort(compare?: (left: Element, right: Element) => int \| number): void` | Exclusive receiver; stable rearrangement without payload cloning; void. Optional comparator returns one concrete int/number type; NaN equals zero comparison; callback failures propagate. |
| `Array.toSorted` | `toSorted(compare?: (left: Element, right: Element) => int \| number): Element[]` | Independent stable sorted copy; same comparison rules; source unchanged; copying requires cloneable payloads. |
| `Array.flat` | `flat(depth?: int): Flattened[]` | Independent flattened output. Depth default 1, nonpositive copies; nested source needs literal depth to fix result rank; rank-one allows dynamic int. Named source borrows/copies, fresh owner can transfer. |
| `Array.flatMap` | `flatMap(callback: (value: Element, index: int, source: Element[]) => Result): Flattened[]` | Borrows source; callback owners transfer into output with one level of flattening; primitive result remains one item; checked result rank. |
| `Array.entries` | `entries(): [int,Element][]` | Eager independent [int,element] array; copies owned payloads; no live iterator. |
| `Array.keys` | `keys(): int[]` | Independent int positions in order; does not copy payloads. |
| `Array.values` | `values(): Element[]` | Independent copied element array in order; no live iterator. |
| `Array.toLocaleString` | `toLocaleString(): string` | Borrowed element formatting with fixed native locale rules and comma separation; no locale/options arguments. |

## Static members

| Symbol | Signature | Behavior and ownership |
|---|---|---|
| `Array.isArray` | `isArray(value?: unknown): boolean` | Optional observed value, omitted false; arrays and fixed tuples true, other supported values false; inspects active data union without narrowing it. Extra arguments evaluate and are ignored; borrows owners. |
| `Array.of` | `of<T>(...items: T[]): T[]` | Zero or more exact/contextual T values move into independent dense array; nonempty infers type, empty needs explicit/contextual type; one int means one element, not a length constructor. |
| `Array.from` | `from<T, U>(source: T[] \| string \| Set<T> \| Map<Key, Value>, map?: Function<U, [value: T, index: int]>): U[]` | Borrows supported array/string/Set/Map source; strings yield Unicode scalars, maps entry tuples. Omitted mapper copies values; mapper receives readonly item and int index (no source parameter), results transfer. OrderedMap follows its ascending traversal. No arbitrary iterator protocol. |

## Callback and ordering details

Ordinary traversal supplies element, original int index and readonly source;
callbacks may omit trailing positions and use supported defaults/rest parameters.
Reduce supplies accumulator first. Array.from supplies only element/index.
Borrowed owning arguments cannot be packed into owning rest parameters. Defaults
run only for absent/undefined supplied positions; synthetic padding is not rest data.

Reusable callable-element traversal has a deliberate exception: a mutate callable
visitor borrows the stored environment exclusively so captured state changes can
persist. Receiver/source access, escape and once-only environments remain subject
to the existing method-specific restrictions. This is not permission for arbitrary
receiver mutation during callback execution. Callback result owners transfer;
filter copies retained source data rather than copying callback environments.

Sorting without a comparator uses native defaults: numeric int order, f64 total
order (signed zeros/NaN bits distinguished), false before true, UTF-8 lexical text
order, and equal homogeneous null/undefined values. Aggregate/callable elements
require a comparator. Comparator sign controls order; number NaN means equal.
No comparator receiver access or consuming/mutating element arguments are allowed.
Comparator invocation order/count are implementation-dependent.

## Source reconciliation and dependencies

Compiler anchors under crates/tsrust-compiler/src:

- analysis.rs receiver dispatch and array_predicate/array_transform,
  array_reduce/array_find enforce predicate and callback access/result contracts.
- analysis.rs::array_splice/array_unshift/array_copying/array_sort/array_flat and
  corresponding emit.rs functions implement mutation, copying, bounds and rank.
- array_entries.rs handles eager tuple snapshots; emit.rs::array_entries,
  array_slice/array_at/array_optional_removal/array_remove preserve copy/borrow/move
  distinctions. Array storage and capture owner modules preserve view provenance.
- analysis.rs::array_from/array_from_mapping and Array.of/Array.isArray dispatch
  implement static calls, shadowing, inference and selected active alternatives.
- emit.rs array_transform/array_for_each/array_predicate/array_reduce/
  array_flat_map construct callback argument tuples and transfer returned owners.
- emit.rs selects existing array/string/sort/runtime helper families from checked
  shapes and emitted operations. No external package or user linker flag is needed.
  Stack/fixed-array representations may be selected internally; this audit does not
  claim per-method binary elimination or measure new performance.

Catalog rows were reconciled with checker/emitter paths and executable examples.
The at entry had a stale nullable-element prohibition: focused native execution
proves (int|null)[] and (string|null)[] retain null separately from out-of-range
undefined. Hover now states that existing supported behavior; runtime is unchanged.

## Focused acceptance

SQLite `tsrust-api-array-*.log` retains focused copying, optional selection,
flattening, collection conversion, tuple snapshot and editor checks, including the
failing stale-documentation witness and corrected native/hover witness. All 169
catalog examples execute in one formatted native program with isolated blocks,
module-level type declarations and renamed example entrypoints; expected output
is preserved. Whole-project formatting runs before checkpoint. No new timing or
complete feature-family regression is claimed.
