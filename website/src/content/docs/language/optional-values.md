---
title: Optional values
description: Rules for null and undefined, optional chaining, nullish fallbacks and nullish assignment.
---

A value is *absent* when it is `null` or `undefined`. Every other value is present,
including `0`, `false` and the empty string. Three operators act only on absent
values:

| Operator | Meaning |
| --- | --- |
| `base?.member` | Skip the rest of the chain when `base` is absent; the result is `undefined`. |
| `left ?? fallback` | Use `fallback` only when `left` is absent. |
| `target ??= fallback` | Assign `fallback` only when `target` is absent. |

None of them converts types, catches errors or copies owned values implicitly.

```tsrust
function main() {
  const zero: int | null = 0;
  const flag: boolean | undefined = false;
  const text: string | null = "";
  console.log(zero ?? 9, flag ?? true, (text ?? "fallback").length);
}
// prints: 0 false 0
```

To run an example, save it as `main.tsr` after [native setup](../../tools/rust-setup/)
and run `tsrust run main.tsr`.

## Optional chaining

`?.` evaluates its base once. When the base is absent, the result is `undefined`
and the rest of the continuous chain is skipped, including computed indices and
call arguments. When the base is present, the ordinary checked operation runs.

| Form | Example |
| --- | --- |
| Field | `row?.label` |
| Index | `values?.[index]` |
| Call | `callback?.(argument)` |
| Method | `text?.repeat(count)` |

```tsrust
type Row = {
  n: int;
};

function count(trace: int[]): int {
  trace.push(1);
  return 0;
}

function main() {
  const absent: Row | null = null;
  const present: Row | null = { n: 7 };
  console.log(String(absent?.n), present?.n ?? 0);

  const trace: int[] = [];
  const none: int[] | undefined = undefined;
  const some: int[] | undefined = [5];
  console.log(String(none?.[count(trace)]), trace.length, some?.[count(trace)] ?? 0, trace.length);
}
// prints: undefined 7
// prints: undefined 0 5 1
```

The index expression `count(trace)` runs only for the present array.

### Where a chain ends

A chain continues through later fields, indices and calls until a parenthesis
ends it. `value?.text.length` skips both accesses when `value` is absent.
`(value?.text).length` reads `length` from a possibly undefined result, so it
needs a guard or fallback. A nullable field later in the chain needs its own `?.`.

The result type adds `undefined` only when the absent branch is reachable. Existing
`null` alternatives of the selected member stay in the result:

```tsrust
type Holder = {
  text: string | null;
};

function main() {
  const holder: Holder | undefined = { text: null };
  const selected = holder?.text;
  console.log(String(clone(selected) ?? "empty"), Boolean(selected));
}
// prints: empty false
```

`?.` does not narrow the base for later statements.

### Rules

- `?.` needs one proven present type. Narrow a union of different record shapes
  before selecting a member.
- Reached operations keep their checks. `rows[0]?.label` indexes first, so an empty
  array still fails its bounds check; `?.` tests the element, not the index.
  String operations keep their UTF-8 contracts.
- `?.` does not catch a failure in a reached operation or an evaluated argument.
- Optional chains are not assignment targets.
- Nullable callable storage and optional invocation follow the
  [callable ownership rules](../nullable-callables/).

## Nullish fallback

`left ?? fallback` evaluates `left` once. It evaluates `fallback` only when `left`
is absent. If evaluating `fallback` fails, completed side effects from `left` remain.

```tsrust
type Box<T> = {
  value: T;
};

function main() {
  const input: int[] | undefined = undefined;
  const selected = input ?? [1, 2];
  console.log(selected.join("|"), selected.length);

  const box: Box<int> | null = { value: 7 };
  console.log((box ?? { value: 0 }).value);
}
// prints: 1|2 2
// prints: 7
```

### Result type

- The present type of `left` and the type of `fallback` must be compatible. A
  literal-union value such as `"horizontal" | "vertical" | undefined` may fall back
  to `"horizontal"`.
- `??` does not add unrelated alternatives or convert between `int` and `number`.
- A fallback can keep an absent variant: `int | null | undefined` followed by
  `?? null` gives `int | null`.
- A statically nonnullable `left` keeps its own type, even when the unreachable
  fallback has another type. A statically absent `left` gives the fallback type.
- A selected callable keeps its parameter, return and invocation contract.

### Narrowing inside the fallback

While the fallback is evaluated, a stable named input or field path is known to be
absent. Writes or conflicting effects end that fact, and `??` does not make the
original input nonnullable afterwards.

### Mixing with `&&` and `||`

Use parentheses when combining `??` with `&&` or `||`.

## Nullish assignment

`target ??= fallback` selects the destination once and reads its old value once.
It evaluates and stores the fallback only when the old value is absent. The
expression returns the value the destination now holds.

