# S11 schema validation

Status: implementation started after [S10 acceptance](../milestones/s10-json-acceptance.md).
Only internal bounded issue collection is under focused verification. No public
schema API, type inference, decoder fusion or performance acceptance is claimed.

## Required surface and coverage

The roadmap is the scope. Zod is a differential behavior reference, never a runtime
dependency or blanket compatibility target. Pin the actual reference package before
executing differential cases; current web documentation alone is not a version pin.

| Family | Required behavior | Status |
| --- | --- | --- |
| Scalar constructors | string, number, int, boolean, null; literal and enum checks | Pending |
| Scalar constraints | explicit string-length units; numeric inclusive/exclusive bounds; no implicit coercion | Pending |
| Objects | inferred typed fields, explicit unknown-key policies, required/missing distinction | Default unknown-key policy awaiting owner decision |
| Arrays | inferred element output, indexed error paths, bounded partial construction | Pending |
| Optional/nullable/default | distinct missing/null behavior, documented default evaluation and ownership | Pending |
| Unions | ordered matching, bounded branch errors, correct rollback and output ownership | Pending |
| Discriminated unions | checked discriminator selection, inferred alternatives and useful errors | Pending |
| safeParse | typed Result, reusable immutable schema receiver, bounded redacted errors | Input ownership awaiting owner decision |
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

## Decisions pending

1. Recommend rejecting unknown object fields by default, with explicit stripping
   and retention policies. Rejection catches misspellings; stripping accommodates
   extra producer fields but discards data. The owner is choosing the default.
2. Recommend consuming existing owned input on both success and failure, moving
   reusable storage into successful output. Explicit clone retains the original.
   The alternative borrows input and builds an independent output copy. This
   question does not concern decoding borrowed JSON text.

Keep these choices out of implemented public signatures until answered. String
length units, default factories and public builder spellings need a concrete
contract review before their individual implementation gates. The existing checked
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
