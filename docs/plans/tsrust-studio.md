# Tsrust Studio

Implementation authorized, 2026-11-10. Full scope below remains required.
Architecture choices and unmeasured acceptance targets remain proposals; source
implementation is not model quality, platform acceptance or release approval.
See [the bounded capability audit](tsrust-studio-audit.md) for exact source
identities, proof contracts, current checks and unresolved prerequisites.

## Owner requirements

- Initial dogfooding workspace: open `apps/studio` itself, not the entire Tsrust
  checkout. Exercise its real source/config/tests through the editor. Broaden the
  open directory later only for a demonstrated workflow; whole-repository
  indexing is not part of the first document/project section.
- Studio lives inside this repository, provisionally at `apps/studio`, and starts
  as an ordinary `tsrust new studio` project using the supported UI setup. Its
  application, worker, agent, evaluation and training orchestration source is
  entirely `.tsr`/`.tsrx`. Use Tsrust's own build, tests, UI and package workflows.
- Native capabilities come from Tsrust and its managed dependencies. Add checked
  Rust-package bindings or standard-library capabilities where necessary. No
  separately installed model server, Python environment, Node application host,
  Ollama, llama.cpp service or MLX command-line setup is required for local AI.
  Compiler-owned native adapters remain part of Tsrust's implementation, like the
  existing GPUI bridge; Studio does not acquire handwritten Rust application code.
- Models, tokenizers, data, themes, configuration and generated build artifacts are
  resources, not a second application implementation language. Use ordinary
  Tsrust project configuration rather than inventing a source-only build format.
- macOS first, then Windows. Linux and Raspberry Pi acceptance are deferred.
- The verified local minimum is this Apple M3 Mac with 8 GiB unified RAM. Complete
  offline assistance is required there, alongside usable editing and development
  tools. The owner's Windows machine has 32 GB system RAM and an RTX 4090; GPU
  memory, drivers and exact configuration must be inventoried on that machine.
- Deliver the agreed full feature set before the signing/packaging milestone.
  Internal development phases are not reduced-scope releases or an early MVP.
- Support Tsrust projects deeply and Rust/HTML/Markdown/JSON and related files as
  first-class editing workflows, including appropriate language services.
- Prioritize local AI: investigate existing models and harnesses, fine-tune a
  Tsrust-specialized model, evaluate it and distribute qualified configurations.
  Later include direct cloud APIs and compatible external coding-agent clients.
- Agents propose diffs. They do not silently modify the user's working documents.
  Background diagnostic fixes must coexist with editing and other agent tasks.
- Compiler, CLI and LSP remain independently usable. Improvements needed by Studio
  should strengthen capabilities available to other editors and applications.
- Study upstream designs and reuse suitable dependencies. Implement application
  behavior in Tsrust; do not assume wholesale embedding of Zed or Code OSS.
- Owner clarification, November 10: model the workspace on established desktop
  editors (VS Code, Visual Studio, Zed, IntelliJ). No nonfunctional UI, including
  demos: no decorative app-name/filename header, pretend tabs or placeholder
  actions. Use actual document tabs and working editor actions. Visual polish
  must serve editing rather than website branding.

Current formatter/workflow section, November 11: native menus/status commands,
global/project rules and save/focus triggers are implemented. Focused formatter3/3,
terminal-link parser1/1, public workflow/model6/6 passed. Native app builds and live
MAT verified a named CLI check terminal plus Cmd-click navigation with destination
highlighting; source-labeled Problems is implemented, LSP is still disconnected.
All8open files/4drafts/zoom-1 survived the final reload. Actual51-file manual CLI
check completion and broader live focus/settings acceptance are still pending.
Exact identities/checks and current supervisor are in docs/agent-progress.md.

## Immediate implementation priority — owner, 2026-11-11

Finish the current formatting/task-terminal/Problems integration, then prioritize
real language services before Git, debugger and extension-management surfaces.
Use a shared configurable LSP connection for Tsrust and external language servers
(including Rust, TypeScript and Go), with diagnostics routed to Problems and
hover, completion and snippets exposed through the existing native editor UI.
Support server lifecycle, document synchronization, stale-version rejection,
Unicode position conversion, cancellation and visible connection/error state.
Language/server registration should permit later language extensions without a
separate integration architecture for each server. Do not claim external server
acceptance until the installed server is exercised.

