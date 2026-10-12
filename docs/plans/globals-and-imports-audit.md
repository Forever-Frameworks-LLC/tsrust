# Globals and imports audit coverage

Status: the public API placement audit is accepted. The original 399 identities
and five JSON namespace additions each have a reviewed disposition (404 total).
See the [original closure review](globals-and-imports-audit-closure.md) and
[JSON namespace addendum](globals-and-imports-json-namespace-audit.md).
The JSON implementation gate remains in progress; naming acceptance does not
replace compiler/editor, benchmark-support or consumer acceptance.

## Reproducible inventory

Run `cargo run --offline --locked -p tsrust-compiler --example public_api_inventory`
with the documented shared Cargo target and `CARGO_INCREMENTAL=0`. The inventory
joins editor catalog entries to the **compiled** native binding registry, native
type declarations and bundled testing exports. Compiled metadata resolves Rust
struct-update defaults, parameter ownership, alternative argument types, feature
selection and return-borrow contracts without guessing from source snippets.

The initial inventory has 393 distinct group/name rows, including 56 native
bindings, 18 native types and 18 bundled exports. These counts overlap: they are
not counts of independent operations. Every imported catalog entry has a native
binding, native declaration or bundled source declaration. Thirteen private
runtime helpers and the two exported expectation types lack a same-group catalog
entry; expectation members have their own catalog groups. These findings need
explicit review, not automatic classification as missing functionality.

Original inventory and build output are stored in SQLite as
`tsrust-api-audit-inventory.json` and `tsrust-api-audit-build.log`. Every generated
row remains `pending`; presence in this inventory is not semantic verification.

## Accepted coverage

| Surface | Source reconciliation | Reviewed contracts |
|---|---|---|
| Native imports/types | Compiled binding registry and exhaustive native identity lookup | JSON, host, storage, collections and testing reviews below |
| Bundled testing | Export declarations, wrapper source and private bridge access | Public exports/matchers and all private runtime helpers |
| Receiver members | Checker dispatch, rewrites, emitter targets and catalog examples | Strings, arrays, collections and scalar formatting |
| Namespaces/globals | Parser/checker targets, constants, name resolution and fallback help | Numeric globals, Math/Number, Object/console, constructors and primitives |
| Error/option record fields | Compiled native field declarations | Reviewed within their owning type rows; no separate imported spelling |
| JSON placement | Approved distinct throwing/Result contracts | Naming accepted; implementation, tests and S10 milestone acceptance remain pending |

The generated inventory is discovery input and deliberately keeps `review: pending`.
The explicit coverage manifest and linked semantic reviews hold acceptance, rather
than mechanically changing discovery rows to accepted. Counts include types,
namespace headings, properties and private bridges; they do not count independent
runtime functions. New identities must receive a review before coverage verification
passes. The closure review records source scope and exclusions.

## Numeric global review

The following six globals are recognized independently by
`syntax.rs::global_number_member`, checked through
`analysis.rs::global_number_name`, and emitted through the existing Number call
or constant targets. Local, pending and function declarations take precedence.
Bare function references are rejected; callers can use a typed callback wrapper.
No import migration is needed. Keep both existing spellings, recommending the
familiar global for simple parsing/predicates and Number members when grouping
numeric utilities. Neither spelling adds a second implementation.

| Symbol | Purpose and signature | Failure and ownership | Backend/dependency | Disposition |
|---|---|---|---|---|
| `parseInt` | Integer prefix: `(text: string, radix?: int): number`; omitted/zero radix selects decimal with hexadecimal-prefix recognition | Missing digits or invalid radix returns NaN; wrong types/arity rejected. Borrows text, copies radix; receiver absent | `NumberParseInt`, `runtime_strings.rs::tsr_number_parse_int`; bundled string support | Retain global and `Number.parseInt`; no implicit string conversion |
| `parseFloat` | Decimal prefix: `(text: string): number`, including signed Infinity | Missing numeric prefix returns NaN; wrong types/arity rejected. Borrows text; receiver absent | `NumberParseFloat`, `runtime_strings.rs::tsr_number_parse_float`; bundled string support | Retain global and `Number.parseFloat`; no implicit string conversion |
| `isNaN` | `(value: number): boolean`, tests NaN | Wrong types/arity rejected; copies scalar, no mutation | `NumberIsNaN`, emitted `f64::is_nan`; no imported native feature | Retain global and `Number.isNaN`; document strict number input |
| `isFinite` | `(value: number): boolean`, excludes infinities and NaN | Wrong types/arity rejected; copies scalar, no mutation | `NumberIsFinite`, emitted `f64::is_finite`; no imported native feature | Retain global and `Number.isFinite`; document strict number input |
| `NaN` | `number` constant for not-a-number | Assignment rejected unless shadowed; scalar copy; unequal to itself | `NumericConstant::NumberNaN`, emitted f64 constant | Retain global and `Number.NaN` |
| `Infinity` | Positive infinite `number` constant; unary minus produces negative infinity | Assignment rejected unless shadowed; scalar copy | `NumericConstant::NumberPositiveInfinity`, emitted f64 constant | Retain global and `Number.POSITIVE_INFINITY` |

