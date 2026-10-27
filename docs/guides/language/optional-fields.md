# Optional record fields

`name?: Type` declares a record field that a record literal may leave out.

```tsrust
import { strictEqual } from "std:testing";

type Point = { x: int; y?: int; };
type User = { name: string; nickname?: string; home?: Point; };

function width(point: Point): int {
    return point.x + (point.y ?? 0);
}

function main() {
    const flat: Point = { x: 1 };
    strictEqual(flat.y === undefined, true);
    strictEqual(width(flat), 1);
    strictEqual(width({ x: 1, y: 2 }), 3);

    const user: User = { name: "Pat", home: { x: 3 } };
    strictEqual(user.nickname ?? user.name, "Pat");
    strictEqual(user.home?.x === 3, true);

    let moving: Point = { x: 0 };
    moving.y = 5;
    strictEqual(width(moving), 5);
    moving.y = undefined;
    strictEqual(width(moving), 0);
}
```

## Meaning

- The type of `name?: Type` is `Type | undefined`. Reading the field gives that union; narrow it, or use `??` or `?.`, before using the value.
- A literal may omit the field. The field then holds `undefined`. Writing `name: undefined` means the same.
- The field can be assigned later, also back to `undefined`.
- Required fields are unchanged: a literal without one is a `type` error.
- Optional fields work in generic record types, nested records, arrays of records, destructuring with defaults and object spread.

## A record keeps its shape

A record type has a fixed set of fields. An omitted optional field is still a field of the record:

```tsrust
type Point = { x: int; y?: int; };

function main() {
    const point: Point = { x: 1 };
    console.log(point, Object.keys(point).join(","), "y" in point);
    // prints { x: 1, y: undefined } x,y true
}
```

JavaScript leaves an omitted property out of the object, so it prints `{ x: 1 }`, lists only `x`, and `"y" in point` is `false`. Programs that test `point.y === undefined`, or use `??` and `?.`, behave the same in both.

An omitted field follows the written fields in print order.

## Difference from `name: Type | undefined`

A field declared as `name: Type | undefined` is required: every literal must write it, possibly as `undefined`. Only `name?: Type` may be omitted.
