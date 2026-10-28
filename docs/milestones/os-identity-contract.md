# Executable target identity contract

This bounded slice adds `std:os.platform(): string` and `std:os.arch(): string`. Both take exactly zero arguments, return stable independent string values, mutate no receiver, and perform no environment or filesystem queries. A wrong argument count or assignment to a non-string result is a compiler diagnostic. An unsupported component is an explicit native compilation error for the referenced export; importing only platform does not require a supported architecture, and importing only arch does not require a supported OS.

The contract follows the executable compilation target. A compiler running on macOS can emit source that identifies a Windows executable when Rust compiles that source for Windows. An executable running under translation reports its compiled architecture, not the physical CPU. Emission retains Rust target predicates; it does not substitute the compiler process's host identity.

## Frozen mappings

| Rust target_os | platform result |
| --- | --- |
| macos | darwin |
| windows | win32 |
| linux | linux |
| android | android |
| aix | aix |
| freebsd | freebsd |
| openbsd | openbsd |
| solaris, illumos | sunos |

| Rust target_arch | Additional selector | arch result |
| --- | --- | --- |
| x86 | none | ia32 |
| x86_64 | none | x64 |
| arm | none | arm |
| aarch64 | none | arm64 |
| loongarch64 | none | loong64 |
| powerpc64 | either byte order | ppc64 |
| riscv64 | none | riscv64 |
| s390x | none | s390x |
| mips | big endian | mips |
| mips | little endian | mipsel |

Every other component is unsupported. In particular, this slice does not infer names for Apple mobile targets, BSD variants other than FreeBSD/OpenBSD, WASI, wasm, 32-bit PowerPC, MIPS64 or MIPS revision-six targets. The support tables define independent components, not a claim that every Cartesian product is a viable Rust or Node target. Android naming matches the documented experimental Node platform spelling; no Android native acceptance is claimed.

## Sources and reference environment

Official [Node OS documentation](https://nodejs.org/api/os.html#osarch) defines identity as the architecture/OS for which the binary was compiled, equivalent to process metadata. The pinned [Node 24.11.1 process documentation](https://github.com/nodejs/node/blob/v24.11.1/doc/api/process.md#processarch) supplies the reference names. Rust's [conditional compilation reference](https://doc.rust-lang.org/reference/conditional-compilation.html#target_arch), [OS constants](https://doc.rust-lang.org/std/env/consts/constant.OS.html), [architecture constants](https://doc.rust-lang.org/std/env/consts/constant.ARCH.html), and [platform support](https://doc.rust-lang.org/rustc/platform-support.html) establish the metadata selectors and target triples. These sources were researched before the bridge implementation; Node 24.11.1 and Rust 1.96.1 are developer evidence, not IntelliSense content.

Native reference on this host: Node reports `darwin arm64`; Rust metadata is `target_os="macos"`, `target_arch="aarch64"`, little endian. The independent test freezes 16 target triples and expected Node names, reads their actual Rust target metadata, and compiles all selected branches into one host witness using renamed test predicates. Unsupported selectors must fail compilation. This proves branch selection and metadata mapping, not cross-platform executable acceptance. Only macOS ARM64 native execution is accepted.

## Integration boundary

The existing primitive native bridge already supports empty parameter lists and string results. The red project witness failed with `module-native: Unknown standard-library module "std:os".` before registration. No parser, type representation, callable carrier, ownership, loan, or flow changes are required. Shared edits are two appended registry entries, referenced-export emission, a pure native error-effect classification, catalog/editor tests, and benchmark registration hooks. Library-owned files hold all target mapping and bridge logic.

The two wrappers return static target text through the existing string representation. No runtime host query is present. Matched workloads retain repeated outputs and can measure collection work, but identity calls may constant-fold; timing is deferred until a meaningful OS family milestone. Untimed verification is acceptance evidence, not measurement.
