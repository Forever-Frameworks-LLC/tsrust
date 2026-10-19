<!-- tsrust-docs
{"id":"strings","group":"standard-library","order":10,"status":"partial","targets":["web","native"],"examples":["strings-trim"],"api":[{"receiver":"string","name":"trim"}]}
-->
# Strings

Tsrust strings use UTF-8 storage. The string reference will combine checked API descriptions, runnable examples and comparable benchmark evidence at each method.

## trim

`trim()` returns text with surrounding Unicode whitespace removed. It takes no arguments and does not mutate the receiver. Interior whitespace remains unchanged.

The example returns `Hello` from `  Hello  `. U+0085 is removed at the edges; U+FEFF is preserved. Those boundary cases matter when porting JavaScript behavior.

Static source text can retain a borrowed slice. Trimming owned input produces independent owned output. Use the generated API description for the current checked signature and ownership details.

## Benchmark evidence

**No accepted isolated trim measurement is currently mapped.** Existing pipelines that combine trimming with other operations cannot be presented as a measurement of `trim()` alone.

Future inline method comparisons and overview graphs will use the same curated SQLite exports, with matching source/measurement scope and explicit provenance. Documentation builds will not run benchmarks or ship the canonical evidence database.
