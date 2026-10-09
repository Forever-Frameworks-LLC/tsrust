# Scoped callable transport

## Replacing a captured array element

A dense local array can hold records whose callbacks borrow different values.
Replacing a known constant index releases the previous element's captures when
no other element or borrowed handle retains them. Surviving callbacks keep their
captures protected. Replacement evaluates its new value before dropping the old
element; it does not clone either environment.

```tsr
type Holder = { run: ReadOnly<Function<int>>; };

function main() {
    const first = [3];
    const second = [5];
    const handlers: Holder[] = [
        { run: (): int => first[0] },
        { run: (): int => second[0] }
    ];
    handlers[1] = { run: (): int => first[0] };
    second[0] = 9;
    console.log(handlers[0].run(), handlers[1].run(), second[0]);
    // 3 3 9
}
```

Mutating `first` here is rejected because both surviving elements borrow it.
A borrowed element handle prevents replacement while the handle is live.
Unknown indices retain conservative capture protection. Precise native release
covers straight-line local dense arrays, arrays in nested record fields, and
explicit moves of arrays or their owning records. Independent sibling fields keep
their own captures protected. A standalone `pop()` or `shift()` statement also drops its removed value and
releases captures belonging only to that value when the dense shape is known.
Repeated removal can release all captures when the array becomes empty. A local bound
removal result keeps only the removed element's captures protected. After that
result leaves scope, its captures are released while surviving elements remain
usable. Moving the result preserves this ownership. Native storage moves surviving
records out of normal lexical blocks without cloning. Direct replacement of the result evaluates its new value before dropping the
previous value. Replacing it with `undefined` releases its captures when no other
owner remains. Fresh callback records and further known removals receive their own
native storage representation in normal lexical sequences, including normal
blocks. Replacements with different captured environments inside branches or
loops remain unsupported and produce source diagnostics. Same-expression release,
aggregate result storage and control-flow slot joins remain unfinished.

## Local callback selection

Conditional callbacks can borrow the variables around them and remain local to
that scope. Selection preserves the original captured references. It does not
clone a value or change a borrowed value into an owned value.

Reusable scoped callbacks work with array mapping, filtering, predicates, searching,
reduction, sorting and flat mapping, as well as Array.from, Set.forEach and
Map.forEach. A mutable callback can update its own captured variables. It cannot
mutate the traversed collection while that collection is borrowed for traversal.
A consuming callback is rejected by these methods, which require a reusable callback.
This contract applies even when a particular collection happens to be empty.

```tsr
import { strictEqual } from "std:testing";

function main() {
    const offsets = [3, 7];
    const chooseFirst = offsets.length === 2;
    const selected = chooseFirst
        ? (value: int): int => value + offsets[0]
        : (value: int): int => value + offsets[1];
    const values = [1, 2];
    const mapped = values.map(selected);
    strictEqual(mapped[0], 4);
    strictEqual(mapped[1], 5);

    const optional = chooseFirst ? selected : null;
    strictEqual(typeof optional, "function");
    strictEqual(optional?.(2) ?? -1, 5);
    strictEqual(optional?.(4) ?? -1, 7);
}
```

Moving a handle into the nullable value transfers that handle. Calling the nullable
value repeatedly is valid when its environment is reusable; using the transferred
original handle is not. Optional invocation skips the call when the handle is absent.
A nullish fallback on the invocation result supplies the fallback value only when
that result is absent.

Nullable scoped handles can cross ReadOnly<Function<...>> and Mutable<Function<...>>
parameter boundaries without losing their concrete environment. Readonly parameters
borrow the handle for reading. Mutable parameters borrow it for invocation that can
change captures. Captured values must remain valid for those calls. A scoped handle,
including one inside a nullable union, cannot escape through a return or throw.

Default and rest arguments retain the selected callback's own behavior. Traversal
supplies its ordinary callback arguments. Additional defaulted positions use their
selected defaults; rest packing preserves the actual supplied tuple. A required
position outside that tuple remains an arity error. Checked callback failures reach
the surrounding try/catch with their original payload.

