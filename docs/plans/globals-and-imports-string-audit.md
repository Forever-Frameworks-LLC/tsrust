# String receiver and static construction audit

Status: 50 members reviewed (47 string receiver members and three String static
members). Global String conversion and the string type keyword remain separate
rows in the final primitive/global audit.

## Placement and ownership

Retain string receiver methods and familiar global String static methods. No
imported duplicates or renaming are proposed. These operations act on ordinary
language values; requiring imports would add ceremony without clarifying resource
ownership. Familiar spelling must not conceal the indexing and Unicode differences
recorded below. Legacy HTML wrappers stay supported for compatibility; they create
text, not a DOM or sanitized markup.

All receiver methods borrow the receiver and any string arguments through argument
evaluation. Scalar arguments copy. Results are scalar copies or independent string/
array values; no mutable receiver operation exists. Static literal storage may
remain shared as immutable static slices. Owned source slices/identity/trim results
copy their text rather than lending a new public borrow into the owner. Native
strings always contain valid UTF-8; lone UTF-16 surrogate strings do not exist.

Argument types/counts are checked statically, including required int positions and
string-only literal patterns. Byte boundary errors, invalid repeat counts,
normalization forms, code points and checked capacity failures terminate execution;
these APIs do not return Result or promise a catchable typed error. Source-located
messages exist where the emitting operation supplies a source span. Allocation
failure behavior is not upgraded to a recoverable contract by this audit.

## Receiver members

