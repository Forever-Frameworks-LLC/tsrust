# JSON namespace focused acceptance

Status: focused implementation accepted on macOS. S10 family acceptance, timing
and installed-consumer refresh remain pending.

All four approved methods are available: throwing JSON.parse/JSON.stringify and
Result-returning JSON.tryParse/JSON.tryStringify. Existing std:json imports retain
Results. Both paths use the same bounded backend and owned JsonValue representation.
Typed errors preserve fields through functions, callbacks and finally. Inputs are
borrowed; no implicit clone, shared owner or panic-based error conversion is added.

## Evidence

SQLite is canonical. Retained records include:

- `tsrust-json-shadow-3.log`: eight namespace tests plus dependency selection pass;
  includes native contract, custom limits, function values, six invalid-call cases,
  lexical shadowing and four catalog examples.
- `tsrust-json-dependency-capture.log`: nine dependency cases pass, including a
  captured local JSON receiver with no JSON runtime dependency.
- `tsrust-json-editor-3.log`: two focused editor tests cover four-method hover,
  signature help and completion, local receivers and public inferred type names.
- `tsrust-json-audit-verification.log`: all 404 inventory identities have one review.
- `tsrust-json-workload-contract.log`: two workload registration/guard/oracle tests.
- `tsrust-json-workload-smoke-3.log`: native and Node-after-tsc execution agree for
  imported and namespace workloads, five cases each, zero timing samples.
- `tsrust-json-order-green.log`: native bridges follow deterministic registry order.
- `tsrust-json-guide.log`: both public guide examples produce their expected output.

Failures are retained too. The benchmark smoke exposed nondeterministic native
bridge ordering between standalone emission and package export. A failing witness
preceded replacing hash-set iteration with ordered registry IDs. The rerun keeps
strict emitted/package source equality; no guard was weakened.

## Benchmark readiness and boundaries

native_json_namespace has a matched TypeScript counterpart, exact source hashes,
operation-placement guards, driver ABI support, automatic dependency preparation,
report registration and an independent `seed + n * 31` oracle. It exercises the
throwing namespace; native_json_roundtrip exercises the shared Result backend.
Both perform one parse, compact serialization and complete output comparison per
iteration. Inputs have ASCII text and already sorted keys to avoid incomparable
string-length/key-order semantics. Neither workload is a general compatibility
claim. Both remain explicitly unmeasured.

The current scope excludes unchecked JSON.parse<T>, revivers/replacers, dynamic
child operations and remaining typed/borrowed streaming work. Those stay in S10.
The required compiler source organization checkpoint, including stdlib strays,
is now [accepted](compiler-source-organization.md). Further S10 API expansion can
continue.
