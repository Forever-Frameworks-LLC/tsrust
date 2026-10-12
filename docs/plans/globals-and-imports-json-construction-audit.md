# JSON scalar construction API review

Five imported identities extend the reviewed inventory from 413 to 418. These
construct native JSON values explicitly; they introduce no globals or JavaScript
coercion. Dependencies continue to follow actual use.

| Identity | Contract | Ownership |
| --- | --- | --- |
| std:json::nullValue | No arguments; returns JSON null, distinct from absence | Owned result |
| std:json::fromBoolean | Boolean to JsonValue, no coercion | Copies scalar |
| std:json::fromInt | Exact signed 64-bit integer to JsonValue | Copies scalar; no floating conversion |
| std:json::fromNumber | Finite floating value to Result<JsonValue, JsonError>; rejects NaN/infinities with number-nonfinite, offset zero | Copies scalar; preserves negative zero |
| std:json::fromString | UTF-8 string contents to JsonValue, without parsing | Consumes string; transfers owned buffer; literal storage becomes owned JSON text |

The fromX spelling complements checked asX extraction and distinguishes dynamic
JSON storage from ordinary primitive conversions. Errors remain Result values for
the fallible number constructor. String ownership is explicit in signatures and
parameter help; clone(text) preserves a caller's source, while String(view) is the
explicit conversion from borrowed text. Construction does not repair precision
already lost in a floating-point argument.

Catalog examples execute through the CLI. Native checks cover exact integer
boundaries, negative zero, nonfinite errors, Unicode and use after transfer.
Matched workload evidence includes all five constructors and encoding, with an
independently counted checksum. It includes native owned-input copying and Node
immutable references; no isolated constructor-cost or timing claim is made.
Container construction and typed conversion remain pending S10 work.
