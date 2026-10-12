# S11 schema validation

Status: implementation started after [S10 acceptance](../milestones/s10-json-acceptance.md).
Public scalar schemas, constraints, nested arrays and required-field object schemas
have focused compiler, native and editor verification. Composite schemas accept
matching owned input or JsonValue. Reusable default factories are integrated.
Exact literals, string-list enums and declared enum schemas preserve inferred types.
Readonly refinements, owned transforms and explicit coercion pipelines have focused
compiler/native/editor acceptance, including nested union rollback and defaults.
Fused JSON text decoding has focused public/compiler/editor acceptance.
Remaining composite APIs and family acceptance remain pending.

## Required surface and coverage

The roadmap is the scope. Zod is a differential behavior reference, never a runtime
dependency or blanket compatibility target. Pin the actual reference package before
executing differential cases; current web documentation alone is not a version pin.

| Family | Required behavior | Status |
| --- | --- | --- |
| Scalar constructors | string, number, int, boolean, null; literal and enum checks | String/boolean/int/number/null, exact literals, string-list enums and declared string/numeric enums checked |
| Scalar constraints | explicit string-length units; numeric inclusive/exclusive bounds; no implicit coercion | String min/max and integer/number min/max/gt/lt integrated |
| Objects | inferred typed fields, explicit unknown-key policies, required/missing distinction | Required typed fields, inferred output, named output annotations, nested objects/arrays and default rejection integrated; explicit strip/retain integrated |
| Arrays | inferred element output, indexed error paths, bounded partial construction | Scalar/nested/object arrays integrated for matching owned and JSON input, plus checked int-to-number arrays; other composite elements pending |
| Optional/nullable/default | distinct missing/null behavior, documented default evaluation and ownership | Optional/nullable and reusable default factories integrated for owned/JSON object fields; S11 family acceptance pending |
| Unions | ordered matching, bounded branch errors, correct rollback and output ownership | Public ordered plans, borrowed preparation, single commitment, defaults/presence and common-field access checked; family timing pending |
| Discriminated unions | checked discriminator selection, inferred alternatives and useful errors | Public literal/enum selection, exact integers, mixed field types and branch narrowing checked; family timing pending |
| safeParse | typed Result, reusable immutable schema receiver, bounded redacted errors | Approved: consume owned input on success and failure |
| Refinements and transforms | explicit input/output types, validation order, callback ownership and failure cleanup | Public readonly refine and owned transform pipelines have focused acceptance; family measurements pending |
| Preprocessing and cross-field checks | declared order, explicit coercion, structured issue paths | Callback-based coerce and object predicates have focused acceptance; richer cross-field issue APIs remain required |
| Recursive/lazy and composition | finite plan construction, guarded recursion, no per-request plan rebuilding | Second tranche, still required |
| Record/map/set schemas | collection ownership, key semantics, inferred output and bounded errors | Public record/map/set constructors, reversible validation, collision rejection, editor types and matched workload checked; family acceptance pending |
| JSON Schema export | explicitly reject unrepresentable checks/transforms rather than silently omit them | Second tranche, still required |
| Async refinements | approved task semantics and cancellation/ownership checks | Follows S14 |
| Regex/date/format validators | corresponding verified runtime semantics | Follow their dependency gates |

Do not substitute dynamic JsonValue output for inferred typed output. JSON text
validation must construct typed output directly through a fused decoder, without
an intermediate dynamic tree or stringify/reparse. Ordinary in-memory validation
and text decoding need separate benchmark scopes. Reusable runtime schema values
must work alongside compiler-specialized literal plans, with explicit ownership.

## Runtime validation and JSON decoding errors

The owner approved separating rule definitions from runtime data validation.
`std:schema` and `Schema<T>` define reusable rules. Invalid rule construction
continues to throw `SchemaConstructionError`. Checking runtime input through
`safeParse` returns `Result<T, ValidationError>`; the former in-progress S11 name
`SchemaError` is replaced before family release. Issue fields and ownership stay
unchanged. Earlier checkpoint descriptions below retain their historical names.

The approved fused text API is `schema.safeParseJson(text)`. Its failure result
separates `{ kind: 'json', error: JsonError }` for syntax/resource failures from
`{ kind: 'validation', error: ValidationError }` for data that violates the rules.
Callback exceptions propagate. Typed JSON imports continue to report compile-time
diagnostics. The public runtime rename is checked across scalar/pipeline results,
explicit type imports and editor aliases; `SchemaConstructionError` is unchanged. The fused decoder checkpoint below records implementation and focused acceptance;
S11 family acceptance and timing remain pending.

## Approved contracts

The owner approved both contracts on 2026-10-12:

1. Object schemas reject unknown fields by default. Explicit stripping and
   retention policies remain available; no silent field removal in the default mode.
2. Validation of an existing owned value consumes the input on both success and
   failure. Move reusable storage into successful output. Callers must explicitly
   use `clone(input)` when they need to retain the original. Decoding borrowed JSON
   text is a separate operation and does not consume the source string.

Acceptance must verify default rejection, explicit alternate policies, nested
unknown-field paths, successful storage transfer, failure cleanup, and checker/LSP
rejection of input reuse after either outcome. Signatures, hover and examples must
state the consuming contract. No implicit clone or shared mutable output is allowed.

The owner also approved `import { s } from "std:schema"`, constructors such as
`s.string()` and `s.object({ name: s.string() })`, and `schema.safeParse(input)`
with inferred output types. String-length constraints count Unicode code points:
`😀` has length one; `e` followed by a combining acute accent has length two.
These are not grapheme-cluster counts. UTF-8 byte and UTF-16 unit constraints, if
exposed, must use explicit alternatives. Default factories use the approved deferred, reusable factory contract
described below; their focused implementation gate is complete. The existing checked
JSON numeric extraction must not be bypassed by schema validation.

### Approved schema builder ownership and construction errors

The owner approved consuming schema-building methods such as .min(), .max() and
.optional(). Each takes ownership of the original schema and returns a new immutable
schema; using the original afterward must fail ownership checking. Validation
continues to borrow the schema, so repeated safeParse calls remain valid. Do not
insert hidden plan clones or shared owners when builders contain nested schemas,
callbacks or other owned data.

Invalid runtime constraints, including negative lengths and contradictory bounds,
throw a structured schema-construction error immediately. Constraint builders do
not return Result. Input validation through safeParse continues to return Result;
construction errors and input-validation errors are distinct contracts. Define
and document the concrete construction-error fields with source/checker/editor
and native catch coverage when exposing these methods.

## Implementation and acceptance sequence

1. Establish bounded issue collection independently of object policy or ownership:
   stable internal codes, property/index path distinction, controlled type summaries,
   issue count and per-path byte/segment limits, explicit truncation and validity
   independent of whether an issue could be retained. Errors must not retain input
   values. Public defaults and code names remain subject to the surface gate.
2. Implement checked scalar schema construction and reusable validation, together
   with inferred types, native emission, diagnostic rejection and complete catalog
   examples. Freeze plans once; do not build the schema again on each parse.
3. Add objects/arrays, literals/enums, optional/nullable/default fields and unions
   as a coherent first tranche, preserving exact ownership and failure cleanup.
4. Add fused text decoding and the required second tranche. Document differences
   method by method, including unsupported features and deferred dependencies.
5. Keep matched TypeScript/Zod counterparts, independent oracles, source hashes,
   allocation/disposal probes and benchmark adapters ready as each batch lands.
   At the family milestone, measure complete outputs and error-heavy cases, compare
   direct Rust paths, review regressions, then optimize against retained budgets.
6. Complete compiler/LSP regressions, executed IntelliSense examples, installed
   consumers and final contract audit. Internal backend tests do not accept S11.

## Reference review