Next prioritize the built-in AI agent and shared semantic editor actions. Agent
review/diffs and eventual debugger stepping must use the same guarded commands
as the user; a tool catalog without executable integration is not acceptance.
Reuse the existing native completion/hover/diff components where appropriate.
Git, debugger and general extension UI follow these priorities.

## Current native terminal integration — 2026-11-11

The native Terminal, checked command/snapshot API and Studio's first shell are now
implemented. Public guarded-action2/2, real-PTY2/2, UI-node/HTML1/1 and compiler
facade1/1 focused witnesses pass. Native shell input, correct apps/studio cwd,
8x136 grid, ANSI red and Unicode/CJK/emoji were observed. This does not prove full
terminal/editor acceptance or integrated AI transport.

The right session rail now supports independent shells, retained output when
switching and per-session removal. New Terminal and Close share the panel header;
the terminal has no contrasting padded frame. Shared rendering makes editor/panel
tab labels follow configured size/zoom, with one panel-header bottom line. Native
MAT observed distinct shell PIDs, retained output, isolated inactive removal and
shared zoom. A delayed chained reply exposed during Interrupt MAT is fixed by
deferring its redraw until after the current frame; a new identified120-second
sleep was interrupted immediately while its shell remained alive. Session model
and registered guarded adapter witnesses pass2/2; integrated AI transport remains
pending. Rail resize, copy/selection, restart/quit, accessibility and dev-shutdown
process-tree behavior remain pending. Restored header inset and a flush terminal
viewport were observed together; the relocated New Terminal button creates and
focuses another independent shell.
Session rows are compact and divided; the ListItem wrapper now honors row utility
styling. Native MAT observed both rows and their controls without clipping.
Unix dev group termination does not cover a PTY's separate session by itself;
prove and fix HUP-ignoring-job cleanup before completing that lifecycle gate.
Installed public consumers are refreshed from the normal pair containing these
native renderer fixes. No broad regression run was used for this UI section.

## Current terminal prerequisite — 2026-11-11

After search checkpoint0b6f35d5, the managed GPUI graph admits exact
alacritty_terminal0.26.0 (Apache2) and async-channel2.5.0 behind opt-in features.
The native Session source provides real PTY/VT state, input, resize, bounded redraw
notifications and Unix force-close/reaping. Two real-process boundary checks pass
in0.03s execution/1.93s setup using the locked isolated dependency graph; whole
formatter/diff checks pass. Backend dependencies remain unchanged.

This is a prerequisite, not a usable Studio terminal or public API acceptance.
Renderer, themed palette query replies, checked bindings/catalog/docs, shared
guarded UI/AI commands, Studio panel and native MAT remain next. Await cleanup
from GPUI application quit; verify IME, selection, alternate-screen interaction,
foreground/detached jobs and Windows lifecycle before broad acceptance. Existing
search preview remains running; exact evidence and next action are in agent-progress.

## Architecture proposal

### Tsrust application and managed workers

Studio owns its window composition, commands, project state, documents, settings,
agent coordination and review flows in Tsrust. Use the existing GPUI/Kit-backed
UI, editable component wrappers and signal semantics. A web view is an additional
component for previews and embedded content, not an alternative editor host.

Keep inference and expensive project work off the UI thread. First evaluate a
Tsrust-built managed worker executable with native inference bindings. Studio
starts and stops it, negotiates its version and capabilities, and communicates
through typed, cancellable messages using existing process/transport facilities.
The worker is shipped and updated with Tsrust; it is not a user-installed server.
Compare against an in-process background task before fixing the boundary. Choose
from measured cancellation, memory, crash recovery and deployment behavior.

The UI and all workers share document/task identities and protocol definitions.
Avoid introducing an HTTP backend merely to connect processes on one machine.
If a service boundary proves useful, implement its application logic in Tsrust.

### Compiler and language services

Reuse the existing CLI and LSP as the initial source of build/check/test and
language semantics. Add structured, versioned APIs where the current interfaces
cannot serve interactive editing efficiently. Do not duplicate the checker in
Studio or scrape human-readable terminal errors.

Tsrust LSP handles `.tsr`/`.tsrx`; rust-analyzer handles Rust. Inventory the
appropriate reusable services for HTML, CSS, JSON, Markdown and related formats,
including their runtime dependencies. Prefer native or Tsrust implementations
when an existing service would impose a separately installed runtime. External
tooling needed for Rust projects is managed through the existing Tsrust toolchain
workflow. Multiple servers must agree on document ownership and versioning.

