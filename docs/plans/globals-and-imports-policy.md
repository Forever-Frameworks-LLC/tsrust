# Globals and imports: inventory and decision

Status: owner approved this placement policy and the throwing `JSON.parse` /
Result-returning `JSON.tryParse` pair on 2026-10-11. Implementation is in progress;
the namespace gate is not accepted.

The four namespace entry points now work in project and single-source analysis.
Result operations resolve to existing native bindings. Throwing operations use
compiler-owned checked functions over the same backend, preserving structured
`JsonError`, ordinary typed failure propagation and ownership. Local JSON bindings
retain their meaning; actual operations select dependencies automatically.

Six focused tests pass: five namespace/native tests plus a dependency-selection
test with eight cases. Coverage includes unchanged imported Results, stored
function references, nested shadowing, callback error propagation, `finally`,
custom parse limits, serialization limits and error-field parity. The original
throwing contract failure is resolved. Evidence is retained in SQLite as
`tsrust-json-namespace-throw-1.log`, `tsrust-json-namespace-throw-3.log` and
`tsrust-json-namespace-throw-4.log`. The third run passed unchanged cases; its
new limits witness needed a named JsonLimits annotation, then passed in the fourth
run. The second run's misplaced test delimiter is retained too. Earlier Result
and red-contract records remain preserved.

Namespace hover, completion and signature help now document all four methods,
including arguments, limits, errors, ownership and executable examples. Two focused
editor tests pass, including local receiver shadowing and public JsonValue display
without an import. Four catalog examples pass native execution. Evidence:
`tsrust-json-editor-3.log` and `tsrust-json-catalog-1.log` in SQLite. Earlier editor
harness failures and the native-name hover failure are retained separately.

Six invalid-call cases produce public diagnostics. Loop, catch and named-function
capture shadowing pass native checks. The capture witness exposed incorrect
lowering after named-function lifting; namespace resolution now consults retained
lexical variable-use identities. Nine focused tests pass, including all four
catalog examples and dependency checks. A separate dependency witness confirms
that a captured local JSON record does not select the JSON runtime.

The API audit now reconciles 404 identities, adding the namespace heading and four
methods in globals-and-imports-json-namespace-audit.md. Evidence:
`tsrust-json-shadow-3.log`, `tsrust-json-dependency-capture.log`, and
`tsrust-json-audit-verification.log`; failed capture and initial diagnostic records
are retained too. Matched workload support and final focused acceptance remain
pending. This checkpoint is not a consumer release or measured S10 acceptance.
The complete current per-symbol audit is accepted in
[the closure review](globals-and-imports-audit-closure.md). The family summary below
is supplemented by 404 explicit reviewed identities; it is not the audit by itself. Existing working imports
remain supported. Alias and typed-data import work retains its current priority.

The current [audit coverage matrix](globals-and-imports-audit.md) records discovery
evidence and remaining per-symbol review.

## Current inventory

Source anchors: `crates/tsrust-lsp/src/builtins.json`, the compiler's
`native_bindings.rs`, and `stdlib/*/bindings.rs`. Catalog entries include methods,
constants, types and language operators; counts must not be presented as a count
of independently implemented functions.

| Current surface | Examples | Recommended disposition |
|---|---|---|
| Familiar utility namespaces | Math, Number, Object, console | Keep existing globals; document supported semantics |
| Familiar constructors and numeric globals | String, Boolean, Array, Set, Map, parseInt, parseFloat, isNaN, isFinite, NaN, Infinity | Preserve existing spellings; avoid adding imported duplicates |
| Tsrust ownership and type primitives | clone, Result, Ok, Err, Mutable, ReadOnly, Once, Owned | Keep language-level availability; clearly identify Tsrust extensions |
| Imported host services | std:fs, std:path, std:os, std:env, std:time | Keep explicit imports; resources and lifecycle remain visible |
| Imported native storage | std:bytes, std:collections | Keep explicit imports for Bytes, views, Deque, Heap and ordered collections |
| Imported testing API | std:testing | Keep explicit imports; do not introduce Jest-style ambient test globals |
| JSON core | std:json parse, stringify, kind, JsonValue, JsonError, JsonLimits | Implement approved JSON namespace with distinct throwing and Result entry points |
| JSON variants and resources | parseJsonc, JsonlReader, openJsonl, readJsonl, closeJsonl | Keep specialized APIs imported unless a later reviewed design supersedes this |

These classifications describe placement, not JavaScript compatibility. For
example, native string indexing and ownership rules remain explicit even where
spelling is familiar. `std:testing-runtime` is an implementation bridge, not a
reason to advertise a second public testing convention.

## Approved placement rule

