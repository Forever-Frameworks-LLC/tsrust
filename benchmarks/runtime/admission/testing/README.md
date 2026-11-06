# Everyday assertion admission

No new crate, serializer or test framework is required. Reuse the current
compiler-owned std:testing runner, B06 task scopes and existing console JSON
quoting. The selected implementation is borrowed typed structural traversal with
an explicit work stack for recursive records; this is a stack-safety/ownership
requirement, not an unmeasured speed claim. Map keys and Set membership keep
existing key semantics. Transparent declared native records may compare fields;
opaque or borrowed native owners and nested callables cannot be expanded.

The two small matched admission workloads are registered in the existing runtime
manifest: testing_deep_data and testing_deep_collections. Each has real public
Tsrust assertions, matched Node strict assertions, a typed direct-Rust control,
independent BigInt checksum oracle, source guards and current hashes. They measure
successful assertion-heavy pipelines, including setup and checksum, rather than
a whole-framework score. The std HashMap/HashSet control is a backend/control
comparison, not proof that Tsrust's collection representation is identical.
No optimization, default policy or public performance claim depends on unrun data.

Once leased, first build coherent compiler/LSP and run testing_everyday,
testing_runtime, testing_async and existing testing_library plus editor
 testing_everyday. Preserve unrelated accepted tests. Verify new catalog examples,
API reference generation, documentation snippets, formatted-source execution and
relocated offline export. Then run both registered workloads with the existing
untimed five-case native/Node correctness gate and direct-Rust control checks;
only afterward use the original 3process/8warmup/31sample timing protocol and
separate profile-native allocation driver. The existing runner machinery is the
orchestrator; do not introduce another benchmark framework or dependency graph.

Native witnesses include exact64-bit integers, NaN/signed-zero nested values,
record construction order, tuples/arrays/data unions, Map/Set insertion order,
10000-level owned trees with stack-safe comparison, opaque/callable negatives,
transparent regex snapshots and checked RegexError/record/union rejection,
resource destruction, literal filtering, NDJSON escaping and source locations,
async deadline teardown and a started child that owns a Channel. The timeout
witness requires the retained Sender to observe a closed receiver in afterEach,
proving actual child-owner drop before teardown rather than merely observing an
expired timer. Blocking callbacks still need the outer process watchdog.

No build, test, profile, formatter or checkpoint acceptance is recorded here.
