<!-- tsrust-docs
{"id":"strings","group":"standard-library","order":10,"status":"partial","targets":["web","native"],"examples":["strings-trim"],"api":[{"receiver":"string","name":"trim"}]}
-->
# Strings

Tsrust strings store UTF-8 text. Methods preserve Unicode boundaries and report invalid conversions.

## trim

`trim()` returns text with surrounding Unicode whitespace removed. It takes no arguments and does not mutate the receiver. Interior whitespace remains unchanged.

The example returns `Hello` from `  Hello  `. U+0085 is removed at the edges; U+FEFF is preserved. Those boundary cases matter when porting JavaScript behavior.

Static source text can retain a borrowed slice. Trimming owned input produces independent owned output. Use the generated API description for the current checked signature and ownership details.
