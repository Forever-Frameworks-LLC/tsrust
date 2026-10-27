# Inferred factories after async generic decoding

Status: compiler repair from `d3f5854f` passes named-inference and recursive-callable
acceptance; editor fixture portability is being repaired. The complete native API
handler remains unaccepted.

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
