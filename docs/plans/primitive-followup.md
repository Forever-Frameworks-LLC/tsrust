# Primitive follow-up

[Native string contract](../guides/language/string-semantics.md) · [Foundation audit](foundation-audit.md) · [Benchmarks](../../BENCHMARKS.md)

The user selected TypeScript-style syntax with Rust-native behavior and costs. The active work replaces JavaScript UTF-16 compatibility with standard Rust UTF-8 strings and removes the obsolete code from the crates. Earlier tuning, experiments, proof records, and measurements remain [archived](../../archive/utf16-strings-2026-10-01/README.md).

Stable integer and boolean array literals use ordinary Rust `[T; N]` under an all-use proof. Indexed writes, independent clones, and proved calls preserve fixed storage; resizing, escaping, and exhausted budgets retain growable storage. This accepted array work continues through the string migration.

String migration must establish the native byte/scalar contract, complete compiler acceptance, and matching cross-runtime comparisons before the next implementation gate. It does not require emulating JavaScript surrogate behavior or winning every historical JavaScript measurement. Use native standard methods first; add optimization only for demonstrated costs with preserved semantics.

The [compiler migration proof](../../benchmarks/runtime/diagnostics/2026-10-01T23-05-37-467603Z-native-utf8-compiler-migration.json) records 475 passing integration tests, formatting, Clippy, release build, and documentation checks. It verifies removal of obsolete encoding/cache symbols from all compiler sources and archives their original versions outside the crates. The generated string support is 65 lines of standard Rust helpers. The [fresh 50-workload/eight-runtime timing run](../../benchmarks/runtime/results/2026-10-01T23-20-28-895Z.json) is complete; the [50 native allocation profiles](../../benchmarks/runtime/diagnostics/2026-10-01T23-21-05-187Z-native-allocations.json) and [eight primitive memory probes](../../benchmarks/runtime/results/2026-10-01T23-21-47-473Z-allocations.json) are complete. Historical records remain unchanged.

In this run, native ASCII trim/lower averages 53,207,868 workload iterations/s, compared with Node after tsc at 19,407,239. Dynamic normalization averages 42,644,764 versus 7,602,528. Dynamic owned string cloning averages 59,536,717 versus 316,323,455: Rust copies owned string bytes, while JavaScript may share immutable string storage. This is an explicit ownership cost, not a reason to restore the retired encoding runtime. Byte/scalar Unicode counterparts include their documented representation costs; their rankings do not establish general JavaScript string performance.

The [final runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-01T23-22-32-000Z-native-utf8-current50-proof.json) verifies all 50 timed/profile emissions against the same compiler inputs, 2,000 independent engine correctness checks, 37,200 timed batches, and 150 native plus 48 memory samples with complete disposal. All 263 pre-refresh raw records remain byte-identical. All 40 benchmark tool tests pass. At 100,000 elements, retained requested native array storage is 1 MiB for integers, 128 KiB for booleans, and 3 MiB for borrowed string values; the scalar and scalar-field record probes require no heap storage. These are requested storage measurements, not total memory footprints.

Native leads Node after tsc on 46 of 50 rows and the fastest scripting runtime on 44. The remaining measured gaps are below; this is a comparison snapshot, not an engine-win acceptance requirement. The clone and long ASCII construction differences reflect owned copying versus potentially shared immutable JavaScript strings. Unicode byte/scalar rows use their disclosed representation helpers and cannot be compared with historical UTF-16 rows.

| Workload | Native M iterations/s | Node after tsc M/s | Fastest scripting M/s | Native / Node | Native / fastest |
| --- | ---: | ---: | ---: | ---: | ---: |
| dynamic_string_clone | 59.537 | 316.323 | 386.180 (ts-node-transpile) | 0.188 | 0.154 |
| boolean_array_includes | 105.558 | 109.958 | 111.990 (deno) | 0.960 | 0.943 |
| boolean_array_index_of | 118.978 | 119.662 | 122.982 (deno) | 0.994 | 0.967 |
| string_array_index_of | 31.641 | 23.269 | 39.420 (bun) | 1.360 | 0.803 |
| string_slice_ascii | 114.310 | 85.853 | 126.859 (bun) | 1.331 | 0.901 |
| string_construct_slice_ascii_long | 14.588 | 69.956 | 94.188 (deno) | 0.209 | 0.155 |

The [two new string-comparison measurements](../../benchmarks/runtime/results/2026-10-01T23-37-48-752Z.json) extend the same UTF-8 epoch without rerunning or replacing the preceding 50 rows. Exact equality averages 348.378 million native workload iterations/s versus 215.968 million for Node after tsc; ASCII ordering averages 71.475 million versus 64.727 million. Equality performs two comparisons per iteration, ordering four. Local array setup explicitly clones the borrowed input into owned Rust storage; JavaScript may share immutable strings. Unicode ordering differences are correctness coverage and are excluded from the timed ASCII case.

The [comparison runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-01T23-38-46-000Z-native-string-operators-runtime-proof.json) validates 80 new independent engine cases, 1,488 timed batches, and six native allocation samples with zero final live bytes. Each invocation allocates twice during setup, with no reallocations or retained output storage: peaks are 170 bytes for equality and 173 for ordering. All 270 preceding raw records and all previous 50 workload source pairs remain byte-identical. The report now contains 52 independently dated rows; it does not claim a combined 52-workload run, and the earlier eight memory probes retain their actual dates.

The [uppercase comparison run](../../benchmarks/runtime/results/2026-10-01T23-55-50-833Z.json) adds two separately dated rows. Literal ASCII uppercase averages 70.368 million native iterations/s versus 45.605 million for Node after tsc. Dynamic Unicode uppercase averages 13.425 million versus 301.223 million. Each iteration retains an actual uppercase result from the standard method; Rust constructs owned output, while JavaScript may share immutable strings. The latter gap is a measured standard-method cost, with no tuning or cross-version Unicode claim. The independent oracle covers only explicitly shared mappings.

The [uppercase runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-01T23-57-07-000Z-native-uppercase-runtime-proof.json) records 80 engine correctness checks, 1,488 timed batches, and six fully disposed native samples. Each 20,000-result invocation performs 20,001 allocations and 12 reallocations; retained requested storage is 1,019,767 bytes for the literal case and 1,226,432 bytes for Unicode, fully released after output disposal. It preserves all 277 preceding raw records and all prior 52 workload pairs. Two preparation failures produced no execution measurements: example-array length needed a named local, and external NUL could not pass through argv. Those concrete repairs are recorded; literal and independent correctness cases preserve NUL coverage. The grouped report now has 54 individually dated workload rows and the previous eight memory probes, without an invented combined run.