| Symbol | Signature | Purpose, defaults, bounds and errors |
|---|---|---|
| `string.isWellFormed` | `isWellFormed(): boolean` | Always true for valid native UTF-8 strings; no arguments. |
| `string.toWellFormed` | `toWellFormed(): string` | Returns unchanged text as an independent value; native strings cannot contain lone surrogates. |
| `string.trim` | `trim(): string` | Removes Unicode whitespace at both ends; U+0085 removed, U+FEFF retained. |
| `string.trimStart` | `trimStart(): string` | Removes leading Unicode whitespace only. |
| `string.trimEnd` | `trimEnd(): string` | Removes trailing Unicode whitespace only. |
| `string.trimLeft` | `trimLeft(): string` | Alias of trimStart. |
| `string.trimRight` | `trimRight(): string` | Alias of trimEnd. |
| `string.toLowerCase` | `toLowerCase(): string` | Locale-independent Unicode lowercase mapping; may change byte length. |
| `string.toUpperCase` | `toUpperCase(): string` | Locale-independent Unicode uppercase mapping; may expand text. |
| `string.toLocaleLowerCase` | `toLocaleLowerCase(): string` | Same mapping as toLowerCase; locale arguments rejected. |
| `string.toLocaleUpperCase` | `toLocaleUpperCase(): string` | Same mapping as toUpperCase; locale arguments rejected. |
| `string.includes` | `includes(search: string, position?: int): boolean` | Literal search from clamped byte position (default zero); boolean result; interior UTF-8 byte boundary fails. |
| `string.startsWith` | `startsWith(search: string, position?: int): boolean` | Literal prefix at clamped byte position (default zero); interior byte boundary fails. |
| `string.endsWith` | `endsWith(search: string, position?: int): boolean` | Literal suffix ending at clamped byte position (default byte length); interior byte boundary fails. |
| `string.indexOf` | `indexOf(search: string, fromIndex?: int): int` | First literal match at/after clamped byte offset (default zero); byte index or -1; invalid boundary fails. |
| `string.lastIndexOf` | `lastIndexOf(search: string, fromIndex?: int): int` | Last literal match whose start is at/before clamped byte offset (default byte length); byte index or -1; invalid boundary fails. |
| `string.slice` | `slice(start?: int, end?: int): string` | Relative byte offsets; defaults zero/byte length; negatives count from end, clamp; reversed/empty range gives empty; nonempty endpoints must be UTF-8 boundaries. |
| `string.substring` | `substring(start: int, end?: int): string` | Required byte start; end defaults byte length; negatives clamp zero; reversed bounds swap; nonempty endpoints must be UTF-8 boundaries. |
| `string.substr` | `substr(start?: int, length?: int): string` | Relative byte start defaults zero; length defaults remainder, negative length gives empty; safely clamps signed extremes; nonempty endpoints require UTF-8 boundaries. |
| `string.charAt` | `charAt(index?: int): string` | Scalar index defaults zero; negative/out-of-range returns empty string. |
| `string.concat` | `concat(...values: string[]): string` | Concatenates receiver and string arguments; no coercion; checked capacity/allocation failures terminate. |
| `string.padStart` | `padStart(target: int, filler?: string): string` | Pads before receiver to target byte length; filler defaults space; short/negative target or empty filler is unchanged; partial filler must end on UTF-8 boundary. |
| `string.padEnd` | `padEnd(target: int, filler?: string): string` | Same byte/filler rules as padStart, appending after receiver. |
| `string.repeat` | `repeat(count: int): string` | Repeats count times; zero empty; negative count or capacity overflow terminates; independent result. |
| `string.replace` | `replace(search: string, replacement: string): string` | First literal match; replacement expands $$, $&, prefix and suffix tokens; no regex/capture groups. |
| `string.replaceAll` | `replaceAll(search: string, replacement: string): string` | Every literal match; same replacement templates; empty search matches scalar boundaries. |
| `string.split` | `split(separator?: string, limit?: int): string[]` | Literal separator; omission yields whole receiver. Empty separator yields Unicode scalars. Optional int limit converts modulo 2^32, default 4294967295; zero gives empty array; excess pieces discarded. |
| `string.anchor` | `anchor(name: string): string` | Returns text wrapped in <a> markup with name attribute; quotes in attribute become &quot;. Receiver body remains raw; no HTML sanitization. |
| `string.big` | `big(): string` | Returns text wrapped in <big> markup. Receiver body remains raw; no HTML sanitization. |
| `string.blink` | `blink(): string` | Returns text wrapped in <blink> markup. Receiver body remains raw; no HTML sanitization. |
| `string.bold` | `bold(): string` | Returns text wrapped in <b> markup. Receiver body remains raw; no HTML sanitization. |
| `string.fixed` | `fixed(): string` | Returns text wrapped in <tt> markup. Receiver body remains raw; no HTML sanitization. |
| `string.fontcolor` | `fontcolor(color: string): string` | Returns text wrapped in <font> markup with color attribute; quotes in attribute become &quot;. Receiver body remains raw; no HTML sanitization. |
| `string.fontsize` | `fontsize(size: string): string` | Returns text wrapped in <font> markup with size attribute; quotes in attribute become &quot;. Receiver body remains raw; no HTML sanitization. |
| `string.italics` | `italics(): string` | Returns text wrapped in <i> markup. Receiver body remains raw; no HTML sanitization. |
| `string.link` | `link(url: string): string` | Returns text wrapped in <a> markup with href attribute; quotes in attribute become &quot;. Receiver body remains raw; no HTML sanitization. |
| `string.small` | `small(): string` | Returns text wrapped in <small> markup. Receiver body remains raw; no HTML sanitization. |
| `string.strike` | `strike(): string` | Returns text wrapped in <strike> markup. Receiver body remains raw; no HTML sanitization. |
| `string.sub` | `sub(): string` | Returns text wrapped in <sub> markup. Receiver body remains raw; no HTML sanitization. |
| `string.sup` | `sup(): string` | Returns text wrapped in <sup> markup. Receiver body remains raw; no HTML sanitization. |
| `string.length` | `length: int` | Readonly UTF-8 byte count as int; no receiver mutation. |
| `string.valueOf` | `valueOf(): string` | Independent string value with unchanged contents; owned input copies, static storage may be reused. |
| `string.toString` | `toString(): string` | Same string identity/copy behavior as valueOf. |
| `string.at` | `at(index: int): string \| undefined` | Required scalar index; negative counts from end; absent gives undefined. Current lowering builds temporary scalar string array, then uses array.at. |
| `string.codePointAt` | `codePointAt(position?: int): int \| undefined` | Scalar position defaults zero; negative/out-of-range undefined; returns int code point. Current lowering uses an optional code slot plus array.at. |
| `string.charCodeAt` | `charCodeAt(position?: int): number` | Scalar position defaults zero; negative/out-of-range NaN; returns first UTF-16 unit of selected scalar as number (high surrogate for supplementary scalar). |
| `string.normalize` | `normalize(form?: string): string` | NFC default (also undefined); accepts NFC/NFD/NFKC/NFKD; invalid form terminates with source location; independent normalized text, no trimming/case conversion. |

