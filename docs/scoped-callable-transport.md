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
