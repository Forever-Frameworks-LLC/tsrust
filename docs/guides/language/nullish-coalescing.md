# Nullish coalescing

`left ?? fallback` evaluates `left` once. It evaluates `fallback` only when the left value is null or undefined. Zero, false and the empty string select the left value. Completed left-side effects remain if fallback evaluation fails.

```tsrust
function main() {
    const zero: int | null = 0;
    const flag: boolean | undefined = false;
    const text: string | null = "";
    console.log(zero ?? 9, flag ?? true, (text ?? "fallback").length);
}
// prints: 0 false 0
```

The result has a checked native type. A nullable payload and its fallback must fit the supported single-payload union domain; the operator does not introduce a heterogeneous union or coerce typed int and Number values. A fallback may retain an absent variant, such as `int | null` from an `int | null | undefined` input followed by `?? null`. A statically nonnullable input selects its existing type even when the unreachable fallback has another type. Statically null or undefined input selects the fallback type. Existing callable values can therefore remain callable without creating a nullable callable type.

Use parentheses when combining `??` with `&&` or `||`. [Nullish assignment `??=`](nullish-assignment.md) is accepted with its own destination and readonly-result rules. [Optional chaining](optional-chaining.md) is accepted with lazy continuous-chain and scoped ownership rules. String content and existing integer, Number and collection limits remain unchanged.

```tsrust
function main() {
    const input: int[] | undefined = undefined;
    const selected = input ?? [1, 2];
    console.log(selected.join("|"), selected.length);
}
// prints: 1|2 2
```

```tsrust
type Box<T> = { value: T; };
function main() {
    const input: Box<int> | null = { value: 7 };
    const selected = input ?? { value: 0 };
    console.log(selected.value);
}
// prints: 7
```

Inside a nullable input’s fallback, a stable named input or record-field path is known to be null or undefined. That absence fact applies only while evaluating the fallback; writes or conflicting effects invalidate it, and it does not make the original input nonnullable afterward.

The editor uses compiler-checked result spans for inferred declarations, grouped receivers and chained methods or fields. Invalid code and conflicting specializations remain conservative; a coalescing expression does not prove that its original input is nonnullable afterward.

```tsrust
function main() {
    const input: string | null = "é".repeat(1);
    const selected = input ?? "fallback";
    const independent = clone(selected);
    console.log(selected, independent);
}
// prints: é é
```

A selected callable keeps its checked parameter, return and invocation contract.

```tsrust
function main() {
    const selected = null ?? ((value: int): int => { return value + 1; });
    console.log(selected(4));
}
// prints: 5
```

```tsrust
function main() {
    const input = "once".repeat(1);
    const selected = null ?? (() : string => { return input; });
    console.log(selected());
}
// prints: once
```

An owning coalescing result transfers an owned nullable input under the existing ownership contract; the old input cannot then be reused. Explicitly clone the input first when it must remain available. A scoped readonly input cannot become an owning result implicitly; readonly uses retain its existing loan restrictions. Copy results are snapshots, and callable invocation contracts remain in force.

Borrowed optional string fields can select a string literal fallback without cloning. See [borrowed optional fallbacks](../../../website/src/content/docs/language/borrowed-optional-fallback.md) for the current source-loan and independent-owner rules.
