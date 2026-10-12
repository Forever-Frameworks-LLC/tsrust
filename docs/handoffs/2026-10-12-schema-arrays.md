# Tsrust handoff: schema arrays

## Current state

Repository: `/Users/seancotter/Desktop/dev/learning/tsrust`, branch `main`.
Latest implementation checkpoint: `acda11ee`.
S01–S10 are accepted; S11 schemas are in progress. S12–S21 remain required.
The overall goal is the existing roadmap through integrated S21 acceptance, not
just schemas. Do not restart completed resource/view, JSON or CLI work from old
handoffs. Read current source and milestone evidence before relying on old prose.

The scalar-array section is checkpointed. Public typed object schemas are next.
S11 has focused correctness evidence only: no S11 timing milestone or installed
consumer refresh has been performed. Installed consumers remain at accepted S10;
the shared debug CLI has current S11 work for development checks.

## Mandatory workflow

Read `AGENTS.md`, `docs/development-workflow.md`,
`docs/standard-library-roadmap.md`, `docs/standard-library-coverage.md` and
`docs/plans/schema-plan.md`. Apply the user's caveman skill for concise chat;
write normal prose in source, documentation and commits.

One agent works in this tree. Inspect status and preserve owner changes. Use
focused failing tests, then bounded native/compiler/editor checks. Complete
IntelliSense examples and benchmark support during implementation. Run full
regressions and timings at meaningful feature-family milestones, not every edit.
Do not reopen approved public contracts. Ask only for unresolved material choices.

Before each source checkpoint, format the whole project:

```sh
PATH=/Users/seancotter/.nvm/versions/node/v24.11.1/bin:$PATH \
TSRUST=/private/tmp/tsrust-gate100-editor-target/debug/tsrust \
node tools/format-project.mjs
```

Use `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`
and Cargo `--offline --locked`. Do not create per-gate targets or source archives.
Keep SQLite (`benchmarks/records.sqlite`) canonical: import closed logs using
`benchmarks/storage/store.py`, restore and compare bytes, then delete loose logs.
Preserve original failed evidence. Formatting must preserve tokens and be idempotent;
source fixtures use `support::format_round_trip` and execute formatted source.

## Preserve these unrelated owner files

- `UI_VISION.md`
- `docs/plans/axum-hono-http-ui-contract.md`
- `docs/plans/editable-list-app-example.md`
- `docs/plans/parallel-ui-foundations-plan.md`
- `docs/plans/ui-feasibility-review.md`
- `docs/plans/ui-route-contracts-first-task.md`

These are untracked owner drafts, not cleanup candidates.

## Accepted schema contracts

- `import {s} from 'std:schema'`; inferred output types.
- Builders consume the old immutable schema and return a new plan. Validation
  borrows a reusable plan, but consumes input on success and failure. Explicit
  `clone(input)` retains an independent original; no implicit shared ownership.
- Invalid construction throws `SchemaConstructionError` with
  `limit: int | number`, preserving exact integer payloads. Invalid data returns
  `Result<T, SchemaError>` from `safeParse`.
- String lengths count Unicode code points. Numeric extraction must preserve exact
  integers and reject precision loss. No JavaScript-only numeric scope.
- Objects reject unknown fields by default; explicit strip/retain policies remain
  required. Internal policy prototypes exist, but public object APIs do not.
- Default-factory evaluation/ownership is still unresolved. Ask with concrete
  alternatives before implementing defaults; continue independent work meanwhile.

## Implemented public surface

Scalar string, boolean, int, number and null constructors and safeParse are
integrated. String min/max and integer/number min/max/gt/lt constraints consume
plans and intersect bounds. Numeric constraints reject empty representable ranges.
Number validation checks finiteness and exact integer conversion before bounds.

`s.array(child)` owns its child inline and infers nested `Schema<T[]>` outputs.
Array safeParse accepts matching owned scalar arrays, JsonValue arrays, and int
arrays for number schemas at matching nesting depth. Input is consumed on every
outcome; schema reuse works. Strings move without cloning. Empty arrays succeed.
Nested shape/type failures retain indexed paths. Integer JSON extraction checks
fractional values and signed range; number extraction checks precision.

Integer-to-number array conversion compares through i128 to avoid saturating-cast
acceptance of rounded i64::MAX. Exactly representable large values are allowed.
Direct integer literals receive the same check. Precision errors precede bounds.

Array errors retain at most 64 issues, 32 path segments and 1024 field-name bytes
per path, with explicit truncation. Validation visits remaining input after a
failure, dropping partial successful output. Current output construction can
allocate; optimize only after the family performance evidence is collected.

## Implementation map

- `crates/tsrust-compiler/src/analysis/schema.rs`: array construction, generic
  Result materialization, ownership and input selection; scalar method lowering.
- `crates/tsrust-compiler/src/emit/schema_validation.rs`: recursive generated
  typed/JSON array validators, checked integer conversion, Result assembly.
