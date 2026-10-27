# Nullish assignment

`target ??= fallback` selects its destination once and reads the old value once. It evaluates and commits the fallback only when the old value is null or undefined. Zero, false and the empty string keep their existing value. The expression returns the selected value, and the destination retains that value.

Targets must satisfy the existing mutable-place and loan rules. Arguments, selection and indexed bounds precede the fallback; an indexed destination is reacquired after fallback effects. A failed evaluation does not commit the outer assignment, while completed fallback effects remain. Both branches must fit the destination’s declared native type: even a statically nonnullable String target rejects a numeric fallback. This differs from a statically selected ordinary `??` expression, whose unreachable fallback may have another type.

A stable nullable target is absent inside its fallback. Its successful assignment result and subsequent stable target view remove only the absent alternatives that the fallback cannot return. A String fallback yields String; a nullable String fallback may keep null. Conflicting writes and effects invalidate these facts. Dynamic selection or a repeated call does not become a stable flow identity.

The editor uses the compiler’s checked expression and projection facts for inferred result bindings, grouped receivers, method signatures and payload fields. The existing nullable domain and strict int/Number distinction apply. This operator does not implement optional chaining, arbitrary payload unions, nullable array elements or coercive assignment.

```tsrust
function main(){let count:int|null=0;let flag:boolean|undefined=false;let text:string|null="";const kept=(text??="fallback");console.log(count??=9,flag??=true,kept.length);}
```

Prints `0 false 0`.

```tsrust
function makeText(value:null):string{return "é".repeat(1);}function main(){let value:string|null=null;const selected=(value??=makeText(value));const independent=clone(selected);console.log(selected,independent,value.length);}
```

Prints `é é 2`.

```tsrust
function main(){let values:int[]|undefined=undefined;const selected=(values??=[1,2]);console.log(selected.join("|"),values.length);}
```

Prints `1|2 2`.

```tsrust
type Row={n:int;};function main(){let value:Row|null=null;const selected=(value??={n:7});console.log(selected.n,value.n);}
```

Prints `7 7`.

```tsrust
function main(){let value:int|null|undefined=undefined;const selected=(value??=null);value=4;console.log(String(selected),String(value));}
```

Prints `null 4`.

```tsrust
type Holder={text:string|null;};function makeFieldText(value:null):string{return "filled".repeat(1);}function main(){const holder:Holder={text:null};const selected=(holder.text??=makeFieldText(holder.text));console.log(selected,holder.text.length);}
```

Prints `filled 6`.

Copy results are snapshots. An owned result is a scoped readonly view of the value retained in its destination. Inferred owned-result bindings are readonly even when declared with `let`; their destination loan lasts until the binding’s enclosing scope ends, rather than ending at its last use. Reads and readonly arguments are allowed, but conflicting mutation or movement is forbidden until that scope ends. An unbound readonly result or `clone(target ??= fallback)` borrows only for its consumer expression. A view cannot escape through ownership transfer or an escaping capture. Explicit `clone(result)` creates an independent owner; removed or replaced owners follow the existing disposal rules.

A conditional initializer may select an assignment view or a fresh owned value. The fresh owner remains in the binding’s enclosing scope, and the result remains readonly. Copy field and index projections are snapshots.

Copy assignment chains can use `a ??= b ??= 7` directly. Owned assignment chains need an explicit clone of the inner readonly result, such as `a ??= clone(b ??= "x".repeat(1))`, so both destinations hold independent owners. Chaining does not silently duplicate an owner.
