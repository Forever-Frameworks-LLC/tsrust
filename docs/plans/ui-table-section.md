# Paired Table section

Continue from accepted Kbd/Markdown checkpoint `44c9fa3b` and storage checkpoint
`ff9dbd70`. The real preview is `tsrust dev --target web --port 4317`; keep it live.
Performance acceptance is `045d8489`, with original observations in SQLite.

The pinned Table source has eight exports: Table, TableHeader, TableBody,
TableFooter, TableHead, TableRow, TableCell and TableCaption. Preserve the outer
`table-container`, all slot names, caller overrides and owned children. Source
blob: `160cf7aa14b8467a777eb2405053d2d968f641b9` at shadcn commit
`daaf156ee00ed91cc7e6bdd937c1b4973152d141`. Nova stylesheet blob:
`293204b2bf08ddfd86dbc49b75dc4749cd2135ed`. The verified source and Table stylesheet
excerpt are current-gate records under `/private/tmp/tsrust-table-*`.

## Implementation order

1. Finish intrinsic column constraints and their native layout integration. The
   `crates/tsrust-ui/src/table.rs` constraints now connect through native_table.rs
   to shaped geometry, with passing native/model witnesses. Full section
   acceptance remains pending. Measure cells across
   headers/body/footer, preserve source/ownership/accessibility order and keep
   narrow tables horizontally scrollable. Hidden retained rows must not corrupt parent column measurements. Nested
   portable tables reject explicitly until recursive constraint sizing is implemented. Preserve the settled explicit layout
   contract of ordinary `View as="table"` users; add checked table display utilities
   rather than changing all semantic tags implicitly.
2. Add the required paired semantics and utilities: tfoot/caption, cell colSpan and
   header scope, table display/caption/vertical alignment, horizontal scrolling,
   numeric pixel widths used in examples, selected row metadata and the bounded
   row/last-child/expanded-descendant style conditions used by Nova. Share metadata
   transport, cleanup, layout invalidation and browser attribute updates. Keep the
   existing ownership and naming validation. The HTML colSpan author range is
   1–1000; reject invalid values instead of applying the DOM's silent coercion.
3. Keep compiler facade types, API compatibility, LSP hover/completion/help and
   documentation examples consistent. Do not expose adapter/lifetime machinery.
   Table wrappers belong in editable offline files with shared attribution and one
   current install version. Do not add installable placeholder components.
4. Complete the paired reference with data, caption/footer, constrained width,
   caller classes, row state and theme examples. Include explicit unsupported
   cases rather than quietly dropping props. Row spans, unrestricted DOM events/
   refs, full CSS table border models and motion need an implementation or a clear
   limitation; this section must not imply a complete browser layout engine.
5. After the section is coherent, batch native/model, generated-native, installer,
   LSP and actual browser checks. Reuse existing OS-window, ownership, style and
   navigation evidence. Preserve successful checks; repair/rerun affected failures
   only. No full compiler/LSP/SQL matrix is triggered by this component section.

## Acceptance and retention

Native and web must agree on column sharing, overflow, span effects, header roles,
retained identity, dynamic metadata and failure cleanup within the declared
portable contract. Browser/native font metrics may differ; neither target may
replace real tables with unrelated stacked cards. Preserve source order while
placing captions/sections according to the selected layout.

Use one Cargo target, no incremental compilation and one build owner. Avoid
rebuilding complete unit-test images when a standard-only helper or focused
integration target provides the required witness. Keep all new checks ready for
later regressions without rerunning unrelated accepted fixtures. Import verified
closed records into SQLite and prune obsolete outputs before handoff.

Primary behavior references: [HTML table cells](https://html.spec.whatwg.org/multipage/tables.html#attributes-common-to-td-and-th-elements) and [CSS table width distribution](https://www.w3.org/TR/css-tables-3/#distributing-width-to-columns). CSS Tables 3 is a draft, not a conformance claim. Compare the declared subset with actual browser behavior and relevant primary WPT witnesses. Pixel preferences now preserve constrained columns while surplus goes to unconstrained columns. Percentage cell widths remain an explicit unsupported case.
