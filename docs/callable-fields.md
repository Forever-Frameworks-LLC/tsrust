# Owned callable record fields

A record field can store a `Function<...>` or a `Once<Function<...>>`. Its signature defines argument types and access, return type and whether it may be called repeatedly. Creating the field transfers an owned environment into the record. Different compatible environments can replace the field without changing the record type.

```tsrust
type Handler = { run: (delta: int) => int; };

function main() {
    const handler: Handler = {
        run: (delta: int): int => { return delta + 1; }
    };
    console.log(handler.run(2));
    handler.run = (delta: int): int => { return delta * 2; };
    console.log(handler.run(3));
}
```

This prints `3` and `6`. Reusable calls borrow the field mutably and can update captured state. Replacing a field disposes of its previous environment. Captured owned data moves into the environment; scalar captures copy. The record binding can be `const`: replacing a field changes its contents without rebinding the record.

Owned-data parameters require an access mode in the signature, such as `(user: ReadOnly<User>) => int`. Arrow bodies infer argument access, so write `(user: User): int => { return user.name.length; }`. The inferred access must match the field signature. Generic records can use type parameters in those signatures; the concrete specialization checks argument access and preserves nominal types.

A `Once` field transfers its environment when called. Another call or whole-record use is rejected until that field is restored. Assigning a new compatible environment restores only that field; other consumed fields remain unavailable. A reusable environment can initialize a single-use field, but its stored contract still permits only one call.

Conditional field construction and replacement accept different compatible environments: `run: flag ? (value: int): int => { return value + 1; } : (value: int): int => { return value * 2; }`. Only the selected branch creates its environment at runtime. Both branches must match the field signature. Owned captures moved on either continuing branch are unavailable afterward; nested conditionals follow the same rule.

Moving a field into another record transfers its ownership. Records containing callable fields cannot be cloned, including through enclosing records, arrays or copy-producing array methods. Record arrays support ordinary push/removal and reusable indexed invocation. To invoke a single-use field in an array, remove its owning record first; partial moves out of indexed records remain unsupported.

Bare callable field signatures own their environments. A borrowed local environment cannot be converted into an owned field merely by giving it a matching signature; use an owned environment with independent captures, or declare a borrowing field as described below. A stored function can move from one record field into another record.

## Borrowing callable fields

A field declared `ReadOnly<Function<...>>` or `Mutable<Function<...>>` holds a callback that borrows the variables it uses. The record then lives no longer than those variables: it stays in the function that created it, and the borrowed variables are free again when the record goes out of scope.

```tsrust
import { strictEqual } from "std:testing";

type Holder = { run: ReadOnly<Function<int>>; };
type Step = { next: Mutable<Function<int>>; };

function call(holder: ReadOnly<Holder>): int {
    return holder.run();
}

function replace(holder: Mutable<Holder>, amount: int): void {
    holder.run = (): int => amount;
}

function main() {
    const base = [3, 4];
    let count = 0;
    {
        let holder: Holder = { run: (): int => base[0] };
        strictEqual(call(holder), 3);

        const step: Step = { next: (): int => { count += 1; return count; } };
        step.next();
        strictEqual(step.next(), 2);

        replace(holder, 10);
        strictEqual(holder.run(), 10);
        holder.run = (): int => base[1] * 2;
        const run = holder.run;
        strictEqual(run(), 8);
    }
    base[0] = 0;
    strictEqual(count, 2);
}
```

A `ReadOnly` field is called through shared access and cannot change what it captures. A `Mutable` field can change captured variables; calling it needs exclusive access to the field. While the record is alive, the ordinary borrowing rules apply to the captured variables: `base[0] = 0` inside the block above is rejected, and accepted after it.

Each field keeps its own borrow. Sibling fields of one record, and fields of nested records, can be passed to two parameters of one call, replaced separately and moved out separately. Moving a field out (`const run = holder.run`) moves its callback and its borrow to the new variable.

Replacing a field ends the borrow of the old callback when the old callback can no longer be reached. A replacement in only one branch, or in a loop body that may not run, keeps the old borrow, because the old callback may still be in the field.

A function can take such a record as `ReadOnly<Holder>` to call its fields, or as `Mutable<Holder>` to call `Mutable` fields and replace callbacks. A callback stored through a record parameter must not borrow from the function that stores it: it can capture numbers and booleans, which are copied, but not a parameter or local of that function. Storing a callback parameter there is rejected for the same reason.

A record that holds a borrowing callback cannot be returned, thrown or stored where it would outlive the borrowed variables. These uses report `callback-escape`.

Current limits of borrowing fields:

- An array, `Map`, `Set` or union type cannot hold such a record. Tuples can.
- A field declared `ReadOnly<Function<...>> | null` (or optional) can be created and read, but not replaced.
- A function that calls a callback field of a record parameter cannot be used in a program region that throws or catches errors when the callback itself can fail; pass the callback as its own parameter there.

Current limitations: callable signatures containing other callable parameter/result types require further native layout support. Recursive by-value record/environment cycles require indirection support and currently report `callable-layout`. Direct arrays of callable values and joins of unannotated callable locals are separate integration steps. These limitations do not prevent replacing a field with distinct compatible environment origins.

Unused callable record declarations compile without creating an environment. Imported records retain private nominal parameter/result types; importing a record does not make those private type names visible in the caller.

Hover, completion details and signature help retain each field's declared argument names and ownership. Documentation comments above a field supply its description and `@param` help. Calls returning strings or other supported data types retain IntelliSense on subsequent member accesses, including generic fields and imported declarations. Unsaved changes to imported parameter names update the help.
