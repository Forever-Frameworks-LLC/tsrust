# Globals/imports audit closure

Original accepted scope: public builtins, imported native types/functions, bundled
testing exports, receiver/static members, namespace headings and language-level
ownership/type primitives. All 399 discovered identities have exactly one review
in globals-and-imports-coverage.json. This closes the requested placement/naming
audit; it does not accept unimplemented JSON globals or the S10 feature milestone.

The later [JSON namespace addendum](globals-and-imports-json-namespace-audit.md)
extends the current manifest to 404 identities. Counts below describe the original
closure evidence.

## Independent discovery reconciliation

The review starts from compiler recognition as well as editor catalog entries:

- analysis.rs direct-call dispatch identifies String, Boolean, Number and clone;
  numeric-global rewriting and Result construction handle the other global forms.
  Namespace dispatch identifies String/Array/Map/Object/Number/Math/console.
- syntax.rs constructor parsing, numeric constant lookup and HTML wrapper mapping,
  type_syntax.rs primitive/callable ownership forms, and the specialized receiver
  checkers were reconciled with the corresponding per-symbol review tables.
- sets.rs/maps.rs, object_keys.rs/object_values.rs, array_entries.rs and the three
  imported collection checkers account for extracted member dispatch outside the
  main analyzer. Source rewrites account for public methods lowered to other calls.
- Native binding discovery uses compiled BINDINGS, including defaulted metadata,
  rather than a textual count of declarations. Exhaustive native type identity
  lookup accounts for all 18 types. Bundled testing exports are discovered from
  their source and private bridge restrictions are independently reviewed.
- LSP catalog loading/receiver family routing and global_builtin_at fallback were
  reviewed separately. The fallback added six initially missing headings/types;
  there is no additional callable call/apply/bind builtin family advertised by the
  editor or accepted by public compiler dispatch.

The independent identifier-candidate scan is retained as
`tsrust-api-closure-discovery.json` in SQLite, with source hashes and per-file
recognized/other names. Candidate strings are review prompts, not an automatic
parser or proof by themselves. Every remaining category was inspected:

| Candidate category | Disposition |
|---|---|
| int/number/string/boolean, Math/console | Included through editor fallback; primitive/namespace rows reviewed |
| code/message/offset/line/operation/osCode and JSON limit names | Record properties reviewed inside imported native type rows; compiled field metadata retained |
| depth/colors/showHidden | console.dir option properties reviewed with that method |
| K/T/V and parameter/place names | Generic placeholders or implementation locals, not additional global/imported APIs |
| Grammar words, true/false, control flow and declaration forms | Language syntax, not independent stdlib operations; boolean and type/operator contracts covered where relevant |
| read/mutate/consume/owned/once internal access spellings | Internal modes or retired-syntax diagnostics; public wrappers are ReadOnly/Mutable/Owned/Once |
| Diagnostic codes and test-fixture identifiers | Not callable APIs; checked against surrounding source rather than guessed from names |
| Names containing # | Compiler-generated internal projection/iteration slots; source identifiers cannot expose them |
| containsValue and other testing bridges | Explicit private testing-runtime review; module access restriction remains tested |

Per-family semantic reviews reconcile signatures, defaults/errors, receiver and
argument access, clone/transfer effects, backend selection and disposition. Native
runtime and editor example checks support those reviews. Inventory identity
coverage alone is not used as semantic evidence.

## Decisions and corrections

Retain familiar global numeric/collection/utility names and receiver methods.
Keep native services, resources, specialized collections and testing imported.
Keep ownership/type primitives language-level. No silent removals, mass renames or
shared-ownership changes. Imported JSON parse/stringify keep their Result contract.

Two contradictions in IntelliSense were corrected with failing editor witnesses
and native behavior checks: countReset's parameter text omitted its non-string
exception; array.at incorrectly prohibited nullable element unions. Runtime
behavior did not change. Other reviewed differences are intentional and documented,
including indexing units, array copying, void mutators and numeric rounding.

Dependency selection currently works through checked types/operations and generated
helper families. The audit does not promise per-method binary elimination. No user
must pass an --extern flag. Focused JSON dependency checks confirm ordinary programs,
a local JSON record and error-type-only use do not select the JSON module, while
an actual imported parse call does. The approved globals must preserve and extend
these checks during implementation.

## Next implementation acceptance

Implement JSON.parse/tryParse and stringify/tryStringify over the existing backend.
Throwing paths must expose structured JsonError; Result paths preserve imported
semantics. Verify shadowing, typed catch propagation, identical limits/errors,
cleanup, IntelliSense, automatic dependency selection, executable examples and
matched workload support. Keep S10 complete regressions, measurements and consumer
refresh separate from implementation acceptance.

Closure evidence is stored in SQLite as `tsrust-api-closure-*`. The coverage verifier
passes against a freshly compiled 399-row inventory. Earlier slice checks remain
preserved; no full matrix or new timing was repeated solely to close this audit.