The [three reverse measurements](../../benchmarks/runtime/results/2026-10-02T00-13-46-688Z.json) add execution-only rows: fixed integers average 30.601 million native reversals/s versus 5.004 million for Node after tsc; growable integers average 21.675 million versus 4.745 million; owned strings average 52.800 million versus 7.850 million. Every iteration observes the reversed order through a position-sensitive checksum. Numeric cases include local array setup each iteration; the owned-string case constructs its five strings once and includes content-equality observers. These are complete workload rates, and reverse returns void rather than JavaScript's array identity.

The [reverse runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T00-15-08-000Z-native-array-reverse-runtime-proof.json) records 120 independent engine checks, 2,232 execution batches, and nine fully disposed native samples. Fixed arrays allocate zero bytes. Growable arrays allocate 20,000 times with a 256-byte peak; owned-string setup allocates six times with a 147-byte operation peak, excluding the separately counted 11-byte external input. All three retain zero output bytes and perform no reallocations. All 284 prior raw records and 54 prior workload pairs remain exact. The grouped report contains 57 independently dated rows; startup, compilation, and the eight primitive memory probes retain their genuine earlier baselines. Execution-only adds no auxiliary measurements. One unsupported else-if fixture failed during preparation before any execution timing; equivalent nested conditionals repaired it without changing the compiler or measured operation.

The [eight standard string-search measurements](../../benchmarks/runtime/results/2026-10-02T00-34-53-278Z.json) extend the report to 65 separately dated rows. Native average rates below include one real search per iteration and the ordered observer. Unicode Node counterparts include coordinate-map setup once per workload call, followed by their built-in search and position/result lookup; these short-input rows are not isolated search costs.

| Method | Native ASCII M searches/s | Node ASCII M/s | Native Unicode M/s | Node Unicode M/s |
| --- | ---: | ---: | ---: | ---: |
| startsWith | 251.204 | 35.228 | 253.779 | 32.504 |
| endsWith | 243.968 | 37.518 | 251.847 | 35.073 |
| includes | 122.375 | 33.942 | 97.042 | 29.466 |
| indexOf | 59.439 | 30.918 | 55.268 | 26.353 |

The [search runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T00-37-25-000Z-native-string-search-runtime-proof.json) records 320 independent engine checks, 5,952 execution batches, 24 native samples with zero operation allocations/reallocations/peak bytes, and complete input/output disposal. External inputs occupy 13 ASCII or 23 Unicode bytes outside the operation counters. The actual timed positions are defaults plus explicit bytes 0, 3, 8, 11, 16, 22, and 999, clamped to input length; alternating selectors leave negative/other query-table entries to the separate semantic tests. All eight rows lead Node after tsc; ASCII indexOf trails Deno's 69.364 million searches/s. All 290 prior raw records and 57 prior workload source pairs remain exact. Execution-only adds no startup/compilation metrics, and the previous eight memory probes retain their genuine dates. A direct-read fixture binding repair happened before any timing; the compiler and operation remained unchanged.

The [twelve control-flow and primitive comparisons](../../benchmarks/runtime/results/2026-10-02T01-08-44-019Z.json) extend the same epoch to 77 separately dated rows. Rates include operand selection, observers, and loop overhead. Eight primitive rows perform one target operator per iteration; short-circuit is explicitly compound and observes tagged right-hand-side calls. Control rows observe intermediate state, including continue/update and break behavior. These are whole workload iterations, not isolated instruction latency.

| Workload | Native M iterations/s | Node after tsc M/s | Native / fastest scripting |
| --- | ---: | ---: | ---: |
| classic_for_control | 52.006 | 7.882 | 2.764 |
| do_while_control | 51.116 | 7.932 | 2.592 |
| else_if_control | 224.060 | 39.717 | 1.501 |
| int_divide | 239.414 | 36.982 | 2.841 |
| int_negate | 262.927 | 43.130 | 3.137 |
| int_less_equal | 279.161 | 48.516 | 3.170 |
| int_greater_equal | 279.073 | 48.964 | 3.175 |
| int_not_equal | 278.881 | 48.789 | 3.171 |
| boolean_equal | 275.786 | 42.281 | 3.189 |
| boolean_not_equal | 277.600 | 43.492 | 3.182 |
| boolean_or | 284.192 | 49.140 | 3.212 |
| boolean_short_circuit | 271.604 | 26.827 | 3.706 |

The [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T01-09-41-000Z-native-control-flow-primitive-runtime-proof.json) records 480 independent engine correctness checks, 8,928 execution batches, and 36 native samples with zero allocations, reallocations, retained/peak requested bytes, or final live bytes. All 188 frozen material inputs, 296 preceding raw records, and 65 prior workload source pairs remain exact. Execution-only adds no startup or compilation measurements; the earlier eight memory probes retain their actual dates. One helper-declaration ordering failure occurred during preparation before execution timing; the repair moved declarations while preserving both function bodies and calls byte-for-byte. No compiler regression or historical workload measurements were repeated.

The [sixteen Number comparisons](../../benchmarks/runtime/results/2026-10-02T01-33-56-966Z.json) add six arithmetic operators, six IEEE comparisons, three explicit conversions, and one compound number-array case. Native arithmetic averages 997.924–1,016.600 million whole iterations/s except remainder at 341.628 million; comparisons average 233.585–256.273 million. Integer conversion averages 1,414.475 million, boolean conversion 1,466.755 million, decimal parsing 103.006 million, and the compound array case 12.897 million. All sixteen lead Node after tsc in this snapshot. Boolean conversion trails Bun’s 2,042.027 million, and the compound array case trails Bun’s 15.689 million. These rates include operand selection, setup, accumulation/digest, and loops; they do not isolate operator latency.

The [Number runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T01-34-50-000Z-native-number-runtime-proof.json) records 640 independent engine cases, 11,904 execution batches, O0/O3 IEEE adapter validation, and exact floating-result tokens checked on every case and measured process. Signed zeros and infinity signs remain distinct; NaN payload/sign is canonicalized. All 332 frozen material inputs, 302 preceding raw records, and 77 prior workload source pairs remain exact. All sixteen timed/profile emissions match. The grouped report now contains 93 individually dated measured rows; no combined 93-workload run or new startup/compile measurements are claimed.

