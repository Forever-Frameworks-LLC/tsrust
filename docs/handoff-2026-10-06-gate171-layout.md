# Handoff: finite named capture layouts

Continue Tsrust at `/Users/seancotter/Desktop/dev/learning/tsrust`. Read `AGENTS.md`,
`docs/development-workflow.md`, `docs/plans/nested-function-declarations-plan.md`,
`docs/milestones/171-nested-function-captures.md` and this file before edits. Inspect
Git status and log; one agent works in this tree at a time. The user requested this
handoff at the next checkpoint. The overall plan remains active and incomplete.

## Scope and decisions

Continue the full plan in isolated, testable chunks, maintaining compiler/editor
parity, efficiency and ease of use. Nested declarations are the current dependency;
do not redefine completion around capture-free functions or metadata alone. Gate170
is accepted. Gate171 borrowed captures is incomplete; gate172 covers escaping owned
callables and gate173 covers full signatures, generics and result inference. The
family milestone follows full scoped acceptance, not a gate-count schedule. Broader
plans remain in `docs/standard-library-roadmap.md`, `docs/plans/javascript-parity-plan.md`
and `docs/editor-foundation.md`.

The owner selected existing ownership rules: borrow local captured owners, transfer
supported owned escaping captures later, preserve scalar copy semantics. Do not add
shared JavaScript capture cells, implicit clones or shorter lexical loan lifetimes.
Hoisting names does not initialize captures. Unused declarations must not form loans.
Later initialized captures require readiness at every use. Preserve source identity,
defaults, rest arity, recursion, module rebasing and fresh block/loop activations.

## Worktree and checkpoints

Before this checkpoint, HEAD is `4ca774f` (recursive capture access), following
`f79ffbb` (typed exceptional readiness) and `8cc9782` (typed named bodies and scope
repairs). The next commit contains the finite layout module and this handoff; use
`git log` to obtain its actual hash.

Four unrelated fixture edits must remain uncommitted, untouched and unstaged:

- `crates/tsrust-compiler/tests/fixtures/comma-contexts/guards.tsr`
- `crates/tsrust-compiler/tests/fixtures/interfaces/shapes.tsr`
- `crates/tsrust-compiler/tests/fixtures/map-for-each/values.tsr`
- `crates/tsrust-compiler/tests/fixtures/map-group-by/documented.tsr`

Use explicit Git paths. Never `git add -A crates` or `git commit -a`. No archives,
source snapshots or binary backups. Git provides source checkpoints; SQLite stores
exact closed evidence. Hourly checks remain paused at the owner's request.

## Current compiler architecture

`local_functions.rs` resolves block names and lifts declarations, retaining logical
owners, declaration anchors and source/emitted names. `local_capture_plan.rs` owns
BindingId (logical owner plus declaration start), variable-use identities, direct
captures, dependency edges and iterative SCC groups. Required identities propagate
once across condensed groups; module linking rebases and qualifies retained data.

`capture_context.rs` projects typed defining bindings without runtime loans.
Context::source_bindings maps declaration starts to typed slots independently of
visible names. `named_capture_context.rs` prepares direct-call contexts, places
hidden capture slots after reserved source parameters and exposes only direct
capture names. Transitive slots stay source-addressable without overriding parameters.
Actual source defaults and body diagnostics are checked. Source parameter_count
remains the supplied argument prefix.

`local_readiness.rs` performs the source precheck and a final pass over actual checked
function, closure and method invocation bodies after typed call effects and resolved
control destinations. Physical declarations and source parameter metadata distinguish
defining values from hidden projected slots. Failure before assignment or in an RHS
leaves catch-path captures unassigned; catch/finally assignment establishes readiness.
Diagnostics mark the named use and related captured declaration. Source closure body
copying remains in the precheck only, not the final typed pass.

`named_capture_access.rs` owns separate finite coordinates keyed by function index
and BindingId. Static named edges propagate capture access into caller capture or
parameter slots. The recursive inference driver joins these separately from source
argument modes and extends its finite budget when coordinates are discovered.
Summaries retain typed slots privately in CheckedProgram. Forwarder mutation and
mutual-recursion convergence are tested; no loan or transfer is formed by this phase.

