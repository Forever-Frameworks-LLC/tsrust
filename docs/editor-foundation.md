# VS Code editor foundation implementation plan

Status: E1–E4 implemented; E5 source assistance added in version 0.2.0 on October 2, 2026. Compiler handoffs C1–C5 remain deferred. See [acceptance evidence](./editor-foundation-acceptance.md).

## Outcome and bounded scope

Deliver a locally installable VS Code extension for `.tsr` with syntax highlighting and live compiler diagnostics for unsaved single-file source. The language server is a Rust application calling the compiler library in process. Checking must not emit Rust, invoke rustc, or require a native backend installation.

The first release provides language recognition, highlighting, bracket/comment configuration, document synchronization, syntax/type/ownership diagnostics, related error locations, stale-result suppression, restart support, and a documented local installation workflow. It does not promise autocomplete, hover, rename, go-to-definition, semantic tokens, formatting, debugging, modules, incremental analysis, or multiple simultaneous errors. These were the first-release deferrals. The user subsequently authorized expanding independent editor capabilities; E5 below supersedes the deferral of source-assisted hover, completion and navigation. Compiler-proven semantic metadata remains separately gated.

## Verified starting contract

`crates/tsrust-compiler/src/lib.rs` exports `check(source: &str) -> Result<Analysis, Diagnostic>`. It performs parsing, resolution, type checking and ownership analysis without file I/O or subprocesses. Analysis currently exposes function contracts keyed by name and closure metadata; it does not expose a general symbol/navigation index.

`Diagnostic` exposes `code`, `message`, primary `span`, and `related: Vec<(Span, String)>`. `Span` is a half-open UTF-8 byte range in the exact input source. All related ranges currently belong to that same source. The public `line_column` helper counts Unicode scalar characters and uses one-based positions; it must not be reused as an LSP UTF-16 conversion.

Checking is synchronous and returns one error. Recovery, diagnostic aggregation, cooperative cancellation, and incremental invalidation are not present. The first release can use this API unchanged. No compiler feature gate has to finish before editor implementation starts.

## Proposed repository boundaries

- `crates/tsrust-lsp/`: server binary, protocol transport, document store, position mapping, scheduler, compiler adapter, protocol integration tests. Add it as a workspace member.
- `editors/vscode/`: extension manifest, TypeScript activation code, language configuration, TextMate grammar, extension tests, packaging scripts, local-install README.
- `docs/editor-foundation.md`: this plan and eventual acceptance references.
- `docs/toolchain-and-development.md`: user workflow and compiler/editor capability limits.

The server depends on the compiler library. The compiler must not depend on LSP or VS Code packages. Compiler changes stay in their own scoped gates. Initial distribution is a local VSIX plus an explicitly configured locally built server binary; downloadable binaries, Marketplace publication, installers, and cross-platform release automation are later tasks.

Choose maintained protocol/JSON libraries during implementation and record pinned versions and licensing. Preserve the compiler crate's dependency-free status. New server dependencies require updating workspace setup documentation: an initial dependency fetch precedes offline builds. Use a VS Code language client rather than inventing an editor protocol.

## E1: extension and highlighting

Register language ID `tsrust` and extension `.tsr`. Supply comments, brackets, auto-closing pairs, and a TextMate grammar derived from implemented lexer syntax. Cover comments, escaped strings, numeric forms, declarations, control flow, ownership annotations, operators, and punctuation. Highlighting is lexical and must not claim that a program is valid.

Provide `tsrust.server.path`, a configurable analysis debounce (default 200 ms), an output channel, and a restart-server command. Launch the configured binary directly with arguments through the language client, using stdio. Report a missing binary with actionable setup instructions. Extension activation occurs for Tsrust documents. Disposal stops the client and its process.

Acceptance: load an Extension Development Host, open representative current fixtures, verify language recognition, highlighting and comment/bracket behavior, then produce a VSIX and verify local installation. A missing server must leave lexical editing usable.

Compiler wait: none. New syntax requires a grammar fixture/update as part of the relevant compiler gate.

## E2: protocol, documents, and source coordinates

Implement initialize/initialized, shutdown/exit, didOpen/didChange/didClose, and publishDiagnostics over correctly framed stdio messages. Stdout contains protocol messages only; logs go to stderr. Advertise only implemented capabilities and explicitly use UTF-16 positions. Begin with full-document synchronization to avoid implementing incremental edit application prematurely.

