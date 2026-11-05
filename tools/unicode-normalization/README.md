# Unicode normalization data

The compiler vendors Unicode 16.0.0 normalization data to match the pinned Node
24.11.1 / ICU 77.1 / Unicode 16.0 reference. Frontend checking and native builds
need no network access or additional Cargo dependency.

Official sources are [UnicodeData.txt](https://www.unicode.org/Public/16.0.0/ucd/UnicodeData.txt),
[DerivedNormalizationProps.txt](https://www.unicode.org/Public/16.0.0/ucd/DerivedNormalizationProps.txt)
and [NormalizationTest.txt](https://www.unicode.org/Public/16.0.0/ucd/NormalizationTest.txt).
The algorithm follows [Unicode normalization](https://www.unicode.org/reports/tr15/).
Exact source SHA256 values and source URLs are retained in `sources.json`.
The [Unicode license](data/LICENSE.txt) is retained beside the data and copied
into generated tables, so emitted native Rust sources retain the notice.

Run `python3 tools/unicode-normalization/generate.py` from the repository root.
Generation validates the pinned source hashes and produces
`crates/tsrust-compiler/src/runtime/normalization_tables.rs`,
`assigned-ranges.json` and `assigned-ranges.rs` offline. The pinned source data contains 934 nonzero
combining classes, 5,913 decomposition mappings, 961 composition pairs and 8,739
mapping scalars. Hangul decomposition/composition is handled algorithmically.
Full composition exclusions come from the derived property file.

The generated decomposition table expands the original chains ahead of time and
packs each leaf's combining class into the upper bits of its scalar word. There
are 9,221 packed mapping words, including 16 canonical mappings whose compatibility
expansion differs. Decomposition kind 0 denotes a canonical mapping, kind 1 is
compatibility-only, and kinds 2 and above index the alternative compatibility
ranges. Composition pairs still come from the original, unexpanded mappings.
This removes the runtime decomposition stack and repeated leaf lookups; ordering
and composition remain runtime operations. Derived data adds 2,120 read-only bytes
relative to the raw mapping layout and introduces no input-size limit.

Generation validates pinned counts, scalar validity, kind/range widths and the
class-zero Hangul Jamo invariant. The focused expanded-decomposition witness
compares both modes against the raw UnicodeData mappings for every valid scalar
at O0 and O3, including all alternative mappings, and verifies scratch allocation
removal and complete output cleanup.

Combining-class and decomposition tables include derived offsets for each
256-code-point page through the last stored key. Lookups search only that page;
empty pages and scalars beyond the last page have no matching entries. These
two read-only indexes use 2,510 bytes. Generation checks sorted unique keys,
offset coverage and the pinned table sizes; normalization data is unchanged.

The focused compiler test `string_normalize_passes_official_unicode_conformance_rows`
checks all 19,965 official rows (399,300 relationships) and all 275,446 assigned
scalars outside Part 1 in all four forms (1,101,784 identity checks). Node validation
of the same rows is retained in `target/editor/string-normalize-node-conformance.log`.
Changing the data version requires a deliberate source-manifest update, regenerated
tables, conformance checks, editor documentation review and refreshed benchmark
source/evidence identities. Do not silently use a host Python Unicode database.
