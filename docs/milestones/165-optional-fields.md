# Gate 165: optional record fields

`name?: Type` declares a record field that a record literal may omit. Its type is
`Type | undefined`. An omitted field holds `undefined`, the same as writing
`name: undefined`, and can be assigned later. A field declared as
`name: Type | undefined` stays required. See docs/optional-fields.md.

A record keeps its fixed shape: an omitted optional field is still a key of the
record, so printing shows `name: undefined`, `Object.keys` lists it and `"name" in
record` is true. JavaScript leaves the property out. This follows the existing record
model, in which every field of a record type is always present. Omitted fields follow
the written fields in print order.

## Implementation

- Parser: the record declaration accepts `?` after a field name and wraps the written
  type with `undefined` (`Parser::optional_field_type`). `RecordField::optional`
  records the declaration form.
- Analysis: `Analyzer::fill_omitted_optional_fields` appends an `undefined` value for
  each omitted optional field before a record literal is checked. Everything else
  uses the existing union field machinery: reads, narrowing, `??`, `?.`, assignment,
  destructuring defaults, spread, generic records and nested records.
- Editor: the record field scan accepts `name?: Type`, shows `name?: Type | undefined`
  and offers the field in completion; definition goes to the declaration.
- Formatting keeps `y?: int;`.

## Focused evidence

- Red: target/editor/optional-fields-red.log (three failing compiler checks) and
  target/editor/optional-fields-editor-red.log.
- Compiler: `optional_fields` 3 checks: a std:testing fixture (omission, explicit
  `undefined`, nested and generic records, arrays of records, later assignment,
  destructuring defaults, spread, optional chaining), the fixed-shape output witness
  and seven static rejections. Affected batch: 25 suites, 306 checks, 0 failures,
  including all depth canaries, record, destructuring, spread, key and project suites
  (target/editor/optional-fields-final-compiler.log).
- The fixture and the first guide example pass the pinned Node reference. Guide
  statements, including the fixed-shape output and the required
  `name: Type | undefined` form, are checked in
  target/editor/optional-fields-guide-verification.log.
- Editor: all suites pass, 263 checks (target/editor/optional-fields-final-editor.log),
  including `optional_fields_have_types_members_and_completion`.
- Harness 392/392 on the refreshed consumers (target/editor/optional-fields-harness.log).
- Five matched native/node-tsc cases pass untimed; the record
  runtime/results/2026-10-06T14-50-18-879Z-verify.json is imported into SQLite and its
  loose copy removed: optional_fields, string_indexing, type_aliases,
  exception_completion, comma_contexts.
- Benchmark support: workload pair, independent arithmetic oracle, source contract
  with a TypeScript syntax-tree check of the optional property, registration test,
  manifest row and oracle/run/profile/report hooks.
- Five consumers rebuilt and replaced by atomic rename; hashes in
  target/editor/optional-fields-consumers.log.

No full regression or timing was performed. The workload is unmeasured.