Keep URI, monotonically accepted document version, exact text, and line index per open document. Analyze received buffers rather than rereading disk. Separate documents remain independent single-file programs, even when open in the same folder. Reject or ignore outdated changes consistently and log protocol problems without corrupting state.

Map byte offsets to zero-based line/UTF-16 character positions using the exact analyzed snapshot. Cover ASCII, non-ASCII BMP characters, emoji/surrogate pairs, combining characters, LF, CRLF, multiline spans, empty spans and EOF. Validate bounds and character boundaries; an invalid compiler span is an internal error and must not produce a misleading squiggle or crash the transport. Preserve legitimate zero-width EOF ranges.

Acceptance: protocol tests drive initialization, two documents, replacements, close/reopen, Unicode ranges and clean shutdown. Closing a document clears diagnostics and invalidates pending results.

Compiler wait: none; preserve the existing source-span contract.

## E3: compiler adapter and responsive scheduling

Call `check` with an immutable source snapshot. Translate failure into an error diagnostic with source `tsrust`, original code/message, primary range and same-document relatedInformation. Success publishes an empty diagnostic list. Include the analyzed document version in publication. Do not parse human-readable CLI output or generate Rust.

Run checking off the protocol loop. Start with one analysis worker, one latest pending snapshot per document, and fair scheduling between documents. Debounce changes; opening a document schedules an immediate check. Queue growth is bounded by open documents, not keystrokes. Existing synchronous analysis cannot be interrupted: superseding an edit invalidates its result and replaces pending work but does not stop a running check. This distinction must be documented.

Publish only when both document generation and version still match. Generation changes on close/reopen, including reopening at the same version. The protocol loop must still accept edits, close and shutdown while checking. Worker failures must be visible in logs/client status; never represent an internal failure as successful validation. Define restart behavior without an infinite automatic restart loop. No forced worker cancellation through unsafe thread termination.

Acceptance: inject a deliberately slow checker and verify rapid edits, edits in another file, close/reopen and shutdown; no obsolete diagnostic is published and pending work stays bounded. Use real compiler fixtures for syntax, type, use-after-move, conflicting access and related transfer locations. Compare adapter code/message/spans against direct `check` on identical buffers.

Compiler wait: none for ordinary bounded files. Long running checks remain a known limitation until C3; report actual measurements rather than claiming cancellation or editor-scale latency.

## E4: usable local release and end-to-end acceptance

Build the server using the pinned Rust toolchain and package the extension. Document exact commands for building, setting the absolute server path, installing the VSIX, restarting, inspecting logs and uninstalling. Test in the user's local macOS VS Code; Linux/Windows support is unverified until separately tested. Do not publish externally as part of this gate.

Demonstration: open a valid `.tsr` fixture, introduce an ownership error without saving, see the invalid-use squiggle and related transfer location in Problems, repair it and observe errors clear. Repeat for syntax/type errors, Unicode before an error, two open files, rapid typing, and close/reopen. Verify neither Rust emission nor native compilation occurs.

Measure edit-to-diagnostic latency on representative current fixtures and larger supported files, recording source size, toolchain, debounce, analysis time and machine. Report incomplete-source behavior and the one-error limitation explicitly. No fixed performance promise is required before measurement.

Verification: focused Rust protocol/scheduler/adapter tests, extension type checking and packaging, representative grammar cases, and the manual Extension Development Host/installed-VSIX exercise. Apply normal workspace format/Clippy checks to new Rust code. Run compiler regressions only when compiler code changes or workspace integration requires the repository gate; do not rerun runtime benchmark tables for editor plumbing.

## Compiler handoff and wait matrix

IDs below are coordination identifiers, not replacements for numbered language milestones. E1–E4 can complete independently using today's API. C1–C5 are explicit future compiler tasks; they are not prerequisites for claiming the bounded first release complete.

