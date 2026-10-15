# Identity-enforcement feasibility

The candidate enforces the adopted identity contract through the supported source
API. Full lifetime feasibility remains blocked by B2: exact resource destruction
count/order is unobserved. B3 immediate retirement is unchanged; B4 remains repaired.
The complete acceptance list is [I01–I20](identity-matrix.json), with every H/R row
reconciled in [resource-matrix.json](resource-matrix.json).

## Mechanism and construction authority

`runtime()` is the sole runtime constructor. A private program-owned module scalar
issues its monotonically increasing runtime ID. The returned owned allocator closes
over a private `Domain` containing that ID and the next owner counter. Calling the
allocator issues an Owner; raw `create` and `Domain` are no longer exports. A single
canonical loaded module owns this scalar for the generated program. Import aliases
and a diamond import graph share it. Teardown never resets it. No object registry,
clock, random number, address, native shim, or SQLite allocator is used.

The runtime allocator owns only two integers. It does not keep a module-storage
borrow, owner state, or registrations. Owner contains a runtime ID and local owner ID;
Handle contains runtime, owner and instance IDs. Root instance zero carries the same
runtime/owner identity. Handle cloning is supported; allocator cloning and Owner
cloning reject. Moving an allocator is supported and consumes its former binding.
Source ownership diagnostics and emitted scalar fields are checked, not inferred
from matching console output. B2 remains the boundary on physical destruction claims.

Runtime, owner and instance IDs start at 1, reserve 0, and stop before 2147483647.
Thus 2147483646 is the last valid identity. Exhaustion throws before incrementing or
changing membership, state, queue or registrations. IDs are never recycled. The
normal-ceiling instance seam remains in `cycles.tsr`; the private runtime/owner limits
use the exact same algorithm with all three ceilings reduced to 3 in a disposable
test module. Only ceiling literals change. Native assertions cover last valid 2,
repeated failure, independent allocators and unchanged live state; source inspection
checks each throwing guard precedes its counter update. No test-only constructor or
counter-reset API is exported. The demo runs this reduced-limit witness explicitly.

## Validation boundary

Supported construction is `runtime()`, invoking its returned allocator, `root()` and
`mount()`. Operations are `valid`, `read`, `add`, `enqueue`, `remove`, `drain`, `values`
and `reorder`. All handle/event construction in owner.tsr is audited:

- `root` and `mount` stamp runtime and owner identity.
- `index` rejects identity mismatch before state lookup; read/add/enqueue use it.
- `valid` checks runtime/owner even for root 0; mount/values/reorder use valid.
- `remove` rejects identity mismatch before stale/idempotent success.
- Enqueue stores all three identity fields; drain reconstructs all three.
- Values and reorder reconstruct handles with the current runtime/owner IDs.

The fixtures check wrong-owner and cross-runtime operations in both directions,
with identical local counters and application keys. Foreign access cannot invoke
callbacks or mutate state; mount throws before changes. Root/parent operations,
stale root removal, local FIFO, stale queued work, same-key replacement, named and
factory callbacks, and repeated teardown/recreation all retain native assertions.

Records are structural, not security capabilities. Deliberately fabricating an Owner
or Handle, mutating counters/identity/queue internals, or copying individual internal
fields is outside the accepted threat model. No claim of opaque/unforgeable values
is made. Ordinary documented constructors, allocator moves, handle clones and
canonical imports are covered. Separate source copies of this library in one program
are separate identity domains and are not supported interoperability. Handles are
not serialized, persisted or sent across processes/threads. The module scalar adds
no thread-safety permission. A retained owned callback cannot directly capture
module storage merely to construct another runtime; existing borrowed-capture rules
still apply. Normal functions may construct independent runtimes.

## Compiler dependency and history

The primary candidate hit an existing-language compiler defect: exported functions
returning owned closures resolved module captures too early. The compiler worker
confirmed the narrow repair and released the shared target. The repair is commit
`66dd80d87c0af9a009981c3ed72a6d310a2463f0`, separately merged into main as
`d6e747f109aacfbc476869967db2213f26f372c5`. See
[dependency evidence](IDENTITY-DEPENDENCY.md). Installed consumers were not refreshed.
No new identity primitive was required; no second representation was attempted.

Original collision and B4 sources/module are read from Git baseline
`b255009e366357858ab964e486353dd2e5ff5bb9` into disposable directories, formatted and
executed with the current compiler. Original aliasing remains historical failure
evidence. Current named/factory witnesses use runtime() and keep their assertions.
The compiler worker explicitly approved the necessary setup/import migration in
`crates/tsrust-compiler/tests/named_handler_specialization.rs`; its declaration-order
coverage and assertions remain intact. No parallel owner implementation or source
snapshot is retained. Historical compiler pins remain nested in provenance.json.

## Isolation and handoff

The UI candidate is in managed worktree
`/Users/seancotter/.codex/worktrees/ui-identity-feasibility/tsrust`, branch
`codex/ui-identity-feasibility`, based on b255009e. Its owned surface is this experiment
plus the explicitly approved compiler regression caller adjustment. Only the
coordinated dependency was merged; the UI candidate awaits review.

Run the two commands in README.md with the exact pinned compiler from provenance.json.
The suite must pass without skips. Demo exit 2 means the B2 observation gap remains;
usage and unexpected tool/assertion failures exit 1. The full matrix distinguishes
identity enforcement from overall lifetime acceptance.

Closed records live in ignored task-local `evidence.sqlite`, through the repository's
storage tool, with integrity/hash and byte-exact restoration checks before loose
records are removed. Dependency records use `identity-dependency`; UI records use
`identity-enforcement`. No main SQLite writes occur. All five owner plan files and
ongoing SQLite/compiler work are preserved. Shared target use is reserved explicitly,
with incremental compilation disabled and one build job for the approved repair.
Disposable native outputs are removed; installed/release consumers are untouched.

No resource observer, runtime rendering, JSX, route execution, production CLI, DI,
networking, benchmark workload or follow-on milestone is implemented. No full
compiler/LSP milestone or performance measurement is claimed.

## Final checks

- Complete focused command: 13 tests passed, 0 failed/skipped; exit 0.
- Native demo: exit 2 as required for B2; identity, exhaustion and both self-replacement traces passed.
- Migrated compiler caller: 2 tests passed; editor integration: 1 test passed.
- Whole-project formatting: 1,341 sources preserve tokens; second pass is byte-identical.
- Dependency repair before merge: 51 compiler tests and 1 editor test passed.

The initial focused run had 11 passes and one emitted-field inspection failure: the
test expected i32 instead of the compiler's i64 representation. Native assertions
already passed. The assertion was corrected; the complete final suite above passed.
Original failed output remains in SQLite. No timeout was treated as acceptance.
