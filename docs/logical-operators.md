# Value selection with logical operators

`left || right` returns the left value when it is truthy; otherwise it evaluates and returns the right value. `left && right` returns the left value when it is falsy; otherwise it evaluates and returns the right value. The left expression runs once, and the right expression runs only when selected.

```ts
const users = ['Alice', 'Bob', 'Charlie'];
const userOne = users.shift();
const alice = userOne || 'Alice'; // string: 'Alice'
const empty = '' || 'Guest'; // 'Guest'
const preserved = '' ?? 'Guest'; // ''
```

Falsy values are `false`, zero, `NaN`, the empty string, `null`, and `undefined`. Arrays and records are truthy, including empty ones. Whitespace strings are truthy. `??` selects its fallback only for null or undefined.

Results follow the compiler's existing union restrictions: a payload type can be combined with null and undefined, but arbitrary mixed payload types such as boolean and string are not supported. Unsupported combinations receive a `logical-result` diagnostic. Void operands cannot be selected. Logical assignment (`&&=` and `||=`) retains its existing boolean restriction; this change does not expand condition syntax.

Reading a result borrows the selected owned value. Storing an owned result transfers the selected payload without an implicit clone. Conditional transfers retain the compiler's availability checks; use explicit cloning when an owned source must remain available.
