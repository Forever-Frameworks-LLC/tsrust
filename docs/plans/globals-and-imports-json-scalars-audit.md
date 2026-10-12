# JSON scalar placement audit

Four identities extend the reviewed inventory from 404 to 408. They remain imports
from std:json because they inspect the native JsonValue representation and have
explicit conversion and ownership contracts. The global JSON parsing/encoding
namespace remains unchanged. Actual calls select runtime dependencies.

| Identity | Contract |
| --- | --- |
| std:json::asBoolean | Borrows JsonValue; Result<boolean, JsonError>, no coercion. |
| std:json::asInt | Borrows JsonValue; Result<int, JsonError>, exact signed 64-bit extraction; rejects fractions and range overflow. Negative zero becomes zero. |
| std:json::asNumber | Borrows JsonValue; Result<number, JsonError>, rejects integer precision loss and preserves stored floating bits. Cannot recover precision already lost during parsing. |
| std:json::takeString | Consumes JsonValue on success or failure; Result<string, JsonError>. Moves string storage without cloning. |

All errors are structured, contain fixed messages without input contents, and use
zero offsets because parsed values retain no source positions. There are no throws
from these bindings. The editor signatures explain consumption versus borrowing.
A borrowed string public API remains deferred until it carries real lifetimes.
