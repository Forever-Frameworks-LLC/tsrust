# Native view lexical scope checkpoint

Continue from the receiver-contract checkpoint at `1a3dfdaa`. Read AGENTS.md,
docs/development-workflow.md and docs/native-byte-views-implementation.md first.
Gate 189 and S07 remain open. Public borrowed signatures remain guarded.

The retained-borrow scope hook now rejects an outer native holder that retains
storage owned by a binding in the block ending here. Replacing the view before
exit and keeping both owner and holder local remain valid. The diagnostic locates
the owner using its declaration, initializer or source-binding identity.

Shadowed holders remain physically alive. An optional native-only identity stack
keeps their loans active until the shadowing block ends. Dead local holders still
release access. This corrects earlier implementation notes that excluded shadowed
holders. Programs without native borrowed types allocate no shadow identity stack.

Four focused tests use parsed, formatted source blocks with an injected borrowed
factory interface. They cover escape, replacement/local holders, shadowing conflicts
and dead-holder release. They do not compile complete public factory bodies.

Verification: 57 private native-view tests, 107 source tests (Result and ten normal
stack depth canaries), nine registered signature tests and 16 native editor tests.
All passed, with no aborted targets. CLI build was warning-free. Whole-project
formatting changed zero files and zero fingerprints. Original red and repair logs
are retained with the verification record in SQLite under
`runtime/results/native-view-lexical-scopes-2026-10-10.json`.
No timing or full regression was run.

Next, cover exceptional exits, break/continue and completion snapshots. Validate
unused consuming closures before emission. Finish real factory-generated source
bodies, escaped/rebound environments, mixed callback records and specialized
callback/storage families against the implementation-plan matrix. Public factories
must remain guarded until these paths agree. Installed consumers still reflect
f569190a; their refresh, including the earlier public ReadOnly capture repair,
remains pending. Full regressions and timing belong to S07 completion.

## Continuation prompt

Continue gate 189 from this handoff. Prove completion-path lifetime enforcement with
focused failing source witnesses, then finish unused consuming closure validation
and complete factory-body acceptance. Preserve real slices and explicit ownership.
Keep public borrowed signatures guarded until the entire required matrix agrees.
Use bounded compiler/native/editor checks, project formatting, SQLite exact evidence
and explicit-path commits. Keep S07 open and refresh executable consumers before
checking them.
