<!-- tsrust-docs
{"id":"reference-language-values","group":"language","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"boolean","name":"valueOf"},{"receiver":"boolean","name":"toString"},{"receiver":"number","name":"valueOf"},{"receiver":"number","name":"toFixed"},{"receiver":"number","name":"toExponential"},{"receiver":"number","name":"toPrecision"},{"receiver":"number","name":"toString"},{"receiver":"number","name":"toLocaleString"},{"receiver":"int","name":"valueOf"},{"receiver":"int","name":"toString"},{"receiver":"int","name":"toLocaleString"}]}
-->
# Values and numeric types

Use explicit types at boundaries. `int`, `number`, strings and booleans have different native representations. Optional and nullable values must be narrowed before use.

Use [checked numeric conversions](/standard-library/numeric) for widths and ranges, [exact decimals](/standard-library/decimal) for decimal data and [bytes](/standard-library/bytes) for binary values. Do not route an exact database value through a floating-point conversion to make it fit.

A complete syntax table and conversion walkthrough are still being written. The method references below retain their actual ranges, errors and examples.