The [48 native profiles](../../benchmarks/runtime/diagnostics/2026-10-02T01-34-23-961Z-native-allocations.json) fully dispose all outputs. Arithmetic table setup allocates twice with a 112-byte peak; comparison setup allocates once with a 64-byte peak. Conversion/parsing rows allocate no operation storage. The 2,000-iteration compound array case allocates 4,001 times and reallocates 2,012 times, retaining 131,072 requested bytes with a 131,152-byte peak before complete disposal. The [new scalar Number memory probe](../../benchmarks/runtime/results/2026-10-02T01-34-37-644Z-allocations.json) contributes six further fully disposed samples with zero native allocations or heap storage. Its observed Node retained-heap deltas are 6,240 bytes, including measurement overhead; they are not the storage size of a JavaScript Number. The previous eight primitive probes retain their genuine earlier measurements, without reruns.

The [four Number predicate comparisons](../../benchmarks/runtime/results/2026-10-02T01-59-12-737Z.json) extend the report to 97 separately dated rows. Native averages are 279.705 million whole iterations/s for isFinite, 278.468 million for isNaN, 278.792 million for isInteger, and 282.147 million for isSafeInteger; Node after tsc averages 49.029, 50.905, 45.843, and 47.221 million respectively. Each row performs one actual static predicate and includes its shared 24-value setup, selection, digest, and loop costs. These are not isolated predicate latencies.

The [predicate runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T01-59-58-000Z-native-number-predicates-runtime-proof.json) validates 160 independent engine cases, 2,976 execution batches, and twelve fully disposed native samples. Each invocation allocates once for the number table, with a 192-byte peak, zero reallocations, and zero retained output storage. All 343 frozen material inputs, 308 prior raw records, and 93 preceding workload pairs remain exact; timed/profile emissions match for all four rows. The nine primitive memory probes and startup/compilation baselines retain their actual earlier measurements, without reruns or a fabricated combined 97-workload run.

The [five standard Math comparisons](../../benchmarks/runtime/results/2026-10-02T02-18-13-833Z.json) extend the report to 102 separately dated rows. Native averages are 1,509.220 million whole iterations/s for abs, 1,501.642 million for floor, 1,508.229 million for ceil, 1,488.031 million for trunc, and 1,713.636 million for sqrt. Corresponding Node after tsc rates are 1,090.519, 1,105.922, 1,120.233, 1,089.093, and 1,076.281 million. Each row invokes one actual method and includes table setup, selection, accumulation, and loop costs. The sqrt table includes ordinary finite 2, 3, and 10 alongside perfect squares; the independent oracle rounds every root and ordered supporting addition into IEEE bits.

The [Math runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T02-18-53-000Z-native-math-unary-runtime-proof.json) validates 200 independent engine cases, 3,720 execution batches, exact IEEE results for every case/process, and fifteen fully disposed native samples. Each invocation allocates once for its eight-number table with a 64-byte peak, zero reallocations, and zero retained output storage. All 356 frozen material inputs, 313 prior raw records, and 97 preceding workload pairs remain exact; all five timed/profile emissions match. The previous nine memory probes and auxiliary baselines retain their actual dates without reruns; no combined 102-workload run is claimed.


The [four Array.slice comparisons](../../benchmarks/runtime/results/2026-10-02T02-43-58-188Z.json) extend the report to 106 separately dated rows. Each whole iteration performs one actual slice and retains its complete result; all sixteen zero/one/two-argument queries are reachable. Input/query setup, bounds selection, output retention, and loops are included. Rust standard Clone may copy owned string bytes and nested arrays, whereas JavaScript shares immutable strings and nested references. These are nonmutating value/shape comparisons, without alias or ownership equivalence.

| Input elements | Native M iterations/s | Node after tsc M/s | Native / fastest scripting | Retained native bytes | Peak native bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| int | 96.692 | 64.737 | 1.486 | 151,304 | 151,304 |
| number | 103.680 | 68.866 | 1.497 | 102,152 | 102,216 |
| owned string | 43.556 | 68.007 | 0.638 | 392,992 | 393,212 |
| nested int array | 65.989 | 68.939 | 0.955 | 727,600 | 727,984 |

The [slice runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T02-45-21-000Z-native-array-slice-runtime-proof.json) verifies 160 independent engine cases, 2,976 execution batches, complete shape/scalar/IEEE tokens on every case and measured process, and twelve fully disposed native samples. O0/O3 observers cover empty ranks, NUL/astral strings, signed zero, NaN, infinity, and an emitted Vec-only number program. The 2,000-iteration profiles allocate 1,251/1,252/2,754/1,252 times for int/number/string/nested rows, with 8/9/9/9 reallocations. All retained outputs are completely released; the separate 14-byte string input also returns to baseline. All 410 frozen material inputs, 318 preceding raw records, and 102 prior workload pairs remain exact, and all four timed/profile emissions match. Two preparation failures produced no execution timing record; correcting rank ABI guards and selecting only available observer container implementations left compiler/workload bodies unchanged. The nine primitive memory probes and startup/compilation baselines retain their actual earlier dates, without reruns or a fabricated combined 106-workload run.


The [four temporary-array indexing comparisons](../../benchmarks/runtime/results/2026-10-02T03-08-58-857Z.json) extend the report to 110 separately dated rows. Every iteration creates a fresh returned array, evaluates its dynamic index, extracts a value, and observes the result. Receiver/index setup, normal disposal of unselected native owners, accumulation or retained output, and loops are included. Native selected owners transfer without hidden clones; JavaScript references and immutable strings may share storage. These are complete nonmutating value/shape comparisons, not isolated indexing latency or ownership equivalence.

| Selected element | Native M iterations/s | Node after tsc M/s | Native / fastest scripting | Native allocations per invocation | Retained / peak bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| int | 1,095.932 | 363.020 | 2.349 | 0 | 0 / 0 |
| number | 59.390 | 206.395 | 0.251 | 32,768 | 0 / 32 |
| owned string | 26.092 | 186.485 | 0.135 | 4,001 | 63,152 / 63,166 |
| nested int array | 105.128 | 57.398 | 1.667 | 1 | 98,304 / 98,304 |

The [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T03-09-58-000Z-native-temporary-array-index-runtime-proof.json) validates 160 independent engine cases, 2,976 execution batches, scalar IEEE/complete retained-shape results on every case and measured process, and twelve fully disposed native samples. Scalar rows run 32,768 iterations and retain no output arrays; string/nested rows retain 2,000 results. Number receivers allocate a fresh four-element standard Vec each iteration and release it during work; string/nested output arrays each grow through eight reallocations. Every retained output and the separate 14-byte string input fully returns to baseline. All 421 frozen material inputs, 323 preceding raw records, 106 prior workload pairs, and existing observer protocols remain exact; all four timed/profile/prepared-preflight bodies match. No preparation repairs, old benchmark reruns, or new memory/startup/compiler measurements occurred. The nine primitive probes and auxiliary baselines retain their genuine dates.


