# Comma expressions in receivers and guards

A parenthesized comma expression evaluates its operands from left to right and
selects the final operand. A selected named array supports the same method access
as the original name. Prefix effects execute once before the method arguments.

```tsrust
const values = [1, 2];
(0, values).push(3);
console.log((0, values).join(",")); // 1,2,3
```

The selected array keeps its original ownership and loans. Selecting a borrowed
array does not permit mutation, and selecting an indexed array keeps its containing
source loaned while the method needs it. Selection does not clone an array.

Strict equality recognizes a null or undefined sentinel selected by a comma
expression. It also recognizes a selected `typeof` expression and a selected
literal type label. These conditions can narrow a union:

```tsrust
function length(value: int | string): int {
    if ((0, typeof value) === (0, "string")) {
        return value.length;
    }
    return value;
}
```

A comparison observes its left value before evaluating its right value. If the
right operand changes the source of the observed left value, the comparison does
not narrow that source. A comparison of a copied integer with null can therefore
be true while its original record field has already become null. Use a new check
of the current field before accessing its payload. Readonly calls preserve the
proof; a write on either reaching branch invalidates it. Equality continues to use
`===` and `!==`; comma expressions do not introduce coercive equality.
