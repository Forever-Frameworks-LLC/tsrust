# Native string implementation history

Current user guidance: [Work with strings](../../../website/src/content/docs/language/strings.md).

This document retains original contracts and milestone observations. Later changes
supersede some entries, including scalar concatenation and template conversions.
Use the current guide and compiler-backed API help for supported behavior; historical
benchmark and representation notes below remain evidence for their checkpoints.

[Documentation index](../../README.md) · [String milestone](../../milestones/33-native-utf8-strings.md)

The user selected native Rust UTF-8 behavior on October 1, 2026. TSR provides TypeScript-style syntax with Rust ownership, standard data structures, and native operation costs. Exact JavaScript Unicode behavior is not a requirement.

Strings contain valid UTF-8. Generated storage uses standard Rust `Cow<'static, str>`: literals borrow static `str`, and runtime results own a standard `String`. Clone uses the standard library. There are no bespoke string tags, surrogate buffers, shared slice roots, interning descriptors, normalization graphs, or encoding caches.

| Source operation | Native behavior |
| --- | --- |
| `value.toString()`, `value.valueOf()` | String primitive receivers only, strict zero arity, receiver evaluated once. Standard Cow::clone preserves exact UTF-8: static storage stays borrowed and owned results copy independently; empty owned clones use normal standard-library behavior. Receiver remains reusable. |
| `value.length` | UTF-8 byte length, using `str::len()` |
| `value.slice(start?, end?)` | Signed byte offsets clamp to byte length. Reversed/empty ranges return empty. Nonempty ranges must fall on UTF-8 character boundaries. Invalid boundaries fail explicitly. |
| `value.substring(start, end?)` | Strict int byte bounds clamp to zero/length, omitted end means length, and reversed bounds swap. Equal empty ranges skip boundary access. Nonempty endpoints must be scalar boundaries. Full range uses standard Cow cloning; partial static text borrows and owned text copies independently. |
| `value.substr(start?, length?)` | Strict int UTF-8 byte start defaults to zero; negative starts count from the end and clamp. Length defaults to the remaining bytes; nonpositive lengths are empty, positive lengths cap safely. Nonempty endpoints require scalar boundaries. Receiver stays reusable; full ranges clone Cow, partial static text borrows and owned text copies. |
| `value.charAt(index?)` | Unicode scalar from `str::chars().nth(index)`. Negative/out-of-range indices return empty. Scalar access can require traversal. |
| `left + right` | Two strings only; exact UTF-8 concatenation into a fresh independent owned Cow result. Both operands are borrowed once, left to right, and remain reusable. Chains construct each intermediate result. No implicit coercion. |
| `target += rhs` | Matching strings and an available mutable named target. Select the target once and borrow its old value before RHS evaluation, construct a fresh owned concatenation, then replace the target. Self-append is supported; existing loans and source reuse rules remain enforced. |
| `value.trim()` | Rust `str::trim()` whitespace rules |
| `value.trimStart()`, `value.trimLeft()` | Rust `str::trim_start()` whitespace rules; preserve the right edge and interior bytes |
| `value.trimEnd()`, `value.trimRight()` | Rust `str::trim_end()` whitespace rules; preserve the left edge and interior bytes |
| `value.toLowerCase()` | Rust locale-independent Unicode lowercase using the pinned toolchain |
| `value.toUpperCase()` | Rust locale-independent Unicode uppercase using the pinned toolchain; case expansion can change byte length |
| `value.startsWith(needle, position?)` | Standard Rust prefix search on the suffix starting at a clamped byte position; default zero |
| `value.endsWith(needle, position?)` | Standard Rust suffix search on the prefix ending at a clamped byte position; default byte length |
| `value.includes(needle, position?)` | Standard Rust substring search starting at a clamped byte position; default zero |
| `value.indexOf(needle, position?)` | First absolute byte offset from standard Rust substring search, or `-1`; default starting position zero |
| `value.lastIndexOf(needle, position?)` | Greatest matching byte start at or before a clamped position, or `-1`; default byte length. Negative positions clamp to zero. Selected positions must be UTF8 boundaries, even for empty needles. Empty needles return the selected position; overlapping matches and matches ending after the selected position remain valid. |
| String array search | Exact UTF-8 content equality; no Unicode normalization |
| `left === right`, `left !== right` | Exact string content comparison through standard Rust borrows; no implicit clone or transfer |
| `left < right`, `<=`, `>`, `>=` | Standard Rust UTF-8 lexical ordering; can differ from JavaScript's UTF-16 order for astral scalars |

For `"😀"`, length is four bytes, `slice(0, 4)` returns the emoji, `slice(0, 1)` fails at a character boundary, and `charAt(0)` returns the emoji. A Unicode scalar is not necessarily a visible grapheme; combining sequences can contain multiple scalars.

String comparison operands evaluate exactly once, left to right. A borrowed left source remains protected while the right operand evaluates. Matching string operands are required; there is no coercion or normalization. Complete compiler, runtime comparison, and disposal acceptance is recorded in [milestone 34](../../milestones/34-string-comparisons.md).

String search positions clamp negative values to zero and values above byte length to byte length. Unlike `slice`, negative search positions do not count from the end. Selected prefix or suffix boundaries must be valid UTF-8 boundaries, even with an empty needle. Receiver, needle, and position evaluate once in that order, with both strings borrowed through position evaluation. Search performs no Unicode normalization. [Milestone 37](../../milestones/37-string-search.md) records complete compiler, comparison, and disposal acceptance.

