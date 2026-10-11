# Handoff: node resumption primitives in progress

This is an implementation checkpoint, not acceptance of resumable callable summaries. Last accepted production extraction is 3a263b5, published in 5099e0f. Fifteen owner fixture changes remain protected and uncommitted.

BodyState now has validated, duplicate-suppressed node requeueing. Dispatch supplies balanced begin/end node hooks around each partition transfer. Tests independently verify delayed normal completion and delayed failure through finally cleanup, with no replay of earlier calls or repeated cleanup. All 97 flow controls pass when the existing general nested budget is excluded. Its red state is intentional. Default dispatch hooks are no-ops; inter-call Engine wiring does not exist yet.

Read docs/plans/resumable-callable-summaries-plan.md and follow gate 3. Retain body drivers in Engine entries, record parent-request/node dependencies and resume from saved pre-node states. Parameter entry/default/rest handling and capture allocation versions need explicit invalidation; do not cache solely by request identity. Preserve recursive fixpoints, opacity, failure and finally completion. Keep the new normal/failure resumption controls and one-shot reference.

Before declaring source acceptance, finish shared-flow depth canaries, affected native contracts and checked editor checks, then refresh consumers. Current installed consumers remain the accepted 3a263b5 extraction. Exact primitive red/green/control logs are imported and read back in SQLite. No throughput or full-regression claim is made.

Read AGENTS.md and docs/development-workflow.md first; use explicit Git paths, pinned Node/Python, one Cargo target and CARGO_INCREMENTAL=0. Parallel std:path.relative publication still requires exclusive ownership and must preserve the intentional scheduling witness. Give a physical handoff at the next checkpoint.