New `named_capture_layout.rs` builds deterministic per-SCC typed fields and member
slot mappings from settled summaries. It deduplicates external binding identities,
checks type agreement and joins read/mutate/consume access. Read scalars use Copy
storage; mutated scalars use Mutate storage. Recursive peers are static members,
not fields containing each other. Captures owned by a member invocation use explicit
Invocation sources, so recursive parameters are not incorrectly hoisted. Layouts
are retained privately after final typed readiness.

Both public analysis paths use finish_analysis. The pending environment barrier still
returns unsupported after readiness/layout derivation. Missing defining contexts and
captured callable values have individual guards. No incomplete capture tree reaches
emission. Exception emitter validation is deferred only for capture programs that
remain blocked; ordinary programs retain validation. Do not remove guards prematurely.

## Next implementation

1. Form native borrowed environments only at valid, initialized use points. Use
   declaration scope/owner anchors and defining source slots; do not eagerly lower
   hoisting to const-arrow creation. Preserve existing lexical capture loans and
   disposal order. An environment slot declared before its owner can have invalid
   Rust drop order even when initialized later.
2. Implement finite native group dispatch and member-slot aliases. Reuse existing
   callable layout/view contracts. Relevant emitter entry points are
   initialized_function_body, declared_function_call, native_call_name and existing
   closure capture setup. Keep shared recursive dispatch thin with dedicated helpers.
3. Forward group fields through calls with correct reborrowing. Cross-group forwarding
   must not create duplicate retained mutable references. Invocation sources need
   their defining activation, not a field copied into shared storage. Prove mutable
   reentry and conflicts, moved captures, scope release, block/loop freshness and
   exceptions with native witnesses.
4. Make the four existing positive native capture witnesses genuinely pass. Do not
   convert them into accepted unsupported assertions. Then add first-class synchronous
   borrowed use, ownership diagnostics, editor/guide examples and complete matched
   benchmark support. Owned escape remains gate172; full signature/result inference
   remains gate173. Audit the full plan before claiming family completion.

Current red native cases are readonly-later.tsr, mutable-block.tsr,
recursive-forwarding.tsr and fresh-loop.tsr under fixtures/nested-function-captures.
The existing arrow-borrowing witness is the reference contract. Captured field type
errors, initialization/transitive/shadow readiness and typed failure paths pass.

## Execution and evidence

Use `CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`,
`CARGO_INCREMENTAL=0`, offline locked Cargo commands and the pinned Node executable
`/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node`. Shell is zsh; no implicit
word splitting. Library caches remain warm; pruned test executables will relink.

After parser/analyzer dispatch changes, run all eight canaries: closures, for_of,
owned_arrays, nullish_coalescing, nullish_assignment, higher_order_function_values,
number_operators and type_syntax. Run the whole editor crate before acceptance.
Use focused TDD and bounded native checks; no routine full compiler matrix or timing.
Matched workloads need TypeScript counterparts, independent oracles, source guards,
registrations, driver/ABI support and manifest fingerprints as implementation lands.
Retain failed evidence and resume affected/unrun checks rather than replaying passes.

Refresh five consumers atomically by copying to a `.refresh-tmp` sibling and replacing:
target/debug/{tsrust,tsrust-lsp}, target/editor/debug/{tsrust,tsrust-lsp},
target/release/tsrust. Existing server processes retain old inodes; restart the language
server or reload VS Code when needed. The installed type-reference grammar already
matches source and tests; no extension reinstall is needed for compiler-only work.

SQLite is canonical: import original logs/results with benchmarks/storage/store.py,
read them back byte for byte, then remove verified loose copies. Keep only concise
current-gate logs. Prune obsolete native/test/runtime-comparison outputs before handoff,
retaining library caches and the five consumers. Measurements remain unperformed;
untimed verification never counts as timing.

## Final checkpoint evidence

Focused capture library27, all eight canaries63 and whole editor275 pass. The
capture suite has seven passes and four genuinely pending native environment
failures. Five matched lexical cases pass native/Node-after-tsc verification with
seven observations each. No timing was run. Canonical result is
benchmarks/runtime/results/2026-10-06T19-04-52-509Z-verify.json; all
109 compiler source fingerprints match current files. 9 original
logs/results are imported into benchmarks/records.sqlite and read back byte for
byte before loose-copy removal.

Five consumers are atomically refreshed and hash verified. 50 scratch test/native
executables and runtime-comparison output are pruned; library caches remain. No
process is left running from these checks. Only the concise target/editor/gate171.log
remains for this checkpoint. Gate171 stays incomplete, and hourly checks remain paused.