The [Zod API reference](https://zod.dev/api) documents default unknown-key stripping
and explicit strict objects. Its [basic usage](https://zod.dev/basics) documents
independent cloned parse output and inferred schema types. These are comparison
points for explicit native ownership decisions, not defaults inherited silently.

## Internal issue collector checkpoint

Five focused backend tests pass: zero issue budget still records invalidity,
bounded issue count preserves order and reports omission, property names remain
distinct from indices, UTF-8 path truncation respects byte/segment limits, and
issues retain controlled type summaries rather than input values. The collector
is test-only backend groundwork until validators and native emission use it.
No public defaults or schema parsing behavior is accepted by this checkpoint.

SQLite evidence: `tsrust-schema-issues-{red,check,format}-32db45b4.log`. The red
witness fails because the issue backend is absent; the passing run executes only
the five schema tests, not the full compiler suite. No timing run was performed.

## Internal numeric validation checkpoint

Reusable integer/number plans now validate JSON scalar input through the accepted
checked extraction functions. Integer comparisons remain i64 operations; no
floating conversion can weaken an exact bound. The number path retains precision
rejection and negative-zero bits. Inclusive/exclusive endpoints are explicit;
reversed bounds, equal exclusive endpoints and nonfinite number bounds are rejected
by the private plan constructor. Public builder behavior is not yet frozen.

Six focused numeric cases plus the five issue cases pass. They cover exact i64
boundaries, unsigned overflow, fractions, wrong types, lossy integer-to-number
conversion, negative zero, repeated plan reuse, range edges, invalid construction
and zero retained-error budget. Plans keep their representation private so callers
cannot bypass constructor checks. Errors retain only controlled type summaries.

This backend reads an existing JsonValue for scalar validation. It does not yet implement
the approved consuming public contract or inferred source schemas, or substitute
dynamic-tree parsing for the required fused decoder. SQLite evidence uses
`tsrust-schema-numeric-{red,check,final,format}-65abb4b2.log`. No measurement or public
API acceptance is claimed.

## Consuming scalar checkpoint

Internal string, boolean and null validators now have statically typed outputs and
consume an optional owned JsonValue. Absence is distinct from present null. String
success moves the original allocation; failures retain only bounded type summaries.
The immutable validator can be reused. This is internal groundwork, not the public
safeParse API or its source-level ownership acceptance.

Four focused cases prove string pointer/capacity preservation, missing/null
separation, secret-free mismatch errors, repeated boolean validation, and rejection
with a zero issue budget. All 15 current schema backend tests pass. SQLite evidence:
`tsrust-schema-scalar-{red,check,format}-2af9dc47.log`. No timings were run.

Public integration must add an identity-bearing namespace export for the approved
`s` builder: native exports currently distinguish only types and
functions (`native/exports.rs`), and module linking handles those two identities
(`project/link.rs`). Do not implement the namespace as an unscoped global rewrite;
import aliases and lexical shadowing must remain correct. Public builder spelling
and default Unicode code-point length units are approved. Integration must verify
aliased imports, lexical shadowing, inferred outputs and consuming ownership.

## Consuming array checkpoint

Generic internal array plans compose typed validators, consume input elements, and
produce Vec of the element output type. Original indices are preserved in nested
issue paths. Successful string elements retain their allocations. On the first
invalid element, partial output is dropped; later elements still validate for
bounded issue collection, but successful values are immediately dropped. Empty
arrays succeed; missing and null inputs fail independently of retained-error budget.

Four focused cases cover allocation/order preservation, nested indexed errors with
truncation, empty/missing/null distinctions, and exact-once disposal of successful
outputs on both sides of a failing element. All 19 schema backend cases pass.
SQLite evidence: `tsrust-schema-array-{red,check,format}-c641b6d9.log`.

This remains internal groundwork. Public source inference, schema recursion and
collection limits, native emission, IntelliSense and matched benchmarks remain
required. No public array schema or S11 acceptance is claimed.

## Namespace import checkpoint

`std:schema` now registers `s` as a distinct native namespace export. Module linking
preserves this identity through aliases, while existing lexical parameter shadowing
continues to work. Namespace-only imports select no native runtime feature. Tolerant
module discovery recognizes the module before callable builders are registered.

All 21 native declaration checks pass, including two namespace cases. The two new
cases also pass after the source fixture uses token-preserving, idempotent formatting.
SQLite evidence: `tsrust-schema-import-{red,check,formatted,format}-169f6300.log`.
This checkpoint supports imports only: constructor dispatch, inferred schema output,
validation methods and complete editor documentation remain unimplemented. Installed
consumers have not been refreshed for this incomplete public feature.

## First constructor integration checkpoint

`s.string()` now resolves through the linked namespace identity and returns an
opaque `Schema<string>`. Imported aliases work. Assigning the result to Schema<int>
and passing constructor arguments reject. The private native binding is not a
source export. The constructor emits a constraint-free, stateless native schema;
this is not a completed validation implementation.

All 23 native declaration checks pass, including a generated native execution of
an explicitly typed function returning the inferred constructor result. All 19
schema backend tests also pass. Constructor catalog prose and its executable
example are present, but namespace member completion/hover/signature routing is
still pending. The example's exact source/output is covered by the native test.

SQLite evidence: `tsrust-schema-builder-{red,check,native,backend,format}-e48749f0.log`.
Next required work: editor routing and public consuming safeParse with structured
errors, followed by constrained and composite plans. Do not refresh installed
consumers or call this a completed public feature gate yet. Matched benchmark
registration and public API audit are still required before feature acceptance.

## Constructor editor routing checkpoint

Imported schema namespaces now use identity-based member completion, hover and
signature help. Aliased calls show the local spelling, and inferred constructor
variables show Schema<string> without internal binding/namespace names. A parameter
shadowing the namespace does not receive schema completions. Output type mapping
materializes the native Schema identity even without an explicit type import.

One focused project protocol test covers these behaviors. The first run exposed
missing namespace routing; a stronger follow-up exposed absent inferred-value hover.
Both failures and passing repairs are retained. SQLite evidence:
`tsrust-schema-editor-{red,check,detail,final,format,final-format}-e10898d4.log`.
Full editor regression, public safeParse, constraints, benchmark support and consumer
refresh remain pending; this is not S11 acceptance.

## String safeParse integration checkpoint

String schemas now expose safeParse for owned JsonValue input. It borrows the
schema, consumes input on both outcomes, moves successful string storage and
returns Result<string, SchemaError>. Non-string input produces one root issue
with type-mismatch, controlled expected/observed names, an empty path and false
truncation flags. The source checker rejects reuse of consumed input. This initial
method does not yet validate ordinary typed records or other schema output types.

Public error declarations are SchemaError (issues, truncated), SchemaIssue (code,
path, pathTruncated, expected, observed), and SchemaPath (optional field/index).
A path segment uses exactly one of field or index; root errors have no segments.
The current native scalar bridge constructs one bounded issue directly; composite
validators must connect the bounded collector and enforce its limits when landed.

Native execution verifies successful output, schema reuse, mismatch details and
failure-path input reuse rejection. All 24 native declaration tests pass. Two
schema project editor tests verify constructor and validation hover/inference;
one focused error-effect registry test passes. The error-effect declaration and
native array constructor repairs retain their original failed evidence. Catalog
entries include ownership, error fields, limitations and a safeParse example;
exact catalog-example execution and matched benchmark support remain pending.

SQLite evidence: `tsrust-schema-parse-{red,check,repair,native,final,effects,format}-3ca7ebd4.log`
and `tsrust-schema-parse-editor-{red,check,final}-3ca7ebd4.log`.
No family timing, consumer refresh or S11 acceptance is claimed. Extend public
scalar coverage, ownership edge cases, constraints and composites next.

## String validation boundary repair

An unrelated record method named safeParse previously caused constructor-only
programs to acquire unused JSON storage/runtime emission. Preparation now requires
an existing JsonValue identity as well as a schema identity. A string-schema call
with unsupported input and no prepared JSON signature reports schema-input instead
of reaching native signature resolution with missing metadata. Ordinary typed-value
validation is still required in the first tranche; this diagnostic does not narrow
that roadmap requirement to JsonValue forever.

All 27 native declaration tests pass. New checks cover no JSON runtime for an
unrelated callback method, exact native execution of both schema catalog examples,
input reuse rejection for six JSON kinds, explicit clone retention, and unsupported
input diagnostics. This closes the previous exact-example execution gap. SQLite:
`tsrust-schema-link-{red,check,format}-cdf5d6f4.log`. No timing or family acceptance.

## String validation benchmark preparation

Registered native_schema_string with a separately authored TypeScript counterpart,
source fingerprints, source/operation guards, integer ABI support, independent
checksum oracle, report category and an untimed verifier. Each iteration parses
and validates one string and one number, observes successful text length, and
checks every exposed mismatch issue field. Two operations are recorded per
iteration. Schema construction is outside the loop. This is parse-plus-validation,
not validation-only or fused decoding. The handwritten Node contract is explicitly
not a Zod measurement; pinned Zod differential coverage remains required.

The verifier uses the production benchmark adapters and offline dependency linker,
compiles TypeScript before Node execution, and checks zero timing samples. Native
and Node agree for (n, seed) = (0, 0), (1, 0), (17, 7), (2000, 1000000).
The run exposed JSON dependency detection assuming a module began a line. Detection
now also handles adjacent generated declarations, with focused regression coverage.
Four benchmark tests pass; the separate legacy dependency integration test is
skipped without TSRUST, while the new verifier directly exercises dependency linking.

SQLite evidence: `tsrust-schema-workload-{red,check,format,final,verify,verify-detail,tests,verify-final,final-format}-e04a2818.log`.
The manifest remains unmeasured. Validation-only workloads, allocation/disposal
probes, fused decoding and the rest of S11 remain required.

## Direct owned-string validation

String schemas now accept concrete owned string input as well as JsonValue.
The direct path transfers the native string into Ok without JSON conversion,
parsing or a hidden clone, and emits no JSON runtime dependency. The schema stays
borrowed and reusable. Input reuse rejects; explicit clone preserves a separate
original. A borrowed ReadOnly<string> parameter cannot be consumed by validation.
Mixed union inputs and other ordinary typed values remain unsupported at this
checkpoint and are documented as pending, not excluded from S11 scope.

Ten focused schema compiler/native cases and two editor protocol cases pass.
The exact catalog example for owned text executes natively. SQLite evidence:
`tsrust-schema-owned-{red,check,final,editor,format}-39a5cff0.log`.
The existing registered workload still measures JSON parse-plus-validation;
a separate validation-only ownership/setup protocol remains required. No timing,
consumer refresh or S11 acceptance is claimed.

## Boolean schema integration

s.boolean() now infers Schema<boolean>. safeParse accepts a concrete boolean or an
owned JsonValue, preserving false as successful output and rejecting non-boolean
JSON without truthiness coercion. Direct booleans retain primitive copy semantics;
JSON inputs transfer ownership. The schema is borrowed and reusable. Scalar native
emission is now shared by string/boolean bindings instead of assuming a string
constructor exists in every schema program. Other typed inputs/unions remain pending.

Eleven focused schema compiler/native tests, three editor protocol tests and two
benchmark contract tests pass. Catalog examples execute exact source natively.
The new native_schema_boolean workload includes matched TypeScript, source hashes,
operation guards, oracle and ABI/report registration. Native and Node-after-tsc
agree on four input sizes each with zero timing samples. It measures parse plus
validation and error observation; it is not validation-only or a Zod comparison.

SQLite evidence: `tsrust-schema-boolean-{red,check,final,editor,format,benchmark-tests,verify}-621ea809.log`.
Both registered schema workloads remain unmeasured. Constraints, numeric/null
constructors, composite schemas, broader typed inputs and S11 acceptance remain.

## Consuming typed numeric backend

Integer and number plans now implement the consuming typed validator contract used
by arrays. Integer output stays i64 throughout; number output uses checked JSON
extraction, rejects precision loss, and preserves negative zero. Missing input is
reported separately from null. Nested failures preserve array indices and bounded
issue collection cannot turn failed validation into success.

Focused internal tests cover exact i64 endpoints in typed arrays, negative zero,
fractional numbers, precision rejection with original paths, missing/null and
zero issue budgets. These backend adapters remain test-only groundwork; public
numeric constructors, generated runtime integration and matched numeric workloads
are still required. No public API, timing or consumer refresh is claimed here.
Evidence: `tsrust-schema-numeric-owned-{red,check,format}-00878231.log` in SQLite.

## Integer schema integration and range correction

s.int() now infers Schema<int>. safeParse copies concrete int input or consumes
JsonValue and returns Result<int, SchemaError>. Exact signed 64-bit integer JSON
values remain exact, including both endpoints. Fractional and range failures retain
checked extraction codes; nonnumeric JSON reports type-mismatch. Constructor,
method hover, inferred results, signature help and executable examples are covered.

The owner's full-range requirement exposed an existing JSON parser defect:
-9223372036854775809 became a floating value rounded to i64::MIN. Dynamic and typed
JSON decoding now inspect integer tokens after syntax validation and reject tokens
outside exact i64/u64 JSON storage. Strings containing those digits are unchanged.
Raw validated text remains unrestricted by numeric storage because it is not decoded.
Decimal/exponent forms retain the existing floating-point contract; this change is
not arbitrary-precision integer support. The added linear token scan requires a
performance review with the affected JSON workloads at the next family milestone.

Twelve focused schema compiler/native cases, four editor cases, thirteen scalar JSON
backend cases and seven typed JSON cases pass. The independent BigInt oracle checks
13 signed/unsigned boundary and beyond-JavaScript-safe-integer cases. The new matched
native_schema_int workload checks the small-number domain (7 and 1.5), all exposed
error fields, guarded source hashes and the existing native ABI. Four sizes agree
across native and Node-after-tsc, with zero timing samples. Three workload contract
tests pass. The benchmark's narrow domain is not evidence of full-range Node JSON
parity; the separate BigInt check supplies that boundary oracle.

SQLite evidence uses `tsrust-schema-int-*-6c2dd78c.log`, including the original
incorrect assertion import, corrected builder rejection, range defect, focused
repairs, formatting and untimed verification. S11 remains incomplete; number/null,
constraints, composites and fused typed decoding are still required. Consumers
remain at the accepted S10 checkpoint. No S11 timing or family acceptance is claimed.

## Unicode string-length backend

The reusable consuming string validator now supports inclusive minimum and maximum
Unicode code-point counts. It moves successful String storage unchanged, rejects
reversed limits during plan construction, and stops scanning once a maximum is
exceeded. It neither normalizes Unicode nor allocates a character buffer. Emoji,
combining sequences and joined emoji distinguish code points from bytes, UTF-16
units and grapheme clusters. Array composition keeps indexed issue paths; bounded
or suppressed error retention cannot accept invalid input.

Focused tests cover Unicode units, inclusive limits, unchanged pointer/capacity,
missing/null, zero issue budgets and redacted nested errors. This is internal
backend groundwork only. Public constraint methods still require plan storage,
compiler/LSP integration, executable catalog examples and matched workloads before
acceptance. Evidence: `tsrust-schema-length-{red,check,format}-3b00c899.log` in SQLite.
No timing, consumer update or S11 family acceptance is claimed.

## Number schema integration

s.number() now infers Schema<number>. safeParse accepts concrete number or owned
JsonValue, preserves negative zero and returns Result<number, SchemaError>.
NaN and infinities return number-nonfinite. JSON integers that cannot convert
exactly to floating point return number-precision; other JSON kinds return
type-mismatch. Direct numbers retain copy semantics and introduce no JSON runtime.
JSON values are consumed; the immutable schema is borrowed and reusable. Concrete
int and mixed union inputs remain pending broader typed-value validation; s.int()
is the current exact integer validator. This is not permission to implicitly
round integers into floating-point output.

Thirteen focused schema compiler/native cases and five editor cases pass, including
exact catalog example execution, inferred outputs and signature help. Four benchmark
contract tests cover registrations and source/operation guards. Native and
Node-after-tsc agree on four sizes each for native_schema_number, without timing.
That workload's matched domain is 1.5 success and true rejection; native precision
and nonfinite boundaries have separate correctness coverage. The handwritten Node
reference is not a Zod comparison. Public constraints, null/composites, fused typed
decoding and S11 family acceptance remain pending.

SQLite evidence: `tsrust-schema-number-{red,check,editor,format,benchmark-tests,verify}-5fc70ac9.log`.
No consumer refresh or measured acceptance is claimed.

## Direct integer input to number schemas

s.number().safeParse now accepts concrete int values through checked conversion.
It returns number-precision when conversion would round; comparing the converted
value in i128 avoids falsely accepting i64::MAX via a saturating cast. Primitive
input remains reusable. No JSON allocation, parsing or dependency is introduced.
The schema's floating result preserves exactly representable integers beyond
JavaScript's safe-integer interval; s.int() remains the exact integer-output path.
Mixed union and unrelated typed inputs still require subsequent integration.

Fourteen focused compiler/native schema tests and five editor cases pass. Tests
cover i64::MIN, i64::MAX, positive/negative odd values beyond 2^53, exact even values,
root issue fields and exact catalog execution. The validation-only workload
native_schema_number_int uses a TypeScript BigInt reference, with 42 success and
9007199254740993 precision rejection. Five workload contract tests pass; four sizes
agree across native and Node-after-tsc with no timing samples. This workload is
separate from existing parse-plus-validation measurements and introduces no JSON
setup cost. Allocation/RSS and timed family evidence remain pending.

SQLite: `tsrust-schema-number-int-{red,check,editor,format,benchmark,verify}-ed9bc998.log`.
S11 and installed-consumer refresh remain pending.

## Null schema integration

s.null() now infers Schema<null>. safeParse accepts concrete null or consumes a
JsonValue, returning Result<null, SchemaError>. Only null succeeds; other JSON
kinds produce type-mismatch with expected null. Undefined and missing arguments
remain distinct and reject during checking. Native signature metadata now carries
null explicitly, preserving its identity in generic results and editor help.
Direct null validation copies the scalar and adds no JSON runtime dependency.

Fifteen focused compiler/native schema tests and six editor cases pass. Exact
catalog examples execute natively, including generic Schema<null> return values.
The matched native_schema_null workload verifies success, every mismatch field,
source fingerprints, operation counts, oracle and ABI/report registration. Six
benchmark contract tests pass; four sizes agree across native and Node-after-tsc
with zero timing samples. No S11 acceptance or installed-consumer refresh is claimed.
Constraints, literals/enums, composites and fused typed decoding remain pending.

SQLite: `tsrust-schema-null-{red,check,editor,format,benchmark,verify}-2ae15d84.log`.

## Optional and nullable backend composition

Internal OptionalPlan and NullablePlan preserve missing and explicit null as
distinct states. Optional accepts absence and delegates present values; nullable
accepts explicit null and delegates missing input. Composition retains the
presence layers in typed output without cloning successful string storage.
Indexed array failures and suppressed issue budgets remain failures. Public
methods and generated composite types are still pending; this is not source-level
acceptance of optional/nullable schemas.

Four focused presence cases join the internal schema suite. Evidence:
`tsrust-schema-presence-{red,check,format}-b3bd707a.log` in SQLite. No timing or
consumer refresh is claimed.

## Generated string-schema plan storage

Generated Schema<string> records now own native StringBounds storage. Constructors
initialize unconstrained bounds; both owned-string and JsonValue validation read
that same stored plan before producing typed output. Other scalar schemas retain
their existing representation. Runtime construction checks use signed limits,
reject negatives and contradictions with structured internal errors, and combine
repeated constraints by retaining the stricter bound. No shared owner, clone or
per-validation plan reconstruction is introduced.

The focused compiler selection passes 35 internal schema-related tests and 15
source/native tests. A separate generated-code witness injects a constrained plan
into the emitted constructor and executes too-small, valid multibyte and too-large
cases. That injection is backend evidence only: it does not accept public .min()
or .max() syntax. The representative string benchmark retains untimed native/Node
parity. Public builder lowering, source construction-error identity and catches,
editor documentation and constraint workloads remain required before acceptance.

SQLite: `tsrust-schema-storage-{red,native,check,generated,format,verify}-c370d757.log`.
No new timing or consumer refresh is claimed.

## Public consuming string bounds

Schema<string>.min(limit) and .max(limit) now consume their receiver and return
Schema<string>. Limits are inclusive Unicode code-point counts. Repeated limits
retain the stricter constraint. Negative and contradictory limits throw the
exported SchemaConstructionError { code, operation, limit }; ordinary checked
wrapper functions unwrap backend Results and preserve normal catch/effect handling.
Invalid input still returns SchemaError through safeParse. Source reuse after a
builder rejects with transferred. Chaining retains Schema<string> inference in
both compiler and editor. Construction does not rebuild plans during validation.

Focused checks cover native Unicode success, JSON too-large rejection, caught
negative and contradictory construction, original-schema reuse rejection, exact
catalog examples, hover, parameter help and chained result types. The matched
native_schema_string_bounds workload constructs min 2/max 6 once, validates hello
and rejects x, then observes every result/error field. Source hashes, oracle,
operation guards, ABI and report registration are ready. Four sizes agree across
native and Node-after-tsc without timing. This is a handwritten counterpart, not
the still-required pinned Zod differential reference.

SQLite evidence uses `tsrust-schema-builders-*-74ac126d.log`, preserving initial
builder rejection, chaining inference repair, signature-test correction and final
focused acceptance. Numeric constraints, optional/nullable source integration,
composites and fused typed decoding remain required. No S11 measured acceptance or
installed-consumer refresh is claimed.

Final focused counts: 35 internal schema-related cases, 17 compiler/native cases,
7 editor cases and 7 benchmark-contract cases pass. Whole-project formatting is
unchanged and all eight native/Node workload executions are untimed.

## Exact integer minimum and maximum

Schema<int>.min(limit) and .max(limit) now consume and return an integer schema.
Every signed 64-bit bound, including negative limits and both endpoints, is valid
unless it contradicts an existing bound. Comparisons stay i64 throughout. Repeated
bounds retain the stricter value. Contradictions throw SchemaConstructionError;
validation violations return too-small/too-large. JSON-origin failures retain
observed number while direct int failures report observed integer. The generated
plan and throwing adapter are shared with the string constraint mechanism without
coercing either limit type or input values.

Eighteen focused schema compiler/native cases pass, followed by the affected
integer case after preserving JSON observed-kind reporting. Eight editor cases
and eight benchmark contract cases pass. The new native_schema_int_bounds workload
uses 7 success and 10 too-large under min 2/max 9, observes every exposed issue field,
and has matched TypeScript, oracle, hashes, guards and ABI/report registration.
Four sizes agree on native and Node-after-tsc without timing. Its small matched
domain does not replace native tests at i64::MIN and i64::MAX. Exact catalog examples
execute natively. Exclusive integer bounds, floating constraints and composites
remain pending; S11 is not accepted and consumers have not been refreshed.

SQLite: `tsrust-schema-int-bounds-{red,check,final,format,benchmark,editor,verify}-a1736712.log`.

## Exclusive integer construction boundary repair

The private numeric plan now rejects exclusive integer intervals with no
representable signed 64-bit value. This includes a lower bound above i64::MAX,
an upper bound below i64::MIN, and adjacent exclusive endpoints. Checked integer
arithmetic determines whether an interval is empty without floating conversion.
Validation retains the original endpoints and their too-small/too-large errors.

A focused failing witness covers both overflow endpoints and adjacent intervals.
Single-value intervals cover both signed endpoints and values beyond JavaScript's
safe-integer range. This repairs internal groundwork only; public exclusive
builders, editor documentation and matched workloads remain pending.
SQLite evidence: `tsrust-schema-exclusive-{red,check,format}-fda48454.log`.

## Consuming exclusive integer runtime

The generated-program IntBounds backend now supports consuming gt/lt constraints.
It converts exclusive limits to inclusive stored endpoints using checked i64
arithmetic, then intersects them with existing constraints. Overflow and empty
intersections return contradictory-bounds with the original operation and limit;
they never wrap or round. Validation continues to borrow the immutable plan.

Three runtime tests pass, covering repeated constraints, inclusive/exclusive
intersection, signed endpoints and construction-error fields. A formatted-source
native witness injects exclusive bounds into generated storage and verifies the
exact integer 9007199254740993 succeeds while both neighboring endpoints fail.
This is backend acceptance only: source syntax, editor/catalog support and matched
exclusive workloads remain pending. No timing or consumer refresh is claimed.
SQLite: `tsrust-schema-exclusive-runtime-{red,check,native,format}-525b4d9f.log`.


## Public exclusive integer constraints

Schema<int>.gt(limit) and .lt(limit) consume the original schema and return a new
immutable integer schema. They exclude the supplied endpoint, intersect existing
bounds and preserve exact signed 64-bit comparisons. Empty intervals throw
SchemaConstructionError immediately, retaining the original operation and limit.
Validation borrows the schema and returns ordinary too-small/too-large issues.
String schemas do not gain numeric exclusive methods.

Focused native tests verify chaining, exact output beyond JavaScript's safe range,
JSON endpoint rejection, both impossible signed endpoints, empty adjacent ranges,
caught error fields and original-schema reuse rejection. Editor hover, signature
help and inferred schema/result types pass; exact catalog examples execute natively.
The new native_schema_int_exclusive workload validates 7 and rejects 10 under
exclusive bounds 2 and 10, observes every exposed issue field, and matches Node
after TypeScript compilation on four sizes with no timing samples. Source hashes,
guards, independent checksum, ABI and report registration are included. Its small
matched domain does not replace native full-range coverage or pinned Zod evidence.

SQLite: `tsrust-schema-exclusive-public-*-b75065bb.log`, including the initial
source rejection and missing report registration caught by benchmark tests.
Floating constraints, composites, fused decoding and S11 acceptance remain pending.
Consumers remain at accepted S10; no measurement or refresh is claimed here.

## Floating constraint runtime groundwork

NumberBounds now owns reusable inclusive/exclusive floating constraints. Builders
consume the plan, intersect prior limits and return structured construction errors
for nonfinite limits or intervals with no finite representable value. Exclusive
bounds use the adjacent representable value, without an arbitrary epsilon. Both
zero signs compare equally; successful validation does not rewrite input bits.
Errors preserve the supplied floating limit and operation. The internal numeric
validator also uses these construction checks, preventing divergent empty-range
acceptance between its plan and the generated runtime backend.

Focused cases cover finite extrema, adjacent values, subnormal neighbors of zero,
NaN/infinity rejection, repeated intersections, negative zero and original error
fields. The runtime error container is generic internally; existing generated
integer/string callers retain an i64 limit. Public floating builders and the public
construction-error representation are not yet integrated. No new source API,
editor acceptance, benchmark measurement or consumer refresh is claimed.
SQLite: `tsrust-schema-number-bounds-*-c998cad1.log`.

## Generated number-schema storage

Generated Schema<number> values now own NumberBounds. Direct floating input,
checked native integer conversion and checked JSON extraction consult the stored
plan before producing successful output. Conversion failures retain priority over
bounds: an inexact integer reports number-precision rather than a range error.
Direct integer bound failures report observed integer; JSON number failures report
observed number. Validation borrows the plan without copying or rebuilding it.

A formatted-source native witness injects exclusive floating bounds into generated
storage, verifying both rejected endpoints, fractional success, direct integer
bounds and conversion-error precedence. Existing number tests retain nonfinite,
negative-zero, JSON and exact-conversion behavior. This is generated backend
integration, not acceptance of public floating builders or their error signature.
SQLite: `tsrust-schema-number-storage-*-b7db8d7b.log`.


### Approved floating construction-error payload

The owner approved retaining one SchemaConstructionError identity and changing
its limit field to int | number. Integer bounds must retain exact signed 64-bit
payloads; floating bounds retain their floating payloads. Never convert integer
limits to floating point merely to unify the error shape. Callers may need to
narrow the union before arithmetic. Compiler, native error emission and editor
documentation must agree before public floating builders are accepted.

The ordinary throwing adapter now projects its limit parameter from the binding,
rather than hardcoding int. This independent prerequisite supports both scalar
limit types; it does not yet change the public error field or expose builders.
Evidence: `tsrust-schema-limit-adapter-*-c35028ae.log` in SQLite.

## Shared numeric construction-error field

SchemaConstructionError.limit now uses the approved int | number field. Native
signature metadata projects explicit union alternatives into the ordinary checked
union arena; borrowed-shape traversal also visits each alternative. Integer builder
errors inject an exact integer variant without floating conversion. Existing
exclusive-builder catches retain their signed endpoint assertions. Public number
builders remain pending; this checkpoint establishes their shared error contract.

Focused checks cover union rendering/projection, borrowed alternative detection,
native integer-builder catches, explicit integer/floating error records and editor
hover of the inferred limit. The catalog documents the union and narrowing before
arithmetic. Evidence: `tsrust-schema-error-union-*-3451e2fd.log` in SQLite.


## Public floating constraints

Schema<number> now exposes consuming min/max/gt/lt builders with number limits.
Inclusive and exclusive constraints intersect stored bounds. Nonfinite limits
throw nonfinite-limit; intervals without a finite representable value throw
contradictory-bounds. SchemaConstructionError retains the original floating limit
in its numeric union. Validation keeps checked conversion before bound checks and
borrows the immutable plan. Source ownership checking rejects original-schema reuse.

Focused native acceptance covers fractional success, rejected JSON and integer
endpoints, contradictory fractional limits, nonfinite construction and all four
consuming methods. Hover, signature help and chained schema/result inference pass.
All exact schema catalog examples execute. The native_schema_number_bounds workload
includes TypeScript, fingerprints, source guards, oracle, ABI and report registration;
four sizes agree across native and Node-after-tsc with zero timing samples. Ten
benchmark contract tests pass. The handwritten counterpart is not the pending
pinned Zod comparison. No S11 milestone or consumer refresh is claimed.

SQLite: `tsrust-schema-floating-*-0c0fc2f5.log`. Literal/enum schemas, composites,
fused decoding and the remaining S11 acceptance scope remain required.

## Typed literal backend

The internal LiteralPlan wraps a typed consuming validator and owns its expected
value. Validation borrows that plan and returns the existing successful input,
without cloning strings or converting exact integers through floating point.
Underlying missing/type/precision failures take precedence over literal comparison.
A same-type mismatch produces literal-mismatch with bounded paths and controlled
type summaries; neither the input nor expected literal is retained in the error.
Finite floating literals use numeric equality, so both zero signs compare equally
while successful output retains the input sign. Nonfinite expected values are
rejected during private plan construction.

Three focused cases cover reusable string storage transfer, redaction, exact i64
endpoints, checked number precision, negative zero, nonfinite construction, null
versus absence, array-index paths and failure with zero retained-issue budget.
This remains test-only backend groundwork. Public s.literal/enum APIs, inferred
literal output types, generated plans, editor documentation and matched workloads
are still required. SQLite: `tsrust-schema-literal-*-70853c77.log`.

## Typed object assembly groundwork and literal-type dependency

Inspection confirmed checked types did not represent literal values. The syntax
prerequisite below now preserves literal annotations; checked singleton identities
remain pending. Precise s.literal output and literal discriminators therefore
need compiler/editor type support before public acceptance; do not silently widen
that requirement to dynamic JSON or claim scalar-only inference completes it.

The internal object backend now assembles statically typed field products from
consumed JSON object storage. Known fields move into their validators; optional
absence remains distinct from null. Unknown fields reject by default with original
field paths and controlled kind summaries. Sibling validation continues to collect
bounded errors, but partial typed output is dropped on any failure. Even a zero
issue budget cannot accept unknown fields. Generated record assembly can consume
the typed product without a dynamic output map or implicit input clone.

Three focused cases cover string allocation transfer, optional absence, nested
typed output, missing fields, wrong types, unknown-field paths, redaction and zero
retained-issue budgets. Field-list construction assumes distinct field names;
public object construction must enforce this. Strip/retain policies, source APIs,
generated records, editor inference, fused decoding and matched workloads remain
required. This backend is test-only groundwork, not public object acceptance.
SQLite: `tsrust-schema-object-*-5eb9891a.log`.

## Object construction and disposal checks

ObjectPlan now has a checked constructor and private field storage. Construction
rejects duplicate field names before consuming validation input; the reusable plan
does not rebuild the name set during validation. Names compare exact strings,
including empty names; canonically equivalent Unicode spellings remain distinct.
This closes the earlier backend assumption about unique field names.

Explicit drop counters verify each successful field output is released exactly
once when a required sibling fails or an unknown field rejects the object. On
success, outputs remain owned by the caller until that caller drops them. These
checks also exercise zero retained-error budget without accepting invalid input.
Five focused object tests pass. Public builders, policies, record inference and
fused decoding remain pending. SQLite evidence:
`tsrust-schema-object-construction-*-b5d20026.log`.

## Explicit object policy backend

Checked object plans now support consuming transitions to strip or retain unknown
fields. Default construction still rejects unknown keys. Strip returns only typed
declared-field output and drops unknown storage. Retain moves unknown entries into
an owned map alongside typed declared fields, without cloning values or including
known keys twice. Both alternate policies preserve ordinary declared-field failures
and bounded issue handling; neither can convert invalid known input into success.

Seven focused object cases pass, including pointer identity for retained string
storage, stripping, invalid known fields under both policies and previous default
rejection/disposal coverage. This is an internal representation: public retained
output spelling and source builder integration remain unimplemented. No dynamic
map replaces the typed declared-field output. SQLite evidence:
`tsrust-schema-object-policies-*-a23e2996.log`. No timing or consumer refresh.

## Composite native signature instantiation

Native type-shape preparation now accepts explicit concrete signature parameters.
Canonical keys resolve those parameters before interning, so repeated identical
specializations reuse metadata while Schema<string[]> and Schema<int[]> retain
distinct checked identities. Missing parameters fail without partial record state.
Existing concrete bindings use the same helper with no arguments. Callable
returned-borrow validation remains separate and mandatory; preparing a type shape
does not register or authorize a native callable.

Five focused binding tests pass, including ordinary/view Results, numeric unions,
distinct array-schema specializations and repeated preparation. This is a type
preparation prerequisite; public array/object constructors, child-plan storage,
validation lowering and editor inference remain pending. SQLite evidence:
`tsrust-schema-instantiation-*-a2daa32e.log`.

## Owned array-schema construction

s.array(element) now consumes a child Schema<T> and constructs Schema<T[]>.
Nested construction preserves nested element types, and generated records own the
child plan inline without cloning or a shared owner. Construction reuses ordinary
record specialization and ownership analysis. Source reuse of a consumed child
rejects; a function can return Schema<string[][]> built from constrained children.

The constructor has catalog documentation and an executed example. A focused
native test verifies nested construction and ownership rejection. Editor coverage
checks inferred nested output; its initial missing inference and repair are retained
in SQLite under `tsrust-schema-array-construction-*-9b24a3c6.log`.

This is constructor-only progress, not array-schema feature acceptance. Array
safeParse, indexed bounded errors in generated execution, validation documentation,
matched workload support and the complete composite milestone remain required.

## Shared consuming array loop

The typed array validation loop now lives in the runtime source included by
generated programs. Internal ArrayPlan delegates to that same loop. It consumes
every input element, preserves indices, clears partial output as soon as failure
is observed, and immediately drops later successful outputs while allowing bounded
issue collection to continue. Empty input succeeds without invoking the child.
No runtime trait object, input clone or shared output owner is introduced.

A new drop-order witness checks disposal before subsequent callbacks, not only at
function return. Existing schema array, numeric, literal and object composition
checks exercise the delegated path. Public array safeParse and native error-path
conversion remain pending; the shared loop is their ownership prerequisite.
SQLite: `tsrust-schema-array-runtime-*-ebecea06.log`.

## Generated bounded error conversion

Generated programs can now convert the shared bounded issue collector into public
SchemaError records. Conversion moves owned field names and preserves index paths,
issue truncation, path truncation and controlled expected/observed type summaries.
It does not retain invalid input values or require the JSON runtime.

A focused generated-native witness covers indexed paths, Unicode field-path
truncation, missing fields and a zero-issue budget. The original compile failure
and repaired checks are retained in SQLite under
`tsrust-schema-issue-bridge-*-9f9d4f7d.log`. This is a prerequisite for composite
validation; public array safeParse and its matched workload remain pending.

## Owned array validation integration

Array schemas now validate matching owned scalar arrays, including nested arrays,
through the normal checker and generated native execution. The schema is borrowed
and reusable; input transfers on both outcomes. Literal, inline-constructor and
variable inputs are supported. String, int, number, boolean and null leaves retain
their scalar constraints; exact integers never pass through floating point.

Generated validators use the shared consuming loop and bounded error collector.
Errors preserve input order and indexed paths. Defaults retain 64 issues, 32 path
segments and 1024 field-name bytes per path; truncation remains explicit. Empty
arrays succeed. Partial output is dropped after failure while remaining elements
are visited. The implementation currently builds output storage; allocation and
throughput optimization remain part of the family milestone.

Focused native checks cover plan reuse, consumed-input rejection, all scalar
storage families, nested constraints, empty input and issue truncation. Editor
coverage verifies inferred nested Result types and substituted ownership signatures.
The catalog includes an executed array validation example. The registered
native_schema_owned_array workload has a handwritten TypeScript counterpart,
independent checksum, indexed error observers, source guards and ABI support.
Four untimed cases agree across native and Node-after-tsc; no timings were collected.
SQLite evidence: `tsrust-schema-array-validation-*-2f62407c.log`.

This completes the matching-owned-input path only. JsonValue arrays, checked
numeric conversion between element types, broader composite schemas and the
remaining S11 milestone acceptance are still required. Installed consumers remain
at the accepted S10 checkpoint.


## JSON array validation integration

Array safeParse also consumes JsonValue and validates each nested array level.
Successful strings move their storage; integer extraction preserves signed 64-bit
values and rejects fractional or out-of-range values. Number extraction rejects
precision loss before applying bounds. Structural mismatches report the expected
and observed kind at the current indexed path. Existing error-count and path
budgets apply, with no retained input payloads.

Focused native checks cover all scalar JSON kinds, nested success and failure,
non-array roots, both int64 endpoints, unsigned overflow, fractional integers,
number precision rejection and consumed-input reuse. Catalog signatures and an
executed JSON-array example cover the public contract; editor signature checks pass.
The native_schema_json_array workload includes source guards, independent result
checks, ABI registration and a matched handwritten TypeScript reference. Four
untimed cases agree with Node-after-tsc; all 12 schema workload contracts pass.
SQLite evidence: `tsrust-schema-json-array-*-5ad48447.log`.

Typed array numeric conversions and the broader S11 surface remain pending. This
checkpoint does not constitute family measurement or a consumer refresh.


## Checked integer-array conversion

Number array schemas now accept owned int arrays with the same nesting depth.
Each integer converts only if the resulting float represents it exactly. The check
compares through a wider integer representation so saturating casts cannot accept
rounded i64::MAX. Precision rejection precedes bounds; both precision and bounds
issues report observed integer and preserve the nested index path. Input transfers
on every outcome, and output remains number[] (or the corresponding nested type).

Focused native checks cover exact conversion, i64::MIN, rejection of i64::MAX and
9007199254740993, direct integer literals, nested bounds, error precedence and
consumed-input reuse. Editor signatures expose the matching integer array alternative.
Catalog examples execute with the current compiler. The new
native_schema_number_array_int workload uses a BigInt TypeScript reference,
independent checksum, indexed error observers, source guards and ordinary ABI
registration. Four untimed native/Node cases agree; all 13 schema workload
contracts pass. No timing or family acceptance is claimed.
SQLite: `tsrust-schema-array-conversion-*-c1c8c95d.log`.

Next composite work is public typed object construction and validation, followed
by presence/default and union composition. Broader typed-input support, fused
text decoding and the other S11 requirements above remain in scope.

## Public required-field object checkpoint

`s.object({ name: s.string(), age: s.int() })` consumes the shape and infers a
record output. Nested object/array fields retain child plans. An explicit
`Schema<Person>` annotation selects a matching declared record, independent of
shape field order. Empty shapes are supported. `safeParse` borrows the reusable
schema, consumes matching owned records or JSON, and returns typed output.
JSON validation reports missing fields separately from unknown fields, rejects
unknown fields by default, and preserves named/indexed nested paths within the
existing bounded error contract. Successful field storage moves into output;
partial output is released after failure. Child schemas and consumed inputs cannot
be reused. Invalid shape values receive a schema diagnostic.

Project metadata carries compiler-inferred structural fields into editor hover
and field access without exposing generated record names. Catalog entries include
executable examples. `native_schema_json_object` has a matched TypeScript reference,
independent checksum, source guards, ABI/report registration and untimed parity
at four input sizes. It is unmeasured, as are the other S11 workloads.

Presence/defaults and the remaining S11 families
are still required. The owner approved explicit retention returning `{ value: T, extra: Map<string,
JsonValue> }`, preserving ordinary typed field access and avoiding collisions.
Internal object policy prototypes do not accept those public APIs.
No S11 family acceptance or installed-consumer refresh is claimed.

Focused acceptance: 31 schema compiler/native checks (including executable catalog
examples), 15 schema editor checks and 14 benchmark contract checks passed.
Whole-project formatting preserved tokens and passed idempotence; one new workload
was formatted and its source fingerprint refreshed. SQLite retains exact logs
`tsrust-schema-object-native-batch.log`, `tsrust-schema-object-editor-batch.log`,
`tsrust-schema-object-contract-formatted.log`, `tsrust-schema-object-parity.log`
and `tsrust-schema-object-format.log`, together with original red and repair logs.

## Explicit object stripping

`.strip()` consumes an object schema and returns an immutable reusable schema of
its same output type. Unknown JSON fields are dropped; declared fields remain
required and checked. Default rejection is unchanged. Empty shapes work, and
receiver consumption is diagnosed. The policy lives in the owned plan, so nested
schemas preserve their individual policies. `.retain()` remains the next public
policy, with its approved separate typed value and dynamic extras.

Three affected object native checks and the focused strip editor check passed.
Executable schema catalog examples passed. The additional matched stripping
workload has four-size native/Node untimed parity and all 15 schema benchmark
contracts pass. Whole-project formatting passed. SQLite retains original red and
repair evidence plus `tsrust-schema-object-strip-native-green.log`,
`tsrust-schema-object-strip-editor-green.log`, `tsrust-schema-strip-catalog.log`,
`tsrust-schema-strip-contract-green.log`, `tsrust-schema-strip-parity.log` and
`tsrust-schema-strip-format.log`. No timing or S11 family acceptance is claimed.

## Explicit unknown-field retention

`.retain()` consumes an object schema and returns a reusable schema whose inferred
output is `{ value: T, extra: Map<string, JsonValue> }`. Declared fields remain
required and checked. JSON keys and values left after declared-field extraction
move into `extra` without cloning. Missing or invalid declared fields still fail;
unknown fields do not produce issues. Ordinary owned record input returns an empty
extras map. Nested JSON plans preserve each child's retention policy, and matching
owned nested retained values keep their extras during validation. Retained plans
are terminal with respect to policy builders; selecting another object policy
requires a fresh schema. Default object rejection is unchanged.

Separate storage prevents keys such as `value`, `extra`, and `get` from colliding
with typed fields or map methods. Compiler project facts distinguish retained
outputs so editor help displays the raw input contract and inferred output shape.
Catalog examples and a matched retained-object workload cover the public API.
The family now has 16 registered unmeasured workloads. Presence/default factories,
remaining composites, fused decoding and S11 milestone acceptance remain pending.

Focused retention acceptance: 33 schema compiler/native checks including executable
catalog examples, 17 schema editor checks, and 16 schema benchmark contract checks
passed. Retention native/Node untimed parity passed at four input sizes. SQLite
retains original red/repair logs and `tsrust-schema-retain-native-batch.log`,
`tsrust-schema-retain-editor-batch.log`, `tsrust-schema-retain-contract-formatted.log`,
`tsrust-schema-retain-parity.log` and `tsrust-schema-retain-format.log`.

## Approved default factory contract

The owner approved factory-based defaults: a reusable zero-argument factory runs
only when validation finds a missing field. Its result is validated against the
child schema. Explicit null and invalid supplied values do not invoke the factory.
This avoids hidden cloning of owned defaults. Both inline callbacks and named
functions are required. Partial working-tree implementation is described below;
the default gate remains unaccepted.

Recommended spelling is `.default(factory)`, consistent with the factory-accepting
names in [Zod](https://zod.dev/api#defaults) and
[Joi](https://joi.dev/api/#anydefaultvalue). The owner requested this naming review;
no additional alias is introduced. Document the semantic difference explicitly:
Zod default values bypass parsing, while Tsrust validates generated defaults
(similar to Zod's prefault behavior). Reference documentation is naming evidence,
not a pinned differential test baseline.

## Public optional and nullable schemas

`.optional()` and `.nullable()` consume the child plan and infer output unions with
`undefined` or `null`. Optional object fields may be omitted; nullable fields are
still required. Combining both allows both alternatives without relaxing child
validation. Supplied invalid values fail. Repeated builders add no duplicate
alternative. Scalar and composite child plans retain their ownership and constraints;
owned input moves through validation. Numeric alternatives preserve checked
integer conversion, including number-precision errors before bounds.

Native witnesses cover JSON and owned object fields, root values, array elements,
combined presence, optional null schemas, exact and lossy numeric conversion, and
rejected reuse of child plans and consumed strings. Editor project facts retain
optional field markers and output unions. Catalog entries explain missing/null
behavior and shared executable examples are verified once per distinct source,
while all duplicated entries must retain identical output contracts.

`native_schema_json_presence` adds a matched TypeScript reference with independent
checksum, named error observations, source guards and ABI/report registration.
The 17 schema workloads remain unmeasured. Defaults, other S11 requirements, fused
decoding, family regressions/measurements and consumer refresh remain pending.

Focused presence acceptance: 34 schema compiler/native checks (including distinct
executable catalog sources), 18 schema editor checks, and 17 schema benchmark
contracts passed. Native/Node untimed parity passed at four input sizes. Whole-project
formatting preserved tokens, refreshed the new workload fingerprint, and passed
idempotence. SQLite retains original red/repair evidence and
`tsrust-schema-presence-native-batch.log`, `tsrust-schema-presence-editor-final.log`,
`tsrust-schema-presence-contract-formatted.log`, `tsrust-schema-presence-parity.log`,
`tsrust-schema-presence-format.log` and `tsrust-schema-presence-format-final.log`.

## Default factory implementation witness

The focused public native test
`schema_defaults_validate_fresh_factory_results_only_for_missing_fields` currently
fails with `unsupported` at `.default(fresh)`. It covers named factories, fresh
owned array output on repeated validation, supplied values bypassing defaults,
explicit null rejection, and validation of an inline factory result against bounds.
The exact failing check is retained in SQLite as
`tsrust-schema-default-red.log`. This is a red TDD witness, not accepted behavior.

Implementation must retain reusable factory environments without hidden cloning
or sharing. Existing owned callback storage requires mutable invocation, while
readonly owning storage currently uses an explicitly shared callback contract.
Schema validation borrows an immutable plan. Resolve that representation boundary
with checked ownership before exposing defaults; accepting only capture-free
functions does not satisfy the full approved runtime-factory contract.

### Factory capture and pipeline contracts

The owner approved readonly factory captures and ordinary side effects such as
logging and fresh allocation. Factories must not mutate or consume captured state.
Factory exceptions propagate normally; invalid returned data produces schema
validation errors. This restriction concerns captured environments, not the owned
input of later transformation stages.

Preserve chainable refinement, transformation and explicit coercion APIs.
Refinements inspect values and report issues; transformations can consume or
mutate owned input and return the next typed value. Explicit coercion precedes
subsequent validation. The owner requested a path for mutation in these pipelines;
readonly default factories must not accidentally prohibit it. The subsequent approval below settles refinement versus transformation roles;
the callback-based coercion contract below settles its arguments.

Pipeline acceptance must include a callback that changes owned input, a subsequent
validation stage that observes the changed value, and editor inference for the
resulting output type. Keep callbacks reusable without allowing mutation or
consumption of their captured environments. Test early failure and exceptions:
later stages must not run, and partially transformed owned values must be dropped
exactly once. These requirements apply to runtime schema plans as well as literals.
The owner approved separating predicate checks (`.refine`, preserving the input
type) from value-producing transformations (`.transform`, inferring the returned
type), with explicit `.coerce` conversion before subsequent validation. Checkers,
hover, completion and signature help must track the input and output of every
stage, including intermediate variables and reusable runtime plans. Negative
tests must reject downstream operations valid only for an earlier stage's type.
The owner approved `schema.coerce((input: Input) => value)`: consume the raw input,
run the conversion callback, then validate its result with the receiving schema.
Infer the accepted raw input type from the callback and preserve the receiving
schema's validated output type. Conversion exceptions propagate normally;
invalid converted values produce validation errors. The conversion may mutate
its owned input, but its captures follow the same readonly, reusable contract as
transforms. A consuming coercion inside a union commits that branch before the
callback runs; subsequent failure must not retry another branch or copy input.
Checker and editor acceptance must cover intermediate schema variables, chained
refinements and transforms, incorrect raw input, invalid callback result types,
and conversion exceptions. This approval is a contract, not implementation or
measurement acceptance. Do not treat readonly default factories
as a decision to remove mutation-capable transformation pipelines.

Pipeline implementation is in progress after the union checkpoint. The checker
now has a separate registry keyed by exact schema record, retaining source plan,
callback family, accepted input and resulting output. Constructor lowering is
being wired to uniquely owned readonly callback storage. Equal output types do
not merge different input contracts. Initial `safeParse` lowering and consuming
stage emission now transport conversion exceptions through the existing callback
error ABI. Coercion precedes receiving-schema validation; transforms follow it.
Checked project facts now carry pipeline input and output separately. Initial
editor completion, hover and signature help use those facts, with catalog
documentation for refinement, transformation and coercion. A new editor witness
checks array input with boolean output after conversion and transformation.
Array plans now retain their exact element schema identity, including nested
pipeline arrays. Initial consuming emission handles typed and eligible JSON
arrays, tracks element issue paths, and stops on callback exceptions. Written
witnesses cover transformed element types and nested array outputs. These paths
have not yet been executed.
Object pipeline plans now retain exact field schema identities and distinct
structural input/output records. Initial consuming emission handles owned and
eligible JSON objects, applies transforms to generated defaults, reports field
paths, rejects unknown JSON fields, and propagates callback exceptions. New
witnesses cover transformed fields, nested array pipelines and transformed
defaults. Initial `.strip()` and `.retain()` lowering and emission now preserve
typed transformed fields and move retained unknown JSON data into the separate
`extra` map. Editor facts expose policy methods only for eligible object plans.
Policy witnesses remain unrun; union preparation and section validation remain
open.
The matched `native_schema_pipeline` workload is registered with TypeScript
counterpart, independent checksum, source guards, fingerprints, shared driver ABI
and report grouping. It observes mutating coercion, receiving-schema rejection,
refinement and transformed output. It is unmeasured and has not yet passed untimed
parity; registration is preparation for section acceptance, not evidence of it.

The current readonly callback ABI expects native typed storage, whereas JSON
strings and records initially live in JSON storage. The pipeline implementation
will bridge this with reversible typed projections after ordinary borrowed
preparation. Payloads move into typed storage; restoration metadata retains the
original representation, unknown fields and missing-field provenance. A rejected
readonly refinement restores the original input before trying another branch.
No payload is cloned, shared or given an erased lifetime. Defaults and checked
conversions prepared earlier must not run again. Explicit source operations such
as `clone` retain their ordinary behavior.

Before a consuming transform or coercion runs, restoration state is discarded.
Later rejection or exception cannot reopen alternative selection. This changes
the internal callback bridge, not the approved commitment contract. Metadata may
require allocation; matched measurements must include that cost. Internal trial
primitives and unrun witnesses now cover ownership transfer, exact restoration,
exception cleanup and commitment. Generated projection/restoration integration
is still required; these primitives do not establish public union-pipeline
acceptance.

Generated JSON projection/restoration covers scalar payloads, exact numeric
representations, arrays, records, prepared defaults, presence wrappers, retained
extras and selected native schema alternatives. Owned projections now preserve
removed fields, exact preconversion scalars, selected alternatives and prepared
defaults as well. Their inverse moves payloads back without revalidation.

Pipeline union construction retains ordered branch identities and separate
input/output unions. The generated trial dispatcher now connects native schemas,
refinements, transforms, coercions, unions and callback-bearing arrays/objects.
Readonly rejection restores input before fallback. A consuming callback commits
the enclosing branch; later rejection never retries another alternative. Nested
callback pipelines stop later callbacks after failure. Ordinary native preparation
still collects bounded issues before projection. Missing-field trials retain
absence separately from generated defaults. Retained JSON extras move into the
public extra map and return to the original object during readonly rollback.
Discriminated pipelines inspect input tags before transforms change output types.

Heterogeneous restoration uses uniquely owned boxed callables with lifetimes
bounded by the actual retained values. These metadata allocations require
measurement. Internal unrun witnesses cover array rollback, commitment, payload
allocation identity and skipped later callbacks. Public unrun witnesses cover
nested array/object rollback, exact integers, missing defaults, retained extras,
coercion failures and discriminated branches transformed into scalars.

Presence/default wrappers now retain exact source plan identity and separate input
and output types. A pipeline default produces source input, then validates through
the receiving chain. Missing fields are distinct from supplied undefined or null.
An outer optional wrapper preserves inner defaults and does not hide their failures.
Owned object input inference preserves omitted fields rather than materializing an
undefined value before the default can inspect omission. Catalog examples and
editor witnesses cover these chains alongside refine/transform/coerce.

Focused pipeline section acceptance is complete on macOS. The preserved runs
cover 17 public pipeline cases (including exact catalog examples), 21 existing
enum/union cases, 68 schema runtime checks, 30 schema editor cases and 25 workload
contract checks. The new workload matches native and Node-after-tsc checksums at
0, 1, 17 and 2,000 iterations without timing. Whole-project formatting changed one
workload file and refreshed its fingerprint after token preservation/idempotence
checks. Source fixtures also execute after formatting through `format_round_trip`.

Repairs retain their original failure evidence: one generated-format brace,
missing type annotations for an absent object field, compiler facts omitted for
lowered pipeline records, and an editor assertion sensitive to insignificant type
spacing. Only affected checks were rerun. SQLite records:

- `tsrust-pipeline-section-compiler.log`
- `tsrust-pipeline-section-compiler-repair1.log`
- `tsrust-pipeline-section-objects-repair2.log`
- `tsrust-pipeline-section-owned-rollback.log`
- `tsrust-pipeline-section-existing-schemas.log`
- `tsrust-pipeline-section-runtime.log`
- `tsrust-pipeline-section-editor.log`
- `tsrust-pipeline-section-editor-repair1.log`
- `tsrust-pipeline-section-editor-repair2.log`
- `tsrust-pipeline-section-workloads.log`
- `tsrust-pipeline-section-parity.log`
- `tsrust-pipeline-section-format.log`
- `tsrust-pipeline-section-format2.log`
- `tsrust-pipeline-section-workloads2.log`
- `tsrust-pipeline-section-parity2.log`

The final workload includes three validations per iteration: successful coercion,
failed receiving-schema validation, and JSON-array rollback before a second union
branch transforms the value. Its checksum is `seed + 7 * n`. The `*2.log` records
verify that expanded workload; initial contract/parity logs retain the earlier
two-operation, `seed + 5 * n` scenario. No timing record is relabeled or overwritten.

This accepts the scoped pipeline implementation, not S11 or measured performance.
Fused text decoding, recursive/lazy composition, collection schemas, richer issue
APIs, JSON Schema export and dependency-gated validators remain on the roadmap.
Full family regressions, timing review and installed consumer refresh remain
pending. No claim of arbitrary Zod compatibility or support for every possible
schema annotation/chain follows from these focused checks.

### Unique readonly factory storage prerequisite

A distinct internal storage proof now admits readonly callable signatures only
when the checked closure owns its environment. It rejects borrowed, mutable and
single-use captures. Native family emission retains captures directly and invokes
through immutable references, without shared handles or a generated Clone
implementation. Existing explicitly shared callback storage keeps its behavior.

This is an internal prerequisite, not public default support. Schema plan storage,
source lowering, factory error propagation, editor documentation and matched
workload acceptance still remain required. A separate generic-function witness
confirmed ordinary array-schema construction and validation currently execute
through specialization; that observation does not prove all reanalysis paths.

Six focused readonly-related checks passed, including generated-native family
execution and rejection of borrowed, mutable and consuming environments. Exact
logs, including original red and repair checks, are verified in SQLite under
`tsrust-schema-owned-read-*.log`. No public default, S11 milestone, performance
measurement or consumer refresh acceptance is claimed.

The unique readonly proof now participates in callable registry identity, origin
validation and production native family emission. Registry tests distinguish it
from borrowed storage with the same invocation signature and verify rejected
mutable origins leave the existing family unchanged. A production witness returns
a uniquely owned callback with array captures and invokes it repeatedly.

Factory exception acceptance must also exercise the saved per-family error ABI.
The initial throwing backend witness added a family after source checking without
retaining its checked error contract; the emitter correctly rejected the incomplete
graph. The repaired witness retains the sole origin's checked exception contract.
Public default lowering must happen before ordinary error-effect analysis so
those contracts are derived through the normal compiler path.

Eight focused readonly checks passed after registry integration. The repaired
production exception witness also passed generated native execution, preserving
success and failure through the family ABI. SQLite retains exact original and
repair records as `tsrust-schema-factory-*.log`. Whole-project formatting changed
zero files. Public defaults remain pending; no feature-family acceptance is claimed.

## Default integration in progress

Source `.default(factory)` lowering now consumes the child and factory, retains
a uniquely owned readonly environment, and attaches it to the runtime plan.
Attaching a factory allocates one owned box; schema values never share captures.
Constraint reconstruction moves the existing factory into the replacement plan.
Plans of the same output type retain separate optional factory instances.

Missing JSON object fields invoke the attached factory and validate its returned
value. Supplied null or invalid data does not invoke it. Executed witnesses cover
fresh arrays, named and inline factories, readonly captured arrays, repeated
validation, invalid generated values, boolean/null/presence defaults, and
independence from ordinary required-field plans. Mutable/consuming captures and
reuse of consumed child plans are rejected. Owned-record omissions now construct
required output through the same validation path, including explicit missing
errors when a runtime plan has no factory.

Hover/signature entries and an executable example have been added with current
limits stated explicitly. A focused editor witness and the distinct catalog-source
execution check passed before the owned-record extension.

Factory exceptions now propagate through the checked family error ABI, preserving
concrete payloads. Validators stop invoking later factories after an exception;
partial outputs are dropped. Focused native witnesses cover nested short-circuiting,
structured errors, forwarded plans and later-discovered factory error alternatives.
The earlier native failure and repairs remain part of the diagnostic evidence.

Default integration now has focused implementation acceptance. Missing owned and
JSON fields use fresh validated values; constraints, optional/nullable widening,
stripping and retention preserve factories without copying their environments.
Heterogeneous owned input shapes select separate typed validator helpers. Inferred
record-producing callbacks receive their schema output as contextual type.
Returned runtime factories with readonly owned captures execute repeatedly.
Existing callable-return restrictions still apply: a function returning different
closure definitions is diagnosed by the general checker. This gate does not
expand that separate callback representation boundary.

Focused compiler/native verification: 45 schema checks passed, including all 11
default-specific checks and executable catalog examples. Editor verification:
19 checks passed initially; the remaining new test expected a space in a rendered
Map type. Correcting that assertion passed the retained/default chain test without
changing editor behavior. All 20 schema editor cases therefore passed across the
preserved runs. Tests verify intermediate/final types, parameter help, required
narrowing and rejection of incompatible inputs and factory results.

The `native_schema_json_defaults` workload has matched TypeScript support,
independent checksum, source guards, fingerprints, report registration and ABI
checks. All 18 schema workload contract tests and four untimed cases per engine
passed. Whole-project formatting changed zero files and fingerprints. Exact red,
repair and green records are retained in SQLite under `tsrust-schema-default-*`.
There is no timing, S11 family acceptance or installed-consumer refresh.

Next required S11 work includes public literal/enum and union families, then the
remaining second-tranche APIs. The approved chain contracts above are required
for refinement, transformation and explicit coercion acceptance; they are not
implemented by the default work.

## Literal annotation prerequisite in progress

Public literal and enum schemas require checked singleton type identities.
The syntax layer now retains string, boolean, integer and number literal
annotations as distinct literal nodes rather than nominal type names. It preserves
exact numeric spelling (including integers beyond JavaScript's safe range and the
i64 minimum), decoded string values and original source spans. Literals compose
with arrays, unions, callable signatures and generic arguments. Generic identity,
substitution, alias/link traversal and editor canonical spelling preserve these
values; a quoted `"T"` is not a reference to a type parameter named T.

Checked singleton identities now preserve exact integer normalization, separate
int/number identities, and finite floating-point values. Equivalent zero spellings
share a type identity while native values preserve their original sign. Underlying
scalar storage and ownership stay unchanged; literal strings are owned strings,
not static enum tags replacing input. Proven values widen to their corresponding
scalar. Arbitrary scalar values cannot narrow to a literal without proof.

Focused checks cover aliases, generic specializations, records, arrays, literal
union injection, stable equality narrowing on both branches, explicit string
cloning, arithmetic returning scalar types, scalar string/number methods, and
rejection of assignments or updates that break the declared literal. Mutable
arrays do not gain unsafe covariance. Dynamic comparison operands do not prove a
particular literal. Editor facts preserve exact values; method hover, completion,
and parameter help use the corresponding scalar API.

This prerequisite checkpoint preceded public `s.literal` integration below; enum
schemas and discriminated unions remain pending. Continue the literal type
integration audit through those APIs: collection key and element contracts, contextual inference, and remaining
wildcard type matches. Add focused
regressions for each supported boundary; reject unsupported cases in the checker
rather than allowing native failures. Preserve exact output types throughout schema
chains and projected editor facts.

Checked evidence: 10 focused compiler/native cases, two numeric identity unit
cases, two editor cases, and six affected existing union regression cases. These
are bounded checks, not full compiler/LSP acceptance. Evidence prefix:
`tsrust-literal-*` in SQLite. Public schema docs/examples, matched workload support,
full family regressions and measurements remain required. No timing is claimed.


### Literal scalar dispatch follow-up

Literal values now use their underlying scalar contracts for Boolean, String and
Number conversion, union `typeof`, string indexing and string iteration. This
prevents literal false/zero/empty strings from accidentally taking object truthiness
and prevents native dispatch from treating literal strings as arrays or JSON values.
Editor inference reports indexed characters and iteration bindings as `string`.

Literal values and literal unions can widen into matching scalar union members.
Widening preserves the existing ownership policy: constructing new owned tagged
storage transfers its source, including when passed to a readonly consumer. An
attempt to introduce implicit scalar borrowing was rejected by existing ownership
and temporary-lifetime regressions and removed. Union payloads retain their ordinary
native representation with no hidden clone. Mutable collection covariance remains rejected.

This dispatch checkpoint preceded the public literal integration recorded below.
Enum constructors remain pending. Focused evidence uses
`tsrust-literal-{conversions,string-access,typeof,widening,readonly-widening,access-editor,dispatch-regression}*`
in SQLite. No full regression or timing acceptance is claimed.

Follow-up verification: 15 literal compiler/native cases, 21 existing nullable,
union, numeric-conversion, string-indexing and string-iteration cases, and three
literal editor cases pass. Failed exploration records remain alongside repaired
checks; no expected failure was ignored. The original ownership contract remains
in force. Whole-project formatting reported no source or fingerprint changes.


## Public literal schema integration

`s.literal(value)` now produces `Schema<Literal>` for a statically known string,
integer, finite number, boolean or null. Literal expressions and values already
carrying a singleton type are supported; broad scalar values cannot select a
static output identity. A constructor argument is evaluated once. Owned literal
strings transfer into reusable immutable plans.

Validation accepts the underlying owned scalar or JsonValue and consumes input on
both outcomes. Success preserves the singleton output type and moves successful
owned string storage without cloning. Different values produce `literal-mismatch`;
wrong JSON kinds produce `type-mismatch`. Exact signed 64-bit integers, checked
integer-to-number conversion, nonfinite rejection, and bounded redacted issues
remain in force. Positive and negative zero compare equal. Null is equivalent to
`s.null()` and remains distinct from undefined.

Literal children compose with typed objects, arrays, default factories and
optional/nullable wrappers. Matching broad owned records and arrays are validated
against literal children at runtime. Output types remain precise through these
chains; editor completion, hover and parameter help distinguish scalar input from
literal output. Literal plans do not expose primitive constraint builders such as
`.min()`; use a broader scalar schema when its output should remain broad.

Focused evidence is stored under `tsrust-schema-literal-*` in SQLite. Nine new
compiler/native cases cover scalar and JSON success/failure, exact numeric output,
owned composite mismatches, defaults, presence, checked numeric conversion,
constructor evaluation, ownership rejection and executable catalog examples. The
45 existing schema checks completed with one null-input rejection defect repaired
and rerun; completed passing checks are retained. Two editor chain cases and 19
workload contract cases pass. `native_schema_json_literal` has a matched handwritten
TypeScript reference, independent checksum oracle, ABI/driver support, guards,
source hashes and reporting registration. Four untimed native/Node-after-tsc cases
agree; no timing or full S11 acceptance is claimed.

Next: enum schemas and remaining first-tranche unions/discriminated unions, then
fused decoding and the approved second tranche. Extend literal integration checks
along those paths rather than erasing literal output identities to scalar types.


## Public string-list enum schemas

`s.enum(['ready', 'done'])` produces `Schema<'ready' | 'done'>`. Its input must be a
nonempty array literal of distinct statically known strings, including expressions
already typed as string literals. A singleton uses the literal schema contract.
Broad runtime arrays, duplicate alternatives and nonstring alternatives are
rejected. Native language-enum declarations and numeric enum inputs are not yet
supported; this checkpoint does not claim general enum or union coverage.

Validation consumes an owned string or JsonValue. Success transfers the original
string into its precise literal alternative; an unlisted string produces
`literal-mismatch`, and a nonstring JSON value produces `type-mismatch`. Errors
remain bounded and redacted. No successful string or reusable plan is cloned.
Constructor expressions run once in source order, even when equivalent unions
were previously declared in a different order.

Enums compose with arrays, owned and JSON objects, defaults and optional/nullable
wrappers. Presence wrappers remap each precise alternative without widening it to
string. Defaults remain owned by the correct nested plan. Checker results, hover,
completion and signature help distinguish broad scalar input from exact output.
All public catalog examples execute through the current compiler and native path.

Focused checks pass: six enum compiler/native cases, nine existing literal cases,
11 default cases, one presence case, one editor chain case, and 20 workload contract
cases. The new `native_schema_json_enum` workload has a handwritten TypeScript
counterpart, independent checksum oracle, source guards and hashes, ABI/driver
support and reporting registration. Four untimed native/Node-after-tsc cases agree.
Evidence prefix: `tsrust-schema-enum-*` in SQLite, including initial failures and
repairs. No timing, full S11 acceptance or installed-consumer update is claimed.

Next: remaining enum input forms, ordered unions and discriminated unions, fused
text decoding and the required second tranche. Refinement preserves type; transform
owns its input and determines the next type. Keep checker/editor chain tracking in
each implementation gate.


## Declared enum schema integration

`s.enum(State)` accepts an enum declaration and produces `Schema<State>`, preserving
nominal identity through defaults, presence, arrays, objects and imported aliases.
The declaration must be nonempty and numeric members must be finite. String-list
construction remains supported and deliberately remains string-only; numeric enums
use the existing language declaration syntax. Declarations are compile-time
identities, not new runtime reflection values. Local value shadowing does not grant
access to the shadowed declaration.

A declared enum schema accepts its own enum values, matching underlying scalars or
JsonValue. Numeric inputs use finite number semantics and checked exact int-to-number
conversion, including within owned arrays/records and presence wrappers. Lossy
integers produce `number-precision`, nonfinite input produces `number-nonfinite`,
wrong scalar values produce `literal-mismatch`, and wrong JSON kinds produce
`type-mismatch`. An unrelated nominal enum remains rejected even if its values match.
Successful output is the existing Copy enum representation. Raw strings are consumed
and released; there is no retained string allocation or implicit clone. Equal-valued
aliases select the first declaration member and keep ordinary enum equality.

Compiler facts expose checked enum scalar representations with nominal declaration
identities. The editor remaps those identities across import aliases, preserving
output names and displaying enum/scalar/int/JSON input alternatives through nested
presence and array chains. This metadata avoids guessing representation from source
initializer text. Stored callback layout now accounts for enum discriminants, enabling
reusable default factories returning enums.

The `native_schema_json_declared_enum` workload validates a finite numeric enum and
observes success and a rejected value. It has a handwritten TypeScript counterpart,
independent checksum oracle, guards and hashes, ABI/driver support and reporting
registration. Large-integer precision is tested separately and is not claimed as a
JavaScript JSON.parse equivalence. Four untimed native/Node-after-tsc cases agree.
Evidence prefix: `tsrust-schema-declared-enum-*` in SQLite. No timing, installed
consumer refresh or S11 family acceptance is claimed. Remaining first-tranche work:
ordered unions and discriminated unions, followed by fused decoding and the required
second tranche.


Declared enum focused acceptance: 10 enum/schema cases (including all executable
catalog examples), nine literal/schema cases, 11 default cases and five existing
enum cases pass. All 25 schema editor cases and 21 workload contract cases pass.
Whole-project formatting verified token preservation and idempotence; formatted
native workload output remains `0 2 41 4001`. These are bounded affected checks,
not complete compiler/LSP regressions or performance measurements.


## Ordered union implementation contract

The owner approved committing a union branch before its first consuming transform.
Branches are considered in declaration order. Before that commitment, validation
borrows the original input and can reject a branch without losing or copying it.
Once a branch's checks before a consuming transform succeed, that branch owns the
input. A later transform or refinement failure ends validation; another branch is
not tried. This deliberately prevents hidden rollback copies of mutated values.
Exceptions propagate and do not select another branch.

Prepare and commit are separate internal operations. Preparation may create and
validate a missing-field default once, retaining that owned result for commitment.
A rejected preparation releases its generated values. A chosen preparation moves
its cached defaults and original successful input storage into the typed output;
it must not rerun factories or validation callbacks. Nonmutating branch failures
can accumulate bounded errors. A later successful branch removes speculative
errors while preserving earlier sibling errors, truncation and zero-budget validity.

Union plans must preserve runtime branch order independently of canonical output
type identity. Overlapping branches, repeated output types and ordinary schemas
with the same output type remain distinct reusable plans. Do not implement unions
by deduplicating schemas through their output types, cloning input between attempts,
or routing typed input through JSON serialization. Discriminator selection must
inspect the tag without consuming input, then validate only the selected branch.

Implementation is in progress. The entire ordered/discriminated-union section,
including public integration, editor inference, executed examples and matched
workloads, will be validated together under the updated development workflow.
Runtime preparation/error checkpoints alone do not accept either public API.


Union section work in progress (not accepted): added internal preparation/commit
control and reversible issue bookkeeping, plus unrun witnesses for storage identity,
shared error budgets, nested selection, default cleanup and exception short-circuiting.
Public integration witnesses cover overlapping runtime plans, typed scalar unions,
owned input transfer, nested arrays/presence, default evaluation, discriminator
selection and field narrowing. They are intentionally not run until the section is
implemented. The current source checkpoint remains `d2c5ce66`; no new source
checkpoint, full regression, timing or public union acceptance is claimed.


Union lowering work in progress: added ordered plan identities separately from
normalized output types, consuming constructor analysis, and discriminator overlap
checks across literal and declared enum values. Equal integer/number discriminator
spellings normalize to the same key. Generated choice storage preserves duplicate
output branches and uses one unique allocation per union plan node; existing schema
outputs not participating in choices retain their original layout. Optional base
slots avoid constructing unused child plans for a choice. These changes are not yet
publicly exposed: namespace wiring waits for prepared validation/commit generation,
then editor/examples/workload integration. No checks have run for this section.

Prepared JSON validation now carries an explicit outer unknown-field policy into
choice branches. A union's `.strip()` overrides rejection for each attempted root
object; without that override, branch-local policies remain intact. Added an unrun
public witness covering rejection, branch-local stripping and outer stripping.
Owned discriminator inspection is being implemented with borrowed field access and
exact numeric comparisons, including integers above the floating-point precision
boundary. It is not yet connected to owned preparation/commit dispatch. Numeric
boundary and ordered-plan identity witnesses are written but have not been run.
Validation effect enumeration also follows preserved branch plans: normalized
output unions must not hide a default factory attached to an intermediate union.
Owned leaf preparation now generates borrowed scalar, literal and nominal enum
checks. Checked numeric conversions are cached in preparation state; owned string
storage is moved only by commitment. Composite dispatch integration remains open.

Composite owned preparation/commit is now wired for arrays, records, presence,
retained outputs, ordered alternatives and discriminated selection. Recursive
default lookup preserves runtime branch identity and cached generated values.
Presence adapters activate only for outputs that can contain runtime choices;
they preserve whole source plans rather than reading an absent base-plan slot.
Public namespace lowering and IntelliSense catalog entries/examples are added,
with an unrun editor chain witness. This supersedes the earlier namespace-wiring
and composite-dispatch status notes above. Workload integration and section
validation remain pending; no implementation acceptance is claimed.

First union-section batch (2026-10-12): 23 workload-contract checks and 26 schema
editor checks passed. Public native witnesses: one rejection test passed; four
execution tests failed. The batch exposed missing record-union common-field access
and discriminator narrowing, plus a generated JSON map type crossing the private
runtime boundary. Map visibility has been repaired through the bundled JSON
runtime re-export, without adding a user dependency or link flag; that repair is
not yet rerun. Record-union access/narrowing remains required before the section
can pass. Do not weaken the existing object/discriminator witnesses to hide it.
Ordered and discriminated union workloads now have matched TypeScript sources,
independent checksums, source guards, registrations and ABI checks. Native/Node
parity and timing remain unrun. The four closed batch logs are byte-verified in
SQLite under `tsrust-unions-section-{compiler-1,compiler-2,workloads-1,editor-1}.log`.

Second native batch: all 10 enum and 9 literal cases passed; three of five union
cases passed, including owned scalar/presence transfer and ordered object defaults.
Common-field reading and literal discriminator narrowing now reach native code.
The two remaining failures exposed equivalent inferred object shapes receiving
different output identities, and a witness moving an owned field directly through
a narrowed record (already prohibited by the ownership contract). Object output
identity now reuses an existing identical ordered shape while preserving separate
runtime plans; the witness moves the whole narrowed record before taking its field.
Both repairs await the next affected pass. Before section acceptance, finish the
common-field audit for nested borrowed fields, exact numeric/enum discriminators,
mutation invalidation and editor narrowing; retain/add regression witnesses.
Second batch evidence is byte-verified in SQLite as
`tsrust-unions-section-compiler-3.log`. No new checkpoint or timing acceptance.

Current union evidence: eight public union cases plus the catalog-example case
pass across the affected runs. The invalidation witness initially had a syntax
typo, corrected without changing its assertion; replacing the union invalidates
the discriminator fact. Nested common record/array reads and distinct exact tags
above 2^53 execute correctly. All 27 schema editor checks and 56 schema runtime
checks pass. Ordered and discriminated workload adapters agree with Node-after-tsc
for n=0,1,17,2000, with zero timed samples. Seven closed logs are byte-verified in
SQLite as `tsrust-unions-section-{compiler-4,invalidation-1,parity-ordered-1,
parity-discriminated-1,editor-2,examples-1,runtime-1}.log`.

Remaining acceptance audit: comparisons/access for discriminator fields whose
alternatives have different nominal enum types or different scalar families.
Tag-selection metadata handles these, but common-field typing currently requires
identical types or a common scalar base. Do not claim the full advertised
discriminator access contract until this path is implemented and exercised.
Whole-project formatting, source checkpoint and family measurements are pending.

Ordered/discriminated union section validation (2026-10-12): the mixed-type and
nominal enum field-access gap above is closed. Shared fields construct borrowed
union views, preserving exact alternatives and nominal enum identity. Literal
guards narrow record alternatives; replacing their owner invalidates the fact.
Outer scalar constraints also validate branch-generated defaults without rerunning
factories or silently retrying a selected default after a later outer failure.

Current evidence: 11 public compiler/native union cases, 28 schema editor cases,
23 workload contracts, and the prior 56 runtime schema cases pass. All documented
union examples execute. Whole-project formatting changed two workloads with token
preservation/idempotence verification and refreshed fingerprints; both formatted
workloads agree with Node-after-tsc at n=0,1,17,2000, untimed. Failed intermediate
runs remain preserved. Final records are in SQLite under
`tsrust-unions-section-{compiler-5,compiler-6,format-1,editor-3,workloads-2,
formatted-parity-1,formatted-parity-2}.log`.

This accepts the implemented union section only. Refinements/transforms,
preprocessing, recursive composition, collection schemas, JSON Schema export,
fused typed decoding and the S11 family acceptance requirements above remain.
Installed consumers remain at accepted S10. No family timing is claimed.


## Runtime validation naming and seeded decoder checkpoint

Runtime result types now use `ValidationError` throughout compiler declarations,
native bridges, hover, signatures and completion. The in-progress S11 name
`SchemaError` is no longer exported. SchemaIssue and SchemaPath retain their
existing names and fields; the schema definition/construction APIs are unchanged.

A private JSON `decode_seed` adapter now supports reusable runtime plans and real
source borrows. Full syntax, Unicode, numeric and resource validation precedes
seed callbacks. Existing direct typed decoding delegates to that adapter without
building a dynamic value tree. This is decoder groundwork, not acceptance of the
public fused schema method, which remains the next implementation section.

Focused checks passed: one public native error-import/pipeline witness, 31 schema
editor cases across the initial and affected rerun, 16 typed decoder runtime cases
and seven existing public typed JSON cases (including catalog examples). Eleven
editor assertions initially failed only because of comma spacing; assertions now
normalize whitespace without weakening type checks. Whole-project formatting
passed with no source or fingerprint changes. No timing or S11 family acceptance.
SQLite preserves exact logs as `tsrust-validation-naming-{typed,native,editor,
editor-repair,json-native,format}.log`. Installed consumers remain accepted S10.


## Fused JSON decoding checkpoint

`schema.safeParseJson(text)` borrows text and the reusable schema, constructing owned
output directly. Its tagged failures distinguish JSON syntax/resource errors from
runtime ValidationError values. Callback exceptions propagate. The complete JSON
syntax, UTF-8/Unicode, integer-token and resource checks run before callbacks,
including ignored and duplicate fields. Default input budgets match JSON.parse:
16 MiB, depth 64, one million entries per container and one million value nodes.
Issue retention remains 64 issues, 32 segments and 1024 path bytes per issue.

Source cursors retain real document borrows. Arrays stream child slices; objects
build a flat index of decoded keys and borrowed value slices. Last duplicate wins
before field validation. Known fields never construct a dynamic JsonValue tree.
Retained extras and an explicit JsonValue coercion parameter request final owned
dynamic data. Unescaped strings borrow during reading, then allocate final owned
storage; escaped strings decode into owned storage. Ordered branch retries may
revisit source and reconstruct rejected candidate outputs, without cloning or
stringifying an intermediate tree. Flat object indexes and candidate reconstruction
are allocation costs to measure at the S11 family milestone.

Native scalar, array, object, presence, defaults, literal/enum, ordered/discriminated
union and callback pipeline plans use the fused path. Coercion decodes its declared
owned input directly, then commits before invoking the conversion. Readonly
rejection permits branch fallback; failures after consumption do not. Commitment
in an earlier object field does not prevent an independent later union from trying
its readonly alternatives. Non-JSON coercion input types are checker errors.
Recursive coercion input shapes remain explicitly diagnosed pending the required
recursive/lazy schema tranche; they must not become a native compile failure.

Focused evidence: six public compiler/native cases, four source-cursor runtime
cases, one editor hover/completion/result/signature-help case and 26 workload contracts passed.
The final callback case was rerun after adding coercion input checking. Failed
intermediate runs are preserved: a generated brace, missing automatic JsonError/
JsonValue registration, and two fixture construction issues were repaired. A
retaining schema now registers its JSON dependency without requiring a JSON import.
Whole-project formatting passed; one new workload was formatted and its fingerprint
refreshed, then a second pass changed nothing. Native versus Node-after-tsc parity
passed at n=0,1,17,2000 for native_schema_json_fused, with zero timing samples.
The workload includes successful typed output and missing/unknown-field errors;
its oracle is seed + 4*n across two operations per iteration. Its TypeScript
reference is handwritten JSON.parse plus validation, not a Zod differential claim.

SQLite records use the prefix `tsrust-fused-section-` with suffixes native1 through
native4, workloads1, editor1, editor2, choices1, choices2, source1, format1, format2, parity1,
inputs1 and callbacks-final. Earlier acceptance and original failed records remain
unchanged. Installed consumers remain accepted S10; no S11 family timing or full
regression acceptance is claimed.

Chain defect found during this section (repaired by the checkpoint below): direct
`s.union([s.string(), s.string()]).min(3)` is rejected before receiver construction;
assigning the union to a variable before `.min(3)` works and its outer bound is
verified by the fused witness. Fix direct constructor chaining before S11 family
acceptance. Then continue recursive/lazy composition, collection schemas, richer
issue APIs and JSON Schema export, followed by the required family acceptance.


## Direct scalar union constraint chaining

Direct `s.union([s.string(), s.string()]).min(3)` and equivalent integer/number
chains now preserve the same scalar schema identity as the intermediate-variable
form. The checker derives an existing identity without analyzing branch arguments;
ordinary call checking still evaluates and consumes each argument exactly once.
The hint requires identical scalar outputs. It does not widen singleton, nominal
or callback pipeline identities merely to select a scalar constraint wrapper.

Focused native execution covers min/max/gt/lt, borrowed fused text validation and
owned validation, plus one branch-factory invocation and one limit invocation.
A separate rejection proves branch ownership is still transferred. One editor
case verifies the chained schema type, member hover and inferred result. All pass
across preserved runs. Whole-project formatting passed without changes. SQLite
records: `tsrust-schema-chain-{native1,native2,native3,editor1,format}.log`.
The initial runs also preserve an overly broad literal-widening experiment and a
rejection assertion that expected the word moved instead of diagnostic transferred.
No runtime behavior or benchmark workload changed; no timing run was needed.

Recursive schema API is awaiting the owner's decision: proposed
`s.recursive<Tree>(self => s.object({value:s.int(), children:s.array(self)}))`.
The proposed builder runs once; the recursion placeholder is reusable only within
the definition, and validation borrows the frozen uniquely owned plan. Do not treat
this proposal as approved until the pending question is answered. Independent
pinned Zod differential acceptance work can proceed meanwhile.


## Pinned differential corpus and JSON error encoding

The untimed differential runner in `benchmarks/runtime/verify-schema-differential.mjs`
compares the current public surface against exact Zod 4.1.12, locked with its
registry integrity. One generated native program covers 18 families and 171
observations across ordinary owned JSON validation and fused decoding. It compares
acceptance, successful JSON output and normalized field/index failure locations.
Typed coercion is exercised through fused decoding because its declared raw input
is an array rather than JsonValue. Each family includes successful and rejected
inputs. Reference adapters and intentional differences are recorded beside the
corpus: strict objects, Unicode code points, validated defaults, consuming ownership,
exact shared integer domain, error multiplicity and consuming union commitment.
Both outputs are normalized through JavaScript JSON serialization for comparison;
native negative zero remains preserved by native encoding. This normalization is
explicit evidence scope, not a change to native numeric semantics.

The initial corpus exposed missing JSON encoding for public validation-error data.
Encoding now handles source-constructible native data records, singleton literals
and nominal enums through their declared fields/scalar values. Opaque resources
and schema plans remain rejected. The native witness verifies structured issue
paths and borrowing the error again after encoding. JSON IntelliSense documents
the supported shapes. The original failed corpus and signed-zero comparison runs
remain in SQLite alongside successful checks under `tsrust-schema-zod-*.log`.

Run with the documented shared Cargo target, `CARGO_INCREMENTAL=0` and `TSRUST`:
`npm run verify:schema-differential` from `benchmarks/runtime`. The runner verifies
the pinned dependency, formats the generated corpus twice, records source hashes
and removes its temporary input after execution. This is correctness evidence only.
The affected JSON encoding suite passed all eight cases, including catalog examples;
the corpus contract suite passed both cases. Whole-project formatting changed no
files or fingerprints. Remaining S11 families must extend the corpus before full family acceptance; no
S11 timing, complete regression acceptance or consumer refresh is claimed here.


## Composite differential coverage

The pinned corpus now covers 24 families and 225 observations. Added array unions,
nested transforms, defaults followed by transforms, retained unknown fields,
readonly union fallback and cross-field predicates. Both owned JsonValue validation
and fused text decoding pass. Retained-output comparison observes the declared
count plus the number of extras; detailed retained contents remain covered by
native pipeline witnesses rather than claiming complete equality from that summary.

The first expanded run exposed a reference-harness error: nested Zod union issues
carry paths relative to the enclosing union. The adapter now concatenates those
prefixes recursively, including unknown-key paths. Three corpus contract tests
pass, including nested union path preservation. The original failure and corrected
run are retained in SQLite as `tsrust-schema-composite-{differential1,differential2,
contracts,format}.log`. This expands correctness evidence only; S11 family
acceptance and measurements remain open.

Collection representation is approved by the owner.
`s.record(valueSchema)` validates arbitrary string-key JSON objects into
`Map<string, T>`; `s.map(keySchema, valueSchema)` and `s.set(elementSchema)` validate
existing native collections and reject JSON text rather than choosing an implicit
wire representation. Map/Set key or element collisions introduced by transforms return validation
errors. Explicit array-to-Set conversions may intentionally deduplicate.


## Fused versus owned allocation probes

The JSON allocation harness now includes `schema_fused`, `schema_owned`,
`schema_fused_mismatch` and `schema_owned_mismatch`. Each constructs a fresh
scalar-array schema inside operation counters. Successful probes return the full
typed integer array; mismatch probes parse strings followed by one number, then
drop partial output and errors. This measures construction plus validation, not
reusable-plan validation alone. Fused probes call safeParseJson; owned probes build
JsonValue first and consume it through safeParse. Counterparts use checked
JSON.parse and direct typed serde_json decoding. Neither counterpart models all
native resource limits, and this is not Zod allocation evidence.

The original three fresh-process samples at sizes 0, 1 and 1000 are retained for
each of four probes, across native, direct Rust and Node. Native requested-byte
counters are distinct from Node post-GC heap deltas. Checksums agree, counters
are deterministic, and native/direct output disposal leaves zero live bytes.
At 1000 integers, native fused requested 16192 bytes and retained 8192; owned
parse-then-validate requested 97952 and retained 8192. Direct Rust requested 16352
and retained 8192. For 1000 strings followed by a mismatching number, fused
requested 55824 bytes versus owned 120896; both retained zero bytes after errors
were dropped. These narrow results support the direct typed path's allocation
benefit without claiming zero allocations or family-wide superiority.

Fifteen allocation adapter contract tests pass. Whole-project formatting expanded
the new fixtures. Emitted-source hashes changed, so the formatted probes were
executed again rather than assuming token preservation proved native equivalence.
Original and formatted records are preserved under `tsrust-schema-allocation-*`
in SQLite. The harness removes generated packages, native binaries and TypeScript
outputs on completion. Timing samples remain zero. Reusable plans, objects, unions
and broader failure profiles remain required for S11 family allocation review.


## Fused decoder boundary acceptance

A public native witness now verifies the documented 16 MiB byte limit, depth 64,
and one-million-node budget before any transform callback runs. Malformed Unicode
in an ignored field rejects before a missing-field default factory can execute.
An array with 81 invalid entries retains exactly 64 ordered issues, reports
truncation and keeps controlled observed-type summaries. A source-cursor witness
checks exact accepted limits, each adjacent rejected boundary, and duplicate-key
entry/node accounting before last-key-wins field selection. Both cases pass;
no public API or runtime behavior changed in this section.

Native source is formatted and checked for a stable round trip by the shared
fixture helper. Whole-project formatting also passes. SQLite preserves
`tsrust-schema-boundaries-{source,native1,native2,native3,native4,format}.log`.
The initial native fixtures were rejected for unreachable returns and consuming
owned fields through a narrowed union; the final witness uses supported read-only
field access. Those failures are retained, not represented as runtime defects.
S11 remains in progress; these are focused boundary checks, not full regressions
or performance acceptance.


## Reusable-plan allocation acceptance

Three additional probes build schemas before resetting allocation counters and
validate borrowed text through a borrowed plan: an integer array, a strict object
with an integer-array field, and an ordered array union whose first all-zero branch
rejects the positive fixture before the unrestricted branch succeeds. Every output
integer contributes to the checksum. Direct Rust and checked Node counterparts
produce the same output for these finite fixtures; they do not reproduce native
union speculation or all native resource budgets.

All nine cases (sizes 0, 1, 1000; three fresh-process samples per engine) pass
checksum, deterministic operation counters and disposal checks. At 1000 elements,
native requested bytes are 16192 for the array, 16648 for the object and 26304 for
the union. Each retains 8192 bytes of output capacity. `schemaRequestedBytes`
reports retained plan heap storage before operation counters: zero for the array
and object, 80 bytes for the union. It does not report cumulative construction
allocations. Output disposal preserves the plan baseline; dropping the input and
plan leaves zero additional live bytes. These results expose union speculation
cost without claiming a timed regression or zero-cost plan construction.

The allocation harness now emits a complete source/rows record to stderr after
each completed probe, in addition to the final JSON report. This permits resuming
remaining probes after a later failure without discarding successful measurements.
The array result is retained in `tsrust-schema-reusable-run3.log`; only object and
union reran in `tsrust-schema-reusable-results4.json`. Original failed runs preserve
a test syntax error, an overly broad reset-marker check, a missing field semicolon
and unavailable Serde derive support. The direct object baseline now uses the same
manual visitor approach as existing baselines. Fifteen existing adapter tests and
two reusable-adapter cases pass. Whole-project formatting precedes the final runs
and checkpoint. All `tsrust-schema-reusable-*` evidence is saved in SQLite.
No timing samples or full S11 acceptance are claimed.


## Collection implementation section

Approved constructors: `s.record(valueSchema)` for arbitrary string-key JSON
objects with `Map<string, T>` output; `s.map(keySchema, valueSchema)` and
`s.set(elementSchema)` for existing native collections. Map/Set do not accept
JSON text through safeParseJson. Record decoding retains the existing checked
JSON numeric and last-duplicate-key contracts. Builders own their child plans;
validation borrows the reusable parent and consumes existing owned input.

Transforms must not silently overwrite a Map entry or collapse Set elements.
A collision is a validation failure. Explicit array-to-Set conversion remains an
intentional operation outside this rule. Collection plan identities must preserve
input/output types and exact child plans through wrappers, objects, arrays and
unions, including callback exception effects and consumption commitment.

Focused collection acceptance: four compiler/native witnesses pass, including
catalog examples, consumed-input checks, nested record decoding, ordered rollback,
committed collisions and callback exceptions. One registry dependency test and one
editor inference/completion test pass. All 27 schema workload contract checks pass.
The first native runs exposed missing collection kind handling and trial-runtime
linking; both are repaired. All failed and passing logs are retained as evidence.

The matched `native_schema_collections` workload performs five operations per
iteration: fused record decoding, native Map validation, native Set validation,
Map collision rejection and Set collision rejection. Native and Node-after-tsc
benchmark drivers agree with the independent `seed + 8*n` oracle at 0, 1, 2, 17 and
2,000 iterations. These are correctness checks, not timing acceptance. The reference
is handwritten and makes no Zod allocation-equivalence claim.

Whole-project formatting preserves tokens and refreshes the new workload hash.
Formatted-source native witnesses execute the catalog examples. SQLite evidence
uses `tsrust-schema-collections-*` names, including failed runs and their repairs.
S11 full regressions, differential expansion and family measurements remain pending.


## Collection differential extension

The pinned Zod corpus now covers 27 families and 257 observations. New record,
nested-record and record-transform cases compare complete value snapshots after
validation, including transformed values, empty objects, invalid entries and
property/index error paths. Both owned JsonValue and fused source decoding pass.
The reference sorts property names to match canonical JSON object traversal;
these checks do not assert Map insertion-order equivalence or implicit JSON Map/Set
support. Native collection and collision semantics retain their separate witnesses.
Evidence: `tsrust-schema-record-differential*.log` in SQLite. No timing acceptance.

Cross-field diagnostic options are approved with a stronger typed-path requirement:
`.refine(predicate, { path: ['confirm'], message: 'Passwords must match' })`.
The predicate remains read-only. Paths must be checked against the inferred input
shape and offer IntelliSense completion; invalid paths must not be accepted as
unchecked strings. The owner also requires constructor-level messages, including
`s.int({ message: '...' })` and `s.object(fields, { message: '...' })`.
Multiple custom issues remain a separate API. Implementation is pending.


## S11 consumer refresh after collections

Release CLI and LSP rebuilt from `b4c0643a` using the shared Cargo target and
disabled incremental compilation. Atomically replaced both `~/.cargo/bin` binaries
and the workspace override at `target/editor/debug/tsrust-lsp`; SHA-256 equality
verified against the release outputs. Installed CLI checks the pipeline workload
and executes the collection workload with expected output `8`. Restarted the
repository editor server and inspected the open schema documents: the Problems
count fell from 104 (33 errors, 71 warnings) to 71 warnings and zero errors.
Visible remaining warnings are Rust diagnostics, including unused runtime helpers
and an irrefutable pattern. These warnings remain pending cleanup; they were not
suppressed or represented as resolved. Build log retained in SQLite.


Custom diagnostic implementation is in progress after `89f78752`. New public
witnesses cover constructor messages, reusable schemas, owned/fused validation,
relative nested paths and invalid segment rejection. A draft analyzer path resolver
walks declared fields, numeric array indices, optional values and unions without
accepting an unchecked field string. Runtime message storage, constructor wrapping,
refinement integration and editor path completion are still outstanding. These
witnesses have not run: section checks remain deferred until integration is complete.
No implementation acceptance or consumer refresh is claimed for these drafts.


Custom diagnostic drafts now include constructor option interception, uniquely
owned message wrappers, relative refinement paths in owned and fused emitters,
and public issue message storage. Child custom messages take precedence over an
outer constructor default. No-input-copy behavior remains unchanged; retained
error messages copy schema text only on failure. Default issue messages currently
use the stable error code. Chained builder handling, complete catalog/editor
integration, runtime budget witnesses and matched workload coverage remain pending.
No tests have run for this incomplete section, and no consumer was refreshed.
The owner's post-S11 cleanup requirement is recorded in the main roadmap before S12.


## Approved module-state direction

The owner approved combining program-owned module state with inferred borrowing
and explicit shared-state facilities for concurrency. Keep let/const binding rules
separate from cloning; no keyword-triggered implicit deep copies. Module storage
retains normal checked borrowing and parameter passing. Detailed initialization,
export and move-out rules still require implementation design and acceptance.
The motivating module-level schema declaration must be included in S11 usability
acceptance. Explicit concurrent shared-state facilities follow the thread/task work.

The owner favors requiring an explicit thread pool for user-created worker threads,
possibly configured at module scope, with workers borrowed and returned through the
pool. This is a preference for discussion, not settled API or lifecycle semantics.
Discuss pool configuration, local versus global ownership, bounded admission, job
settlement, cooperative cancellation and synchronized state during S12 design.


The owner has now prioritized globals first, including checker and IntelliSense,
then continuation of S11. Follow [the module-state plan](module-state-plan.md).
Thread-pool discussion is explicitly deferred until after async implementation.
Earlier suggestions for mandatory global pools, batch and state APIs remain
unapproved. Diagnostic-option drafts are saved at `8d47c02b` and remain unverified.


### Diagnostic section continuation after globals

Globals and their consumer refresh are checkpointed through `9f47a6bb`.
Diagnostic options remain unaccepted. Added unrun witnesses cover constructor
message constraint chains, child-message precedence, malformed constructor options,
issue-budget preservation and speculative-branch message rollback. In particular,
message wrappers must forward scalar constraint builders without copying the owned
plan or evaluating the receiver twice. Complete that forwarding and typed editor
path support before running the section checks. The installed consumers still
represent the verified globals checkpoint, not diagnostic-option acceptance.


### Diagnostic options: focused integration

Constructor messages and relative refinement paths now preserve scalar constraints,
object strip/retain policies, callback output types and structured construction
errors. Message expressions and constraint limits evaluate once in source order.
Non-string messages are rejected in constructors and refinements. Child custom
messages override outer fallback messages; speculative branch rollback discards
its messages. Issue-count and path budgets retain their existing behavior.

Editor completion uses checked schema output fields, including nested array paths
and common fields of union alternatives. Constructor/refinement signatures and
parameter help document diagnostic options; message-bearing scalar plans retain
constraint completion. See [schema diagnostics](../guides/schema-diagnostics.md).

Focused evidence under `tsrust-schema-diagnostics-*` includes six compiler/native
witnesses, four editor witnesses, seven isolated issue-runtime tests, two executable
catalog examples, 27 benchmark contract tests and ten native/Node-after-tsc parity
cells. Failed attempts and repairs are retained. The isolated runtime test wraps
the unchanged source in its module context. The updated pipeline workload retains
three operations per iteration and checksum `seed + 7*n`, now observing the custom
message. All parity runs have zero timing samples. Full S11 regressions, differential
coverage, measurements and remaining schema requirements are not accepted here.
