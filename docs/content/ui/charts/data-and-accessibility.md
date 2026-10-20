<!-- tsrust-docs
{"id":"reference-ui-charts-data-and-accessibility","group":"charts","order":20,"status":"partial","targets":["web","native"]}
-->
# Chart data and accessibility

Chart data must preserve numeric meaning. Exact decimals, wide integers and timestamps require explicit conversion or formatting; do not silently round them to fit a plot.

Every chart needs a readable data-table alternative, keyboard-accessible values, labeled axes/series and non-color distinctions. Tooltips must work with keyboard focus and native pointer input. Empty, loading and error states need examples.

Shared chart primitives and their interaction contracts remain unimplemented. This page records requirements rather than offering a fake API.
