# B11/S07 resources and bytes acceptance map

This is the current implementation map. S07 is not accepted or measured.

| Scope | Status | Authoritative evidence |
| --- | --- | --- |
| Typed native declarations, signatures, imports, aliases and Result payloads | Gate 187 accepted | milestones/187-native-byte-foundation.md |
| Owned byte construction, access, mutation, allocation, growth and range copying | Gate 188 accepted | milestones/188-owned-byte-operations.md; native-owned-bytes fixtures |
| Independent clone, generic/record/array/Map/nullable/Result storage and callable captures | Gate 188 accepted | ownership.tsr; native_declarations.rs; project_intelligence.rs |
| Validation/reservation atomicity and one argument evaluation | Gate 188 accepted | bytes/runtime.rs tests; access.tsr; growth.tsr; copy-append.tsr; ownership.tsr |
| Exact disposal and allocation restoration at O0/O3 | Gate 188 accepted | native_byte_cleanup.rs; cleanup.tsr; exact SQLite gate 188 records |
| Scoped zero-copy readonly/exclusive views, provenance, conflicts and escape diagnostics | Gate 189 pending; private borrowed identities, contracts and slice helpers implemented; source integration pending | native-byte-views-implementation.md; resources-bytes-plan.md |
| Strict UTF-8 encoding/decoding and error offsets | Implemented increment; gate 189 still pending views | native-owned-bytes/utf8.tsr; utf8_runtime.rs; resources-bytes-plan.md |
| Opaque resource consuming close, drop and failure ownership | Gate 190 pending | owner consuming-close decision; resources-bytes-plan.md |
| Useful synchronous file proof and private instrumented resource backend | Gate 190 pending | resources-bytes-plan.md |
| Complete regressions, matched measurements and optimization review | Family milestone pending | resources-bytes-plan.md; standard-library-roadmap.md |

All current byte workloads have matching sources, independent oracles, source hashes,
ABI support and harness hooks. Untimed verification is distinct from measurement.
Native thread-transfer metadata is backend eligibility, not source threading acceptance.
The owner chose consuming close even on Err, static rejection of later use/double
close, and a nonthrowing destructor fallback. Bytes does not require explicit close.