### Document, edit and task ownership

One document service owns buffers, revisions, selections, undo/redo and save state.
Define compound edit transactions and atomic multi-file actions before wiring
AI into editing. Test Unicode offsets, IME composition, CRLF, external file changes,
symlinks, rename/delete, unsaved buffers, undo grouping and crash recovery.

Diagnostics, completions and proposals carry source revisions. Cancel obsolete
requests and reject stale responses. LSP and agent edits use the same transaction
boundary as user actions. Accepting an edit must not reset selection or erase
newer typing. A compiler pass alone does not prove behavioral correctness.

One coordinator owns agent tasks, resource scheduling and affected-file claims.
Agents operate against isolated project snapshots or worktrees, incorporating
unsaved buffer snapshots explicitly. Changes and tool side effects stay isolated
until review. Test commands run in that isolation; filesystem or process tools
cannot bypass the proposal boundary by writing directly into the live workspace.

For a diagnostic fix: capture the diagnostic revision and task scope, investigate,
propose a repair, execute relevant checks, then present the diff and evidence.
On acceptance, compare current revisions and apply transactionally. Conflicts
require renewed review. Rejection leaves user content untouched. No agent can
approve its own proposal or use another agent to bypass review.

### Required UI and AI action parity — owner, 2026-11-11

Every semantic editor/user action must have a direct integrated-AI action route.
This is an implementation requirement, not an optional automation layer after
editor completion. UI controls, menus, shortcuts and AI tools invoke the same typed
commands and services. Keep a single command definition for its arguments, result,
availability, target scope, preconditions and effects; derive discovery/tool
contracts from it rather than maintaining a second behavior table.

Actor identity and grants come from the trusted host, never model-supplied flags.
Tool requests target explicit workspace/document/session identities and expected
revisions. Read/search operations return current unsaved-buffer snapshots where
appropriate. Background agent work must not steal focus or change the user's
active tab unless that UI action was requested. Progress, cancellation and failure
are observable through the same underlying job used by the UI.

Parity does not authorize unrestricted execution. Preserve the existing isolated
proposal/revision boundary for agent edits and filesystem/settings writes. Run
commands with file/code-execution effects in isolated task workspaces unless an
applicable host grant explicitly permits them. Scope external disclosure and
consequential/destructive actions to the appropriate approval. Agents cannot
approve their own proposals, change grants or bypass review through another tool,
terminal, process or agent. Safe inspection/navigation/view commands can execute
within granted scope without inventing an approval for every action.

A feature is complete only when its UI and registered AI action routes work through
the same implementation, with relevant stale-target, denied-action, error and
cancellation behavior demonstrated. Retain already passed UI evidence separately;
metadata, exported model functions and placeholders are not proof of AI integration.
Model quality qualification remains a separate acceptance condition.

Current source audit (November 11; AI action integration remains pending):

| Existing feature family | Reusable implementation | Missing AI hookup |
| --- | --- | --- |
| Files and content search | workspace entries/projectFiles and shared search actions, including draft search/regex/case and UTF-8 match ranges | Search-content/result-navigation catalog adapters pass host-state/permission/stale-input checks; live agent transport, filename-search and presentation-command routes remain pending |
| Open/select/close documents | workspace openFile/selectFile/closeFile and recovery publication | Explicit-target command path; retain dirty-close/recovery guards |
| Edits, saves and proposals | document edit/propose/apply, saveDocument and stale byte/session/revision checks | Registered proposal/review transaction commands; no direct AI write bypass |
| Selection and editor operations | std:ui readInput/controlInput expose snapshots and some actions | Domain-target adapters; checked undo/redo and remaining native operation hooks |
| Tabs and split panes | Native Dock interaction and saved layouts | Checked programmatic reorder/split/resize commands; snapshots alone cannot execute actions |
| Settings, themes, locale and zoom | Validated settings/actions and zoomWorkspace | Shared invocation with guarded persistent writes and explicit view-state targeting |
| Project checking | .tsr worker with real CLI JSON, result/cancellation channels and bounds | Host-owned job commands and AI transport; current controls are UI-owned |
| Markdown/images and view toggles | Native viewers and component signals | Shared preview/navigation/panel commands, preserving focus and user state |
| Terminal, language services, Git, debug, extensions and AI review | Some native/library primitives; product services incomplete | Implement real services and register both UI and AI actions together |

