# JSON namespace audit addendum

This adds five identities to the accepted 399-identity placement audit. The current
inventory has 404 identities. This review accepts placement and documented contracts;
JSON implementation acceptance remains tracked separately in the policy plan.

| Identity | Disposition and semantics |
| --- | --- |
| globals::JSON | Keep as the approved familiar namespace. Local bindings shadow it. Its name alone does not select runtime dependencies. |
| JSON::parse | Throwing strict parser, returning owned JsonValue. JsonError retains code, message and byte offset. Uses the existing bounded native parser. |
| JSON::tryParse | Explicit Result alternative with the same input rules, limits, error fields and owned result as parse. Reuses std:json.parse. |
| JSON::stringify | Throwing bounded serializer, returning an owned UTF-8 string. Borrows JsonValue; serialization failure throws structured JsonError. |
| JSON::tryStringify | Explicit Result alternative with the same serialization and output-limit semantics. Reuses std:json.stringify. |

The namespace does not return dynamic objects, add unchecked generic decoding,
introduce shared ownership, or change existing imported parse/stringify contracts.
Parsed values own their storage. Inputs are borrowed. Default budgets and numeric,
Unicode, duplicate-key and sorted-key behavior remain those reviewed for std:json.

Compiler recognition lives in stdlib/json/namespace.rs. It resolves members before
native signature specialization and builds typed throwing functions over existing
bindings. The editor catalog contains all four operations; the namespace heading
is discovered from the editor fallback. No new native binding or parser is added.

Evidence includes executable catalog examples, editor hover/signature/completion,
namespace/native contract tests and dependency-selection tests. See the policy
plan for exact SQLite records and remaining feature acceptance work. Performance
measurements are not implied by this semantic review.


## Checked typed decoding extension

Explicit `JSON.parse<T>` now returns checked owned T; `JSON.tryParse<T>` returns
Result<T, JsonError>. Untyped calls and imported std:json.parse retain their
existing contracts. The type argument drives direct decoding rather than an
unchecked assertion. Native dependencies are still selected automatically.

The supported owned shape set, optional/missing/duplicate field policy, checked
numeric conversions and unsupported target diagnostics are documented in the
JSON guide and both catalog entries. Compiler-owned generic adapters preserve
normal typed exception propagation and borrowed arguments. This adds no global
name or native binding inventory row. Focused evidence is recorded in the JSON
plan; performance acceptance remains pending.
