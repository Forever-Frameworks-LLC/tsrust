# Global number names

Six number helpers can be written without the `Number.` prefix. Each one is the same function or value as its `Number` member, so the two spellings can be mixed.

| Global name | Same as | Type |
| --- | --- | --- |
| `parseInt(text, radix?)` | `Number.parseInt` | `number` |
| `parseFloat(text)` | `Number.parseFloat` | `number` |
| `isNaN(value)` | `Number.isNaN` | `boolean` |
| `isFinite(value)` | `Number.isFinite` | `boolean` |
| `NaN` | `Number.NaN` | `number` |
| `Infinity` | `Number.POSITIVE_INFINITY` | `number` |

```tsrust
import { strictEqual } from "std:testing";

function smallest(values: number[]): number {
    let lowest = Infinity;
    for (const value of values) {
        if (value < lowest) {
            lowest = value;
        }
    }
    return lowest;
}

function main() {
    const readings = ["4.5", "n/a", "2.5"].map((text) => parseFloat(text));
    const usable = readings.filter((value) => !isNaN(value));
    strictEqual(usable.length, 2);
    strictEqual(smallest(usable), 2.5);
    strictEqual(isFinite(smallest([])), false);
    strictEqual(parseInt("ff", 16), 255.0);
    strictEqual(isNaN(parseInt("none")), true);
    strictEqual(-Infinity < 0.0, true);
}
```

## Rules

- `parseInt` and `parseFloat` return a `number`, not an `int`, because the result can be `NaN`. Assigning the result to an `int` is a `type` error.
- `isNaN` and `isFinite` accept a number only. A string or boolean argument is a `type` error; nothing is converted first. Convert explicitly with `Number(value)` when that is what you mean.
- `NaN` is not equal to anything, itself included. Test with `isNaN(value)`, never with `value === NaN`.
- `typeof NaN` and `typeof Infinity` are `"number"`.
- `NaN` and `Infinity` cannot be assigned to.
- A declaration with one of these names takes precedence in its scope: a function named `parseInt`, or a variable or parameter named `NaN`, is yours.
- The four functions must be called. Passing one as a value, as in `texts.map(parseInt)`, is `unsupported`; write `texts.map((text) => parseInt(text))`.

The editor shows the same documentation, parameter help and completion for the global names as for the `Number` members.
