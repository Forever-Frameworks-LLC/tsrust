---
title: Borrow optional text safely
description: Select fallback text without copying, or clone it when independent ownership is needed.
---

Use `?.` to read a field only when its record is present, then `??` to supply text
when the result is `null` or `undefined`. An empty string is present and stays empty.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`.

## Select a literal fallback

```tsrust
type Row = {
  label: string;
};

function main() {
  const rows: (Row | null)[] = [{
    label: "first"
  }, null, {
    label: ""
  }];
  for (const item of rows) {
    const label = item?.label ?? "none";
    console.log(label.toUpperCase());
  }
}
```

This prints `FIRST`, then `NONE`, then an empty line. The same selection works
outside a loop with `rows[0]?.label ?? "none"`.

When present, the field is borrowed. The literal fallback supplies static text;
neither selection clones an owner. Reading the selected text leaves the source
and its strings unchanged.

## Keep the source alive and unchanged

The selected `const` is a readonly borrow. It cannot transfer ownership or mutate
the selected string. The source cannot be changed or transferred until the borrowed
binding leaves scope, even when the fallback was selected.

Use an inner block when the source must change afterwards. This example finishes
without output:

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

  rows.push({
    label: "next"
  });
  strictEqual(rows.length, 2);
}
```

Changing `rows` inside that block conflicts with its active loan. Passing `label`
to a function that consumes a string is also rejected. Read it, pass it to a
`ReadOnly<string>` parameter, or clone it.

## Bind a computed fallback first

A fallback from a stable named string is also borrowed. A freshly allocated owner
cannot be selected directly into this view: `item?.label ?? "new".repeat(2)` is
rejected with `borrowed-view`.

Bind computed text before borrowing it. This example prints `newnew`:

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
```

This computes `fallback` before the selection, regardless of whether it is needed.
Both possible sources remain subject to their checked borrowing rules.

## Keep an independent string

Use explicit `clone()` to keep text while changing its source. This example
finishes without output:

```tsrust
import { strictEqual } from "std:testing";

type Row = {
  label: string;
};

function main() {
  const rows: (Row | null)[] = [{
    label: "first"
  }];
  const label = clone(rows[0]?.label ?? "none");
  rows.push(null);
  strictEqual(label, "first");
  strictEqual(rows.length, 2);
}
```

The new string has independent ownership. Neither `?.` nor `??` introduces an
implicit clone.

## Check array bounds and record types

Optional member access requires one proven present record type. Narrow a union of
unrelated record shapes before selecting its fields.

In `rows[0]?.label`, indexing happens first. An empty array still fails its bounds
check; `?.` checks the selected element's null/undefined state, not whether index
zero exists.

## Related guides

- [Use optional record fields](/tsrust/language/optional-fields/)
- [Pattern defaults and rest](/tsrust/language/pattern-defaults-rest/)
