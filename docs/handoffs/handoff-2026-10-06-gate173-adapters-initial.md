# Handoff: initial generic value adapters

Read AGENTS.md, docs/development-workflow.md, both function plans and
docs/plans/generic-value-adapters-plan.md. Red checkpoint: 1678da5. Gate173 and full
adapter acceptance remain open.

generic_value_adapters.rs solves the original declaration using concrete context
or an array transform's supplied parameter types. It builds an ordinary checked
wrapper whose capture is the existing generic value environment. A private
lexical alias prevents parameter-name shadowing; closure capture projection now
retains all lexical aliases. Wrapper source defaults still forward omission to
the declaration rather than evaluating a second initializer.

Owned wrappers receive temporary synthetic escape classification while checking.
Borrowed wrappers retain the original loans. Classification and private context
names are restored on success and error. Array transform analysis uses its checked
adapter once, avoiding replay after the private lexical name has been removed.
The compatibility guard remains for contexts without actual adapters. No void
wildcards, owner cloning or shared capture cells were introduced.

All three original native witnesses pass: concrete borrowed/owned contracts,
map callbacks and typed record/array storage. Additional controls reject original
environment reuse after transfer, live exclusive access, consuming reuse and
incompatible type/access relations. Native O0/O3 checks observe exactly three
retained record owners dropped once across successful and caught-failure paths.
Earlier invalid-prototype uses still fail; the three affected rejection
assertions now require arity/type errors instead of the superseded unsupported
boundary. Original failed evidence remains preserved.

Evidence: 17 generic-function-value tests; five adapter tests followed by the
added exclusive/consuming control; all eight depth canaries (63 tests); whole
editor crate 298 tests. Adapter editor help and original alias navigation pass.
The original strict TypeScript references remain in SQLite from 1678da5; their
fixture sources are unchanged. All processes are terminal.

Five existing generic_function_results native/Node-after-tsc cases pass untimed.
Canonical control record:
benchmarks/runtime/results/2026-10-06T23-49-37-642Z-verify.json.
This is result-context regression evidence, not adapter benchmark coverage.
Adapter-specific pair/oracle/guards/registration/hooks still must be implemented
before accepting the adapter family. No timing or full gate acceptance is claimed.

Thirteen current logs use tsrust-generic-adapter- prefixes. Original failures and
repairs are imported into benchmarks/records.sqlite with exact readback verified;
loose copies are removed. All 119 compiler and 262 oracle input hashes match final
source. Five consumers are atomically refreshed and hash-verified. Obsolete
executable scratch is pruned, retaining libraries and five consumers. Four unrelated owner fixtures
remain untouched and uncommitted. Hourly checks stay paused.

Next complete adapter benchmark support and audit default/rest/pattern/result-only
and nullable contexts. Integrate reduction, search, sort and other callback paths:
currently array_transform owns traversal adaptation. Check arbitrary callable
expressions, consuming arguments, callback failure and imported nominal contracts.
Do not broaden guards or rebuild environments to make unsupported paths pass.
Continue conditional/optional alias/editor audits and the full gate173 plan.

Continuation prompt: Continue Tsrust from this handoff. Complete matched adapter
benchmark support, then widen real adaptation to defaults/rest/patterns/nullable
contexts and the remaining callback families. Preserve ownership, exact disposal,
owner fixtures and SQLite evidence. Run bounded tests and required canaries/editor
checks. Do not mark gate173 complete until its entire plan and audit pass.
