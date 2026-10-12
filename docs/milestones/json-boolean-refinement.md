# JSON boolean refinement: focused source acceptance

A stable typeof boolean guard permits boolean payload access from JsonValue or
JsonView, including optional carriers. Reads copy the boolean without consuming
the JSON wrapper. The proof follows ordinary mutation invalidation, branch joins
and loop back-edge checks. No public API identity or import was added.

String and numeric refinement remain pending. This batch does not introduce
implicit integer rounding, string copies or borrowed string lifetime erasure.
Installed consumers were refreshed from 818459f3, including child views and
boolean refinement; see the S10 installed guarded-read acceptance. S10 remains
open, with family regressions and measurements pending.

## Evidence

Original records are retained in benchmarks/records.sqlite:

- tsrust-json-refinement-red.log: initial boolean return rejected.
- tsrust-json-refinement-green.log, invalidation.log, final.log and module.log:
  native boolean returns, explicit/inferred locals and optional child access pass;
  reassignment, false branches, unproven joins and stale loop proofs reject.
- tsrust-json-refinement-diagnostics.log: rejection cases report diagnostics
  rather than compiler panics.
- tsrust-json-refinement-example.log: the updated typeof catalog example executes
  with boolean refinement and its documented output.
- tsrust-json-refinement-editor.log: project checking and boolean-local/keyword
  hover pass.
- tsrust-json-refinement-regressions.log: 16 data-union, two JSON typeof and four
  child-view regressions pass.
- tsrust-json-refinement-contract.log: all seven JSON workload registrations,
  exact fingerprints, operation guards, ABI and independent oracles pass.
- tsrust-json-refinement-smoke.log: five matched native/Node-after-tsc cases
  agree; zero timing samples. Each iteration parses a boolean, checks its category
  and branches on its payload. This does not isolate projection cost from parsing.
- tsrust-json-refinement-format.log and format-final.log: whole-project
  token-preserving formatting, fingerprint refresh and stable final pass.

The next refinement work must implement borrowed string storage and a numeric
representation that preserves exact JSON integers before exposing those payloads.
