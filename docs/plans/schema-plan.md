# S11 schema validation

Status: implementation started after [S10 acceptance](../milestones/s10-json-acceptance.md).
Internal bounded issue collection and checked numeric plans have focused verification. No public
schema API, type inference, decoder fusion or performance acceptance is claimed.

## Required surface and coverage

The roadmap is the scope. Zod is a differential behavior reference, never a runtime
dependency or blanket compatibility target. Pin the actual reference package before
executing differential cases; current web documentation alone is not a version pin.

| Family | Required behavior | Status |
| --- | --- | --- |
| Scalar constructors | string, number, int, boolean, null; literal and enum checks | Pending |
| Scalar constraints | explicit string-length units; numeric inclusive/exclusive bounds; no implicit coercion | Pending |
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