| Compiler task | Required contract/deliverable | Editor capability waiting on it | Acceptance evidence |
| --- | --- | --- | --- |
| C0: preserve analysis boundary (ongoing) | In-process, I/O-free `check`; half-open UTF-8 ranges in input; diagnostic codes/messages/related spans; source errors returned rather than backend-dependent checks | Existing diagnostics; changes require coordinated adapter update | Current API tests plus representative editor adapter fixtures remain valid |
| C1: incomplete-source recovery | A separate editor-oriented analysis entry point can recover at defined statement/declaration boundaries and retain usable independent regions; invalid regions cannot be represented as fully checked code | Useful diagnostics while typing past an unfinished construct | Missing brace, incomplete expression and broken declaration fixtures; later independent declarations still analyzed; bounded progress/no loops; strict `check` behavior preserved |
| C2: diagnostic aggregation | Structured analysis report with deterministic ordered diagnostics and explicit complete/partial status; bounded diagnostic count and suppressed cascades; severity if non-errors are introduced | Multiple Problems entries; richer recovery feedback | Two independent errors both reported, stable order, cascade policy, Unicode ranges; editor diagnostics agree with strict semantics in valid regions |
| C3: cooperative cancellation | Editor analysis API accepts a cancellation mechanism and checks it at bounded intervals including recursive inference/specialization; cancellation is a distinct outcome, not a source diagnostic | Interrupting obsolete checks and bounding shutdown/work latency | Cancellation during representative expensive passes; no partial success published; responsiveness measured; uncancelled results equal existing checks |
| C4: source-indexed semantic metadata | Public symbols/types/contracts with declaration/reference and parameter ranges, source-scoped identity and an explicit validity policy for partial analysis; no exposure of private AST internals | Contract/type hover first, then navigation/completion in separate scopes | Shadowing, nested scopes, callable parameters, captures and ownership modes resolve to correct source symbols; incomplete-source policy tested |
| C5: project/module analysis and invalidation | Defined module resolution, source/file identity, cross-file related spans, dependency graph and changed-file invalidation contract | Imports, project diagnostics and cross-file navigation | Unsaved dependency changes update dependents; unrelated files avoid invalidation; file rename/removal and versioned results tested |

C1 and C2 should be designed together, but can ship as separate compiler gates. C3 can precede recovery. C4 depends on the scoped navigation/hover requirements; full completion is not automatically unlocked by adding symbols. C5 waits for module semantics and is not a reason to delay single-file tooling. Incremental single-file analysis is a separately measured optimization, not an E1–E4 prerequisite.

Suggested progression: finish the active language gate; implement E1–E4 alongside subsequent compiler gates; schedule C1/C2 when first-release editing demonstrates the actual recovery pain; schedule C3 when measured obsolete-check latency warrants it; scope hover using C4; defer C5 until modules are defined.

## Obligations for subsequent compiler gates

For each new syntax/semantic gate, record editor impact in its milestone: grammar changes, diagnostic/range changes, new related locations, incomplete-source behavior, expensive analysis paths, and metadata changes. Update relevant editor fixtures when they exist. No editor dependency is implied by a new language feature unless its bounded compiler acceptance requires one.

Changes to public analysis/diagnostic contracts must state the adapter migration and be tested against the editor crate before landing. New error paths must retain valid source ranges including Unicode/EOF and remain frontend-only. Never mark recovery, multiple diagnostics, cancellation, hover or project support complete solely because the strict compiler accepts new language constructs.

When a compiler handoff lands, record its exact public API, acceptance evidence and consuming editor task here. The editor task must remain deferred until that contract is implemented and verified; it must not infer contracts from generated Rust or private compiler state.

## Completion checklist

E1–E4 are complete when the local installed extension demonstrates live unsaved compiler diagnostics, Unicode-correct locations, related information, clearing on repair/close, stale-result suppression, bounded pending work, clean lifecycle, reproducible setup and documented first-error/incomplete-source limitations. C1–C5 remain individually visible compiler backlog items. E1–E4 implementation and verification are recorded in [acceptance evidence](./editor-foundation-acceptance.md).

## E5: source assistance without a new compiler API

Implemented in `crates/tsrust-lsp/src/intelligence.rs`: tolerant UTF-8 token/scoped declaration indexing, source-declared function and callable signatures, adjacent block/line documentation, parameter help, visible-symbol completion, annotated record-field completion, supported built-in catalog, same-file definitions/references/highlights, outline and folding. Lexical comments/strings cannot introduce false declarations or references. Loop variables remain inside their loop scope; straightforward source types support receiver suggestions without guessing compound-expression types. Unknown types remain unknown.

