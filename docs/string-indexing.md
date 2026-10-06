# String indexing

`text[position]` is the character at a position of a string, as a new one-character string.

```tsrust
import { strictEqual } from "std:testing";

function initials(words: string[]): string {
    let result = "";
    for (const word of words) {
        result += word[0];
    }
    return result;
}

function main() {
    const text = "tsrust";
    strictEqual(text[0], "t");
    strictEqual(text[text.length - 1], "t");
    strictEqual(initials(["type", "safe", "rust"]), "tsr");
    const maybe: string | undefined = "ok";
    if (maybe !== undefined) {
        strictEqual(maybe[1], "k");
    }
}
```

## Positions

- The position is an `int`. Position `0` is the first character.
- Positions count whole Unicode characters, the same unit that `charAt`, `at`, `codePointAt` and `for...of` use. In `"a€😀b"`, position `2` is `"😀"`.
- `length` counts UTF-8 bytes. For text outside ASCII, `text[text.length - 1]` is not the last character; use `text.at(-1)`.

## Out of range

A negative position, or a position at or after the end, is a range error, like an array index out of bounds. JavaScript gives `undefined` there; Tsrust keeps the result type `string` and reports the mistake.

```text
const text = "abc";
try {
    console.log(text[3]);
} catch (error) {
    console.log("out of range");   // runs
}
```
 Inside `try` the error can be caught. Outside any `try` the program stops and reports `string index out of range` with the source position.

Other ways to read a character differ only in the missing case: `charAt(position)` gives `""`, and `at(position)` gives `undefined` and accepts negative positions from the end.

## Rules

- The result is a `string`. It is a new value; the text is only read and stays usable.
- A string cannot be changed through a position. `text[0] = "x"`, `text[0] += "x"` and `text[0]++` are `unsupported` errors. Build a new string instead.
- A `string | undefined` value must be narrowed before it can be indexed. The optional form `text?.[position]` is `unsupported` for now; use `text?.at(position)`.
- A position that is not an `int`, such as `text["0"]` or `text[1.5]`, is a `type` error.
