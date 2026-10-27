# Variadic lexical path composition and byte order

This contract accepts ordinary string arguments through `std:path.join(...paths: string[]): string`, and adds `std:os.endianness(): string`. It continues the isolated branch from `bd209b4`; main integration remains separate.

## Joining behavior

Joining ignores empty segments, inserts the target separator between the remaining segments, and normalizes the composed path lexically. Zero arguments and entirely empty arguments return `.`. Later rooted segments do not discard earlier segments. Root syntax, dot/parent segments and trailing separators follow the accepted target normalization contract. Inputs remain unchanged; the result owns independent UTF-8 text. No filesystem or working-directory lookup occurs, and joining is not a containment check.

On Windows, the first nonempty segment establishes UNC intent only when it starts with exactly two separators followed by a non-separator. Otherwise repeated leading separators are reduced to one before normalization. Backslash-delimited components whose prefix before a colon is a reserved device name bypass normalization and replace forward slashes with backslashes. This preserves reference device and colon behavior rather than substituting Rust PathBuf semantics. All native strings are valid UTF-8; non-string arguments or spread elements are compiler diagnostics rather than runtime coercion or TypeError.

| Input | POSIX | Windows |
| --- | --- | --- |
| `join()` / `join("", "")` | `.` | `.` |
| `join("a", "..", "report")` | `report` | `report` |
| `join("/tmp", "/report")` | `/tmp/report` | `\tmp\report` |
| `join("//server", "share")` | `/server/share` | `\\server\share\` |
| `join("///server", "share")` | `/server/share` | `\server\share` |
| `join("é😊", "..", "文")` | `文` | `文` |

Official [Node joining documentation](https://nodejs.org/api/path.html#pathjoinpaths) establishes variadic strings, normalization, ignored empty paths and dot results. The pinned Node 24.11.1 embedded `lib/path.js` establishes the Windows UNC and reserved-device rules; executable reference comparisons are independent of the Rust implementation. The helper corpus contains 707 input vectors and 1,414 platform comparisons, including 1,024 segments and a 49,152-byte Unicode segment. Native execution is macOS ARM64 only. Portable Windows helper comparisons do not establish Windows native acceptance.

## Native rest bridge

`NativeParameter.rest` marks a final homogeneous element contract. This slice implements a single read-only string tail, reusable by another ordinary string variadic native API; it does not claim arbitrary mixed native varargs or optional/fixed prefix combinations. There is no public argument-count ceiling. Existing fixed/optional bindings explicitly retain `rest=false`; the registry accepts zero, 128 and the full representable argument-count range for joining.

Known native calls check each ordinary string under the existing read access contract. Native-only spread classification borrows homogeneous array elements, and existing once-only operand staging preserves left-to-right evaluation. Temporary owners stay live through native lowering and drop on argument failure. No implicit string clones or language ownership exceptions are introduced. Aliases, stored native values, direct/spread calls and concrete `Function<string, [...paths: string[]]>` annotations are exercised.

An abstract language Function rest parameter uses the already accepted consuming pack contract. Native forwarding receives that owned pack and lends its text to the lexical helper, then drops it. This adapter preserves the language contract; it does not introduce unsupported read-only rest annotation syntax. Known native calls retain their borrowed element behavior. Higher-order forwarding is verified separately from known-origin direct calls.

The implementation adds only native metadata, native-origin shape/argument handling, native-only spread classification/lowering, and per-export library helpers. It changes no grammar, general type inference, ownership rules or callable carrier representation. Shared source paths are listed in the handoff. A native cleanup witness measures repeated checked-program invocations after the process panic hook is installed once, proving balanced per-call allocations on both success and typed argument failure.

## Byte-order behavior

`endianness()` takes exactly zero arguments and returns `BE` for `target_endian="big"` or `LE` for `target_endian="little"`. No architecture-name inference or compiler-host substitution is allowed. An unsupported selector produces an explicit native compilation error. The result is constant during execution, including translation; it is an independent string value and mutates no external state. [Official Node documentation](https://nodejs.org/api/os.html#osendianness) describes the compiled CPU byte-order contract. The existing Rust target metadata corpus exercises both selectors and unsupported selection. Actual native execution returns `LE` on this host.

## Workloads and measurement scope

Three matched composition workloads cover ordinary rooted joins, empty/root behavior and 67-segment Unicode array spread. Independent expected outputs are spelled from case definitions and target separators rather than calling the tested path operation. Each has current formatted fingerprints, mutation controls, the existing string-array observer/ABI, and run/oracle/profile/report support. Input setup, variadic adaptation, normalization, retained output and observation are included; no isolated API latency or allocation-equivalence claim is made.

These joins perform actual path composition and are a meaningful bounded composition milestone. Only the three new join workloads are selected for execution-only native versus Node-after-tsc measurement under unchanged process, warmup and sample protocols. Previously accepted path and OS workloads are not retimed. Byte-order identity remains untimed because it can constant-fold; its matched registration and correctness support are complete, with timing deferred to a meaningful OS family workload.

Original measurements and allocation profiles are preserved before optimization. Joining reserves the checked composed byte length once, then uses the existing normalization result buffer. A focused native witness reduces six composition reallocations to zero for 67-segment Unicode input at both optimization levels. Only the three affected join workloads are rerun under unchanged protocols. Spread adapts references, not copied string contents. Final native average throughput is 2.03×, 1.22× and 3.23× Node-after-tsc for ordinary, empty/root and many-segment Unicode workloads respectively. These ratios describe matched full workload execution on this host, not isolated API latency or cross-platform results.
