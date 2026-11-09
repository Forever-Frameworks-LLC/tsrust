# Explicit locale operations

Import from std:locale. parseLocale(text) returns a checked canonical Locale; formatLocale returns its tag. Create reusable Collator and NumberFormat owners with explicit locale input and optional overrides. No operation consults the host language. Collator.compare and string.localeCompare(other,collator) return the same -1/0/1 comparison. Locale keywords carry numeric ordering and case-first preferences.

createSegmenter() exposes locale-independent grapheme segmentation. segment(text) returns owned {start,end} UTF-8 byte ranges with exclusive ends; empty input returns[]. Combining characters and joined emoji stay together. Ranges do not retain text.

NumberFormat.format accepts exact int or Decimal. Existing Decimal owners are normalized and do not retain original trailing-zero spelling. Default formatting preserves the normalized numeric value without float conversion or implicit rounding. grouping supports auto, never, always and min2; absent uses backend auto, and always currently equals auto. Explicit fractionDigits0..32768 requests fixed fractional scale, padding zeros and using half-even rounding. It is a formatting limit derived from the backend's i16 position, unrelated to Decimal input precision. Negative or unrepresentable counts return Range when creating the formatter.

Malformed tags return Syntax; unavailable data returns Data. Unsupported option fields and enum values are rejected by the checker. All owners are opaque; calls borrow inputs and return independently owned strings/ranges. Locale owners, including NumberFormat, can move to thread workers. Formatting borrows immutable configuration; started tasks can safely retain a locally created formatter across await.

Locale data uses backend fallback; a distinct bundle for every syntactically valid tag is not promised. Numbering-system digits and symbols can fall back separately: en-u-nu-arab uses Arabic digits with English comma/period separators. Unknown well-formed locales may use root data. This subset does not promise full ECMAScript Intl, host locale selection, currency/date/plural/list/relative-time formatting, or search collation data.
