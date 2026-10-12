# Borrowed JSON child views: focused acceptance

S10 remains open. JsonView, get and at have focused source acceptance. Existing
read-only observers and serialization accept owners or views. Nested views retain
the original owner lifetime without copying child data. Missing/error branches
release the child loan; present JSON null retains it. Views cannot clone or escape.

Installed consumers were refreshed from 818459f3, including this child-view batch;
see the installed guarded-read acceptance in the S10 plan. No performance timing or full-family acceptance is
claimed. Safe scalar refinement, borrowed string projections, construction,
direct typed paths, public writers and family measurements remain pending.

## Evidence

Canonical records reside in benchmarks/records.sqlite:

- tsrust-json-views-final.log: four public/native tests pass, including nested
  helper returns, ownership rejection, missing/error loan release and all three
  new catalog examples. Initial failing witnesses and repair runs are retained.
- tsrust-json-views-editor.log: project diagnostics, hover, signatures and lifetime
  documentation pass.
- tsrust-json-views-namespace-green.log: eight namespace regressions pass; one
  bridge-order witness required recognizing generic function signatures.
  tsrust-json-views-order.log retains the passing repair. The remaining native
  target resumed in tsrust-json-views-namespace-resume.log with three passes.
- tsrust-json-views-scalars.log: all four existing scalar regressions pass.
- tsrust-json-views-contract.log: six workload source guards, registrations,
  adapters and independent-oracle checks pass.
- tsrust-json-views-smoke.log: five native/Node-after-tsc cases agree, zero timing
  samples. The child workload parses once per invocation and repeatedly performs
  object lookup, array lookup and exact integer extraction in a shared domain.
- tsrust-json-views-inventory.json and tsrust-json-views-audit.log: all 411 public
  identities have exactly one placement review.
- tsrust-json-views-format.log and tsrust-json-views-format-final.log: whole-project
  token-preserving formatting and source fingerprint refresh; final pass stable.
