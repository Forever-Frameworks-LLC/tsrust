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

Focused acceptance: 18 compiler tests pass, including the generated suspension /
drop witness and later-argument failure responsibility. Three async LSP tests,
the completion matrix, checker agreement and editor consistency pass. Compiler
library regression: 1224 passed, five ignored. Callable / exception integration:
172 tests passed across the 14 existing affected targets. Extension compilation,
executable discovery tests and real TextMate tokenization pass.

SQLite records include `tsrust-s13-acceptance-final.log`,
`tsrust-s13-family.log`, `tsrust-s13-family-repair.log`,
`tsrust-s13-editor-2.log`, `tsrust-s13-editor-4.log`,
`tsrust-s13-editor-final.log`, `tsrust-s13-extension.log` and
`tsrust-s13-examples.log`. Failed attempts are preserved alongside repairs.

Initial matched measurement (before the source checkpoint): 93 samples per engine,
median batch 0.009458 ms native and 0.137 ms Node after tsc. Both produce 16777216
in every sample. Records:
`benchmarks/async/lazy-await-2026-10-15T12-01-23.129Z-verify.json` and
`benchmarks/async/lazy-await-2026-10-15T12-03-08.985Z.json`.
This workload has no comparable prior async measurement; it is not evidence about
Tokio, multicore throughput or suspended I/O latency.
