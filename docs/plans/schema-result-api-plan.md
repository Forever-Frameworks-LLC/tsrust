# Schema result API alignment

Status: owner-requested follow-up, 2026-10-14. Compiler, native execution, catalog
text and source fixtures implemented on 2026-10-14; editor inference, workload
fingerprints and matched measurements pending (see Progress). The owner
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

## Progress (2026-10-14)

Implemented:

- `safeParse` and `safeParseJson` lower through the source adapter
  `@schema:safeParseOutcome` (`stdlib/schema/compiler/builders.rs`), which moves
  the internal Result into `SafeParseSuccess<T> = { success: true; data: T }` or
  `SafeParseError<E> = { success: false; error: E }`. Nothing is copied. `parse`
  keeps checking the raw internal Result (`raw_schema_results`) and still throws.
  `SafeParseSuccess` and `SafeParseError` are reserved names when safeParse is used.
- Truthiness narrowing on boolean literal discriminants: `if (r.success)` and
  `if (!r.success)` narrow like `=== true` / `=== false`
  (`analysis/mod.rs::boolean_discriminant`).
- Consuming an owned field of a narrowed local alternative (`const data = r.data`)
  moves the whole local, as destructuring does; later use is rejected.
- Generic applications render checked argument names in diagnostics
  (`SafeParseError<ValidationError>`, `Box<int>[]`).
- Compiler tests, runtime workloads and catalog examples migrated from
  `match(...){Ok/Err}` to success narrowing. Affected compiler suites pass
  (schema*, native_declarations, module_state, json_typed).
- `console.log(result)` prints `{ success: true, data: ... }`.

Pending:

- Editor inference. `crates/tsrust-lsp/src/intelligence.rs::primitive_method_type`
  still spells safeParse results as `Result<...>`. The editor must know the two
  outcome records, offer `success` before narrowing and `data`/`error` after
  `if (r.success)`/`if (!r.success)`, and hover them. Failing editor tests at
  this checkpoint: `collection_schema_types_and_json_completion_follow_input_contracts`, `completion_answers_at_typing_speed`, `completion_is_not_empty_where_hover_reports_a_type`, `documented_builtin_examples_match_native_output`, `every_catalog_entry_has_a_current_compiler_witness`, `imported_result_matches_keep_nominal_payload_help_and_pattern_navigation`, `jsonl_editor_contract_exposes_ownership_limits_and_error_locations`, `recursive_checked_outputs_keep_nested_members_and_bounded_hover`, `recursive_named_and_array_outputs_keep_member_completion`, `recursive_union_root_keeps_common_fields_and_finite_hover`, `schema_object_constructor_exposes_inferred_fields`, `schema_object_retained_output_exposes_typed_value_and_map`, `schema_pipeline_default_help_uses_raw_input_and_presence_keeps_output`, `schema_pipeline_help_keeps_raw_input_distinct_from_transformed_output`, `schema_safe_parse_help_explains_consumption_and_result`, `schema_validation_error_alias_tracks_scalar_and_pipeline_results`, `typing_a_project_document_produces_no_misleading_answer`.
- Runtime workload fingerprints in `benchmarks/runtime/manifest.json`: the
  migrated `native_schema_*.tsr` sources changed tokens intentionally, so
  `tools/format-project.mjs` refuses the automatic refresh. Validate the matched
  workloads (`node benchmarks/runtime/run.mjs --execution-only --engines=native,node-tsc --only=...`),
  then update their source hashes as part of this section's acceptance. Original
  S11 samples remain the old API's measurements.
- Documentation pages outside the catalog that show `match` on schema results.
