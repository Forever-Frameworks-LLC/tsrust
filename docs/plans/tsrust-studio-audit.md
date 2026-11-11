# Studio capability audit and proof contract

Started 2026-11-10 against source `11f63fd4` plus the Studio working changes.
This is an implementation audit, not product acceptance. The owner authorized
full implementation on November 10. The required inventory in
[the Studio plan](tsrust-studio.md) remains the release scope.

## Reuse and gaps

| Boundary | Existing source to reuse | Concrete missing witness / implementation |
|---|---|---|
| Ordinary project | `tsrust new apps/studio --ui`; `apps/studio/tsrust.config.json` | Current native browser previews its own apps/studio package with working file tabs; project create/save and recovery remain required. The CLI requires the parent directory to exist. |
| Editor | `stdlib/ui/gpui_kit.rs`, CodeEditor initialization and InputEvent subscription; `ui/kit.json` | Full-text `onChange`, controlled values, highlighting and `controlInput`/`readInput` exist. No Studio revision-aware LSP deltas, multi-cursor, IME or large-file acceptance yet. |
| State | `std:ui` typed data snapshots | Compiler now accepts plain records, arrays and nullable data, with independent constructor/get/set snapshots and nested resource/callback rejection. Native snapshot/composition suites pass 12/12. Use an explicit scalar `computed<T>` for live text projections. Direct data-field JSX text lowering remains a gap; hidden native state retention is now verified in Studio; this is not the complete document service. |
| Proposals | `apps/studio/src/documents.tsr` | Whole-buffer path/revision/base-content comparison implemented. Test typing-and-reverting, duplicate acceptance, wrong paths and Unicode. Atomic multi-file application, grouped undo, recovery and isolated tools remain absent. |
| Language services | Existing `tsrust-lsp`, `std:process`, `std:io`, `std:json` | Build a revisioned LSP client in Tsrust; prove stdio framing, UTF-16/UTF-8 conversion, cancellation, crash recovery and stale answers. Existing standalone LSP acceptance is not Studio integration. |
| Rust | rust-analyzer native executable | Inspect/install exact tool through managed tooling; prove lifecycle and project agreement. Not selected or installed by this audit. |
| HTML/CSS/JSON/Markdown | Kit syntax highlighting and Markdown rendering; `std:json` validation | Appropriate language services remain open. Microsoft's HTML/JSON libraries are TypeScript packages, not drop-in native servers. Do not add a Node host implicitly. Inventory native options before choosing. |
| Build/run/test | Independent CLI and checked process streams | Structured diagnostics exist via `check --json`; prove streamed jobs, cancellation, bounded output and stale diagnostics in Studio. |
| Terminal/debugger | Process pipes are reusable transport | Pipes do not establish PTY behavior. Native PTY resize/input/process-group lifecycle, DAP and source maps require separate public capabilities and real workflows. |
| Git | Existing process transport, Kit Diff | Git status/staging/history/conflicts and transactional proposals remain unimplemented. Never run agent tools in the live user tree. |
| Preview | Kit Markdown and native window lifecycle | HTML web-view admission needs origin/focus/disposal tests; Markdown widget alone is not a browser. |
| Credentials | Existing environment APIs | Native credential storage/provider adapters not yet audited/admitted. Never persist secrets as ordinary settings. |
| Native AI | Checked `.bindings.json` packages | Concrete data/slices/Result supported; private-field/Drop resource owners and returned borrows rejected by `project/rust_data.rs` and `project/rust.rs`. Do not fake persistent models with public integer handles/global registries. A checked resource boundary is a prerequisite for reusable streaming model sessions. |
| Training | Compiler-owned `stdlib/ai/admission.rs` via experimental checked bindings | CPU scalar autodiff/SGD boundary and Tsrust iteration witness prepared. This cannot prove adapters, checkpoint files, optimizer-state resume, tokenizer agreement, quantized training or model quality. |

## Bounded local research decision

Investigate Candle **0.11.0** first, without changing backend-only dependencies.
Its exact core/nn manifests default to no accelerator features. The admission
package adds those exact versions with defaults disabled; its lock is checked in.
Source review found tokenizers/Oniguruma and native tokenizer build dependencies
in the closure even for CPU core. Metal inference and CUDA training are separate
admissions. No backend has been qualified for distribution.

mistral.rs remains an alternative. Its current core manifest includes vision,
audio, MCP and code-execution/sandbox dependencies alongside inference; this is
an observed dependency breadth, not a measured memory or quality comparison.
Do not inherit upstream agent execution as Studio orchestration.

Primary sources read November 10:

