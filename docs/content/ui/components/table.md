<!-- tsrust-docs
{"id":"component-table","group":"components","order":90,"status":"partial","targets":["web","native"],"component":"table","examples":["invoice-table","table-states","table-overflow"]}
-->
# Table

Present tabular data with a caption, column headings, body rows and an optional
footer. Columns share their width across every section on web and native.

## Installation

```sh
tsrust ui add table --offline
```

Import the eight components from the editable local `table.tsrx` file. The outer
`table-container` owns horizontal overflow; `className` styles the inner table.
Components preserve the Base UI + Nova export names, semantic tags and slots.

## Composition

Use `TableHeader` / `TableBody` / `TableFooter` around `TableRow`. Put `TableHead`
or `TableCell` inside each row. `TableCaption` describes the entire table and
appears below it by default. Malformed row/group children reject on both targets. Source order remains keyboard and accessibility
order even when the caption is placed visually below the rows.

Table adds no sorting, filtering, pagination or row-selection state. Application
code owns data and handlers. Use a real Button in a cell when a row needs an action.

## Props

| Component | Tag | Default slot |
| --- | --- | --- |
| Table | table, inside div | table, table-container |
| TableHeader | thead | table-header |
| TableBody | tbody | table-body |
| TableFooter | tfoot | table-footer |
| TableRow | tr | table-row |
| TableHead | th | table-head |
| TableCell | td | table-cell |
| TableCaption | caption | table-caption |

All components forward supported View props, children, caller slots and reactive
classes. `TableHead` and `TableCell` accept construction-time `colSpan` integers
from 1 through 1000. Invalid spans throw instead of being clamped. `TableHead`
accepts `scope="col"`, `"row"`, `"colgroup"` or `"rowgroup"`; other tags reject scope.

## Selection and customization

`data-state="selected"` paints the selected surface. A live state expression
updates the retained row. This is styling metadata; it does not claim ARIA
selection or turn the table into a keyboard grid. The default row hover and
expanded-descendant surfaces use the current muted theme token.

Use `text-right` for numeric columns, `w-[100px]` for a column preference and
`whitespace-normal` when a cell should wrap. Caller classes override defaults.
Muted surfaces, foreground, border, typography and spacing follow the enclosing
theme. Width preferences participate in intrinsic sizing; they do not clip text
or promise identical font metrics across platforms.

## Overflow and accessibility

A narrow container scrolls horizontally rather than cutting off columns. A
minimum table width can keep related columns readable. Name and focus an outer
scroll region when the application needs direct keyboard scrolling.

Provide a useful caption, use header cells for headings and set header scope
where association would otherwise be ambiguous. Nested controls keep ordinary
focus order. Selection, hidden rows and reactive classes preserve node identity;
removing a subtree releases its children and evaluator captures once.

## Portable limits

This contract covers intrinsic columns, spans, normal header/body/footer groups
and top/bottom captions. It does not implement nested tables, rowSpan, col/colgroup elements,
CSS fixed table layout, separate border spacing, collapsed cell-border conflict
resolution, percentage cell widths, unrestricted DOM events/refs or motion.
Checkbox-dependent padding awaits the Checkbox component. Nested portable tables reject before either adapter renders them. Native layout uses
shared logical-pixel constraints; font metrics and surplus-column distribution
can differ from browser layout. These limits are not a full HTML/CSS table claim.