The [eight Array.map/filter comparisons](../../benchmarks/runtime/results/2026-10-02T03-32-12-893Z.json) extend the report to 118 separately dated rows. Each whole iteration invokes one real method over eight source elements, retains its complete output, and updates exact mutable callback trace/count state. Input setup, actual transformation/predicate work, retention/ownership, loops, and final trace-marker construction are included. Third-parameter source access remains read-only. Native selected-owner filter Clone may copy strings/nested rows while JavaScript shares immutable values/references; map-returned native owners transfer. These are nonmutating value/shape comparisons, not isolated method or callback latency.

| Method / elements | Native M iterations/s | Node after tsc M/s | Native / fastest scripting | Retained / peak native bytes |
| --- | ---: | ---: | ---: | ---: |
| map int | 25.700 | 4.495 | 2.222 | 226,304 / 226,368 |
| filter int | 16.074 | 4.115 | 1.577 | 204,960 / 205,024 |
| map number | 27.200 | 3.596 | 4.047 | 177,168 / 177,232 |
| filter number | 23.700 | 3.759 | 3.126 | 113,168 / 113,232 |
| map owned string | 9.776 | 3.819 | 1.137 | 658,528 / 658,758 |
| filter owned string | 11.070 | 4.346 | 0.975 | 585,246 / 585,476 |
| map nested int | 10.840 | 2.975 | 2.175 | 1,289,600 / 1,289,984 |
| filter nested int | 9.112 | 3.728 | 1.198 | 1,177,600 / 1,177,984 |

The [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T03-33-27-000Z-native-array-map-filter-runtime-proof.json) validates 320 independent engine cases, 5,952 execution batches, complete shape/IEEE/captured-effects results on every case and measured process, and 24 fully disposed native samples. All 16,000 callback visits per 2,000-iteration invocation are counted, including rejected filter elements. Numeric map preserves signed zero; string trim uses the documented shared ASCII-whitespace domain, and nested map includes actual row slices. Every output and the separate 19-byte string input fully returns to baseline. All 440 frozen material inputs, 328 preceding raw records, 110 prior workload pairs, and existing observer protocols remain exact; all eight timed/profile/prepared-preflight bodies match. No preparation repairs, historical reruns, or new memory/startup/compiler measurements occurred. The nine primitive probes and auxiliary baselines retain their genuine dates.


The [fixed-Number refresh](../../benchmarks/runtime/results/2026-10-02T04-02-21-910Z.json) adds two whole-iteration comparisons and updates the 22 existing rows whose actual emitted bodies changed from standard Vec tables to proved fixed storage. The other 96 measured rows keep their earlier dates. Direct indexing retains each selected IEEE value and exact source-order trace/count; the compound local sequence retains every saved and reversed value after a dynamic write, plus ordinary `0.5` membership and signed-zero search. NaN remains observed but is excluded as an includes needle because native PartialEq differs from JavaScript SameValueZero. Setup, trace/support work, output retention and loops are counted; neither row isolates indexing latency or claims total allocation-free execution. Escaping constructor-return arrays retain their existing Vec ABI.

| Comparison | Native M iterations/s | Node after tsc M/s | Native / fastest scripting | Native allocations / reallocations | Retained / peak native bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| direct Number literal index | 32.676 | 5.686 | 3.711 | 1 / 9 | 16,384 / 16,384 |
| compound stable Number locals | 98.416 | 7.345 | 7.394 | 1 / 14 | 524,288 / 524,288 |
| existing Number Array.slice | 102.561 | 68.780 | 1.491 | 1,251 / 9 | 102,152 / 102,152 |

The 21 scalar Number/Math rows now allocate no heap storage in their [native profiles](../../benchmarks/runtime/diagnostics/2026-10-02T04-02-54-173Z-native-allocations.json); both new rows allocate only the retained output vector. All 72 profile samples fully return to baseline. The [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T04-03-15-000Z-native-fixed-number-array-runtime-proof.json) seals 447 material inputs, 335 preceding raw records, 118 unchanged source pairs and existing observer protocols; all 24 timed/profile/preflight emissions agree. It verifies 960 independent engine cases and 17,856 execution batches with exact scalar/retained IEEE results, alongside 102 light Node groups. The initial compiler proof's stale release-binary claim remains preserved and is superseded only by the [actual release correction](../../benchmarks/runtime/diagnostics/2026-10-02T03-56-46.843Z-fixed-number-array-release-correction.json); the accepted 531 debug groups and other checks were retained without repeating the full regression. The report has 120 individually dated measured rows, nine earlier primitive probes and unchanged auxiliary baselines, without a fabricated combined run.


The [24 Array predicate/index-search comparisons](../../benchmarks/runtime/results/2026-10-02T04-35-52-687Z.json) extend the report to 144 individually dated rows. Each whole iteration constructs an empty or eight-element receiver, invokes one actual some/every/findIndex/findLastIndex method, and retains its result plus ordered callback trace/count. Eleven modes exercise empty, absent, early, late, multiple and all-candidate patterns. Forward prefix and reverse original-index suffix traversal are independently checked; search result index+1 preserves miss and all indices through the unchanged integer-tree observer. Setup, domain-specific predicates, captured effects, ownership, retention and loops are included. These are whole-method workload comparisons, not isolated API latencies or equal support costs across domains. Native constructor clones may copy owned string input bytes while JavaScript shares immutable storage; these methods select no output owners. All 24 rows lead Node after tsc and the fastest scripting counterpart in this snapshot. The ranges below span the four methods within each domain.

| Receiver domain | Native M iterations/s | Node after tsc M/s | Native / fastest scripting | Native allocations / reallocations | Retained / peak native bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| int | 32.615–35.534 | 5.907–6.815 | 3.610–4.091 | 1,819 / 8 | 98,304 / 98,368 |
| Number | 36.967–38.823 | 6.593–6.740 | 3.729–3.989 | 1,819 / 8 | 98,304 / 98,368 |
| Boolean | 33.646–34.627 | 5.367–6.418 | 3.985–4.208 | 1,819 / 8 | 98,304 / 98,312 |
| owned string | 12.834–13.295 | 7.304–7.594 | 1.246–1.304 | 5,455 / 8 | 98,304 / 98,526 |
| record | 32.016–34.696 | 5.631–6.265 | 4.167–4.647 | 1,819 / 8 | 98,304 / 98,432 |
| nested int | 25.850–26.787 | 5.229–6.032 | 3.069–3.909 | 1,819 / 8 | 98,304 / 98,688 |

