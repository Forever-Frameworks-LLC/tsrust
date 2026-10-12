# S10 implemented-feature regression checkpoint

The complete workspace run at 6d40badf reached all 355 Cargo test/doctest targets
from the captured metadata inventory. No targets were missing or aborted. The
original run has four failures and remains preserved. Focused repairs establish
3,395 compiler tests and 576 editor tests passing, including one new alias
regression. Four intentional ignores remain: worklist/front-end/editor measurement
probes and the cache subprocess worker exercised by its parent test.

Repairs:

- The JSON example runner froze its catalog size at 12. It now verifies unique,
  nonempty entries and examples, then executes all 41 examples across 39 entries.
- Two union editor tests invoked bare rustc after their examples gained JSON
  dependencies. They now use automatic CLI linking, formatted source, disposable
  directories and exact native output checks. Nullable examples retain debug and
  release coverage. Their editor assertions needed no semantic changes.
- Exported names in aliased imports were omitted from references. Resolution now
  maps the exported token to its foreign alias binding before local namesakes.
  Both the original reference-count test and a new collision case pass. All 116
  project-intelligence tests pass, including complete imported JSON binding hover.

Other completed evidence includes VS Code build/discovery/grammar and isolated
integration with the current shared language server, plus 26 JSON benchmark,
dependency-linking and measurement-protocol checks. No complete suite was restarted
for the repairs; only affected checks were repeated. Intermediate failed repair
attempts remain preserved alongside the original results.

SQLite records: tsrust-s10-regressions-6d40badf.log, the matching test inventory and
regression audit, tsrust-s10-regression-reconciled-9fd1874e.json, and the focused
repair/extension/benchmark logs referenced there and in the JSON plan.

This accepts regression correctness for the implemented feature set. It does not
close S10: public raw fragments, latency/throughput/RSS and wrapper-cost review,
justified optimization, and final installed-consumer refresh remain required.
Future changes need their affected checks; these completed results remain valid
for unchanged behavior and must not be silently discarded.

## Final raw-feature regression continuation

The full run at f524c755 completed 905 compiler library tests (one intentional
ignore), 13 CLI unit tests and the 25-test acceptance target before stopping on
two stale acceptance expectations. Both expected reporting-only check to reject
fixtures carrying intentional editor suppression directives. Strict analysis still
rejected those sources.

The repaired corpus test checks strict analysis and CLI emission of the original
fixtures, with no Rust backend available. The separate reporting test disables the
directive in its temporary copy and retains exact diagnostic JSON checks. All 25
acceptance tests now pass. A first repair compile failure is retained alongside
the original run and passing repair; no production compiler behavior changed.

The continuation selects only unrun compiler integration targets, compiler
doctests and the editor suite, preserving completed library/CLI evidence. Its
explicit target inventory and commands are recorded. Final totals and acceptance
remain pending completion and target-by-target reconciliation.

Evidence prefix: `tsrust-s10-final-{regressions,inventory}-f524c755`,
`tsrust-s10-acceptance-{repair,repair-final,format}-f524c755` and
`tsrust-s10-resume-*-f524c755`. Closed records are verified in SQLite; active logs
remain temporary until the corresponding process finishes.
