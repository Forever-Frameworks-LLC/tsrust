# The `in` operator and `for...in`

`key in value` tests whether a record, tuple or array has a key. `for (const key in value)` visits the keys.

```tsrust
import { strictEqual } from "std:testing";

type Point = { x: int; y: int; };

function main() {
    const point: Point = { x: 1, y: 2 };
    strictEqual("x" in point, true);
    strictEqual("z" in point, false);

    const names: string[] = [];
    for (const key in point) {
        names.push(clone(key));
    }
    strictEqual(names.join(","), "x,y");

    const values = [5, 6, 7];
    strictEqual(1 in values, true);
    strictEqual(3 in values, false);
    let indices = "";
    for (const index in values) {
        indices += index;
    }
    strictEqual(indices, "012");
}
```

## `key in value`

The result is a boolean. It is the same own-key test as `Object.hasOwn(value, key)` for supported receivers. Inherited names such as `"toString"` are absent. JavaScript also checks inherited properties with `in`.

- For a record, the keys are its field names. Every field of a record type is always present, so the result depends only on the type and the key text.
- For an array or tuple, the keys are the positions `0` to `length - 1`, as numbers or as decimal strings, and `"length"`.
- The key is a string, an int or a number. The value is a record, a tuple or an array. Other types are a `type` error.
- The key evaluates before the value. Both sides evaluate once. Ordinary borrowing rules reject mutation or transfer of a key while the value is evaluated.
- `in` binds like `<` and `>=`: `"x" in point && ready` tests the key first, and `!("x" in point)` needs the parentheses.

## `for (const key in value)`

The loop visits the same keys as `Object.keys(value)`, in the same order, as strings: field names in constructor order for a record, and `"0"`, `"1"`, ... for an array, tuple or string.

- The binding is declared with `const` or `let`. A destructuring pattern or an existing variable in that place is `unsupported`.
- `break` and `continue` work as in other loops. The body can be a single statement.
- The key is a `string`. To read the element at an array position, use `for...of` or a classic `for` loop with an int index.
- The value is read once to collect a snapshot of its keys. Changes to the source during the loop do not change the collected keys. Both `const` and `let` loop keys borrow each collected string and cannot be reassigned; use `clone(key)` when storing an owned copy. Primitive numbers and booleans have no enumerable keys.
