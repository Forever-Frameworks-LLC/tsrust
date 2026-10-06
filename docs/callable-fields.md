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

Bare callable field signatures own their environments. `ReadOnly<Function<...>>` and `Mutable<Function<...>>` borrow and cannot be stored in record fields. A stored function can move from one record field into another record. A borrowed local environment cannot be converted into an owned field merely by giving it a matching signature. Use an owned environment with independent captures.

Current limitations: callable signatures containing other callable parameter/result types require further native layout support. Recursive by-value record/environment cycles require indirection support and currently report `callable-layout`. Direct arrays of callable values and joins of unannotated callable locals are separate integration steps. These limitations do not prevent replacing a field with distinct compatible environment origins.

Unused callable record declarations compile without creating an environment. Imported records retain private nominal parameter/result types; importing a record does not make those private type names visible in the caller.

Hover, completion details and signature help retain each field's declared argument names and ownership. Documentation comments above a field supply its description and `@param` help. Calls returning strings or other supported data types retain IntelliSense on subsequent member accesses, including generic fields and imported declarations. Unsaved changes to imported parameter names update the help.
