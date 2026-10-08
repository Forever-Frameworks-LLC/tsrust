# Standard library

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Core proof; later expansion.

**Scope:** Define library boundaries and compatibility policy; implement only the subset needed by each prototype.

**Planning deliverable:** A staged module/API inventory with experimental and stable boundaries.

**Dependencies to coordinate:** [primitives-and-collections](./primitives-and-collections.md), [error-handling](./error-handling.md)

## Implemented experimental primitive subset

Primitive arrays provide `includes(value, fromIndex?)` and `indexOf(value, fromIndex?)` alongside length, indexing, push, removal, and explicit cloning. Search supports dense integer, boolean, and valid-Unicode-string values; it reads borrowed storage without transferring string needles. The [search gate](./milestones/32-array-search.md) records the exact contract and compatibility restrictions. Shared generated helpers provide methods without allocating method implementations per value. This bounded experiment does not settle broader standard-library stability, generics, owned object identity, sparse storage, or source view semantics.

Array `reverse()` mutates existing storage through standard Rust slice reversal and returns void. It preserves fixed storage where the compiler proves length remains unchanged; other array representations reverse their live elements. [Milestone 36](./milestones/36-array-reversal.md) records acceptance and matched runtime comparisons.

String `startsWith`, `endsWith`, `includes`, and `indexOf` use borrowed UTF-8 input and checked byte positions. [Milestone 37](./milestones/37-string-search.md) records complete compiler and runtime acceptance. See the [string contract](./string-semantics.md) for defaults, clamping, and boundary failures.

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-14"></a>

## 14. Standard Library

Initial standard library should provide familiar JavaScript-style APIs where sensible.
Major categories:
Core
Array
Map
Set
String
Math
Date / DateTime
Regex
JSON
URL
UUID
Result
Option
Iterator
Range
Buffer / Bytes
Filesystem
Node-like ergonomics:
await fs.readFile(path);
await fs.writeFile(path, data);
Networking
fetch()
Request
Response
Headers
WebSocket
TCP
UDP
TLS
Processes
process.env
process.args
process.exit()
spawn()
Consider preserving familiar naming where it is useful without pretending a Node runtime exists.
Crypto
Native safe cryptographic APIs.
Testing
Testing should be built into the toolchain.
Serialization
First-class JSON support.
Potential later formats:
MessagePack
CBOR
Protobuf
YAML
TOML

<a id="core-48"></a>

## 48. Standard Library Stability

Establish a compatibility policy early.
Research:
* stable API guarantees
* experimental modules
* deprecation periods
* language editions
* standard-library versioning

## Lexical path parent and extension

`std:path` exports `dirname(path: string): string` and `extname(path: string): string`. Both borrow the input and return owned text without filesystem access. The target chooses POSIX or Windows separators. `dirname` preserves parent spelling and does not resolve dot components; empty input returns `.`. `extname` returns the last dot suffix, including its dot; leading-dot names and `..` have no extension. Trailing separators are ignored.

```tsrust
import { dirname, extname } from "std:path";
function main() {
    console.log(dirname("/tmp/report.tar.gz")); // /tmp on POSIX
    console.log(extname("report.tar.gz")); // .gz
}
```

Executable assertions: `crates/tsrust-compiler/tests/fixtures/lexical-path/parent-extension.tsr`. Missing, extra or non-string arguments are compile-time errors.

`isAbsolute(path: string): boolean` tests lexical root syntax. Empty input is false; POSIX requires a leading slash. Windows also accepts backslash and drive-letter-colon-separator syntax; `C:report.txt` is relative.

`normalize(path: string): string` collapses separators and resolves dot components without filesystem access. Empty input returns `.`. Absolute parents stop at the root, relative leading `..` remains, and trailing separators remain. Windows retains drive/UNC roots and emits backslash. Ordinary paths are idempotent; special device/colon forms can change on repeated calls. These operations do not establish containment or resolve symlinks.

```tsrust
import { isAbsolute, normalize } from "std:path";
function main() {
    console.log(isAbsolute("/tmp/report.txt")); // true
    console.log(normalize("reports/../archive/")); // archive/ on POSIX
}
```

Both inputs are borrowed and unchanged. The boolean is a value; normalized text owns its contents. Executable assertions: `crates/tsrust-compiler/tests/fixtures/lexical-path/absolute-normalize.tsr`.

## Executable identity: `std:os`

`platform(): string` returns the operating-system name for which the executable was compiled: `aix`, `android`, `darwin`, `freebsd`, `linux`, `openbsd`, `sunos`, or `win32`. `arch(): string` returns its compiled CPU architecture: `arm`, `arm64`, `ia32`, `loong64`, `mips`, `mipsel`, `ppc64`, `riscv64`, `s390x`, or `x64`. Both take no arguments and remain constant during execution. Neither queries environment variables, physical hardware, or the current kernel. A translated executable reports its compiled architecture. An unsupported target component produces an explicit native compilation error for that export.

```tsrust
import { platform, arch } from "std:os";
function main() {
    console.log(platform(), arch()); // darwin arm64 for a macOS ARM64 executable
}
```

There are no defaults, receiver mutation, or recoverable runtime errors. Each return is an independent string value. [Exact target mappings and acceptance limits](os-identity-contract.md) define supported Rust components; [executable assertions](../crates/tsrust-compiler/tests/fixtures/os-identity/identity.tsr) also exercise aliases and stored calls. This slice accepts only these two exports, not the whole OS module.
