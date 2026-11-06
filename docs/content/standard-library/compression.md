<!-- tsrust-docs
{"id":"reference-standard-library-compression","group":"standard-library","order":24,"status":"partial","targets":["native"],"api":[{"receiver":"std:compression","name":"CompressionOptions"},{"receiver":"std:compression","name":"DecompressionOptions"},{"receiver":"std:compression","name":"CompressionError"},{"receiver":"std:compression","name":"gzip"},{"receiver":"std:compression","name":"gunzip"},{"receiver":"std:compression","name":"deflate"},{"receiver":"std:compression","name":"inflate"},{"receiver":"std:compression","name":"deflateRaw"},{"receiver":"std:compression","name":"inflateRaw"},{"receiver":"std:compression","name":"CompressionFormat"},{"receiver":"std:compression","name":"CompressionReader"},{"receiver":"std:compression","name":"CompressionWriter"},{"receiver":"std:compression","name":"compressReader"},{"receiver":"std:compression","name":"decompressReader"},{"receiver":"std:compression","name":"compressWriter"},{"receiver":"std:compression","name":"decompressWriter"},{"receiver":"CompressionReader","name":"read"},{"receiver":"CompressionReader","name":"readInto"},{"receiver":"CompressionReader","name":"readExact"},{"receiver":"CompressionReader","name":"readAll"},{"receiver":"CompressionReader","name":"finish"},{"receiver":"CompressionReader","name":"close"},{"receiver":"CompressionWriter","name":"write"},{"receiver":"CompressionWriter","name":"writeAll"},{"receiver":"CompressionWriter","name":"flush"},{"receiver":"CompressionWriter","name":"finish"},{"receiver":"CompressionWriter","name":"close"}]}
-->
# Compression

`std:compression` encodes and decodes owned byte buffers using an explicit bounded `ThreadPool`. Every operation returns a lazy `Task<Result<Bytes, CompressionError | SpawnError | ThreadError<undefined>>>`. The Task owns its input immediately and submits CPU work only when first polled. Keep the pool owner alive through completion; the Task retains weak admission identity.

| Encode | Decode | Format |
| --- | --- | --- |
| `gzip` | `gunzip` | Gzip framing and CRC |
| `deflate` | `inflate` | Zlib framing and Adler checksum |
| `deflateRaw` | `inflateRaw` | Raw DEFLATE, without a framing checksum |

Compression options have one optional `level`: integer0 through9, default6. Decompression options have one optional `maxOutputBytes`: nonnegative and target-size representable. No ordinary expansion ceiling is imposed. An explicit limit is checked incrementally; zero permits a valid stream whose decoded output is empty. Failure discards the result rather than returning a partial buffer.

```tsrust
import {gzip,gunzip} from 'std:compression';
import {ThreadPool} from 'std:threads';
import {fromUtf8,toUtf8} from 'std:bytes';
async function main():Task<void>{
  const pool=new ThreadPool({workers:1,capacity:1});
  const encoded=unwrap(await gzip(pool,unwrap(fromUtf8('hello'))));
  const decoded=unwrap(await gunzip(pool,encoded,{maxOutputBytes:5}));
  console.log(unwrap(toUtf8(decoded))); // hello
  unwrap(pool.shutdown());
}
```

Decoders require complete streams, including trailers and checksums where provided. `gunzip` accepts concatenated gzip members. Raw and zlib decoders reject data after the first completed stream; malformed data after a gzip member is also rejected. Gzip output has zero modification time and no optional metadata; its native OS header byte is platform-specific. Identical operations are deterministic on a target. No configurable headers, window size, dictionaries or backend selector are exposed.

`CompressionError` contains a finite `code`, failing `operation` and explanatory `message`. `range` rejects invalid options; `codec` includes invalid framing/checksums; `truncated` means an incomplete stream; `trailing-data` rejects a suffix; `output-limit` exceeds the caller's policy. `allocation` covers fallible output reservations, without promising catchable backend OOM. `format` and `closed` describe format/transform-state failures. Full/closed pool admission stays `SpawnError`; worker panic/cancellation stays `ThreadError<undefined>`. Cancellation cleanup waits for actual worker completion.

Ownership: pass owned `Bytes`; borrowed byte views are not silently cloned. Reuse requires an explicit `clone`. No other owner is mutated. An unpolled Task drops its input without submitting work.


