# Node.js console compatibility

Reference: the repository's pinned Node.js v24.11.1, with redirected stdout/stderr and default console settings. This is parallel console work, separate from numbered language gates. Compatibility means tested observable behavior within Tsrust's printable value domain; complete JavaScript/Node host equivalence is not claimed.

## Implemented surface

The global console exposes log, info, debug, warn, error, table, assert, dir, dirxml, group, groupCollapsed, groupEnd, count, countReset, time, timeLog, timeEnd, clear, trace, profile, profileEnd and timeStamp. Methods return void. Arguments evaluate once left to right before output; primitive snapshots preserve source order and owned arguments remain borrowed through later evaluations. Later operands cannot mutate or consume earlier borrowed roots.

Logging aliases accept variadic int, number, boolean, string, null, undefined, array and record values. log/info/debug/dirxml use stdout; warn/error use stderr. Zero arguments print a blank line. Formatting recognizes %s/%d/%i/%f/%j/%o/%O/%c and %%, preserves missing placeholders and appends extra values. Default inspection uses depth two and at most 100 array elements. Top-level strings are raw; nested strings are quoted/escaped. Number formatting handles NaN, Infinity, negative zero and JavaScript decimal/exponent thresholds. %i uses integer-prefix parsing, %f decimal-prefix parsing, and %d whole-value numeric conversion. Diagnostic storage is released after output. Single string/int/boolean logging retains a direct borrowed/display path without diagnostic-tree allocation.

Inline object literals and empty arrays can be inferred specifically in console arguments, including nested printable fields. This does not enable general anonymous structural object types or heterogeneous arrays. Aggregate inspection conservatively uses ordinary storage layouts; unrelated fixed-storage specializations are retained.

console.table(data?, properties?) prints Node-style box borders. Arrays provide index rows; records provide named rows. Primitive rows use a Values column; structured rows expose fields or indices. The optional string[] column list preserves selection order and includes unknown columns with empty cells. An empty selection retains only indices. Primitive data prints normally; omitted data prints undefined. Cells use shallow quoted inspection.

assert uses supported-value truthiness and prints without throwing. group/groupCollapsed share two-space indentation across streams; groupEnd saturates at zero. count and countReset maintain independent text-labelled counters. dir supports depth, showHidden and colors; dirxml aliases log. Timers use a native monotonic clock, timeLog retains the timer and appends inspected extra data without placeholder substitution, and timeEnd removes it. clear emits ANSI clearing only for a terminal whose TERM is not dumb. Inspector-only profile/profileEnd/timeStamp evaluate arguments but do nothing without inspector integration. Destination write errors are ignored like Node's default console.

## Explicit remaining differences

This is not literal 1:1 Node equivalence. Keep these visible in editor documentation and coverage:

- Tsrust lacks callable inspection, heterogeneous arrays, sparse arrays, symbols, BigInt, cyclic objects, getters, custom inspectors and special JS object classes. Null/undefined values and their console defaults are implemented; see [value foundation evidence](value-types-and-namespaces.md).
- Long collection wrapping/column grouping, advanced Unicode display widths and the complete util.inspect options are not yet reproduced. Diagnostic trees have a native depth ceiling of 128; deeper descendants truncate. dir ignores options beyond depth/showHidden/colors and caps depth at 128.
- int retains its signed 64-bit domain; values outside JavaScript's safe integer range do not share identical numeric semantics. Native shortest floating-point digit selection can differ at rounding boundaries.
- Timings depend on the executable; durations are reported in milliseconds rounded to three fractional digits rather than reproducing every Node duration-unit choice. Missing/duplicate-label warnings are synchronous messages without Node process identifiers, stack suggestions or warning scheduling.
- trace captures native executable frames, not JavaScript frames. Frame paths, debug symbols and stack formatting differ. Inspector methods have no native Node inspector integration. Default automatic TTY coloring is not reproduced; dir colors:true explicitly supports ANSI colors.

## Verification

crates/tsrust-compiler/tests/console_output.rs runs common-value fixtures against the pinned Node executable and compares stdout and stderr byte for byte at native O0 and O3. It covers variadic streams, argument order, inline/nested objects, tables, selected/empty columns, control escaping, placeholder coercions, depth cutoffs, assertion truthiness, groups, counters, dir options including colors, dirxml and inspector no-ops. Separate native checks cover ownership rejection, release of temporary diagnostic allocations, timer structure, synchronous warnings and native trace output. Node is a test reference, not a runtime dependency of emitted executables.

crates/tsrust-lsp/tests/intelligence.rs verifies discovery, hover and signature help for the complete method surface, current compiler witnesses and executable documentation examples. Nondeterministic timer/trace output is verified separately rather than recorded as fixed example text.

Focused fixed-storage and nested-record checks remain part of acceptance. Full workspace regression and accumulated performance comparisons remain assigned to the five-gate checkpoint. Console I/O is outside execution-throughput comparisons: buffering, destination and OS scheduling dominate printing. No method ops/s or new allocation measurements are fabricated; preserve existing measurements and rerun affected contracts if lowering changes.