Next implementation section: connect a shared typed dispatcher and executable AI
adapter to existing search/navigation/project-check services, then extend guarded
mutations and missing native command hooks. A model is not required to exercise
that adapter, but invoking a helper directly is not a substitute for testing the
registered route. Do not label the current app AI-wired before that work lands.

### Reusable AI capabilities

Investigate a public inference capability (working name `std:ai`) backed by a
managed Rust library, plus a training capability if necessary. These are proposed
module boundaries, not implemented APIs. Keep editor-specific prompts, retrieval,
task planning, tool selection and agent loops in Tsrust application code.

The public boundary needs typed model/device configuration, tokenizer/template
identity, streaming, stop reasons, cancellation, bounded contexts and queues,
structured/tool-call output, memory observations and deterministic error reporting.
Ownership tests must cover disposal, cancellation and late callbacks. Generic
bindings are preferable to compiler special cases when they can express the API.

Initial inference candidates:

| Candidate | Evidence and reason to investigate | Still to prove |
|---|---|---|
| Candle | Rust ML framework with CPU/CUDA/Metal support and quantized model examples; exposes differentiation and optimizer primitives. | Exact selected coding-model coverage, usable streaming/tool output, device memory, adapter training and checkpoint portability through Tsrust. |
| mistral.rs Rust SDK | Rust-library integration, quantization and supported-model loading are documented. | Dependency closure, optional-feature isolation, selected model/device support, cancellation, Windows build and 8 GiB behavior. |
| llama-cpp-2 Rust SDK | Pinned 0.1.161 static native library now passes the bounded Metal/mmap quantized-model smoke with strict tokenizer agreement. | Persistent streaming/session API, cancellation, held-out coding quality, full-machine acceptance, clean build/export closure and Windows. Training remains a separate Candle investigation. |

Do not choose a backend from feature claims alone. Pin exact revisions, review
transitive native build requirements and test through the actual Tsrust facade.
Do not enable an upstream Python/shell agent runtime as a shortcut: our agent and
training orchestration belongs in Tsrust. The inference engine only evaluates
models and exposes the necessary generation primitives.

### Local model research, specialization and distribution

1. Build versioned evaluation tasks from compiler contracts, API docs, checked
   examples, real application tasks and known failures. Split by task lineage and
   source families before deriving training samples, keeping held-out evaluation
   private from the training/retrieval inputs for that evaluation.
2. Establish pretrained-model baselines with a Tsrust-authored harness, current
   documentation retrieval, source navigation and compiler/test feedback. Test
   fill-in-the-middle completion separately from instruction/chat/tool use.
3. Compare candidate architectures, sizes, quantizations and context budgets.
   Keep task success and patch correctness distinct from speed and memory.
   Establish a reproducible quality floor before selecting the distribution.
4. Implement the Tsrust-driven adapter/fine-tuning pipeline over checked native
   tensor/training facilities. Prove optimizer updates, loss/masking, tokenizer
   agreement, checkpoint save/resume and an overfit control before expensive runs.
   Quantized inference support does not establish quantized training support.
5. Train on curated, licensed Tsrust examples and repair trajectories. Compare
   tuned versus unchanged baselines on held-out tasks and retain general Rust and
   related-file competence. Do not automatically train on a user's private project.
6. Export/merge adapters and quantize where supported; validate the exact resulting
   download, not only the higher-precision training checkpoint. Include model and
   dataset provenance, compatible toolchain/API versions and independent oracles.
7. Distribute only evaluated configurations with manifests, license notices,
   hashes, tokenizer/template files, measured resource requirements and rollback.
   Support resumable downloads, interrupted installation, removal and offline use.

Minimum-machine acceptance concerns full offline inference and the editor workflow.
Training on 8 GiB is a separate feasibility question; do not promise it. Evaluate
the owner's Windows GPU for larger training runs when available. The same
Tsrust-authored training program should run on supported devices without Python.
Do not begin costly training or model publication merely because research is
authorized; record the dataset, run budget, hardware and distribution decision.

Start the minimum-memory experiment with one resident model and a scheduler for
logical completion/chat/background sessions. Prioritize interactive requests,
cancel obsolete completion work, bound caches and schedule builds alongside AI.
Multiple simultaneously resident models are optional higher-tier capabilities,
not a requirement for representing multiple agents. Do not use hidden cloud
fallback to claim the offline minimum passes.