## Reader and writer transforms

`compressReader`, `decompressReader`, `compressWriter` and `decompressWriter` take an explicit pool, consume a base `std:io.Reader` or `Writer`, and require one format literal: `gzip`, `zlib` or `raw`. Their options are the same level or decoded-byte policy described above. They return `Result<CompressionReader | CompressionWriter, CompressionError>` with the concrete owner type selected by the constructor. Construction validates options and allocates working storage; it initializes the codec only on a later nonempty operation. Rejected construction still consumes/drops the base endpoint.

A reader supports `read`, `readInto`, `readExact` and `readAll`; a writer supports `write`, `writeAll` and `flush`. The corresponding `std:io` functions accept these concrete endpoints too. Successful payloads match B05. Compression overloads return `CompressionError | IoError | SpawnError | ThreadError<undefined>`; ordinary endpoints retain `IoError`. Mutable receiver/destination loans remain held until their operation settles. Caller slices never enter a CPU job.

`copy` supports all four pairs of base/compression readers and writers. It borrows both, copies with bounded working storage and returns the count. It does not finish or close either owner. CPU steps settle and release pool capacity before asynchronous source/sink I/O, so a decoder→encoder pipeline can use one worker and one slot without channel deadlock.

```tsrust
import {gzip,decompressReader,compressWriter} from 'std:compression';
import {ThreadPool} from 'std:threads';
import {reader,writer,copy,close} from 'std:io';
import {create} from 'std:fs';
import {fromUtf8} from 'std:bytes';
async function main():Task<void>{
  const pool=new ThreadPool({workers:1,capacity:1});
  const bytes=unwrap(await gzip(pool,unwrap(fromUtf8('pipeline'))));
  const input=unwrap(decompressReader(pool,reader(bytes),'gzip'));
  const output=unwrap(compressWriter(pool,writer(unwrap(create('pipeline.z'))),'zlib'));
  console.log(unwrap(await copy(input,output,8192))); // 8
  unwrap(await close(input));
  unwrap(await close(output));
  unwrap(pool.shutdown());
}
```

`flush` uses native Sync, drains output and flushes the base writer. Its prefix can be decoded immediately, and later writes continue the same stream. It does not signal EOF or produce a completed trailer. `finish` consumes the transform and returns its original base endpoint: readers drain and validate through EOF while discarding unread output; writers emit/validate stream completion and flush. `close` finishes and then closes the base endpoint. A failure consumes/drops the owner without returning a base endpoint. Streamed bytes may be observed before a later checksum/trailer failure; no rollback is promised.

Dropping an unfinished owner discards codec state. It does not finalize, create hidden cleanup jobs or retain a public pool owner. Cancelling started CPU work uses the existing task cleanup to await actual settlement; queued cancellation skips the step and retains owned staged input for a retry. Full/closed admission also retains the staged owner state. A cancelled sink write can have unreported transport effects, so the writer is poisoned rather than replaying possibly written bytes; later nonempty operations return `closed`.

Reader working storage initially contains three8192-byte buffers; a writer contains two, plus state/backend allocation. Input is staged once in reusable owned storage without per-chunk payload clones. These sizes are working defaults rather than expansion caps. Returned `read`/`readAll` buffers and the backend allocate additional storage. Fallible reservations report errors where supported; general/backend OOM is not guaranteed catchable. `readAll(maxBytes)` and `copy(...,maxBytes)` retain their separate B05 materialization/copy bounds and `IoError` category; `DecompressionOptions.maxOutputBytes` is the cumulative codec policy.

### Copying between TCP and transforms

`std:io.copy` also accepts a full `TcpStream` at either endpoint. For example,
copy incoming TCP bytes into a `CompressionWriter`, finish that writer, then copy
a `CompressionReader` into the same connection's writable side. Copy retains
exclusive loans and never finishes or closes either endpoint. Call
`stream.shutdownWrite()` when the peer needs EOF; `stream.close()` consumes the
full stream synchronously.

Any copy involving a transform preserves `CompressionError | IoError |
SpawnError | ThreadError<undefined>`. Plain Reader/Writer/TCP combinations retain
`IoError` only. A one-worker, one-slot pool suffices for the transform pipeline.
See `examples/compression/tcp-pipeline.tsr` for an executable request/reply example.