This support covers monomorphic conditional families and nullable handles. General
scoped record/array storage, combining distinct carrier families in a fallback,
polymorphic selection/replacement and mutable factory restoration remain separate
parts of the callable transport plan.

### Discarded comma removal results

For a known dense local array of capturing records, a standalone discarded comma
sequence releases each directly removed result before evaluating its next operand.
For example, `(handlers.pop(), second[0] = 9)` permits mutation of `second` when
only the removed element captured it. Nested discarded sequences preserve source
order, and empty removal remains harmless. Surviving elements retain their captures.
A saved indexed view still prevents mutation of its array.

This refinement currently applies to direct removals combined with scalar
operations or scalar console output. It does not release a result passed to a
function before that function's later arguments are evaluated. Returning comma
results, storing them in aggregates and joining different environments through
control flow require their own storage planning.

A local declaration may retain the direct final removal in the same sequence:

```tsrust
const removed = (handlers.pop(), second[0] = 9, handlers.shift());
```

The discarded pop result releases its captures before the assignment. The shift
result transfers into `removed`; its captures remain protected until normal
scope disposal or supported direct replacement. Nested prefixes preserve order,
and an empty final removal produces `undefined`. Discarded owned string prefixes
are borrowed and remain available afterward. This does not yet extend to comma
results stored through aggregate fields, arbitrary final expressions or direct
sequence reassignment.

Direct named reassignment supports the same final-removal form:

```tsrust
removed = (handlers.pop(), second[0] = 9, handlers.shift());
```

The previous `removed` value remains alive through the whole RHS. The discarded
prefix result drops before the next operand. Once the final removal finishes,
the old destination drops and the new result takes its place. Mutation of a
capture belonging to the old destination inside the RHS remains an error.
This applies to statement assignments of existing concrete scoped result storage;
expression-valued assignments and declaration-only nullable record storage still
need their own transport support.

### Nullable local callback records

A nullable local can begin without a callback instance and later receive one:

```tsrust
type Holder = { run: ReadOnly<Function<int>>; };
const values = [3];
let saved: Holder | undefined = undefined;
saved = { run: (): int => values[0] };
if (saved !== undefined) console.log(saved.run()); // 3
saved = undefined;
values[0] = 7;
```

Use `Holder | null` for null-based absence, or include both nullish types and guard
both before accessing fields. Direct replacement evaluates the new value before
dropping the old one. Clearing releases captures with no other owner. Moving the
record into another local retains that local's capture protection. No callback
object is constructed for the absent runtime variant, and no implicit clone is
introduced.

A nullable local or field can also start populated. The initializer keeps its
capture protection until the value is replaced or cleared:

```tsrust
let saved: Holder | undefined = { run: (): int => values[0] };
if (saved !== undefined) console.log(saved.run()); // 3
saved = undefined;
values[0] = 7;
```

Direct nullable fields support population, replacement and clearing in normal
lexical sequences, including ordinary blocks and nested record paths such as
`outer.inner.saved`. The right operand can read the old field value; the old
environment is released only after the new value exists. Each field retains its
own capture owners. Clearing one field does not release a sibling field's
captures, and ordinary sibling fields remain assignable. A record may combine
these fields with a scoped callback array.

### Nullable callback records stored through control flow

A nullable callback record local, or a nullable field of a local record, can be
stored inside a branch, a loop or a `try` block. A record that has such a field can
also be replaced as a whole:

```tsrust
type Holder = { run: ReadOnly<Function<int>>; };
function pick(flag: boolean): void {
    const first = [3];
    const second = [4];
    let saved: Holder | undefined = undefined;
    if (flag) {
        saved = { run: (): int => first[0] };
    } else {
        saved = { run: (): int => second[0] };
    }
    if (saved !== undefined) console.log(saved.run()); // 3 when flag is true, 4 otherwise
}
```

These stores change when captured variables become free. When any assignment to the
local or to one of its nullable fields sits inside a branch, loop, `try` block or
labelled block, or replaces the whole record, every environment stored in that local
keeps its captures borrowed until the local's scope ends. Clearing the value does not
release them earlier:

