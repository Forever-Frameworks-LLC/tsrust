# Owned native bytes

Import `Bytes`, `ByteError`, `fromArray` and `length` from `std:bytes`.
`fromArray(values)` borrows an integer array and copies valid elements into an
independent owned byte buffer. Each integer must be between 0 and 255 inclusive.
An empty array succeeds with an empty buffer. The input remains unchanged and reusable.

```tsrust
import { fromArray, length } from "std:bytes";

function main() {
    match (fromArray([0, 65, 255])) {
        Ok(bytes) => { console.log(length(bytes)); },
        Err(error) => { console.log(error.message); },
    }
}
```

This prints `3`. `length(bytes)` borrows the buffer and returns its byte count as
`int`; it neither copies nor mutates the buffer. Byte counts are not Unicode
character counts.

Construction returns `Result<Bytes, ByteError>`. Invalid elements produce code
`byte-range`, a readable message and the first invalid array offset. Lengths outside
`int` produce `length-range`; failed reservation produces `allocation`. These last
two errors omit offset, which reads as `undefined`. Reservation failure is recoverable,
but recovery from every process-wide memory exhaustion is not guaranteed.

Passing Bytes by value transfers ownership; use `ReadOnly<Bytes>` for borrowing.
Storage is private and record literals cannot construct a buffer. Console inspection,
string conversion, property reflection, object spread and value comparison are
currently rejected. Wider byte operations and resource handles remain in progress.
