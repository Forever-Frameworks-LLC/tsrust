# Paired Table implementation

This section is in validation. It is not accepted until generated-native, editor
and actual browser checks complete. Reuse the earlier theme, ownership, window,
navigation and Markdown evidence; no full regression or benchmark run is implied.

## Contract

The editable offline Table source has the eight pinned Base UI exports: Table,
TableHeader, TableBody, TableFooter, TableHead, TableRow, TableCell and TableCaption.
It retains tags, slot names, caller overrides and the outer table-container.
Source commit: `daaf156ee00ed91cc7e6bdd937c1b4973152d141`, Table blob
`160cf7aa14b8467a777eb2405053d2d968f641b9`, Nova stylesheet blob
`293204b2bf08ddfd86dbc49b75dc4749cd2135ed`. The source identities were verified
against Git blob hashes. Installation requires compiler API 11 and shares the
existing attribution file.

Ordinary semantic View tags retain their explicit flex/grid contract. New table
utilities opt into shared native column constraints and browser table display.
The native renderer measures shaped cell content across header/body/footer,
accounts for spans, then applies shared tracks. Captions change visual placement
without reordering owned nodes, focus or accessibility. Narrow tables overflow
the local container. Pixel dimensions retain logical-pixel precision.

Checked construction-time colSpan accepts 1–1000; scope accepts the four HTML
header scopes only on th. Native accessibility exposes caption/footer/header
roles and column spans. Live data-state updates retained styling metadata, not
ARIA selection. Checked row-descendant, final-row and expanded-descendant rules
provide the current Nova styles. Unsupported cases remain listed on the reference
page rather than implying a complete HTML/CSS table engine.

## Evidence so far

The native/model batch passes 20 checks, including seven new column/semantic/style
and actual shaped native geometry witnesses. The broad substring filter also ran
existing editable-control witnesses; their success is retained and need not be
repeated. The constrained-column repair adds passing native width and hidden-row witnesses; pure constraints verify that unconstrained columns receive surplus first. Shared validation also passes explicit nested-table and malformed-group rejection witnesses. The run has no GPU-window requirement. Generated native installation and ownership, current LSP behavior, paired documentation execution and browser interaction remain pending. Initial compiler failures preserve the original evidence: unused catch payloads were replaced with payload-free catches, and the metadata fast path now reads config.scope after scope transfers into that record. A direct ordinary-file facade check correctly refuses private runtime imports; it is not facade acceptance. The ordinary module/compiler path supplies the actual check.
