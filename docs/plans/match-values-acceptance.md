# Match expressions and scalar/record patterns

Accepted on 2026-10-14. Source checkpoints: `05e0a16a` and `7c3a326c`.

- Concise arms yield their expression. Block arms exit explicitly; return targets
  the enclosing callable. Concise arrow functions retain expression returns.
- String, integer, number, boolean, null and undefined literal patterns are checked
  for coverage. Open scalar types require `_`; finite alternatives can enumerate.
- Named record unions bind the selected record, including three-way unions.
  Result variants bind their payload. No new variant declaration/constructor syntax.
- Shared checker scheduling passes a 64-level match witness on the normal stack.
  No broader depth-scaling claim. Native witnesses cover once-only evaluation,
  source order, short-circuiting, enclosing returns, typed throws/finally, owned
  closures/arrays and readonly/exclusive payload rules. A selected-variant local
  preserves native-view provenance, including discarded error payloads.
- Three readable examples execute after formatting: `examples/match-values`,
  `examples/match-literals` and `examples/match-variants`.
- Focused compiler checks: 7 value-match, 18 Result-match and 7 syntax tests pass.
  The consolidated compiler milestone has 4,008 passes and 7 existing ignored
  tests across 332 targets. Three initial failures were repaired and rerun; original
  evidence remains in SQLite. No unresolved failure remains.
- Consolidated editor milestone: 679 passes and 1 existing ignored test across
  54 targets. Payload hover/navigation, variant completion, unrelated-error recovery,
  typing-speed matrix, signature help and executable catalog examples pass.
- Whole-project formatting preserves tokens and is idempotent. Native/TypeScript
  workload guards and matched checksums pass. The local TypeScript editor config
  uses the existing pinned Node types and passes `tsc --project benchmarks/matching`.
- First timing: `benchmarks/matching/match-values-2026-10-14T21-27-19-471Z.json`
  in SQLite. Three processes, eight warmups and 31 samples per process/engine;
  all checksums 4203521. Native median 0.002167 ms; Node-after-tsc 0.012125 ms.
  No earlier comparable timing exists; this is not an application speedup claim.

Exact records include `tsrust-match-family-compiler-regression.log`,
`tsrust-match-family-editor-regression.log`, `tsrust-match-worker-final-focus2.log`,
`tsrust-match-syntax-final.log`, `tsrust-match-native-view-final2.log`, and
`tsrust-match-worker-editor-final.log`. Closed loose copies are removed only after
byte-exact SQLite restoration. See [default worker acceptance](default-worker-cleanup.md)
for consumer refresh and remaining S12 work.
