# Portable source packs and Rust/editor agreement

Windows x86_64 section following checkpoint 2d3a6e09. Source implementation and
affected acceptance are complete. The local checkpoint/installed-tool refresh is
recorded separately in ../agent-progress.md; production installation is not complete.

## Source distribution

Removed compiler build.rs, its build dependencies, implicit Cargo vendor invocation
and include_bytes dependency payloads. Ordinary backend runtimes retain the accepted
small exact-lock workspace and shared Cargo sources. Explicit setup --portable uses
upstream Cargo vendor --locked --versioned-dirs, optionally --offline, into an
exclusively created staging directory. It verifies expected manifest, lock, stub,
offline configuration, package identities and Cargo vendor file checksums before
publishing by rename. Failed staging is removed; existing modified packs are rejected
and preserved. No custom resolver or package-manager implementation was introduced.

The optional pack retains the full legacy portable closure for existing UI/binding
exports. It lives under CARGO_HOME/tsrust/source-packs or TSRUST_SOURCE_PACK_DIR,
versioned by source/configuration identity. Export validates and copies prepared
sources/licenses without executing Rust/Cargo or accessing the network. Unprepared
exports give the exact setup command. Plain dependency-free exports need no pack.
Export includes tsrust-export.json with schema, pinned toolchain, source mode,
exporting host and platform build prerequisites. Its host does not certify cross-
compilation or execution on other platforms. Installation/update trust is separate.

TSRLIB3 artifacts retain Windows import libraries for source-free cache restores;
TSRLIB2 reading remains supported. Fresh imports are verified against pinned file
SHA256 values checked against the retained 0.52.6 Cargo vendor receipts, then copied
from shared Cargo sources or the optional pack. Existing differing bytes and links
are preserved/rejected. Backend dev reports offline runtime preparation. Packaging
instructions describe explicit source preparation and no longer claim embedded
sources or frontend tooling prerequisites.

Observed CLI size: previous payload build 671702016 bytes; first payload-free build
40804864 bytes (before subsequent metadata/helper edits). These are build observations,
not matched performance benchmarks. The one-time compiler library rebuild still costs
minutes; removing the payload is not a claim that all compiler compilation is fast.

## The reported 43 Rust diagnostics

The owner supplied the complete diagnostic JSON. Real missing-module errors came
from an integration test importing CLI runtime internals without their new root
context; private prepare re-export errors were also corrected. native_json_build
now uses the actual CLI and std:testing strictEqual instead of maintaining a duplicate
CLI-root fixture. Its invalid artifact checks use current TSRLIB3 framing rather
than rejecting all inputs through an obsolete header.

Renderer helpers now have their actual feature consumers. Shared authoring metadata
and headless ownership remain retained; narrow configuration-specific expected lint
annotations explain intentional retained fields. No global warning suppression or
compiler private-binding diagnostic filter was added. New tests exercise whitespace,
palette lookup, visibility enumeration, cache clearing, environment args, UTC floor,
HTTP context consumption/status/query readers and Router TLS/configuration. The
existing static-serving behavior test is shared between its two source harnesses.
Generated UI rewriting handles both spaced and compact runtime feature predicates.
The workflow now requires all compiler/LSP targets at helper milestones and a
separate backend-only check, since workspace UI feature unification can hide warnings.

Initial workspace check: zero errors; three uncovered HTTP warning groups were
repaired through real coverage. Subsequent backend/compiler-LSP all-target and full
workspace/all-target checks both completed with zero warnings/errors. Their JSON
streams were parsed and required successful build-finished records. Final backend-only and workspace/all-target refreshes after the shared helper
both passed with zero warnings/errors and successful terminal JSON records.

## Bundled facade context