The [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T04-36-28-000Z-native-array-predicates-runtime-proof.json) seals 498 material inputs, 340 preceding raw records and 120 unchanged source pairs. All 24 timed/profile/prepared emissions agree; 96 native preflight cases, 960 independent engine cases and 17,856 execution batches validate complete result/trace/count outputs. The [72 native profile samples](../../benchmarks/runtime/diagnostics/2026-10-02T04-36-18-227Z-native-allocations.json) fully dispose every output and input; the string input separately requests 15 bytes. Earlier nine primitive probes and auxiliary baselines keep their real dates, without reruns or a combined 144-workload claim. One preparation attempt failed before measurements because ES2022 library declarations omitted findLastIndex. The narrow ES2023 declaration-library correction kept the ES2022 output target, workload/native wrappers, and emitted JavaScript byteexact; its original failure and successful affected preparation are archived without repeating native preflight or compiler regression.


The [twelve string-construction comparisons](../../benchmarks/runtime/results/2026-10-02T05-09-07-504Z.json) extend the report to 156 individually dated rows. Each complete iteration invokes one actual repeat, first/all replacement, or split operation and retains all returned content/pieces plus exact receiver/argument evaluation markers. Native input cloning, tables, construction, wrapping/retention, markers and loops are included; this is whole-workload throughput, not isolated API latency. Replacement tokens include original prefix/suffix and dollar escapes. Empty-pattern/empty-separator rows use the explicit BMP shared domain; other rows retain astral, combining, and literal NUL content. Native leads Node after tsc and the fastest scripting counterpart on eight rows. The repeat and normal-split gaps below remain measured costs rather than targets for a representation experiment.

| Whole-iteration workload | Native M iterations/s | Node after tsc M/s | Native / fastest scripting | Native allocations / reallocations | Retained / peak native bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| repeat ASCII | 60.872 | 73.929 | 0.823 | 1,503 / 12 | 220,528 / 220,530 |
| repeat Unicode | 51.309 | 70.792 | 0.725 | 1,503 / 12 | 240,528 / 240,534 |
| repeat empty / zero | 74.812 | 78.902 | 0.911 | 2 / 12 | 214,528 / 214,528 |
| first replacement plain | 28.797 | 24.546 | 1.020 | 2,003 / 12 | 278,528 / 278,536 |
| first replacement templates | 16.695 | 10.987 | 1.206 | 2,003 / 1,012 | 255,528 / 255,536 |
| first replacement empty pattern | 12.119 | 8.576 | 1.261 | 2,003 / 3,012 | 264,528 / 264,533 |
| all replacement plain | 25.481 | 9.846 | 1.376 | 2,003 / 512 | 235,528 / 235,536 |
| all replacement templates | 7.882 | 3.127 | 1.108 | 2,003 / 2,512 | 295,528 / 295,536 |
| all replacement empty pattern | 5.092 | 1.676 | 1.169 | 2,003 / 5,012 | 324,528 / 324,533 |
| split normal / omitted | 18.056 | 25.054 | 0.650 | 4,803 / 12 | 401,728 / 401,738 |
| split limits | 22.171 | 8.069 | 1.696 | 3,253 / 12 | 373,528 / 373,538 |
| split empty separator | 24.715 | 9.748 | 1.888 | 4,753 / 12 | 363,528 / 363,533 |

The [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T05-09-41-000Z-native-string-construction-runtime-proof.json) seals 525 material inputs, 346 preceding raw records and all 144 previous source pairs. All twelve preflight/timed/profile emissions agree; 48 native cases, 480 independent engine cases and 8,928 execution batches validate complete shape/content and ordered effects. All [36 native samples](../../benchmarks/runtime/diagnostics/2026-10-02T05-09-29-570Z-native-allocations.json) fully dispose their owners. External input requests 0–10 bytes separately; native peak/retained counts exclude allocator metadata and RSS. The nine earlier primitive probes and startup/compiler baselines retain their actual dates without reruns or a fabricated combined 156-workload run. Two preparation-only failures produced no timing records. The narrow record/declaration spacing and zero-argument split type-overload repairs preserved all executable bodies and authored/prepared JavaScript; original sources/logs are archived, and the original parse failure produced no native body for an instruction-identity claim.


The [seven exact-Math comparisons](../../benchmarks/runtime/results/2026-10-02T05-39-50-064Z.json) extend the report to 163 separately dated rows. Each whole iteration calls one real round/sign/fround/min/max/clz32/imul method with runtime-derived bounded finite operands and scalar accumulation. Selection, conversion, argument construction, supporting addition and loops are included; min/max receive three dynamic operands. The independent rational/bit oracle handles ties, f32 quantization, uint32 conversion and signed wrapping without invoking measured methods. Bounded supporting sums are exactly representable. Signed-zero/nonfinite/empty-min-max identities are separate semantic coverage; sixteen constants have compiler acceptance rather than performance rows. All methods return Number, including integer-valued clz32/imul. These are whole-workload rates, not isolated instruction latencies.

| Whole-iteration method | Native M iterations/s | Node after tsc M/s | Native / fastest scripting |
| --- | ---: | ---: | ---: |
| round | 953.637 | 687.711 | 1.344 |
| sign | 1579.980 | 769.794 | 1.376 |
| fround | 1438.612 | 1270.684 | 1.104 |
| min | 448.174 | 546.586 | 0.813 |
| max | 435.953 | 545.357 | 0.781 |
| clz32 | 627.167 | 74.742 | 1.377 |
| imul | 399.756 | 680.963 | 0.584 |

Native leads four of these seven rows in this snapshot; min, max and imul remain measured gaps. All [21 native resource samples](../../benchmarks/runtime/diagnostics/2026-10-02T05-40-06-108Z-native-allocations.json) perform zero allocations/reallocations, request and retain zero heap bytes, and fully dispose to zero live bytes. Resource samples check the independent IEEE observer checksum; preflight and measured processes additionally preserve and validate raw scalar bit tokens. The [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T05-40-43-000Z-native-exact-math-runtime-proof.json) seals 543 material inputs, 351 preceding raw records and all 156 previous pairs. Seven actual emitted/prepared/profile kernels agree; 28 native preflight cases including every full-size invocation, 280 engine correctness cases and 5,208 execution batches pass. No preparation/timing repairs or compiler regression repeats were needed. The nine earlier memory probes and startup/compiler baselines retain their actual dates, with no fabricated combined 163-workload run.


