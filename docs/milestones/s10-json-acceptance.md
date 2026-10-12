# S10 JSON acceptance

S10 is accepted for the documented JSON contract. Source acceptance, measurement
scope and retained limitations are distinguished below. S11–S21 remain pending;
this milestone does not imply JavaScript compatibility beyond the authored scope.

## Correctness and ownership

The [contract audit](s10-json-contract-audit.md) and [JSON plan](../plans/json-plan.md)
map dynamic and direct typed decoding, encoding, exact numeric extraction,
Unicode/errors, missing/null/undefined, duplicate keys, resource limits, JSONC,
JSONL streaming, typed file imports and aliases to executable witnesses. Typed
imports retain immutable canonical data with explicit clone for mutation.

Raw fragments preserve real source lifetimes through fields, arrays, optional and
recursive records and returns. Exact-text encoding preserves whitespace, ordering,
escapes and number spelling while enforcing enclosing depth and output limits.
JSONL rejects literal CR/LF before writing a raw record. No hidden source owner,
clone, static lifetime or unsafe erasure was introduced. The selected subtree
still requires a depth scan; its measured cost is retained.

The [final regression reconciliation](s10-json-regressions.md) accounts for all
359 compiler/LSP test and doctest targets: 3,436 compiler tests and 582 LSP tests
pass, with four intentional ignores. Original failures remain preserved. Focused
repairs correct three suppression-sensitive expectations and one missing JsonRaw
hover code block. The 317 unaffected intelligence successes were retained when
only the failed catalog test was repeated. No target is missing or aborted.

## Measurements and review

All 22 JSON workload registrations link to immutable native/Node execution records
with matching current fixture hashes and original sample counts. The full shape
matrices retain decoding, encoding, streaming and raw validation/selection costs
against direct Rust and Node, including invalid inputs, Unicode and deep data.
Allocation/retention, process RSS, first-operation latency and warmed timing remain
separate metrics. The metrics exporter exposes 387 source-keyed rows from ten
records without pooling implementations or rerunning workloads for presentation.

The JSON plan records the bounded-output and raw-depth optimization budgets,
original measurements, creation regression and validation-pass repair. Prepared
raw encoding improves by 14.81x/4.14x/14.61x for the measured numeric/Unicode/deep
cases without changed allocation counters. Creation now collects depth during
validation. Selective-field creation remains 13.7% above its earlier baseline,
with a 12.6% direct-Rust control increase; no statistical-equivalence claim is made.
The raw end-to-end workload takes median 0.270250 ms native versus 0.321667 ms Node
for 2,000 operations. Node explicitly preserves input text after validation, so
this is not an ordinary JSON.stringify comparison. Earlier runtime records retain
their original compiler identities; source-matching is not a claim of remeasurement.

These results satisfy the authored measurement coverage and review requirement,
not a universal speed advantage. Larger typed-decoding/Unicode costs and differing
validation budgets remain documented. No safety check was weakened for performance.

## Public surface and installed consumers

Fresh compiled discovery reconciles all 433 public identities exactly once.
JSON retains four namespace and 41 std:json catalog entries. Signatures, parameter
help, hover, completion and executable examples pass their regression witnesses.
The [raw API audit](../plans/globals-and-imports-json-raw-audit.md) records placement,
ownership and serialization semantics without adding another global convention.

CLI consumers were refreshed from a462dc76; the final LSP correction was refreshed
from 54aa804e. Installed native output, raw parse/type/encoding hover and complete
nested/nullable JSON import hover pass. Both LSP copies have SHA-256
94f8fdfc4052281f6b35706fe180ef947924b130883c9c015c6c9b209bac088d.
The live VS Code server was restarted and shows checking enabled, zero errors and
45 warnings. Those warnings are not claimed resolved. Extension compilation,
discovery/grammar checks and isolated extension/module/std integration pass.

Final consumer evidence uses `tsrust-s10-final-*-54aa804e` in SQLite. Cleanup uses
`tsrust-s10-cleanup-{plan,result}-54aa804e.json` and
`tsrust-s10-obsolete-test-cleanup-54aa804e.json`.
Cleanup removes 358 completed test executables and 624 obsolete variants matching
known target names (27.92 GiB apparent size total), leaving the shared target at
7.4 GiB. Consumer hashes remain intact. Future tests must rebuild deleted outputs;
no source, library archives, analytics records or owner files were removed.

The next dependency stage is S11 schema validation. Keep its implementation and
acceptance separate from this completed JSON milestone.