```tsrust
let saved: Holder | undefined = undefined;
if (flag) saved = { run: (): int => first[0] };
saved = undefined;
first[0] = 9; // error: capture-conflict
```

Declare the local in an inner block to end the borrow sooner. A local whose stores
are all direct assignments in a normal lexical sequence keeps the precise rule
described above.

Different callbacks can share one record variable, field or array element, whether
they borrow local storage or own every capture, for example an arrow that reads only
a number parameter.

Moving the containing record transfers its field owners without cloning.

### Arrays and Maps of capturing records across function boundaries

A function can borrow an array or a Map whose elements are capturing records. A
`ReadOnly<...>` parameter can iterate, index and look up; a `Mutable<...>` parameter
can also remove elements:

```tsrust
function total(handlers: ReadOnly<Holder[]>): int {
    let sum = 0;
    for (const handler of handlers) sum += handler.run();
    return sum;
}
const first = [3];
const handlers: Holder[] = [{ run: (): int => first[0] }, { run: (): int => first[0] + 1 }];
console.log(total(handlers)); // 7
```

The same function accepts arrays whose callbacks capture different variables. Arrays
with nullable elements and Maps such as `Map<string, Holder>` work the same way. An
array of capturing records can start empty and never receive an element.

A function can return an array of records whose callbacks own every capture. A
callback that borrows a parameter or a local of the function reports
`callback-escape`.

### Record parameters with nullable callback fields

A function can borrow a record that has a nullable callback-record field and read
that field, with a guard or an optional call. Both `ReadOnly<...>` and `Mutable<...>`
parameters can read it; a mutable parameter can still change ordinary sibling fields:

```tsrust
type Box = { saved: Holder | undefined; };
function use(box: ReadOnly<Box>): int {
    return box.saved?.run() ?? -1;
}
const box: Box = { saved: undefined };
console.log(use(box)); // -1
box.saved = { run: (): int => first[0] };
console.log(use(box)); // 3
```

A mutable parameter can clear such a field with `undefined` or `null`; the caller's
captures stay borrowed. Storing a new callback record into the field through a
parameter reports `unsupported`; store it in the function that owns the record. Two parameters of one function cannot both carry a
nullable callback record of the same type; that form reports `unsupported` too.

### Nullable callback records in conditional values, results and array elements

A conditional expression can select between two callback records, or between a
callback record and `undefined` or `null`. The value holds whichever environment was
selected, so both branches' captures stay borrowed while the value is alive:

```tsrust
const chosen: Holder | undefined = flag ? { run: (): int => first[0] } : undefined;
if (chosen !== undefined) console.log(chosen.run()); // 3 when flag is true
```

A function can return `Holder | undefined` or `Holder | null`, and a record result
can leave a nullable callback field absent on one path and populated on another:

```tsrust
function make(seed: int): Holder | undefined {
    if (seed > 0) return { run: (): int => seed };
    return undefined;
}
const made = make(4);
if (made !== undefined) console.log(made.run()); // 4
```

A returned callback must own every capture; a callback that borrows a parameter or a
local of the function reports `callback-escape`. Returns with different callbacks
share one result environment.

A nullable callback-record field of an array element can be stored through an index,
and whole elements can be pushed, replaced and removed:

```tsrust
type Box = { saved: Holder | undefined; };
const boxes: Box[] = [{ saved: undefined }];
boxes[0].saved = { run: (): int => first[0] };
console.log(boxes[0].saved?.run() ?? -1); // 3
```

Every environment stored through an element stays borrowed until the array's scope
ends; clearing one element field does not release it. Read such a field in place, with
a guard or an optional call. Moving it out of the element, as in
`const saved = boxes[0].saved`, reports `unsupported`.

An array whose element type is itself nullable holds callback records and absence
side by side:

```tsrust
const slots: (Holder | undefined)[] = [undefined, { run: (): int => first[0] }];
slots[0] = { run: (): int => first[0] + 1 };
for (const slot of slots) console.log(slot?.run() ?? -1); // 4, then 3
```

Such an array, as a local or as a field of a local record, supports literals,
indexed stores, `push`, `pop`, `shift`, iteration and element views such as
`const slot = slots[0]`.
