# Generated native dispatch

Implementation starts from accepted source/evidence checkpoints `f729211` and
`7b1871db`. The HTTP client and SQLite repository pass their focused acceptance;
this section now passes focused Windows native acceptance. Shared compiler/editor blockers are accepted
at `96220939` with fifty focused cases and the actual SDK verifier passing; see
[inference evidence](async-inferred-factories.md). Current generator materialization
rebuild/contract batch `76390` completed with nine passing cases, output
`target/api-native-materialization-focused.log`. Actual native execution rerun
`62237` completed successfully, output `target/api-native-dispatch-execution.log`.

## Accepted Windows evidence

`node tools/verify-native-api.mjs` generates and stale-checks separate public and
private packages, proves private owner edits block replacement without changing
the public marker, checks absence of private policy/source in the client, builds
the native executable and calls it through the real generated HTTP transport.
Current fixture identity is
`b6faac184c48bf84e624c2dffad74144e8e90806ba4a03210c0d0d6287c7cbc1`;
retained v1 identity is
`09a4eb5a9b63f5c13b61484f6f3e45916ecd167c1f9b8fc702a8d8bb54401acc`.

Passed runtime coverage includes named sync/async dispatch, private borrowed
Context state, auth before business execution, strict input/result DTOs,
record/array/tuple/union materialization, optional record/boolean absence and
presence (including false), null/present records, both discriminator branches,
literal `__proto__` data keys, finite numbers beyond 64-bit integer storage,
safe-integer rejection, old-client result projection, incompatible operation
rejection, media/protocol failures, unselected operation rejection, sanitized
private exceptions, typed readiness and authorized orderly shutdown with exit 0.
No server process from this fixture remains live.

The SQLite task service's separate verifier is next. TLS, production signal
handling, cancellation, frontend/package targets, full family regressions and
performance measurements remain pending. Installed compiler/LSP binaries are not
refreshed by this acceptance. No CI, release, publication or analytics import ran.

Earlier diagnostic notes below are retained history and are superseded by this
current acceptance summary, not a queue to repeat failed probes.

## Current materialization repair

The complete disposable handler preview now passes ordinary strict project
checking (`target/api-native-owned-elements-preview.log`). Its changes have been
admitted to the source generator, with actual generated/native execution pending:

- Tuple transforms bind decoded members before generic materialization, and
  tuple/object adapters bind source fields before nested generic calls. This
  supplies checked argument types without casts or weakened return contracts.
- Optional adapters use ordinary if/return branches so their generic target keeps
  the complete optional context. Helpers are deduplicated by child adapter.
- Array iteration borrows elements, so adapters explicitly clone each element
  into a local before reconstructing nominal DTO fields. This has an extra copy
  cost, not a measured performance claim. No consuming iterator/runtime is added.
- Dispatch moves the successfully narrowed decode outcome to a local before
  consuming nested owned input fields. Existing context/resource checks remain.

The real native verifier now covers absent/present optional records and booleans
(including false), null/present nullable records and both nested discriminator
branches. The fixture adds one optional boolean to its private Nested DTO; the
retained v1 public compatibility contract is unchanged. The expanded matrix is
pending execution. Both whole-project formatting passes report zero changes and
zero fingerprint updates after these edits.

The first real native verifier reached a previously uncompiled fixture entry.
Its anonymous async stop callback failed error-contract resolution and its
anonymous readiness JSON could not infer its generic argument. The fixture now
uses a named async stop handler and a named Ready record, retaining auth and
Channel-based orderly drain. Entry checking passes in
`target/api-native-named-stop-check.log`. These are fixture source adaptations;
no catch contract, shared-state guard or compiler check was disabled. The task
fixture separately uses a named helper with a readonly retained sender; its
production entry does not contain a test stop route.

## Dispatcher implementation

- `Selection.context` is opt-in and defaults false. `CheckedApi` keeps one checked
  manifest plus private endpoint/async facts. An explicitly hidden first parameter
  must be a required borrowed native HTTP Context, identified by native type id
  and read/mutate mode. Owned contexts and arbitrary resources reject. The public
  manifest remains independent of private context and source paths. Nine API
  integration cases covering these boundaries pass at the shared compiler checkpoint.
- `api/native_codecs.rs` composes the existing `std:schema` constructors and
  generates ordinary checked generic materialization functions. No new runtime,
  unchecked type assertion or new schema built-in is intended. It is now wired
  into `api/native.rs` and the CLI; generated-source/native acceptance is pending.
- The task example now has a private native Context, operation wrappers and an
  application-owned bearer/origin policy. Its new `server`/`context` API config is
  wired into the in-progress CLI; the installed compiler does not yet
  understand these fields. The earlier repository acceptance remains retained.
