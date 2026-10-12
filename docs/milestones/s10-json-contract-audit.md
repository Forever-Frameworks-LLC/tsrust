# S10 JSON contract audit before public raw fragments

The table below preserves the audit at `81678b22`. The current reconciliation
at `5682a032` follows it; earlier raw gaps must not be read as current status.
S10 is **not accepted**: raw serialization now has an approved preservation policy
and initial backend/source integration; remaining implementation/acceptance is pending. Existing source and measured evidence
remain canonical in SQLite.

| Contract requirement | Source and executable witnesses | Assessment |
| --- | --- | --- |
| Dynamic and direct typed decoding | `stdlib/json/runtime.rs`, `typed.rs`; `tests/json.rs`, `json_typed.rs`. The emitted-code witness requires typed visitors and rejects a dynamic parse bridge; allocation measurements retain complete outputs. | Implemented and checked; typed validation is a separate bounded pass, not schema fusion. S11 owns schema fusion. |
| Missing/null/undefined | `json_typed.rs` checks required, nullable and optional fields; `json_encoding.rs` checks omitted optional fields and rejects undefined root/array entries; `json_views.rs` separates absent children from null. | Implemented within supported data types. |
| Duplicate keys and ordering | Dynamic maps replace earlier keys; typed record occurrences must each satisfy their type and the last valid occurrence wins. Recursive typed encoding witnesses assert sorted output. | Deliberate typed/dynamic distinction documented in JSON plan and IntelliSense. |
| Numbers | `typed.rs::typed_numbers_never_round_integer_payloads`, scalar backend tests and native typed witnesses cover exact integers, checked fractional/range/precision errors and negative zero. Encoding rejects nonfinite numbers. | Implemented; no implicit exact-integer-to-float rounding promise. |
| Unicode and errors | Runtime tests assert surrogate/UTF-8 policy, escaped output, original byte offsets, trailing data, truncated input and redacted messages. `lines.rs::records_survive_every_chunk_boundary_and_eof_is_not_null` sweeps buffer boundaries across Unicode and escapes. | Implemented and covered. |
| Resource limits | Runtime Budget and typed validation enforce bytes/depth/entries/nodes, including ignored fields and duplicate entries. Output writer checks encoded bytes; JSONL uses bounded file/record buffers. | Implemented; allocation and RSS metrics remain distinct. |
| Serialization and ownership | Native encoding covers recursive trees and nullable links, borrows input, and rejects callbacks/unsupported collections. Unique ownership does not create shared cycles. Depth checks bound recursive traversal. | Supported data contract checked; no arbitrary JS-object or shared-cycle support claimed. |
| Explicit relaxed formats | Strict parse rejects comments; JSONC has explicit trailing-comma policy and preserves source offsets. Typed JSONC witnesses exercise the same limits. | Implemented. |
| Streaming cleanup | JSONL runtime tests cover EOF/null distinction, terminal malformed/I/O errors, interrupted reads and early drop. Native borrowed reader/writer witnesses check owner reuse, unread bytes, close/flush and rejection of escape/clone/conflicts. | Implemented; drop cannot report flush errors, explicit close can. |
| Borrowed parsed views | `json_views.rs` and string-view witnesses cover real owner borrowing, access and invalidation/escape rejection. | Implemented; these are not source-backed raw fragments. |
| Raw fragments | `stdlib/json/raw.rs` validates source while preserving a real borrow and all budgets. No public raw type/constructor, embedding behavior, catalog examples or raw workload exists. | **Incomplete.** Preserve-versus-normalize output decision pending. Public API, ownership checks, editor surface, native examples, allocation/timing/RSS and consumer refresh must follow. |
| Typed file imports and aliases | `data_imports.rs` checks one canonical embedded immutable value, explicit clone for mutation and offline builds without runtime input files/parser linkage. Project-intelligence tests cover complete binding hover and foreign references. | Implemented and installed smoke checked. |
| Globals/import naming and linking | `json_namespace.rs`, `json_dependency_selection.rs` cover throwing/Result forms and actual-use linking, including local namespace namesakes. Public naming audit is recorded in JSON plan. | Implemented surface audited; raw additions must extend the same audit. |
| IntelliSense and examples | Current catalog has four JSON namespace and 39 std:json entries with signatures/documentation. The regression checkpoint executes 41 std:json examples and retains focused namespace/typed/view/editor checks. | Accepted for current surface; structural catalog inspection alone is not execution proof. |
| Regression and installed consumers | [Regression checkpoint](s10-json-regressions.md): 3,395 compiler and 576 editor tests after preserved focused repairs. Runtime optimization adds affected 42 runtime/three native checks. Installed 12a6426e CLI/LSP hashes and borrowed/native/hover smokes are retained. | Current implementation verified; future raw changes need affected checks and refresh. |
| Performance | JSON plan records original/optimized allocations, 20 matched API workloads, direct Rust/Node encoding/decoding/streaming timing and RSS matrices, plus throughput/p50/p95 derivation. Original records remain immutable. | Covered for implemented shapes. First-operation timing is not disk cold-cache timing. Raw payload/field measurements remain missing. |

Evidence remains canonical in SQLite. `tsrust-json-contract-audit-81678b22.json`
records inspected source hashes and catalog counts; it is a read-only source audit,
not a new test run. Existing full regression evidence is not rerun or discarded.

The measured encoding optimization has a pre-review budget and explicit small-value
memory tradeoff. Valid-input typed decoding and large Unicode encoding still cost
more than direct Serde. This is documented cost, not a hidden claim of parity or a
reason to weaken checks. Further optimization requires evidence and preserved
correctness. S10 closure requires the missing raw feature and its acceptance, then
a final audit incorporating that work; S11–S21 remain outside this audit.

## Reconciliation at 5682a032

- Raw validation and typed raw-field decoding are implemented with real source
  lifetimes. Compiler witnesses cover arrays, optional/recursive records, field
  extraction, borrowed returns, owner conflicts, escapes and clone rejection.
- Raw numeric, Unicode, deep, rejection and selective-field allocation evidence
  is retained alongside native/direct-Rust/Node timing and separate RSS matrices.
  The metrics exporter includes 45 raw case/engine rows, for 252 total derived
  rows from five immutable records. It does not pool samples between records.
- The catalog now has four JSON namespace entries and 41 std:json entries. The
  two raw imported identities are explicitly reviewed in the raw API addendum;
  fresh compiled discovery reconciles all 433 identities exactly once.
- Installed CLI/LSP consumers contain raw decoding. Native installed witnesses,
  source hashes and LSP diagnostics/hover are verified. The hover placement defect
  found by the installed smoke was repaired at 5682a032; its failed evidence stays.
- Original full regression totals remain historical. Subsequent focused tests
  supplement them; they do not constitute a new complete regression pass.
- Remaining S10 work: decide and implement raw serialization semantics, verify
  its ownership/limits/examples/editor behavior and matched measurements, then
  perform the final affected acceptance and integrated contract review. S11–S21
  remain pending. No current measurement proves serialization acceptance.
