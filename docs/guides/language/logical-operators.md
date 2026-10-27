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

Results follow the compiler's existing union restrictions: a payload type can be combined with null and undefined, but arbitrary mixed payload types such as boolean and string are not supported. Unsupported combinations receive a `logical-result` diagnostic. Void operands cannot be selected.

Conditions in `if`, loops and conditional expressions use the same truthiness rules. `!value` returns the opposite truth value; `!!value` returns its truth value. Testing an owned value borrows it. Testing a nullable variable or fixed record field also narrows it in the branch where it is present.

`target ||= value` stores the right value only when the target is falsy. `target &&= value` stores it only when the target is truthy. A skipped right expression does not run. Non-boolean targets currently require a variable or fixed record field; computed targets are unsupported. The right value must fit the target type. The result reads the target after the optional store; binding an owned result transfers the target, while read-only use leaves it available.

```ts
let name = '';
name ||= 'Guest'; // name is 'Guest'
name &&= 'Member'; // name is 'Member'
if (name) console.log(name.length); // 6
```

Reading a result borrows the selected owned value. Storing an owned result transfers the selected payload without an implicit clone. Conditional transfers retain the compiler's availability checks; use explicit cloning when an owned source must remain available.
