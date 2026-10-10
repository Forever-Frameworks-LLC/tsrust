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


## Checked byte access and mutation

`at(bytes, index)` borrows the buffer and returns `Result<int, ByteError>`.
Indices are zero-based bytes, from zero through `length(bytes) - 1`; negative indices
never wrap. Out-of-range indices, including every index on an empty buffer, return
`index-range` with the requested index in `offset`. The buffer is unchanged.

`set(bytes, index, value)` borrows the buffer exclusively and returns
`Result<void, ByteError>`. A successful write replaces one byte without changing
length. Values must be integers in 0..255. Bounds are checked first; a bad index
returns `index-range`. With a valid index, a bad byte returns `byte-range` and that
index as offset. Either failure leaves every byte unchanged. A readonly parameter
cannot be passed for writing; passing the owner does not transfer or clone it.

Match a successful void result with `Ok(_)`. For example, after constructing a
buffer with `[65]`, `set(bytes, 0, 90)` succeeds and `at(bytes, 0)` returns `Ok(90)`.
The complete catalog examples print `255` for reading `[65, 255]` at index one and
`90` after writing the single-byte buffer. Both execute with the current compiler.

These two operations are the first gate 188 increment. Allocation, resizing,
append, independent range copies and the exact cleanup matrix remain in progress.
