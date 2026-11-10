# Tsrust Studio

Planning draft, 2026-11-10. Owner requirements are recorded below; architecture
choices and unmeasured acceptance targets remain proposals. This document does
not claim implementation, model quality, platform acceptance or release approval.

## Owner requirements

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
frameworks. No model has been downloaded, run, fine-tuned or qualified by this plan.

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
