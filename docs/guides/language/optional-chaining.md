# Optional chaining

`?.` evaluates its base once. A null or undefined base produces undefined and skips the remaining continuous chain, including computed indices and call arguments. A present base follows the ordinary checked operation. Zero, false and empty strings are present values. Fields use `base?.field`, indices use `base?.[index]`, and calls use `callee?.(arguments)`; optional method access and optional invocation can be combined.

Parentheses end a continuous chain. `value?.text.length` skips both accesses when value is absent; `(value?.text).length` accesses a potentially undefined result and needs a guard or fallback. A later nullable field requires another optional hop. The result adds undefined only when an absent branch is reachable and flattens existing nullable alternatives. Optional access does not establish a nonnullable fact for later statements.

Indices and arguments retain their ordinary strict types and ranges. Reached array accesses still check bounds; reached String operations keep their UTF-8 byte/scalar contracts and boundary errors. Optional access does not catch failures in a reached operation or evaluated argument. Native dense array families, canonical single-payload nullable types and callable contracts remain in effect. Optional chaining does not add coercion, arbitrary mixed-union operations, optional assignment targets or prototype lookup. Nullable callable storage and optional invocation are supported under their [callable ownership contracts](nullable-callables.md).

The editor uses compiler-checked whole-expression results and successful-hop payload facts, cached once per document revision. Incomplete named/direct-field `value?.` help uses a readonly check of the unchanged prefix; absent-only, unknown, conflicting specializations and rejected prefixes stay conservative. This optional help never authorizes ordinary `value.` access.

```tsrust
type OptionalRow={n:int;};
function main(){
  const absent:OptionalRow|null=null;const present:OptionalRow|null={n:7};console.log(String(absent?.n),present?.n??0);
}
```

Expected stdout: `undefined 7`.

```tsrust
function optionalIndex(trace:int[]):int{trace.push(1);return 0;}
function main(){
  const trace:int[]=[];const absent:int[]|undefined=undefined;const present:int[]|undefined=[5];console.log(String(absent?.[optionalIndex(trace)]),trace.length,present?.[optionalIndex(trace)]??0,trace.length);
}
```

Expected stdout: `undefined 0 5 1`.

```tsrust
function optionalCount(trace:int[]):int{trace.push(1);return 1;}
function main(){
  const trace:int[]=[];const absent:string|null=null;const present:string|null="é".repeat(1);console.log(String(absent?.repeat(optionalCount(trace))),trace.length,String(present?.repeat(optionalCount(trace))),trace.length);
}
```

Expected stdout: `undefined 0 é 1`.

```tsrust
type OptionalTextRow={text:string;};
function main(){
  const absent:OptionalTextRow|null=null;const present:OptionalTextRow|null={text:"é"};console.log(String(absent?.text.length),present?.text.length??0);
}
```

Expected stdout: `undefined 2`.

```tsrust

function main(){
  const values:string[]|null=["é".repeat(1)];const selected=values?.[0];const independent=clone(selected);console.log(String(selected),String(independent),values?.length??0);
}
```

Expected stdout: `é é 1`.

```tsrust
type OptionalHolder={text:string|null;};
function main(){
  const holder:OptionalHolder|undefined={text:null};const selected=holder?.text;console.log(String(clone(selected)??"empty"),Boolean(selected));
}
```

Expected stdout: `empty false`.

Copy selections are snapshots. Existing owned fields or indexed values can produce scoped readonly views: reading the source remains allowed, while conflicting mutation, moves, escaping returns and captures follow its checked loans. An explicit canonical nullable annotation can preserve the same scoped readonly view; it does not create an owner. Such views cannot escape through a return or Consume argument. Use `clone` for an independent owner. Fresh receivers may transfer selected values, and methods creating new storage return owned results; no implicit owner duplication is added.

Borrowed optional string fields can select a string literal fallback without cloning. See [borrowed optional fallbacks](../../../website/src/content/docs/language/borrowed-optional-fallback.md) for the current source-loan and independent-owner rules.
