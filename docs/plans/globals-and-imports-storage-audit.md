# Byte and file symbol placement review

Status: reviewed the sixteen `std:bytes` and fifteen `std:fs` exports against
binding metadata, argument alternatives, native types, runtime bridges and catalog.
Retain all as explicit imports. Ownership-bearing storage and host resources are
not candidates for ambient globals; existing local aliases remain supported.

## Bytes

In this table, `Readable` abbreviates `Bytes | ByteView | ByteViewMut`, and
`Writable` abbreviates `Bytes | ByteViewMut`. These are table shorthand, not new
public types. Observers borrow their storage; mutators borrow it exclusively.
Every listed Result uses `ByteError`. Coordinates count bytes, never characters.
Wrong types/arity are compile-time errors. Backend dependency is bundled byte
support; no external resource or user linker configuration is required.

| Export | Inputs and result | Behavior, errors and ownership | Disposition |
|---|---|---|---|
| `Bytes` | Opaque owned buffer | Explicit clone independently copies; no forged record or public representation fields | Retain imported owner type |
| `ByteView` | Opaque read-only borrowed range | Real source lifetime; cannot clone; owner cannot mutate/move while borrowed | Retain imported view type |
| `ByteViewMut` | Opaque exclusive borrowed range | Real source lifetime; writes affect source; cannot clone or introduce a second owner | Retain imported exclusive view type |
| `ByteError` | `{code:string,message:string,offset?:int}` | Ordinary owned record; optional coordinate identifies failure; explicit clone supported | Retain imported error type |
| `fromArray` | `(values: ReadOnly<int[]>): Result<Bytes,ByteError>` | Validate 0–255 then copy; byte-range, length-range or allocation failure; source unchanged | Retain explicit conversion import |
| `length` | `(bytes: ReadOnly<Readable>): int` | Byte count, no mutation or operation error | Retain shared observer name |
| `at` | `(bytes: ReadOnly<Readable>, index:int): Result<int,ByteError>` | Index must be in range; negative does not wrap; result 0–255 | Retain checked observer |
| `set` | `(bytes: Mutable<Writable>, index:int, value:int): Result<void,ByteError>` | Bounds checked before value range; either error leaves bytes unchanged | Retain explicit mutator |
| `alloc` | `(length:int, fill?:int): Result<Bytes,ByteError>` | Nonnegative length; fill defaults zero and must be 0–255 even for empty result; reservation errors recoverable | Retain explicit allocation |
| `resize` | `(bytes:Mutable<Bytes>, length:int, fill?:int): Result<void,ByteError>` | Preserve prefix; growth defaults zero; validation/reservation failure preserves data; invalidates incompatible outstanding loans | Retain owner-only mutator |
| `append` | `(bytes:Mutable<Bytes>, values:ReadOnly<int[]>): Result<void,ByteError>` | Validate length and all values before reserve/mutation; error preserves buffer | Retain owner-only mutator |
| `copy` | `(bytes:ReadOnly<Readable>, start:int, end:int): Result<Bytes,ByteError>` | Independent half-open range; require 0 ≤ start ≤ end ≤ length; no clamping; range/allocation errors | Retain explicit copying name |
| `fromUtf8` | `(text:ReadOnly<string>): Result<Bytes,ByteError>` | Copy UTF-8 bytes into independent storage; reservation/length errors | Retain explicit encoding import |
| `toUtf8` | `(bytes:ReadOnly<Readable>): Result<string,ByteError>` | Strict UTF-8 validation, independent text; malformed input reports offset without replacement | Retain explicit decoding import |
| `view` | `(bytes:ReadOnly<Readable>, start:int, end:int): Result<ByteView,ByteError>` | Checked half-open range, no copying; result carries source loan through Result payload | Retain common range API; no view-only observer names |
| `viewMut` | `(bytes:Mutable<Writable>, start:int, end:int): Result<ByteViewMut,ByteError>` | Same bounds; exclusive source loan, mutation through returned view | Retain explicit exclusive range API |

Reservation errors do not promise recovery from all process-wide memory exhaustion.
Returning a view does not extend owner lifetime. `copy(view,...)` deliberately
creates ownership; `clone(view)` remains rejected. Stored native function values
retain declared signatures; alternative byte argument forms apply to checked
native calls rather than silently introducing polymorphic function values.

## Files

