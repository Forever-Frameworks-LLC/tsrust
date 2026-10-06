# Gate 150: checked array callback errors

Accepted on focused evidence. No timing or full regression was requested or run.

## Behavior

Checked callback failures now reach the surrounding catch/finally region for map,
filter, flatMap, some, every, find, findLast, findIndex, findLastIndex, reduce,
reduceRight, sort and toSorted. Traversal stops on the first reached error.
Successful callbacks preserve ordering, short-circuiting and existing ownership
rules. Readonly abstract callbacks use the readonly invocation contract.

Fallible transforms and predicates use explicit loops so an error can leave the
active typed completion region without crossing a Rust iterator closure. Pure
callback paths retain their existing adapters. Reductions and flatMap unwrap
registered callback Result transport within their existing explicit loops.

Fallible sorting compares a stable permutation of indices before applying swaps.
An error leaves the original receiver untouched and prevents further callback
calls. Successful sorting applies the permutation without copying owned payloads.
Copying toSorted retains its explicit copy semantics. This fallible path allocates
index storage; its performance remains unmeasured. Pure sorting keeps its existing
path.

## Evidence

The independent Node fixture passes. Original native red evidence contains Result
payloads used as values and predicates. A separate readonly abstract map witness
fails with E0277 before the invocation repair. Both red logs are retained.

The expanded fixture asserts all thirteen methods, original source validity,
fallible success, duplicate sorting values, owned strings, short-circuiting and
finally completion. Tests use std:testing assertions. An additional native test
covers checked errors through a readonly abstract callback.

Untimed matched benchmark verification passes: runtime/results/
2026-10-06T07-33-38-998Z-verify.json. The independent oracle retains thrown label,
finally marker, original order and sorted copy. Source contracts, registration,
Result-returning ABI guard, driver and profiler hooks are ready. Guard support is
restricted to the registered callback-error family, preserving rejection for
unknown fallible families. No measurements are claimed.

Final sources pass eight compiler suites, 34/34 checks: callback errors 2,
find 4, forEach 4, predicates 4, reduce 5, sort 4, transforms 4 and Set.forEach 7.
Editor exception checks pass 3/3, including formatted checked callbacks and catch
hover. Harness passes 377/377. Five consumers were refreshed atomically; hashes
are in target/editor/array-callback-errors-consumers.log. The final LSP refresh
includes the catalog changes. All gate commands are terminal.

The first harness pass exposed the Result workload ABI guard restriction. Its
failure is retained, and the final harness and explicit guard regression pass.
Original verification manifest is retained in archive/array-callback-errors-verification-start;
acceptance flags are updated afterwards with focused registration checked again. The catalog documents error
propagation for all thirteen existing methods; no new method signature is added.
