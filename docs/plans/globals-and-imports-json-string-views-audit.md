# JSON borrowed string API review

Two identities extend the reviewed inventory from 411 to 413.

| Identity | Behavior | Placement and ownership |
| --- | --- | --- |
| std:json::StringView | Read-only UTF-8 text; length and five searches; typeof returns string; console displays text | Imported opaque view type; owner lifetime retained; no clone or implicit owned string conversion |
| std:json::asString | JsonValue or JsonView to Result<StringView, JsonError>; wrong kind returns expected-string with offset zero | Imported checked extraction; no payload copy; failed extraction retains no owner loan |

`String(view)` explicitly creates an independent owned string. Optional views keep
that behavior after typeof narrowing. The existing `kind` API remains compatible;
`typeof` remains the ordinary category observation. Neither addition creates a
new global or automatically links unused runtime dependencies.

Borrowed strings expose only length, startsWith, endsWith, includes, indexOf and
lastIndexOf. Coordinates are UTF-8 bytes. Search arguments may be ordinary strings,
StringView values or JSON strings with a stable typeof proof. These argument
alternatives apply to string searches; array searches retain their element types.
StringView is not implicitly interchangeable with every owned string operation.

Public signatures, editor discovery and executable catalog examples are covered
by the focused string-view gate. The matched workload parses once and compares
per-iteration checked extraction and reads with Node immutable string references.
No timing acceptance is claimed. Automatic string projection after a JSON typeof
guard and exact numeric refinement remain separate pending work.