Every function returns `Result<..., FsError>`. Paths and input byte storage are
borrowed, and returned Bytes own their data. File operations currently support
macOS/Linux; unsupported targets produce a native compilation diagnostic. Backend
support is bundled filesystem/resource code, plus byte support for data I/O.

| Export | Inputs and result payload | Behavior, errors and ownership | Disposition |
|---|---|---|---|
| `File` | Opaque resource owner | Unique handle/cursor; no clone or source construction; transfer by value, cleanup on drop | Retain imported resource |
| `BufferedReader` | Opaque resource owner | Owns File and buffer; unique cursor; no clone | Retain imported resource |
| `BufferedWriter` | Opaque resource owner | Owns File and pending bytes; no clone; drop attempts flush/close and discards errors | Retain imported resource; explicit close remains necessary to observe failure |
| `FsError` | `{code:string,operation:string,message:string,osCode?:int}` | Owned structured failure; OS code optional; ordinary clone | Retain imported error |
| `open` | `(path:string)` → File | Open existing file for reading; I/O errors; borrows path | Retain explicit host action |
| `create` | `(path:string)` → File | Create/truncate writable file; I/O errors; borrows path | Retain separate destructive creation name |
| `read` | `(file:Mutable<File \| BufferedReader>, length:int)` → Bytes | Cursor advances; short read/EOF allowed; length 0–67108864; zero performs no I/O; mutably borrowed resource, independent output | Retain shared resource operation |
| `close` | `(file:Owned<File \| BufferedReader \| BufferedWriter>)` → void | Consumes on success/failure; releases once, never retries a released descriptor; writer flushes before close | Retain visible consuming operation |
| `write` | `(file:Mutable<File \| BufferedWriter>, bytes:ReadOnly<Readable>)` → int | May write/accept fewer bytes; count records progress; failure may follow earlier external progress | Retain explicit partial-progress API |
| `writeAll` | Same inputs as write → void | Retry short/interrupted writes; zero progress is failure; external partial writes cannot be rolled back | Retain explicit complete-write API |
| `readFile` | `(path:string, maxBytes?:int)` → Bytes | Bounded whole-file read; maxBytes defaults 67108864, range 0–67108864; closes internal file on success/failure | Retain convenience import with visible byte limit |
| `writeFile` | `(path:string, bytes:ReadOnly<Readable>)` → void | Create/truncate, write all, close; reports errors; not an atomic replacement transaction | Retain explicit convenience import |
| `bufferReader` | `(file:Owned<File>, capacity?:int)` → BufferedReader | Capacity 1–1048576, default 8192; consumes file even on invalid capacity/allocation failure; failure releases it | Retain explicit ownership transfer |
| `bufferWriter` | `(file:Owned<File>, capacity?:int)` → BufferedWriter | Same capacity and transfer contract; writes may remain buffered | Retain explicit ownership transfer |
| `flush` | `(writer:Mutable<BufferedWriter>)` → void | Writes pending bytes; unwritten suffix retained after failure; retries possible; does not promise durable storage | Retain explicit borrowed operation |

## Source reconciliation and evidence

Compiler anchors under `crates/tsrust-compiler/src`:
`stdlib/bytes/bindings.rs` and `stdlib/fs_bindings.rs` define signatures and modes;
`native_bindings.rs::argument_alternatives` supplies accepted view/buffer variants;
`native_return_borrows.rs` validates view provenance; `native_types.rs` supplies
opaque representations, error fields and cloning restrictions.
`stdlib/bytes/runtime.rs` and byte emission preserve checked ranges, atomic byte
mutations and slice lifetimes. `stdlib/fs.rs` transfers descriptors exactly once;
`stdlib/buffered_fs.rs` retains unwritten suffixes and consumes constructors;
`stdlib/resource.rs` owns fallback cleanup. `stdlib/fs_emit.rs` bridges the same
Result/error fields and resource modes. The catalog's `std:bytes` and `std:fs`
groups describe these same contracts, including consuming close and byte-view
alternatives. A plain `File` spelling in catalog parameter help means ownership
transfer; it does not imply a borrow.

Focused audit checks reuse the buffered-file ownership/flush witnesses, byte-view
ownership rejections and editor view/observer documentation witness. Logs remain
in SQLite as `tsrust-api-storage-*`. Prior S07 acceptance supplies broader native
cleanup and performance evidence; this audit does not rerun or replace it.
No storage API relocation or silent error-contract change is proposed.