Globals reduce ceremony and make familiar code discoverable. Their costs are
ambient namespace growth, shadowing questions and compatibility expectations.
Imports make dependencies explicit and support local aliases; their costs are
boilerplate and unfamiliar spelling for otherwise familiar language utilities.
Neither choice should determine whether unused runtime features are linked.

Prefer familiar global utility namespaces, language-level ownership primitives,
and explicit imports for native services, resource owners and specialized APIs.
Avoid two equally recommended spellings for every operation. Keep old supported
imports during any migration, with one documented preferred surface and a tested
compatibility policy. Do not remove working APIs as part of this inventory.

All new global surfaces need checked shadowing/name resolution, completion,
hover, signature help, diagnostics, examples and selected-feature linking. An
editor-only JSON namespace or eager inclusion of all JSON runtime code is not
acceptance.

## Approved JSON contract

- `JSON.parse(text, limits?)` returns owned `JsonValue` on success and throws a
  typed `JsonError` on failure. It never returns `any` or claims an unchecked
  record shape. Callers may use normal try/catch.
- `JSON.tryParse(text, limits?)` returns `Result<JsonValue, JsonError>`. Parsing,
  limits, offsets and error information must match the throwing entry point.
- Preserve the current `std:json.parse` Result contract for compatibility; do not
  silently change the meaning of existing imports.
- Apply the same distinct-name pattern to serialization in the implementation
  design: `JSON.stringify` for throwing encoding and `JSON.tryStringify` for
  Result-based encoding, retaining output limits and existing imported behavior.
- Both paths use the same validated backend. Changing error delivery must not
  introduce implicit clones, hidden shared ownership, unbounded parsing, loss of
  error fields, or a second parser with diverging acceptance rules.
- Typed error propagation must work through functions and callbacks, with catch
  inspection and cleanup verified. No panic-based error conversion or flattened
  message-only substitute for JsonError.

Example target behavior (not implemented yet):

```typescript
const data = JSON.parse(text); // throws on malformed input
match (JSON.tryParse(text)) {
  Ok(data) => { /* inspect the checked JSON value */ },
  Err(error) => { /* handle the structured parse failure */ }
}
```

Runtime parsing still requires safe shape inspection or actual validated typed
decoding. An unchecked `JSON.parse<T>` is not authorized. Typed JSON/JSONC file
imports remain separate: the compiler infers their shape from validated input
at build/check time and exposes that same type to the editor.

## Required thorough audit and acceptance

Before adding the global JSON convention, complete an exhaustive per-symbol
inventory of current globals, constructors, namespace members, imported public
exports and private bridges. The family table above does not satisfy this gate.

1. Reconcile compiler recognition, type checking, ownership/effects, native binding
   registration and emission against editor catalogs and completions. Include
   globals implemented outside the native registry and imports implemented as
   bundled source. Record omissions, editor-only claims and undocumented exports.
2. For each public symbol record current spelling, category, global/imported
   placement, purpose, argument/result types, failure behavior, ownership effects,
   runtime dependencies, source locations and proposed disposition. Distinguish
   language primitives from library operations and private bridges.
3. Review every disposition against familiarity, namespace cost, dependency
   visibility, aliases, shadowing, semantic differences and migration impact.
   Do not manufacture JavaScript compatibility from matching names. Retain
   intentional Tsrust extensions with explicit explanations.
4. Record accepted placement, justified exceptions and remaining decisions in a
   coverage matrix. No silent removals or unrelated mass renaming. Preserve old
   supported imports while publishing one preferred spelling per error contract.
5. Implement the approved JSON pair only after the audit identifies its consistent
   placement. Test success/error equivalence between both paths, default and
   explicit limits, numeric/Unicode boundaries, malformed input, structured catch
   fields, propagation, cleanup, shadowing and import compatibility.
6. Verify completion, hover, signature/parameter help and diagnostics show actual
   return types, errors, limits and ownership. Execute the documented examples.
   Checked runtime JsonValue and inferred file-import types must not be conflated.
7. Prove usage-based linking: a program without JSON and a program that shadows
   the name must not acquire JSON dependencies. Actual JSON operations must select
   required support automatically in build/run/test/dev and exported packages.
8. Record focused compiler/native/editor checks and matched benchmark support as
   implementation acceptance; preserve S10 family regressions and measurements
   as separate outstanding requirements. Refresh installed consumers only from a
   verified checkpoint and verify real editor behavior.

Store immutable audit/check evidence in SQLite and keep this plan plus the current
coverage matrix concise. Audit completeness requires every discovered public
symbol to have a reviewed row; a family count or passing examples alone is not
proof of exhaustive coverage.
