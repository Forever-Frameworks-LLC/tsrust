# S11 default factory checkpoint

Continue the existing active goal through S21 integrated release acceptance.
S01–S10 remain accepted. S11 is incomplete. One agent works in this tree.
Read AGENTS.md, docs/development-workflow.md, docs/standard-library-roadmap.md,
docs/standard-library-coverage.md and docs/plans/schema-plan.md first.

## Accepted implementation

Public `.default(factory)` consumes the child schema and uniquely owns a reusable,
readonly callback environment. Missing owned/JSON object fields invoke it; null
and invalid supplied values do not. Generated values are validated. Factory
exceptions propagate with their concrete payloads and stop later factories.
Captures cannot be mutated or consumed. Fresh allocations and logging are allowed.
No hidden capture/value cloning or shared ownership is introduced.

Constraints, optional/nullable widening, stripping and retention preserve defaults.
Inferred record-producing callbacks receive contextual output types. Distinct
owned input shapes use distinct validator helpers. Returned runtime factories with
one closure definition work; the language's existing rejection of functions
returning different closure definitions remains. This checkpoint does not expand
that separate callback representation boundary.

The preceding source checkpoint is `42bb0906` (owned readonly callable foundation).
Use the Git commit containing this handoff for the default implementation.

## Approved upcoming chain contracts

The owner approved `.refine(predicate)` preserving the input/output type,
`.transform(callback)` producing the next inferred output type and permitting
mutation or consumption of owned input, and explicit `.coerce` conversion before
subsequent validation. Callback captures remain readonly/reusable. The checker,
hover, completion and signature help must track every stage, intermediate variable
and reusable runtime plan. Reject downstream operations that require an obsolete
input type. Exact coercion arguments/placement remain unresolved. These APIs are
required future S11 work, not implemented defaults.

## Verification and evidence

- 45 affected compiler/native schema tests pass, including 11 default tests and
  executable catalog examples.
- 20 editor schema cases pass across preserved runs: 19 initially passed; the
  remaining test's expected Map whitespace was corrected and its rerun passed.
- 18 benchmark contract tests pass. `native_schema_json_defaults` has matched
  TypeScript, independent oracle, source guards/hashes, ABI and report support.
- Four untimed cases per native/Node-after-tsc engine pass. No new timing.
- Whole-project formatting changed zero files/fingerprints. Native fixtures check
  formatter idempotence and execute formatted source.
- Exact red/repair/green logs are stored and byte-verified in
  `benchmarks/records.sqlite`, named `tsrust-schema-default-*`.
- Installed consumers remain at the accepted S10 checkpoint; no refresh here.

Reuse `/private/tmp/tsrust-gate100-editor-target`, always `CARGO_INCREMENTAL=0`.
The verified CLI is that target's `debug/tsrust`. Node is available at
`/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node`.

## Remaining work

Continue public literal/enum schemas and union/discriminated-union validation,
then the remaining required S11 APIs. Internal backend prototypes do not establish
public support. Preserve type inference, owned partial-output cleanup, bounded
redacted diagnostics, IntelliSense/examples and ready benchmark support as each
family lands. Fused JSON decoding and measured S11 acceptance remain pending.
Continue S12–S21 in dependency order afterward. Do not mark the full goal complete.

Preserve unrelated owner files: `UI_VISION.md` and the plans
`axum-hono-http-ui-contract.md`, `editable-list-app-example.md`,
`parallel-ui-foundations-plan.md`, `ui-feasibility-review.md`, and
`ui-route-contracts-first-task.md`. They remain untracked and outside this work.

## Copyable continuation prompt

```text
Use $caveman. Continue Tsrust in /Users/seancotter/Desktop/dev/learning/tsrust
under the existing goal through S21. Read AGENTS.md and
 docs/handoffs/2026-10-12-schema-defaults.md, then the linked workflow/roadmap/plan.
S11 defaults have focused implementation acceptance; S11 family acceptance and
measurements remain open. Continue literal/enum and union families, then remaining
S11 requirements. Preserve approved refine/transform/coerce type-chain contracts,
owner files, single-agent work, focused TDD and SQLite evidence. Keep checker and
IntelliSense types consistent throughout chains. Do not reopen settled decisions
or count backend prototypes as public APIs. Ask only unresolved material semantics.
```
