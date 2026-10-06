# String iteration

`for (const character of text)` visits Unicode characters in source order. The
iterable must be a string, and the loop variable has type `string`. The iterable
expression runs once. Empty strings run no body. Surrogate-pair characters such
as emoji occupy one iteration; combining marks have their own iterations.

```tsrust
let text = "";
for (const character of "Aé🙂") {
    text = text + character + "|";
}
console.log(text); // A|é|🙂|
```

Iteration uses UTF-8 Unicode scalar values. It agrees with JavaScript string
iteration for supported well-formed text. Isolated UTF-16 surrogate strings are
not representable in native strings. A character's `length` still counts UTF-8
bytes: `"é".length` is 2 and `"🙂".length` is 4. Iteration does not count bytes or
grapheme clusters.

The source is borrowed until the loop exits; mutation, replacement and transfer
of a named source inside the body are rejected. Character bindings are readonly
borrows, including a binding written with `let`. Use `clone(character)` to keep an
independent owned string. Readonly string parameters and string operations can
read the character directly. Normal break, continue, return, throw and finally
rules apply.

The current native string ABI constructs one owning character string per
iteration, then lends it to the loop body. It does not clone the source string or
build an array of characters. Character lifetimes end after each iteration.
Live iterator objects and a general iterator protocol remain separate features.
