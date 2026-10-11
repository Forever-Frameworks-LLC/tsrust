# Globals and imports: inventory and decision

Status: owner requested a consistency review on 2026-10-11. The inventory below
reflects current source. Recommendations are not approved API migrations. Keep
existing behavior while resolving this policy; alias and typed-data import work
can continue independently.

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
| JSON core | std:json parse, stringify, kind, JsonValue, JsonError, JsonLimits | Decide global JSON surface and error contract before introducing aliases |
| JSON variants and resources | parseJsonc, JsonlReader, openJsonl, readJsonl, closeJsonl | Keep specialized APIs imported unless a later reviewed design supersedes this |

These classifications describe placement, not JavaScript compatibility. For
example, native string indexing and ownership rules remain explicit even where
spelling is familiar. `std:testing-runtime` is an implementation bridge, not a
reason to advertise a second public testing convention.

## Tradeoffs and recommended rule

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

## JSON decision still needed

The existing `parse` returns `Result<JsonValue, JsonError>` and accepts resource
limits. Existing `stringify` also returns a Result and accepts an output byte
limit. A global name does not decide their error behavior or erase these limits.

| Option | Benefit | Cost |
|---|---|---|
| JSON.parse/stringify return checked Result values | Reuses existing safe contracts and explicit error handling | Familiar JavaScript spelling has different return/error behavior |
| JSON.parse/stringify throw typed errors and return checked values | Familiar control flow and names; JsonValue still requires safe inspection | Adds a distinct throwing contract and must define limits and error conversion |
| Keep core JSON imported | No migration or second contract | Preserves the naming inconsistency identified by the owner |

Recommend the familiar JSON namespace for eventual placement, but obtain the
owner's choice of error contract before changing the public surface. Do not
infer approval of a throwing API, Result-returning global, `any`, or unchecked
`JSON.parse<T>` from the naming discussion alone. A safe typed decoder needs
actual validation, not a generic assertion.

Typed JSON/JSONC file imports are separate and already approved. Their shapes
are known at build/check time and must appear in the editor. They do not depend
on choosing a runtime global JSON API.