## Static String members

| Symbol | Signature | Purpose, effects and errors |
|---|---|---|
| `String.fromCodePoint` | `fromCodePoint(...codes: int[]): string` | Reads int code points 0..1114111; out-of-range terminates at call source location after argument evaluation. Adjacent surrogate halves combine; unpaired halves become U+FFFD. Empty call yields empty text; scalar spread sources stay usable. |
| `String.fromCharCode` | `fromCharCode(...codes: int[]): string` | Uses low 16 bits of each int; decodes UTF-16, combining pairs and replacing unpaired halves with U+FFFD. Empty call yields empty; read-only spread sources remain usable. |
| `String.raw` | `raw(template: { raw: string[] }, ...substitutions: unknown[]): string` | Borrows template.raw string array and substitutions; interleaves String-converted substitutions, missing values contribute empty text, extras are evaluated but unused. Empty raw array yields empty string; existing backslashes preserved. No tagged-template syntax implied. |

## Source reconciliation and dependencies

Compiler anchors under crates/tsrust-compiler/src:

- analysis.rs receiver dispatch recognizes trimming, casing, searching, slicing,
  construction and identity methods; string_construct checks method-specific arity
  and integer/string argument access. string_position/string_index and readonly
  position checks keep indexing distinct from mutable array access.
- analysis.rs::string_at, string_locale_case, string_code_point_at,
  string_from_codes and string_raw/string_raw_arguments implement the public
  spelling rewrites before ordinary dispatch. Internal codePointAt#slot and
  groupBy-style helper spellings are not additional public APIs.
- analysis.rs::string_normalize/string_well_formed check their independent public
  contracts. syntax.rs::string_html_target recognizes all thirteen wrapper names.
- runtime_strings.rs supplies byte slicing/search/padding, scalar charAt, Unicode
  casing/trim, templates, identity and code-unit/code-point construction. The old
  internal tsr_normalize(trim/lower) helper is not the public Unicode normalize
  contract; public emission selects tsr_string_normalize and normalization tables.
- emit.rs targets implement scalar character-code reads, helper argument order,
  source-located failures, string construction and static raw concatenation.
- runtime.rs::STRING_SPLIT yields scalar strings for empty separator;
  runtime_normalization.rs and runtime_normalization_tables.rs implement Unicode
  normalization. emit.rs includes normalization support only when its generated
  helper is referenced. String support is otherwise selected through existing
  emitted runtime families; this is not a per-method binary-size guarantee.

No external package or manually supplied linker flag is required. Some methods
allocate intermediate storage, notably current string.at lowering; borrowed input
alone does not imply an allocation-free operation. This audit preserves current
semantics and records costs rather than silently changing indexing units.

Catalog signatures and documented examples match these reviewed source contracts.
Native string reflection via Object uses UTF-16 positional names; that separately
reviewed exception does not redefine the byte length or scalar access rows here.

## Focused acceptance

SQLite `tsrust-api-string-*.log` records targeted compiler/native boundary,
normalization, construction and ownership checks plus editor checks. All 73 catalog
examples run in one native program with isolated blocks and module-level type
declarations, compared to expected output after formatting/idempotence checks.
The first example harness incorrectly nested a type declaration; its failure is
retained, and the corrected harness reruns the same examples without source semantic
changes. Whole-project formatting runs before checkpoint. No timing or complete
family regression is claimed.
