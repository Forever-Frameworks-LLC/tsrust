# Gate 128: own-property membership

Status: focused compiler/native, editor, documentation, consumer rebuild and matched benchmark acceptance complete. Full regressions and measurements are deferred.

## Behavior

`Object.hasOwn(value, key)` returns whether a supported value owns the selected property. Fixed record fields count even when their value is undefined. Inherited names do not count, and a field named hasOwnProperty does not interfere. Both arguments are required and evaluate once, left to right.

Dense arrays and tuples own length and canonical existing nonnegative integer positions. Strings own length and positions measured in UTF-16 code units. Numeric negative zero selects position zero; the textual names -0 and 01 do not select positions. Numbers and booleans have no supported own properties. Primitive keys convert to text; string keys borrow their existing storage, including primitive union alternatives.

Records, tuples, arrays, strings, numbers and booleans are supported receivers. Nullable and union receivers, functions, object/callable keys, symbols, sparse arrays and dynamic record names remain unsupported. The operation does not mutate or clone receiver data. A key expression cannot mutate a receiver while its read loan is active.

## Evidence

- The baseline unsupported-call failure is retained in `target/editor/object-has-own-baseline-red.log`.
- Three focused compiler/native checks pass in `object-has-own-native-current.log`. Executable checks use std:testing assertions, including present undefined fields, evaluation order, key conversion, Unicode positions, source reuse and primitive union keys. Rejection checks cover argument count, unsupported inputs and overlapping access. Adjacent Object.keys checks pass in `object-has-own-focused.log`.
- Current editor hover and Boolean result inference pass in `object-has-own-final-editor.log`. Complete catalog documentation and an assertion-based example are present in builtins.json. Native and independently adapted Node example executions pass with empty stdout in `object-has-own-example/verification.json`.
- Two runtime registration/oracle checks pass in `object-has-own-registration.log`. The matched pair retains nine property-query results per iteration. Its independent oracle uses declared record membership, dense array bounds and UTF-16 length, without calling Object.hasOwn. Guards bind current source hashes, query retention and the operation denominator; runtime and allocation-profile hooks fingerprint both helpers. Report categorization is registered.
- Current debug/compiler/LSP/editor consumers are rebuilt and refreshed; release build succeeds in `object-has-own-final-release.log`. Before-overwrite binaries are retained in `archive/object-has-own-consumers-start`.

Initial changed-source snapshots are retained in `archive/object-has-own-start`. No timing, allocation profiling or complete compiler/LSP regression is claimed. SQLite measurements remain unchanged.

## Final acceptance

All five matched native/Node-after-tsc cases pass untimed in `benchmarks/runtime/results/2026-10-05T10-19-08-715Z-verify.json` and `target/editor/object-has-own-verify-current.log`. Initial signature-format and TypeScript receiver-type preparation failures are preserved. The TypeScript counterpart uses a type-only cast for string receivers; runtime values and calls are unchanged. Source hashes and guards were refreshed after these repairs.

The runtime harness passes 339/339 in `object-has-own-harness.log`. After repairing the counterpart, all 76 affected registration, oracle and report checks pass in `object-has-own-affected-harness.log`; passed unrelated checks were retained. Hover, completion, two-argument parameter help and Boolean inference pass together in `object-has-own-editor-protocol.log`. Coverage records implementation acceptance separately from pending measurements.

Owner decisions: spread/rest syntax may explicitly request copies, including object spread; ordinary bindings still must not clone implicitly. Mixed-alternative default sorting stays unsupported; callers can specify ordering with a comparator. These decisions do not claim spread/rest implementation. Next gate is 129.
