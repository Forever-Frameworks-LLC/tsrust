# Call spread

## Expand an iterable into arguments

`...values` in a call argument list expands a dense array, string, Set or Map into individual arguments, in iteration order. A call can mix ordinary arguments and any number of spread arguments. An empty source supplies no arguments.

After native setup, save each complete example separately as `main.tsr` and run
`tsrust run main.tsr`. Comments show expected output.

```tsrust
function total(first: int = 5, ...tail: int[]): int {
  let result = first;
  for (const value of tail) {
    result += value;
  }

  return result;
}

function main() {
  const values = [1, 2];
  console.log(total(0, ...values, 3), total(...values), total(...[])); // 6 3 5
}
```

The expanded values feed the target's existing fixed, optional, default and rest parameters. A default runs when its position receives no argument or receives explicit `undefined`: `total(...[])` supplies nothing, so `first` takes its default and the rest array is empty. Padding for an omitted parameter never becomes a rest element.

## Argument counts

An array literal has a known length, so the compiler checks the expanded count against the target. Other arrays, strings and collections have an expanded length checked when the program runs. The compiler then checks every position that the source could reach, and the program checks the actual count before the call.

```tsrust
function area(width: int, height: int): int {
  return width * height;
}

function main() {
  const size = [3, 4];
  console.log(area(...size)); // 12
}
```

`area(...[3, 4, 5])` is rejected with an `arity` error. Calling `area(...short)` with a one-element source throws a structured TypeError with code `call-arity`, message `expanded call arity outside checked contract` and the source byte range of the call. A catch handler can recover, and finally runs normally. The count check happens after all arguments evaluate and before the target, its defaults or its body run. Extra arguments are never dropped.

Every element that could reach a parameter must satisfy that parameter's type. A source that could reach parameters of incompatible types is rejected at the spread expression.

## Evaluation order

The callee or method receiver evaluates once, before any argument. Arguments and spread sources then evaluate once each, left to right. A spread reads its elements when it is reached, and all supplied values are held before the target runs.

```tsrust
function trace(log: int[], value: int): int {
  log.push(value);
  return value;
}

function pair(log: int[]): int[] {
  log.push(2);
  return [20, 21];
}

function sum(...values: int[]): int {
  let total = 0;
  for (const value of values) {
    total += value;
  }

  return total;
}

function main() {
  const log: int[] = [];
  console.log(sum(trace(log, 1), ...pair(log), trace(log, 3)), log.join(",")); // 45 1,2,3
    let optional: (ReadOnly<Function<int, [...values: int[]]>>) | null = null;
  console.log(String(optional?.(...pair(log))), log.length); // undefined 3
}
```

An optional call with an absent callee skips its arguments, including spread sources: `pair` does not run in the last line.

## Ownership

Scalar elements are copied. A parameter that reads its argument borrows the element in place, and a parameter that changes its argument changes the source element.

```tsrust
function grow(values: int[]): int {
  values.push(9);
  return values.length;
}

function main() {
  const rows: int[][] = [[1]];
  console.log(grow(...rows), rows[0].join(",")); // 2 1,9
}
```

Owned elements such as strings, records, nested arrays and stored callables transfer into a parameter that takes ownership, including a rest array. The source array is then no longer available. Clone the source first when it must stay usable.

```tsrust
function words(...values: string[]): string[] {
  return values;
}

function main() {
  const source = ["é".repeat(1), "x".repeat(1)];
  const kept = words(...clone(source));
  console.log(kept.join("|"), source.join("|")); // é|x é|x
    const moved = words(...source);
  console.log(moved.length); // 2
}
```

Using `source` after `words(...source)` is a `transferred` error. Nothing is cloned implicitly, which differs from JavaScript reference sharing. A borrowed element cannot move into an owning rest array, and a later argument cannot change an array that an earlier spread still borrows.

## Built-in calls

Spread works with built-in functions and methods that accept the expanded arguments under their existing contracts.

```tsrust
function main() {
  const numbers = [1.5, 3.0, 2.0];
  console.log(Math.max(...numbers), Math.min(0.5, ...numbers)); // 3 0.5
    console.log("a".concat(...["b", "c"])); // abc
    const list = [3];
  list.unshift(...[1, 2]);
  console.log(list.join(","), list.concat(...[[4], [5]]).join(","), Array.of<int>(...list, 4).join(",")); // 1,2,3 1,2,3,4,5 1,2,3,4
}
```

A built-in with a fixed argument count keeps that count: `list.push(...[2, 3])` is an `arity` error because `push` accepts one value. Console calls accept spread sources of one element type.

## Callable values

Spread calls work through named functions, generic and imported functions, closures, callable parameters, record fields holding callables, and nullable callables. Use `?.(...)` for an optional call, or narrow a nullable callable with `!== null` or `!== undefined` before calling it. A `Once<Function<...>>` callable cannot be invoked through `?.`; narrow it first.

## Current boundaries

Supported sources are dense arrays, strings, Sets and Maps. Named functions, methods and callable expressions keep their existing checked contracts. Generic inference uses expanded element types rather than the source collection type. Sparse slots and arbitrary iterator objects are unsupported. Heterogeneous tuple arrays and declared union elements retain ordinary parameter compatibility rules.

## String and collection arguments

Strings construct owning Unicode scalar character strings, including supplementary
characters. The source stays available, even when a parameter consumes its
character argument or captures it in a returned function. Combining marks remain
separate characters. Native string indexing and length still use UTF-8 bytes.

```tsrust
function join(a: string, b: string, c: string): string {
  return a + "|" + b + "|" + c;
}

function main() {
  const text = String("Aé🙂");
  console.log(join(...text), text); // prints A|é|🙂 Aé🙂
}
```

Sets supply insertion-ordered keys. Maps supply borrowed stored key/value tuples
for readonly parameters, including tuples with noncloneable values. A consuming
parameter or owning rest array transfers owning collection elements. Scalars copy.
The aggregate access required by reachable parameters governs the whole source,
as with array call spread. No implicit clone is added.

Mutable borrowing of stored Set keys or Map entry tuples is rejected, because
changing keys would invalidate collection lookup. Explicitly copy to an array
first when a detached mutable snapshot is wanted and its elements can be copied.
Readonly owning inputs cannot transfer; readonly scalar Sets can supply copies.

Expected parameter types can supply types for empty collection constructors.
String.raw additionally accepts string and Set<string> spread substitutions; it
keeps its existing explicit output-copy behavior and preserves source values.
Code constructors and variadic Math operations accept compatible Set elements.
Optional calls with absent targets skip sources; source failures skip later
arguments, and completion follows ordinary catch/finally rules.

## Supply undefined to a defaulted parameter

An explicit `undefined` remains a supplied argument for count checking. Once the
call is valid, the default initializes that slot. This example prints `9`:

```tsrust
function choose(value: int = 9): int {
  return value;
}

function main() {
  console.log(choose(...[undefined]));
}
```

This does not make `undefined` acceptable for a required `int` parameter without
a default. Nor does a skipped default create an extra rest element.