- HTTP number serialization now adds a decimal point to unsafe bare integer
  number tokens after host JSON escaping. JavaScript can emit a finite integral
  float outside native JSON's signed/unsigned 64-bit integer storage as a bare
  token (for example 1e20). Floating decimal syntax preserves that number without
  weakening native strict integer parsing. Strings/keys and already decimal or
  exponent tokens stay unchanged. Updated source-runtime loopback cases pass;
  actual generated-package/native finite-number coverage is pending.

The handler emitter enforces an explicit authorize hook, method/MIME/body limits,
declared operation selection, per-operation compatibility, checked decoding and
DTO materialization, output validation/projection and private exception handling.
`compatibility.rs` validates retained public manifests and compares safe input/
output relations; additional integration cases are ready. Old output schemas strip
new fields recursively instead of weakening old clients' strict result codecs.

CLI generation now pins input/config versions through the existing Candidate
overlay, checks formatted private source in memory and preflights/locks both
separate outputs before replacing either. Output hashes are rechecked against
preflight observations. `package_validation::normalize` preserves existing-path
canonicalization and extends virtual path handling to missing ancestor directories.
No frontend manifest, lock, script or dependency is changed by this integration.

`examples/api-contract/tsrust.server.api.json`, its server fixtures and
`tools/verify-native-api.mjs` cover private native Context state, checked nested
records/arrays/tuples/unions, exact special keys, very large finite numbers,
authorization, safe integer output failures, private exceptions and real old-client
projection. `contracts/v1.json` is a versioned public protocol policy fixture,
not a backend-source archive. The fixture runner also checks that owner edits in
private output prevent either output marker from changing, and drains the test
server through an authenticated fixture-only control route.

## Earlier batched verification

- `cargo check --offline --locked -p tsrust-compiler --bin tsrust --test api_contract`
  passes in completed session `43655`, output `target/api-native-metadata-check.log`.
  This is Rust metadata checking, not execution of API tests or generated Tsrust.
- Updated HTTP source-runtime loopback checks pass, including numeric text not
  being rewritten and decimal encoding for integral floats outside safe-integer
  range. The temporary runtime directory was removed. This is not a substitute
  for the pending complete generator/native witness.
- Whole-project formatting passes with a stable second zero-change pass and no
  fingerprint changes. `git diff --check` and both Node verifier syntax checks pass.
- `cargo test --offline --locked -p tsrust-compiler --test api_contract -- --test-threads=1`
  completed in session `90197`: all nine tests pass, build observation 11m31s.
  Output is `target/api-native-contract-focused.log`. One target/job and
  `CARGO_INCREMENTAL=0`; no competing builds.
- The actual complete generated-client/TypeScript/ownership/HTTP verifier passes
  with that binary (`target/api-native-client.log`). Its numeric cases include
  large integral floats and preservation of numeric text inside strings.
- `node tools/verify-native-api.mjs` stops safely during virtual source checking,
  before either private/client fixture output is replaced. No native handler
  acceptance is claimed. Output: `target/api-native-server.log`.

## Earlier repair before shared inference fixes

The first virtual handler fails at `input: _tsrustSchema0()` with
`Builtin object fields must be printable values`. A typed local probe confirms
the inferred factory result is being observed as void in the complete handler.
The same standalone schema factory works in a small exported async handler, so
do not declare opaque factories universally unsupported or weaken checking.

A small disposable Rust consumer linked the existing compiler rlib and emitted
`target/api-generated-preview.tsr`, using target-relative imports. This lets the
current compiler check source-level reductions without another whole compiler
build or writing invalid source over private application output. Consumer source/
binary are `target/preview-native-api.rs` and `.exe`; use the exact normal compiler
serde-json rlib (`libserde_json-e89000ad8322c178.rlib`), not the newest differently
featured native-program copy. A first dependency-feature mismatch was corrected;
there is no dependency-resolution blocker. Remove these disposable consumers once
canonical generator witnesses replace them.

The preview isolated two real emitter mistakes: generic adapters cannot apply
ReadOnly to scalar specializations, and nested anonymous protocol literals do not
provide enough context for JSON.stringify inference. Source now uses plain V
parameters and explicit typed failure/success envelopes. These latest emitter
repairs have not been rebuilt yet. Existing nine contract tests and client passes
remain evidence for the preceding binary, not a claim that repaired native dispatch
passes. Do not rebuild until the remaining factory inference reduction is settled.

