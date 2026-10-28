---
title: Pattern defaults and rest
description: Supply missing values and copy remaining array, tuple or record contents.
---

Each complete example below runs separately as `main.tsr` with `tsrust run main.tsr`
after [native setup](/tsrust/tools/rust-setup/). Examples with `strictEqual` finish
without output; console examples show their output in comments.

## Default values

Use `=` after a binding name or nested pattern to select a fallback only when that selected value is undefined. Null, false, zero and empty text are present values and do not trigger a default. A default runs only when needed. Names are bound from left to right, so a later default can read an earlier name. Referring to the current or a later name before its declaration is an error.

```tsrust
function sum([first = 3, second = first + 2]: int[] = []): int {
  return first + second;
}

function main() {
  console.log(sum(), sum([7])); // 8 16
}
```

### Where defaults work

Defaults work in `const`/`let` declarations, function and arrow parameters, callbacks and `for...of` patterns.

Parameter initialization runs from left to right, including each pattern before the next parameter default; later parameter defaults can read earlier destructured names. State `ReadOnly<T>` for a later parameter that borrows an earlier owned name; that loan blocks transfers or mutation of the earlier source.

### Missing positions and fields

Defaults also work with renamed record fields and nested patterns, such as `const [[value] = [12]] = rows;`.

A missing array position or a tuple position outside the declared tuple selects its default. Positions without a default retain ordinary indexing/bounds behavior. A missing field of a fixed record can select a stated default; this does not add dynamic fields or prototype lookup. Source expressions are evaluated once.

```tsrust
type Box = {
  text?: string;
};

function take({ text = "new" }: Owned<Box>): string {
  return text;
}

function main() {
  console.log(take({}), take({
    text: "old"
  })); // new old
}
```

Defaults preserve source access. Copied values are copied, owning record/tuple positions can transfer, and ordinary array selections or readonly parameters remain borrowed. Explicit owned parameter patterns can transfer selected array elements.

### Keep borrowing boundaries

A freshly allocated fallback has local storage lasting through the borrowed binding's scope; it is not cloned. A default cannot mutate the same array while a selected value borrows it. Source loans remain lexical, even when the fallback is selected. Use an inner block to end the loan, or explicit `clone()` when independent ownership is needed.

Use `text?: string` when a record constructor may omit the field. Its value then starts as `undefined`, so a destructuring default applies. An explicit `text: string | undefined` also permits an undefined value, but the constructor must supply the field. Destructuring defaults do not remove other alternatives from a union and do not provide implicit coercion.

## Array rest copies

`const [first, ...others] = words` binds the first element under the usual source-access rules and creates an independent array containing the remaining elements. `const [...copy] = words` copies all elements. Skipped positions count toward the starting position; a rest start at or beyond the end returns an empty array. Rest copies nested owners as the existing slice operation does. Rest itself does not transfer the source. An `Owned<T>` parameter still consumes its argument at the call.

```tsrust
import { strictEqual } from "std:testing";

function main() {
  const words = ["a", "b"];
  const [...copy] = words;
  copy.push("c");
  strictEqual(words.length, 2);
  strictEqual(copy.length, 3);
}
```

### Rest types and ownership

The same syntax works in function/arrow parameters and for...of patterns. A named rest binding has independent owned storage, even when its source is readonly. Earlier borrowed positions retain their lexical source loan; copying the rest does not end that loan. Ordinary bindings still introduce no implicit clone.

Tuple rest produces a tuple of the remaining position types, including an empty tuple when no positions remain. Previously transferred fields before the rest start are not read; a transferred remaining field is an error. [Record spread](/tsrust/language/record-spread-assignment/) also supports explicit copies.

Nested owned values are copied independently, following the existing native slice/clone semantics. JavaScript rest copies only the outer array and shares nested object references.

## Record rest copies

`const { name: label, ...rest } = row` binds `row.name` as label and copies the other declared fields into an independent record. Exclusion uses the source field name, including renamed and nested selections. A missing field selected with a default is excluded without adding it to the copied record. A rest name must come last, without a default or trailing comma. The source must be a fixed record with cloneable remaining fields. Opaque native resources cannot be projected with record rest.

### Fields, order and ownership

Remaining field types are preserved. Empty rest is an empty record. Constructor order is preserved when inspecting keys or displaying the result.

Earlier selected fields may transfer according to source access; rest reads only the fields that remain. A transferred remaining field is an error.

Nested owned values copy independently, following the existing native clone semantics. Record rest works in declarations, function/arrow parameters and for...of patterns.

```tsrust
import { strictEqual } from "std:testing";

type Row = {
  name: string;
  count: int;
};

function main() {
  const row: Row = {
    count: 2, name: "ada"
  };
  const { name, ...rest } = row;
  strictEqual(name, "ada");
  strictEqual(rest.count, 2);
  strictEqual(Object.keys(rest).join(","), "count");
}
```

## Nested array-rest targets

An array rest target may itself be an array or object pattern. The rest copy is created once, then its pattern binds positions or properties under the ordinary access rules. For example, `const [first, ...[second, ...others]] = words` copies the tail, binds its first position as second, and explicitly copies its remaining positions as others. `const [...{ length }] = words` binds the copied array's length. Tuple rest targets retain each remaining position type.

Nested target positions may have defaults. The outer rest target still must be last, with no default or trailing comma. Pattern nesting keeps the existing limit of 32. Record rest continues to require a single identifier target.

```tsrust
import { strictEqual } from "std:testing";

function main() {
  const words = ["a", "b", "c"];
  const [first, ...[second, ...others]] = words;
  strictEqual(first, "a");
  strictEqual(second, "b");
  strictEqual(others.join(","), "c");
  strictEqual(words.length, 3);
}
```

## Related guides

- [Destructure values](/tsrust/language/destructuring/)
- [Record spread and pattern assignment](/tsrust/language/record-spread-assignment/)
