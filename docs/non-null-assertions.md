# Non-null assertions

Postfix `!` selects the present payload of a nullable value and removes `null` and `undefined` from that expression's type.

```tsrust
const users = ['Alice', 'Bob', 'Charlie'];
const lastUser = users.pop()!;
console.log(lastUser.toUpperCase()); // CHARLIE
```

The assertion evaluates its operand once. You can continue a chain (`users.pop()!.toUpperCase()`), invoke a nullable callback (`action!()`), or access a nullable array (`values![0]`). An already nonnullable value keeps its type and ordinary behavior.

If the value is actually null or undefined, native execution fails with `Non-null assertion failed: value is null or undefined` and the assertion's source-byte range. This is a runtime check, unlike TypeScript's erased assertion. It does not supply a default. Use `users.pop() ?? 'Guest'` for a fallback, or a strict nullish guard when absence is expected. A statically absence-only or void value is rejected: there is no present payload to select.

Ownership: `!` does not clone, turn a borrow into an owner, or restore a moved value. Removing an owned element with `pop()` supplies an owned result. Selecting a named nullable owned value for a new owning binding transfers its payload; selecting it for readonly inspection borrows it. A readonly array-element view stays borrowed; use `clone(users.at(0)!)` when an independent owned string is needed.

The editor displays the asserted payload type, keeps method hover/completion available after assertions, and supports completion in unfinished chains such as `users.pop()!.to`.
