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

These operations belong to gate 188. Append, independent range copies and the
exact cleanup matrix remain in progress.


## Allocation and resizing

`alloc(length, fill?)` returns `Result<Bytes, ByteError>` with an independent buffer.
`resize(bytes, length, fill?)` exclusively borrows a buffer and returns
`Result<void, ByteError>`. Both default fill to zero. Length must be nonnegative and
fit native storage; fill must be an integer in 0..255. Length is validated first,
then fill, including empty, unchanged and shrinking requests. Invalid arguments
return `length-range` or `byte-range`; their offset is undefined.

Resizing preserves the existing prefix. Growth fills only new bytes; shrinking
removes the trailing bytes and may retain capacity. Growing again initializes new
bytes rather than restoring discarded contents. Failed reservation returns
`allocation` before changing the buffer's length or contents. This does not promise
recovery from every process-wide memory exhaustion. No arbitrary byte limit is imposed.

```tsrust
import { alloc, resize, length } from "std:bytes";

function main() {
    match (alloc(1)) {
        Ok(bytes) => {
            match (resize(bytes, 3, 65)) {
                Ok(_) => { console.log(length(bytes)); },
                Err(error) => { console.log(error.message); },
            }
        },
        Err(error) => { console.log(error.message); },
    }
}
```

This prints `3`. The original byte stays zero; the two new bytes are 65.
The owner remains usable after resize; readonly byte parameters cannot be resized.


## Appending and independent range copies

`append(bytes, values)` exclusively borrows Bytes and readonly borrows an int array.
It appends the values in order and returns `Result<void, ByteError>`. An empty array
succeeds without reserving storage. The combined length is checked first; then every
value must be in 0..255. Invalid length returns `length-range` without offset; invalid
value returns `byte-range` with its index in the input array. Reservation failure
returns `allocation` without offset. Every failure leaves length and contents intact.
The input array stays unchanged. This is an explicit integer-to-byte copy, not wrapping.

`copy(bytes, start, end)` readonly borrows Bytes and returns `Result<Bytes, ByteError>`
with an independent owner. Bounds must satisfy `0 <= start <= end <= length(bytes)`.
The half-open range excludes end. Equal bounds return an empty buffer, including at
the source end. Negative indices never wrap; bounds are never clamped. Start is checked
before end. Invalid start returns `range` with start as offset; invalid or reversed
end returns `range` with end as offset. Reservation failure returns `allocation`
without offset. Success and failure leave the source unchanged. Mutating either
owner later cannot affect the other. Coordinates count bytes rather than characters.

```tsrust
import { fromArray, copy, length } from "std:bytes";

function main() {
    match (fromArray([0, 65, 255])) {
        Ok(bytes) => {
            match (copy(bytes, 1, 3)) {
                Ok(part) => { console.log(length(part)); },
                Err(error) => { console.log(error.message); },
            }
        },
        Err(error) => { console.log(error.message); },
    }
}
```

This prints `2`. The selected buffer contains 65 and 255. No source owner transfers
or implicit clones occur. As with other byte operations, fallible reservation does
not guarantee recovery from every process-wide memory exhaustion.


## Explicit cloning, storage and cleanup

`clone(bytes)` readonly borrows a buffer and returns an independent Bytes owner.
It uses the ordinary clone operation rather than a separate byte-only spelling.
Changing or resizing either buffer does not change the other. Explicit cloning also
copies owned Bytes descendants in arrays, records, Maps and nullable/Result payloads.
Null and undefined alternatives remain absent. Clone does not return Result and does
not promise recoverable allocation failure. Assignment, parameter passing and storage
transfer owners without implicit cloning.

Bytes can live in generic parameters, records, arrays, Maps with supported scalar
keys, nullable unions and Result payloads. Borrowing parameters use ReadOnly or Mutable.
Escaped callable captures transfer owners; reusable callbacks can readonly inspect
their captured buffers. A Once callback can transfer its captured owner out. Normal
scope exit, early return, replacement, Result failure, throw, finally and discarded
closures release byte owners deterministically. Bytes needs no explicit close.

Bytes and records containing Bytes cannot be collection keys, inspected as console
objects, compared as values, reflected into properties or spread as records. These
forms produce source diagnostics. Copy/clone and explicit byte operations are the
supported ways to observe and transform buffers. Native thread-transfer metadata
describes backend eligibility only; source thread APIs are not yet implemented.