Explicit Schema<T> annotations eliminate the initial void result but are not a
general repair: tuple pipelines have a distinct inferred plan type, and old stripped
object plans also have nominal output identity differences. Experiments also
exposed downstream generic V inference. Keep ordinary checked materialization;
never cast or skip validation. The current preview is regenerated source with
plain V and typed envelopes, still failing at the unannotated scalar factory;
it is a disposable current diagnostic, not the canonical generator output.

Next reduce the complete preview to a minimal factory/mode/result-inference
reproducer. If a compiler repair is needed, checkpoint current API work first,
then add a focused compiler/native/editor witness and repair the shared path.
Otherwise fix the emitted source shape using existing contracts. Preserve every
passing prior check and rerun only affected cases, then resume full native fixture.

After the source repair, retain prior results and rerun affected checks, then run
`node tools/verify-api-client.mjs` and `node tools/verify-native-api.mjs`
with normal loopback/junction/private-temp permission. The new native verifier
first generates/checks the complete virtual private adapter, then builds/runs a
real temporary-port native server. Its generated-source check may expose remaining
nominal/generic materialization issues: inspect the source diagnostic and repair
the emitter rather than bypassing checks. Also generate/check the task server
configuration before claiming its policy/Context integration accepted. Installed
compiler and LSP remain unchanged until this section passes.

## Design uncertainties resolved with bounded probes

A temporary tuple-composition witness was formatted, checked and built/executed
with the installed verified compiler, one shared target/job and no incremental
compilation. Session `67060` completed, exit 0; output in
`target/api-tuple-composition-native.log` is `Tuple composition passed` after the
normal build line. The source is `target/api-tuple-composition.tsr` and should be
replaced by a canonical generator-driven fixture during section acceptance,
rather than treated as a retained source snapshot.

The working composition is an array of a union of child schemas; first refine
exact length, then refine each position with the child `safeParseJson` result,
then transform to a typed tuple. Each child conversion remains checked. `min/max`
are not array size methods. A schema's safe-parse `success` discriminant must be
tested in an ordinary local guard; direct call-result field access is unsupported.
Typed union scalar values cannot be passed to a scalar schema's `safeParse`
without projection, so the composition serializes via the existing bounded JSON
implementation and decodes through the child schema. The generated comment tracks
the repeated validation cost; consider direct shared tuple plans only if measured
bridge behavior justifies them. Native tuple/union fallthrough and nested records
still need generator-driven witnesses.

Another source-check probe proved that schema-owned record identities cannot be
passed directly to a named DTO parameter, even with identical fields. Ordinary
generic adapters solve this without casts: `adapt<T,V>(value:ReadOnly<V>):T`
returns a record literal, and the called operation's expected argument infers T.
The ordinary compiler checks the concrete specialization. Literal string computed
field access is supported. Native execution and nested/array/union specialization
coverage are still pending. Tuple transform callback parameters should be inferred
from the actual schema output; explicitly naming a structurally equal generated
record there can introduce the same nominal mismatch.

## Coherent section acceptance checklist

1. Validate the emitted private native handler from `CheckedApi` and generated codecs. Use
   separate private output, never put server imports/bodies in the public package.
   Require an explicit application authorize hook, validate method/MIME/body and
   declared operation selection, decode arguments, call only selected functions,
   validate/project outputs and return bounded structured public failures without
   leaking host/SQL exceptions. Preserve authorization before operation execution.
2. Validate checked compatibility with explicitly retained old manifests. Do not demand
   exact total contract equality for additive rolling upgrades. Old strict clients
   reject extra result keys, so approved compatible responses need projection to
   the old output shape. Check input contravariance and projected output safety;
   reject unsupported/incompatible operations usefully.
3. Validate the separate `server` configuration in CLI generation. Reuse
   `cli/package_validation.rs::Candidate` for virtual checked source and source
   stability; extend virtual path handling deliberately if parent directories do
   not yet exist. Pin configuration and checked source bytes until commit. Protect
   both public/private output ownership and preflight before replacements. Preserve
   editable frontend metadata, scripts, locks and arbitrary normal dependencies.
4. Finish source fixtures and batch affected compiler/client/native checks. Cover
   actual private Context identity/ownership, record/optional/array/tuple/union
   materialization, special keys, safe integers, very large finite numbers,
   structured errors, authorization and compatibility projection. Retain earlier
   passing evidence and repair/rerun affected cases only.
5. Connect the existing SQLite repository to a runnable native service, then the
   TanStack adapter and frontend targets. Keep the full prototype goal intact.

Do not launch another expensive Cargo rebuild for unfinished helper edits. Finish
the coherent section before batch validation, except when a concrete uncertainty
blocks implementation. No public publication, remote validation, desktop control
or broad benchmark/regression milestone has run for this section.
