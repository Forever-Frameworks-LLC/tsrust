# Milestone 22: owned array elements

[Scalar arrays](./20-scalar-arrays.md) · [Borrowed iteration](./21-for-of.md) · [Collections workstream](../primitives-and-collections.md) · [Roadmap](../roadmap.md)

**Status:** Complete. [Milestone 23](./23-nested-arrays.md) subsequently adds nested arrays; the scope below records this gate's original boundary. The full 271-group regression suite passes, including 12 owned-element groups. Formatting, Clippy, and local documentation links are verified.

## Types and construction

Arrays now support `string[]` and arrays of a named record such as `User[]`, alongside the existing copied integer and boolean arrays. Generated Rust stores owned `String` or concrete record values in native vectors. Array elements have one exact nominal type; records with identical fields but different names cannot be mixed.

Literal construction, push, and indexed replacement transfer owned elements into the array. The source becomes unavailable until explicitly reinitialized under the existing let rules. An array literal can infer its element type from a string expression or named record value. Record literals require an expected record type from the array annotation, a preceding typed element, or a function/assignment/return context. Empty arrays still require type context.

```typescript
type User = { name: string; active: boolean; };
function activate(user: User) {
  user.active = true;
}
function main() {
  const users: User[] = [{ name: "Pat", active: false }];
  users.push({ name: "Sam", active: false });
  activate(users[0]);
  for (const user of users) {
    console.log(user.name);
    console.log(user.active);
  }
}
```

This prints `Pat`, `true`, `Sam`, and `false`. `activate` infers mutable record access. The call borrows the existing element; it does not transfer or clone it.

## Indexed access and explicit ownership

Indexing an owned element borrows that element. A read argument receives a shared borrow, while a mutable argument receives exclusive access. Record fields can be read or replaced through an indexed record element, including owned string fields. Scalar fields copy normally. Whole element replacement destroys the old value and transfers the replacement into its slot.

Declaring an owned local from `values[index]`, returning that element, passing it to a consuming argument, or pushing it into another array receives `borrowed-element`. Moving a slot would leave a hole in the source vector. Use `clone(values[index])` to construct an owned independent value. String transforms such as `values[index].trim()` already produce an owned result. Removal and consuming iteration need separate syntax/contracts.

`clone()` now accepts strings, records, and arrays. Cloning an owned-element array clones each element and its owned string fields. The clone has independent storage and can be changed without affecting the original. These allocations are explicit; construction, indexed access, argument forwarding, and ordinary iteration insert no clones.

Element argument loans conservatively protect the whole array for the duration of the call. Two shared element arguments are allowed. An exclusive element argument conflicts with another element borrow or with access that would grow, replace, or transfer the source array, even for different literal indices. This gate does not attempt to prove that indices select disjoint elements.

## Borrowed iteration and captures

For-of over strings or records uses `.iter()` and gives each iteration a shared element reference. The iterable expression evaluates once, temporary storage remains alive through the loop, and the existing source-array loan remains active. Scalar iteration continues to use `.iter().copied()`.

An owned iteration value can be read, passed through read contracts, used for string transforms, or explicitly cloned. It cannot be moved, mutated, or rebound, including when declared with `let`; those actions receive `borrowed-element` and the iteration source location. To modify an independent value, declare a local from `clone(element)`. To modify an existing array element, use indexed access after the shared iteration ends. Mutable or consuming iteration is a distinct future contract.

Local read closures may borrow an iteration element within its lexical scope. A returned or retained owned closure cannot keep that borrowed element after iteration ends. Clone the element into a local owned value before retaining it. Nested local closures preserve this restriction. Existing scalar capture copies, whole-array capture loans, owned returned array environments, reusable callbacks, and single-use callbacks remain supported.

## Evaluation, bounds, and cleanup

Literal elements evaluate left to right and once. Index writes and indexed record-field writes evaluate the index, then the replacement, then check the index and mutate the slot. A replacement expression may grow the same array before the final access; the compiler rejects transferring the receiver during operand evaluation. Push evaluates its owned argument before taking the mutable receiver borrow, so `values.push(clone(values[0]))` works without overlapping native borrows.

Negative and out-of-range indices terminate with the original indexing expression's source byte range. These checks remain active with optimization enabled and backend overflow checks disabled. Owned elements use deterministic native cleanup when replaced, when their vector is destroyed, or when temporary iteration storage expires. No reference counting or tracing collector is added.

Whole arrays retain existing move, clone, reinitialization, branch join, loop restoration, parameter inference, capture, and explicit callable argument contracts. Borrowed element results cannot escape through function returns.

## Evidence and remaining scope

The [owned-array suite](../../crates/tsrust-compiler/tests/owned_arrays.rs) covers construction and transfers, exact contexts and nominal types, read/mutable indexed arguments, field writes, clone independence, extraction diagnostics, temporary and named iteration, captures and callback contracts, overlapping loans, operand evaluation, loop restoration, native bounds checks in two optimization profiles, backend-independent analysis, and valid deep iteration. Accepted programs compile generated Rust and compare exact native output.

The [users fixture](../../crates/tsrust-compiler/tests/fixtures/owned-arrays/users.tsr) prints `Pat`, `true`, `Sam`, `false`, `Pat`, and `Alex`. Its cloned first record retains the original name after the source element changes.

Nested arrays, arrays in record fields, callable elements, slices, borrowed local declarations and results, mutable or consuming iteration, extraction/removal, destructuring, disjoint-index loan proofs, iterator pipelines, shared ownership, async, modules, and the Rust language server remain separately scoped work.