### Cloud and external agents

Add direct provider adapters and investigate ACP for external compatible agents.
Keep transport, authentication and provider capabilities explicit. All agents
participate in the same review, revision and task-coordination system. A cloud
planner may delegate bounded tasks to local workers and inspect their evidence;
delegation is not permission to write into the user's live files.

Cloud use is optional and user-selected. Distinguish standard API credentials from
provider-specific external-agent sign-in. Do not assume a subscription grants
API access or every agent/provider supports the same tools and context behavior.

## Full feature acceptance inventory

Turn these rows into concrete witnesses during the initial capability audit. The
rows are required scope, not claims about the current CodeEditor implementation.

| Area | Required behavior |
|---|---|
| Editing | Large and small files; Unicode/IME; selections and multiple cursors; undo/redo; indentation, comments, bracket matching, snippets, folding, find/replace; reliable saves and recovery. |
| Workspace | File tree, tabs, split panes, multiple windows/projects, recent projects, commands/keybindings, settings/themes, drag/drop, file watching, fast file and workspace search. |
| Intelligence | Completion, hover, signatures, semantic highlighting, symbols, definitions/references, rename, code actions, formatting, diagnostics and navigation for supported language services. |
| Project tools | New/import/open projects, dependencies/bindings, build/check/run/test, streaming diagnostics, cancellation, test navigation, toolchain/version management and independent CLI agreement. |
| Terminal/debugger | Native PTY with resize/input/shell lifecycle; debug-adapter integration, launch/attach, breakpoints, stepping, variables, stack and console; Tsrust source mapping for generated native code. |
| Git/review | Status, stage/unstage, diffs, hunks, history, conflicts and proposal review; preserve existing uncommitted work and support multi-file undo. |
| Previews | Markdown and HTML previews and a managed web-view component; navigation, focus, keyboard, resize, disposal and content-origin boundaries. |
| AI | Local model management, inline suggestions, chat/context, project tools, background diagnostic fixes, local agents, cloud coordination, external agents, cancellation and proposed diffs. |
| Reliability | Keyboard/accessibility, persistence/recovery, worker/server crashes, corrupt models, interrupted downloads, unavailable GPU, low memory and stable project data. |
| Delivery | Matched editor/toolchain compatibility, offline setup/use, qualified model distribution, macOS and Windows acceptance, then signing/packaging/update recovery. |

Extensions are lower priority than built-in workflows; research existing Kit
facilities without committing to VS Code extension compatibility or designing a
new marketplace. Browser-hosted Studio, multiplayer, mobile clients and Linux
delivery are outside this agreed initial scope. Supported-file syntax editing and
language-service capabilities must be stated separately for each file type.

## Purposeful layout and configuration — owner clarification, November 10

UI design and reusable native capabilities progress together with safe document
editing and storage. Do not leave styling/interaction cleanup until the end.
Every region needs consistent surfaces, deliberate dividers, compact tooltip
icons, shared text zoom, truncation/overflow behavior, resize/collapse and verified
keyboard/focus states. Keep `.tsrx` composition split into chrome, navigation and
editor panes; format coherent UI sections during development as well as checkpoints.

Configuration must cover locale/i18n and every user-visible label, appearance and
text sizing, navigation side, activity-rail side, terminal docking and AI-chat
docking. Defaults are navigation/rail left, terminal bottom beneath the editor
(with navigation full height) and AI chat right. These are movable defaults.
Use one validated, persisted settings model with explicit user/workspace scopes;
retain drafts/editor state when settings or layout change. Reuse std:ui locale,
translation, Dock, Resizable and Settings capabilities, extending public wrappers
where native Kit functionality is missing. Do not add separate ad-hoc layout
switches, invented docking machinery or empty terminal/chat placeholders.

Current shell is an incremental implementation, not acceptance of those settings
or docking requirements. Persistent configuration, translation catalogs, full
native docking, working terminal/chat, drag tab reorder and reliable save/recovery
remain required alongside the current UI correction.

## Internal execution phases