The remaining Tsrust diagnostic concerned the original stdlib/ui/api.tsr importing
its private runtime. Entry resolution now shares project::bundled_entry between the
compiler file provider and LSP snapshots. Only this tool's canonical implementation
path with exact bundled bytes receives the existing logical std:ui context. A copied
file, modified buffer or imported user module keeps normal private-import checks.
The compiler's guard itself is unchanged. LSP snapshots still observe the physical
file; an index entry alias preserves physical cursor requests and invalidation.

New facade cases and the extended real protocol verifier must prove original clean,
physical publication/hover, modified rejection/repair and copied-file rejection.
They passed: 2/2 focused cases, plus the real protocol clean/rejection/repair and
physical-file Node hover. The compiler provider agrees on the logical entry. No owner editor/process
was controlled. Replacing a binary does not replace a language server already loaded
in the owner's editor; adoption must be stated separately after validated installation.

## Accepted affected checks so far

| Check | Result |
| --- | --- |
| Distribution packaging | 6/6 passed; edited public entrypoint links rechecked 1/1 |
| Native source/import/artifact integration | 3/3 passed |
| Explicit source pack/tamper/failed staging | 2/2 passed |
| Toolchain setup integration | 2/2 passed |
| Shared cache missing sources/reuse/deployment | 2/2 affected cases passed |
| Portable export with no tools; relocated offline rebuild/metadata | Repaired timeout case 1/1 passed, 65.93s |
| Retained UI default unit contracts | 106/106 passed |
| HTTP runtime behavior, TLS and cleanup | 20/20 passed |
| Environment backend exact standard Rust tests | 2/2 passed after correcting expected args to exclude executable |
| Date runtime contracts | 3/3 passed |
| Cache backend | 12/12 passed |
| HTTP static signatures/emission/ownership | 1/1 passed; shared HTTP cases reused |
| Retained CLI-generated UI content | 1/1 passed, 97.16s |
| New real CLI JSON fixture | 4/4 passed, including exact runtime reuse and artifact rejection |
| Original/copied/modified bundled facade | 2/2 passed |
| Real facade LSP diagnostics, guard, repair and hover | Passed; disk source unchanged |
| Final backend/compiler-LSP and workspace all-target checks | Zero warnings/errors in both configurations |
| Direct CLI check of original UI facade | Passed |
| Fresh CLI backend native HTTP/TS/Rust SDK | Passed, 41.27s; server stopped |
| Fresh backend real LSP clean/error/repair | Passed; disk source unchanged |

The first portable copy hit the former 40s test deadline; the actual larger verified
copy now has a bounded 120s deadline and its complete relocated rebuild passed.
One newly added env test incorrectly expected argv[0]; its implementation was
unchanged, the expectation was repaired, and the exact runtime/tests were compiled
with rustc --test in an owned temporary standard test wrapper and passed. This
avoided another full compiler unit-executable build for a test-only assertion repair.
The wrapper was removed. No custom assertion framework or source snapshot was kept.
A new JSON fixture was corrected to the existing strictEqual API before its next run.
Passed checks were retained, not rerun as a broad regression loop. Disposable current
logs and terminal/live sessions are listed in agent-progress.md; no analytics import
or matched performance measurement occurred.

Fresh review project: generated-projects/backend-portable-sources, created by the
current CLI and explicitly built before its live verifier. The first verifier call
preceded the required application build and failed to spawn the missing executable;
building it and rerunning that affected verifier passed. Runtime workspace inspection
confirmed vendor=false and privateCargoHome=false. SDKs were generated from live
and saved descriptions and exercised with real TypeScript/Rust consumers.

## Required remaining work

Finish formatting, the full-tree checkpoint and guarded consumer refresh. Preserve old running owner processes. Prune obsolete
compiler-owned vendor/build payloads with validated absolute paths. Native docs
startup stack overflow, server-owned API registration, ordinary binding shared-cache
migration, Rust replica compatibility proof, typed body helpers/async std:testing,
trusted paired CLI/LSP installers/update/recovery and real macOS/Linux acceptance
remain open. No release, push, tag, publication or production signing key was created.
