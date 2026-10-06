# Gate 152: borrowed Set relation predicates

Accepted on focused evidence. No timing or full regression.

## Behavior

isSubsetOf, isSupersetOf and isDisjointFrom accept exactly one Set and return
boolean. They borrow both collections and their owned keys without cloning or
mutation. Different currently supported key types are accepted, including
union/scalar overlap. The usual empty-set and self-set identities apply.

Lookup uses existing hash buckets with a supplied borrowed cross-type comparator.
Scalar/union combinations use the established native key rules: number
SameValueZero, exact int/int equality, compatible numeric alternatives and enum
type identity. Large int-to-number comparisons can lose precision. No cardinality
shortcut is applied because this cross-type matching may not be one-to-one for
large native integers. Set-like records are not accepted.

The new analysis and emission helpers stay outside recursive dispatch. Receiver
is evaluated before the argument and remains loaned during argument evaluation.
Wrong arity, non-Set arguments, incompatible result context and conflicting
receiver mutation during argument evaluation are static errors.

## Evidence

Compiler integration and installed compiler check reject unsupported isSubsetOf
before implementation. Exact source snapshots are in archive/set-relations-compiler-start.
Independent pinned Node passes the expanded assertion fixture after adapting only
values() snapshots to Array.from. Three predicates themselves are unchanged.
It covers empty/self cases, scalar/string/union keys, numeric overlap, NaN/zero,
enums, source preservation and ordered receiver/argument factory evaluation.

The coherent native batch passes Set relations 2/2, Set entries 2/2, Map.forEach
2/2 and Set 6/6 (12 checks). A separate native-only precision/enum-identity
witness passes (13 checks total); it distinguishes current native limits from Node's number
and erased enum representations. Editor checks pass 6/6: relation help/completion
1, Set catalog/editor 4 and Set.forEach 1. Thirteen complete catalog examples
match current native execution and adapted independent Node.

Matched workload support includes sources, independent six-field result oracle,
source fingerprints/denominator checks, registration and runner/profiler/report
hooks. Its contract catches a corrupted TypeScript method name caused by global
text replacement; token replacement repairs it before verification. Workload
check passes before five consumers refresh atomically; hashes are in
target/editor/set-relations-consumers.log. Harness passes 379/379. Final untimed native/node-tsc verification passes five
cases per engine: runtime/results/2026-10-06T07-54-16-588Z-verify.json. The first
verification failed because the pinned compiler configuration did not load Set
relation declarations. A scoped esnext.collection reference in the counterpart
fixes that; the target configuration stays unchanged. Focused registration and
matched verification pass after this final support change. The earlier complete
harness remains preserved; it was not repeated for the declaration-only change.

All gate commands are terminal. Original verification manifest remains in
archive/set-relations-verification-start; accepted flags follow verification.
Performance remains unmeasured.
