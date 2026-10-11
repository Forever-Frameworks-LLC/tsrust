# S08 collections acceptance

Status: implementation and complete regression acceptance are verified. Family
measurements, performance review and installed-consumer refresh remain pending.
S08 is not yet accepted as a complete roadmap milestone.

## Regression evidence

The complete run at `f414c3ea` observed all 304 compiler/LSP test and doctest
targets from Cargo metadata. It preserved 53 failed tests across six completed
targets and six stack-overflow aborts. All other targets completed successfully.

Focused repairs and reruns now accept 3,119 compiler tests and 511 editor tests,
with three and one intentional ignores respectively. No target remains failed,
aborted or unrun. Completed successful targets were retained rather than restarting
the complete run. The collection suite adds a regression covering 15 invalid
borrowed-query selections. All 40 collection catalog examples execute natively.

The repairs address three causes:

- Borrowed-call detection used for nested Deque projections also made complete
  conditional, logical and non-null selections look like readonly place aliases.
  Restrict that detection to projections and preserve existing ownership diagnostics.
- Borrowed record projections were annotated with their owned representation.
  Infer their borrowed native representation while preserving checked source types.
- Native constructor temporaries enlarged recursive analyzer/emitter frames.
  Extract constructor and collection-hint handling without increasing worker stacks
  or relaxing the original depth guards. All six original depth targets pass.

The general catalog witness runner now recognizes the collection families, and
the dedicated executable runner includes Deque members as well as constructors.

SQLite evidence includes `tsrust-s08-regression-audit-f414c3ea.json`, the complete
original log `tsrust-s08-family-regressions-f414c3ea.log`, and
`tsrust-s08-repaired-regression-audit.json`. Original failures, probes and affected
reruns remain available. The repaired audit maps each target to retained or
replacement evidence and distinguishes compiler targets from similarly named LSP
targets.

## Remaining acceptance

Measure all 21 accumulated workloads marked pending for the collection family,
including earlier Map/Set and iterable workloads. Exact source and counterpart
fingerprints are verified in `tsrust-s08-measurement-plan-f414c3ea.json` in SQLite.
Keep the original protocol: eight warmups, 31 samples, three processes, 11 startup
samples and three compilation samples. Review allocation and comparable performance
signals before justified optimization. Refresh all installed consumers and verify
editor diagnostics afterward. Then begin the requested project-wide diagnostics
milestone before S09.
