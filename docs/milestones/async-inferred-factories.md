# Inferred factories after async generic decoding

Status: shared factory/receiver/contextual-union fixes pass 34 focused compiler
and editor cases plus the actual generated SDK verifier. All sixteen affected
recursive-callable cases also pass. The
complete native API handler remains unaccepted.

## Current focused acceptance

- API contracts: nine pass, `target/api-contextual-receiver-repair.log`.
- Generic expression flow: four pass, including native contextual union values,
  argument side effects exactly once and invalid discriminator rejection,
  `target/api-contextual-receiver-acceptance.log`.
- Named inference: five pass, including native direct schema factory outcomes,
  tuple pipelines, retained refinements and resource transfer rejection, same log.
- Editor: new contextual generic result case passes and all fifteen schema
  editing cases pass, `target/api-contextual-editor-acceptance.log`. Completion,
  narrowing, concrete generic signature and inferred factory hover are checked.
- Actual SDK/HTTP verifier passes, `target/api-contextual-sdk-acceptance.log`.
- Recursive-callable repeat: sixteen pass, completed session `4178`, output
  `target/api-contextual-recursive-acceptance.log`.

The native union witness explicitly clones an array's borrowed element before
reading its discriminator. The first version failed at that borrowed access;
an owned-element reduction checked successfully, then the canonical native
witness passed. The new editor fixture also required canonicalizing its virtual
source ID via the existing file-URI conversion before workspace insertion.
These repairs preserve compiler ownership/URI requirements rather than changing
production validation. No full-family benchmark/regression claim is made.

## Reproducer and cause

The generated-handler failure reduces to typed JSON decoding followed by the first
call of an inferred schema factory in an async function. A condition is not
required. The same sequence in a synchronous main function passes, and an async
function without the preceding typed decode also passes.

Canonical regression source is
`crates/tsrust-compiler/tests/fixtures/named-result-inference/async-schema-factory.tsr`.
It is formatted/idempotent. The previous compiled compiler rejects it at
`input: integerRule()` with `Builtin object fields must be printable values`;
`target/api-factory-regression-before.log` retains the concise before result.
This is not evidence that a schema factory has an actual void runtime result.

Callable-result discovery can revisit an async body when a generic is instantiated.
`function_request_ready` previously skipped every unchecked callee during that
discovery. An omitted result still held its initial void placeholder, which then
became an invalid object-field type. Checking a typed local confirmed the same
placeholder. Forcing all factories to return Schema<T> is incorrect: pipelines
have distinct concrete plan types and stripped object identities can differ.

## Candidate change and guards

`analysis/function_scheduling.rs` now checks a still-unresolved inferred factory
when its context is independent of caller-created environments. It reuses the
existing recursive-summary criterion: no unresolved lexical captures/templates,
only required scalar/string parameters, and no active visit or recursive edge.
Other context-dependent body scheduling stays unchanged.

That check uses the ordinary strict ownership path, temporarily disabling both
discovery and effects-only modes, then restores those flags even on error.
Resource errors must not disappear merely because a factory was requested during
discovery. The change does not treat an unchecked omitted result as a real void
contract, weaken schema/data validation, add casts or introduce a new runtime.

The regression includes native execution of a bounded integer plan, a tuple
pipeline and a reusable string refinement retaining its captured owner. A negative
case rejects returning an already closed/moved tracked resource. Existing named
result/recursive callable cases and LSP schema editing cases are selected too.
The new editor witness checks inferred result members and hover after typed async
decoding, using checked project facts.

## First batch and diagnosed repair

Session `62239` completed. API contracts passed all nine cases. Named-result
inference passed three existing cases and failed the two new cases. Cargo stopped
before recursive-callable and editor targets, so neither target has new evidence.
The ownership rejection was correct (`error[transferred]`, with its transfer
location); its assertion incorrectly expected the word `moved` and now checks the
actual diagnostic code and related location text.

The positive case exposed consumed retry source: strict factory checking during
discovery can finish helpers before `run()` saves bodies used for callback/error
contract rechecking. A later retry then sees an empty body and reports that
`@schema:__minInt` must return `Schema<int>`. The candidate now retains eligible
bodies at their first consumption until `run()` collects retry sources. The same
eligibility applies as before (record parameters or exception regions), and the
temporary source map is taken when the normal retry table is built. Native
signatures, ownership checks and return requirements remain unchanged.

## Running repair batch

Session `58322` completed, output `target/api-inferred-factory-repair.log`:

```sh
cargo test --offline --locked -p tsrust-compiler -p tsrust-lsp \
  --test named_result_inference \
  --test recursive_callable_results --test schema_editing -- --test-threads=1
```

All five named-result tests passed (including native execution of the new factory
case and transfer rejection), and all sixteen recursive-callable cases passed.
Editor tests passed eight and failed seven before analysis: their Unix-rooted
`file:///name.tsr` fixtures do not name absolute Windows paths. The fixture now
uses the existing `file_uri(SourceId(...))` conversion on an absolute local path;
production URI validation remains unchanged. The editor rerun completed:
session `72526`, `target/api-inferred-factory-editor.log`,
`cargo test --offline --locked -p tsrust-lsp --test schema_editing -- --test-threads=1`.
Fourteen passed. The new case reaches analysis and exposes a remaining bug:
`rule().safeParseJson(...)` produces an internal Result instead of the public
success/data wrapper when the inferred receiver is not resolved before method
recognition. Do not weaken the test to require a separate schema variable.