1. **Capability and acceptance audit.** Inventory current APIs and editor behavior,
   upstream reuse candidates and exact language-service dependencies. Register
   concrete gaps for editor text/deltas, PTY, debugger/source maps, web view,
   credentials, inference, training and hardware observations. Establish benchmark
   workloads and quality tasks. Set numerical latency/memory/quality gates from
   baseline measurements before optimizing or claiming acceptance.
2. **Native local-AI proof.** Through Tsrust only, load a candidate model, stream
   responses, cancel, use a compiler/test tool, produce a diff and unload on the
   M3. Measure baseline/tuned-model feasibility and device/dependency compatibility.
   Failure to meet the 8 GiB/quality gate remains an open blocker, not a scope cut.
3. **Studio project and document foundation.** Create the ordinary app under
   `apps/studio`, implement document transactions, recovery and the shared project
   service, then prove editor/CLI/LSP agreement. Use Studio on its own sources as
   soon as useful, without labeling that point feature-complete.
4. **Complete development workflows.** Finish the full editor, language services,
   search, terminal, Git, tests, debugger and previews across public Tsrust APIs.
   Close gaps in coherent families, retaining regression and ownership evidence.
5. **Integrated local AI and specialized model.** Complete inline/chat/background
   flows, revision-aware review, scheduling, measured fine-tuning, model downloads
   and minimum-machine acceptance with realistic projects and active builds.
6. **Cloud/external integration and Windows.** Complete provider/agent adapters,
   local-worker delegation and Windows acceptance on the owner's reference machine.
   Preserve Mac evidence and rerun only affected checks after platform repairs.
7. **Full acceptance, then delivery.** Every required row passes, qualified model
   artifacts are evaluated, and no data-loss/stale-edit issue remains. Only then
   proceed to signing/packaging and update/install tests. Publication/version
   approval remains a separate explicit owner decision.

Each implementation phase carries compiler/native/editor documentation and
examples when public APIs change. Use stable `std:testing` suites and `tsrust test`,
one coherent compilation per family, the shared Cargo target, incremental off and
jobs=1. Respect the under-one-hour test execution requirement, excluding builds.
Do not restart unchanged broad matrices after UI-only edits. Store actual
performance measurements through the existing SQLite workflow; no new canonical
logs, snapshots or parallel benchmark store.

## Minimum-machine proof and remaining decisions

Measure the whole machine while Studio edits a realistic project, runs the
relevant language services, serves inline requests, completes an agent task and
executes builds/tests. Include loading peaks, model weights, context/KV buffers,
worker queues, language-server/index memory, GPU allocations and memory pressure.
A model-only load test, tolerating unlimited swap or dropping context until tasks
fail is not acceptance. Report cold/warm startup, first-token/completion latency,
task success, stale-result rejection, cancellation and responsiveness separately.

Unresolved: exact backend and model revisions; public API shapes; worker boundary;
training hardware and budget; numeric quality/resource limits; final list of
language-service implementations; debug/source-map depth; and exact component
gaps. Resolve these with bounded research witnesses, not speculative general
frameworks. The initial bounded quantized inference witness now passes through a Tsrust
worker/test harness using a pinned native library. No model has been fine-tuned
or qualified for distribution or full minimum-machine acceptance; see the audit.

## Primary research sources

Checked 2026-11-10. Upstream claims are research inputs, not local test evidence.

