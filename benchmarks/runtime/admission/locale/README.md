# Explicit locale admission and profile

Selected backend: exact ICU4X components with icu_provider2.3.1/sync, which uses thread-safe Arc cart storage instead of Rc. The feature has no dependencies and changes storage capability, not locale defaults, options, compiled data or fallback. Direct-Rust admission passed; public compiler/native/editor/export/benchmark acceptance remains pending.

## Public contract

Import from std:locale. parseLocale(text:string):Result<Locale,LocaleError> parses explicit BCP47 input and applies the extended canonicalizer; formatLocale(locale):string returns canonical spelling. Malformed tags, including underscores, return Syntax; no host locale is selected.

createCollator(locale,options?):Result<Collator,LocaleError> makes a reusable owner. Collator.compare(left,right):int and String.localeCompare(other,collator):int share the same borrowed comparison and return -1/0/1. Options map strength primary/secondary/tertiary/quaternary/identical, alternateHandling nonIgnorable/shifted, maxVariable space/punctuation/symbol/currency, and caseLevel boolean. Unset fields preserve backend/locale defaults. Numeric ordering and case-first preferences use explicit locale Unicode keywords. Default compiled data omits search collations.

createSegmenter():Segmenter exposes locale-independent extended graphemes. segment(text):SegmentRange[] returns owned {start:int,end:int} half-open UTF-8 byte ranges. Empty input returns[]. Ranges retain no text and do not copy substrings.

createNumberFormat(locale,options?):Result<NumberFormat,LocaleError> creates a reusable formatter. format(value:int|Decimal):string preserves exact normalized Tsrust Decimal values, never f64. Existing Decimal normalizes input and does not retain input trailing zeros or negative-zero spelling. Absent options perform no rounding/padding and preserve backend Auto grouping. grouping supports auto/never/always/min2; always currently equals auto. Explicit fractionDigits0..32768 requests fixed displayed scale with zero padding and half-even rounding. Checked negation and i16 conversion map to positions0..-32768, including i16::MIN; Decimal's input precision does not impose a formatting padding cap. Invalid counts return Range at construction before mutation.

LocaleError has code/message. Syntax represents parser failures; Data represents backend data errors; Range represents checked numeric option conversion; Option covers unsupported runtime option enum text. Unsupported source fields/enum values are checker diagnostics. Opaque owners and inputs are borrowed during calls; results own their strings/ranges. The selected actual Locale/Collator/Segmenter/NumberFormat wrappers all passed Send+Sync compile witnesses. No new generic suspension analysis is introduced.

## Backend fallback and data

ICU fallback is data-driven and may use parent/script/root bundles for well-formed locales. A distinct bundle for every tag is not promised. Numbering-system digits and symbols can fall back independently: selected ICU4X formats en-u-nu-arab1234.50 as `١,٢٣٤.٥٠`, retaining English comma/period separators; pinned Node24.11.1 ICU77.1/CLDR47 emits `١٬٢٣٤٫٥٠`. Packaged icu_decimal2.3.0 decimal_formatter.rs64-104 implements this split fallback and its upstream test269-270 explicitly asserts the English-symbol/Arabic-digit result:
https://github.com/unicode-org/icu4x/blob/icu%402.3.1/components/decimal/src/decimal_formatter.rs#L269-L270

The oracle asserts BOTH exact raw outputs and reports one untimed backendDifferences record. Every other matched comparison remains strict. Timed numeric cases use only en/de/hi-IN. No custom symbol normalization is admitted. Selected data headers identify CLDR48.2.1 / ICU release-78.1rc. This is an explicit-locale subset, not full ECMAScript Intl or host-user locale discovery.

## Exact dependency and feature closure

Direct pins: collator2.3.1, decimal2.3.0 (alloc for format_to_string), locale2.3.1, segmenter2.3.0 (default-features=false, compiled_data only), fixed_decimal0.7.3 (no float feature), provider2.3.1 (sync), and existing bigdecimal0.4.11. Individual components are selected, never the ICU meta-crate. All preexisting production versions remain unchanged. ICU packages require Rust1.88, fixed_decimal1.82; package licenses/checksums/MSRVs are in identity inputs.

lock-closure.json retains all57 locked registry identities with verified archive checksums. closure.json records54 ACTUALLY built identities and strict selected features. Cargo metadata resolve.nodes over-approximates weak optional edges: its inactive normalizer→properties request incorrectly described properties compiled_data; actual Cargo tree/build JSON have properties features[], serde without alloc and serde_core without alloc. icu_plurals/plurals_data/properties_data remain locked but are not built. Metadata features are not treated as build evidence.

