# Rest parameters

## Collect remaining arguments

A terminal `...values: T[]` parameter collects the remaining supplied arguments into a fresh owning array, in order. No remaining arguments produces an empty array. The annotation must be a supported array type. Its element type can include checked union alternatives, including null or undefined. Every supplied tail argument must fit that element type.

After native setup, save each complete example separately as `main.tsr` and run
`tsrust run main.tsr`. Comments show expected output.

```tsrust
function sum(...values: int[]): int {
  return values.reduce((total: int, value: int): int => {
    return total + value;
  }, 0);
}

function main() {
  console.log(sum(), sum(4), sum(1, 2, 3)); // 0 4 6
}
```

A rest parameter must be last. It cannot have `?`, a default initializer or a read/mutate access modifier. Its array owns the collected values. Callable annotations use the same rest syntax, such as `ReadOnly<Function<int, [...values: int[]]>>`.

## Fixed parameters and supplied values

Fixed parameters retain their required, optional and default behavior. All supplied arguments evaluate once, left to right, before reached fixed defaults run. A default does not add an element to the rest array. An explicit `undefined` in the tail is an actual element and must satisfy its element type.

```tsrust
function absent(...values: undefined[]): int {
  return values.length;
}

function main() {
  console.log(absent(), absent(undefined)); // 0 1
}
```

Calling an `int[]` rest parameter with `undefined` is a type error. Empty generic rest calls need enough type information to infer the element type; an explicit type argument can supply it.

## Ownership

Scalar leaves copy into the pack. Owned strings, records, nested arrays and stored callable environments transfer into it, following array-literal rules. Reading the array in the function does not clone those arguments. Explicitly clone an owner before passing it when an independent retained owner is needed. This differs from JavaScript object-reference sharing.

Changing the rest array changes its own collection. Fixed-prefix borrowing and mutation contracts remain unchanged. Strings retain native UTF-8 behavior; `string.length` counts bytes, and native integer limits remain in force. Parameter rebinding remains prohibited.

## Array callbacks

Array methods supply their existing argument tuples. A rest suffix receives only the supplied positions beyond its fixed prefix. Ordinary value visitors supply value, index and source array; reductions also supply the accumulator. Mutable callable-element visitors retain their shorter tuples and source-array loan restrictions. Rest packs cannot transfer a borrowed owner out of those tuples.

```tsrust
function main() {
  const result = [3, 1, 2].toSorted((...pair: int[]): int => {
    return pair[0] - pair[1];
  });
  console.log(result.join(",")); // 1,2,3
}
```

An optional or defaulted fixed callback slot beyond the supplied tuple remains omitted. Its padding never becomes a rest element. Callback defaults and pack creation happen only on actual invocation.

## Current boundaries

[Call spread](call-spread.md) expands arrays into rest and fixed parameters. Array-literal spread and supported iterable expansion have their own [spread rules](array-literal-spread.md). Rest parameters collect arguments into an array rather than inferring a heterogeneous tuple. Mixed and nullable elements follow the checked array element type. Borrowed callable environments cannot become owning rest-array elements.

## Collect a union element type

A rest array can contain different allowed alternatives. This example prints
`1,two,`: `join` renders its null element as empty text.

```tsrust
function show(...values: (int | string | null)[]): string {
  return values.join(",");
}

function main() {
  console.log(show(1, "two", null));
}
```
