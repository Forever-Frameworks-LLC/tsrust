# Enumerations

An enum names a closed set of copyable values. Each declaration creates a distinct
type. Members of another enum and raw numbers or strings cannot be assigned to it.

```tsrust
enum Phase { Ready, Busy = 4, Done }
enum Label { Short = "s", Long = "long" }

function main() {
    const phase: Phase = Phase.Done;
    console.log(Number(phase)); // 5
    console.log(String(Label.Long)); // long
}
```

Numeric members start at zero unless initialized explicitly. Each omitted value
is the previous numeric value plus one. String members require explicit values.
A declaration cannot mix numeric and string members. Initializers are constant
expressions and may refer to earlier members. Arithmetic and bitwise expressions
are allowed for numeric values; bitwise constants follow 32-bit JavaScript rules.

Members are readonly. Enum values can be passed, returned, stored in record fields,
and used in arrays. Equality compares their underlying values, so members with
duplicate values compare equal. Different enum types remain incompatible even
when their underlying values match. `Number(value)` and `String(value)` provide
explicit primitive conversion. `typeof` reports the underlying value category.

Access copies an enum value without an allocation. This does not relax ownership
rules for surrounding records or arrays.

The declaration is a static namespace rather than a mutable runtime object.
Reverse numeric lookup, dynamic member names, heterogeneous declarations,
runtime initializers, and `const enum` are not supported.
