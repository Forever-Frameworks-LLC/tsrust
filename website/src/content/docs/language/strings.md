---
title: Work with strings
description: Understand UTF-8 byte offsets, character access, iteration and concatenation.
---

Strings contain valid UTF-8. Some operations use **byte offsets** and others use
**Unicode scalar positions**. Neither unit counts visible grapheme clusters: a
letter followed by a combining accent can contain two scalars.

## Choose the right position unit

| Operation | Unit |
| --- | --- |
| `length` | UTF-8 bytes |
| `slice`, `substring`, `substr` | Byte offsets and lengths |
| Search positions and returned indexes | Byte offsets |
| `[index]`, `charAt`, `at`, `codePointAt` | Unicode scalar positions |
| `for...of` | One Unicode scalar per iteration |

Byte ranges must respect UTF-8 boundaries for nonempty selections. For example,
`"😀".slice(0, 4)` returns the emoji, while `slice(0, 1)` fails because it cuts
through its encoding. Do not use `length - 1` to find the final character of
non-ASCII text; use `at(-1)`.

Save each complete example below as `main.tsr` and run it with
`tsrust run main.tsr` after [preparing native tools](../../tools/rust-setup/).

```tsrust
function main() {
  const text = "a€😀b";
  console.log(text.length); // 9 bytes
  console.log(text[2]); // 😀
  console.log(text.at(-1) ?? "missing"); // b
  console.log(text.slice(1, 4)); // €
}
```

## Read a character

An index must be an `int`, starting at zero. `text[index]` returns a new `string`
and leaves the source usable. A negative or missing position raises a catchable
range error. Outside a `try`, that error stops the program and reports the source
position. This differs from JavaScript's missing-index `undefined` result.

```tsrust
function main() {
  const text = "abc";
  try {
    console.log(text[3]);
  } catch (error) {
    console.log("out of range");
  }
}
```

This prints `out of range`. Choose a different access method when absence is expected:

- `charAt(position)` returns `""` for a negative or missing position.
- `at(position)` returns `undefined` when missing and accepts negative positions
  counting from the end.

A string cannot be changed through an index. Assignments such as `text[0] = "x"`
and compound index writes are rejected. Build a new string instead.

### Optional strings

Narrow a `string | undefined` value before indexing it, or use `text?.[position]`.
Optional access returns `undefined` for an absent receiver; it still checks the
index when the receiver is present.

```tsrust
function main() {
  const present: string | undefined = "ok";
  const absent: string | undefined = undefined;
  console.log(present?.[1] ?? "missing"); // k
  console.log(absent?.[0] ?? "missing"); // missing
}
```

## Iterate over characters

The iterable expression runs once. Empty strings run no body. Emoji such as `🙂`
occupy one iteration; combining marks have their own iterations.

```tsrust
function main() {
  let output = "";
  for (const character of "Aé🙂") {
    output += character + "|";
  }

  console.log(output); // A|é|🙂|
}
```

The source is borrowed until the loop exits. Mutating, replacing or transferring a
named source inside the body is rejected. Character bindings are readonly borrows,
even when written with `let`. Use `clone(character)` to retain an independent owned
string. Each iteration constructs its character string; it does not clone the
whole source or create an array of all characters.

## Combine strings and scalar values

When either operand of `+` is a string, the other may be a string, `int`, `number`,
`boolean`, `null`, `undefined`, or a union containing only these types. Scalars use
`String(value)` conversion. Operands evaluate once, left to right; the result is
an independent string.

```tsrust
function main() {
  console.log(1 + 2 + "x"); // 3x
  console.log("x" + 1 + 2); // x12
  let message = "ready:";
  message += true;
  console.log(message); // ready:true
}
```

Addition is left associative. Parentheses can request numeric addition first.
A string result annotation alone does not turn numeric addition into concatenation.
Integers retain their signed 64-bit precision during conversion.

String `+=` also accepts scalar conversion for mutable string fields and array
positions. It is a standalone statement. The old target stays protected while
the right operand runs, and that operand runs once. Sources are borrowed rather
than transferred. Arrays, records and functions do not concatenate implicitly;
use an explicitly supported conversion when appropriate. String subtraction,
coercive equality and other numeric operations on strings remain rejected.

## Compare text and handle Unicode

Equality compares exact content without normalization. Ordering compares UTF-8
text lexically and can differ from JavaScript's UTF-16 ordering. Matching string
operands evaluate once, left to right, without transferring them.

Lone surrogate escapes are rejected; valid escaped surrogate pairs decode to one
scalar. Native strings cannot represent isolated UTF-16 surrogates. Trimming and
case conversion follow native Unicode rules: for example, trimming removes U+0085
but preserves U+FEFF. Case conversion can change byte length.

For each method's argument defaults, bounds and errors, use
[compiler-backed API help](../../library/overview/#read-the-exact-contract).
