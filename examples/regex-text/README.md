# Reusable regex and text ranges

`compile(pattern, options?)` returns an owned reusable Regex or a RegexError.
Matching follows Rust regex syntax: Unicode is enabled by default; lookaround and
backreferences reject. Inline flags such as `(?i)` configure the pattern.
`escape(text)` quotes regex metacharacters for literal matching.

`RegexOptions` has optional `sizeLimit`, `dfaSizeLimit` and `nestLimit` fields.
Omitted fields preserve backend defaults: a 10 MiB compiled-NFA size limit and
2 MiB hybrid-DFA cache. Limits are nonnegative integers representable by the
execution target; `nestLimit` is 0..4294967295. Explicit zero is passed through.
Negative/unrepresentable options return Range before compilation. Invalid syntax
returns Syntax; an exceeded compiled-size limit returns SizeLimit. Compile is the
fallback code for an otherwise unclassified backend compilation error.

`isMatch(text)` returns a boolean. `find(text)` returns a RegexMatch or undefined;
`findAll(text)` returns an eager owned array. Every range has `start` inclusive and
`end` exclusive, measured in UTF-8 bytes. For `xé🦀 é`, `(é)(🦀)?` matches [1,7)
and [8,10). These are byte positions, not JavaScript UTF-16 indices.

`captures(text)` returns RegexCaptures or undefined; `capturesAll(text)` returns
an eager array. `match` holds the whole-match range. `groups[n - 1]` holds numbered
capture n; it excludes group 0. An absent capture is undefined, while an empty
capture has equal start/end offsets. Named captures are also numbered in pattern
order. No capture names or matched substrings are copied into each result.

`replace(text, replacementTemplate)` replaces the first match;
`replaceAll(text, replacementTemplate)` replaces all matches. Both return owned
text, including when no match occurs. Templates use Rust expansion: `$1`, `${1}`,
`$name`, `${name}` and `$0` for the whole match. Missing references expand to empty
text; `$$` inserts a literal dollar. Bare references consume the longest
alphanumeric/underscore name: `$1x` names capture `1x`, while `${1}x` appends x to
capture 1. These templates are not JavaScript replacement templates.

`split(text)` returns ranges for the segments between matches, preserving empty
segments at boundaries and between adjacent delimiters. Slice the original text
explicitly when owned segment strings are needed. No-match input gives one range
covering the whole input; an empty no-match input gives [0,0).

String `match(regex)`, `matchAll(regex)` and `search(regex)` use the same contracts.
`match` returns the first capture record; `matchAll` returns eager capture records;
`search` returns the first byte offset or -1. There is no mutable lastIndex.
Zero-width iteration advances at valid Unicode boundaries and suppresses empty
matches immediately following a match at the same ending position. An empty
pattern over `é🦀` produces [0,0), [2,2), [6,6).

Matching borrows the compiled owner and text only during the call. Returned ranges,
capture records, arrays and replacement strings are independent values; mutation
or disposal of them does not change Regex or the original text. Ranges carry no
text reference: retain the corresponding original text to interpret them.

Compile once outside repeated searches. Eager iteration allocates result arrays
and can have quadratic total search work for adversarial patterns/haystacks;
there is no implicit result cap or global regex cache.

The example prints `Regex text checks passed`. Public API integration and native
example verification remain pending while this coherent feature is implemented.