The rebuilt CLI passes `tools/verify-api-client.mjs` in
`target/api-inferred-factory-client.log`. Actual private generation now advances
beyond the earlier factory failure but rejects nested generic tuple materializers
with `Cannot infer type parameter V`; see
`target/api-inferred-factory-server-generation.log`. Disposable previews also
expose the same direct schema-receiver bug and nested union result inference.
Replacing materialization with typed JSON decoding is not a valid solution:
typed JSON explicitly rejects multiple-shape unions. Keep the full union contract.
No preview adaptation has been admitted to the production generator.

## Direct receiver candidate after checkpoint 57fdd547

The shared schema-method recognizer now prepares an unchecked inferred named
receiver before reading its return type. It uses ordinary named capture
preparation and scheduling, avoids local shadows/active visits/recursive edges,
and does not evaluate a source receiver twice. Both throwing parse and safe
outcome methods share this recognition. The canonical native fixture now includes
direct stripped-object and literal factories as well as its prior tuple and
captured refinement cases. The existing new editor witness remains unchanged.

Candidate acceptance is running in session `31563`, output
`target/api-inferred-receiver-focused.log`:

```sh
cargo test --offline --locked -p tsrust-compiler -p tsrust-lsp \
  --test api_contract --test named_result_inference --test schema_editing \
  -- --test-threads=1
```

One Cargo job, incremental disabled, shared target/native cache. Poll this live
handle; the sixteen recursive-callable passes remain recorded. Formatting after
the receiver edit changed one fixture and then zero files; fingerprints stayed
unchanged. Installed compiler/LSP binaries remain at their accepted versions.

## Next isolated failure: generic contextual union

The generated nested union materializer failure reduces independently of schema
handling. `target/api-contextual-union-repro.tsr` calls `choose<T>(first:boolean):T`
inside a record field whose type is a two-record union; inference rejects T.
The same source with explicit `choose<A|B>` checks successfully. Before evidence:
`target/api-contextual-union-before.log` and
`target/api-contextual-union-explicit.log`.

`continuations/prelude.rs::prepare_injection_work` removes a multi-shape union
from the child result context before generic call preparation. This prevents an
otherwise ordinary contextual result variable from binding. The next candidate
should prepare generic calls with the complete union context on the existing
heap driver, then perform ordinary conversion from the concrete result. Do not
introduce nested host-recursive preparation, skip final union validation, or
change input-established generic payloads merely to match a result union.

Canonical pending witness:
`tests/fixtures/source-call-expression-order/contextual-union.tsr`, registered in
`generic_call_expression_flow.rs`. It covers record fields, array arguments,
direct returns, input-established nullable identity and argument mutation exactly
once; an additional negative case rejects an absent discriminator. Both formatting
passes complete (one changed file, then zero; zero fingerprint updates).
This witness was added after receiver batch `31563` started and was not selected
by it. That batch completed: nine API cases passed, four named-inference cases
passed, and the expanded factory fixture failed at generic strictEqual on a
literal-typed result. Replacing only that equality call in a disposable reduction
with direct equality checks successfully; the canonical fixture now asserts the
same equality as a boolean through strictEqual. This does not establish generic
strictEqual support for literal-typed arguments.

Warm rerun `28546` completed: all five native/named-inference cases passed. Editor
tests passed fourteen and the new case reached hover with correct completion, but
hover remained `function rule()` without its inferred result. Standalone source
indexing already applies inferred top-level results; checked project indexing
only applied local-function results. The candidate exposes a checked module/name
lookup returning ProjectType, then reuses existing project type spelling and
apply_function_result for unannotated top-level symbols. No linked display-name
parsing or second compiler analysis is used for this result.

The generic driver candidate now prepares a named generic call with its complete
union result context before ordinary injection. A dedicated continuation resumes
ordinary conversion after the callee is concrete. Argument preparation stays on
the existing heap driver; source arguments are not evaluated during preparation.
Inputs still establish generic payloads before result context, and normal union
membership/ownership checks remain in place. A new checked-editor witness tests
narrowed completion and the concrete generic signature.

Session `23159` is live, `target/api-contextual-receiver-focused.log`:

```sh
cargo test --offline --locked -p tsrust-compiler -p tsrust-lsp \
  --test api_contract --test named_result_inference \
  --test generic_call_expression_flow --test schema_editing \
  --test contextual_generic_results -- --test-threads=1
```

This combined candidate is not accepted. One shared target/job, incremental off,
shared native cache. Latest whole-project formatting passed twice with zero files
or fingerprints changed. Poll this handle, then retain passes and repair only
affected failures before returning to actual generated dispatch. The generator
still needs the tuple/local-value materialization repairs proved in disposable
previews; do not infer native dispatch acceptance from compiler regression passes.

Batch `23159` stopped at LSP compilation: hover enrichment attempted to access
SourceIndex's private symbols/helpers from project_intelligence. It ran no tests.
The enrichment now lives in `local_function_intelligence.rs` behind one
pub(crate) method, preserving the private implementation boundary. Rerun session
`20107` is live with the same package/target selection, output
`target/api-contextual-receiver-repair.log`. The compiler source is unchanged
between these two batches; do not start a duplicate rebuild.

One shared Cargo target/job, `CARGO_INCREMENTAL=0`, shared native cache, normal
private-temp permission. Poll the editor handle instead of restarting the passed
compiler tests. Whole-project formatting passed twice
after this repair: zero changed files and zero fingerprint updates, using the
previous accepted installed compiler. The observed single rustc process used
about 1,061 MiB working set during this rebuild. Owner docs source and `.agents/`
are preserved.

After completion retain passes, repair/rerun affected failures, then resume the
actual generated native API fixture. A source-generator adaptation may still be
needed after the inference repair; never infer complete dispatch acceptance from
this focused compiler case. No full regression/performance milestone, remote CI,
publication, installed binary refresh or desktop control is claimed.
