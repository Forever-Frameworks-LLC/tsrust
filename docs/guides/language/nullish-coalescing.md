# Nullish coalescing

`left ?? fallback` evaluates `left` once. It evaluates `fallback` only when the left value is null or undefined. Zero, false and the empty string select the left value. Completed left-side effects remain if fallback evaluation fails.

After native setup, run each complete example separately as `main.tsr` with
`tsrust run main.tsr`. Expected output follows each example.

```tsrust
function main() {
  const zero: int | null = 0;
  const flag: boolean | undefined = false;
  const text: string | null = "";
  console.log(zero ?? 9, flag ?? true, (text ?? "fallback").length);
}
// prints: 0 false 0
```

## Choose a compatible result type

The result has a checked native type. A nullable payload and its fallback must have compatible checked types. Compatible literal-union fallbacks are supported: for example, a `"horizontal" | "vertical" | undefined` value may fall back to `"horizontal"`. The operator does not accept arbitrary new payload alternatives or coerce typed `int` and `Number` values. A fallback may retain an absent variant, such as `int | null` from an `int | null | undefined` input followed by `?? null`. A statically nonnullable input selects its existing type even when the unreachable fallback has another type. Statically null or undefined input selects the fallback type. A selected callable keeps its parameter, return and invocation contract. Nullable callable storage is also supported under the [callable ownership rules](nullable-callables.md).

## Combine operators explicitly

Use parentheses when combining `??` with `&&` or `||`. [Nullish assignment `??=`](nullish-assignment.md) is accepted with its own destination and readonly-result rules. [Optional chaining](optional-chaining.md) is accepted with lazy continuous-chain and scoped ownership rules. String content and existing integer, Number and collection limits remain unchanged.

## Select arrays and records

```tsrust
function main() {
  const input: int[] | undefined = undefined;
  const selected = input ?? [1, 2];
  console.log(selected.join("|"), selected.length);
}
// prints: 1|2 2
```

```tsrust
type Box<T> = {
  value: T;
};

function main() {
  const input: Box<int> | null = {
    value: 7
  };
  const selected = input ?? {
    value: 0
  };
  console.log(selected.value);
}
// prints: 7
```

## Narrow the absent branch

Inside a nullable input’s fallback, a stable named input or record-field path is known to be null or undefined. That absence fact applies only while evaluating the fallback; writes or conflicting effects invalidate it, and it does not make the original input nonnullable afterward.

The editor uses compiler-checked result spans for inferred declarations, grouped receivers and chained methods or fields. Invalid code and conflicting specializations remain conservative; a coalescing expression does not prove that its original input is nonnullable afterward.

## Keep an independent string

```tsrust
function main() {
  const input: string | null = "é".repeat(1);
  const selected = input ?? "fallback";
  const independent = clone(selected);
  console.log(selected, independent);
}
// prints: é é
```

## Select a callable

A selected callable keeps its checked parameter, return and invocation contract.

```tsrust
function main() {
  const selected = null ?? ((value: int): int => {
    return value + 1;
  });
  console.log(selected(4));
}
// prints: 5
```

```tsrust
function main() {
  const input = "once".repeat(1);
  const selected = null ?? ((): string => {
    return input;
  });
  console.log(selected());
}
// prints: once
```

## Preserve ownership boundaries

An owning coalescing result transfers an owned nullable input under the existing ownership contract; the old input cannot then be reused. Explicitly clone the input first when it must remain available. A scoped readonly input cannot become an owning result implicitly; readonly uses retain its existing loan restrictions. Copy results are snapshots, and callable invocation contracts remain in force.

Borrowed optional string fields can select a string literal fallback without cloning. See [borrowed optional fallbacks](../../../website/src/content/docs/language/borrowed-optional-fallback.md) for the current source-loan and independent-owner rules.