production-graphs.json records current workspace/backend/native/reference manifest+lock hashes and build-aware normal/build selection (compiler validation includes dev edges). Actual native54 identities/features equal measured70545 sync compiler-artifact JSON. Active backend and handwritten locale references use identical features and identities minus unrelated color0.3.3. References reuse existing backend-dependencies+locale; broad parity-dependencies is unchanged, including B06 task-join. Workspace validation's extra host/target variants are explicitly recorded. Only eight older production transitives differing from the original standalone profile were newly audited; cached RustSec audit found0vulnerabilities/0warnings. All57 archives were verified. No product dependency version was upgraded.

Official metadata/source:
- https://github.com/unicode-org/icu4x/tree/icu%402.3.1
- https://index.crates.io/ic/u_/icu_collator
- https://index.crates.io/ic/u_/icu_decimal
- https://index.crates.io/ic/u_/icu_locale
- https://index.crates.io/ic/u_/icu_segmenter
- https://index.crates.io/ic/u_/icu_provider
- https://index.crates.io/fi/xe/fixed_decimal
- https://rustsec.org/

## Correctness, allocation, footprint and timing

43 correctness records comprise36 matched contracts, one explicit untimed fallback difference, and six backend-only limit controls. Cases cover malformed/canonical tags, numeric/canonical-equivalent collation, empty/combining/CRLF/emoji/flags, exact negative/large normalized Decimal values, backend trailing-zero controls, grouping, half-even ties, integer/short-fraction padding, unchanged-on-invalid options and32767/32768-scale edges. Large outputs are fully checked natively; JSON reports their byte length/prefix/zero count. i64 extreme invalid-option identities are strings to avoid JSON precision loss.

99 rows/process separate construction/reuse, shared String.localeCompare, fixed input and normalized BigDecimal.to_plain_string→fixed_decimal parsing, explicit rounding, and grapheme/collation scaling at16/64/256/1024 repetitions. Fixed-decimal text roundtrip is a separate floor/control, not a bound for BigDecimal. Normalization occurs before timing because Tsrust Decimal already owns a normalized value. Owned ranges and copied strings consume identical boundary checksums; long comparisons share a full common prefix. Locale parse/canonicalization includes owned spelling serialization. These are direct Rust prototypes, not generated facade performance.

Timing uses three processes,31samples after8warmups,32operations/sample and quiet gating. Allocations use a separate instrumented binary and count allocations/reallocations/requested/retained/peak bytes. Six same-source footprint controls retain the shared BigDecimal/fixed_decimal baseline and isolate locale/collator/decimal/grapheme/all roots. Deltas include reachable service CODE PLUS DATA and are not additive or pure data-byte totals. Mach-O size output includes virtual __PAGEZERO; executable file bytes are the footprint comparison.

Original standalone results, production Rc base80999 and sync91144 remain distinct evidence. Sync passed the same43 correctness controls,99rows×3×31 and six footprint controls plus concrete public-wrapper Send+Sync compile requirements. All-service executable2103864→2104184bytes (+320). Across99 pooled median ratios, median sync/base1.0025, range0.8187..1.104; sampled normalized Decimal bridges cost2.4–7.0% more. Selection claims safe thread capability, NOT zero overhead. Kernel/reference/footprint/public-runtime source hashes and selected active graph match the successful sync evidence; no retiming of unchanged code/graph is needed. Public generated acceptance remains pending.

## Reproduction (only after a heavy-slot lease)

Use absolute pinned Node24 and the single coordinated target:

```sh
export CARGO_TARGET_DIR=/Users/seancotter/Desktop/dev/learning/tsrust/target
export CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
python3 /tmp/tsrust-watch.py cargo +1.96.1 build --release --offline --locked --features thread-safe-locale --manifest-path benchmarks/runtime/admission/locale/Cargo.toml --message-format=json > /tmp/tsrust-locale-selected-sync-artifacts.jsonl 2> /tmp/tsrust-locale-selected-sync-build.log
python3 /tmp/tsrust-watch.py /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node benchmarks/runtime/admission/locale/profile.mjs --candidate=sync --artifacts=/tmp/tsrust-locale-selected-sync-artifacts.jsonl > /tmp/tsrust-locale-selected-sync-profile.json 2> /tmp/tsrust-locale-selected-sync-profile.log
```

The driver checks all57 lock checksums, every active54 package and exact features, source/production graph hashes, root candidate feature and release artifact identities. It compiles separate timing/allocation binaries plus six footprint controls, checks correctness before timing and cleans scratch executables. The sync variant includes actual public locale/runtime.rs and requires Send+Sync for all four wrappers. Historical Rc control remains available through a build WITHOUT thread-safe-locale and --candidate=base; it explicitly omits the selected provider/sync feature and is not called the selected production graph. Preserve original result files rather than overwrite them. Native compiler/editor/docs/examples/export/matched benchmark validation remains separately required.
