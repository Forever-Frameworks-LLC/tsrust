# Nested named functions

A named function may be declared inside a function or an explicit block. Its name
is available throughout that block, including before the declaration. Nested
blocks may declare the same name without changing the enclosing declaration.
A declaration does not leak outside its block. A function named main inside another
function is an ordinary local function; only the file entry function runs implicitly.

```tsr
function main() {
    console.log(double(3)); // 6

    function double(value: int): int {
        return value + value;
    }
}
```

Capture-free declarations support self recursion, mutual recursion and use as
callable values. Parameter and result annotations use ordinary declaration syntax.
Basic generic, default and rest signatures share the top-level implementation.
Write the result annotation when returning a value: an omitted named-declaration
result currently means void. Nonrecursive result inference remains planned.
The native fixtures in tests/fixtures/nested-function-declarations verify forward
calls, factorial, mutually recursive even/odd functions, block shadowing, Array.map
callbacks and generic/default/rest signatures with assertions.

A declaration conflicts with another declaration of the same name in its block.
A function-body declaration also cannot replace a parameter of that function.
A closure parameter or a declaration in a nested block may shadow the name.
Function declarations require a braced body and cannot be a lone unbraced if/loop
body. Named function expressions are still a separate unsupported syntax feature.

Capturing an enclosing variable is not implemented yet for named declarations.
The diagnostic identifies the capture use and its declaration. The planned rules
borrow local captures and transfer supported captures when an owned callable escapes;
there is no implicit clone or shared capture cell. Use existing arrow or anonymous
function expressions for currently supported capture behavior while this foundation
is completed.

Completion includes hoisted declarations throughout their block. Hover and signature
help show source names and checked types, including imported nominal results.
Definitions and references respect shadowing; the document outline nests local
functions under their enclosing function. A later local binding masks an outer name
without being offered before its declaration. Formatting retains block structure and
blank lines. The capture-free retained-result benchmark is verified against an
independent oracle for native and Node execution; it has not been measured.

Gate 170 accepts this lexical foundation. The capture, escape and comprehensive
signature stages in nested-function-declarations-plan.md remain open.