- [Candle overview and device/model examples](https://github.com/huggingface/candle)
- [Candle differentiation/optimizer boundary](https://github.com/huggingface/candle/blob/main/candle-nn/src/optim.rs)
- [mistral.rs and Rust SDK](https://github.com/EricLBuehler/mistral.rs)
- [Agent Client Protocol](https://agentclientprotocol.com/get-started/introduction)
- [Zed editor/project architecture](https://zed.dev/docs/development)
- [Zed editor dependency and license boundary](https://github.com/zed-industries/zed/blob/main/crates/editor/Cargo.toml)
- [Code OSS license](https://github.com/microsoft/vscode/blob/main/LICENSE.txt)

Existing repository foundations: [GPUI application platform](gpui-application-platform.md),
[development acceptance workflow](../development-workflow.md) and the current
[continuation record](../agent-progress.md). This planning work does not silently
change unrelated v0.2.0 acceptance or authorize a release.

## Current implementation acceptance — November 10

- Initial capability/gap audit and bounded resource contract recorded in
  [the audit](tsrust-studio-audit.md). Final language-service selections, coding
  quality floors and full-workflow latency/resource gates remain open.
- Studio now opens its own apps/studio package with real folder browsing, native
  file tabs, full-height read-only code views and native Cmd+F Find. Navigation
  uses std:ui Tree/TreeItem with workspace contents directly at the top level; no
  redundant root row or convenience button bar. Earlier theme switching passed
  before removing the theme button; the current layout starts dark.
  File-opening tests pass3/3; native data-state/composition/navigation tests pass12/12.
  macOS dev identity and failed-build recovery are verified; existing proposal
  revision guards pass4/4. Native tab scroll/selection/per-file Find retention is
  verified after the runtime fix. Full AX/focus, editing,
  safe saves, recovery and the rest of the editor inventory remain open. The
  current dev supervisor stays running; current evidence is in the audit.
- Package tabs now own revisioned document sessions with saved-text baselines.
  Fresh session identities reject proposals from previously closed/reopened files,
  even with identical bytes and revision numbers. Dirty reload/close guards pass
  in the model. Integrated native document/workspace suite10/10 passes; native
  close prompts, safe disk writes and crash/draft recovery remain required.
- Shell alignment is checkpointed at acbcb8ed: compact custom chrome, tooltip
  icons, explicit dividers, restrained surfaces, actual sidebar resizing/collapse,
  filename search, close controls and shared zoom wiring. Model suite11/11 and
  final affected workspace5/5 pass. Tree sizing/ellipsis and stable moved rows are
  built and covered by focused runtime tests; the new native renderer still needs
  its authorized live load. Persistent settings/i18n/docking remain required.
- Dev checkpoint79c37206 retains an unchanged live application after successful
  rebuilds. Focused lifecycle regression covers test-only PID retention, real
  replacement, failure preservation, exited-child relaunch and shutdown cleanup.
  The retained Studio supervisor still embeds the earlier image.
- Atomic storage primitive has focused macOS acceptance: std:fs.writeTextFileAtomic
  stages/synchronizes replacement, checks optional expected text and observed
  revisions, and preserves macOS metadata. Native failure/metadata witnesses
  pass3/3, with final flags/mtime witness1/1. Public runtime/example batch3/3,
  compiler contracts, affected completion/help phases and backend all-target
  compilation pass. Exact identities and timings are in the audit.
  Windows remains unsupported for this new operation. It does not by itself
  enable Studio saving or recovery; commit/durability outcomes, drafts and
  editable-document integration remain required.
- Candle scalar optimizer proof3/3 passes. Three Candle CPU quantized-model
  attempts stopped on resource conditions, without answers. Those results remain.
- Exact same official Qwen1.5B Q4 model and arithmetic oracle pass with pinned
  llama-cpp-2/sys0.1.161, Metal/mmap, static linking, strict tokenizer agreement,
  unchanged context/output/deadline/swap ceilings and the Studio dev window open.
  Two native tests pass; all source/resource identities and observations are in
  the audit. This proves a bounded buffered inference primitive only. Streaming,
  cooperative cancellation measurement, repeated unload/reload, compiler/test
  tools, proposed diffs, held-out quality and full8GiB acceptance remain required.
- Native cache checksum/extraction whole-artifact buffers replaced by bounded
  streaming with protocol/validation regressions passing. Performance measurement
  remains separate; retained dev still uses its existing supervisor image.
- No cloud integration, model specialization/distribution, Windows, complete
  application, signing/packaging milestone or release acceptance is claimed.


## Owner editor acceptance checklist — November 11

Each item requires observed behavior; source changes alone do not close it.

| # | Requirement | Current acceptance |
|---|---|---|
| 1 | Separate Settings window; tsrust_studio.config.json persistence | Own custom-chrome window; Kit searchable General/Workbench pages with GUI theme/font/locale/region controls, immediate persistence, formatted full-height optional JSON and Save/Reload; external-write protection and legacy defaults verified |
| 2 | Matching navigation/editor text sizing | Shared explicit font-mono family, zoom and per-workspace restart retention observed; Cmd+0 records reset and Settings Reload retains offset; full layout acceptance pending |
| 3 | Content search, regex/case toggles; distinct filename command | Model regression passes literal/case/regex/UTF-8/open-draft cases; live regex/case toggles and result-line navigation observed; Enter/worker/cancellation pending |
| 4 | Image and appropriate non-text viewers | Actual package PNG decoded in app; other image formats and failures pending |
| 5 | Clearly distinguished active tab | Dark active-tab contrast observed; light/other themes pending |
| 6 | Tab/tree drag/drop, resizable editor splits | Native tab reorder/draft/undo, direct right/down splits, nested-pane redraw, all visible editor AX nodes, named tabs/close actions, pane maximize/restore and focused save observed; nested split axes/sizes, dragged tab order and per-group selection restore with exact serialized equality; fast horizontal/vertical/sidebar divider drags now observed with managed GPUI release fix (13 resize regressions pass); edge gestures, duplicate views, cursor/scroll persistence, focused-group restoration and tree moves pending |
| 7 | Markdown preview | Native preview observed after shared row-state fix; preview survives tab switches |
| 8 | Distinct JSON syntax categories | Shared lexical roles pass nested key/value/escape/number/boolean/null witnesses; native Studio Dark, Gruvbox Dark and Catppuccin Latte show distinct keys and values |
| 9 | Workspace launch argument; later code-dot/dock directory opening | Root uses launch argument/current directory; package default observed; alternate launch/drop pending |
| 10 | Editable buffers, saving and recovery | Formatted suite18/18 passes; native Unicode typing/Cmd+S/undo/redo/tab switch and unsaved recovery across supervisor restart observed; clean open files/active selection now restore (recovery2/2 passes), native Dock layout/zoom restore verified; IME/conflict/close UX and larger recovery scope pending |
| 11 | Broad theme catalog; configuration round-trip | 38 themes including36 pinned upstream variants; all38 persistence round-trips pass; detailed capture names retained with nearest-parent fallback; native import/parameter/JSON styling observed in Studio Dark, Gruvbox Dark and Catppuccin Latte, saved configuration preserved; full catalog contrast/rendering pending |
| 12 | Selected-tab auto-reveal; respect manual scrolling | New last tabs and clipped first/final files selected through named overflow are visibly revealed; manual scrolling remains unverified |
| 13 | Useful status bar with truthful file/diagnostic/service state | File type/encoding, unsaved state and unconnected-LSP status implemented; type/LSP text observed; services not yet connected |
| 14 | Terminal toggle; planned-feature surfaces and controls | Independent terminal/task and AI-chat toggles/close controls plus configurable left/right/bottom regions observed; PTY/debug/Git/AI/extension functionality remains pending |

Layout configuration now has model acceptance for all36combinations of navigation
and activity sides plus task/chat placement. Three native arrangements verify all
supported positions, independent visibility, saved-setting reload, Unicode draft,
split-pane and Undo preservation. English/Brazilian-Portuguese catalogs and live locale Save/Reload/startup recovery
are verified for owned Studio UI, with query/editor Undo retained across language
changes. Wider locale/RTL/native-control coverage, AX selected-range support and
cursor/scroll/focus restoration and duplicate views across restart remain required.

Owner permits supervisor restarts whenever needed without another permission
question. Keep one current dev session running. Planned-feature placeholders are
now explicitly requested and must be labeled honestly. This supersedes earlier
placeholder restrictions; it does not authorize fake completion claims.


November11 feature update: Settings now uses native Kit categorized GUI controls,
custom chrome and optional formatted full-height JSON, with immediate persistence
and stale-write protection. Tree/editor explicitly share font-mono. Explorer file
icons are data-driven, bundled and configurable; native off/on preserves selection,
expanded folders and panes. Focused mapping/settings proofs and native observations
are recorded in the audit. Owner prioritizes feature throughput/manual acceptance;
next work is actual task/terminal/language services, not repeated unchanged suites.


November11 organization/service update: src now separates categorized components,
lib models/actions, .tsr services and bundled assets. Tests/imports/localization
coverage follow the moves. Native saved-file project checking works in Output via
CmdShiftB or play, with verified Stop/child cleanup and actual success result.
Shared compiler fixes preserve concrete race/settled future types and keep async
GPUI entry on the OS main thread. Focused acceptance and remaining standalonequit
uncertainty are in the audit. This is compiler-task integration; LSP/PTy remainpending.


Search-result presentation now has separated full-width click targets, dividers,
file/line labels and cropped UTF-8-safe match highlights. Activation selects the
exact matching range. Shared UI/tool actions and tested search tool adapters are
implemented; live AI transport and full command parity are still pending. Native
visual/navigation evidence and focused boundary tests are recorded in the audit.