- [Candle 0.11.0 core manifest](https://github.com/huggingface/candle/blob/0.11.0/candle-core/Cargo.toml)
- [Candle 0.11.0 transformers manifest](https://github.com/huggingface/candle/blob/0.11.0/candle-transformers/Cargo.toml)
- [Candle 0.11.0 optimizer API](https://github.com/huggingface/candle/blob/0.11.0/candle-nn/src/optim.rs)
- [Quantized Qwen implementation, mutable research branch](https://github.com/huggingface/candle/blob/main/candle-transformers/src/models/quantized_qwen2.rs)
- [mistral.rs core manifest, mutable research branch](https://github.com/EricLBuehler/mistral.rs/blob/master/mistralrs-core/Cargo.toml)
- [Qwen2.5 Coder base model](https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B)
- [Official instruction GGUF](https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF)
- [rust-analyzer installation](https://rust-analyzer.github.io/book/installation.html)
- [Microsoft HTML library](https://github.com/microsoft/vscode-html-languageservice)
- [Microsoft JSON library](https://github.com/microsoft/vscode-json-languageservice)

Initial model candidate is official Qwen2.5-Coder-1.5B-Instruct Q4_K_M,
repository revision `f86cb2c1fa58255f8052cc32aeede1b7482d4361`, file
`qwen2.5-coder-1.5b-instruct-q4_k_m.gguf`, 1,117,320,768 bytes, SHA-256
`cc324af070c2ecbfd324a30884d2f951a7ff756aba85cb811a6ec436933bb046`.
This deliberately small baseline is not a claim that it meets quality requirements.
Evaluate fill-in-the-middle with an appropriate base artifact separately.
Check tokenizer/template identity against the actual pinned download.

## Resource observations and stop conditions

Host inventory: `Mac15,13`, Apple M3, exactly 8,589,934,592 bytes physical RAM.
Initial observation: 6,144 MiB swap allocated, 4,864.19 MiB used, system memory
free percentage 35. These are host observations, not benchmark samples or Studio
memory measurements. Other owner applications remain running and untouched.
Disk available at initial inspection: 144 GiB.

For first model-load feasibility, use one model, 2,048 input tokens maximum,
32 generated tokens, one request, no queued requests, and a 180-second inference
execution ceiling. Stop on sustained critical memory pressure or more than
256 MiB additional swap attributable to the experiment; preserve failed evidence.
Measure load peak separately from steady generation, including device allocations.
These are exploratory safety ceilings, not tuned acceptance thresholds or permission
to lower context/task requirements for minimum-machine qualification.

Final quality/resource gates must be established after baseline observations and
before optimization. Planned independent task strata: Tsrust API completion,
ownership/error repairs, multi-file compiler/test-guided tasks, and Rust/related-file
retention. Split by lineage before training/retrieval. Record exact model, tokenizer,
quantization, prompts, tool limits and toolchain; score compiler acceptance, runtime
oracles and task completion separately. Incomplete, cancelled, missing and duplicate
tasks cannot count as passes. Require zero stale applications, live-tree tool writes
or data loss in every correctness run. Qualification requires real full-context
workflows alongside editing/LSP/builds, rather than extrapolating from this load test.

## Current evidence

CLI SHA-256: `6d06fe81356b97a38eb2798801a58a891e1d779dac10e67d920d1dfcffec1ce3`.
Training binding lock SHA-256:
`0d82221a7ebc310726fdfdf004b474fe8aaeee6abf498f5a2988871703993de6`.

- Project created by the ordinary CLI; generated demo replaced with a named
  scratch-buffer editor. `tsrust check apps/studio --json` passes.
- Document suite: 4/4 native tests passed; execution 0.115208 ms, setup/build
  1,610.7825 ms. The sandboxed `/usr/bin/time -l` wrapper then exited 1 because
  its `kern.clockrate` query was denied; test NDJSON explicitly reports zero
  failures. No process-tree memory claim is derived from that wrapper.
- Training package sources resolved through `tsrust bindings resolve`; initial
  sandbox DNS failure retained as setup failure, network-enabled retry succeeded.
  Tsrust project check passes. Native scalar training execution passed 3/3, 1.5945 ms total execution;
  setup/build 414,405.813042 ms (including shared-target wait). `/usr/bin/time -l`
  reports 665,583,616 bytes maximum RSS and 564,593,624 bytes peak footprint;
  those are wrapper/process observations, not summed whole-machine memory.
  No model-training or comparative performance claim follows.
- Whole-project formatter: 4 files, 0 documentation snippets, 0 fingerprints.
  Idempotence and formatted-source native behavior pending final checkpoint.
- No inference, model fine-tuning, quality evaluation, timing benchmark, Windows,
  full editor workflow or distribution acceptance claimed.

Tokenizer revision is `2e1fd397ee46e1388853d2af2c993145b0f1098a`.
Downloaded tokenizer JSON SHA-256:
`c0382117ea329cdf097041132f6d735924b697924d6f6fc3945713e96ce87539`;
config/template JSON SHA-256:
`959e7f1d9a1b7641a6d6ce05ca97b75c7894fcb66cbe5a040406458fb1128ee4`.
The model's downloaded size/hash match the official metadata. These resources
are ignored under `target/studio-models`, not committed or published.

The checked inference boundary uses exact Candle transformers0.11.0 and
Tokenizers0.22.2 with Oniguruma. It is CPU-only and buffered, using the existing
`std:threads` TaskContext cancellation boundary; its engine token loop performs
no editor/agent/tool orchestration. Tsrust owns the prompt template, worker launch
and assertions. Both worker entry and inference tests type-check. Native inference
build/test is pending under an execution watchdog. Native worker input does not
validate artifact hashes itself; this admission invokes only externally verified
pinned files. A production loader must own manifest verification before loading.

A separate Cargo compiler/LSP regression process already owned the shared target
when the training proof queued. It was neither started nor stopped by this Studio
work. No result from that process is claimed as Studio acceptance. New native
proof builds are serialized. This concurrent host activity precludes comparable
quiet-machine timing or whole-machine minimum acceptance from these observations.

First native inference attempt: dependency/setup50,531.681334ms; invalid-limit
control passed0.3185ms. The independent host watchdog stopped the owned process
group at54.910s total after whole-machine swap grew261.5MiB beyond the256MiB
exploratory ceiling. No model answer or suite summary was received. Returncode-15;
actual model inference remains incomplete, not failed arithmetic and not accepted.
Concurrent compiler/LSP regression activity prevents attributing that swap growth
solely to the model. No unchanged retry is justified until the host state or
implementation hypothesis changes.

Exact Candle0.11.0 Qwen source inspection shows token embeddings are fully
dequantized at model load (`quantized_qwen2.rs`), even with quantized projections.
That is a concrete memory-cost candidate, not proof of the stop's cause. Further
backend/device/embedding investigation must preserve the original model/task
results; model size changes require distinct baseline identities and quality runs.
No inference occurs in the preview UI. Studio native preview build is pending.

The host changed to no concurrent compiler builds before attempt2, with one
Studio scratch preview open. Exact same candidate/task and exploratory limits:
setup/build7,060.773708ms, invalid-limit control0.480042ms PASS. Watchdog stopped
at10.603s total after468.5MiB swap growth; sampled owned process-tree peak RSS
607,256,576 bytes before stop. No answer or summary. This preserves the first
attempt and falsifies the hypothesis that overlapping compiler builds were the
only necessary trigger; attribution to a specific allocation remains unproven.
No further unchanged retries. Review a memory-mapped quantized inference library
before changing model size or relaxing resource limits.

Additional primary research: [llama-cpp-2 Rust library](https://github.com/utilityai/llama-cpp-rs),
[version0.1.161 API](https://docs.rs/llama-cpp-2/0.1.161/llama_cpp_2/).
A compiled native library would satisfy the no-separate-server requirement; it
would not prove training. Main-branch0.1.162 manifests are mutable research, not
an admitted release. CMake is installed at /opt/homebrew/bin/cmake. Review exact
release build closure, static linking, tokenizer equivalence and Metal ownership
before selection. No llama-cpp dependency is admitted yet.

Native scratch preview successfully built. Initial custom output directory was
missing; CLI misleadingly reported a compiler defect when Rust could not create
its temporary output directory. Creating that output parent repaired the build;
no generated-code defect was inferred. Ordinary configured build/dev remains the
intended workflow. Native CUA verified named source editor, Unicode paste and undo,
keyboard entry,100-line scrolling, window zoom/restore and both themes. This does
not cover IME, large files, multiple cursors or accessibility completeness.

Owner correction: decorative Studio/Untitled branding row and debug revision
counter removed. The revised preview uses compact working Find/Replace/theme
controls. CUA observed matches, actual Hello->Hi replacement, undo restoration,
light/dark paints and control labels, and search close. One dark independent
preview remains. One initial search query lost its first character (Studio became
tudio); subsequent select-all then typing Hello matched exactly. Investigate
initial focus/first-key behavior and unnamed Kit search AX controls before keyboard/
accessibility acceptance. No fake document tabs or disabled placeholder actions
were added. This is a development scratch editor, not completed Studio.

Formatted document suite rerun passes4/4 with cache hit: setup322.716ms,
execution0.207125ms. Unchanged training acceptance is retained without replay.
Whole-project formatter after owner correction reports0files/0snippets/0fingerprints.

## Retained dev workflow and profiling — November 10

Owner requested continuous ordinary `tsrust dev`, real recovery/recompile tests,
resource/leak observations and optimization only when evidence identifies costs.
Do not replace that with duplicate standalone previews or stop dev after a check.

Fixed macOS UI dev identity: the supervisor previously spawned a bare temporary
`program` executable, which native app discovery rejected. It now moves that fresh
executable into a temporary .app using the existing package metadata/plist path,
with a separate .dev identifier. Other-platform/backend dev execution is unchanged.
No signer, notarizer, icon conversion, archive, updater manifest or release executes.
Studio supplies package.name/id in its ordinary configuration. Focused metadata/
config tests2/2, real plutil validation, process cleanup1/1 and compiler rebuild pass.
Native CUA binds dev.tsrust.studio.dev, sees the working source editor and action bar.

A deliberate parser error produced a precise source diagnostic while the previous
app remained alive and Find still worked. Removing that error and changing the
actual action label to Find in file rebuilt/replaced the child under the same
supervisor; native AX/screenshot shows the changed label. Original supervisor was
restarted once only after explicit owner approval following an automatic-review
rejection. New dev session is retained; live identities are in the single handoff.

Observed dev phases (single executions, no comparable benchmark claim):

| Observation | Total | Important stages |
|---|---:|---|
| First cold dev |47,767ms|runtime prepare1671.1ms, runtime build/publish38815.8ms|
| Metadata edit, same emitted program |2787ms|check1947.8ms, emit301.6ms, program cachehit|
| Fresh supervisor with fixed CLI |46,762ms|check1898.8ms, emit292ms, runtime build/publish37338ms|
| Valid semantic edit after parser-error recovery |11,903ms|check2067ms, emit297.9ms, runtime restore/prepare5016.6ms hit, native+link9097.1ms including prepare|

The 5-second runtime-cache preparation is an investigation target. Cached runtime
artifacts are currently copied/unpacked for safe independent compilation. Profile
that boundary before changing it. Preserve shared-target locking, graph identity,
artifact validation, cancellation and independent file ownership. All-environment
cache identity is intentional because upstream build scripts may depend on
arbitrary environment variables; do not remove that guard to manufacture hits.

Read-only vmmap/leaks observations of the first dev tree: supervisor footprint
124.0MiB, peak1.2GiB, zero detected leaks. App100.0MiB, peak161.7MiB. Two independent
scans of appPID70994 report identical421 unreachable objects/20,368 bytes: mostly
three NSXPCConnection/dispatch cycles and two NSMutableArray roots. That is a stable
scanner finding, not a proved Rust leak, a leak-free result or a growth trend.
Post-recovery supervisor82.5MiB/peak1.2GiB, new app118.2MiB/peak178.7MiB. macOS RSS
varies with compression; physical footprint and retained allocation evidence matter.
No allocation-content dumps, memory graphs or canonical logs were stored.

The new native dev bundle emits a nonfatal macOS sandbox-extension warning for
its temporary .app path; startup, AX, actions and rebuild recovery work. Do not
hide it or weaken system protections. Investigate if it affects required workflows.

Current CLI SHA256 d8d5f8e3a2fd909797ff6b42f87a379882dc528babf0b5e03f517e4b93cb6c85.
Installed CLI/LSP atomically refreshed to current repository0.2.0/0.3.0; existing
VS Code config already selects the repository pair. No LSP source changed and no
owner editor was restarted. Whole formatter0files/0snippets/0fingerprints; earlier
proposal/training proofs retained. Signing/publication remains unauthorized.

## Bounded cache buffers — accepted source, measurements pending

Inspection found an802MiB GPUI runtime artifact. NativeCache checksum and bundle
unpack each used fs::read on the complete artifact. Replaced those whole-artifact
buffers with64KiB incremental hashing and a metadata-first, payload-streaming
extractor. Cache checksum bytes remain compatible with the previous byte-slice
Hash protocol; no namespace/version/identity, lock, inode replacement, corruption
or provenance checks are removed. Interrupted reads retry; changed-length cache
files fail validation. All bundle metadata is checked before outputs are created.
Existing1GiB and512-entry bounds and v2/v3 distinctions remain. ASCII leaf names
are bounded to255 bytes, matching supported filesystem leaf limits;255 succeeds
and256 rejects before creating outputs. Payload bytes and native import behavior
remain covered by existing round-trip controls (Windows execution still pending).

Focused proof: native_cache11/11PASS12.28s, with its ignored subprocess helper
invoked by the process test. Covers legacy checksum bytes at multiple buffer
boundaries, corruption/repair, independent executable readers, concurrent/process
publication, stable cross-process identity, backend discovery and CLI warm cache
invalidation. Bundle tests2/2PASS0.02s; filename-boundary extension rerun2/2PASS0.02s,
compile/setup8.70s. Initial selected test compilation153s. No full compiler/LSP
matrix was repeated. Source-level allocation bounds improved; no quiet-machine
speed comparison or numerical peak-memory reduction is claimed yet. The retained
dev supervisor deliberately keeps its existing executable image.

Third inference attempt retained the exact model, task, token budgets and stop
conditions with the streamed-cache CLI. Setup/build6702.002625ms; invalid-limit
control0.362375ms PASS. Native model proof stopped on critical memory pressure at
10.239s total, system swapdelta520.06MiB, sampled owned-treeRSS507,559,936 bytes.
No answer or suite summary. Pressure subsequently returned to normal level1.
The cache change did not establish inference feasibility; a changed native backend
hypothesis is required before another model run. Do not relax limits or hide any
of the three incomplete resource attempts.

## Native library inference candidate — bounded gate passed

Source adds compiler-owned ai/llama_admission.rs, a checked managed binding package
and an ordinary admission-project path alias. The same worker and test assertions
select @engine through tsrust.config.json; no alternate pass criteria were added.
Dependency roots llama-cpp-2/llama-cpp-sys-2 are both0.1.161, root VCS commit
`e7af607ebd5ec2932709ef918c17cca2f9b0fce4`. Binding-lock SHA256:
`66eb63527e2cbbd31e682c8034efaa14ed62e3fde411a1f9266167d52e1f329a`.
Uses the previously verified unchanged official model and tokenizer artifacts.
The native function requires GPU offload/mmap support, checks embedded token IDs
against Tokenizers0.22.2 exactly, uses context2080 (2048 input+32 output admission
budget), batch2048/microbatch128 and2 CPU threads. All orchestration stays in Tsrust.

Build setup163,195.539917ms; invalid-limit control0.225625ms PASS; actual offline
model generation31,969.934667ms PASS, entire execution31,970.943958ms,2/2 tests.
The exact text oracle is5 with EOS; tokenizer equality is checked before decode.
Host supervisor exit0, stopnull,195.485s total, observed peak swap-usage delta0MiB,
sampled owned process-tree RSS1,251,000,320 bytes. Native logs identify Apple M3,
unified Metal memory and embedded shader libraries (initial compilation5.787s).
This is not a speed benchmark, complete GPU/whole-machine allocation inventory,
proof of zero swap I/O, coding-quality result or full8GiB/editor/LSP/build acceptance.
Retained Studio dev supervisor74090/app76050 stayed alive throughout.

Actual native test executable cache094d7eedb78f0baf is27,548,728 bytes. otool lists
only macOS system libraries/frameworks (System, iconv, Foundation, Metal/MetalKit,
Accelerate, libc++, CoreFoundation, objc); no separate AI runtime, OpenSSL dylib or
model server. Actual CMake: shared=OFF, Metal=ON, OpenMP=OFF, server/tools/app=OFF.
The SDK defaults are disabled, but its sys dependency still enables common=ON
transitively; do not claim otherwise. CMake detects Homebrew OpenSSL headers/libs
for common, yet none is linked into this executable. Clean-machine build closure,
portable export, licensing distribution notices and Windows remain unverified.

CMake parallelism was explicitly1 (observed --parallel1), because upstream chooses
all cores when CMAKE_BUILD_PARALLEL_LEVEL is absent even if Cargo jobs=1. Library
profileRelease and LLAMA_BUILD_SHARED_LIBS=0 were explicit. No dependency was added
to ordinary backend manifests; managed experimental binding isolation remains.

Native owners drop on return; this run's log includes context buffer teardown and
Metal deallocation. Repeated load/unload, cooperative cancellation during stages,
streaming and safe task/tool coordination still require dedicated witnesses.


## Package browsing and typed state — November 10

The current Studio window opens `apps/studio` itself, not the Tsrust checkout.
`workspace.tsr` reads canonical in-root UTF-8 files, de-duplicates paths and leaves
existing tabs untouched after outside/missing/binary-file errors. Listings place
folders first and omit generated target/.git/node_modules trees. Reads are bounded
to 4 MiB and directory enumeration to 10,000 entries. The browser performs no disk
writes. Navigation reuses std:ui Tree/TreeItem, with package contents directly at
the top level and no redundant studio root. Removed the convenience button bar;
Find uses native Cmd+F. Workspace switching, file watches and large-file editing
remain required.

New workspace suite: 3/3 PASS, 3.93 ms execution, 2,953.0 ms build/setup. Existing
four proposal tests are retained without an unchanged rerun. Whole native UI
snapshot/composition/navigation suite: 12/12 PASS, 3.30 ms execution; build/setup
163,863.8 ms. It verifies independent constructor/get/set nested snapshots, Unicode,
CRLF, nullable updates, empty typed record arrays and scalar arrays. The focused
compiler state check passes 1/1 in 1.21 s; six unchanged UI-authoring checks passed
in the preceding 17.36 s run. Early rejection-witness mistakes and the nullable
union emission failure were repaired before these passes; no failed run is a pass.

The emitter reuses owned cloning and the existing borrowed-union `tsr_owned`
conversion. State payloads reject native resources and callable environments even
when nested. Generic Signal ownership and weak captures remain in the existing
native runtime; no JSON state representation or Rust editor implementation was added.

Editor support derives operation names from compiler `ui_method` metadata and
substitutes the checked payload in documentation signatures. Runtime UI declarations
now use public std:ui help when imported through the trusted native bridge. Shared
native identity lookup handles linked compiler names and imported aliases. Debug
LSP builds now use opt-level 1 with debug checks retained, matching the compiler's
existing profile. Before that change, UI project indexing missed the existing
3-second request deadline; no deadline was raised. New completion rows pass in all
four server modes, with identical answers including syntax-error recovery (21.47 s total), and typed hover/signature/documentation/counter
consistency passes all four modes (11.22 s). Header/checked project alias unit
passes (12.98 s in the prior unoptimized test host); native alias unit passes0.63 s.

The first Signal member-agreement run completed in418.10 s but failed because its
oracle treated the compiler's explicit rejected `.length` receiver diagnostic as
an accepted field. A direct CLI check confirms Signal.length rejects. The shared
oracle now distinguishes that rejection; the actual-compiler regression retains
ordinary array length acceptance and passes0.53 s. The corrected full selected Signal row rerun passes1/1 in413.88s with the
complete candidate scan. No members/receivers were excluded. Do not claim full
editor or compiler/LSP milestone regressions from these focused checks.

Owner approved replacing the scratch supervisor with the built browser. A first
sandboxed launch crashed at NSPasteboard generalPasteboard after XPC denial. The
same authorized replacement was completed with desktop access, preserving the
existing permission model. That earlier supervisor was PID15826/session51314,
child29484, bundle dev.tsrust.studio.dev. Cold dev47,677 ms; observed warm
source rebuilds14,767/14,566/14,624 ms, runtime preparation5,324.4/5,609.7/5,023.4 ms
with runtime cache hits. These are development observations, not quiet-machine
benchmarks. This supervisor includes the accepted streaming cache buffers.

Earlier native CUA verified light/dark themes before removing the theme button.
Latest native CUA verifies std:ui Tree, folder click/Right-key expansion, direct
workspace contents without a root wrapper, no button bar, native Cmd+F, and real
source/config/workspace tabs with full-height flat code views. It also verifies correct live
status through an explicit computed<string> projection, and binary-file errors
without replacing the active file. Find targets the active config file and matches
schemaVersion. Initial CUA typing again lost the first character; raising the window
and select-all/retyping yielded the exact query. AX labels/focus remain open. A
read-only typing attempt left source unchanged; full editable/IME acceptance is
not implied. This earlier viewer lost scroll position: sync_kit's collect
used visible_root and retired hidden CodeEditor state. The retention fix and
subsequent native acceptance are recorded below; undo and full focus acceptance
with editable buffers remain required. The browser
also constructs child Nodes once outside reactive JSX list expressions so passing
a Signal alone does not rebuild and reset directory state.

No complete editor, safe-save/recovery, agent integration, AI quality/resource
qualification or release acceptance is claimed.

Historical appPID19889 read-only vmmap observations with3open file tabs/main active: supervisor physical
footprint49.3M/peak408.0M; app258.6M/peak317.0M (units verbatim). Different inputs
and sessions from the earlier1.2G supervisor peak; not a controlled comparison.
No allocation-content dumps were retained.

Restricted macOS leaks scan of that historical appPID19889:420leaks/20,336total leaked
bytes,523,615malloc nodes/192,915KB. The process is not debuggable; no allocation
contents were saved. Attribution and growth are unresolved, so neither leak-free
operation nor an expanding Rust leak is established.

After focused acceptance, the installed CLI0.2.0/LSP0.3.0 pair was refreshed with
staged copies and atomic replacements. Hashes match the current repository tools:
CLI0f14ca6e610ae4366a995dab150a275d2c1aca2841ef6c2a3115b71984752bd7;
LSP4d55f9d43dcabaf843210256fe81ab7aaecd47beb1cc1584a5b567777e7bc688.
No active editor/server process was found at refresh. Running Studio supervisor
keeps its accepted image and remains active.

Latest native Tree/root-removal source rebuild passed in13,356ms, including
4,419.3ms runtime preparation. Typed data/compiler/LSP prerequisite is locally
checkpointed at791d4d43; the package viewer and these records form the next checkpoint.
No unchanged acceptance executables were replayed for the layout correction.

## Retained native panel state — November 10

The package viewer is checkpointed at89cd2a72. Investigation of tab scroll resets
found that sync_inputs and sync_kit both used visible nodes as the disposal set.
The renderer now retains native editor/input/component state by mounted identity,
including hidden panels and retained routes, while creation and prop synchronization
still visit visible nodes only. Removed nodes retire state. Hidden application
menu owners remain an exception: their application-wide menus are removed while
hidden and reinstall when shown. No app-side overlapping-panel workaround is used.

The new node-lifecycle regression passes1/1, with0.00s execution reported by the
Rust harness and3.66s compile/setup. It checks hidden membership, stable identities,
conditional removal, and exactly-once destruction of retained state. The original
compiler build passed in2m22s; review then added the menu side-effect safeguard,
and the final compiler build passed in2m41s. These are setup observations, not
performance benchmarks. Workspace/all-target compilation passed in2m30s before
that final renderer-string safeguard, with warnings in unrelated existing test
helpers. Final backend/all-target compilation passed in3m26s, again with warnings in
unrelated test helpers. The final native Studio executable builds successfully.
The final workspace/all-target compilation also passed in1m28s. Warnings
remain confined to unchanged test helpers; no renderer/node warning was emitted.

Owner authorized “Restart once and verify tab state.” The old supervisor15826/
app29484 stopped normally; session6863/supervisor41326/app41377 now uses the fixed
renderer and remains running. Exactly one Studio preview is present. Startup was
45,087ms: check2,069.4ms, emit312.5ms, runtime preparation1,498.0ms, runtime
build/publish36,864.4ms and native+link42,310.5ms. Runtime cache missed. These are
development setup observations, not benchmark comparisons.

Actual CUA verifies main.tsrx scroll position (line34 at the top) survives a trip
to the config tab. Keyboard-selected final brace survives a tab round trip: native
Cmd+F adopts that selection as its query. Separate setTheme and schemaVersion Find
queries remain attached to their own documents across tab switches. CUA phase
“Verified retained editor tab state” completed. AX selected-text range is unsupported;
initial config search typing still dropped its first character and needed
select-all/retyping. Search AX names, full focus/IME, editable undo/redo and actual
hidden application-menu side effects remain unverified. No unchanged native test
suites or AI executables were replayed.

Accepted CLI was atomically refreshed at /Users/seancotter/.cargo/bin/tsrust;
repository, installed and live supervisor use SHA256
ebf586f90a92b181001b1a8d0012c6e7a415517453fd197063be1d6146f6c899.
LSP stays on its prior accepted image; no intelligence API changed.

After native tab-state acceptance, vmmap -summary on the new processes reports
supervisor41326 physical footprint384.5M/peak384.5M and app41377
253.1M/peak253.1M, with two open tabs and per-file Find queries. Units are verbatim.
This is a fresh-session baseline, not a controlled comparison or leak/growth proof.
No allocation contents were dumped.

## Workspace document sessions — November 10

OpenFile now owns the shared Document record instead of duplicating path/text
in a disconnected preview model. createDocument uses existing std:uuid OS
randomness and keeps its formatted session ID as plain typed data. Reopening a
closed document creates a fresh session; proposals validate session, canonical
path, revision and original text. A saved-text baseline derives dirty state.
Clean external reload updates text/baseline and revisions; dirty reload refuses
without changing either. Model close refuses dirty text unless the caller makes
an explicit discard decision, then preserves another active file or selects the
remaining neighbor. These are model operations, not native close prompts or saves.
No new dependency, native editor implementation or disk-write control was added.

openFile reuses existing open sessions, so selecting the same file cannot reload
and discard a draft. It now refuses non-regular paths before a text read and
handles a filesystem-root workspace prefix without doubling the separator.
Canonical in-root/UTF-8/size guards remain. This is not race-safe hostile filesystem
access: worker-backed file identity checks and transactional writes remain required.

Final formatted native suite:10/10 PASS,4.31ms execution,3,039.5ms build/setup.
The original proposal and file-opening assertions are preserved with the integrated
record representation. New cases cover identical-content reopen isolation, clean
reload/stale proposal rejection, dirty reload preservation, no-op revisions, dirty
close rejection, explicit discard, active-neighbor selection, last close and
reopening from actual disk. The added non-file error case preserves the current
tab/text. Earlier integrated10/10 passed4.21ms; expanded workspace-only4/4 passed
3.57ms before formatting. These are correctness observations, not timed benchmarks.

The first check rejected unwrapping a Result<Document,string> inside the FsError
catch (mixed error payload has no universal message field). The final code matches
that Result explicitly; the live dev supervisor preserved the old app on failure.
No compiler or error-contract weakening was used. Whole-project formatter changed
3files, verified tokens/idempotence, and the final formatted suite above passed.

Normal source rebuilds use the retained session6863/supervisor41326. Initial
session-model build51,764ms included a runtime cache miss (prepare1,599.8ms,
build/publish41,486.4ms); final formatted-source rebuild12,184ms used a runtime
cache hit. Current app48203 opens main.tsrx and the actual documents.tsr with native
Tree and tabs. CUA phase “Verified workspace document session integration” completed.

Observed dev limitation: changing only the test source still restarted the app
after an identical native-cache hit (3,015ms), resetting open tabs. Suppress
unchanged-app replacements and preserve unsaved drafts before exposing editing;
this is not accepted as safe live editor development. Keyed panel reconciliation
is also required before native close/reordering: current reactive lists retain
rows by position/shape. Safe-save atomicity/external-change detection, recovery,
native close prompts, editable undo/redo, full AX/IME and agent file isolation
remain open. The app remains read-only.

## Editor shell alignment — November 10 (in progress)

Owner requests visual design alongside technical work: coherent gray surfaces,
explicit rail/tree/editor dividers, collapse/resize, tooltip icons in a custom
header, no app-name banner or development footer, text zoom, close/reorder,
localization and configurable navigation/terminal/chat placement. Full details
are required scope in the plan and apps/studio/AGENTS.md. Persistence, localization
and docking are not implemented by merely adding a settings menu.

The shell now uses a custom draggable macOS header with native traffic lights,
Files/Find file activity buttons, Kit Resizable, real filtered filename results,
native tabs with close buttons, appearance/text/move-tab menu actions and keyboard
commands. Chrome, navigation and editor components are separate `.tsrx` files.
Errors display on failure; the permanent read-only/path status bar is removed.
The new JSON theme uses two neutral surfaces and explicit borders.

Removed the Studio-specific extension/language function and stored language field.
CodeEditor receives the extension token; existing std:ui Tsrust aliases and Kit's
syntax lookup select the grammar. In the active lightweight Syntect backend,
imports map to preproc, whose prior theme color matched plain text. The Studio
theme now gives preproc the keyword color. Actual main.tsrx/workspace.tsr imports
render distinctly. An initial Tree-sitter hypothesis was unrelated to the selected
backend and was discarded: no backend/grammar dependency change remains. Its
standalone experiment compiled in4m43s but ran0tests and is NOT acceptance evidence.
The queued obsolete compiler build was stopped; the dev supervisor was preserved.

Native testing caught a stale click after a close button: the parent tab event
could reselect a path already removed, leaving a blank editor. selectFile now
rejects paths that are no longer open. The native close round trip restores the
remaining main.tsrx editor without an extra click. Model moves preserve active
session and dirty text. Full suite11/11PASS6.47ms execution/120,031.5ms setup; after
the stale-selection witness, workspace5/5PASS4.23ms/93,605.9ms setup. These setup
times include rebuilding affected native support and are not benchmark claims.

The shared Node list reconciler now uses distinct nonempty document IDs for rows,
preserving unchanged nodes across moves and sibling removal. Unlabelled rows keep
position semantics. Mixed insertion cannot steal a retained keyed row. Duplicate
IDs reject before discarding the previous tree. Live class bindings are excluded
from structural fingerprints so font changes preserve state. New ownership/identity
witness passes1/1 in0.00s (final3.53s setup), including Unicode draft preservation,
move/removal, unlabelled prefix insertion and duplicate rejection. Existing list
control passes1/1 in0.01s on the final test binary.

Tree rendering now forwards the component's text size to ListItem (which otherwise
sets its own default size) and constrains labels to one-line ellipsis. Shortcut
and Ticker native elements are now absolute/zero-size: their previous empty flex
items created80px of trailing gap in the title bar. App commands also sit outside
the visual action group. This fixes the shared nonvisual layout behavior rather
than adding compensating margins. Final compiler/native build results after the mixed-row edge repair are below;
the live renderer update remains pending.

Before that update, actual CUA verifies custom header/no visible app-name, visible
rail divider, left-aligned search, exact workspace query/results and opening a
result, closing back to an intact editor, sidebar collapse/full editor width,
actual divider resize (image coordinates588,500 to788,500), settings actions and
separate Cmd+=/Cmd+- editor zoom. The action icons now reach the intended right
inset after removing invisible-command gaps. Resize and commands work, but Tree
zoom/ellipsis, reordered native editor state, light theme, complete IME/AX and
docking/localization/persistence still need focused acceptance.

Workspace/all-target compilation passed1m31s and separate compiler/LSP backend
all-target compilation passed1m20s. Warnings are in unchanged test helpers, including
unused imports/dead code and an existing fetch_update deprecation. No changed UI
source emitted a warning. Final mixed-row code is covered by its library witness;
final native load still pending. Current live supervisor remains41326/
session6863, running the previous accepted renderer. Do not restart without the
required keep-running authorization.

Final mixed-row compiler build passed2m16s and the final native Studio executable
build passes. Candidate CLI SHA256 is
42db19302c4f46245262af62e6e2351c6044e6d4cb7e22957d323b1171a88519.
The retained supervisor still embeds the previous renderer; new Tree behavior
and keyed native editor state need a one-time authorized load before acceptance.
CUA phase “Verified compact editor shell controls” completed for the already live
shell. The existing unlabelled-row control also passes on the final test binary.


## Retaining unchanged development applications — 2026-11-10

UI checkpoint acbcb8ed preserves the compiled/tested shell section. The next
prerequisite addresses observed test-only source edits resetting the live app
after an identical application cache hit. The dev supervisor now compares exact
native cache input bytes and generated macOS Info.plist bytes, then checks the
existing child is alive before retaining it. Changed app inputs or app metadata
still replace the child. An exited child relaunches after the next successful
build, even if inputs match. Failed builds retain the last working process.
Fresh unused candidate directories are dropped. No new dependency, application
host, source exclusion or unbounded historical identity list is introduced.

New identity unit passes1/1 in0.00s, covering equal/changed native inputs and
changed/removed macOS metadata. Initial Cargo exact-filter invocation selected
0tests after2m12s build; this is setup only, not acceptance. Correct direct binary
invocation ran the fully qualified witness. Existing public dev process regression
is extended for unchanged test-file edits and relaunch after child exit; final
process test passes1/1 in3.22s execution (12.51s build/setup); backend compiler/LSP
all-target compilation passes1m19s. Warnings remain in unchanged host test helpers
and reference controls, including unused/dead code and existing fetch_update
deprecation. No changed dev/cache source warning. Repository CLI SHA256 is now
074d80e8c16f828d279e44d00c9df866d3baf8b1fd3e7c8f61693ac7b7a5b426; installed
consumers retain their previous images pending native acceptance. Kept the actual Studio
supervisor41326/session6863 running throughout. Restart question is pending;
no authorization reply or live load is claimed.


## Atomic storage prerequisite — 2026-11-10, implementation under validation

Previous continuation made progress with checkpoint79c37206: unchanged successful
application builds retain the live process. No supervisor restart reply yet;
keep session6863/supervisor41326 running. UI checkpointacbcb8ed remains compiled
and partially observed, with new Tree/keyed-row renderer acceptance pending load.

Studio cannot use existing writeTextFile for reliable saves: it truncates before
completion. Added reusable writeTextFileAtomic(path,text,expectedText?) behind
managed std:fs bindings, rather than hiding a Rust editor implementation in the app.
The existing temporary-files dependency supplies secure same-directory staging;
ordinary file and temporary-directory callers do not include the new atomic module.
Native separation/runtime-feature detection includes that selected module.

The operation synchronizes staged data before rename and its directory afterward.
It rejects non-regular/symlink, read-only and multi-hardlink targets; exact optional
expected bytes plus observed identity/metadata are checked. Fresh targets publish
without clobbering a concurrently created entry. Post-commit directory-sync failure
returns committed-not-durable, explicitly meaning contents DID change. Studio must
retain recovery state and reconcile the baseline on that outcome. Final rename
still has a race with uncooperative external writers; this is not a cross-process
compare-and-swap or filesystem sandbox. No arbitrary read-size cap is introduced.

On macOS fcopyfile copies metadata from the held source descriptor, with explicit
owner/group/mode verification and a fresh modification time. Other Unix paths
preserve permission bits only; Windows returns unsupported before touching storage.
Full Windows Studio save acceptance remains required. Reference semantics:
[Apple copyfile manual](https://github.com/apple-oss-distributions/copyfile/blob/main/copyfile.3),
[tempfile persist](https://docs.rs/tempfile/3.27.0/tempfile/struct.NamedTempFile.html#method.persist),
[Rust rename](https://doc.rust-lang.org/std/fs/fn.rename.html).

Full callable/argument/error/ownership docs and a portable documented example are
in builtins.json; Windows example expects unsupported rather than pretending a
save. Added public .test.tsr runtime coverage and selected-module/type rejection,
completion, hover and parameter-help witnesses. Backend tests inject staging and
post-commit sync failures, external mutation and same-byte inode replacement; macOS
host metadata witness checks permissions/ownership/ACL/xattr and old-reader data.
Whole formatter changed one fixture and one snippet; token preservation and
idempotence checks pass. Native unit compilation is running; final executable and
editor acceptance, consumer refresh and checkpoint remain pending. No UI editing
or saving is enabled by this prerequisite alone.


Atomic backend witnesses pass3/3 in0.03s (3m17s build/setup). Selected runtime
module/dependency isolation passes1/1 in0.23s (2m14s setup), retaining the old
safe temporary-directory and ordinary-file exclusions. Expanded parameter/type
rejection control passes1/1 in0.02s (0.95s setup). Initial public source checks fail
before execution with error-effect: the operation was absent from the audited
native failure-contract list, making both direct and first-class calls opaque.
Failure-only tracing identified that omission; no disk operation ran. Corrected
both production/audit registrations and added mutating-observer exclusion. The
original failing alias and direct-call cases remain, with no weaker error checks.
Focused compiler effects validation is building; public native runtime/example
and LSP checks remain pending. No unchanged backend metadata witness is replayed
for this compiler-only repair.


Validation continued November11. Corrected audited native-effect witness passes1/1
in0.01s (3m16s build/setup), and mutating-observer exclusion passes1/1 in0.00s on
that final test binary. Rebuilt CLI/LSP pair passes2m28s. Public native runtime
cases and the actual generated catalog example compile together once and pass3/3
in63.47ms execution,1,561.3ms build/setup. The original alias witness remains.
Affected completion matrix2rows x4modes passes2.37s; filesystem help consistency
8cases x4modes passes8.78s; catalog-help contract passes1/1 in0.03s. No new receiver
type; existing FsError checker agreement is retained. The legacy exhaustive
catalog host test's count and Windows-output selection are maintained, but that
per-example native loop is not rerun for this change. The new ordinary runtime
acceptance uses std:testing/tsrust test and a single batched native program.

A final metadata check now also verifies copied BSD flags before publication.
Extended macOS host witness proves UF_HIDDEN retention and modification-time
advance, in addition to ACL/xattr/mode/ownership and old-reader behavior:1/1PASS
in0.03s,3m17s build/setup. Unchanged fault/effect/editor controls are retained.
Final consumer rebuild, affected generated native rerun, all-target compilation,
installed refresh and checkpoint are pending. No live app restart or editable
buffer integration is implied by these results.


Final pair rebuild passes2m27s. Final affected public native runtime/example batch
passes3/3 in45.58ms execution and1,874.7ms build/setup. Backend compiler/LSP all-target
compilation passes1m18s. Every warning was inspected: only unchanged host helper
and reference unused/dead code, an HTTP import and existing fetch_update deprecation;
no changed source warning. These durations are observations, not benchmark samples.
Installed CLI/LSP were staged and atomically replaced, independently matching repo
SHA256 values f5e31107f51340b11e6a512c63a50551cfb858e6e43a1861ab88612a001fa292
and6d55dbde9e6143242f57132cb8067c9cf6f47169b1a2b00cc9f67c38a011963a.
Configured repo overrides resolve to that pair. VS Code has an empty editor and no
active tsrust-lsp; CUA “Verified no active editor documents” completed. No document
or server restart was needed. Live Studio41326/76014/session6863 stays running on
the earlier image, with a one-restart request still unanswered.

Deleted disposable generated catalog sources and empty private runtime cwd, the
obsolete pre-flags atomic test application cache variant, and171,774,617B of obsolete
test executables/dependency files. Current tools, final compiler unit binary and
current native cache remain. No archive/source snapshot/command-log import, broad
unchanged suite replay, performance timing campaign, push or release occurred.
Atomic storage has focused macOS acceptance; safe Studio saves, recovery, settings
and Windows implementation remain required.


## Native document docking — November 11

The section following bd88e292 uses Kit Dock for editor tabs/panes. Public
Dock.tabbed/value/onChange/onClose and DockPanel.value/closable/padded/splitControls
are checked/generated alongside their LSP help. Native keyed reconciliation keeps
user splits and editor instances across panel additions/removals. Final-file close
uses the controlled close callback; dirty panels disable closing. Shared row
identity/duplicate regression passes1/1 (0.00s execution,1m49s setup including lock
wait); existing branch tests6/6 (0.00s,3.58s). The formatted Dock callback contract
witness passes1/1 (1.63s,2m22s): borrowed strings check/emit; int callbacks reject.

Actual CUA observes native tab reorder with an unsaved Unicode draft and retained
undo, final-file closing, direct right/down splits, nested three-pane redraw and
focused Cmd+S writing exact Unicode bytes only to the focused file. The .gitignore
test bytes were undone/saved back to SHA256ad8ebeaeb880ec8c2388bbdf361e28d59d1ed0d7ae34a5c0a0e8ba60b7265483;
config remains6facc291d6b15db7aac40cb3ad3230d45aba8bfc0c12d0e6c1bc817ce7bc8439.
Recovery contains0drafts afterward. No owner draft was discarded.

Failed native compile attempts exposed defer closure arities, missing builder
trait and wrong legacy icon enum; corrected with current APIs/asset icons. A
real first-document panic exposed reentrant TabGroup reads from its toolbar;
can_split is now cached during root synchronization. Native nested splits then
exposed stale cached pane content: root/editor notifications alone failed. A
coalesced next-frame invalidation of cached DockContent plus editor/root after
DockEvent::LayoutChanged passes the exact no-focus-workaround three-pane witness.
No upstream installed dependency source was changed. Latest native dev compiles,
links and starts current source; prior failed attempts are not acceptance passes.

Pointer resize/edge drops, AX tab names and inactive editor nodes, overflow reveal,
layout persistence and duplicate views remain incomplete/unverified. The optional
owner mouse-gesture check has no response yet. No complete Dock/editor or broader
Studio goal acceptance is claimed. Native startup66,407ms includes40,724.0ms runtime
build/publish; these are setup observations, not comparable performance measures.


### Native dock accessibility — 2026-11-11, after b0504b89

Shared GPUI SDK source SHA25678e5dcb91eeaf81cb2672ebd87dd09c42ad664e0a91cf84cf89a9f931fe042cb.
Scope is the shared native renderer only; no public signatures, generated bindings,
Studio models or editor document ownership changed. DockArea/TabGroup renderer
hooks retain Kit layout/drop/resize behavior and Kit Tab/TabBar components. Tabs
now use label/aria_label; close and pane actions have explicit labels. The outer
pane AnyView cache is removed while retained editor entities keep their history.
Selection reveal tracks panel identity and visible index, without scrolling on an
unchanged selection. Overflow/manual-scroll behavior remains unverified.

Before: live app99663 has three visible panes but one accessible editor and empty
AX tab names. After: app22030 full-tree assertions pass for three named active
tabs and three editors following nested splits. Unicode draft survives native
AGENTS.md/.gitignore tab reordering; Undo returns the original clean buffer.
Labeled close works, dirty documents have no close action, pane menu maximize and
Restore pane retain all three editors. A diff-count assertion was inappropriate;
the corrected full AX snapshot proves restoration. Divider pointer dragging still
shows no movement under CUA; no gesture pass claimed. Overflow-menu buttons remain
unnamed. No installed dependency source or alternate UI toolkit was introduced.

Compiler/LSP buildPASS (2m37s setup); public tsrust buildPASS and dev native
runtime/application buildPASS. One premature public build used the earlier CLI
while replacement was still linking and failed on a corrected private-module
path; current binary/source identity was then checked and the build rerun. No
failed run is counted as a pass. Preserve prior unchanged compiler callback,
branch identity, Studio18/18 and atomic-storage evidence; no broad replay needed.
Whole-project formatterPASS0files/0snippets/0fingerprints before native build;
final formatter/diff checks precede checkpoint. Runtime assertion checks in CUA
cover names, all visible editors, Unicode retention, undo and restoration.

Current PTY94264, supervisor21372, app22030; installed CLI
2c3cb0c391a6b041ae1e8fcc700fb6e3caf984dd348cd5cf3807bc9e1c4b5c14,
LSP70ca9934ec7c2748efb5fc3356ff42bc2eb92caaef0ad5b84b59bf0c4f8759ab,
independently verified against target/debug. Zero recovery drafts and original
.gitignore/config hashes confirmed afterward. No draft was saved to disk during
this witness. Three clean panes remain open. Startup66,552ms includes runtime
miss41,047.9ms; RSSapp76,480KiB/supervisor5,440KiB are single observations, not
comparable benchmarks or leak acceptance. Removed966,874,559bytes obsolete
runtime754fd283c74c75dd/appaceb2dcec93b341d caches; keep current dev/runtime,
public-build and unchanged test consumers. Full Studio goal remains active.


### Named tab overflow and selected reveal — 2026-11-11, after8020b8e3

Shared SDK source SHA2569a389a78ff5c4e7fca0c86219003371c32fd2b1aaecf125680ee5bf4f1a328ae.
Replaced the unnamed Kit TabBar overflow control with a named Kit Button/PopupMenu.
Each menu item retains the source group index, file title and active check state;
selecting it invokes Kit's existing TabGroupContext.select_tab. Kit Tab/TabBar,
scroll tracking, editor entities, split controls and close guards are unchanged.
No public bindings or installed dependency source changed.

Native app31387 passes: named Open tabs control; menu lists all four left-pane
files with active check; newly opened final source tab is fully visible despite
overflow; selecting clipped first .gitignore reveals its complete tab and buffer;
selecting clipped last document-actions.tsr reveals that complete tab/buffer.
The config pane remains visible. Full AX assertion verifies final active tab.
A menu-selection regex initially missed AX diff markers; using the observed
current item and a full snapshot resolves the test harness issue. Manual CUA
horizontal wheel scrolling shows no movement, so respect for deliberate manual
scrolling remains unverified. No divider/edge gesture pass is claimed.

Compiler/LSP buildPASS2m42s setup, public native Studio buildPASS, current dev
runtime/application buildPASS. FormatterPASS0files/0snippets/0fingerprints and
final diff check before checkpoint. Unchanged Studio/compiler/storage evidence
is preserved, not replayed. Installed CLI/LSP independently match target/debug:
00ef7f70abe8dbfc1997fec921b489ba6252fe69934958257503d7e41cbc7bb9 /
33ef7da029e516410b5427144160833df1dafea724868b98982a1fcdb81902f3.
PTY24959, supervisor30710, app31387 remain live. Five clean documents across two
panes, recovery drafts0. No text edited during this witness. Startup65,916ms
includes runtime miss40,529.8ms; app RSS67,312KiB/CPU0.0%, supervisor5,600KiB/0.0%
are single observations, not benchmarks or leak acceptance. Removed1,934,800,837
bytes obsolete8020 caches; current dev/public/test consumers retained.


### Persistent live region placement — 2026-11-11, afteraff7501e

Studio config now validates navigationSide/activityBarSide (left/right), and
panelPosition/chatPosition (left/right/bottom). Optional stored fields migrate
existing schemaVersion1 theme/font files to defaults without rewriting until Save.
Settings applies the complete resolved model to the existing workspace. AI Chat
has an independent close/toggle region, allowing it and task/terminal panels to
coexist. layout.tsrx composes native Resizable panels, with appropriate seam borders
and full-height navigation/activity surfaces. No handwritten Studio Rust.

Shared Resizable.reverse is a live boolean; it reverses native rendering order
without moving owned source Nodes. Divider state uses the group's axis/order key
because Kit sizes belong to displayed indices. The attempted reactive keyed-list
approach required illegal Node cloning and was removed; no ownership rule was
weakened and branch.rs is unchanged. Helper numeric/capture/constructor diagnostics
were corrected before final acceptance. API/LSP docs and exports are generated
from kit.json. SDK SHA2566ed27646458b2deac70d92cc6eefd392e75a60282cc5b1181bde80c002746725.

Acceptance:
- std:testing settings suite2/2PASS454.62ms execution; setup not separately timed.
  Existing38theme round-trips plus old-file defaults,36placement combinations and
  invalid-placement writes preserving the prior file. An initial borrowed-loop
  fixture rejection was fixed by deliberately copying owned config strings.
- New ui_authoring reverse contract1/1PASS1.20s execution,2m30s setup. Live boolean
  input checks after format round-trip; integer input rejects. Main checkPASS.
- Final compiler/LSP buildPASS2m44s setup; public native Studio buildPASS; latest
  dev compiles/links/starts all changes. Generator parityPASS and repeated whole
  formatter0files/0snippets/0fingerprints. Unchanged Studio/storage/Dock evidence
  is preserved; intermediate old-API checks and cancelled builds are not passes.
- Actual app55250: default left/left/bottom/right; right/right/left/bottom; and
  left/right/right/left placements observed through Settings Save/close/reopen.
  These exercise every supported region position;36combinations are model tests,
  not36native checks. Terminal/chat coexist and close independently. The unsaved
  Unicode draft survives all moves. Two split editors and both AX fields survive
  restoration of defaults; Undo then restores clean original bytes.

Original settings restored byte-for-byte through its Settings editor:
SHA2560c806c00cece149957766b2f47196beb82fceb1917e0d1f732997fcfae7fdddb.
.gitignoread8ebeaeb880ec8c2388bbdf361e28d59d1ed0d7ae34a5c0a0e8ba60b7265483,
config6facc291d6b15db7aac40cb3ad3230d45aba8bfc0c12d0e6c1bc817ce7bc8439
unchanged. Recovery drafts0; no test draft saved. Two clean panes remain open.
PTY45614/supervisor54542/app55250 remain live. Installed/target CLI
ce804822cd883ed595af20bddabc718107c80ffd5230d0c851a1756371ef6444 and LSP
dd5bb636a475ea7f87c7027d68c73a9671adecf1f03bbfe97b837e35604c64b7 independently
match. Startup68,348ms includes runtime miss41,471.2ms. App RSS76,000KiB/CPU3.2%,
supervisor5,536KiB/0.0% are single observations, not leak/benchmark acceptance.
Removed1,934,753,405bytes obsoleteaff native caches; current dev/public/settings/
previous full-suite consumers retained. Locale/i18n, geometry/view persistence,
manual gestures and actual terminal/AI services remain incomplete.