The [22 approximate-Math comparisons](../../benchmarks/runtime/results/2026-10-02T06-20-44-810Z.json) extend the report to 185 individually dated rows. Every complete iteration performs one actual standard Math operation with runtime-derived nontrivial bounded finite inputs and ordered scalar accumulation. Input selection, conversion, argument arithmetic, supporting addition and loops are timed; these are whole-workload rates rather than isolated math latency. All 22 rows lead Node after tsc and the fastest scripting counterpart in this snapshot.

| Whole-iteration method | Native M iterations/s | Node after tsc M/s | Native / fastest scripting |
| --- | ---: | ---: | ---: |
| acos | 300.726 | 189.213 | 1.196 |
| acosh | 225.086 | 105.053 | 1.187 |
| asin | 262.720 | 178.262 | 1.201 |
| asinh | 148.314 | 99.054 | 1.274 |
| atan | 288.658 | 162.772 | 1.283 |
| atanh | 227.459 | 101.279 | 1.940 |
| cbrt | 351.614 | 171.331 | 1.263 |
| cos | 339.068 | 157.527 | 1.207 |
| cosh | 283.551 | 141.432 | 1.184 |
| exp | 396.042 | 169.890 | 1.243 |
| expm1 | 348.693 | 134.365 | 1.376 |
| log | 363.941 | 162.796 | 1.227 |
| log10 | 328.273 | 97.518 | 1.134 |
| log1p | 267.895 | 183.475 | 1.197 |
| log2 | 343.059 | 147.286 | 1.164 |
| sin | 317.927 | 162.581 | 1.253 |
| sinh | 278.120 | 99.839 | 1.191 |
| tan | 250.127 | 106.524 | 1.160 |
| tanh | 261.221 | 91.439 | 1.141 |
| atan2 | 182.099 | 114.035 | 1.569 |
| pow | 114.061 | 92.043 | 1.146 |
| hypot | 100.355 | 33.960 | 1.496 |

Numerical acceptance remains the policy declared before any engine outputs: eight adjacent binary64 ranks per operation, or 32 for four-argument hypot, over the frozen finite domains. This comparison criterion is not a language-wide precision guarantee. Genuine directed MPFR references and exact supporting-add arithmetic establish the final acceptance intervals; results are never quantized or replaced by shared checksums. Untimed operation probes differ by at most one rank from the rounded references. Native and Bun tan accumulate a final distance of 559 ranks (about 1.017e-9 at a result near 11,983), inside the independently propagated interval; other final distances are at most two ranks. The per-operation allowance is not a final-accumulator rank limit. Raw point, scalar, warmup and measured result bits and exact errors remain separately recorded. Untimed probes validate their own calls, not every timed intermediate invocation.

The [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T06-23-17-000Z-approximate-math-runtime-proof.json) verifies 623 frozen material inputs, 356 preceding raw records, all 163 previous source pairs, and matching preflight/timed/profile source, emitted-kernel, wrapper and immutable reference identities. Native O0/O3 preflight accepts 1,760 point calls, 220 scalar cases and 44 untimed allocation cases; compiled TypeScript accepts another 880 point calls and 110 scalar cases. All eight runtimes pass 7,040 point calls and 880 scalar correctness cases before timing. The run preserves 16,368 measured batches and 20,592 independently checked warmup/sample results. All [66 native profiles](../../benchmarks/runtime/diagnostics/2026-10-02T06-22-55-371Z-native-allocations.json) perform zero allocations/reallocations, request/retain/peak zero heap bytes and fully dispose to zero. These resource samples use a distinct untimed raw-result protocol rather than timing payloads or checksums. Nine earlier primitive probes and startup/compiler baselines retain their actual dates, without historical reruns or a fabricated combined 185-workload run. No preparation/timing repair or numerical-policy change was needed.


The [eight Number-operator comparisons](../../benchmarks/runtime/results/2026-10-02T07-01-29-093Z.json) extend the report to 193 separately dated rows. Each whole iteration performs one actual &, |, ^, ~, <<, >>, >>> or ** operation with runtime-derived operands and scalar accumulation. The seven exact rows also retain ordered once-evaluation markers through an ordinary mutable record. Tables, finite fractional/wrapped operands, function calls, marker updates, supporting additions and loops are counted; these are whole-workload rates, not isolated instruction latencies. Independent BigInt truncation/wrapping/bit arithmetic validates exact IEEE outputs. Exponentiation reuses the accepted pow mathematical recipe and independent MPFR payload through checked source/policy identity rebinding, preserving original provenance and unchanged numerical criteria. Native leads Node after tsc and the fastest scripting counterpart on all eight rows in this snapshot.

| Whole-iteration operator | Native M iterations/s | Node after tsc M/s | Native / fastest scripting |
| --- | ---: | ---: | ---: |
| & | 144.938 | 116.554 | 1.180 |
| \| | 136.133 | 113.362 | 1.147 |
| ^ | 144.281 | 114.345 | 1.167 |
| ~ | 297.770 | 225.467 | 1.289 |
| << | 139.977 | 120.852 | 1.143 |
| >> | 141.945 | 118.854 | 1.149 |
| >>> | 143.664 | 120.856 | 1.177 |
| ** | 113.906 | 91.242 | 1.148 |

The [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T07-03-33-000Z-native-number-operators-runtime-proof.json) verifies 644 material inputs, 361 preceding raw records, all 185 earlier source pairs and matching native preflight/timed/profile kernels. Seven real prepared TypeScript and emitted JavaScript files match the earlier accepted source checks. Native O0/O3 preflight includes 38 builds, 80 scalar cases, 80 exponent points, 126 direct runtime-bit operator results covering NaN/infinity/signed zero/wrap/count boundaries, two complete expression/order fixtures and 16 untimed allocation checks. All eight engines pass 320 scalar cases and 320 exponent point calls before timing; 5,952 measured batches are recorded. Exponent point and accumulated-result errors are zero ranks across all eight engines, without quantization, shared checksum or changes to the predeclared eight-rank criterion. Untimed probes cover their own calls rather than every timed intermediate invocation. All [24 native resource samples](../../benchmarks/runtime/diagnostics/2026-10-02T07-02-51-562Z-native-allocations.json) perform zero allocations/reallocations, request/retain/peak zero heap bytes and fully dispose to zero. Exact profiles use the independent IEEE observer checksum, with raw IEEE bits additionally validated by scalar preflight and runtime processes; exponent profiles use the distinct untimed raw-result/error protocol. Earlier nine primitive probes and auxiliary baselines retain their true dates. One initial preparation failure is preserved: a temporary direct-bit fixture attempted to log its array, before native compilation or accepted work. Its unused main alone was corrected to log length, with a fresh directory preserving the original attempt. No benchmark sources or compiler implementation changed for the repair, no full regression was repeated, and no combined 193-workload run is claimed.