Source anchors: `syntax.rs::global_number_member` and `NumericConstant`,
`analysis.rs::number_parse_int`, `number_parse_float`, `number_predicate` and
`global_number_name`, `emit.rs` Number target arms, and
`runtime_strings.rs::tsr_number_parse_int` / `tsr_number_parse_float` (all under
`crates/tsrust-compiler/src`). Catalog rows in `globals` and `Number` agree with
these signatures. LSP uses the global catalog and resolves user declarations
before builtin help. Three compiler/native tests in `global_number_functions.rs`
and the focused editor test pass, covering values, errors, callback use,
shadowing, hover, completion and parameter help. Evidence:
`tsrust-api-globals-compiler.log` and `tsrust-api-globals-editor.log` in SQLite.

## Editor fallback discovery

The catalog-only pass omitted six documented spellings: `Math`, `console`, `int`,
`number`, `string` and `boolean`. Their hover text lives in
`crates/tsrust-lsp/src/intelligence.rs` rather than `builtins.json`. `void` also has
fallback text but already had a catalog row. The inventory now discovers these
fallback arms and records their source anchor, producing 399 rows. This is a
discovery correction, not a public API change. Review of these spellings and receiver/namespace operations is now complete in
the linked slices; the original discovery correction remains recorded here.

## JSON review slice

All twelve current JSON exports have a [per-symbol review](globals-and-imports-json-audit.md),
including ownership, errors, dependencies and the approved migration disposition.
The completed cross-library disposition is recorded in the closure review.

## Host-service review slice

All thirteen path, OS, environment and time exports have a
[per-symbol review](globals-and-imports-host-audit.md). Explicit imports remain
preferred; the existing throwing relative-path exception is recorded.

## Storage review slice

The sixteen byte and fifteen file exports have a
[per-symbol review](globals-and-imports-storage-audit.md). Keep explicit imports,
real borrowed views and consuming resource operations.

## Testing review slice

The [testing review](globals-and-imports-testing-audit.md) covers 18 public exports,
29 matcher members and 14 private bridges. Public imports and private retention
boundaries remain unchanged; completion visibility is verified.

## Imported collections review slice

The [collections review](globals-and-imports-collections-audit.md) covers four
constructors/types and 29 members. Keep specialized collections imported.
Generated helper selection is currently by family; per-method elimination is not
claimed. Global Set/Map and remaining receiver/namespace surfaces are reviewed below.

## Global Set/Map review slice

The [Set/Map review](globals-and-imports-set-map-audit.md) covers 29 constructors,
receiver members and static grouping entries. Preserve global placement and
explicit native ownership/copy contracts. Other namespace and receiver families are covered below.

## Numeric namespace review slice

The [Math/Number review](globals-and-imports-math-number-audit.md) covers 58
namespace members, independently reconciled against compiler targets/constants.
Keep global placement and explicit numeric contracts. Number conversion, other namespaces and receiver/type surfaces are covered below.

## Object/console review slice

The [Object/console review](globals-and-imports-object-console-audit.md) covers 27
members, reflection limits, output effects and console argument exceptions.
Keep global placement. countReset parameter documentation now matches its existing
non-string-label exception; runtime behavior is unchanged.

## String review slice

The [string review](globals-and-imports-string-audit.md) covers 50 receiver/static
members, indexing units, errors and copying costs. Preserve current spellings;
string conversion, remaining receiver families and type primitives are covered below.

## Scalar conversion/formatting review slice

The [scalar review](globals-and-imports-scalar-audit.md) covers fourteen global
conversion and scalar receiver APIs. Preserve global/receiver placement and
explicit rounding, parsing and fixed-locale contracts.

## Array review slice

The [array review](globals-and-imports-array-audit.md) covers 43 receiver/static
APIs, including ownership and callback exceptions. Preserve placement; corrected
at hover's stale nullable-element restriction without changing runtime behavior.

## Language primitive review slice

The [primitive review](globals-and-imports-primitives-audit.md) covers the final
22 discovered rows. [Coverage manifest](globals-and-imports-coverage.json) maps
each current identity to its review. Per-row accounting and independent discovery
reconciliation are complete; see the closure review for acceptance scope.

After generating a fresh inventory, run
`python3 tools/verify-api-audit.py /path/to/inventory.json`. The verifier rejects
unreviewed, stale or duplicated identities and missing review files. It verifies
coverage accounting, not semantic correctness or discovery completeness.
