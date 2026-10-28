---
title: Declare nested functions
description: Use block-scoped declarations, forward calls and checked return inference.
---

Declare a named function inside another function or a braced block to keep a
helper near the code that uses it. The declaration is available throughout its
block, including before its source position.

After [native setup](/tsrust/tools/rust-setup/), save each complete example separately
as `main.tsr` and run `tsrust run main.tsr`. Comments show expected output.

## Call a helper before its declaration

```tsrust
function main() {
  console.log(double(3)); // 6

  function double(value: int): int {
    return value + value;
  }
}
```

Parameters use the same annotations as top-level functions, including
[optional and default parameters](/tsrust/language/optional-default-parameters/)
and [rest parameters](/tsrust/language/rest-parameters/). A function declaration
needs a braced body. To declare one inside an `if` or loop, put it inside that
statement's braced block.

## Keep names inside their block

A nested block can declare the same name independently. Leaving the block restores
the outer binding; its inner declaration is unavailable outside that block.

```tsrust
function main() {
  function value(): int {
    return 10;
  }

  {
    console.log(value()); // 20
    function value(): int {
      return 20;
    }
  }

  console.log(value()); // 10
}
```

Two declarations cannot share a name in the same block. A function-body declaration
also cannot redeclare a parameter of that function. An arrow parameter or a binding
in a nested block can shadow an enclosing name.

A nested function named `main` is an ordinary local helper. Only the file's entry
function runs implicitly. Use a [named function expression](/tsrust/language/function-expressions/#give-an-expression-a-private-recursive-name)
when you need a callable value with a private recursive name instead.

## Infer a result or annotate recursion

A nonrecursive helper can omit its result annotation when checked return expressions
establish the result. This includes results derived from parameters, captures and
arrays.

Value-returning paths must agree on a type, and every reaching path must return a
value. A function with no value returns has result `void`; the entry function
`main` must have result `void`.

```tsrust
function main() {
  function double(value: int) {
    return value + value;
  }

  function factorial(value: int): int {
    if (value <= 1) {
      return 1;
    }

    return value * factorial(value - 1);
  }

  console.log(double(3), factorial(5)); // 6 120
}
```

Nested declarations support self recursion and mutual recursion. An explicit result
annotation makes a recursive contract clear. Without one, inference needs a return
value whose type is known independently of the recursive call, such as:

- A literal base case, such as `return 1`.
- One known branch of a conditional expression.
- An arithmetic operand with a known type.

Write the result type when recursion supplies the only evidence or the independent
evidence is available only through a local variable.

## Find and use helpers in the editor

Completion offers hoisted declarations throughout their block. Hover and signature
help show checked source signatures. Definitions and references follow the selected
binding when a name is shadowed, and the document outline nests local functions
under their enclosing function.

A nested function may also capture enclosing variables or be used as a callable
value. Those uses follow the same ownership and capture checks as
[arrow callbacks](/tsrust/language/arrow-functions/) and
[local callable contracts](/tsrust/language/callable-locals/).