## Explicit UTF-8 encoding and strict decoding

`fromUtf8(text)` borrows a valid UTF-8 string and returns `Result<Bytes, ByteError>`
with independent encoded bytes. No normalization, BOM stripping or replacement occurs.
For example, encoding `é😀` produces six bytes, not two. Empty text produces an empty
buffer. Invalid native length returns `length-range`; failed reservation returns
`allocation`. Both omit offset, and no partial owner escapes.

`toUtf8(bytes)` readonly borrows Bytes and returns `Result<string, ByteError>` with
independent owned text. It rejects malformed, overlong, surrogate, out-of-range and
truncated sequences. The `utf8` error has message `Byte buffer contains invalid UTF-8.`
and the first invalid sequence's starting byte offset. For `[65, 195, 40]`, offset
is 1, the start of the malformed sequence. Valid bytes are checked once before
output reservation and copying. NUL and a leading U+FEFF are preserved. Empty input
produces an empty string; invalid bytes never turn into replacement characters.

Neither operation consumes or changes its input. Both require exactly one correctly
typed argument, with no coercion or defaults. Reservation failure returns `allocation`
without offset; this does not guarantee recovery from every process-wide memory
exhaustion. Encoding/decoding copies are explicit. Decoding also accepts a readonly
or mutable byte view and reads only that view's range.

## Borrowed byte views

`ByteView` is a readonly borrowed range. `ByteViewMut` is an exclusive mutable
borrowed range. Neither owns or copies the underlying bytes. `view(bytes, start,
end)` accepts `Bytes`, `ByteView` or `ByteViewMut` and returns
`Result<ByteView, ByteError>`. `viewMut(bytes, start, end)` accepts `Bytes` or
`ByteViewMut` and returns `Result<ByteViewMut, ByteError>`.

Both indices are required integer byte offsets. The start is inclusive and end
exclusive; require `0 <= start <= end <= length(bytes)`. Empty ranges, including
one at the end, are valid. Invalid bounds return a `range` error with the relevant
offset. Nested views use offsets relative to their immediate input range. For a
buffer encoding `hello`, `view(bytes, 1, 4)` borrows the bytes for `ell`.

The existing `length`, `at`, `copy`, `toUtf8` and `view` operations accept either
owner or view type. `set` and `viewMut` require a mutable owner/view. There are no
separate view-only read/write function names. Writing through a mutable view changes
its owner. `copy` produces an independent owned buffer; views cannot be cloned.
Byte offsets need not align to UTF-8 characters, so decoding a partial character
returns the ordinary strict UTF-8 error.

A view cannot outlive its owner. Live views restrict conflicting writes, resizing,
ownership transfer and mutable reborrowing. Views need no `close`. Callbacks may
capture views and return a view received as an argument; captured and returned
owners retain their own provenance. Stored callbacks and parameter replacement
remain subject to the same owner protections.

An explicit readonly `view` of a `ByteView` retains the original byte owner lifetime.
It can outlive the intermediate view or its Map while the byte owner remains live.
Reborrowing `ByteViewMut` retains a borrow of that exclusive view storage; a view
borrowed from a local Map of exclusive views cannot escape that Map.

Records can contain views and recurse through arrays, Map values or nullable record links.
Each present nullable recursive link uses a uniquely owned heap allocation; this
also applies to nullable links between mutually recursive record types. Layouts
without a terminating indirection are rejected. Nonrecursive layouts are unchanged.
A recursive record containing views uses one shared borrow lifetime, so its entire
value must fit within every referenced owner's lifetime. View-containing values
remain non-clonable. Deep clone is available only when all contained types support it.

Executable examples cover [readonly access](../crates/tsrust-compiler/tests/fixtures/native-byte-views/read.tsr),
[callbacks](../crates/tsrust-compiler/tests/fixtures/native-byte-views/callbacks.tsr)
and [recursive links](../crates/tsrust-compiler/tests/fixtures/native-byte-views/recursive-links.tsr).
The [S07 evidence](milestones/s07-resources-bytes.md) separates these guarantees from
matched workload measurements and the direct Rust comparison.
