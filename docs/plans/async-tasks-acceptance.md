# S13 lazy async tasks

Source contract: `async function work(...): Task<T>`, with owned lazy tasks and
consuming `await`. Arguments/defaults run at construction. Body errors reach
await; argument errors reach the call. Retained loans last through completion or
disposal. Local futures do not acquire a Send requirement.

Implementation uses distinct task contracts over concrete owned environments,
real native async methods, and a wake-aware local entry driver. S14 remains the
Tokio scheduling/I/O and started-job lifecycle section. No start/cancel API,
shared Promise settlement, async arrows or detach is published by S13.

Focused witnesses cover owned and borrowed inputs, mutable loans, defaults,
generics, named/default imports, Task parameters, record fields, arrays, nested
tasks, conditional tasks, error transport, finally, single-use rejection and
unused-path warnings. Warning paths include implicit call failure and finally
handling. Independent tasks abandoned at one location retain separate creation
locations in CLI diagnostics. Build/run/test also render warnings, without
changing emit output. Suppression does not authorize invalid codegen.

The generated-future witness forces real Pending/Ready transitions, retains the
owned string allocation and verifies exactly one disposal before first poll,
while suspended and after completion. An Rc-bearing local future proves no
unnecessary Send bound. Source finally is verified for normal error/completion;
future disposal does not promise execution of source cleanup statements.

Readable programs: `examples/async-tasks/main.tsr` and `borrow-and-recover.tsr`.
Editor witnesses cover Task and await-result types, source diagnostics, imported
async functions, warning severity, keyword highlighting and reviewed absence of
premature task lifecycle methods.

Matched workload: `benchmarks/async/lazy-await.tsr` and `.ts`; TypeScript uses an
explicit lazy thunk, compiled with tsc before Node. Independent checksum 4096²
is 16777216. Protocol reuses three processes, eight warmups and 31 samples;
verification uses one process and one sample without timing claims. This is a
sequential construction/await workload, not a scheduling or I/O benchmark.

Validation counts, immutable SQLite record names and installed consumer hashes
are recorded below after the final gates.

Focused acceptance: 23 compiler tests pass, including the generated suspension /
drop witness, later-argument failure responsibility, tasks returned by await,
value-producing task matches, discarded sequence operands, literal preservation
Task-only parameter contracts and generic awaited values. The borrowed-data
fixture verifies record/view loans through generic task transfers. Returning a
child task retaining borrowed data is rejected at source; return its awaited
value instead. Nested owned task results remain supported. Three async LSP tests,
the completion matrix, checker agreement and editor consistency pass. Compiler
library regression: 1224 passed, five ignored. Callable / exception integration:
172 tests passed across the 14 existing affected targets. Extension compilation,
executable discovery tests and real TextMate tokenization pass.

SQLite records include `tsrust-s13-acceptance-final.log`,
`tsrust-s13-family.log`, `tsrust-s13-family-repair.log`,
`tsrust-s13-editor-2.log`, `tsrust-s13-editor-4.log`,
`tsrust-s13-editor-final.log`, `tsrust-s13-responsibility-checks.log`,
`tsrust-s13-editor-responsibility.log`, `tsrust-s13-emission-checks.log`,
`tsrust-s13-contract-checks.log`, `tsrust-s13-extension.log` and
`tsrust-s13-examples.log`. Failed attempts are preserved alongside repairs.

Initial matched measurement (before the source checkpoint): 93 samples per engine,
median batch 0.009458 ms native and 0.137 ms Node after tsc. Both produce 16777216
in every sample. Records:
`benchmarks/async/lazy-await-2026-10-15T12-01-23.129Z-verify.json` and
`benchmarks/async/lazy-await-2026-10-15T12-03-08.985Z.json`.
This workload has no comparable prior async measurement; it is not evidence about
Tokio, multicore throughput or suspended I/O latency.

Complete family regressions: 4074 compiler tests passed (seven ignored),
including the added generic-transfer witness; 693 LSP tests passed after the
Task catalog registration repair. Original failures and affected repairs remain
in SQLite. Remaining compiler integration evidence:
`tsrust-s13-compiler-remaining.log` (319 targets, 2637 passed, two ignored).
Final affected checks: `tsrust-s13-final-generic-checks.log`,
`tsrust-s13-final-editor-generics.log`, `tsrust-s13-final-callback-checks.log`.
These focused repairs do not restart the completed full-family regression.

Final release source checkpoint: `fe90dc36`. Release examples and the borrowed
record/view fixture execute successfully. Installed CLI SHA-256:
`62a57e2273f7eeec13b6788d242b87b0bd33a7bd443668f094509b941f70ace5`.
Installed global and configured editor server SHA-256:
`7020c26136d8012f45ad0f9e88902879e58e0370404683d18c8ad46596093463`.
The primary VS Code language server was restarted and reports no problems.

Final matched measurement: 93 samples per engine; median batch 0.009208 ms native
and 0.130958 ms Node after tsc. All checksums are 16777216. SQLite records:
`benchmarks/async/lazy-await-2026-10-15T13-25-32.690Z-verify.json` and
`benchmarks/async/lazy-await-2026-10-15T13-25-46.524Z.json`. Source hashes, compiler
hash, versions and the unchanged protocol are retained in each record. The tree
dirty flag reflects pending handoff documentation and the loose verification
record; the compiler source checkpoint is recorded above.

Final release/install evidence: `tsrust-s13-final-release.log`,
`tsrust-s13-installed-final.log`, `tsrust-s13-final-example-main.log`,
`tsrust-s13-final-example-borrow.log`, `tsrust-s13-final-benchmark-verify.log` and
`tsrust-s13-final-benchmark-timed.log`. Whole-project formatting preserves tokens
and the repeat changes zero files/fingerprints. Closed records were imported,
read back byte-for-byte and removed; SQLite quick_check passes. Retired test
executables were pruned (13.8 GiB); verified executable consumers remain.
