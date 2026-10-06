# Gate 153: independent Set algebra results

Accepted on focused evidence. No timing or full regression.

## Behavior

union, intersection, difference and symmetricDifference take one typed Set and
return an independent Set. Both inputs borrow, and selected owned keys explicitly
copy. Ordinary bindings/access do not implicitly clone. Union and symmetric
difference infer a joined key type; intersection and difference preserve the
receiver key type. Set-like records remain unsupported.

Union keeps receiver order and appends new other keys. Difference keeps unshared
receiver keys. Symmetric difference keeps unshared receiver keys followed by
unshared other keys. Intersection follows the smaller Set's order, with ties using
receiver order. It copies matching receiver keys so result typing is preserved
when the other key type differs.

Existing numeric and enum key rules apply. Large int/number overlap can lose
precision. A native-only red witness exposes a smaller-other intersection path
retaining only the first of multiple exact receiver keys matching a rounded
number. Borrowed matches_with now yields every matching key, retaining all exact
receiver values. Ordinary one-to-one key matches keep Node insertion order.

## Implementation and evidence

New algebra analysis and emission helpers stay outside recursive dispatch.
Generic result Set types materialize before resolution. Pure result hints avoid
pretending joined keys retain receiver type; syntactic receiver recognition keeps
nested Set algebra/member calls checked correctly. Runtime matching borrows
existing hash buckets without copying lookup keys. Only selected result keys copy.

Independent pinned Node passes expanded assertions after adapting only values()
snapshots to Array.from. Original compiler red rejects unsupported union. Native
checks cover four methods, smaller/tied intersection order, owned text, source
preservation, result independence, joined scalar/union keys, chained calls,
evaluation order, empty/self and NaN/zero. Static checks reject wrong arity/type,
wrong result context and conflicting receiver mutation during argument evaluation.
A separate native-only fixture covers exact integer overlap beyond Node precision.

The coherent compiler batch passes 25 checks: algebra 2, relations 3, Set 6,
higher-order function values 6 and type syntax 8. After the native precision repair,
affected algebra/relations rerun (3+3 checks), making 26 distinct focused checks.
Unchanged parser/type canaries are not repeated for a lookup-only runtime repair.

Editor work fixes pre-existing Set/Map signature substitution: union array types
need parentheses, and injected enum/type names must not be substituted a second
time. Token substitution preserves names such as ValueKey and independent OtherKey.
The red logs retain incorrect int|string[] signatures. The focused editor batch
covers algebra/result typing 2, Set catalog/editor 4, relations 1, Map 4 and Map
snapshots 1 (12 checks). Seventeen catalog examples match final installed-consumer native execution and
adapted independent Node.

Matched workload support includes typed sources, a scoped TypeScript declaration
reference, snapshot adaptations, independent six-field result oracle, fingerprints,
denominator/source contracts, registrations and all runtime/profiler/report hooks.
Owned-key independence is observed after source mutation. Workload check passes
before consumer refresh. Five consumers are refreshed atomically; hashes are in
 target/editor/set-algebra-consumers.log. Harness passes 380/380. Untimed matched
verification passes five cases per native/node-tsc engine:
runtime/results/2026-10-06T08-19-19-751Z-verify.json. All gate commands are terminal.
Original verification manifest remains in archive/set-algebra-verification-start;
accepted flags follow verification with final focused registration checked again.
Performance remains unmeasured.
