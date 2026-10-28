# Destructuring

## Take positions and fields by name

A `const` or `let` declaration can name the positions of a tuple or array, or the fields of a record.

```ts
type Point = { x: int; y: int; label: string; };
function pair(): [int, string] { return [7, "seven"]; }
function main() {
    const [count, word] = pair();
    console.log(count + 1, word); // 8 seven
    const point: Point = { x: 1, y: 2, label: "p" };
    const { x, label: text } = point;
    console.log(x, text); // 1 p
}
```

- `[a, b]` takes positions 0 and 1. `[, b]` skips a position.
- `{ x, y }` takes the fields `x` and `y`. `{ label: text }` takes the field `label` under the name `text`.
- Patterns nest: `const [first, [inner, flag]] = value;` and `const { from: { label } } = line;`.
- `let` makes each name reassignable.

## Ownership follows the source

Ordinary pattern positions behave like taking each position from the value by hand. Rest syntax explicitly requests a copy of the remaining array elements.

- **A copied position** (int, number, boolean) is copied. The source stays usable.
- **An owned position of an owned source** moves out. After `const { label } = point;` the field `point.label` is transferred, as after `const label = point.label;`. The other fields stay usable. Write `clone(point)` to keep the whole source.
- **A call result or other temporary** is held once, and each name takes its position from it.
- **A `ReadOnly` parameter** lends its positions. The names are readonly borrows and cannot be transferred.

```ts
function describe(point: ReadOnly<Point>): string {
    const { x, y, label } = point;
    return label + String(x + y);
}
```

## In for...of

A loop can destructure each element. The names borrow the element for reading, as the loop variable does.

```ts
type Row = { id: int; name: string; };
function main() {
    const row: Row = { name: "ada", id: 7 };
    for (const [key, value] of Object.entries(row)) { console.log(key, String(value)); }
    // name ada
    // id 7
    const rows: Row[] = [row];
    for (const { id, name } of rows) { console.log(id, name.toUpperCase()); } // 7 ADA
}
```

Pass a borrowed name to a `ReadOnly` parameter, read it, or `clone()` it. Transferring it is a `borrowed-place` error.

The same rule now applies to a plain `const` inside a loop: `const name = item.name;` borrows the field for reading.

## In parameters

A function or arrow parameter can be a pattern. Its names open the body.

```ts
type Point = { x: int; y: int; label: string; };
function describe({ x, y, label }: Point): string { return label + String(x + y); }
function main() {
    const point: Point = { x: 1, y: 2, label: "p" };
    console.log(describe(point), point.label); // p3 p
    const pairs: [string, int][] = [["a", 1], ["b", 2]];
    console.log(pairs.map(([name, amount]) => name.repeat(amount)).join("")); // abb
}
```

A destructured parameter with no stated access is read: its names borrow the argument, and the caller keeps the value. State `Owned<Point>` to let the names move out; the call then consumes its argument.

### Take elements from an owned array parameter

An `Owned<T[]>` parameter pattern can transfer selected elements, including nested
array elements, without cloning them. The caller gives up the array. A plain
`T[]` or `ReadOnly<T[]>` parameter pattern still borrows its owned elements.

After native setup, save this complete example as `main.tsr` and run
`tsrust run main.tsr`. It prints `owner`:

```tsrust
function head([first]: Owned<string[]>): string {
  return first;
}

function main() {
  console.log(head(["owner", "extra"]));
}
```

Skipped positions and defaults keep their original positions. A rest binding
still explicitly copies the remaining elements; ordinary selected elements move.

## Arrays

In a local declaration, array positions follow indexing. A copied element is copied. An owned element is borrowed for reading, and the array cannot be changed while the name is in scope. The explicit `Owned<T[]>` parameter pattern above is different: it can transfer its selected elements.

```ts
function main() {
    const words = ["alpha", "be"];
    {
        const [first, second] = words;
        console.log(first, second.length); // alpha 2
    }
    words.push("gamma");
}
```

A position past the end of an array stops the program at run time, as indexing does. The same borrow applies to a plain `const word = words[0];`.

## Errors

- A tuple position outside the declared tuple or an undeclared record field without a default is a `bounds` or `type` error respectively.
- Transferring a borrowed name is a `borrowed-place` error. Changing or transferring the source while a borrowed name is in scope is a `view-conflict` error; put the borrow in an inner block, or `clone()` the part.
- An array rest name must be last and cannot have a default or trailing comma. A pattern declaration is not accepted in the update clause of a classic `for` loop.

