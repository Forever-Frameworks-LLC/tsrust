# Native view completion witnesses

Continue from `838f694c`, reading AGENTS.md, docs/development-workflow.md and
docs/native-byte-views-implementation.md. Gate 189 and S07 remain open.

Three new private source tests extend lexical lifetime coverage to break, continue,
typed throw/catch and finally replacement. All seven scope tests pass. The helper
uses the actual exception callable checker for exception bodies; a bare Context
does not enable that analysis. Earlier failed runs exposed helper setup errors,
not production regressions. Those original logs remain in SQLite.

The controls prove dangling outer holders are rejected and dead local holders
release loans across the tested exits. Finally replacement permits later owner
mutation, while retaining a view through catch rejects that mutation. No production
code changed in this checkpoint. Complete public factory bodies and arbitrary
completion joins or borrowed throw payloads are not covered by these tests.

Evidence: seven scope tests passed; project formatting changed zero files and
fingerprints. SQLite record:
`runtime/results/native-view-completion-witnesses-2026-10-10.json`.
Prior 189 focused checks remain evidence for the unchanged production source.
No full regressions or timing. Installed consumers remain f569190a.

## Continuation prompt

Continue gate 189. Validate unused consuming closures before emission, then finish
real factory-generated source bodies, escaped/rebound environments, mixed callback
records and specialized callback/storage families against the implementation plan.
Add targeted completion join and borrowed payload controls where these features
require them. Keep public borrowed signatures guarded, real slices and explicit
ownership. Use focused TDD, bounded checks, SQLite evidence and explicit-path
commits. Refresh executable consumers before checking them. Full regressions and
timing belong to S07 completion.