The [28 reduce/reduceRight comparisons](../../benchmarks/runtime/results/2026-10-02T07-46-32-501Z.json) extend the grouped report to 221 separately dated rows. They observe scalar, record and nested-array results, complete callback effects and forward/reverse original-index traversal. Each whole iteration includes receiver setup, callback construction, accumulator initialization, ordinary native ownership/copying, result retention and loop costs; these are not isolated reduction-call latencies. JavaScript may share references and immutable strings where native Clone copies owned contents. No alias/identity equivalence or hidden JavaScript deep-copy adaptation is claimed. Native leads Node after tsc and the fastest scripting counterpart on 18 of 28 rows, with native/fastest ratios from 0.367 to 3.468. Records, nested numbers/records, deep int arrays and int-to-record accumulators remain gaps in this snapshot.

| Reduction result domain | reduce native M iterations/s | Native / fastest scripting | reduceRight native M iterations/s | Native / fastest scripting |
| --- | ---: | ---: | ---: | ---: |
| int | 45.977 | 2.682 | 40.768 | 2.423 |
| number | 37.853 | 2.435 | 35.075 | 2.146 |
| Boolean | 55.803 | 3.043 | 60.190 | 3.468 |
| string | 6.175 | 1.107 | 6.150 | 1.084 |
| record | 7.785 | 0.587 | 7.797 | 0.569 |
| nested int | 34.323 | 1.984 | 34.908 | 2.053 |
| nested number | 9.425 | 0.485 | 9.275 | 0.516 |
| nested Boolean | 31.783 | 1.807 | 36.236 | 2.017 |
| nested string | 18.613 | 1.013 | 18.806 | 1.031 |
| nested record | 6.580 | 0.367 | 6.866 | 0.379 |
| deep int | 10.054 | 0.556 | 10.127 | 0.567 |
| int → record | 5.106 | 0.562 | 5.003 | 0.580 |
| int → array | 30.188 | 1.935 | 28.753 | 1.820 |
| int → nested | 19.335 | 1.476 | 18.902 | 1.440 |

The [runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T07-49-56-000Z-array-reduction-runtime-proof.json) binds all actual record/container ABIs, emitted/timed/profile kernels, 710 frozen material files and 366 preceding raw records. Scalar outputs remain scalars; complete typed external observers preserve string Unicode/NUL content, Boolean values, record fields, all array levels, empty rows and IEEE bits outside timing. Actual native O0/O3 preflight retains 118 builds, 280 cases, 56 untimed allocation checks and six complete order/expected-empty-failure/observer-disposal fixtures. All eight engines pass 1,120 independent preparations before 20,832 measured batches. All [84 native resource samples](../../benchmarks/runtime/diagnostics/2026-10-02T07-48-00-274Z-native-allocations.json) return to zero live bytes after output disposal and final owner disposal. Requested bytes measure transient copying as well as retained results: 6,005–2,227,072 bytes per workload; output retention ranges from zero to 716,800 bytes, with peak live requested bytes up to 716,880. Observer checksum traversal allocates no storage. All previous 193 runtime tables and nine memory tables are byte-identical, with their original dates; no old-row, startup/compiler or memory rerun and no combined 221-row run is claimed. The original preparation failure is retained: after all 28 native rows passed, a separate callback-factory fixture required explicit read access. Its declaration alone was corrected in a separate directory, with byte-identical TypeScript erasure; only six unrun fixture builds followed and all 112 prior row builds were retained.


Gate52 Array.forEach comparisons

The [sixteen execution-only rows](../../benchmarks/runtime/results/2026-10-02T08-30-03-080Z.json) extend the report to 237 independently dated measurements. Whole iterations include receiver setup/copies, callback construction, value queries and recursive traversal, mutable effect traces and loops. Ascending original indices, read-only third parameters, empty/singleton behavior and complete scalar results are independently checked. Native owned copies and JavaScript sharing differ. Discarded callback return construction may be optimized away; these are observed whole-workload costs, without isolated API latency or equal allocation claims. Native leads Node after tsc on 15/16 and the fastest scripting counterpart on 12/16.

| forEach domain | Native M iterations/s | Native / Node after tsc | Native / fastest scripting |
| --- | ---: | ---: | ---: |
| int | 45.060 | 4.141 | 2.560 |
| number | 15.064 | 1.303 | 1.249 |
| boolean | 54.830 | 4.914 | 3.015 |
| string | 19.685 | 2.283 | 1.311 |
| record | 17.523 | 2.483 | 1.427 |
| nested int | 35.596 | 3.187 | 2.425 |
| nested number | 8.450 | 1.313 | 0.889 |
| nested boolean | 35.703 | 3.762 | 2.460 |
| nested string | 13.916 | 1.935 | 1.280 |
| nested record | 9.329 | 1.225 | 0.800 |
| deep int | 16.424 | 1.642 | 1.233 |
| recursive record | 9.861 | 1.396 | 0.896 |
| discard string | 43.469 | 5.041 | 3.294 |
| discard record | 43.577 | 5.074 | 3.477 |
| discard nested | 6.906 | 0.806 | 0.548 |
| discard callable | 42.883 | 4.988 | 3.280 |

The [sealed runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T08-32-59-000Z-array-for-each-runtime-proof.json) binds 745 frozen materials, all 66 actual preflight binaries, compiler/source/fixture identities, 384 process cells and all 128 published min/arithmetic-mean/max rate rows. O0/O3 preflight passes 160 cases, 32 distinct untimed allocation checks and two complete order/empty/singleton outputs; normal preparation passes 640 engine cases before 11,904 measured batches. All [48 native profiles](../../benchmarks/runtime/diagnostics/2026-10-02T08-31-27-294Z-native-allocations.json) return to zero after output and final owner disposal. Requested bytes include transient copying (6,005–1,422,040 per workload), with no retained scalar heap output and peak live requested bytes of 10–2,000. Nested numbers/records, recursive records and discarded nested results remain measured gaps. Earlier 221 runtime tables, nine memory tables and startup/compiler tables are byte-identical; 442 preceding source files and 371 prior raw records are unchanged. All dates remain genuine, without a combined 237-row run or historical rerun.


