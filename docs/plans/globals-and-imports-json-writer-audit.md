# JSON Lines writer public API audit

Five explicit std:json identities extend the existing bounded streaming family:
JsonlWriter, createJsonl, writeJsonl, flushJsonl and closeJsonlWriter. None becomes
a global. JsonlWriter is uniquely owned, opaque and nonclonable. Creation's name
and hover explicitly disclose creation/truncation after option validation.

writeJsonl infers its typed value and borrows it without implicit cloning.
Writer access is mutable for write/flush and consuming for close. All operations
return Result with JsonlError. Output errors use offset zero (not a byte-location
claim), a current/next record counter, operation and optional OS code. Encoding
errors are terminal but write none of the rejected record; I/O may leave a prefix.
Close always attempts cleanup and preserves the documented error precedence.
Drop cannot report failure. No implicit durability sync is promised.

Native dependencies follow usage: writer runtime and file support are selected
only when the writer resource is present. Catalog entries supply parameter help,
ownership notes and a shared executable example checked against actual file bytes.
The inventory grows from 422 to 427 identities. Typed and dynamic writer inputs
share the direct serializer's exact integers, finite numbers, undefined policy,
key ordering and container-depth budget.


## Borrowed buffered writers

JsonlWriterView and borrowJsonlWriter add two explicit std:json identities, bringing
the inventory to 431. Construction borrows an existing BufferedWriter without
opening, truncating, seeking, writing or flushing. maxBytes has the same per-record
contract as createJsonl; capacity stays with the existing owner. The same write,
flush and close operation names accept both representations, including inferred
and explicit typed values and import aliases.

The view retains a real exclusive borrow. Adapter close/drop flushes the owner
without closing it. Explicit close reports a new flush failure in preference to
a previously reported terminal error; drop cannot report errors. The caller must
close the file owner afterward. Invalid limits leave the owner untouched and an
encoding failure emits no prefix of the rejected record. Offsets remain zero;
record numbering starts at adapter construction. Documentation and native examples
verify the distinction between adapter completion and owner closure.
