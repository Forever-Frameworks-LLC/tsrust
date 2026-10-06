Current continuation: [generic value red checkpoint](handoff-2026-10-06-gate173-generic-red.md).

# Handoff: captured rest function values

Read AGENTS.md, docs/development-workflow.md, docs/nested-function-declarations-plan.md
and docs/milestones/173-nested-function-signatures.md. Inspect Git for this checkpoint
hash. Gate172 is accepted at d74f34d; inferred named results landed at dd43e11 and
captured optional/default values at 7ef5e3e. Gate173 remains incomplete.

Preserve the four owner fixtures without staging or editing them: comma-contexts/guards.tsr,
interfaces/shapes.tsr, map-for-each/values.tsr and map-group-by/documented.tsr. Use
explicit Git paths. One agent in this tree. Hourly checks remain paused.

## Implementation and lessons

local_function_values::Header now records the rest flag. The generated closure's
last parameter retains rest syntax, and its hidden declared call spreads that pack.
Fixed optional/default slots preserve the previous omission forwarding, so the
original declaration alone evaluates defaults. No implicit clone or shared cells.

The initial red witness rejected a captured rest value. Removing the guard and
adding spread exposed a second real defect: ordinary_spread_contract checked the
named body before preparing capture context. Direct non-spread calls already
prepared it; spread lookup now does so too. Isolation controls show this was not
specific to reduce: both reduce and for-of bodies failed as stored values before
the preparation repair. A direct spread witness retains the preparation boundary.

Rest signature help exposed the prior source-span collision in its spread form.
projections recognizes a generated single-return wrapper through CheckedSpreadCall's
checked call, and excludes hidden Call/SpreadCall result facts only at verified
wrapper spans. Do not broadly exclude same-span expressions; earlier numeric,
object-spread and in-operator regressions must remain covered.

A draft array callback assertion was invalid: map supplies value, index and source
array, which cannot fit homogeneous int[] rest. Keep that precise type rejection.
The positive callback witness instead uses a compatible fixed two-int callable
contract. This corrects the test contract; it does not weaken the compiler.

Five rest tests cover empty/ordered/spread packs, reduce and for-of bodies, direct
spread preparation, fixed defaults and undefined, compatible callback adaptation,
independent returned mutable factory state, owned string elements, once captures
and invalid arguments. The guide example shares an executable std:testing fixture.
Existing rest adapters and optional/default values remain covered.

## Matched support and remaining work

nested_function_rest is appended at the end of the manifest as text. Seven
observations cover empty, supplied and spread packs plus omitted/undefined/explicit
fixed prefixes and input length. Matched TypeScript, independent oracle, source
mutation guards, registration, native ABI, run/profile/report hooks and oracle
fingerprints are complete. implementationAccepted remains false until full gate173;
no planned milestone timing or protocol/sample-count changes.

Next implement generic declaration values with genuine polymorphic/contextual
specialization witnesses, preserving source identity, parameter access, captures,
arity and ownership. Header still rejects type parameters. Do not merely remove
that guard or require arrows from users. Audit inferred callable factories, type
alternatives, parameter patterns, abstract callbacks, imported nominal types,
catch/finally scopes, deep nesting and fixed-array caller storage. Then audit the
full plan before meaningful feature-family regressions and timing. Named function
expressions are still separate. Preserve accepted disposal and dead-inner behavior.

## Environment and retention

Reuse CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target with
CARGO_INCREMENTAL=0 and offline locked Cargo. Node:
/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. ReadOnly<T> is borrowing syntax.
Refresh five consumers via .refresh-tmp and atomic rename before checks. SQLite is
canonical: import closed logs/results, verify exact bytes, remove loose copies.
Preserve failed attempts. No archives, binary backups or routine whole compiler
matrix. Prune executable test/native scratch while preserving library caches and
five current consumers; tests may relink.

## Verified checkpoint evidence

Final rest suite passes five native/contract tests; existing rest_parameters passes
19 and captured_function_defaults passes five. All eight canaries pass 63 (68
including the rest suite). The complete editor crate passes 281, including 150
intelligence tests. Source registrations and runner tests pass six. Five native and
Node-after-tsc cases pass, with seven independent observations per iteration.

Canonical untimed record: benchmarks/runtime/results/2026-10-06T21-23-50-710Z-verify.json in
benchmarks/records.sqlite. All 113 compiler fingerprints, oracle input hashes and
the measurement-plan hash match current source. Startup, compilation and execution
sample arrays are empty. Earlier verification and failed checks are preserved too.

I mistakenly used unsupported --verify-only first. The runner silently ignored it
and performed its default timing protocol. The original sampled record is retained
as benchmarks/runtime/results/2026-10-06T21-23-00-466Z.json in SQLite; it is not a
completed family performance review. After a red witness, measurementPlan now
rejects unknown options before execution; use --verify for untimed checks. Existing
sample counts and supported engine/scope options remain unchanged. The final
verification was repeated after that guard to retain current harness fingerprints.

Imported and verified byte-for-byte all 27 original logs/results before removing loose copies.
Pruned 51 obsolete executable test outputs and native/example scratch. Library caches and five consumers remain; tests may relink.
