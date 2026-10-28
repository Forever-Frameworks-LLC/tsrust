---
title: Write arrow callbacks
description: Use contextual parameter types, expression bodies and checked captures.
---

An arrow expression creates a callable value. Methods such as `map` supply checked
parameter context, so their callbacks can omit annotations. Without that context,
annotate the parameters; Tsrust does not silently introduce `any`.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Infer callback parameters from context

Array callbacks receive the method's checked element and index types. The result
is inferred from the arrow body when its checked returns establish a compatible
type.

```tsrust
function main() {
  const users = ["Alice", "Bob", "Charlie"];
  const upper = users.map(user => user.toUpperCase());
  console.log(upper.join("|")); // ALICE|BOB|CHARLIE
}
```

A declared callable contract also supplies parameter context. Inference preserves
the contract's argument access and result requirements.

## Choose an expression or block body

Parentheses are optional for one unannotated parameter. Use them for multiple
parameters or type annotations. An expression body returns its value automatically;
a block body uses explicit `return` statements.

```tsrust
function main() {
  const values = [1, 2];
  const expression = values.map(value => value + 1);
  const block = values.map((value) => {
    return value + 1;
  });
  const add = (value: int) => value + 1;
  console.log(expression.join(","), block.join(","), add(4)); // 2,3 2,3 5
}
```

A result annotation is optional when the checker can infer a consistent result.
A function that returns a value must return on every reaching path. Parenthesize
an object expression body to distinguish it from a statement block.

## Use a declared contract or return a record

```tsrust
type Row = {
  value: int;
};

function apply(action: (input: ReadOnly<string>) => string): string {
  return action("Pat");
}

function main() {
  console.log(apply(value => value.toUpperCase())); // PAT
  const make = (): Row => ({
    value: 7
  });
  console.log(make().value); // 7
}
```

## Keep ownership and call rules explicit

Arrow syntax preserves the method's parameter count, argument access and receiver
loans. This applies to mapping, filtering, predicates, iteration, sorting and seeded
reductions. Captured owners, scoped borrows and single-use invocation follow the
same contracts as explicitly annotated block arrows.

Defaults, rest parameters and destructuring use their normal checked rules. See
[optional and default parameters](/tsrust/language/optional-default-parameters/),
[rest parameters](/tsrust/language/rest-parameters/) and
[local callable ownership](/tsrust/language/callable-locals/).

Inferred parameter and result types feed editor hover, completion and signature
help. The formatter preserves source tokens and makes arrow layout consistent.
For the block-bodied `function` spelling and private recursive names, see
[function expressions](/tsrust/language/function-expressions/).