Borrowed optional string fields can use a string literal fallback without cloning: `const label = item?.label ?? "none";`. See [borrowed optional fallbacks](borrowed-optional-fallback.md) for ownership and lexical source loans.

## Default values

Use `=` after a binding name or nested pattern to select a fallback only when that selected value is undefined. Null, false, zero and empty text are present values and do not trigger a default. A default runs only when needed. Names are bound from left to right, so a later default can read an earlier name. Referring to the current or a later name before its declaration is an error.

```ts
function sum([first = 3, second = first + 2]: int[] = []): int {
    return first + second;
}
function main() {
    console.log(sum(), sum([7])); // 8 16
}
```

Defaults work in const/let declarations, function and arrow parameters, callbacks and for...of patterns. Parameter initialization runs from left to right, including each pattern before the next parameter default; later parameter defaults can read earlier destructured names. State ReadOnly<T> for a later parameter that borrows an earlier owned name; that loan blocks transfers or mutation of the earlier source. They also work with renamed record fields and nested patterns, such as `const [[value] = [12]] = rows;`. A missing array position or a tuple position outside the declared tuple selects its default. Positions without a default retain ordinary indexing/bounds behavior. A missing field of a fixed record can select a stated default; this does not add dynamic fields or prototype lookup. Source expressions are evaluated once.

```ts
type Box = { text: string | undefined; };
function take({ text = "new" }: Owned<Box>): string { return text; }
function main() {
    console.log(take({ text: undefined }), take({ text: "old" })); // new old
}
```

Defaults preserve source access. Copied values are copied, owning record/tuple positions can transfer, and ordinary array selections or readonly parameters remain borrowed. Explicit owned parameter patterns can transfer selected array elements. A freshly allocated fallback has local storage lasting through the borrowed binding's scope; it is not cloned. A default cannot mutate the same array while a selected value borrows it. Source loans remain lexical, even when the fallback is selected. Use an inner block to end the loan, or explicit clone() when independent ownership is needed.

Declared record fields that may hold undefined use an explicit union such as `text: string | undefined`; optional `text?` field annotations remain unsupported. Destructuring defaults do not remove other alternatives from a union and do not provide implicit coercion.

## Array rest copies

`const [first, ...others] = words` binds the first element under the usual source-access rules and creates an independent array containing the remaining elements. `const [...copy] = words` copies all elements. Skipped positions count toward the starting position; a rest start at or beyond the end returns an empty array. Rest copies nested owners as the existing slice operation does. The source is not transferred.

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

The same syntax works in function/arrow parameters and for...of patterns. A named rest binding has independent owned storage, even when its source is readonly. Earlier borrowed positions retain their lexical source loan; copying the rest does not end that loan. Ordinary bindings still introduce no implicit clone. Tuple rest produces a tuple of the remaining position types, including an empty tuple when no positions remain. Previously transferred fields before the rest start are not read; a transferred remaining field is an error. Fixed-record object spread also supports explicit copies, described below.

Nested owned values are copied independently, following the existing native slice/clone semantics. JavaScript rest copies only the outer array and shares nested object references.

## Record rest copies

`const { name: label, ...rest } = row` binds `row.name` as label and copies the other declared fields into an independent record. Exclusion uses the source field name, including renamed and nested selections. A missing field selected with a default is excluded without adding it to the copied record. A rest name must come last, without a default or trailing comma. The source must be a record.

Remaining field types are preserved. Empty rest is an empty record. Constructor order is preserved when inspecting keys or displaying the result. Earlier selected fields may transfer according to source access; rest reads only the fields that remain. A transferred remaining field is an error. Nested owned values copy independently, following the existing native clone semantics. Record rest works in declarations, function/arrow parameters and for...of patterns.

```tsrust
import { strictEqual } from "std:testing";
type Row = { name: string; count: int; };
function main() {
    const row: Row = { count: 2, name: "ada" };
    const { name, ...rest } = row;
    strictEqual(name, "ada");
    strictEqual(rest.count, 2);
    strictEqual(Object.keys(rest).join(","), "count");
}
```

## Object spread copies and merges

`const copied = { ...row }` creates an independent record with the source's fields. Sources must be fixed records; array, string, primitive, nullable and union sources are unsupported. Ordinary fields may surround multiple spreads. Parts evaluate once, left to right. Later values replace earlier values of the same name, while each key keeps its first insertion position. An annotation checks the final fields and types, including supported union widening. Without an annotation, the final fixed shape is inferred. Empty records and inline record sources are supported.