```tsrust
type Row = {
  n: int;
};

function main() {
  let count: int | null = 0;
  let flag: boolean | undefined = false;
  console.log(count ??= 9, flag ??= true);

  let values: int[] | undefined = undefined;
  const selected = (values ??= [1, 2]);
  console.log(selected.join("|"), values.length);

  let row: Row | null = null;
  console.log((row ??= { n: 7 }).n, row.n);

  const slots: (int | null)[] = [null, 2];
  console.log(slots[0] ??= 7, slots[1] ??= 9, slots.join(","));
}
// prints: 0 false
// prints: 1|2 2
// prints: 7 7
// prints: 7 2 7,2
```

### Destinations

- The target must be a writable place under the usual mutability and loan rules:
  a variable, a field or an array element. A `const` array still has mutable
  elements.
- Arguments, selection and index bounds are evaluated before the fallback. An
  indexed destination is reacquired after the fallback's effects.
- If the fallback fails, nothing is assigned; its completed side effects remain.
- Both the old and the fallback value must fit the destination's declared type.
  Unlike `??`, even an unreachable fallback must fit: a nonnullable `string`
  target rejects a numeric fallback.

### Narrowing after assignment

A stable nullable target is absent inside its fallback. After the assignment, the
target loses the absent alternatives the fallback cannot produce: a `string`
fallback yields `string`; a `string | null` fallback may keep `null`. Conflicting
writes end that fact. Dynamic selections and repeated calls are not stable targets.

```tsrust
function main() {
  let value: int | null | undefined = undefined;
  const selected = (value ??= null);
  value = 4;
  console.log(String(selected), String(value));
}
// prints: null 4
```

### Chains

Copy chains work directly: `a ??= b ??= 7`. Owned chains need an explicit clone of
the inner result so each destination holds its own owner:
`a ??= clone(b ??= "x".repeat(1))`.

## Ownership of selected values

The three operators follow one rule: they never duplicate an owner implicitly.

- **Copy values** (`int`, `number`, `boolean`) are snapshots.
- **Existing owned values** selected by `?.` or `??` become scoped readonly views.
  You can read the source; mutating, moving, returning or capturing it is rejected
  while the view is in scope.
- **`??=` results** are readonly views of the value kept in the destination. A
  binding of that result is readonly even when declared with `let`, and its loan
  lasts until the enclosing scope ends.
- **Fresh values**, such as array literals or calls returning new storage, are owned
  by the result.
- **An owned nullable input** passed to an owning `??` result is transferred and
  cannot be reused. Clone it first to keep it.
- **`clone(...)`** makes an independent owner from any selection.

```tsrust
function main() {
  const values: string[] | null = ["é".repeat(1)];
  const selected = values?.[0];
  const independent = clone(selected);
  console.log(String(selected), String(independent), values?.length ?? 0);
}
// prints: é é 1
```

### Borrowed text with a fallback

`row?.label ?? "none"` borrows the field when it is present and uses the static
literal otherwise. Neither branch copies text.

```tsrust
type Row = {
  label: string;
};

function main() {
  const rows: (Row | null)[] = [{ label: "first" }, null, { label: "" }];
  for (const item of rows) {
    const label = item?.label ?? "none";
    console.log(label.toUpperCase());
  }
}
// prints: FIRST
// prints: NONE
// prints an empty line
```

While `label` is in scope, `rows` cannot be changed or transferred, even when the
fallback was selected. End the borrow with an inner block, or clone the text:

```tsrust
import { strictEqual } from "std:testing";

type Row = {
  label: string;
};

function main() {
  const rows: (Row | null)[] = [null];
  {
    const label = rows[0]?.label ?? "none";
    strictEqual(label, "none");
  }
  rows.push({ label: "next" });

  const kept = clone(rows[1]?.label ?? "none");
  rows.push(null);
  strictEqual(kept, "next");
  strictEqual(rows.length, 3);
}
```

A borrowed fallback must come from a stable source. A freshly allocated string
cannot be selected into the borrowed view: `item?.label ?? "new".repeat(2)` is
rejected with `borrowed-view`. Bind the computed text first:

```tsrust
type Row = {
  label: string;
};

function main() {
  const rows: (Row | null)[] = [null];
  const fallback = "new".repeat(2);
  const label = rows[0]?.label ?? fallback;
  console.log(label);
}
// prints: newnew
```

`fallback` is computed whether or not it is needed.

## Editor help

Hover shows the checked type of the whole expression. Completion after `value?.`
lists members when the receiver has a known present type. A suggestion after `?.`
does not mean that plain `value.` access is safe.

## Related

- [Optional record fields](../optional-fields/)
- [Pattern defaults and rest](../pattern-defaults-rest/)
- [Nullable callables](../nullable-callables/)
