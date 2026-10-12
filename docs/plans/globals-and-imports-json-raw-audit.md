# Raw JSON API placement and ownership review

`std:json::JsonRaw` is an explicit imported type for validated source-backed JSON.
`std:json::borrowRaw` is an imported constructor returning Result<JsonRaw,
JsonError>. The name exposes borrowing; the signature and documentation describe
input limits, errors and the input lifetime. Neither introduces a global name.
JsonRaw cannot be cloned or constructed directly and has no implicit shared owner.

Existing JSON.parse<T>/JSON.tryParse<T> accept JsonRaw as an explicit opt-in target
or field type. Their ordinary owned result contract stays unchanged when T has no
raw fields. Raw results retain the input borrow, including through record storage,
array elements and function returns. JSONC and JSONL keep owned output because
normalization and streaming use temporary input storage. The approved serialization
policy preserves original raw text through JSON.stringify/JSON.tryStringify,
including whitespace, key order, escapes and numeric spelling. Existing enclosing
depth and output byte limits apply. These operations borrow the fragment and add
no public names. JSONL writers accept single-line raw text but reject literal CR/LF
before writing the record; escaped newline sequences remain valid. JSONL decoding
still requires owned fields.

The native binding registry and native type table both contain these two imported
identities. The editor catalog exposes matching declarations and executable
examples. Typed raw fields use the existing JSON namespace signatures; they add
no further names. Native execution, invalidation/escape/clone rejection and the
installed LSP hover checks are retained in the JSON plan's raw acceptance evidence.
The manifest grows from 431 to 433 reviewed identities; fresh compiled discovery
must match each identity exactly once.

Fresh compiled discovery at f524c755 again reconciles all 433 identities exactly
once. The catalog retains four JSON namespace and 41 std:json entries. The six
encoding-related identities have nonempty signatures, documentation and examples;
no parse entry claims raw encoding is unsupported. This structural audit
supplements, rather than replaces, the executable catalog and LSP protocol tests.
SQLite evidence: `tsrust-json-final-api-{inventory,review}-f524c755` and
`tsrust-json-final-catalog-audit-f524c755.json`.
