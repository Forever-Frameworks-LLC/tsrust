# S11 schema validation

Status: implementation started after [S10 acceptance](../milestones/s10-json-acceptance.md).
Public scalar schemas, constraints, nested arrays and required-field object schemas
have focused compiler, native and editor verification. Composite schemas accept
matching owned input or JsonValue. Reusable default factories are integrated.
Literal/enum schemas, remaining composite APIs, fused decoding and family acceptance
remain pending.

## Required surface and coverage

The roadmap is the scope. Zod is a differential behavior reference, never a runtime
dependency or blanket compatibility target. Pin the actual reference package before
executing differential cases; current web documentation alone is not a version pin.

| Family | Required behavior | Status |
| --- | --- | --- |
| Scalar constructors | string, number, int, boolean, null; literal and enum checks | Basic string/boolean/int/number/null API checked; literal/enum pending |
| Scalar constraints | explicit string-length units; numeric inclusive/exclusive bounds; no implicit coercion | String min/max and integer/number min/max/gt/lt integrated |
| Objects | inferred typed fields, explicit unknown-key policies, required/missing distinction | Required typed fields, inferred output, named output annotations, nested objects/arrays and default rejection integrated; explicit strip/retain integrated |
| Arrays | inferred element output, indexed error paths, bounded partial construction | Scalar/nested/object arrays integrated for matching owned and JSON input, plus checked int-to-number arrays; other composite elements pending |
| Optional/nullable/default | distinct missing/null behavior, documented default evaluation and ownership | Optional/nullable and reusable default factories integrated for owned/JSON object fields; S11 family acceptance pending |
| Unions | ordered matching, bounded branch errors, correct rollback and output ownership | Pending |
| Discriminated unions | checked discriminator selection, inferred alternatives and useful errors | Pending |
| safeParse | typed Result, reusable immutable schema receiver, bounded redacted errors | Approved: consume owned input on success and failure |
| Refinements and transforms | explicit input/output types, validation order, callback ownership and failure cleanup | Second tranche, still required |
| Preprocessing and cross-field checks | declared order, explicit coercion, structured issue paths | Second tranche, still required |
| Recursive/lazy and composition | finite plan construction, guarded recursion, no per-request plan rebuilding | Second tranche, still required |
| Record/map/set schemas | collection ownership, key semantics, inferred output and bounded errors | Second tranche, still required |
| JSON Schema export | explicitly reject unrepresentable checks/transforms rather than silently omit them | Second tranche, still required |
| Async refinements | approved task semantics and cancellation/ownership checks | Follows S14 |
| Regex/date/format validators | corresponding verified runtime semantics | Follow their dependency gates |

Do not substitute dynamic JsonValue output for inferred typed output. JSON text
validation must construct typed output directly through a fused decoder, without
an intermediate dynamic tree or stringify/reparse. Ordinary in-memory validation
and text decoding need separate benchmark scopes. Reusable runtime schema values
must work alongside compiler-specialized literal plans, with explicit ownership.

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
exact coercion arguments remain open.

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
The precise arguments and placement of `.coerce` remain to be settled before that
implementation gate. Do not treat readonly default factories
as a decision to remove mutation-capable transformation pipelines.

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

Public literal and enum schemas still require checked singleton type identities.
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

This remains prerequisite work, not acceptance of public `s.literal`, enum schemas,
or discriminated unions. Before exposing those APIs, complete the literal type
integration audit: scalar-to-union widening and union remapping, scalar conversion
and observation dispatch, indexed/iterated strings, collection key and element
contracts, contextual inference, and remaining wildcard type matches. Add focused
regressions for each supported boundary; reject unsupported cases in the checker
rather than allowing native failures. Preserve exact output types throughout schema
chains and projected editor facts.

Checked evidence: 10 focused compiler/native cases, two numeric identity unit
cases, two editor cases, and six affected existing union regression cases. These
are bounded checks, not full compiler/LSP acceptance. Evidence prefix:
`tsrust-literal-*` in SQLite. Public schema docs/examples, matched workload support,
full family regressions and measurements remain required. No timing is claimed.