String reverse search follows the same borrowing and operand-order rules. Position bounds constrain the start of the match, allowing its end to extend past that position. [Milestone 60](../../milestones/60-string-last-index-of.md) records complete `lastIndexOf` acceptance. ASCII counterparts use identical offsets directly; Unicode counterparts translate UTF8/UTF16 positions and results, with translation costs included in the measured comparisons.

Lone surrogate escapes are rejected with source diagnostics. Valid escaped surrogate pairs decode to their Unicode scalar. Embedded NUL and other valid scalars remain supported. Runtime input conversion must also reject invalid Unicode rather than silently replacing it.

Rust whitespace and Unicode tables intentionally differ from JavaScript. In particular, Rust trim removes U+0085 and preserves U+FEFF. Toolchain pinning makes native behavior reproducible; it does not imply parity with Node's Unicode version.

Directional trimming uses the same whitespace rules and Cow ownership contract. The receiver evaluates once and remains reusable. Borrowed static literals can return borrowed static slices; owned receivers produce independent owned results. `trimLeft` and `trimRight` are exact aliases of `trimStart` and `trimEnd`. [Milestone 61](../../milestones/61-string-trim-sides.md) records compiler, matched runtime and disposal acceptance. Timed JavaScript comparisons use the common whitespace domain; separate source witnesses retain U+0085/U+FEFF differences.

Benchmarks must compare the same byte/scalar operation and validate output independently. Historical JavaScript UTF-16 measurements remain immutable in the benchmark history and [archive](../../../archive/utf16-strings-2026-10-01/README.md). They are not evidence for the native UTF-8 contract.

String addition uses checked combined byte capacity, standard String::with_capacity and push_str. It does not normalize Unicode or alter whitespace. Conflicting RHS mutation while the left source is borrowed remains rejected; an explicit clone can provide an independent snapshot. [Milestone 62](../../milestones/62-string-addition.md) records complete contents, contextual typing, source reuse and normal/unwind disposal. Matched JavaScript + comparisons include construction and complete output retention; native owned copying and chain intermediates remain explicit costs. Matching String/String += now has its own [Milestone 63](../../milestones/63-compound-assignment.md) acceptance and ASCII/Unicode comparisons. String.concat and directional padding now have separate [Milestone 70](../../milestones/70-string-concat-padding.md) acceptance. Concat borrows strict String operands once in source order; real concatenation owns its result, while zero/all-empty operands use standard Cow::clone. Padding takes a strict int byte target and optional String filler (default space). Negative/short targets and empty filler clone the receiver before capacity or prefix checks. Real padding uses one checked fallible reservation and rejects a filler prefix cutting a Unicode scalar; valid FEFF, Chinese, astral and combining-scalar prefixes preserve their contents. Static no-op receivers remain borrowed, owned no-ops copy independently, and every argument still evaluates. JavaScript counterparts translate byte targets to equivalent UTF-16 targets over common outputs, with tuple setup and selection timed; native boundary failures are separate evidence.

[Milestone 71](../../milestones/71-string-substring.md) accepts substring with receiver and bounds borrowed/evaluated once in source order, including empty/full selections. Source-span boundary failures and normal/unwind disposal preserve the existing Cow representation. Its six comparisons call actual JavaScript substring over matched output values, with byte/UTF-16 endpoint tuple setup and selection timed. These whole workloads retain complete String[] results; no isolated-call or allocation/identity equivalence is claimed.

[Milestone 72](../../milestones/72-primitive-identity.md) accepts String identity methods alongside Boolean text/identity, Number identity and the native int.valueOf extension. These primitive methods introduce no boxing or numeric formatting. Its four String comparisons preserve complete Unicode/NUL/FEFF text without a coordinate bridge; setup, selection and natural Cow costs stay in the whole workload. Static sharing and owned copies do not claim JavaScript identity or allocation parity.

## Ordinary quoted source literals

Both single and double quotes decode to native UTF-8 scalars at compile time. Strict escapes include NUL (without an immediately following decimal source digit), vertical tab, hexadecimal bytes, four-hex-digit Unicode and braced code points, plus Unicode identity escapes. LF, CR, CRLF, U+2028 and U+2029 line continuations contribute no content. Raw LF/CR are rejected; raw U+2028/U+2029 are retained. Surrogate halves pair across empty continuations and either Unicode escape form; lone halves are rejected. Legacy decimal/octal escapes remain unsupported. [Milestone 77](../../milestones/77-quoted-string-literals.md) records implementation acceptance; matched comparisons are queued for checkpoint 81.

## Ordinary template literals

Backticks support nested `${expression}` substitutions of String, int, number and boolean. Each expression evaluates once and is appended before the next expression runs, releasing that operand borrow. The result owns independent UTF-8 text; templates without substitutions remain static borrowed literals. Raw CR/CRLF normalize to LF, and strict escapes follow quoted literals. Number text uses Rust Display (including `-0`, `inf` and `-inf`), which can differ from JavaScript formatting. Object/callable coercion and tagged templates remain unsupported. [Milestone 78](../../milestones/78-template-literals.md) records focused acceptance and pending checkpoint comparisons.

## Split storage and limit documentation

`split(separator?, limit?)` retains its [milestone 47](../../milestones/47-string-construction.md) contract: strict String separator, optional signed int limit converted modulo 2^32, default 4294967295, retained empty pieces for nonempty separators and scalar pieces for empty separators. A zero effective limit returns no pieces after evaluating supplied arguments. Static receivers now return static borrowed pieces; owned receivers produce independent copied pieces. The result array stays growable, and source strings remain reusable. [Milestone 79](../../milestones/79-split-storage-and-documentation.md) records complete documentation, examples, focused ownership/disposal checks and pending checkpoint measurements.
