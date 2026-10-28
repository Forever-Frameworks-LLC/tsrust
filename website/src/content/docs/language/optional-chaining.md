---
title: Use optional chaining
description: Select fields, indices and calls while skipping absent receivers.
---

`?.` evaluates its base once. A null or undefined base produces undefined and skips the remaining continuous chain, including computed indices and call arguments. A present base follows the ordinary checked operation. Zero, false and empty strings are present values. Fields use `base?.field`, indices use `base?.[index]`, and calls use `callee?.(arguments)`; optional method access and optional invocation can be combined.

## Keep or end a chain

Parentheses end a continuous chain. `value?.text.length` skips both accesses when value is absent; `(value?.text).length` accesses a potentially undefined result and needs a guard or fallback. A later nullable field requires another optional hop. The result adds undefined only when an absent branch is reachable and flattens existing nullable alternatives. Optional access does not establish a nonnullable fact for later statements.

## Preserve checked errors

Indices and arguments retain their ordinary strict types and ranges. Reached array accesses still check bounds; reached String operations keep their UTF-8 byte/scalar contracts and boundary errors. Optional access does not catch failures in a reached operation or evaluated argument. An optional hop needs one proven present payload type. Narrow a union with several present types before selecting a member. Callable access and invocation contracts still apply. Optional chaining does not add coercion, arbitrary mixed-union operations, optional assignment targets or prototype lookup. Nullable callable storage and optional invocation are supported under their [callable ownership contracts](/tsrust/language/nullable-callables/).

## Editor help

Hover shows the checked result type of the whole expression. Completion after `value?.` offers members when the receiver has a known valid payload. Unknown or invalid receivers may have no suggestions. A suggestion after `?.` does not establish that ordinary `value.` access is safe.

After [native setup](/tsrust/tools/rust-setup/), run each complete example
separately as `main.tsr` with `tsrust run main.tsr`.

## Read an optional field

```tsrust
type OptionalRow = {
  n: int;
};

function main() {
  const absent: OptionalRow | null = null;
  const present: OptionalRow | null = {
    n: 7
  };
  console.log(String(absent?.n), present?.n ?? 0);
}
```

Expected stdout: `undefined 7`.

## Skip a computed index

```tsrust
function optionalIndex(trace: int[]): int {
  trace.push(1);
  return 0;
}

function main() {
  const trace: int[] = [];
  const absent: int[] | undefined = undefined;
  const present: int[] | undefined = [5];
  console.log(String(absent?.[optionalIndex(trace)]), trace.length, present?.[optionalIndex(trace)] ?? 0, trace.length);
}
```

Expected stdout: `undefined 0 5 1`.

## Skip method arguments

```tsrust
function optionalCount(trace: int[]): int {
  trace.push(1);
  return 1;
}

function main() {
  const trace: int[] = [];
  const absent: string | null = null;
  const present: string | null = "é".repeat(1);
  console.log(String(absent?.repeat(optionalCount(trace))), trace.length, String(present?.repeat(optionalCount(trace))), trace.length);
}
```

Expected stdout: `undefined 0 é 1`.

## Continue through a field

```tsrust
type OptionalTextRow = {
  text: string;
};

function main() {
  const absent: OptionalTextRow | null = null;
  const present: OptionalTextRow | null = {
    text: "é"
  };
  console.log(String(absent?.text.length), present?.text.length ?? 0);
}
```

Expected stdout: `undefined 2`.

## Borrow an array selection

```tsrust

function main() {
  const values: string[] | null = ["é".repeat(1)];
  const selected = values?.[0];
  const independent = clone(selected);
  console.log(String(selected), String(independent), values?.length ?? 0);
}
```

Expected stdout: `é é 1`.

## Keep existing null alternatives

```tsrust
type OptionalHolder = {
  text: string | null;
};

function main() {
  const holder: OptionalHolder | undefined = {
    text: null
  };
  const selected = holder?.text;
  console.log(String(clone(selected) ?? "empty"), Boolean(selected));
}
```

Expected stdout: `empty false`.

## Understand ownership

Copy selections are snapshots. Existing owned fields or indexed values can produce scoped readonly views: reading the source remains allowed, while conflicting mutation, moves, escaping returns and captures follow its checked loans. An explicit canonical nullable annotation can preserve the same scoped readonly view; it does not create an owner. Such views cannot escape through a return or an argument that consumes ownership. Use `clone()` for an independent owner. Fresh receivers may transfer selected values, and methods creating new storage return owned results; no implicit owner duplication is added.

Borrowed optional string fields can select a string literal fallback without cloning. See [borrowed optional fallbacks](/tsrust/language/borrowed-optional-fallback/) for the current source-loan and independent-owner rules.