Gate53 Array.concat comparisons

The [25 new execution-only comparisons](../../benchmarks/runtime/results/2026-10-02T09-37-35-119Z.json) measure zero-argument copying and variadic scalar/array operands in twelve domains, plus an ordered-effect row. Whole iterations include setup, concat, native owned copies, result retention and loop control. Rust clones nested owners while JavaScript concat can share nested references and immutable strings; the comparisons observe complete nonmutated values rather than alias or allocation equivalence. Native leads Node after tsc on 8/25 and the fastest scripting counterpart on 6/25.

| Concat domain | Zero-argument native M iterations/s | Native / fastest scripting | Variadic native M iterations/s | Native / fastest scripting |
| --- | ---: | ---: | ---: | ---: |
| int | 38.572 | 0.869 | 16.038 | 2.319 |
| number | 39.732 | 0.873 | 16.080 | 1.750 |
| Boolean | 41.962 | 0.937 | 17.267 | 2.485 |
| string | 18.074 | 0.394 | 6.049 | 0.860 |
| record | 17.365 | 0.396 | 6.199 | 0.892 |
| nested int | 29.122 | 0.633 | 11.411 | 1.186 |
| nested number | 10.579 | 0.236 | 5.588 | 0.580 |
| nested Boolean | 29.908 | 0.645 | 11.979 | 1.242 |
| nested string | 13.678 | 0.284 | 6.252 | 0.626 |
| nested record | 7.823 | 0.179 | 4.287 | 0.438 |
| deep int | 13.439 | 0.287 | 5.862 | 0.600 |
| recursive record | 8.384 | 0.182 | 3.115 | 0.499 |

The separate ordered-effect row measures 15.192 million native iterations/s (4.188 times Node after tsc, 3.134 times the fastest scripting counterpart); its receiver/argument evaluation and trace work differ from the retained-output rows. The [sealed proof](../../benchmarks/runtime/diagnostics/2026-10-02T09-40-32-000Z-array-concat-runtime-proof.json) binds 806 frozen materials, 377 prior raw records, all actual result ABIs and 200 published min/arithmetic-mean/max engine rates. Normal preparation accepts 1,000 engine cases before 18,600 timed batches. Complete outside-timer observers preserve every array/record level, empty container, Unicode/NUL string and floating sign. All [75 native profiles](../../benchmarks/runtime/diagnostics/2026-10-02T09-39-24-356Z-native-allocations.json) return to zero live bytes after output and final disposal. Requested bytes include transient copies (53,960–1,950,056 per workload), retained outputs range from zero to 974,576, and peak requested storage from 120 to 976,976. Owned and nested copy costs remain measured gaps; no JavaScript deep-copy adaptation hides them.

The initial recursive scalar concat preparation failure and narrow fixed-storage repair are retained. Twenty-three refreshed emissions exactly match the prior accepted bodies, so their 92 binaries, 230 cases and 46 cleanup checks were retained without native reexecution. Only the remaining two rows and order fixture required ten new builds, twenty cases, four cleanup checks and two order outputs; combined preflight binds 102 binaries, 250 complete cases and 50 untimed allocation checks. The original single 566-pass compiler regression and targeted repair checks remain separately identified. Old 237 runtime tables, nine memory tables and auxiliary baselines retain their genuine dates and values, with all 474 preceding pair files preserved. The report now has 262 independently dated measurements; no old-row rerun or combined 262-row measurement is claimed.


Gate54 declared-function-value comparisons

The [twelve new execution-only comparisons](../../benchmarks/runtime/results/2026-10-02T10-37-47-527Z.json) extend the report to 274 independently dated rows. Whole iterations include setup, callback/factory evaluation, ordinary native ownership/copies, retained results and loop costs. Natural scalar and complete array/record/tree outputs remain unchanged by outside-timer observers. Native leads Node after tsc on 9/12 and the fastest scripting counterpart on 7/12. These matched values do not imply equal JavaScript sharing, ownership or allocation behavior.

| Declared-function workload | Native M iterations/s | Native / Node after tsc | Native / fastest scripting |
| --- | ---: | ---: | ---: |
| int callback reduction | 53.833 | 2.273 | 1.596 |
| number callback reduction | 61.036 | 0.892 | 0.892 |
| Boolean callback reduction | 65.313 | 1.104 | 1.104 |
| string callback map | 6.558 | 1.361 | 0.834 |
| record callback map | 16.404 | 0.683 | 0.636 |
| local reuse | 252.564 | 4.820 | 2.969 |
| local move | 255.779 | 4.985 | 3.056 |
| returned function | 250.449 | 5.099 | 3.984 |
| factory target | 12.566 | 1.389 | 0.741 |
| global use | 270.247 | 5.057 | 3.550 |
| bound capture | 263.378 | 5.071 | 3.249 |
| owned tree move | 27.613 | 0.588 | 0.487 |

The [sealed runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T10-39-37-000Z-declared-function-runtime-proof.json) binds 846 materials, 383 preceding raw records, all actual result ABIs and 96 recomputed min/arithmetic-mean/max rate rows. Normal preparation accepts 480 independent engine cases before 8,928 batches. Native O0/O3 preflight binds 52 binaries, 120 complete cases, 24 untimed allocation checks and four order/IEEE outputs. All [36 resource samples](../../benchmarks/runtime/diagnostics/2026-10-02T10-39-26-251Z-native-allocations.json) return to zero after output and final disposal. Local reuse/move, returned function, global use and bound capture allocate zero heap bytes. Other rows include ordinary receiver/output copies; requested bytes reach 238,508 and retained output reaches 120,832 per workload. Number and record callback reductions, string mapping, factories and owned-tree outputs retain measured counterpart gaps.

The initial separate order-fixture preparation failure and exact two-annotation repair remain archived. All twelve accepted rows retained 48 binaries, 120 cases and 24 cleanup observations; only four unrun fixture builds followed. The corrected canonical fixture is bound by an explicit one-file supplement without rewriting the historical source proof. All old 262 runtime tables, nine memory tables and auxiliary tables remain byte-identical, and all 524 preceding pair files and 383 prior raw records retain their hashes. No old comparisons were retimed and no combined 274-row run is claimed.
