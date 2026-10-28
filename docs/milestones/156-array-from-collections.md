# Gate 156: Array.from collection sources

Array.from now accepts typed Sets and Maps, alongside arrays and strings. Unmapped
calls explicitly copy Set values or Map key/value tuples into independent arrays.
Mapping borrows each item and its zero-based index. Map tuple inputs can contain
noncloneable values; mapping does not create an intermediate entries() snapshot.

## Implementation and repaired defects

Collection copies reuse checked values()/entries() operations. Mapped calls lower
to a private traversal marker and the established array callback checker/emitter.
Map mapping borrows stored tuple records through the gate154 entry representation.
Native copying recursively copies owned descendants; it is a deliberate native
variant, not a JavaScript shallow aliasing guarantee.

All Array.from mappers now receive exactly two supplied positions. The prior array
lowering used map()'s three-position tuple, supplying an unexpected source array to
defaults/rest packs. Normal omitted/default/rest behavior works with array, string,
Set and Map sources. Required extra positions remain rejected. Existing mutable
stored-callable array mapping is preserved; an initial regression was reproduced
against the gate155 consumer and corrected in the focused native fixture.

Editor results recognize collection copies, mapped output types and grouped union
arrays. Hover and parameter help show the checked result type. The Array.from
catalog entry and Array namespace summary document all sources, arguments, result
ownership, errors, examples and native differences. Syntax guide: docs/guides/language/array-from.md.

## Focused evidence

- Red CLI check rejects collection inputs. Initial Node fixture passes; expanded
  native independent-copy checks deliberately differ from shallow JS references.
  The adapted Node fixture changes only that copy to clone(Array.from(records)).
- Final assertion fixture covers ordered copies, source independence, borrowed
  noncloneable values, tuple parameter patterns, abstract readonly mappers, unions,
  empty input, nested constructors, callback failures/finally and source-before-
  mapper factory evaluation. Stored-callable array behavior matches the old consumer.
- Compiler: 32 distinct focused checks across preserved runs: array_from 3,
  array_from_collections 2, array_callback_errors 2, map_iteration 2, HOF 6,
  map_for_each 2, set_for_each 7 and type_syntax 8. Final affected groups and expanded
  collection fixture pass after repair; unchanged canaries were not repeated.
- Editor: array_from_collections 2 (including union result types), existing
  Array.from intelligence check 1 and map_iteration 1 pass. Whitespace in tuple
  labels is normalized in assertions; type structure is checked unchanged.
- All six catalog examples match native and Node output. An additional assertion
  program verifies default mapper arguments on all four source forms.
- Harness 383/383. Untimed native/node-tsc verification passes for the new workload
  and existing array_from_map_int/array_from_map_string:
  runtime/results/2026-10-06T09-17-22-901Z-verify.json.
- Workload checked before atomic refresh of all five consumers. Hashes:
  target/editor/array-from-collections-consumers.log. Pre-acceptance manifest saved.

Sources, previous consumers and records have exact archive snapshots. Logs remain
under target/editor/array-from-collections-*.log and array-from-defaults-*.log.
Initial failures (required-position diagnostic wording, unreachable test returns,
shallow-copy Node expectation and callable lending) are retained. Fixture failures
were corrected without weakening ownership or completion assertions.

Matched TypeScript source, independent array-based oracle, fingerprints, source
contracts, registration and all runtime/profile/report hooks are complete. No full
regression or timing; collection measurements remain pending. Arbitrary iterator
objects, array-like records, spread arguments and thisArg remain unsupported.