The extension adds a standard light/dark language file icon and documented function/type/control-flow snippets. Its installed VSIX integration exercises actual provider requests on unsaved and incomplete source. Every catalog entry has a witness accepted by the current strict compiler; the catalog is not copied wholesale from TypeScript's standard library.

C4 now waits for **compiler-proven** symbol identity, inferred types, ownership contracts and complex callable/member resolution, rather than blocking declared-source hover or lexical navigation. C1/C2/C3/C5 remain unchanged. Semantic rename and cross-file navigation must not use the tolerant index as a soundness claim.

`editors/vscode/scripts/check-compiler.py` fingerprints compiler and editor source plus coverage, validates current LSP contracts on change, and lists newly supported catalog candidates. Its cached successful state is a build artifact. Hourly read-only monitoring is active with explicit user approval under the automation `check-tsrust-compiler-and-editor-compatibility`. It inspects current milestones/API each hour, runs focused compatibility checks after changes, stays quiet on unchanged passing state, and reports regressions or newly available compiler capabilities. It does not modify source or reinstall extensions.

## JavaScript value and namespace progression

[The implementation plan](javascript-parity-plan.md) is the compiler/editor dependency contract for null/undefined, optional unions, dynamic values and standard namespaces. Stage 1 and Array.isArray, Boolean and String conversion are accepted in [the value milestone](milestones/value-types-and-namespaces.md). Their catalog witnesses, detailed examples and scoped inference are implemented. Array constructor completion stays separate from array instance methods. Union narrowing, optional results, identity and property metadata must land before the corresponding JSON/Object/optional-array capabilities are advertised. Keep runtime measurements pending until the scheduled checkpoint.

## Compiler-backed quick fixes

[Quick-fix implementation and future handoffs](editor-quick-fixes.md) document the public SuggestedFix diagnostic contract and versioned code actions. Missing punctuation candidates are offered only when the edited current source passes strict checking. Semantic spelling, argument and ownership repairs remain separate compiler tasks. This does not supply incomplete-source recovery or diagnostic aggregation.

Compiler handoff: `Analysis::generic_records()` returns `GenericRecordInfo` declaration provenance (concrete nominal name, template name/span and structured concrete argument annotations) in deterministic record order. Nested resolved names refer to another concrete record entry and must not be reparsed as source syntax. This additive API does not provide project-aware expression types or generic function inference. Its structured provenance witness, all 39 project contracts and 64 affected editor checks pass; details are recorded in the S06 milestone. Existing source editor inference continues through its current adapter.

Compiler handoff: CheckedTypeFacts now includes generic_calls: Vec<GenericCallInfo>. Each successfully checked generic call retains UTF-8 call/callee/original-declaration spans, ordered GenericParameterInfo {name,ty:Option<String>,access:Mode} and a concrete result spelling. Conflicting specializations at a shared source span remain separate; adapters must suppress ambiguous concrete overrides. Unknown callable display types stay None. Standalone hover, implicit/explicit parameter help and data result chains consume unique facts; source documentation and callee aliases remain source-index responsibilities. This does not supply project-aware generic facts or incomplete-source recovery. Focused evidence is recorded in the S06 milestone.

Project handoff: ProjectAnalysis::generic_calls() now returns original source-aware locations and structured ProjectType nominal/array/union data. Nominal declaration identity includes source and range, so imported aliases can resolve without reading generated names. The editor's successful project-analysis branch consumes these facts through ProjectIndex::checked; header-only/invalid projects retain declared-source assistance. Focused acceptance includes direct aliases, generic record result chains, dependency definitions/references, Unicode ranges and unsaved docs/result updates. Private/transitive identity and callable metadata coverage remain explicit follow-up requirements in S06.

Callable parameter handoff: ProjectType::Callable retains CallableOwnership, ordered ProjectParameterInfo children and a boxed result. Successfully checked generic callback parameter facts preserve declared names/access and source-aware nominal children; the editor resolves local aliases and updates names from unsaved dependencies. Standalone owned parameter facts retain the declaration contract through concrete callback specialization. Abstract callable shapes and callable-result display/storage remain separate requirements. Focused evidence is recorded in generic-callable-inference-integration.md.
