# S11 schema validation

Status: implementation started after [S10 acceptance](../milestones/s10-json-acceptance.md).
Public string and boolean schemas have focused compiler, native and editor verification.
Internal bounded issues, consuming arrays and checked numeric plans are also verified.
Other constructors, composite source APIs, decoder fusion and family acceptance remain pending.

## Required surface and coverage

The roadmap is the scope. Zod is a differential behavior reference, never a runtime
dependency or blanket compatibility target. Pin the actual reference package before
executing differential cases; current web documentation alone is not a version pin.

| Family | Required behavior | Status |
| --- | --- | --- |
| Scalar constructors | string, number, int, boolean, null; literal and enum checks | Basic string/boolean/int/number/null API checked; literal/enum pending |
| Scalar constraints | explicit string-length units; numeric inclusive/exclusive bounds; no implicit coercion | String min/max and integer/number min/max/gt/lt integrated |
| Objects | inferred typed fields, explicit unknown-key policies, required/missing distinction | Approved: reject unknown keys by default |
| Arrays | inferred element output, indexed error paths, bounded partial construction | Pending |
| Optional/nullable/default | distinct missing/null behavior, documented default evaluation and ownership | Pending |
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
exposed, must use explicit alternatives. Default factories still need a concrete
contract review before their implementation gate. The existing checked
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

Inspection confirms the compiler annotation/type surface does not yet represent
literal-value types. Precise s.literal output and literal discriminators therefore
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
