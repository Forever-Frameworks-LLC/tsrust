# Borrowed optional values with a string fallback

An optional string field can be read from a borrowed record or array element and selected with a string literal fallback:

```ts
type Row = { label: string; };
function main() {
    const rows: (Row | null)[] = [{ label: "first" }, null, { label: "" }];
    for (const item of rows) {
        const label = item?.label ?? "none";
        console.log(label.toUpperCase()); // FIRST, NONE, then an empty line
    }
}
```

The field is borrowed when present. The fallback runs only for null or undefined; an empty string stays empty. A string literal supplies static text and needs no cloned owner. The source and its owned strings stay unchanged. The same selection works with `rows[0]?.label ?? "none"`.

The selected const is readonly. It cannot transfer ownership or mutate the selected string. Source loans follow the existing lexical rules: the source cannot be changed or transferred until the borrowed const leaves scope. A fallback from a stable named string is also borrowed. Selecting a freshly allocated owner into this view, such as `item?.label ?? "new".repeat(2)`, is rejected; bind that owner separately before borrowing it.

Use explicit `clone()` to retain an independent owner:

```ts
const label = clone(rows[0]?.label ?? "none");
rows.push(null); // allowed: label has independent ownership
```

No implicit clone is performed. Optional member access still requires one proven present record type; a union of unrelated record shapes must be narrowed first. Ordinary indexing continues to reject an out-of-bounds array position at runtime. Optional access checks the selected element's null/undefined state; it does not change array bounds behavior.
