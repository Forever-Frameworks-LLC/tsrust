# Schema result API alignment

Status: owner-requested follow-up, 2026-10-14; implementation pending. The owner
wants schema parsing as close to Zod as practical and requests removing the
unexplained `{ payload: ... }` surface or providing an explicit success outcome.
This direction supersedes retaining the generic Result-shaped schema surface as
an unquestioned settled decision. It does not change general Result semantics.

## Observable target

Use Zod-shaped schema results:

```ts
{ success: true, data: Output }
{ success: false, error: ValidationError }
```

`success` is validation outcome metadata. It is not a compiler diagnostic.
The result must discriminate these alternatives so checking `result.success`
narrows access to `result.data` or `result.error`, with precise inferred types
and complete editor help. Do not expose an undifferentiated payload that could
mean either valid data or a validation error. A boolean added only to console
output would not implement this API.

`schema.parse(input)` returns Output directly and throws ValidationError on
validation failure. Current lowering in `analysis/schema/mod.rs::schema_parse_call`
already unwraps its internal Result through `@schema:parseResult`; existing
`tests/schema_parse.rs` covers direct scalar/object output and throwing behavior.
Preserve this behavior and verify no implementation wrapper leaks in output,
hover or inferred types. An output schema that itself declares a field named
`payload` must retain that user field; never flatten arbitrary user records.

Reference: [Zod basic usage](https://zod.dev/basics), reviewed 2026-10-14. The
reference describes direct parse output and a success-discriminated safeParse
result with data/error branches. This is targeted API alignment, not a claim of
complete Zod compatibility or permission to import every Zod feature.

## Preserve native contracts

Keep existing uniquely owned validation, borrowed reusable schemas, precise
inferred output, redacted typed issues/paths, truncation information and callback
exceptions. Do not adopt Zod's deep-clone behavior implicitly: Tsrust input remains
consumed on success and failure, with explicit clone for independent ownership.
Outcome construction must not duplicate input, validated output or error storage.

General `Result<T,E>`, Ok/Err matching and their representation remain unchanged.
Schema-facing results may use a dedicated representation or a checked lowering,
but the public fields, narrowing and native execution must agree. Existing schema
examples/tests that match Ok/Err must migrate deliberately; leaving them broken
or silently redefining all Result values is not acceptance.

Audit `safeParseJson` alongside safeParse for consistent outcome access. Preserve
its distinction between JSON syntax and validation errors, including error kind
and paths. Do not accidentally widen this request to unrelated JSON APIs.

## Implementation and acceptance section

Implement when the connected checker/type/emitter changes can land coherently.
Keep the nesting priority intact, then complete this schema API section before
collecting new schema optimization measurements and before S12 feature work.
Public-result changes alter consumers and benchmark observations; original S11
samples remain immutable and must not be relabeled as the new API's measurements.

Cover primitive, object, array, optional/nullable, union, recursive, transformed
and JSON validation outcomes. Verify success/failure narrowing, direct parse
returns, throwing behavior, callback exceptions, moved-input diagnostics,
allocation/disposal, evaluation exactly once, and a user-owned payload field.
Include console output, project imports and native execution. A success result
must not accidentally require an error value; a failure must not fabricate data.

Complete compiler signatures/types, diagnostics, all schema catalog entries,
completion/hover/parameter help, executable examples, documentation and matched
TypeScript/native benchmark support as one section. Run editor typing, agreement
and consistency guards. Migrate source fixtures and callers from the old schema
Result surface, preserving general Result regression coverage. Preserve original
failures and validate matched independent oracles before timing. Format the whole
project, checkpoint, import and verify evidence in SQLite, and refresh actual CLI
and editor consumers after acceptance. Until then, describe this as pending.