- `crates/tsrust-compiler/src/stdlib/schema/emit.rs`: scalar validation, constraints
  and conversion of bounded issues into public error records.
- `crates/tsrust-compiler/src/stdlib/schema/runtime.rs`: bounds and shared consuming
  array loop. Generated code and backend tests use the same loop.
- `crates/tsrust-compiler/src/stdlib/schema/issues.rs`: redacted bounded errors.
- `crates/tsrust-compiler/src/stdlib/schema/namespace.rs` and `builders.rs`:
  linked namespace identities, native bindings and throwing builder adapters.
- `crates/tsrust-compiler/src/native/binding_types.rs`: parameterized signature
  shape preparation/resolution; does not itself register generic callables.
- `crates/tsrust-compiler/tests/native_declarations.rs`: source/native acceptance
  and executable catalog examples (includes SchemaArray).
- LSP: `intelligence.rs`, `native_type_intelligence.rs`, `builtins.json`, and
  `tests/project_intelligence.rs`. Array input alternatives are substituted into
  both hover and parameter help. Output types remain the schema's output type.

Internal `stdlib/schema/{object,presence,literal,array,numeric,scalar}.rs` modules
are test-only groundwork. ObjectPlan has consuming typed field products, duplicate
name rejection, reject/strip/retain policies, bounded errors and cleanup witnesses.
Do not describe that backend as a completed public API. Literal-value compiler
and editor types remain prerequisites for precise public literal/enum schemas.

## Verification and evidence

Recent implementation commits:

- `2f62407c`: generated bounded error records.
- `5ad48447`: owned scalar/nested arrays with indexed errors.
- `c1c8c95d`: JSON arrays with exact numeric extraction.
- `acda11ee`: checked integer-array to number-array conversion.

Focused native checks cover every scalar storage family, nested arrays, empty
input, truncation, exact int64 endpoints, fractional/overflow rejection, numeric
precision, bounds precedence, direct literals and consumed-input rejection.
Catalog examples execute. Latest schema editor group: 14 passed. Shared schema
backend group: 47 passed at the owned-array checkpoint. Latest workload contract
group: 13 passed. Do not imply those were full compiler/LSP regressions.

New registered workloads: `native_schema_owned_array`, `native_schema_json_array`,
`native_schema_number_array_int`. Each has a TypeScript counterpart, source hashes,
independent checksum, source guards, ABI/report registration and indexed observers.
Conversion uses BigInt as the exact reference. Four untimed cases per workload
agree between native and Node-after-tsc. These are handwritten references, not Zod.
All schema workloads remain unmeasured.

SQLite log prefixes:

- `tsrust-schema-issue-bridge-*-9f9d4f7d.log`
- `tsrust-schema-array-validation-*-2f62407c.log`
- `tsrust-schema-json-array-*-5ad48447.log`
- `tsrust-schema-array-conversion-*-c1c8c95d.log`

Closed logs were imported, byte-verified and removed. No test/build process is
pending at handoff. Obsolete debug test executables are removed during handoff;
subsequent tests may rebuild. Preserve debug/release CLI and LSP consumers.

## Next work and remaining scope

1. Integrate public `s.object` construction with inferred typed fields and owned
   child plans, then validation and explicit unknown-field policies. Test actual
   source/native behavior, ownership, error paths and disposal. Reuse internal
   groundwork, but avoid another long sequence of test-only prerequisites.
2. Integrate optional/nullable, resolve default factories, then unions and
   discriminated unions. Preserve missing/null distinctions and rollback cleanup.
3. Complete the entire S11 plan: literal/enum types, broader typed input handling,
   fused text decoding (no intermediate dynamic tree or stringify/reparse),
   refinements/transforms/preprocessing, recursive/lazy composition, record/map/set,
   and explicit JSON Schema export limitations. Dependency-gated async/regex/date
   coverage stays on the roadmap; do not silently omit it.
4. At the meaningful S11 milestone, pin the actual Zod reference package, complete
   regressions and planned performance/allocation/RSS/direct-Rust comparisons,
   review regressions, optimize from evidence, and refresh installed consumers.
5. Continue S12–S21 in documented dependency order through release acceptance.

Other approved contracts are recorded in `docs/plans/native-threads-plan.md`,
`async-compiler-plan.md`, `cache-plan.md` and `globals-and-imports-policy.md`.
Threads require explicit join/detach. Rust interop design belongs alongside S12,
including an automatic lazy shared bounded CPU pool (Rayon to evaluate), without
per-build --extern flags. Async tasks are lazy and single-use; `.start()`,
`.cancel()`, `.timeout()` and combinators are approved. Unused lazy tasks warn;
started jobs left unsettled must error. Cancellation does not promise finally.
Caches borrow values and clean expiration during operations. Preserve all existing
ownership guarantees and the broader globals/imports audit requirements.