```tsrust
import { strictEqual } from "std:testing";
type Row = { name: string; count: int; };
function main() {
    const row: Row = { count: 2, name: "ada" };
    const copied = { ...row, count: 3 };
    strictEqual(copied.count, 3);
    strictEqual(row.count, 2);
    strictEqual(Object.keys(copied).join(","), "count,name");
}
```

A spread source is borrowed while copying; copying finishes before evaluating the next part. Sources remain available, including readonly parameters. A partially transferred source cannot be copied whole. Nested owners copy independently under native clone semantics; JavaScript shares nested object references. Ordinary fields retain their usual access rules and do not gain implicit cloning.

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

## In classic for loops

The initializer of a classic `for` loop can be a pattern declaration. The source expression is evaluated once, before the first condition check. The names are in scope for the condition, the update clause and the body, and end with the loop.

```tsrust
import { strictEqual } from "std:testing";
type Bounds = { start: int; end: int; };
function bounds(): Bounds { return { start: 1, end: 4 }; }
function main() {
    const seen: int[] = [];
    for (let { start: i, end } = bounds(); i < end; i++) {
        if (i === 2) { continue; }
        seen.push(i);
    }
    strictEqual(seen.join(","), "1,3");
}
```

Use `let` for names that the update clause or body changes. Defaults, rest and nested patterns work as in other declarations: `for (let [i, end = 3] = [0]; i < end; i++)` runs three times. `continue` runs the update clause and `break` leaves the loop as usual.

Access follows the source. A named source keeps the positions that were copied or not taken. A rest name is an independent copy, so `for (const [first, ...tail] = words; tail.length < 2; tail.push("c"))` changes only `tail`. A name that borrows an array element keeps the array loaned until the loop ends; the array can be changed again after the loop.

The names share one binding for the whole loop. They are not created again for each iteration. A name cannot repeat a name of the enclosing scope. A declaration list can mix patterns and ordinary names under one `const` or `let`
keyword. Initializers run left to right. Run this complete example separately;
it prints `3`:

```tsrust
function main() {
  let total = 0;
  for (let [i, end] = [0, 3], step = 1; i < end; i += step) {
    total += i;
  }

  console.log(total);
}
```

See [declarations](../../../website/src/content/docs/language/declarations.md) for
initialization, scope and declaration-list errors.

## Assignment to existing places

A pattern on the left of `=` assigns to names and places that already exist. No new name is declared.

```tsrust
import { strictEqual } from "std:testing";
type Point = { x: int; y: int; label: string; };
function main() {
    let a = "x".repeat(1), b = "y".repeat(1);
    [a, b] = [b, a];
    strictEqual(a + b, "yx");
    const point: Point = { x: 1, y: 2, label: "p" };
    let x = 0, text = "";
    ({ x, label: text } = point);
    strictEqual(x, 1);
    strictEqual(text, "p");
}
```

- A target is anything a plain assignment accepts: a `let` name, a record field such as `point.x`, or an array element such as `values[0]`. A `const` name is rejected.
- A record pattern at the start of a statement needs parentheses: `({ x, y } = point);`. In a record pattern, `{ x }` assigns the field `x` to the name `x`, and `{ label: text }` assigns the field `label` to `text`.
- Patterns nest, and an array pattern may end with a rest target: `[first, ...rest] = values;` assigns an independent copy of the remaining elements.
- Default values and record rest targets are not available in an assignment. Declare with `const` or `let` to use them.

The right side is evaluated once, before any target changes. When the right side is a written array, each value is held first and then assigned in order, so `[a, b] = [b, a]` swaps two values without building an array. Owned values move through the swap; nothing is cloned. A written array with fewer values than targets is a `bounds` error.

For any other right side, each target takes its position or field under the ordinary rules: copied values are copied, a tuple or record source gives up the owned positions that are taken, and an owned array element cannot be taken by position. A named source keeps the positions that are not taken.

A destructuring assignment is a statement. It can also be one of the update expressions of a classic `for` loop: `for (let i = 0; i < n; [a, b] = [b, a + b], i++)`. It has no value, so `const t = ([a, b] = [b, a]);` is an error.

## Shorthand record properties

In a record literal, a name alone stands for a field of that name with the value of that variable: `{ x, y, label }` means `{ x: x, y: y, label: label }`. The variable is used under the ordinary rules, so an owned value moves into the record.

```tsrust
import { strictEqual } from "std:testing";
type Point = { x: int; y: int; label: string; };
function main() {
    const x = 3, y = 4, label = "p".repeat(2);
    const point: Point = { x, y, label };
    strictEqual(point.x + point.y, 7);
    strictEqual(point.label, "pp");
}
```
