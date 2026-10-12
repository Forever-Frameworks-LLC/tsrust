# JSON container construction API review

Four imported identities extend the reviewed inventory from 418 to 422.

| Identity | Contract | Ownership |
| --- | --- | --- |
| std:json::JsonBuildLimits | Required int fields maxDepth, maxEntries and maxNodes | Ordinary record, borrowed by constructors |
| std:json::JsonEntry | Required key:string and value:JsonValue fields | Owned fields; explicit deep clone supported |
| std:json::fromArray | JsonValue array and optional limits to Result<JsonValue, JsonError> | Consumes array/children on either outcome |
| std:json::fromEntries | JsonEntry array and optional limits to Result<JsonValue, JsonError> | Consumes entries/keys/children on either outcome |

The constructors complement the imported scalar fromX family and introduce no new
global convention. Explicit entry records avoid implicit coercion of object keys.
The last duplicate key wins; output retains the existing sorted-key representation.
All supplied entries and descendant values count before duplicate replacement.

Omitted limits are depth 64, one million entries per collection, and one million
values. The new root counts as one value and one container level. All limits are
nonnegative; maximum configurable depth is 64. Root limits are checked before
allocating converted container storage, then all descendants are validated. Errors
have offset zero. Limits do not count pre-existing input bytes or encoding output.

Children move without deep copies. Container storage and literal-backed keys may
allocate. A consumed source cannot be reused even when validation returns Err;
clone explicitly when preserving it is required. Views cannot be substituted for
owned JsonValue children. Dependencies continue to follow actual API use.

Focused acceptance covers compiler/native behavior, limits, ownership, executable
catalog examples, editor hover/signature help and a matched workload with independent
checksum and source guards. No timing or installed-consumer acceptance is claimed.
