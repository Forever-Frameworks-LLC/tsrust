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
`assigned-ranges.json` and `assigned-ranges.rs` offline. These contain 934 nonzero
combining classes, 5,913 decomposition mappings, 961 composition pairs and 8,739
mapping scalars. Hangul decomposition/composition is handled algorithmically.
Full composition exclusions come from the derived property file.

The focused compiler test `string_normalize_passes_official_unicode_conformance_rows`
checks all 19,965 official rows (399,300 relationships) and all 275,446 assigned
scalars outside Part 1 in all four forms (1,101,784 identity checks). Node validation
of the same rows is retained in `target/editor/string-normalize-node-conformance.log`.
Changing the data version requires a deliberate source-manifest update, regenerated
tables, conformance checks, editor documentation review and refreshed benchmark
source/evidence identities. Do not silently use a host Python Unicode database.
