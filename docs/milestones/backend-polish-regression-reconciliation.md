# Backend polish regression reconciliation

Reviewed 2026-10-28 at `e33f4674`. This reconciles the 26 failed targets in the
original completed compiler/LSP milestone against subsequent focused repairs.
It is accumulated acceptance across source checkpoints, not a claim that one
complete run passed on this exact commit. Original successful cases remain retained;
the workflow requires rerunning affected cases rather than restarting the matrix.

The original disposable diagnostic log is `/tmp/tsrust-backend-milestone.log`.
Follow-up filenames below are also under `/tmp`; their conclusions are recorded
here so routine command logs need not enter SQLite or become permanent artifacts.
Source and implementation history remain in Git. Timing measurements remain in SQLite.

## Compiler targets

| Original failed target | Reconciled result | Follow-up evidence |
| --- | --- | --- |
| `cache_retention` | All 4 current cases pass. Names changed with the accepted removal of implicit cache quotas; staging, concurrency and private-copy protections remain. | `tsrust-cache-retention-repair.log`; cache-policy checkpoint `c0eeb633` |
| `data_unions` | All 17 pass after spread storage repair. | `tsrust-spread-repair.log` |
| `dev_aliases` | All 3 pass, including failed-edit recovery and cache reuse. | `tsrust-cli-fixture-repair.log` |
| `documentation_content` | Both failed cases pass: authored native data and public receiver reference coverage. Other cases passed originally. | `tsrust-doc-publication-repair.log`, `tsrust-doc-reference-repair.log` |
| `exception_handling` | All 22 pass. The registration guard is now `prepared_test_entry_runs_cleanup_and_stops_registration`, preserving cleanup and stopped-registration behavior. | `tsrust-registration-tail-repair.log` |
| `generic_default_void` | All 3 pass. | `tsrust-provider-regression-repair.log` |
| `http` | Failed catalog case passes with native socket execution; other cases passed originally. | `tsrust-http-catalog-socket-repair.log` |
| `interactive_cli` | All 7 pass, including PTY navigation, cancellation, colors and small terminals. | `tsrust-cli-fixture-followup.log` |
| `native_declarations` | All 64 pass. | `tsrust-provider-regression-repair.log` |
| `project` | All 95 pass. | `tsrust-provider-regression-repair.log` |
| `project_scaling` | Required case passes; the existing optional ignored case remains ignored. | `tsrust-provider-regression-repair.log` |
| `runtime_cache_cli` | All 4 pass after union-array carrier repair and complete theme fixture setup. | `tsrust-union-carrier-repair.log` |
| `rust_bridge` | All 6 pass using the current binding schema. | `tsrust-cli-fixture-followup.log` |
| `temporal_values` | Failed documentation example batch passes; other cases passed originally. | `tsrust-temporal-catalog-repair.log` |
| `ui` | All 3 failed cases pass: compiler API compatibility, scoped theme layout and color serialization. | `tsrust-compiler-api-repair.log`, `tsrust-retained-native-ui.log`; latest repairs `e33f4674` |
| `ui_application` | Failed docs foundation case passes formatting, native assertions and emitted renderer checks. | `tsrust-retained-native-docs-fixed.log`; `e33f4674` |
| `ui_content` | Isolated table renderer build passes (1 test, 80.55s) after the original contention timeout. | `tsrust-retained-table-build.log` |

## Language-server targets

| Original failed target | Reconciled result | Follow-up evidence |
| --- | --- | --- |
| `call_spread` | Both cases pass with current signature formatting. | `tsrust-spread-lsp-repair.log` |
| `collections_catalog` | Complete catalog example case passes with canonical temporary paths. | `tsrust-collections-catalog-repair.log` |
| `http` | All 4 pass, including body default/ownership help. | `tsrust-editor-repair.log` |
| `intelligence` | Declared callable labels and all 15 scoped cases pass. Backend compiler witnesses and native catalog continuation pass. Retained UI compiler/native catalog wrappers still fail on historical documentation gaps; see scope below. | `tsrust-editor-repair.log`, `tsrust-editor-scoped-repair.log`, `tsrust-backend-catalog-check.log`, `tsrust-backend-catalog-native-http.log`; `tsrust-retained-catalog.log` |
| `native_environment_catalog` | Complete example case passes. | `tsrust-editor-repair.log` |
| `native_file_catalog` | Complete example case passes. | `tsrust-editor-repair.log` |
| `native_threads` | All 12 pass. | `tsrust-editor-repair.log` |
| `native_time_catalog` | Complete example case passes. | `tsrust-editor-repair.log` |
| `ui` | All 3 failed cases pass: current ExampleCard/gallery dependencies and spacing-variable hover. The shared hover panic was repaired rather than hidden by retries. | `tsrust-retained-editor-fixtures.log`, `tsrust-retained-gallery.log`, `tsrust-retained-card-help-fixed.log`; `d23dbf20` |

## Remaining retained UI catalog failures

The isolated retained UI catalog checks still fail: the compiler witness reports
`unverified family UiComputed`, and the native witness reports `UiComputed.get`
lacks a fenced example. Catalog inspection finds 59 entries without fenced
examples and 49 without executable examples across std:ui, UiComputed and UiSignal.
These historical UI catalog gaps remain in the default tests; no assertions were
removed or skipped. Completing that catalog belongs to the explicitly deferred UI
scope. Backend catalog acceptance does not imply a green complete suite.

## Separate requirements

This table does not close platform execution, production deployment, long-session
editor validation or the remaining public-page audit. Follow the
[public content checklist](../plans/public-documentation-content-audit.md),
[paired installation plan](../plans/paired-toolchain-updates.md) and
[HTTP reader measurement record](../plans/http-request-body-readers.md).

Backend and workspace all-target checks passed without warnings/errors for
`d23dbf20`; the subsequent `e33f4674` changes are accepted fixtures and documentation.
No full regression or performance run should be repeated solely to replace this
accumulated evidence with a single green transcript.
