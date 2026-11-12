# Tsrust continuation

This is the only handoff. Read AGENTS.md, docs/agent-goal.md and the complete
B00–B20 checklist in docs/plans/stdlib-0.2.0.md. Full goal remains active;
v0.2.0 merge/publication is not approved. v0.1.2 is released.

## Current continuation — 2026-11-08

### Tsrust Studio active implementation — 2026-11-11

Latest accepted native interaction — 2026-11-12, source checkpoint10777c1f.
Final checkpoint/cleanup:10777c1f contains the complete native help/snippet section
and focused evidence. Worktree was clean after that commit. Final MAT verified
single-line ellipsized completion details, syntax-colored editor-sized documentation,
correct snippet body/closing indentation, placeholder replacement/Tab navigation and
preserved src/services expansion through dirty/clean transitions. Only the owner's
original project-check draft remains; no probe was saved. Source identity is10777c1f;
this handoff update records the following cleanup, not another implementation gate.
Pruned20obsolete Studio native cache entries/build-workspace payloads under namespace
and per-workspace locks, preserving three latest runtime/program artifacts, two latest
build payloads and all build lock/identity files. Studio cache12.3GiB ->4.5GiB; shared
root Cargo target and model qualification artifacts untouched. Removed the closed
manual preview bundle and temporary numeric-snippet test executable. Future build
pruning must filter for actual native-support/artifacts/target payloads before sorting;
empty lock/identity shells deliberately remain and their directory mtimes changed.
Current dev supervisor81803/PTY55205, app82069/LSP82075 confirmed alive at~6min;
LSP idle0%CPU,RSS259024KiB (RSS is NOT the footprint/peak). Latest CUA phase completed;
no permission needed for legitimate dev restart. Continue with bounded retained
location/reference-memory investigation rather than another broad regression run.

The preceding goal turn made progress: native build/MAT completed; three observed UI
bugs repaired, built and verified without broader regressions. Native popup docs now
use editor-sized syntax-highlighted examples with no inner border or language badge.
Completion rows constrain primary/detail labels, flatten detail newlines and ellipsize
instead of overlapping. Numeric snippet Ok default selection/replacement, Tab to body,
body insertion and corrected indentation all worked in the actual app. Targeted LSP
space/tab indentation1/1 passed0.04s; SDK pair build2m44s. Native Tree refresh now
preserves expansion and selected ID, honoring explicit controlled expanded props;
real dirty/clean transitions kept src/services expanded and selection intact. No new
public Tree API or application-host Rust introduced.

CURRENT ONLY PREVIEW supersedes process details below: supervisor81803 / PTY55205,
app82069, LSP82075, bundle
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-81803-1794460353665739000-0/Tsrust Studio.app.
Old78662 / PTY36743 stopped after replacement GUI was confirmed. New supervisor has
explicit shared CARGO_TARGET_DIR, jobs1/incremental0, TSRUST=~/.cargo/bin/tsrust,
NO_COLOR/CODEX_TOOL_CALL_ID cleared, elevated macOS desktop access. Native build
322241ms: check72863.0ms,emit3312.7ms,runtime222432.4ms,native/link245292.6ms,
runtime/program misses. Log /tmp/studio-editor-language-polish-dev.log. Installed
CLI fda077eb5701cdfcdb88bf42288ba103fdea7459ed65b898f8b51d7745452c2c,
LSP cf8bc15c3a22a8f413c5861d84c669d21cf535dcc8401f6588a21066871c872e,
atomic installation and independent hash matches. All earlier passed evidence below
remains valid; no full compiler/LSP rerun, push or release.

CUA refresh issue was stale control state after replacing processes, NOT proven owner
activity: js_reset + first-call getApp(full current bundle) fixed it. Repeat this on
replacement; after compaction first rewriteDocumentation. Last CUA phase completed
with completion_summary Verified repaired snippets and explorer. Two probes were
ONLY unsaved appended recovery.tsr text; first143chars/second151chars removed exactly
from EOF after verifying prefix matched savedText. Never saved them. Remaining sole
owner draft is still project-check.tsr4spaces on its blank line. Owner tabs9/zoom-1/
active project-check preserved; src/services expanded and original tree selection
restored. Do not erase/save the owner draft during cleanup.

Memory remains unresolved. After real completions, snippet edit and143backspaces,
previous LSP79444 settled617.9MiB/peak947.7MiB at12min,0%CPU, vs initial418.6MiB
active at2min. This does NOT qualify a leak fix or formal improvement over old
708.2MiB idle/1.0GiB peak. Trace /tmp/studio-editor-language-memory-after-mat.txt.
Sharing eliminates proven duplicated catalogs, but large retained storage persists.
Next evidence-led target: ProjectIndex.references stores a serde_json::Value location
for every reference token; SourceIndex.foreign_definitions and nominal_definitions
also store nested JSON locations. reference_locations eagerly clones/materializes
both definition and reference JSON. The old heap's497,836640-byte nodes may include
these JSON B-trees (inference, not allocation-stack proof). Inspect/count retained
locations, consider compact typed locations with JSON generated only for replies,
and verify navigation/reference equivalence plus repeated real-use memory. Do not
cap caches or weaken diagnostics to hide growth. Full intelligence, signature/
keyboard-hover UI, external servers, integrated AI hosting and8GiB qualification
remain open. Planned ignore traversal follows current intelligence/memory slice.

Native language help, snippets and memory — active uncommitted section after95b491c8.
Previous goal turn made progress: real native bridge, guarded AI reads, sender sharing,
presentation fixes, staged cancellation, immutable completion-catalog sharing and
focused validation; ignore discovery added to the plan at owner's request. Full
Studio goal remains active, one writer/no agents. No push/release/full matrix.

Implementation: CodeEditor onLanguageRequest/onLanguageCancel + answerEditorLanguage;
existing HoverProvider/CompletionProvider with Ctrl-Space, window-owned pending
requests/5s timeout/drop cancellation/process-wide unique native IDs, UTF-16/scalar
range conversion, unchanged opaque completion data, and stale text/revision guards.
Tsrust session/controller synchronizes exact document identities and versions,
correlates/cancels reads; LanguageState is App-owned and shared with editors/status.
Registered language_hover/completion/read_result/cancel_read use invokeLanguageTool
and trusted host allowRead/root + explicit target identity. No model hosting or
full editor action parity claim. New registry is sourcepacked through stdlib/ui.

Required Sender.share creates another owned channel endpoint with independent close.
Future-native emission now honors access wrapping for temporary returned sender/token
values; this fixed actual native Studio E0308. Complete builtin docs/example and
matched Tsrust/TypeScript/Rust async_channel_share_try workload/oracle/driver/manifest/
contract support are ready, UNMEASURED. No SQLite timing claim.

Native popovers now read the editor's DisplayMap font; hover/completion/diagnostic
Markdown code has no inner border/radius or language badge. Presentation-only Tsrust
fences use existing Kit TypeScript/TSX grammar mapping; MarkedString language values
render as fenced code. Gutter removes code-editor left padding, trailing line-number
margin and fold-icon inset. Native numeric snippets expand stops/defaults, select
mirrored occurrences, support Tab/Shift-Tab, adjust offsets on edits, cancel on
undo/reset/Escape. Variables/transforms/nested defaults remain unsupported and are
rejected before insertion. Signatures/external servers/full snippet scope still open.

Memory work: obsolete document/workspace snapshots set the existing worker interrupt
flag. Skip later recovery/index/action stages, including between repaired graph checks;
an already-running compiler pass stays synchronous. Immutable native import completion
catalog JSON uses Arc<Vec<Value>> shared across source indices; project export catalogs
share per dependency within a graph. Clone selected response items before per-document
text edits. Unknown empty module catalogs are not retained. No arbitrary cache cap or
reduced checking/inference coverage.

Passed focused checks (preserve, do not rerun unchanged): native registry2/2 (0.00s);
public Studio language-service9/9 (802.90ms), real worker hover/completion/restart,
stale/identity/grants/no focus or edits, Sender.share and readonly temporary witnesses;
existing async task suites15/15 (8.71ms); benchmark contracts4/4 (257ms). Direct services
and workspace-view checks passed. New numeric snippet source witness1/1 (0.00s);
Markdown mapping/isolation1/1 (filter also ran one existing i18n case, total2/2,0.00s);
cancellable recovery + existing array capability/generic prefix3/3 (0.14s);
protocol stale edit/close/reopen1/1 (0.00s); native catalog storage/reply isolation1/1
(0.04s); existing imported inherited help/Signal alias2/2 (4.70s). Native Studio builds
passed. Whole-project formatter latest0files/0snippets/0fingerprints; diff check passed.

Installed CLI8099246bf55af8fbdda6049faaf7948d4fc9aa8c26462fdf0128cf8de85ef4c3,
LSP21e249aa47c153dde406ccfa4f26f1636d44892ed221991ba41a7d9a6b695b42, atomic copies
and independent hashes matched. Final pair build2m49s; LSP-only sharing refresh18.10s.
First native candidate built850398ms (runtime746434.6ms) but sandbox launch failed
NSPasteboard generalPasteboard NULL; supervisor69130 / PTY24597 stopped. It omitted
CARGO_TARGET_DIR. Corrected elevated dev launch explicitly sets shared root target:
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/Users/seancotter/Desktop/dev/learning/tsrust/target.
It built324464ms: check71328.4ms,emit3309.7ms,runtime220936.1ms,native/link248967.7ms,
runtime/program misses. No need another pre-build or repeat tests of the executable.

CURRENT ONLY PREVIEW: supervisor78662 / PTY36743, app79412, LSP79444, bundle
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-78662-1794459016898094000-0/Tsrust Studio.app.
Log /tmp/studio-editor-language-desktop-current.log. One benign mac sandbox_extension
warning after successful GUI launch. Old58964 / PTY52232 stopped after replacement.
Brief cached-binary preview /private/tmp/studio-language-preview-vuy7vups was closed
(PTY84407); no duplicate apps/LSPs remain. That temporary bundle can be removed.

Live acceptance: BEFORE latest styling/snippet edits Ctrl-Space showed real declarations
and Channel docs, automatic console completion log/error appeared, accepting filtered
log replaced l and cleared diagnostic. Probe removed exactly, no saved file changes.
Owner later observed actual hover but reported old oversized/unhighlighted/badged style.
LATEST app screenshot confirms compact gutter, connected service,0errors/0warnings,
9owner tabs/zoom-1/active project-check and1owner draft restored. CUA actions on the new
window repeatedly reject with 'user changed ... re-query'; full AX refresh did not
resolve it. Do not force edits while owner is interacting. Latest CUA phase completed
with completion_summary Loaded updated Studio preview. New popup styling/snippet tab
stops still need interaction MAT; not accepted from compilation or source alone.
Next CUA action may reset a stale control session after multiple app replacements,
then first call exactly getApp(full CURRENT path); after compaction rewriteDocumentation.

OWNER DRAFT: project-check.tsr has4indent spaces on blank line103, appeared AFTER
our old completion probe was fully removed/0drafts verified. Preserve it. Recovery
~/.tsrust/studio/recovery/0623928fcb7d7c52bea51167921ccac51f064395e1d1b9efc6df8004bb8ee2cf.json.
Do not treat it as our scratch text or save/reset it. Any next snippet probe must
use a different clean tab and restore only its exact appended content, leaving this
owner draft intact and returning active selection afterward.

Old LSP59142: active footprint1107592608bytes (~1056.3MiB),~98–100%CPU;3s sample
showed with_diagnostics -> recover_project_member_receivers -> whole-graph compiler
check. Eventually idle708.2MiB,1.0GiB peak. Bounded heap inspection succeeded:2,517,558
malloc nodes,483,601,312live bytes;497,836640-byte nodes account for318,615,040bytes.
vmmap~461.2MiB allocated +243.1MiB fragmentation. JSON B-tree attribution is an
INFERENCE without allocation stacks, but repeated catalogs are proven by source/test.
NEW LSP79444 at~2min:418.6MiB footprint/peak,100%CPU,RSS382736KiB.3s sample now in
ordinary project check_graph_diagnostics/analyze_recursive_with_proofs, protocol
main parked. This is a DIFFERENT PHASE/workload, not a comparable improvement or
leak qualification. Logs /tmp/studio-editor-language-memory-current.txt and
/tmp/studio-editor-language-sample-current.txt; older heap/memory traces under same
prefix are disposable. Continue repeated-use/settled-memory observation on current
process; no restart-savings claim or quiet-machine benchmark claim.

Owner ignore0.4.28 follow-up: documented in docs/plans/tsrust-studio.md section
Shared workspace discovery and ignore rules. After active native intelligence/memory
section, before broader workspace index/search expansion and Git/debug/extensions.
Shared caller policy, explicit imports/compiler inputs preserved, configurable hidden/
ignored visibility, deterministic results, symlink/errors/cancellation/AI grants.
Dependency NOT added/traversal NOT changed/performance NOT measured. Primary docs
verified at https://docs.rs/ignore/0.4.28/ignore/.

Next: finish native popup/snippet MAT when control is stable, measure current LSP
settled/repeated-use memory, final format/local checkpoint with exact remaining limits,
then remaining fast intelligence/signature/keyboard hover and prioritized discovery.
Prune obsolete native variants under namespace/per-directory locks, keeping current
and two recent/shared target. Root target is~82GiB (debug~62GiB); preserve model
qualification artifacts and unrelated work, do not indiscriminately clean everything.
Full goal, external servers, integrated AI host and8GiB whole-app qualification remain
open. No broad compiler/LSP matrix or release authorization inferred.

Early declaration help and stale-result release — 2026-11-11, follows9d7fc272.
Previous goal turn made progress: verified panel inset and local checkpoint.
Owner now requests6px instead of12px on both Close and terminal trash. Applied
pr-1.5 to shared panel action group and terminal SessionItem, preserving left
row padding. Live screenshot confirms same action column; no extra borders.

LSP analysis worker now publishes ProjectIndex::declarations from current module
headers before full body checking. This path skips compiler-based recovery;
preliminary completions use CompletionList isIncomplete:true. Checked and existing
recovery constructors remain unchanged and still replace the preliminary index.
Intermediate declaration messages retain busy/in_flight ownership; current document
and full workspace snapshot guard publication. Protocol witness proves imported
hover arrives BEFORE body diagnostics, including when body checking fails. This is
not a universal low-latency guarantee: queued work/cold scans, local fallback
SourceIndex::new and later compiler-based member repair remain concerns. Native
Studio hover/completion provider bridge is STILL MISSING; no UI intelligence
acceptance claimed. Next executable section is that bridge plus correlated,
cancellable Tsrust-authored read requests and guarded AI read actions.

ProjectSession now releases invalid indices/actions on edits, open/close, watched
changes and result insertion; dependency registrations and diagnostic contributions
remain for rescheduling/publication. No arbitrary cache cap or reduced checks.
Repeated20dependency edits prove old observation Arc destruction and preserve an
unrelated current project; watched invalidation/close covered. First witness failed
because its own unrelated local input retained all open buffers; dropped that input,
then witness PASS0.02s. Declarations hover/signature/incomplete completion witness
PASS0.04s despite broken bodies. Public stdio ordering witness1/1PASS0.45s,
/tmp/studio-lsp-declarations-protocol.log. Existing session4/4PASS0.02s and server
lifecycle6/6PASS0.02s, /tmp/studio-lsp-declarations-lifecycle.log. No full matrices.

LSP-only build17.27s, /tmp/studio-lsp-declarations-build.log; atomically installed
and independently hash-matched853f67ff17b1cc1ef30aa8fa2527e8d80d99bfe71467d3304424cdae55b81942.
Compiler unchanged; no paired compiler rebuild needed. Real status-menu restart
replaced child37371 with42232; same app37370/supervisor30396/PTY88187 remained.
LSP connected,0errors/0warnings. Owner selected project-check.tsr and expanded
services tree; preserve that newer selection. Bundle:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-30396-1794450506353980000-0/Tsrust Studio.app.
Native spacing rebuild97608ms (check67994.1ms,emit3244.6ms,backend175.3ms,
native/link25668.5ms,runtimehit). CUA ended "Verified margins and LSP reload".
Required formatter0files/0snippets/0fingerprints and diff check PASS.

Bounded live memory observation before installing this LSP: old37371 active100%CPU,
footprint811041896bytes (773.5MiB),635224064compressed/swapped; Malloc Small
770703360bytes,Large33112064,Metadata3670016. /tmp/studio-lsp-footprint-before.txt.
No heap ownership trace or repeat-use growth qualification yet. The stale-cache
witness proves a specific release, NOT that it accounts for this footprint or that
the overall memory issue is fixed. Fresh child numbers are not comparable savings.
Keep profiling alongside actual hover/completion use; avoid broad compiler speed
experiments or delaying native intelligence behind full-check optimization.

Panel-header inset — 2026-11-11: shared Panels TabsSuffix action group now
uses pr-3 (12px), matching terminal ListItem px-3. Applies to Close in every
panel and keeps New Terminal/Copy Problems together; no semantic/action changes.
Required whole-project formatter0files/0snippets/0fingerprints; diff check passed.
No new automated test for this spacing-only change. Live screenshot confirms
Close moved12logical pixels inward, matching the terminal trash action column;
Close hides panel and terminal toggle restores it. CUA ended "Verified inset panel
close control". Automatic dev rebuild99981ms, app35512 under supervisor30396/
PTY88187; current bundle tsrust-30396-1794450270616835000-0/Tsrust Studio.app
under the same temporary parent. Keep running; LSP connected,0errors/0warnings.

Argument wrapping and asset imports — accepted, follows 4e7cfa23.
Shared CLI/LSP format.argumentWrapThreshold defaults to5 (0 disables,0–100).
Wraps function/callback parameters and call arguments above the direct-item limit;
nested commas/generic types are excluded. Import groups are std:, code paths,
then asset/data paths, separated by blank lines and sorted within groups. Bare
side-effect imports remain ordering barriers. Studio global/project overrides,
pretty config, native Settings number input and en/pt-BR strings use existing
formatting policy and guarded format_document routes. Applied to Studio itself.

Focused formatter family4/4PASS3.97s, including639lexable fixtures and86standalone
contracts for token preservation/idempotence. Public Studio runtime6/6PASS25.96ms,
including actual CLI/guarded proposal, override threshold, disk unchanged and
invalid permission grants rejected. Logs /tmp/studio-argument-format-{tests,runtime}.log.
No full compiler/LSP matrix. Whole-project formatter15files/1documentation snippet/
5fingerprints; explicit Studio organization21files; final idempotence0/0/0.
Other source changes are required token-preserving formatting. Diff check passed.
Final SDK build2m54s, /tmp/studio-argument-format-sdk-final.log; atomically installed
and hashes independently matched CLI f9f8acbac1dabb2209bc5c1d973a16f0a2dc90347012b4d35755002b983b39fb,
LSP c9b2ad74d293a301f782822a47435ed15beeec074d9fa5e158a187612d1cb62e.
Native final build102900ms (check70605.4ms,emit3209.9ms,backend191ms,
native/link28291.1ms,runtimehit). Setup observations, not benchmarks.
Live native Settings5→6→5 persisted correctly; Main App signature multiline and
assets last. Final preview LSP connected,0errors/0warnings,9clean tabs,zoom-1.
CUA ended "Verified the running Studio preview". Supervisor30396/PTY88187/app30480,
LSP30483; leave running. Bundle:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-30396-1794449456062244000-0/Tsrust Studio.app.
Old22770/19783/app26971 stopped only after replacement started; one preview.

Owner asked about544MB LSP. Idle old child26972 measured with footprint --swapped:
physical570754152bytes (544.3MiB),peak813270120 (775.6MiB),compressed/swapped
569868288; Malloc Small548061184. RSS1360KiB alone was misleading. One sample,
not proof of leak or acceptable8GiB qualification. Final reload legitimately
restarted LSP; lower fresh-process use MUST NOT be presented as a memory fix.
Read-only hypothesis: ProjectSession retains per-root ProjectIndex values owning
whole graph/source/token/symbol maps; overlapping retention is unmeasured. Next
bounded investigation needs cache counts/ownership and repeated-edit growth,
without delaying usable hover behind full checks. No heap dump/growth test yet.
Actual integrated agent hosting remains pending; existing guarded formatter action
is executable, not a claim that model hosting or all intelligence features work.

Quiet timer redraws — accepted, follows c94cefc9.
Owner steering2026-11-11: prioritize lightning-fast hover and interactive LSP
feedback; full compiler checks are secondary. Do not continue general compiler
optimization before wiring usable intelligence. Scope of this checkpoint is only
idle timer painting; no new LSP intelligence feature is claimed.

Root::emit uses non-forced settlement for timer ticks only. Bindings still refresh;
changed bindings/global signal revisions/queued commands/locale changes request
rendering. Other input events retain forced redraws. commands::has_pending and
i18n::has_changed are private, non-consuming checks.150UI library tests PASS0.05s,
3.36s setup, /tmp/studio-timer-tests.log: command owner isolation, locale pending
state and real timer callbacks with ordinary mutable bindings/shared state/commands.
Initial test compile needed missing imports fixed before the passed run. Timer
witness moved from dock_layout_tests to general kit tests afterward, code unchanged;
final targeted witness1/1PASS0.00s/3.63s setup, /tmp/studio-timer-final-witness.log.
No public API/config/builtins change and no AI action semantics change.

A separate error-effect fixed-point reuse experiment passed49internal tests1.07s
(3m28s setup), but Studio CLI observations were64.93s before/65.14s after
(64.53/63.90s user), complete stdout identical. NOT a meaningful speedup. Removed
experimental code/tests/counter, restoring error_effects/mod.rs exactly from
c94cefc9. Logs /tmp/studio-effects-before.log, /tmp/studio-effects-after.log and
/tmp/studio-effects-tests.log are disposable observations, not benchmarks. Do not
repeat that rejected cache or present it as an accepted compiler optimization.

Final UI-only SDK pair PASS2m49s, /tmp/studio-performance-sdk.log. Installed
atomically and independently matched CLI64884661489e9fe02a8d352ccf9d5aa825232f06ff6026305ccffedd96c0156b;
LSP90a333c9bff90bc00b241d46711ef47645af5b6b458be4b92412a07c963b4aba.
Native build PASS141.859s:check65.6638s,emit3.184s,backend178.9ms,
runtime-prepare4.6324s/runtime-build50.4769s,native/link72.2472s. Setup observation,
not a benchmark. Current supervisor22770/PTY19783/app22879; keep running. Bundle:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-22770-1794445953601229000-0/Tsrust Studio.app.
Old15484/PTY94187/app17336 stopped only after replacement started. One preview.

Idle CPU single5s observations: before app17336used0.20CPU-seconds/5.016s wall
(3.99%of one core); after app22879used0.09/5.012s (1.80%). Point observations, NOT
statistical benchmarks/leak acceptance. Live UI shows LSP connected,0errors/
0warnings. Status-menu restart replaced real child22882with23095 and reconnected.
Main tab active,9clean tabs,zoom-1,recovery drafts[]; panel hidden. Current owner
selection differs from earlier terminal-panel baseline; preserve main selection.
CUA ended "Verified quiet timers and reconnect". No new unsaved diagnostic probe
this section; prior live Problems/navigation evidence retained. No full compiler/
LSP matrices. Required formatter0files/0snippets/0fingerprints; diff check passed.
Pruned1obsolete Studio program and1runtime variant under cache locks, keeping
current/previous variants and the shared Cargo target; no cold rebuild afterward.

NEXT implement actual native hover, completion/signatures/snippets and fast
feedback. Useful localization already read: GPUI Base InputBaseState has an existing
HoverProvider in patched-dependencies/gpui-base/src/input/editor/lsp/hover.rs;
provider is missing from std:ui CodeEditor integration. Native hover waits150ms
before querying; keep cancellation/stale-document safeguards. Existing LSP main.rs
has ANSWER_WITHIN=3seconds and defers imported-document queries without a current
project index. Documents::index (src/lib.rs) calls SourceIndex::new; that constructor
synchronously calls cache_checked_views/compiler check, unsuitable for an always-
fast request path. SourceIndex::for_project already builds without that local
check. Build/use a current-version lightweight header/symbol index for fast answers,
while full checking continues independently. Do not invent ownership/flow facts or
serve stale replies. Reuse the native hover Markdown/popover, not a custom website
popup. Studio's Tsrust service currently transports diagnostics only; add correlated
read requests/results and checked native provider hooks, with shared executable AI
action routes and host grants. Integrated agent hosting remains pending separately.
Keep native Tree expansion on edits as a UI follow-up. Full goal remains active.

Real LSP diagnostics — connection/Problems slice accepted, follows2081e21f.
Studio now has a Tsrust-authored stdio client (services/lsp), independent bounded
pipe reader/writer tasks, UTF-8 Content-Length framing, initialize/open/full-change/
close lifecycle, monotonic document versions and connection generations. Problems
uses lsp:tsrust independently of manual checks. UTF-16 positions convert to scalar
columns. Stale/session-mismatched/closed-file or unversioned reports are discarded;
edits and closes clear previous reports. Status menu exposes real state, Problems
and a restart action. No background selection/focus changes. Server applyEdit is
rejected for review; unknown requests receive MethodNotFound. Inputs are bounded
(8 KiB headers,16 MiB incoming,128 MiB outgoing/queued,128 queued writes); writes
have15s timeout, initialization15s. The paired SDK sibling tsrust-lsp is used;
programs from project configuration are never executed. Root launch args now
canonicalize before services start. Standard-library process/io/futures only;
no new native adapter, installed dependency or SDK change.

Focused public tests5/5PASS447.99ms, /tmp/studio-lsp-tests.log: real server unsaved
error/correction/restart, Unicode/version filtering, framing rejection/EOF,
controller stale-edit/close behavior, executable action grant/root/generation
checks. The AI status/restart adapter shares the UI action, but integrated model
hosting/registration remains pending; this is not full AI parity acceptance.
Services run as independent jobs, joined by one managed owner for cancellation
and settlement. Inputs are cloned before lazy task construction, preventing the
legitimate unused-task warning on a later fallible argument evaluation. Controller
is separate from the small status component. The real-server test exercises this
exact three-service startup/cleanup, not only an isolated LSP worker.

Native acceptance PASS after repairing a startup stack overflow. Initial inline
all([worker tasks]) composed large unoptimized futures and aborted a Tokio worker;
The macOS crash report Tsrust Studio-2026-11-11-193422.ips identifies task startup.
Starting workers independently before joining their Jobs fixes it without changing
thread-stack limits or the SDK. Affected startup witness added to the same public
suite. Final native build PASS130.604s:check63.9152s,emit3.1418s,backend30.8ms,
native/link25.1845s,runtimehit (runtime prepare8.6586s); wall includes queued work,
not a benchmark. Initial runtime miss build140.197s/setup50.5433s was NOT accepted
because its program crashed. No full compiler/LSP matrices or broad benchmarks.

Old supervisor97262/PTY44811 was checking an obsolete snapshot; replacement queued
before it stopped. CURRENT supervisor15484/PTY94187, app17336; keep it running.
Bundle /var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-15484-1794443929169414000-0/Tsrust Studio.app.
Live MAT: LSP connected; unsaved syntax probe in terminal-panel.tsrx produced a
source-labelled Problems row; clicking it highlighted the offending line. Undo
restored the exact clean buffer and cleared the old report, without disk writes.
Status-menu restart replaced server17340with17521 and reconnected. Restored panel
hidden, original terminal-panel tab active,9clean tabs/zoom-1/recovery drafts[].
CUA ended "Verified live LSP diagnostics". Tree expansion collapsed during editing;
track native Tree state preservation separately. Existing image resize acceptance
retained; this section did not repeat unchanged image or formatter matrices.

Performance acceptance remains PENDING. Initial Studio workspace analysis used one
LSP core (343952KiB RSS observed); live diagnostics were delayed during that scan.
After restart, LSP17521was still scanning at100%CPU/146384KiB RSS after35s; app
17336was5.9%CPU/74192KiB RSS. These are point observations, not leak/idle benchmarks.
Brief1s sampling of the old frontend showed repeated error-effect annotation,
220.2MiB footprint/465.9MiB peak. Before broad intelligence work, investigate this
real-source latency and timer polling cost; do not claim the tiny fixture proves
interactive Studio analysis. Whole formatter0files/0snippets/0fingerprints and
diff check passed before checkpoint. Pruned11obsolete Studio program variants,
3runtime variants and1old build workspace under nonblocking cache/build locks;
kept the current app/test/runtime variants and shared Cargo target. No cold build
after cleanup. NEXT compiler/LSP hot-path investigation,
then hover/completion/snippets and generic server registrations/settings. Native
squiggles, unversioned/project report policy, graceful LSP shutdown and full agent
hosting/registration remain required. No push.

Image resize flicker — accepted 2026-11-11, follows32082607.
Two defects fixed. Reactive row shape compared live zoom/aggregate paragraph text,
remounting the whole preview as its percentage changed. Shape now compares static
text scaffolding, binding markers and child structure, retaining live text nodes.
Native ImageViewport also loaded the resource again for painting and treated
pending decode/zero-size setup as errors. Private ImageAsset retains one decoded
Arc per mounted preview; painting uses ImageSource::Render. Source changes/unmount
release owned refs. Pending/setup stays blank; genuine load errors still show alt
text. No artificial resize delay or public API/AI action change.

Evidence: initial lifecycle reproduction FAILED (preview identity6instead3),
/tmp/studio-image-lifecycle-repro.log. Fixed affected branch lifecycle8/8PASS0.01s,
3.81s setup, /tmp/studio-image-lifecycle-tests.log (existing row/tab/local-state
lifecycle cases included). Conditional import corrected its non-GPUI unused-import
warning before the final SDK build. Native cache1/1PASS0.00s,1m31s setup,
/tmp/studio-image-resize-test.log:128repeat resolves never invoke loader again;
pending/errors distinct; source changes/unmount release owned refs. First isolated
native test build accidentally used debug symbols; canceled after42s before test
execution, reran with documented DEV/TEST_DEBUG=0. No full compiler/LSP matrices.

SDK pair PASS2m35s, /tmp/studio-image-resize-sdk.log; atomically installed and
matched target hashes: CLI884f9efce2aebec3fd27eed325d1529ef45e1aaa931f868aa3b5a05d8a35a3b1;
LSP444368cec9e4c00a60017e1129292cce9e2ea0dfee1ee77b806c24252f732de8. Dev
replacement queued before old71128/PTY31480 stopped, with empty recovery drafts.
New native app build PASS119.678s:check47.410s,emit2.8684s,backend163ms,
native/link68.6824s (runtime rebuild48.7827s); dev observation, not benchmark.
CURRENT supervisor97262/PTY44811, app98817; keep one preview running. Bundle:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-97262-1794440871066896000-0/Tsrust Studio.app.

Live PNG MAT: four successive split-divider drags changed Fit40→25→54→20→40%;
image visible in inspected frames, no fallback/error text observed. Actual100%
rendering and Fit40% both worked afterward. Original split position restored;
icon.png left/problempanel source right,10clean tabs,0errors/0warnings. CUA ended
"Verified stable images across resizing". No per-frame recording or non-PNG/web
resize acceptance claimed. Cache is one decoded image per open/mounted preview;
global GPU-cache lifetime/resource budgets remain part of later resource acceptance.
Required formatter/idempotence0files/0snippets/0fingerprints; diff check passed.
Pruned1old program and1old runtime variant under cache locks, keeping current
preview/recent caches and the shared target. No cold rebuild after cleanup.
NEXT real LSP/hover/completion/snippets, known DialogFooter slot and dense-layout
minimum-width follow-ups. No push.

Floating image controls — accepted 2026-11-11, follows70148fc1.
ImagePreview now renders the image first across the full relative pane, followed
by compact absolute top-right controls. Removed full-width row height/padding and
bottom border. No backplate/frame introduced. Existing zoom/fit/accessibility/
guarded AI view actions unchanged; no SDK/native runtime changes.

Live PNG MAT PASS: toolbar is floating with no full-width separator/reserved row;
Zoom in changed40%Fit→50%, Fit restored40%; controls stayed clickable. Left icon.png
active in the first editor split, existing Problems source in the second. All10tabs
clean,0errors/0warnings; terminal/chat still hidden, original docking settings
preserved. CUA ended "Floated image controls without borders".
Auto dev build PASS122.024s:check45.893s,emit2.8492s,backend180.2ms,native/link23.7084s,
runtimehit; wall includes queued work, not a benchmark. CURRENT supervisor71128/
PTY31480, app94501; one preview, no supervisor restart. Bundle:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-71128-1794439679671329000-0/Tsrust Studio.app.
Required whole formatter1file/0snippets/0fingerprints; final idempotence0/0/0 and
diff check passed. Pruned2obsolete program-cache variants, keeping current ones. No automated regressions for this style-only change; prior image behavior
acceptance retained. NEXT real LSP/hover/completion/snippets, plus known DialogFooter
slot and dense-layout minimum-width follow-ups. No push.

Single workbench dividers — accepted 2026-11-11, follows1c67f8c2.
Native Resizable handles already paint the divider for their axis/display order.
Removed extra Region left/right/top wrapper borders, activity-bar left/right
borders and terminal session-rail left border. Native Dock split frames already
have no extra borders. Three Studio style files only; no runtime/SDK/API changes,
no action/AI semantics changed. Native divider remains the resizable hit target.

Live placement MAT passed for single boundaries:
A navigation/activity left, terminal bottom, chat right;
B navigation/activity right, terminal left, chat bottom;
C navigation left/activity right, terminal right, chat left.
Editor splits and terminal session-rail divider also inspected. Restored persisted
owner settings left/left/bottom/right and original terminal/chat hidden state;
all10tabs clean,0errors/0warnings, recovery drafts[]. CUA ended "Verified single
dividers across placements". Dense C layout exposed an existing nested minimum-
width overflow/squeezed editor with every panel open; broad tiny-window layout
acceptance is not claimed. Keep this as a layout follow-up, not a border regression.
A shell startup pyenv rehash warning was visible; no owner environment changes.

Auto dev build PASS107.769s:check45.834s,emit2.8572s,backend153.3ms,native/link23.0319s,
runtimehit. Wall includes queued preparation; not a benchmark. CURRENT supervisor
71128/PTY31480, app89605; one preview, no supervisor restart. Bundle:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-71128-1794439161195846000-0/Tsrust Studio.app.
Whole formatter0files/0snippets/0fingerprints, diff check passed. No automated
regression for this reversible styling-only change. NEXT real LSP/hover/completion/
snippets and known DialogFooter slot follow-up; no push.

Import formatting rules — accepted 2026-11-11, followsc0046c0c.
Owner requested std: imports first, one blank line before other imports, path
sorting, and wrapping >5 bindings. Shared compiler formatter organizes consecutive
import declarations as whole source slices. Binding order, aliases, literal paths
and attached comments remain intact; bare side-effect imports are order barriers.
Lists above5named bindings wrap one binding per line, including inline comments.
organizeImports defaults true; importWrapThreshold defaults5, accepts0–100,0off.
Studio global/project models, GUI/settings persistence and existing formatting/
CLI-task/guarded AI adapters share both fields. Actual AI hosting and Studio LSP
transport remain unimplemented; this change does not claim either acceptance.

Focused evidence (no full regression/performance run):
- Internal grouping/sorting/wrapping/token-multiset/options/barrier/idempotence
  witness1/1PASS0.01s,3m58s setup, /tmp/studio-import-rules-test.log. Initial test
  compile used an incorrect crate self-reference; corrected before execution.
- Existing formatter entrypoints/options/stdin3/3PASS0.79s,2m25s setup,
  /tmp/studio-import-existing-format-tests.log. Inline-comment wrapping then
  tightened; affected public witness1/1PASS0.00s,2m24s setup,
  /tmp/studio-import-comment-test.log. Unchanged passed checks not replayed.
- LSP stdio grouping/wrapping/idempotence1/1PASS0.40s,1.39s setup,
  /tmp/studio-import-lsp-test.log. First fixture incorrectly used languageId tsrx;
  existing registered LSP ID is tsrust for the .tsrx path; fixture corrected.
- Public std:testing settings persistence/partial project overrides1/1PASS2.22ms
  test execution (setup not instrumented), /tmp/studio-import-settings-test.log.
- Installed CLI custom organize=false/threshold2 via stdin:PASS plus idempotence,
  no files written. Locale258keys/2catalogsPASS. Diff checkPASS.

Studio57files intentionally organized. Required whole formatter356files/192doc
snippets/64benchmark fingerprints; repeated pass0/0/0. Checkpoint tool explicitly
sets organizeImports=false to preserve benchmark import-evaluation order/token
identities; wrapping stays on. Extra diff review proved tokens/comments identical
for410format-only sources, allowing only header import-declaration reordering in
Studio. Four Studio settings/test implementation files reviewed separately.
Final normal SDK build2m57s after embedded stdlib/docs whitespace updates;
/tmp/studio-import-final-sdk.log. Pair atomically installed and independently
matched target hashes: CLI308e0cba820750051fbb318c601b27bf8b9e3661bb00cd541c1b0ff9deac5d83;
LSPe4fd6544c161a6c8cd499d6927b67afd4f7de877a92b802fae21486e28d721d3.

Live MAT PASS: split check-panel/problems-panel show std-first/path-sorted imports,
group gap and wrapped lists. Settings controls on/5; increment6,restore5,toggle
off/on,Reload retained on/5; independently read persisted values. Settings window
left on Formatting rules. CUA ended "Verified import rules and preferences".
Owner's format/check on-save preferences remain on. Recovery drafts[]; all10tabs
clean,0errors/0warnings after automatic reload. An earlier transient7Problems
state disappeared on reload; no claim about unrelated persisted diagnostics.
CURRENT supervisor71128/PTY31480, app79429; keep one preview. No supervisor restart
needed for formatter-only compiler changes; normal dev source reload succeeded.
Bundle:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-71128-1794438364788629000-0/Tsrust Studio.app.
Last auto reload85.887s:check49.7471s,emit2.9708s,backend60.2ms,native/link32.5896s,
runtimehit. Development observation, not a benchmark. Pruned50obsolete native cache
variants under nonblocking cache/build locks, retaining current app/test programs,
their two recent runtimes/workspaces and the shared Cargo target. Studio private
cache13GiB→2GiB (rounded du observations). No cold rebuild after cleanup.
NEXT real LSP/diagnostics/hover/completion/snippets before Git/debugger/extensions;
known DialogFooter native footer-slot follow-up remains below. No push.

Panel tab underline — accepted 2026-11-11, follows191e9a0b.
Owner wants the active underline across the entire padded tab, flush with the
panel's left border. Removed Studio's spacer-only TabsPrefix. Shared Underline
variant now uses normal per-size inner label padding and zero inter-tab gap,
instead of unpadded labels with external gaps. Indicator tracks full tab bounds.
Live macOS MAT PASS: Terminal underline starts exactly at panel content edge and
extends beyond both sides of its label; selecting Problems moves the full padded
underline to that tab. Text padding preserved; actions/copy/close stay at right.
CUA phase ended "Extended underlines across padded tabs". Problems selected at end.
No command/state/AI semantics changed. No automated suite for style-only change.

SDK pair build PASS2m44s, /tmp/studio-tab-underline-sdk.log; atomically installed.
Dev build PASS286.528s:check46.6555s,emit2.8551s,backend173.9ms,native/link236.278s,
runtime rebuild216.2542s. Native dependency recompilation, not test time/benchmark.
CURRENT supervisor71128/PTY31480, app71315; one preview, GUI launch with -u NO_COLOR.
Bundle:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-71128-1794436930520486000-0/Tsrust Studio.app.
Old64691/PTY94719 stopped. All10tabs clean,0errors/0warnings, recovery drafts[].
Whole formatter:0files/0snippets/0fingerprints; diff check passed. Keep dev running.
NEXT remains real LSP/hover/completion/snippets before Git/debugger/extensions;
known DialogFooter native slot follow-up remains below. No push.

Settings TSRX terminology — 2026-11-11 (after6453782e).
Owner corrected the visible “Parenthesize JSX returns” wording to TSRX and noted
future per-language rules. Settings title/accessibility name and both locale
catalogs now say “Parenthesize TSRX returns”. Existing jsxReturnParentheses config
key remains compatible; no formatter behavior/schema migration. Language-specific
format/lint overrides recorded as planned in Studio plan, not implemented here.
Locale coverage PASS2catalogs/253keys. Dev94719 watches the UI change; no SDK rebuild
or automated regression needed for label-only correction. NEXT whole formatter,
local checkpoint; preserve current preview and pending LSP priority.

Terminal session row alignment — accepted 2026-11-11, follows80a40f04.
Owner wants action buttons anchored right and long labels ellipsized. Shared native
ListItem content now uses flex_1/min_w_0 and a nonshrinking suffix; std:ui content
row fills the constrained width. Existing Studio min-w-0/truncate label gets the
remaining space. No command semantics/AI action changes. Live MAT compared Terminal1
and Check project rows at120pt/180pt/270pt rail widths: both ellipsis/trash columns
stay aligned at the right edge, long labels ellipsize, full labels return when
widened. Left the rail wider for inspection. CUA phase ended "Aligned terminal row
action buttons". Check project was started through its real status command as the
second row; still running when layout MAT finished, do not claim its result.

SDK pair build PASS2m39s, /tmp/studio-terminal-alignment-sdk.log; atomically installed.
Dev build PASS289.433s:check46.9594s,emit2.8636s,backend174.6ms,native/link238.8796s,
runtime rebuild218.8535s. This included native dependency recompilation, not tests
or a benchmark. No automated regression rerun for this style-only change.
CURRENT supervisor64691/PTY94719, app64958; one preview, GUI launch with -u NO_COLOR.
Bundle:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-64691-1794436254281332000-0/Tsrust Studio.app.
Old58640/PTY9543 stopped. All10tabs remain clean; recovery journal drafts[].
Whole formatter:0files/0snippets/0fingerprints; diff check passed. Keep dev running.
NEXT remains real LSP/hover/completion/snippets before Git/debugger/extensions;
known DialogFooter native slot follow-up remains below. No push.

Terminal color correction — accepted 2026-11-11, source checkpoint follows815d4989.
Owner reported CLI colors missing. Live native terminal probe proved ANSI red/green
rendering already worked; TERM=xterm-256color,COLORTERM=truecolor, but NO_COLOR=1
was inherited from the Codex launch. No owner shell profile sets it. Fix automation
launch with env -u CODEX_TOOL_CALL_ID -u NO_COLOR, not a global PTY override:
deliberate user terminal preferences still apply. README documents this launch.

CLI project-check success and formatter batch summaries/progress also ignored
--color=always. Existing stream color policy now styles progress cyan, success
green, needs-format yellow and partial-failure summaries red. JSON and raw source
remain ANSI-free. Focused host CLI test1/1PASS1.18s execution,2m32s setup,
/tmp/studio-cli-color-test.log; one existing TempDirectory fixture/public commands,
no native programs per case. No full regression. Normal CLI/LSP build PASS14.23s,
/tmp/studio-cli-color-sdk.log. Both atomically installed; SHA256 CLI
8f05b6d062655faf5487f3607014d7b738717cb8bf5403d9718b033e125b78e8,
LSP ad9b6cb09f1ec0cbf6b1456f2217d4896e0dd74ba5aac95d33752fc4373673bd.

CURRENT supervisor58640/PTY9543, app60228. One preview, launched require_escalated
with -u NO_COLOR. Bundle:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-58640-1794435805963737000-0/Tsrust Studio.app.
Dev build PASS122.269s:check46.9036s,emit2.8711s,backend149.5ms,native/link71.7837s,
runtime cache miss/runtime build50.9825s; development observation, not benchmark.
Live fresh shell showed NO_COLOR=unset, cyan automatic tsrust --help headings and
cyan read-only format --check src/lib/workspace/zoom.tsr --progress output.
CUA phase ended "Verified automatic CLI terminal colors". All10tabs clean,
0errors/0warnings; recovery journal drafts[]. Keep this preview running. Old
supervisor44454/PTY91897 stopped. No terminal parser/palette changes needed.
Whole formatter/idempotence:0files/0snippets/0fingerprints; git diff check passed.

NEXT: real LSP connection/diagnostics/hover/completion/snippets before Git/debugger/
extensions, per owner. Shared DialogFooter slot placement remains a known bounded
layout follow-up: custom footer is appended inside dialog.content instead of native
Dialog.footer, allowing save-conflict action clipping in small windows. Owner's
four obsolete drafts were explicitly reconciled; all files saved. Do not restore
those old drafts. Keep one current dev preview; restart permission is standing.

Latest accepted image/Save All section — 2026-11-11, 16:48.
Source checkpoint followsc87c2019; current section ready for local commit, no push. Last UI-only
localization correction translates the100% label (2catalogs/253keys). Its dev reload
PASS129.011s:check48.2769s,emit2.887s,backend178.3ms,native/link25.743s,runtimehit;
wall time includes queued work, not a quiet benchmark. All buffers were clean before
this automatic reload; journal remains drafts[].
CURRENT dev supervisor44454 / PTY91897, latest app53634; keep it running. Bundle:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-44454-1794434802879116000-0/Tsrust Studio.app.
Launched require_escalated: the sandboxed launch built but NSPasteboard returned
NULL/macOS XPC access failed. Normal GUI launch works. Standing restart permission
applies; queue replacement before stopping old. Waiting with old supervisor active
did not avoid a blank interval; do not assume concurrent preparation is available.

ImagePreview is extracted from editor.tsrx. Default Fit never enlarges, follows
pane bounds; controls offer zoom,100% natural size and Fit. Cmd+=/-/0 route to image
zoom/Fit while an image is active; otherwise normal persisted text zoom remains.
New optional std:ui Image.zoom (0Fit or1–3200%),scrollTo JSON{x,y,request} and
onZoomChange use the GPUI asset cache, measured viewport and one ScrollHandle for
native wheel/scrollbars and programmatic pan. Canvas cannot flex-shrink; scrollbar
overlay belongs to the fixed outer viewport. Initial MAT caught horizontal shrink
and floating bars; final native layout fixes are now built and visually accepted.
services/image-actions.tsr exposes guarded executable set_image_zoom/pan_image
with explicit path/session; no selection, document revision or file changes.

File→Save All/Cmd+S queues every dirty text file, including JSON/non-active tabs,
through services/save-actions.tsr. save_files uses the same executable queue with
host-only live-save permission and explicit path/session/revision targets. Per-file
format/proposal/CAS/recovery remain intact; stale targets reject without retargeting.
One conflict does not stop later saves; per-file errors remain in Problems. One
check-on-save runs after queued saves and any earlier check finish. saveTarget
replaces saveActive. New image state is ephemeral OpenFile view state, not file data.

Focused runtime2/2PASS24.31ms (image actions + Save All temp-file conflict/later JSON
save/stale target/denied grants), /tmp/studio-image-save-tests.log. Geometry and
invalid Image prop guard1/1PASS0.00s execution/3.74s setup,
/tmp/studio-image-geometry.log. No unchanged suite replay/full matrix. Final SDK
normal pair PASS2m45s /tmp/studio-image-scroll-layout-sdk.log, atomically installed:
CLIe0c1d71b0b2c77a74c6ae1eb1d4f2da0d8a25cfe6c78fa59e325f96af15ad395 /
LSPad9b6cb09f1ec0cbf6b1456f2217d4896e0dd74ba5aac95d33752fc4373673bd.
Dev native gate PASS122.561s:check48.9842s,emit2.8864s,backend172.1ms,
native/link69.9555s including runtimebuild49.283s/cachemiss. Operational observation,
not a comparable benchmark/speedup claim. Do not repeat a standalone tsrust build
for this section; its earlier pass did not warm the dev runtime variant.

Live PNG MAT PASS: owner10open files/4dirty drafts recovered. Whole-image Fit64%,
Actual100%, keyboard125%/100%/Fit, vertical wheel pan, horizontal scrollbar thumb
pan and fixed right/bottom bars. Opening Terminal shrank pane and Fit became48%
with full image still visible; closed panel afterward. File menu shows Save All.
Owner exercised Save All, producing4protected conflicts. Owner then explicitly
asked the agent to decide what to keep and save the actual files. Compared every
recovery draft against its saved baseline:3formatting-only changes; editor.tsrx had
formatting plus one // test marker. Formatting both variants with the public CLI
proved equivalence after excluding that marker. Kept newer disk implementations
(ImagePreview/Problems/check/terminal updates) and used Discard draft and reload
for all4reviewed conflicts. Native counters reached0errors/no unsaved menu, no dirty
tab markers; recovery journal independently confirms drafts[]. No newer source was
overwritten. The last2dialogs had clipped footer buttons at normal height, so the
window was temporarily zoomed to reach them. A restore-window action was rejected
because the user changed the app; re-queried and left their current window state.
Final fresh AX confirms10clean tabs,0errors/0warnings and no remaining dialogs;
CUA final phase ended "Verified all tabs are saved". All10tabs remain open.
The save-all witness was strengthened to ensure clearing one file's formatter
error preserves another:1/1PASS27.55ms /tmp/studio-save-all-problems-test.log.
 CUA horizontal wheel injection produced no
visible movement; hardware trackpad horizontal gesture not claimed verified.
Scrollbar pan works. An accidental sidebar drag during MAT was restored to its
original592physical-pixel boundary. Left icon.png at Fit64%, all4drafts preserved during that earlier image MAT, then reconciled as above.
Other image decoders and web viewport behavior not separately checked. CUA phase
ENDED "Verified image zoom and panning"; reacquire actual .app path after reload.

Whole formatter + idempotence PASS0files/0snippets/0fingerprints,
/tmp/studio-image-save-checkpoint-format.log; generator--check,2locales/253keys,
diff-check pass. README updated. Removed superseded standalone validation output
apps/studio/target/studio (16:16 build, predates final layout); active dev
bundle/shared caches retained. NEXT: local checkpoint, then real LSP connection/diagnostics/
hover/completion/snippets before Git/debugger/extensions, followed by integrated AI.
Known narrow UI follow-up: std:ui custom DialogFooter is currently appended inside
Dialog.content in gpui_kit.rs instead of using the native Dialog.footer slot. This
lets long conflict content clip its actions at shorter window heights. Fix that
shared composition rather than shrinking each caller's Diff with arbitrary vh
values; existing GPUI dialog footer-bounds tests cover the native slot. Data is
already reconciled; do not recreate conflicts on real files just to test layout.
No LSP transport yet. Existing executable AI adapters are plumbing; built-in agent
hosting/action registration acceptance remains open. Never mark full goal complete.

Latest small owner correction: .tsrx file icons now share .tsr's #CE7045 tint;
their distinct glyphs remain. Existing file-icon suite2/2PASS0.33ms execution;
whole formatter0files/0snippets/0fingerprints and diff-check pass. Dev supervisor
23759/PTY74802 remains running and watches the catalog change. Visual reload not
separately inspected for this two-line catalog/assertion edit. Previous full
save/diff/status section checkpoint is f1b158da; LSP remains next feature priority.

Latest owner-feedback acceptance — 2026-11-11, 15:42.
The apparent double titlebar was SaveConflicts' empty normal-flow View placed
before Chrome. Its overlay host is now absolute/size-0; native creation options
are unchanged. Live screenshot confirms a single36px custom row, sidebar/settings
aligned with the traffic-light area. Terminal/Format/Check share one gap-1 row
and equal size-5 frames. Errors, warnings and unsaved counts have explicit icon
spacing; errors/warnings independently display truthful zeros. Warning counter
clicks open the same Problems surface. Structured compiler warning preservation
1/1PASS0.87ms execution, /tmp/studio-warning-report-test.log. No full regressions.
Whole formatter PASS2files then idempotence0files/0snippets/0fingerprints;
generator--check, locale2catalogs/235keys and diff-check pass. Live MAT retained
all8open files/4dirty drafts/zoom-1. Supervisor23759/PTY74802 remains running;
app30137, bundletsrust-23759-1794429655350340000-0/Tsrust Studio.app. Dev build
123.261s (check45.7174s,emit2.8833s,backend226.7ms,native/link24.5143s,runtimehit),
an observation under concurrent SDK work, not a comparable benchmark claim.
CUA phase ended "Fixed title-bar row and spacing". No owner drafts discarded.
An exploratory native titlebar-options edit was reverted after finding the actual
layout cause. Final-source normal CLI/LSP pair build PASS2m42s,
/tmp/studio-chrome-sdk-final.log. Both rebuilt binaries exactly match the installed
verified01b1b68b4fcc436cd59bb96912c51030a39728d5f4d9b45dc8de0127db98c778 /
f1f33fff1c4caa80744f8f599cb8040f84e749091f711597708bd72a309f18a8 pair; no
installed refresh/restart or unchanged-suite replay needed. Source is ready for
the local save/unsaved/diff/feedback checkpoint followingfa0d3634. LSP/hover/
completion/snippets remains next implementation priority; no transport implemented
yet. Keep the current single preview running; do not discard owner drafts.

Latest live acceptance — 2026-11-11, 15:24. Current save-conflict/unsaved/diff/
status section is implemented and native-MAT verified, still UNCOMMITTED after
fa0d3634. No LSP transport implementation has begun yet; that remains the immediate
next priority before Git/debugger/extensions, followed by built-in AI.

SDK final normal pair PASS2m46s, /tmp/studio-comparison-sdk-final.log. Installed CLI
01b1b68b4fcc436cd59bb96912c51030a39728d5f4d9b45dc8de0127db98c778,
LSPf1f33fff1c4caa80744f8f599cb8040f84e749091f711597708bd72a309f18a8.
Native app build PASS, /tmp/studio-comparison-native.log. Queued replacement BEFORE
stopping17056. CURRENT supervisor23759 / PTY74802; last observed app23896. Live
bundle (rediscover before any CUA after reload):
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-23759-1794428404528944000-0/Tsrust Studio.app.
Dev build68.039s, check42.5274s, emit2.7493s, backend0.1493s,
native/link21.9589s, runtime cache hit. This is an observation, not a comparable
benchmark/speedup claim. Keep one current preview running. After MAT a tiny source
change explicitly hides the native List's duplicate empty illustration in Problems
(ListEmpty/View, same pattern already verified in Terminal). Dev may auto-reload;
rediscover PID/path instead of using old CUA binding.

Live MAT PASS:8open files/4dirty drafts/zoom-1 recovered. The owner enabled format
on save, so Cmd+S first ran async formatter, then CAS correctly opened conflict
comparison rather than overwriting changed disk. Shared split Diff visibly shows
red/green line backgrounds and inline word emphasis, compact numbers, accessible
Original/Modified line metadata, expandable unchanged context and syntax coloring.
Clicked ONLY Keep editing, never overwrite/reload on owner files. Formatting may
have adjusted whitespace in that draft; its semantic changes remain protected.
Unsaved-file popup now has editor-sized text and appears on left with Problems,
LSP and current-file state. Terminal/format/check controls on right. Copy All
Problems verified adjacent to Close in panel header. Clipboard byte-copy itself
not read back in MAT; executable action/denied-host-grant model check passed below.
CUA phase ended "Verified diff highlighting and status layout"; no active CUA phase.

Focused native snapshot comparison1/1PASS0.00s execution,3m20s setup,
/tmp/studio-text-comparison-test.log; no full GPUI/compiler/LSP matrix. Test checks
real changed-line counts/retained context, Unicode and quoted/control-character
paths, equal snapshots, empty files and missing-newline changes. Compact final
Diff gutter arithmetic and doc-comment cleanup followed this pass; native app
compilation and live visual MAT cover that style-only delta. The tracked component
Cargo.lock's2temporary local-patch substitutions were restored to HEAD after test;
no dependency addition/network, no lockfile change to checkpoint. Prior wording
calling this lock ignored was wrong. Runtime6formatting,4storage,1document-action
passes retained below; no rerun of unchanged executables.

Latest whole formatter PASS0files/0doc snippets/0fingerprints, generator--check,
2catalog/233locale keys and diff-check PASS before the tiny ListEmpty source edit.
Run the mandatory formatter + source idempotence after that edit, update final
handoff/checkpoint identity, then local commit. No push/release. Reuse passed tests.
Review/prune exact obsolete native outputs while retaining current runtime/apps;
prior scoped cleanup reclaimed15GiB. Do not blindly clear shared Cargo target.

Latest continuation — 2026-11-11, 15:05. Uncommitted section expanded by owner
live feedback: unsaved-file menu should be left with diagnostics; actions including
terminal toggle on right. Popup text must follow font/zoom. Comparison must actually
highlight changes; previous two read-only CodeEditors had no diff highlighting.
Line numbers/gutter looked too large throughout app.

Prior direct feedback source checks PASSED; native dev reloads now confirmed.
Supervisor8979 eventually finished its pending build before restart (check44.4088s,
emit2.7423s, native/link23.9146s,total116.662s). One-second sample caught checker
work, but this is NOT proof of a stuck/infinite cache or root-cause diagnosis.
Fresh supervisor17056/PTY11398 built in45.554s, check41.9279s, emit2.7321s,
cachehit137.8ms/no native link. App17091, bundle historical reference:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-17056-1794426374251758000-0/Tsrust Studio.app.
Keep it running, but new SDK must replace supervisor to support new Diff/Menu props.

Live MAT PASS on this app:8open/4drafts retained; filled-circle dirty marker in
Tree and tabs; Unsaved files4 status popup lists4relative paths. Real File menu
now separate from macOS app menu. Cmd+S on protected owner editor.tsrx draft opened
plain-English conflict Dialog showing disk/draft and Keep editing / Discard draft
and reload / Overwrite with my draft. Clicked ONLY Keep editing: no owner draft or
disk discarded/overwritten. CUA phase ended "Verified conflict choices preserve
drafts". User then requested true diff emphasis, compact gutters, smaller popup
text, left diagnostics/right actions. Do not reuse CUA handles after replacement.

Current source implements that feedback: shared GPUI DiffFile::compare_texts uses
existing similar2.7 Patience/1s deadline/full context, quoted paths and parser;
DiffState compares on background executor with generation guard/cancellation,
loading spinner/errors. New std:ui Diff originalText/modifiedText/filePath/fontSize
props (patch is alternative, cross-field guards in native install); native renderer
tracks input identity and uses shared comparison. Diff code/gutter font can follow
Studio zoom instead of fixed theme size. Conflict Dialog now uses actual split
Diff with word-level/background emphasis and expandable unchanged context.
DropdownMenu/ContextMenu fontSize8..40 new capability; PopupMenu respects size and
scales row height. Source status diagnostics/dirty menu/file state left, actions/
terminal right; unsaved menu passes font.get(). Editor line numbers use85% of code
size and reserve2rather than3digits for small files; folding hitbox retained.

Normal SDK pair after these new capabilities PASS2m45s,
/tmp/studio-comparison-sdk.log (PTY1621complete). NOT YET installed or loaded.
Current installed pair still8c54844d.../75286aaa... from older section below.
Focused shared Diff unit test is RUNNING PTY16063,
/tmp/studio-text-comparison-test.log. It compiles the component's existing focused
workspace offline, shares root target and disables incremental/debug symbols.
Command: cargo test --offline --manifest-path
crates/tsrust-compiler/patched-dependencies/gpui-component/Cargo.toml
--config 'patch.crates-io.gpui-base.path="/Users/seancotter/Desktop/dev/learning/tsrust/crates/tsrust-compiler/patched-dependencies/gpui-base"'
--config 'patch.crates-io.url.path="/Users/seancotter/Desktop/dev/learning/tsrust/crates/tsrust-compiler/patched-dependencies/url"'
--config 'profile.dev.package.gpui-pre.opt-level=3'
--config 'profile.dev.package.gpui-component.opt-level=3'
--lib text_comparison_tests -- --test-threads=1.
Only2local patched packages were temporarily substituted in the tracked focused-test lockfile; restored afterward. no network or
new dependency. Earlier attempts root-manifest -pcomponent failed because optional
package not selected, then --locked correctly refused local-patch lock refresh.
Do not repeat those invocations. Test verifies changed-line counts, context/path
Unicode/control quoting, equal text and missing-newline changes. Runtime/native
Root bridge must still compile and real split-diff/zoom MAT is still required.

Next: inspect focused test output, repair narrow compile/errors if any; final
whole formatter/catalog/locales/diff checks; installed pair refresh; queue new dev
supervisor BEFORE Ctrl+C17056/PTY11398. Source-only current-app build may fail on
unknown fontSize until supervisor embeds new SDK; don't mistake that for sourcebug.
Use existing runtime cache/shared target. Then verify actual colored diff, compact
gutter, popup font/left-right status alignment, Copy All Problems. Preserve4drafts.
After acceptance local checkpoint, resume real LSP/hover/completion/snippets and
integrated AI priority. Full goal active; no pushes/releases/full matrix.

Latest owner steering and active correction — 2026-11-11, 14:44.
Checkpoint fa0d3634 committed the prior formatter/CLI-task/Problems section,
including mandatory whole-project formatting (1679files). New UNCOMMITTED section
fixes confusing save conflicts, project formatting requiring saves, Copy All
Problems, and more visible unsaved markers/status menu. Owner clarified ordinary
edits should save: CAS compares current disk to originally loaded disk baseline,
not to draft. Another process (including our self-hosted source changes) changed
these files, so explicit conflict choices are needed; never silently overwrite.

Implemented source: Document.saveConflict captures path/session/revision/diskText;
storage.resolveSaveConflict rejects stale drafts/unapproved actions/second external
disk edits; CAS overwrite and explicit reload preserve safety. Existing normal
save semantics retained. Kit Dialog comparison shows disk and draft side by side,
plain English explanation, Keep editing / Discard draft and reload / Overwrite
with my draft. services/document-actions registered executable list_unsaved_files,
inspect_save_conflict, resolve_save_conflict shares UI resolver and host grants.
services/problem-actions list/copy uses same live collector and guarded clipboard
operation; Copy All icon next to Problems Close. Collector moved into lib/problems.
Filled-circle markers in tabs AND Tree, status dirty-file count/relative-path menu.
New Application menu with Settings retains a separate File menu on macOS (previous
first File menu was renamed by the OS). Real UI/MAT for this new section pending.

Project Format no longer refuses dirty buffers. Shared QueuedFormatting captures
explicit session/revision targets for all open Tsrust buffers; on-focus uses same
queue with mode focus. Disk task receives explicit unopened source paths only,
with --write; canonical root/path checks prevent symlink escape and aliases to
open files. Open buffers remain unsaved proposals; none are disk-formatted.
AI format_project returns corresponding open-buffer proposals plus approved
unopened-file task output. No shell interpolation, no new CLI option/SDK rebuild.

Acceptance retained: storage4/4PASS53.32ms (including explicit overwrite/reload,
denied host grant and second external disk change), log
/tmp/studio-save-conflicts-tests.log. Last batch formatting+document-actions7/7
PASS34.50ms, /tmp/studio-conflict-actions-tests.log. Follow-up formatting6/6
PASS25.02ms after corrected fixture explicitly edits its open buffer and canonical
path exclusion hardening, /tmp/studio-project-format-drafts-tests.log. Public
project format initially hit dynamic push spread arity error; implementation now
uses ordinary loops. Regression witness passes; compiler push-spread defect itself
not repaired. Earlier temporary-path fixture mistakes were fixed to use canonical
workspace identities. These are execution times, setup not measured separately.
Locale2catalogs233keys pass; final whole formatter currently running
/tmp/studio-save-conflicts-format.log. Final root idempotence/diff check pending.

Dev worker SLOWDOWN diagnosis: supervisor8979/PTY46813 with old app9078 still
alive, but 100%CPU/~310MiB frontend kept analyzing an older snapshot for minutes.
One-second stack sample /tmp/studio-save-conflicts-sample.txt is in checker
continuation/effects/recursive-view-shape analysis, not Rust native compilation.
FRESH current-source read-only check PASS41.89s,
/tmp/studio-save-conflicts-fresh-check.log. Do not launch duplicate broad tests.
Restart queued supervisor before stopping8979 to load current source. Standing
restart permission applies. Need investigate persistent ProjectSession stale/cancel
behavior later; fresh check passing is not proof that slowdown is fixed. Current
SDK installed pair remains 8c54844d.../75286aaa... below (no SDK changes this section).

Owner priority still: after this direct feedback correction, real configurable
LSP + Problems, hover/completion/snippets, then built-in AI before Git/debugger/
extension management. No LSP connection or agent transport implemented yet.

Latest live acceptance, 14:19 — corrected JSON argument encoding is loaded.
SDK normal pair PASS2m39s; installed CLI
8c54844d081591fc355105664b08f57522299d1972d771817b154567be731454,
LSP75286aaa017dcb2685efbdd755ac9a203e409eb244264c120bf6cce9a7589e85.
Queued replacement BEFORE stopping8139. CURRENT supervisor8979 / PTY46813,
app9078, live bundle (rediscover before CUA after any reload):
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-8979-1794424644569030000-0/Tsrust Studio.app.
Dev build62.724s: check37.4627s, emit2.5012s, backend0.1724s,
native/link22.0661s, runtime cache hit. Keep this session running.

Live MAT PASS: recovered8open files,4drafts,zoom-1; active terminal-panel.tsrx.
Check Project status button starts named direct CLI terminal, visible
"Checking51 source files…" and running spinner. Manual check still running at
handoff; do not claim whole-project check passed until its exit is observed.
Independent Terminal1 retained while check task ran. Printed
src/components/tasks/terminal-panel.tsrx:4:1 in shell; Cmd-click selected existing
editor and visibly highlighted line4 through line end, without changing draft.
No giant raw error strip. CUA phase ended with completion summary
"Verified task terminals and navigation". CUA bindings MUST be reacquired after
reload. Existing settings/focus GUI interactions and long-running Check completion
still need live acceptance; model/config/safety6/6 evidence below remains passed.

Earlier live task crash was args string[] encoded via display-list join. Fixed
catalog json:true; generated wrapper uses JSON.stringify for static/reactive args,
then SDK rebuilt/installed/restarted and actual click verified. Native grid index
fix first-1usize also native-build passed. Task shells cannot bypass project-format
dirty checks via generic Restart Shell item: that item disabled for task sessions;
rerun explicit task through Project commands. No full compiler/LSP matrix replay.

Pruned85 obsolete Studio native cache entries with nonblocking locks. Studio
target18GiB→2.8GiB, preserving current runtime/current app outputs. Root shared
Cargo target still79GiB; no blind clean. Mandatory final whole formatter currently
PTY (see active tool handle), /tmp/studio-workflows-format.log. Final source
idempotence, generated catalog/locales and diff check then local checkpoint remain.
No checkpoint yet; all source uncommitted after79474f87. Immediate next product
section: real configurable LSP, then hover/completion/snippets and built-in AI,
before Git/debugger/extensions. Do not lose owner drafts during further reloads.

Active section — formatter/workflows/Problems/CLI task terminals (2026-11-11),
uncommitted after79474f87. Owner's next priority is real LSP (configurable Tsrust,
Rust/TypeScript/Go/external servers), hover/completion/snippets, then built-in AI
before Git/debugger/extension UI. See immediate-priority section of Studio plan.
Full goal stays active. Do not restart full compiler/LSP matrices.

Compiler FormatOptions implements indentWidth1..8, JSX return parentheses,
separate JSX children and blank lines around control blocks; canonical token,
JSX text and exact-comment guard; raw quoted attrs and callback comment trivia
preserved. Latest argument-layout fix prevents openWindow(<View/>,...) drift.
CLI format --stdin-filepath path --options JSON takes <=16MiB stdin and never
writes source; nearest project format fields overlay global defaults. CLI check
and format accept --progress for visible tasks. project_config accepts format and
four editor trigger flags. LSP formatter shares these rules and client tabSize.

Studio global/project rules and GUI support independent format/check on save and
editor focus loss (manual when both disabled, both when enabled). Focus formatting
keeps draft/disk distinction; explicit path/session/revision guards survive tab
switches. Focus-loss paths queue through shared Signal<string[]> and the workflow
controller's own sender endpoints, avoiding escaped borrowed channels. Save uses
existing atomic conflict/recovery boundary. AI format_document returns isolated
proposal; check_project and format_project have executable host-grant adapters.
No actual integrated AI transport claimed. Manual Project menu/status checks and
project formatting now launch named native task terminals via direct program/args,
with running/exit icons and retained output. Project format refuses dirty buffers;
a successful task reloads only clean open text buffers. Background checks remain
structured worker tasks and do not steal focus/panels. Output has cancellation,
no duplicate manual Play action.

Problems receives source-labeled structured check/formatter errors and Studio/
terminal operation errors; raw giant red strips are removed. No fake LSP link.
Problems navigation and terminal Cmd-click file links share guarded openProblem,
workspace canonicalization, Unicode scalar columns to UTF8 selection and highlight
through line end. Native terminal links handle soft-wrapped paths and emit bounded
JSON onOpenLink. Open_problem is an executable registered editor tool. Array-valued
computed is unsupported: collectProblems is pure, called from JSX/scalar count;
never restore computed<Problem[]> or a factory alias state constructor.

Latest retained acceptance: focused Rust formatting3/3PASS0.67s, setup2m37s,
/tmp/studio-formatting-final.log (includes JSX-argument idempotence, raw quotes,
Unicode attrs, comments, option toggles, stdin/project override/no write).
Earlier corpus639 lexable fixtures+86 callable contracts PASS2.35s, setup3m26s,
/tmp/studio-formatting-corpus.log; do not replay it after native-only edits.
Pure terminal file parser1/1PASS0.00s, setup4.98s,
/tmp/studio-terminal-links.log. Public affected suites together6/6PASS26.22ms,
/tmp/studio-workflows-models.log: explicit/stale/unauthorized format requests,
legacy/global/project config, save conflicts, actual CLI proposal/no disk writes,
forged project-write denial, focus draft/selection behavior, task grant/dirty guards,
Unicode byte selection, structured check diagnostics. No full regression matrix.

Whole formatter already ran1648 files+254 doc snippets+1002 fingerprints, then
latest pass39 files and one follow-up file; docs0left/fingerprints0changed.
/tmp/studio-formatting-whole.log and /tmp/studio-workflows-format.log. Expect huge
~1678-file mechanical style diff; required owner whole-project formatting, not
permission to revert. All new files git add -N. Full source idempotence exit0,
/tmp/studio-workflows-idempotence.log (before last1-file serialization formatting).
Generated226-component Kit catalog check and2locale/211key coverage PASS.
Need final formatter/idempotence/diff check after source settles. Diff-check's only
extra EOF blank in formatting/jsx.rs has been removed.

Final native/consumer status is in the latest live acceptance at the top. Earlier
intermediate failures were repaired; do not restart the obsolete supervisors or
reuse their CUA handles. The current supervisor is8979/PTY46813. Retain the owner’s
four drafts; save conflicts remain intentional protection, not permission to
replace external disk edits. Root Cargo target cleanup beyond the scoped Studio
cache cleanup has not been performed.

LSP investigation: existing GPUI base EditorState has lsp_mut hover/completion/
definition/code-action providers and diagnostics_mut. Reuse these native surfaces;
Studio's process/stdio/document lifecycle should remain Tsrust. Installed external
commands found rust-analyzer and gopls; typescript-language-server absent from PATH.
No external server or new LSP integration was started/implemented yet. Owner explicitly
wants hover, completion and snippets before Git/debugger/extensions and integrated
AI next. Future AI review/diffs/debug actions must share guarded UI command paths.

Current continuation: right terminal session rail and shared tab typography are
implemented and running in supervisor91447/PTY74141/app91525. Current bundle:
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-91447-1794416983704481000-0/Tsrust Studio.app.
Reacquire this exact live bundle from ps before CUA; never reuse a handle after
replacement. Owner's single editor pane and five files remain intact, active
editor.tsrx, no edits made by MAT. Zoom was increased once and restored to -1.

Rail uses existing List/Resizable/DropdownMenu, stable semantic session ids,
independent retained Terminal views, trash/actions on the right and New Terminal
beside Close in the panel header. No extra sparse controls row. Owner clarified
that contrasting colors are fine; remove viewport padding, retain header spacing.
Viewport is flush, header has an8px native TabsPrefix inset and one bottom line.
New Terminal button in that header created/focused shell2PID88596; shell1 remains.
Pure model and executable guarded session adapter added;
public terminal-sessions.test.tsr2/2PASS0.67ms execution, build18.064s
(check1.790s, emit0.302s, native/link15.629s). Both tab types now receive explicit
font sizing; vendor Tab preserves that size after its defaults and expands height
for larger text. Dock refreshes its groups when the shared size changes. Panel
Close lives in TabsSuffix with one native bottom border. SDK pair PASS2m41s;
native Studio build PASS, then current dev load56.116s. Installed normal pair
refreshed after density fix: CLI93b7b4ec8099f9d6cc3373fd04f9a50a33d3e0222e73962af9bce1e681cb9cf7,
LSP71ee69204c8caa634c8a0e6fee2ad91a1ad15ec49fda663a58183a9bd7485503.
Whole formatter0files/0snippets/0fingerprints, locales2catalogs163keys,
generator --check and diff check PASS. New native Tab geometry test is written
but UNRUN; do not count it as passed or launch the full UI matrix.

Live MAT: configured10px visibly reaches editor and panel tabs, both change with
Cmd+= alongside tree/editor/terminal, Cmd+- restores owner zoom. One panel bottom
line extends through Close. Shell1PID77143 and shell2PID78266 were distinct;
New focused shell2, selecting shell1 retained its output/PID. Inactive shell2 trash
removed only that session and process (ps confirmed); shell1 remained. Invalid
trash-2 icon caught visually and replaced with admitted trash. List row controls
are clickable, but nested buttons are flattened in AX; direct secondary action
Kill terminal was rejected, coordinate click worked. Accessibility remains pending.

MAT found chained native replies require another UI event: Interrupt queued after
readTerminal but sleep79429 remained at26s, then next menu event delivered Ctrl+C
before its120s deadline. Shared Root.reply now defers an extra notify until after
the current frame, so reply-created commands can run without another event.
SDK pair PASS2m47s, /tmp/studio-terminal-reply-sdk.log; native dev load98.914s
(runtime compile/publish45.483s, check32.570s, emit2.233s). Header source rebuild
72.941s (runtime hit, check32.041s, emit2.229s, native/link19.112s); source timings
are build observations, not comparable benchmarks. Fixed-reply MAT: shell84332
started sleep85471 with120s deadline; ps confirmed it live at15s. Menu Interrupt
then showed Ctrl+C/prompt immediately, ps confirmed sleep gone and shell retained,
without another UI event. Earlier shells77143/78266/job79429 were also gone after
restart. Final header inset/flush viewport/new button observed in app87441, two
shells left available. Installed pair refreshed, formatter0/0/0 and diff check pass.
No full regressions. Native Tab test,
shell restart/active removal/rail resize, SIGTERM/HUP-ignoring-job cleanup and
actual integrated AI transport remain pending. Full Studio goal stays active.

Final owner density followup: ListItem wrapper had ignored row utility styles.
It now refines the native row style; terminal rows use py-0, a square edge and a
subtle bottom divider. Native MAT app91525 shows two compact rows, separators and
unclipped controls. New Terminal still creates/selects the second shell. Native
header inset, flush viewport and same-row plus/close all remain visible. No more
source edits after that MAT. Normal SDK pair PASS2m47s; dev native build PASS97.621s
(runtime45.259s, check32.051s, emit2.159s, native/link62.779s includes runtime).
Whole formatter0/0/0. Installed CLI/LSP match the above normal pair. No full suites.

The following first-integration notes are retained evidence; current identities
and the session-rail implementation above supersede their live-state references.

Current terminal continuation — first native integration, before the owner's new
multi-session rail and typography requests. Full Studio goal active. New std:ui
Terminal wraps alacritty_terminal0.26.0 through compiler-owned native modules;
checked controlTerminal/readTerminal, opt-in marker/runtime feature, catalog/docs,
HTML unsupported output, generation guards, typed guarded terminalTools catalog
and callable adapter are implemented. Studio's initial single-session panel is
running. The extra terminal controls header is explicitly superseded by owner
feedback: replace it with a VS Code-style right session rail supporting multiple
shells, selection/new/close; remove that extra full-width row. Owner also reports
panel and editor tab labels ignore configured font size, and the panel header has
a duplicate bottom line. Address all tab typography in shared rendering.

Evidence for this first integration: public terminal-actions.test.tsr2/2PASS0.87ms
execution/setup20.967s (check1.282s, emit0.259s, native/link19.203s); actual registered
adapter denies forged input/terminate grants, malformed/unregistered requests;
command definitions generate tool schemas and validate targets/generations/bounds.
Core PTY2/2PASS0.05s after completion/palette support; UI model1/1PASS0s/setup2.69s
(font/id bounds, HTML never spawns a shell); compiler facade1/1PASS2.99s/setup2m34s
(typed commands, invalid action rejection and no Terminal dependency marker when
only its facade is used). No full compiler/LSP/UI matrix. Whole formatter passed,
last run2files changed; locales2catalogs/150keys and diff checks pass.

SDK normal pair built after four coherent native source iterations; each build
about2m34–2m37s. Native compilation first found five GPUI API mismatches (focus
needs cx, danger color, Stateful accessibility id), then one entity read-borrow
held across painting. Fixed by copying frame content under a short read, then
painting without the entity borrow. Wide-character backgrounds now precede all
glyphs; fixed cell advance supplied to shaping. GPUI startup stays on main thread.
The first native dependency compile rebuilt GPUI after optional dependency feature
changes; later runtime repairs reused it. These are build observations, not timing
benchmarks. Installed CLI/LSP have NOT yet been refreshed for this public checkpoint;
the focused compiler test may have replaced target/debug/tsrust with its test-profile
variant. Restore normal pair before atomic installed refresh and project-check MAT.

LIVE supervisor62791/PTY11755/app65759, one preview; current bundle is
/var/folders/yr/ylgbj3r132gb01sm8q41z92c0000gn/T/tsrust-62791-1794412820045355000-0/Tsrust Studio.app.
First load50.238s; formatter-triggered rebuild53.806s. Owner journal retained five
open paths, active src/components/editor/editor.tsrx, single pane, drafts0, zoom-1.
Owner was actively navigating; preserve their current state rather than older
left/right splits. Native Terminal first mount verified pwd=apps/studio, stty8x136,
Unicode/CJK/emoji/combining text and red ANSI foreground. Screenshot confirms full
wide glyphs and shared text scale in the terminal itself. Initial shell64584 was
replaced by formatter reload; current shell65845 (parent65844), app65759. The
interrupt button was clicked during /bin/sleep30, but its timing/outcome is NOT
proved; do not count it as a cancellation pass. Need repeat with a long identifiable
job and inspect prompt/child state after the UI/tool route. Copy, select, hide/show,
resize, zoom, restart and quit cleanup MAT remain pending.

Two Finder missing-file alerts were caused by CUA using stale App objects after
dev replaced/deleted its temporary bundle. Tool errors name the obsolete .app,
not an edited document. Reacquire using the current live bundle path from ps after
reload; never reuse a native App binding across replacement. Bundle-ID resolution
can retain a deleted path. Finder had no alert in its AX tree; CoreServicesUIAgent
is running (com.apple.coreservices.uiagent) but CUA could not inspect it (timeouts).
Do not claim the alert was dismissed. CUA phase closed with Identified stale preview
launch warning. Future inspections must use the live bundle path and avoid racing
rebuilds. User's two screenshots correspond to these tooling failures.

Important lifecycle gap found by source inspection: Unix dev Process.stop only
kills the app process group. PTY shells create another session/group. Native normal
quit closes/reaps through GPUI hooks and a completion-only App-global ledger
(no resource-handle registry), but forced dev SIGTERM can bypass those hooks.
Default-shell HUP likely exits; HUP-ignoring foreground jobs are NOT proved cleaned
by dev shutdown. Add a real witness, then a graceful termination bridge or managed
cleanup fix before claiming complete dev terminal lifecycle. No speculative pass.

Public terminal limitations: current OSC override queries report themed defaults,
though override colors draw; Windows/IME/middle/right mouse and broader resource
acceptance remain unverified. Raw local selection, scroll, paste, F keys, mouse and
focus modes are implemented but need native witnesses. Programmatic tools require
trusted host permissions and observed session generation; live AI transport is
still unconnected. Agent live-shell execution remains denied absent host grant.

Next immediate owner-priority work: shared Tab text-size override (native Kit Tab
currently overwrites custom font with text_sm), pass Studio font into panel Tabs and
Dock labels, use TabsSuffix for close control to keep one underline, and replace
TerminalPanel with a right-hand multiple-session rail. Reactive rows must have
semantic View ids; hidden nodes retain PTY state, actual removal drops it. Reuse
std:ui Sidebar/List/Resizable components. Then build once, load and perform native
MAT using fresh current-path CUA handles. Keep full scope and all above evidence.



Latest owner feedback: Save belongs in the native File menu (Save, Save As and
related file commands), not the workspace title bar. Removed that standalone
button from components/workbench/chrome.tsrx; existing Cmd/Ctrl+S still invokes
saveActive through its unchanged Shortcut. Recorded placement direction in
apps/studio/AGENTS.md. File menus/Save As remain pending, not newly implemented.
Whole-project formatter0changes; diff check passed. Automatic dev rebuild passed
in47.525s (runtime cache hit); native AX confirms only Settings remains in the
right title-bar actions and the Save button is absent. Existing project-check/
chrome editor panes and open tabs remain. Did not trigger saving owner files or
rerun unchanged tests. CUA completion: Removed title bar Save button.



Current continuation after search checkpoint0b6f35d5: search request is implemented
and visible. Re-inspected app52643: three separated, padded result rows, file paths,
line numbers and themed match backgrounds. No new app/source edits or restart for
this inspection. One preview remains supervisor37767/PTY50581/app52643; owner tabs,
splits, query and zoom preserved. CUA completion: Confirmed separated highlighted
search results. Previous search3/3 and exact-match activation evidence below reused.

Active next feature is the real terminal. Native prerequisite now in
crates/tsrust-compiler/src/stdlib/ui/terminal.rs (SHA256
0fea7700d63772dcbe8841726c079ec004b1f5edae689944ce2fde74ba048720).
Alacritty terminal0.26.0 Apache2 provides the VT grid and native PTY/event loop;
async-channel2.5.0 was already present transitively. Added direct optional managed
GPUI dependencies and internal terminal/gpui-terminal features; backend manifests
and lockfiles untouched. Eight new locked packages, no existing package upgrades;
restored bindgen's existing valid itertools0.11 edge after Cargo selected0.13.
Primary library source/API: https://docs.rs/alacritty_terminal/0.26.0/ and its
exact downloaded registry source, inspected read-only. No Kit0.9 Terminal exists.

Session owns PTY/VT state, coalesced one-slot redraw notifications, bounded latest
title/status, input, dimensions and force-close/reaper lifecycle. Native parser owns
Unicode, ANSI colors and alternate screen. OSC52 clipboard access is disabled;
terminal output never opens URLs. No polling redraw loop. Resize releases its
metadata lock before acquiring the terminal lock, avoiding parser size-query lock
inversion. Unix force-close targets the controlling terminal foreground process
group and owned shell group; it is not graceful command cancellation. Hiding must
retain the mounted session. UI must distinguish hide from force-close and expose
effects through trusted host grants, not allow agent writes to the live user shell.

Focused real-process boundary tests live beside that source and are included by
the isolated GPUI dependency package's cfg(test) module. Command: CARGO_INCREMENTAL=0
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
CARGO_TARGET_DIR=/Users/seancotter/Desktop/dev/learning/tsrust/target cargo test
--locked --offline --manifest-path crates/tsrust-compiler/gpui-dependencies/Cargo.toml
--features terminal --lib terminal::tests -- --test-threads=1.
Final2/2PASS0.03s execution/1.93s setup: actual TTY detection, Unicode/red VT cells,
100x30 resize via stty, input/exit7, coalesced wakes, exited-input/invalid-size errors,
and force-close/reaping of a shell ignoring HUP. First run selected GPUI and debug
symbols unnecessarily; stopped that build and narrowed to the internal PTY feature.
First completed1/2 run exposed a fixture Rust NUL escape, corrected; then2/2PASS0.04s,
followed by the final affected run after lock-order correction. No compiler/LSP/full
regression run and no installed-binary rebuild needed for this unexposed prerequisite.
Whole-project formatter0changes; diff check passed. Target79GiB, disk114GiB free.
One idle observation: app40,464KiB/0.0% CPU, supervisor3,296KiB/0.1%; not leak proof.

NOT terminal feature acceptance: renderer, checked Terminal catalog/API, public
command/reply hooks, AI routes, Studio panel and native terminal MAT are pending.
Color-query replies need the renderer's themed palette; shutdown completion must
be awaited from GPUI on_app_quit before process exit. Windows cleanup, detached
jobs, IME/selection/clipboard, keyboard/mouse modes and scrollback resource profiling
remain unverified. Next: embed this module under gpui-terminal, add opt-in runtime
selection, implement native grid/input renderer using real font metrics, checked
control/read APIs plus catalog docs, then .tsrx panel/shared guarded UI/AI actions.
Keep application orchestration in .tsr/.tsrx and retain the live preview during builds.
Full Studio goal stays active; no push/release or completion claim.


Full persistent Studio goal remains active, not an MVP or delivery claim. Owner
approved real implementation and compiler prerequisites, one writer/no delegation,
local checkpoints only. No pushes/model publication/signing/tags/releases. Full
requirements: [Studio plan](plans/tsrust-studio.md). Exact proofs, model identities,
failed resource attempts and historical dev profiling: [audit](plans/tsrust-studio-audit.md).
Attachment: /Users/seancotter/.codex/attachments/59d3ec78-9ac4-4015-8b60-f6fc29fb58a3/pasted-text-1.txt.

Initial workspace remains **apps/studio only**. The November 11 owner request
supersedes the former no-placeholder/status-bar direction: restore useful status,
terminal toggle and clearly labeled planned panels. Standing authorization:
**restart the dev supervisor whenever needed without asking again**, then leave
one preview running. Editable drafts must survive those restarts.

Latest accepted checkpoint127104c6: shared native fast-divider fix.
Managed gpui-base0.9.0 path patch applies the final release position and returns on
unchanged size before syncing stale painted bounds. Source from published .crate
SHA256cff6377e30f0c857681662e7edc7286bf30f5606033390401b0706d7bd930833;
registry untouched. Generic exact-byte/path/restore guards now handle URL+GPUI;
legacy URL alias and backend cache identity preserved. GPUI app/runtime/support
keys include lock+patch bytes. Production locked/offline gpui metadata selects
path source; lock only removes gpui-base registry source/hash. Roster221files.

Checks: patch safety3/3PASS0.17s exec/6m42s setup incl lock wait; CLI cache2/2PASS
0s+0.17s, support restoration2/2PASS0.03s, URL binding relocation1/1PASS0.04s.
Initial GPUI setup omitted test-support; no test ran. First complete resize run
12/13PASS0.04s exposed stale no-op sizes; corrected worker passed13/13 in0.04s,
setup2m45s incl lock wait. Final CLI/LSP buildPASS3m14s incl lock wait. Formatter,
UI generator,2locales/89messages and diff checks PASS. Full matrices not replayed.

Native app96611: fast body drag horizontal412/412 to472/352; vertical360.5/360.5
to421/300; sidebar248to288logicalpx, then restored248. Three panes main top-left,
zoom bottom-left,chrome right. Warm restart app98589 restored exact nested layout,
open paths and owner-updated zoom-3 (11px/base14). No drafts or file text changed.
Global active path was main before restart and chrome afterward; whether caused by
initial Dock notifications or concurrent owner use remains unproven. Do not claim
focused-group restoration accepted. CUA completion: Verified native resize and
restoration. Temporary mouse tracer/padding experiments gone from source and live
image. User changed zoom during checks; preserve their latest value, not historical2.

At127104c6 dev was PTY31628/supervisor96902/app98589 (superseded below).
Standard launch below. Initial patched native build236748ms:check14043.6,emit1103.1,
backend175.4,runtime prep3037.9,build+publish206695.9,native+link220781.9ms. Warm
restart16222ms:check14249.3,emit1079.7,backend127.8,app hit112.3,native+link0ms.
Single launch observations, not benchmarks. Current caches appe8f9c034f19d55dc,
runtime6c110a65bedc33e5. Removed6proven trace/padding caches under locks:
2,063,201,456apparent bytes, plus diagnostic target/studio56,644,840bytes.
Installed/target CLIc17e15a1bd76fcf987a28d676fc8d7e438193542891a1c431e158cef9eeb0848,
LSP2f60607825455025a1e10bf15ba1fa2060a551674a15022b1db82ca509821ef5.

Accepted checkpoint0bfaf967: detailed syntax capture/theme fix.
Owner Gruvbox screenshot and Zed request traced to active Syntect import=>preproc,
theme schema dropping unknown detailed names, and JSON inner strings hiding outer
meta.mapping.key.json. Community catalog104names/65detailed. Pinned GPUI component
patch preserves extra style names in both backends, exact then nearest-parent
fallback; imports/from/export=>keyword.import, parameters=>variable.parameter,
JSON keys=>property.json_key, true/false=>boolean, null=>constant, escapes retained.
Existing Syntect language coverage/palettes retained, no Tree-sitter product switch.
References https://zed.dev/docs/extensions/languages#syntax-highlighting and
https://github.com/zed-industries/zed/pull/47435 ; no copied GPL source.

Published component0.9.0 archive SHA25645bbfdba0efa392190f2fd446ec311325302eb047d5ce45dc1bcf189cb18f5eb.
Managed291files:285original +note +4pinned omitted test fixtures +Lucide license.
Fixture exact commit7591a35eced761e3613c01e71ff04c10129dae1a, hashes/paths in note;
5test-only include paths adjusted, assertions retained, fifth uses active sibling
GPUI base source. Registry unchanged. GPUI lock only removes component registry/
hash; all versions retained. Generic patch manager third payload, composite/path/
independent identity tests; existing keys already incorporate patch bytes.

Checks: highlighter8/8PASS2.01s execution/21.78s setup; other backend theme1/1PASS
0s/58.32s setup; managed patch3/3PASS0.35s/3m56s incl lock wait. Earlier failures:
offline missing optional metadata, then published missing fixtures (no tests),
7/8PASS1.98s JSON failure, temporary scope-only failure2.11s identifying enclosing
key scope. TEMP TRACE REMOVED. Final CLI/LSP buildPASS5m04s incl lock wait; public
native Studio buildPASS before preview restart. Roster/fixture hashes/diff pass;
whole formatter0changes before checks, repeat before checkpoint after docs edits.
No broad compiler/LSP matrix replay. Detailed audit appended.

Native app8224: Studio Dark blue keys/salmon values; Gruvbox orange import/from vs
light symbols, light keys/green strings; Catppuccin Latte purple imports, red
parameters, blue keys/green values. Reload restored saved Studio Dark without Save.
Closed only added clean JSON tab. Original3files, nested layout, owner zoom-3/base14,
drafts0 exactly match pre-check fields; settings bytes exactly preserved. CUA
completion: Verified detailed syntax theme colors. Full catalog contrast, semantic
LSP coloring and focused-group/cursor/scroll restoration remain unaccepted.

At0bfaf967, live app8224 used PTY73753/supervisor8045; superseded by the feature session below.
Launch same env -u CODEX_TOOL_CALL_ID/shared target/jobs1/incremental0. New dev
31840ms:check14006.9,emit1060.4,backend152.1,runtime prepare5940.4,runtime hit,
native+link15945.2,app miss0.1ms. Current runtime4c12f8cdc8fbbb9e,
public app0416073655ad5cc1/dev app5b682d0a7d42a547. Installed+target refreshed AFTER
all host/native checks: CLI3cbf37e3a88278d72858a80dcdd43a8be80f862c3f434848c068dbcbb2143333;
LSP0b55e2ea5d604b1affb6cdbdb7c8d70c8a33ed37f0cd6bd69c38c7cf317834f9.
Public target/studio SHA25662155c1cfb4c102fb5fc9fc37ffbc231d12d76d7618d423847387af9a8c5b34a.

Next executable action: whole formatter/diff, inspect status and local checkpoint
for this accepted section; no more source tests without changed inputs. Then resume
complete editor scope, prioritizing active/focused-pane cursor/scroll retention,
tree moves and real terminal/task/LSP services, with existing Kit/native APIs.
Keep one writer/no agents, one preview; protect owner changes. Do not claim full
Studio/AI goal complete. No pushes/releases/signing/model publication.


Accepted feature checkpoint90bd6a8d: GUI-first native Settings.
Owner now prioritizes feature throughput and human manual acceptance over frequent
narrow automation; NO full regressions ran this turn. Updated apps/studio/AGENTS.md
with exact direction. Use VS Code/Xcode/Zed patterns and Kit components. Remaining
priorities include custom file-type icons and connected terminal/task/LSP services.

Settings now uses std:ui Settings/Page/Group/SettingItem: searchable General and
Workbench pages; theme/font/locale and four region placement controls. GUI edits
save immediately. Custom draggable chrome with GUI/JSON/reload actions; optional
formatted JSON editor fills remaining window height, explicit Save/Cmd+S. Reuse
contextual save-file command name to avoid overwriting the app's global key binding
with a separate Settings command. Error window also has a drag region. Actions/model
separated in settings-actions.tsr/settings.tsr. Records with opaque Signal fields
cannot be cloned; each callback gets an independent context constructed with shared
signal handles and its own cloned path. Earlier ownership/numeric typing errors
fixed without compiler changes. No callback-clone workaround or framework added.

saveConfig accepts optional exact expected bytes. GUI tracks baseline/presence and
refuses unreadable-file saves until Reload; existing external edits produce conflict.
Initial creation retains one-writer shortcut documented in source. formatConfig is
an explicit typed settings serializer with scalar JSON escaping (generic JSON
pretty printing is not currently in stdlib). Existing invalid JSON stays visible
for repair. Tree/editor both explicitly use font-mono; removed top terminal button,
leaving status-bar toggle and existing shortcut.112messages/2locales pass audit.

Native app12683 verified Settings categories/custom titlebar/layout controls, GUI
font14to15 persisted on disk immediately, formatted full-height JSON font15, then
GUI restored14. Home config now pretty with explicit layout defaults, all original
values restored; owner zoom-3 retained, drafts0. CUA: Built categorized native
Settings controls. Focused settings.test.tsr4/4PASS527.01ms execution (includes
existing schema/placement/locale cases and new formatting/stale-write witness),
setup not separately timed. No full matrix replay. Whole formatter ran; final
source auto-rebuilt after contextual-save and formatting updates to app14694.
**KEEP DEV RUNNING: supervisor8045/PTY73753,app14694**,one preview. CLI/LSP still0bfaf967 installed
pair3cbf37e3/0b55e2ea (no compiler change). Latest auto rebuild50215ms:check15983.2,
emit1227,backend188.2,native+link20253.1,runtime hit. Single observation, not benchmark.

Next: final whole formatter/diff/local GUI-feature checkpoint, then custom file
icons. Found existing editors/vscode/icons/tsrust.svg. Kit Icon supports inline SVG,
but TreeItem exposes only named icons; extend its existing shared renderer/catalog
with SVG/tint support, not a custom tree. Prefer data-driven icon mappings and
existing/pinned standard language glyphs, with resource provenance. After that,
real terminal/task/LSP integration remains required. Keep one preview and preserve
owner feedback; no full suite unless meaningful family milestone requires it.

Latest search-results section — November 11, after ab9c975d.
Result rows now have two lines, full-width padded click targets (minimum height
3*font+16px), dividers, muted file paths, visible line numbers, monospace snippets
and theme-colored match highlights. Snippets crop around the first match with
UTF-8-safe boundaries and a bounded display window; original source/ranges remain
available. One result and first highlighted occurrence per matching line.

Shared search-actions.tsr now backs UI searches and result activation. Navigation
opens the exact match and checks current source bytes before publishing workspace
state; host context controls navigation/focus. Numeric bounds are checked before
arithmetic to reject overflowing offsets and interior UTF-8 ranges safely.
services/editor-tools.tsr provides two cataloged/callable routes, search_content
and open_search_result, using those same actions. Registered routes were exercised
against real host-owned Signal state; model transport and a full editor command
catalog are NOT connected. Schemas currently mirror typed records (explicit source
shortcut); generate them before expanding the catalog. Host identity/grants never
come from model JSON. Input parsing has explicit byte/depth/entry/node bounds.

Focused search.test.tsr3/3PASS20.35ms execution/setup5.380s: retained search/draft/
regex behavior; Unicode/cropped/zero-width/long-match previews; registered tool
search/open, denied grants including forged permission fields, stale data, invalid
args/names, maximum-int offsets and invalid UTF-8 endpoints. Earlier runs stopped
at unsupported native-range JSON/byte accessor and ownership diagnostics, fixed
without compiler changes. First runtime2/3FAIL28.92ms was a fixture's /var vs
/private/var expected path; corrected to canonicalize. Native RegexMatch is now
copied into an ordinary SearchRange DTO, retaining correct byte endpoints.
Whole formatter passed; locale coverage124keys/two catalogs and diff checks pass.
No full matrix or compiler rebuild. SDK/installed pair unchanged at c2f44df5/1aa17d7a.

Native MAT app47740: projectCheckWorker produced three separated rows, readable
paths/line numbers and colored matches. Clicking line151 opened existing main.tsrx,
scrolled to line151 and visibly selected projectCheckWorker exactly. Restored left
settings/rightchrome via Open tabs/focus; no file text edited. Owner had opened
recovery.tsr during work; retained it. Journal active chrome, zoom-1, drafts0.
CUA: Verified spaced highlighted search results, then Displayed readable highlighted
search results. Final accessible names include file:line and bounded preview; three
results left visible in app52643. Final source rebuilt successfully in47.295s.
LIVE supervisor37767/PTY50581/app52643, one preview. Earlier native runtime miss41.521s;
app build83.963s, then runtime-hit rebuild51.539s and unchanged-app check32.327s.
These are observations, not benchmark or leak acceptance. Formatter, locales, diff
and two tool-schema JSON admission checks passed. Next: local checkpoint, then
continue real editor/AI/service features. Keep
current owner state and prior acceptance. Full goal active, no push/release.

Owner update — every feature must have a direct integrated-AI action hookup.
Recorded in apps/studio/AGENTS.md and the Studio plan's required action-parity
section. UI/AI must share typed command implementations, trusted host identity,
capability checks, explicit document targets, revision guards and job lifecycle.
Preserve isolated agent proposals/review; no self-approval or terminal/tool bypass.
Existing UI acceptance is retained, but current exported helpers are not AI wiring.
The plan inventories current search/documents/editor/Dock/settings/check/viewer
hooks and missing routes. Next feature section is the shared dispatcher/executable
AI adapter for existing services, with native undo/redo/Dock hooks and guarded
mutation routes following. No agent transport or new runtime behavior is claimed
by this requirement checkpoint. Dev37767/PTY50581/app39551 remains current; no
restart needed for documentation. Full Studio goal remains active.

Latest accepted fix — content-search InputGroup and typography, November 11.
The screenshot request is implemented with existing InputGroup/Addon/Button:
inline case-sensitive, regex and search icons, tooltips and accessible names.
ContentSearch receives the shared font Signal; field, summary, errors and result
rows follow it. Filename-search input also binds the same size. Empty errors no
longer reserve a blank line. No new component framework or settings field.

The shared native input macro now applies explicit node typography to the actual
Input/Textarea, after Kit's control-size defaults; the former outer frame style
could not override the native field. InputGroupButton also receives its node style.
Compact text labels have their own Kit size, so the search toggles use the standard
CaseSensitive/Regex icons rather than text labels. No vendored dependency edit.

CLI/LSP build passed (2m41s); public native build passed. Whole-project formatter
passed with zero changes; locale coverage119keys/two catalogs and diff checks pass.
No full regression or unchanged executable test was run. Native MAT app37898:
projectCheckWorker query produced three matching lines/one skipped file. Cmd+= twice
scaled field, summary and results from10to12px, with query retained; case/regex
buttons still worked. Cmd+- twice restored owner zoom-1/base11. Settings unchanged
(Alduin, fileIcons:true), drafts0. Final app39551 visually confirms the icon group.
CUA completion: Verified grouped search and zoom.

CURRENT LIVE supervisor37767/PTY50581/app39551, one preview, superseding older
process markers below. Final source auto-rebuilt in45,660ms; native runtime hit.
Installed/target CLIc2f44df5a51101b64c603f11fd2b28cffe3a82c9fca7a8eb8555857c984d8287;
LSP1aa17d7abef5ed53269091ad96412f5ba9883668b7f37845ceb6b7a5faf7f07c;
both refreshed atomically and SHA-verified after native acceptance. Next: local
checkpoint, then resume real editor/service features. Keep existing acceptance,
one writer, no agents, one current preview; no push/release/signing. Full goal active.

CURRENT ACCEPTED SECTION — native source organization and project checks.
Local source checkpoints:0f5ef449 icons,ff925577 evidence restoration,ca66fae4
TSRX distinction/check draft,57e2a1cb35source moves; current compiler/service fixes
pending final local checkpoint. src/components groupedworkbench/explorer/editor/
settings/tasks; lib groupedworkspace/settings/explorer pluslocalization; services
owns .tsr workers; assets holdsicons/themes/locales. main/configentry unchanged.
Relative imports/tests and recursive locale auditupdated. Preserve originalevidence
below; old LIVE process markers are superseded by this paragraph.

LIVE **supervisor32953/PTY22133/app35512**, one current preview. Main-thread fix nativeaccepted: asyncUI startup nowstaysOSmain;
CLI/test8MiB helper unchanged. Focusedui_authoring hosttest1/1PASS1.14s execution,
setup2m32s. Final CLI/LSPbuildPASS2m36s; publicnativebuildPASS. Installed/target
CLIfe37a0e2a8c8c01bfda4db72becd93766497f9fb9c53e851f9892ccbcbb8cacd;
LSP09e980d2c6a1027c31edbceed73681aea6e451972ca08a56b7e17c78b5271721.
Cargo test temporarily selected differentCLI profile235b21d7; guardedrefresh refused
mismatch beforecopy. Normalpairbuild0.36s restoredexactfe37/09e9, thenatomicrefresh
SHAverified both. No activeexternalLSP process found; StudioLSP stillunconnected.

Native MAT: categorizedfoldersvisible, TSRXpurplebrackets vsTSRcoralTr. Migrated
currentopen/Dock/draftpaths atomically onlyafterold19300exited,beforereplacement;
0drafts,zoom-1 andallcurrentopenpathsexist. OwnerAlduin/base11 retained. Ownerhad
closedzoom pane duringwork, so currenttwo-pane layout isintentional; do notrestore
oldthree-panecontrols. Savedsettings/currenttabsremainownerstate.

Check toolbarlaunchedrealCLIchild33338 underapp33172; Stop displayedCheckcancelled
andreapedchild(psabsent). MATcaughtshortcutinsidehiddenpanel; movedglobalShortcut
toAppwithsharedrunning/outputsignals. App33677 CmdShiftB fromclosedpanelopenedOutput,
launchedchild33837, finishedwithvisibleCheckpassed.; noleftovercompilerchild.
Outputread-only/selectable andsamefont. Checks usepublicstructuredJSON onsaved
workspacefiles,cwdroot,PATH/TSRUSTexe,5min/4MiBexplicitbounds,noShell. Onejobatonce,
latecancelmessagesignored. Worker .tsr main-owned Job/cancel/await. No fakeLSP/PTy.
CUAcompletion: Verified organized workspace and checks. Final formatterPASS1file;
formatted-source auto rebuildPASS44166ms and startedapp35512. lastrebuild44026mscheck24136,emit1382.2,backend73.6,native+link18086.3,
runtimehit. Single observation, no comparablebenchmark/leakclaim. Appidle47872KiB
onesample; supervisorduringanalysis171600KiB99.7%; notleakqualification.

Passedrelatedproofs preserved: mapping/reportbatch3/3PASS0.90ms/setup5.361s;
changedfutureadaptertaskbatch15/15PASS13.19ms/setup4.718s; hostregistrationcargo
checkPASS9.59s; uientry1/1above; wholeformatter/generator119keys2locales/diffPASS.
NO fullregressions. Standaloneappquit whilecheckrunning remainsunverified: GPUI
exitsprocess, whereasdevsupervisorprocess-groupcleanup isexistingacceptedbehavior.
Removed1obsoletegeneratedlongtype diagnostic (511bytes). Formatter/generator/locale/
diff checks pass. Next: localcheckpoint, then actualPTY/LSP/Git/editorfeatures;
useexisting .tsr worker/channel/processpattern andnativeKit whereavailable. Keep
onepreviewrunning,onewriter/noagents,nopush/release/signing/modelpublication.
FullStudio/localAIgoal remainsactive; noMVP/fullcompletionclaim.

Latest runtime prerequisite: typed enum adapter replaces dynFuture erasure for
literal race/allSettled, preserving inferred Send/borrow lifetimes. New stable
literal-combinators.test.tsr registered in existing task host runner alongside13
cases. Public batch15/15PASS13.19ms execution/setup4.718s. Initial fixture indexed
owned Result rejection fixed with clone, no production ownership change. CLI/LSP
buildPASS2m36s; target hashes4cd0eae6/326a2328. Host registration cargo checkPASS9.59s.
Native public Studio buildPASS after adapter. Source organization35moves and current
relative imports resolve; root config still src/main.tsrx. Prior evidence retained.

Restart review initially rejected CtrlC99098 because original wrapper mentionedold
8045; read-only ps proved actual19300/app23352 onttys002 and queued30887 onttys001
waiting19300. Same matching CtrlC retry passed after evidence, no bypass. Migration
ran after oldsupervisor exited and preserved0drafts/zoom-1/openpaths/Dockstrings.
Owner active chrome path nowcomponents/workbench/chrome.tsrx; allopenpathsexist.

**LIVE SUPERVISOR30887/PTY63728, APP CURRENTLY EXITED101**: async main uses compiler
8MiB helperthread, violating GPUI OSmain requirement. Fix now in emit/mod.rs:
UI entry adapter gets OSmain BEFORE async helper branch; nonthrowing async UI
threaded flag likewise excludes UI. CLI/test workers retain8MiB unchanged. Focused
host regression in ui_authoring.rs contrasts UI vs CLI emitted entry. Build CLI/LSP
currentlyPTY16795, /tmp/studio-ui-entry-build.log. Next finish it, run only new host
regression, buildpublic native and queue exact30887 replacement (wrapperwaits30887,
no secondpathmigration), then CtrlC63728. Keep one currentpreviewafterfix.
Project-check service remains nativeunaccepted; complete UI play/stop/result/MAT,
confirm no leftovercheckprocesses, consumerrefresh, finalformatter/localcheckpoint.
No fullmatrix. Lastuserrequest sourceorganization nowimplemented checkpoint57e2a1cb.

Source organization now moved35files into components/{workbench,explorer,editor,
settings,tasks}, lib/{workspace,settings,explorer}, services and assets/{themes,
icons,locales}. main remains src/main.tsrx; config unchanged. All33relative-source/
test module references resolve; locale audit recurses now119keys/2catalogsPASS.
README reflects paths/features. Whole formatterPASS0changes after moved imports.
Path migration prepared /tmp/studio-migrate-source-paths.py and
/tmp/studio-source-moves.json, NOT RUN. Run only after stopping current supervisor,
before replacement. Preserve freshest drafts/open paths/Dock selection/zoom.

**Supervisor19300/PTY99098 currently SIGSTOP**,oldapp23352 stays open; paused reload
loop prevents loading reorganized paths before migration. Native public build
PTY18100 exited1, /tmp/studio-check-service-build.log. Root actualE0277: literal
race erases borrowed futures to dynFuture WITHOUTSend inside a started owned worker.
Existing emitter future_owned equates borrow lifetime with thread safety; literal
all already avoids this with typed pinned futures. Fix shared literal race/settled
adapter to preserve inferred Send without rejecting local borrowed cases. No UI
fake success and no unnecessary async architecture. Restart by queued replacement
first, CONT19300 then CtrlC99098 exact; migrate paths in waiting replacement before
newdevlaunch. Owner restart permission standing, no question needed. Native project
check service not yet accepted; initial preview still old. Current source checkpoint
before compiler prerequisite, no full regressions.

Current section after0f5ef449/ff925577: distinct TSRX glyph plus real project-check
service IN PROGRESS. TSR coral Tr vs TSRX purple component brackets, original own
SVG provenance updated. New .tsr worker uses std:futures channels and std:process
run --json; main owns/cancels/awaits worker around native UI. Check runs saved
project files in workspace cwd, bounded5min/4MiB, no shell. race cancellation drops
check task/reaps compiler; no Cargo/native children. Kit Output editor, play/stop,
CmdShiftB,100ms completion ticker active only while running. Live still old app23352
under19300/PTY99098 while changed-source builds occur. Native service unverified.
Model+icon selected batch3/3PASS0.90ms execution/setup5.361s; structured report
file/Unicode/location/failure/malformed guards. Race/stop added after this pure
format witness, so actual lifecycle acceptance still pending. No full regressions.
119localized messages (rerun checker final); formatter before checkpoint pending.

Owner additionally requests source organization: categorize lib/components/subdirs.
Next: checkpoint current feature draft, move code by area, rewrite all relative
imports/tests/assets, preserve current journal open/draft/Dock paths via a one-time
atomic path migration AFTER stopping old supervisor/BEFORE queued replacement.
Journal currently drafts0 but read freshest bytes at migration, do not discard
later owner drafts. Current source open:main/chrome/settings/config. Main stays
src/main.tsrx. Keep one writer and preview. No push/release.

Accepted file-icon section (checkpoint pending): existing std:ui TreeItem now
has documented iconSvg/iconColor props and generated wrappers/LSP help, using Kit
Icon.data/tint. Decorations excluded from structural fingerprint so toggles keep
selection/expansion. Data-driven14associations; own Tr alpha cutout and11Simple
Icons glyphs from official16.0.0 commit28241d80f8422eaed317ef968ad1e40b6d0d9163.
LICENSE/DISCLAIMER plus original/packaged hashes in src/icons/sources.json retained.
No new dependency, remote assets or custom explorer. XML/hash admission passed.

fileIcons defaults true, GUI Switch and formatted JSON persist it. Tiny stable
file-icons.test.tsr2/2PASS0.29ms execution; settings4/4 from90bd6a8d retained.
Generator,114messages/2locales and diff checks PASS. CLI/LSP buildPASS2m28s;
public native buildPASS. First native launch caught Computed UI value is disposed:
loop-owned computeds were not retained by a returned node. Moved each decorated
row to FileTreeItem component so its computed lifetimes follow that node; native
rebuildPASS34770ms(check16425.3,emit1218.1,backend50.8,native+link16445.8,runtimehit).
Initial malformed sibling SettingsSession declaration repaired before native build.
No full regression replay. Whole formatter passed including final helper.

LIVE **supervisor19300/PTY99098/app21033**. Owner changed theme/font/open files during
work: preserve Alduin/base11 and current journal, not earlier StudioDark/base14.
Native screenshot verified Tr/Markdown/JSON/Git/image glyphs. GUI togglefalse then
true persisted and painted immediately; exact selected settings.tsr, expanded src,
three panes/open tabs remained. No drafts edited. Current settings normalized with
fileIcons:true, other owner values retained. CUA: Verified configurable native file
icons. CLI SHAaf342ff25a0e31aabb7491ae552baf6565602661e942b6073860f441b4aa3cd2;
LSP SHA5ec7890a51babce6f2af1a6e4c2458c053f96052ff8d39e5807588322661d98b.
Installed pair atomically refreshed and SHA-verified; no replay of unchanged executables.

Next: final formatter/local checkpoint/consumer refresh, then real services. Found
std:process subprocesses/pipes but no terminal/PTY component. UI callbacks are sync;
trace existing task/channel/worker contracts before adding an asynchronous bridge.
Keep orchestration in .tsr/.tsrx and native facilities behind checked stdlib APIs.
No shell-command textbox disguised as a terminal. One writer/no agents, one preview;
no push/release/signing. Full Studio/AI goal remains active.

Current section follows7d192232 (open-file restoration) and36fe5ddc
(localization). Native Dock now exposes initialLayout/onLayoutChange/onLayoutError
through the generated std:ui facade and LSP documentation. It reuses Kit's own
versioned serialization, restoring keyed live panels with a DockLayout builder;
missing values skip and new children append. Shared validation rejects malformed
shape/version/axis/sizes/duplicate values and excessive input before builders.
Invalid state falls back with a technical diagnostic; Studio retains the bad
snapshot rather than overwriting it. Native malformed-fallback UI remains untested.
Editor composition moved into workspace-view.tsrx; no new dependency.

Owner additionally reported zoom loss. Zoom now persists per workspace as a
bounded offset from Settings.fontSize, in the existing journal. Startup and Settings
Save/Reload apply base+offset within8–48px; Cmd+0 resets offset to0 and configured
base, not hardcoded14. Shortcut changes do not write or normalize the user's
settings JSON. Legacy journal defaults remain empty Dock layout/zoom0. Document
sessions regenerate; dirty/external-file/journal write protections remain.

At b8aedbde dev was PTY16049/supervisor39042/app39391, superseded by the live session above. Two clean source
panes: main.tsrx left,chrome.tsrx right; zoomLevel2/base14 (16px text), drafts0.
Current launch from apps/studio, login=false,tty=true, desktop access:
env -u CODEX_TOOL_CALL_ID CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
CARGO_TARGET_DIR=/Users/seancotter/Desktop/dev/learning/tsrust/target
/Users/seancotter/Desktop/dev/learning/tsrust/target/debug/tsrust dev --verbose --color=always.
Keep one preview; do not restart merely to replay checks. Queue replacement first,
print exact target/current PID evidence, then send Ctrl+C to the matching old PTY.
Review initially rejected81403 despite queued57452; ps proved12055/app13849 on
81403's tty and replacement26620 explicitly awaiting12055. The same retry passed.
Subsequent queued restarts passed without permission questions. No refusal bypass.

Passed focused checks: CLI/LSP build2m32s setup; core Dock validator1/1PASS0s
execution/2m35s setup including Cargo lock wait; compiler Dock contracts1/1PASS
2.42s execution/2m30s setup (2positive/4negative format-round-trip cases). Project
check passed before zoom addition; latest main/Settings/zoom compiled and ran under
dev after it. Public native Dock buildPASS (build has no --verbose option).
Final formatted recovery+zoom batch3/3PASS75.05ms execution; setup not separately
timed. Covers opaque layout+dirty/clean/legacy recovery, persistence of zoom,
clamp/no-op bounds, nondefault base reset and invalid retained zoom. Initial fixture
integer Math.min use and consumed path were corrected without compiler changes.
Formatter/generator/catalog/diff checks pass. No broad matrix replay.

Native app29756: right/down buttons built three nested panes; actual drag reordered
documents.tsr ahead of main.tsrx. Journal recorded axes0/1, sizes412/412 and360.5/
360.5, correct order and selected tabs. App31412 after restart restored all three
visible editors and exact JSON structural equality. Both horizontal/vertical
divider drag attempts produced no size changes: interaction defect remains open.
App38249 automatically replaced after zoom source edits, retaining Dock layout;
closed only the two added clean test tabs, returning to owner's original source
pair. Cmd+= twice recorded2, Cmd+0 recorded0, then2 again; Settings Reload left
zoom intact. App39391 restart retained the same two panes and16px tree/editor
text. CUA completions: Verified restored Dock layout; Verified zoom survives restart.

Latest warm launch15,656ms:check13718.0ms,emit1049.3ms,backend170.0ms,app hit120.8ms,
native+link0ms. Zoom automatic rebuild40,281ms included runtime hit/native+link
19595.3ms. First new runtime71,193ms included runtime miss41469.0ms. Single
observations, not benchmarks. App RSS67104KiB/0% and supervisor5584KiB/0.1%, one
sample, not leak acceptance. Current dev runtime5d3257ac6b383b7b/appd336bcffd08df53a;
latest tests runtimeb317663627d28ec1/app5b81102062a7469a. Under namespace locks,
removed13obsolete cache directories/3,064,615,625apparent bytes; current dev,
public, test caches and prior full-suite control retained. No cache guard weakened.

Installed/target CLI SHA25687633e1999a55788244ce3c59e32274ca30b0a250931b058e9dac794dff3a648;
LSPc79c86fa516533be93cb10c9b95fb48502ecaee1fe7a7058bf7d6e02d05da80e match.
Exact source hashes and original settings/.gitignore/config identities in audit.
Home settings still exact {"schemaVersion":1,"theme":"Studio Dark","fontSize":14};
no source test drafts saved. Localization36fe5ddc's89messages,4/4runtime and native
language/Undo/startup proofs remain preserved, not replayed.

Next: diagnose native divider dragging from the real pinned gpui-base resize handle
and current scene; then shared worker/task/terminal/LSP services. Actual compiled
source is registry gpui-base0.9.0 (target/debug/deps/gpui_base-967f9078c9669a36.d),
not an unknown stale copy. Never edit registry files. Also pending: cursor/scroll/
focus recovery, duplicate/focused-group views, AX selected-range/unnamed popup rows,
IME, tree moves, JSON key scopes, broader locale/RTL and full editor/AI requirements.
No full goal completion claim; keep one normalized dev process running.

Previous checkpointbd88e292 added separate persistent Settings, 38 named themes,
content regex/case search, image routing, Markdown source/preview, status/panel
controls, workspace launch arguments, editable buffers/Cmd+S, atomic saves and
workspace recovery. Full goal remains active; drag/drop, splits, selected-tab
reveal, JSON key colors, docking/i18n configuration and native workflows remain
required. No completion claim. The per-item table in the Studio plan distinguishes
observed behavior from implementation.

The current live session is identified above. Command from
apps/studio, desktop access, login=false, tty=true:
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/Users/seancotter/Desktop/dev/learning/tsrust/target
/Users/seancotter/Desktop/dev/learning/tsrust/target/debug/tsrust dev --verbose --color=always.
Previous app31675 was read-only. The current supervisor loads the shared Kit
bound-prop identity fix required for editable row/preview state retention.

Historical bd88e292 installed consumer identities (superseded above):
CLI SHA256280dbb8fbd9954946e94378ce4730da61cca4f132bd962a3125fe0e9baaefd08;
LSP SHA2561392874ec607b81a3b05620bc3bf34508b8beda37979797c8c9af7ff066e602f.
No active VSCode document/server at previous observation; do not open the entire
checkout to force a server restart.

Retained bd88e292 evidence:
- Combined Studio model/storage/recovery/search/settings/workspace suite18/18PASS,
  90.14ms test execution. Setup timing was not captured for that run.
- Expanded 38-theme settings round-trip1/1PASS291.02ms execution,
  2728.9ms build/setup. Unknown themes and invalid fields reject before writing.
- Shared native branch/list regressions6/6PASS0.00s execution,1m05s setup
  (included shared Cargo lock wait). New controlled CodeEditor row keeps identity
  after bound value change; static prop differences remain structural changes.
- Explicit Studio main check passed after editable/recovery wiring and theme
  registration. Latest native dev rebuild also checks/emits/links the final picker
  synchronization/error-handling repair and starts successfully.
- Whole-project formatter PASS:10files,0documentation snippets,0fingerprints;
  stable second pass0files/0snippets/0fingerprints. Formatted suite18/18PASS
  329.44ms execution,4356.4ms setup. Final settings UI repair formatted1file;
  Studio format--check passes. Latest native dev rebuild checks/emits/links it.
- CUA before editable renderer refresh observed actual PNG decode, separate
  Settings window, active-tab contrast, status/footer and terminal placeholder.
  Markdown preview click reverted immediately: row rebuild reset local state.
  Shared bound-prop fingerprint fix passes native recheck: Markdown preview stays
  open and survives switching away/back.
- Native tree now uses monospace at shared font size. Screenshot appeared matched
  to editor; full zoom/resize/overflow and accessibility acceptance remain open.
- Upstream36community themes are vendored with source revision/hash manifest and
  Apache license; plus2Studio themes. Live Catppuccin Latte preview/Save/Reload
  and shared font16 configuration were observed; restored Studio Dark/font14.
  Full catalog contrast/rendering remains required. Failed first fetch used nonexistent LICENSE name; corrected to
  LICENSE-APACHE and retained its original bytes.


Native acceptance completed in app42379 and app46011:
Unicode paste creates a dirty tab; Cmd+S writes exact UTF-8 bytes; the dirty dot
clears. Undo/redo survive a switch to Markdown preview and back. Unsaved recovery
survives a real supervisor stop/start; disk remains unchanged. The temporary
.gitignore comments were removed through the app and Cmd+S; original SHA256
ad8ebeaeb880ec8c2388bbdf361e28d59d1ed0d7ae34a5c0a0e8ba60b7265483 verified,
and journal drafts are empty. No user draft remains from these checks. Content
search opens storage.tsr with the cursor at line4; uppercase anchored regex finds
it case-insensitively and disappears with Aa enabled. Own Settings window saves
actual ~/.tsrust/studio/tsrust_studio.config.json and Reload restores persisted
values. Final app49918 verifies picker selection follows direct JSON edits.

Latest native dev rebuild44,434ms: runtime hit,prepare7350.2ms,check11229.2ms,
emit911.0ms,backend172.8ms,native+link19406.8ms. Earlier fresh supervisor startup
65,687ms included runtime miss40,370.7ms. These are observations, not comparable
benchmarks. A bounded1second sample of earlier build supervisor39406 reported
537.6MiB physical footprint/537.7MiB peak; sample showed runtime unpack/build wait.
No leak or steady-state memory acceptance claim. Whole-environment native cache
identity can differ across supervisors; inspect before changing that contract.
Pruned14,100,780,330bytes of obsolete Studio cache variants plus old staging;
apps/studio/target/tsrust-native is now about2.0GiB. Keep active runtime1ff49c6f253063ac
and test runtime366be1fa48a2a0fc; retained current application/test caches.

Next executable section: implement native tab reorder/selected-tab reveal and
resizable editor splits using Kit facilities; fix JSON key scopes in a reusable
managed upstream integration. Complete file moves, configuration/docking/i18n,
close/discard UX, async search and recovery limitations. Do not retest unchanged
save executables or repeat the whole compiler matrix for routine UI edits.

Recovery limitations remain explicit: synchronous bounded journal writes per
change; one writer per workspace (first creation not coordinated); missing-file
recovery fails visibly and preserves the journal. Corrupt/outside-root journals
must never be overwritten. Saving rejects external changes. The app still needs
native editing/undo/IME/restart acceptance, close/discard UX and richer recovery.

Historical pre-bd88e292 evidence below (superseded UI behavior; retain passed checks).

CUA completed: Opened Studio package in tabs; Verified package tabs and status;
Verified native tree without clutter.
Observed actual main.tsrx, tsrust.config.json and workspace.tsr tabs; source folder
expansion remains open across file selection. Latest native Tree folder clicks
and Right-key expansion work; file selection opens real tabs, with no root wrapper
or button bar. Cmd+F opens native search. Both themes passed on the earlier viewer;
the current layout starts dark and has no theme button. Full-height layouts work. Binary icon.png reports UTF-8 error without replacing current file. Explicit
computed<string> status follows selected file/error. Find targets active config;
initial CUA typeText again lost the first character, while Raise + select-all/type
produced exact schemaVersion. Focus/first-key attribution and unnamed Kit search
AX controls remain gaps. A read-only typing attempt left source unchanged; this
is not full editable, IME, selection or keyboard acceptance.

**Native tab scroll/selection/Find retention now passes in the running app.**
Checkpoint76713c16 fixes sync_inputs/sync_kit disposal using all mounted identities,
while creation/synchronization remain visible-only. Hidden AppMenuBar ownership
still removes its external menus and reinstalls on reveal. Actual scroll position
(line34 at the top), keyboard selection of the final brace (native Cmd+F adopted
it), and independent setTheme/schemaVersion Find queries survive tab switches.
Undo/redo with editable buffers and full AX/focus remain unverified.
TabsContent's current native
layout only gave the editor intrinsic height, so Studio composes Kit's real tab
bar with full-height hidden document panels instead. browser.tsrx constructs its
child Nodes once outside reactive JSX spread expressions; merely passing Signal
inside those expressions previously recreated folders and reset expansion.

Typed data prerequisite is checkpointed at791d4d43: signal admission in
analysis/ui.rs, owned snapshots in emit/ui.rs, facade docs, LSP method help/native
alias identity and opt-level1 debug LSP profile. The current checkpoint adds
apps/studio workspace/browser/UI and these acceptance records.
No JSON-encoded state or handwritten Rust editor code. Generic native Signal
already supported Clone payloads. Scalars, nullable values, plain records/arrays
are admitted; nested native resources/views/callbacks reject. The nullable union
emitter uses existing tsr_owned conversion. Direct record/array-field JSX text is
still a construction-time snapshot under scalar_text's current form recognition;
use explicit computed<T> for live scalar text projections. Inference of computed
from this data projection needed explicit <string>. These reusable language gaps
must remain visible in acceptance rather than being called fully general reactivity.

Passed checks to preserve; do not replay unchanged executables:
- Compiler UI authoring: six existing cases passed in the earlier17.36s run;
  corrected new data/resource rejection case passes1/1 in1.21s (149s setup).
- Native gpui-starter composition/navigation/data snapshots:12/12PASS,3.30ms
  execution,163,863.8ms build/setup. Deep constructor/get/set copies, nullable,
  Unicode/CRLF, empty record arrays and scalar arrays are covered.
- Studio workspace suite3/3PASS,3.93ms execution,2,953.0ms build/setup. Canonical
  de-duplication, exact text/language, sorted filtered listing, outside/missing
  errors preserving tabs. Earlier document proposal4/4 retained unchanged.
- Native alias unit1/1PASS0.63s; full header/checked graph alias unit1/1PASS12.98s
  before debug LSP optimization. Debug LSP now uses opt-level1 with debug checks,
  matching existing compiler optimization. Actual server had exceeded existing
  3-second deadline with UI indexing; deadline was not raised.
- New completion matrix2rows x4modesPASS21.47s and editor consistency4modesPASS11.22s,
  including typed signatures/hover/docs and zero stale/empty/non-member counters.
- Initial selected Signal checker-agreement run finished418.10s with one false
  oracle disagreement: Signal.length actually rejects with a specific temporary-
  receiver error. Oracle had mistaken any diagnostic different from unknown-field
  for field acceptance. Shared predicate now rejects that diagnostic; actual-
  compiler regression preserves ordinary array.length and passes0.53s.
- Corrected full selected Signal agreement rerun PASS1/1,413.88s. Exact binary
  target/debug/deps/checker_agreement-0e7ca4f34a237d39, environment
  TSRUST_LSP_AGREEMENT_TYPE='Signal<string[]>'. Session10828 is terminal. The
  complete candidate scan was preserved; no member or receiver was excluded.

Historical accepted tool pair before subsequent renderer/API updates: CLI SHA256
0f14ca6e610ae4366a995dab150a275d2c1aca2841ef6c2a3115b71984752bd7;
LSP SHA2564d55f9d43dcabaf843210256fe81ab7aaecd47beb1cc1584a5b567777e7bc688.
Installed PATH pair was refreshed with staged copies and atomic replacements,
matching those SHA256 values; CLI0.2.0/LSP0.3.0. At that earlier refresh, no active tsrust-lsp or VS Code
process was found. Existing VS Code settings point to repository
target/debug pair. Constructor hover now explicitly documents computed<T> for
live data-field text; getter/setter documentation and their passed tests are unchanged.

Current cold elevated dev47,677ms; warm source rebuilds14,767/14,566/14,624ms,
runtime preparation5,324.4/5,609.7/5,023.4ms, all warm runtime hits. Development
observations only. The retained supervisor now includes bounded streaming cache
checksums/extraction from65665fb6 (legacy checksum protocol and corruption guards
passed11native-cache +2bundle tests). Historical appPID19889 read-only vmmap session25690 completed: supervisor physical footprint49.3M,
peak408.0M; app258.6M, peak317.0M, with3open file tabs and main active. Units are
reported verbatim from vmmap; these are not comparable benchmark runs. A subsequent restricted leaks scan reports420leaks/20,336bytes from523,615
allocated nodes/192,915KB. Allocation contents were not saved; macOS marks the
process not debuggable, and attribution/growth remain unproven. The earlier
scratch scan was a different PID and is not a controlled comparison.
No AI executable was rerun during this editor section.

Prior accepted checkpoints: 3c59b679 initial app/proposals/scalar training;
e28b8d1f macOS dev identity and live failed-build recovery;65665fb6 bounded cache
buffers;acb92c73 dev identity ignores release-only options;02de4640 bounded Metal
inference proof; e09f8fcd initial package scope. Audit retains exact results.
Candle scalar training3/3 proves only scalar SGD orchestration, not model training.
Three Candle CPU Qwen attempts stopped without answers on resource limits. Pinned
llama-cpp0.1.161/Metal/mmap with the same official Qwen1.5B Q4/tokenizer passes2/2
buffered inference tests (exact5/EOS,31,970.943958ms execution,163,195.539917ms setup,
zero observed swap-usage growth, sampled treeRSS1,251,000,320B). No full8GiB coding
quality, streaming, cancellation, training/specialization or distribution acceptance.
Model artifacts remain ignored under target/studio-models. No unchanged replay.

Latest native Tree/root-removal rebuild13,356ms passed, with runtime preparation
4,419.3ms. No unchanged acceptance executables were replayed for this layout change.

Native retention checkpoint76713c16: new lifecycle regression1/1PASS,0.00s execution,
3.66s setup. Compiler build after final menu safeguardPASS2m41s. Final backend
all-target compilationPASS3m26s; final workspace/all-target compilationPASS1m28s.
Warnings are in unchanged test helpers; no node/renderer warning. Native Studio
buildPASS. No unchanged runtime suite or AI executable was replayed.

Owner answered “Restart once and verify tab state.” Old session51314/supervisor15826/
app29484 stopped normally, and one replacement is live: session6863/supervisor41326/
app41377. Startup45,087ms, check2,069.4ms, emit312.5ms, native+link42,310.5ms,
runtime preparation1,498.0ms and runtime build/publish36,864.4ms (cache miss).
Development observations only, not comparable performance measurements. Exactly
one Studio preview remains, and the new supervisor must stay running. CUA phase
“Verified retained editor tab state” completed. Initial config Find typing still
lost its first character; select-all/retyping gave the exact query. Native AX
selected-text range is unavailable; keyboard selection was observed and then
verified by Find inheriting it after a tab round trip. Missing search AX names
and full focus/IME/editable acceptance remain open.

Repository/running/installed CLI SHA256 is now
ebf586f90a92b181001b1a8d0012c6e7a415517453fd197063be1d6146f6c899.
Installed CLI was staged and atomically replaced after native acceptance. LSP
remains on the prior accepted4d55f9d43dcabaf843210256fe81ab7aaecd47beb1cc1584a5b567777e7bc688;
no language-service signatures changed in this renderer fix.

Fresh vmmap baseline after native acceptance (two tabs): supervisor384.5M/peak384.5M;
app253.1M/peak253.1M, units verbatim. Not a controlled comparison or leak proof.

Current document section: workspace OpenFile owns the shared Document (path,
session, revision, text, savedText). OS-random std:uuid IDs isolate reopened sessions;
proposals now match session too. Dirty state compares saved text; clean reload
updates baseline/revision, dirty reload preserves drafts. closeFile model refuses
dirty close unless explicitly discarded and preserves/selects the remaining tab.
openFile reuses live sessions and rejects non-regular paths before reading.

Final formatted tests10/10PASS4.31ms execution,3,039.5ms build/setup, using
`tsrust test tests --profile` from apps/studio. Earlier10/10PASS4.21ms; expanded
workspace4/4PASS3.57ms; preserved old assertions, added identical-byte session
reopen, baseline/dirty reload, no-op revision, discard/close and actual reopen.
First mixed string/FsError catch failed checking; fixed by explicitly matching
createDocument Result, without changing compiler contracts. Dev retained the
working app during failure. Formatter changed3files and final native tests passed.

Latest appPID48203, same supervisor41326/session6863. Normal rebuild after format
12,184ms, runtime preparation5,072.4ms cache hit. Earlier new UUID runtime build
51,764ms cache miss. CUA “Verified workspace document session integration” completed:
actual documents.tsr opens in a native tab through the integrated document record.
No native save/close control is exposed; app remains read-only.

UI alignment checkpoint acbcb8ed: chrome.tsrx, navigation.tsrx,
editor.tsrx and themes/studio.json now separate the shell. Native custom header
uses View.windowDragRegion and reserves the macOS traffic-light leading area.
Settings/search use icon triggers with tooltips. Explicit rail dividers and a
two-surface palette replace mixed grays. Filename results align left and truncate;
Input has explicit available-width constraints. Sidebar collapse and actual divider
resize are verified. Tree text sizing/ellipsis and stable keyed rows are in the
new renderer awaiting a live load. No Git/extension/terminal/chat filler controls.

Removed workspace.language and its hardcoded extension map. editor.tsrx passes
the extension to std:ui/Kit, reusing existing tsr/tsrx aliases and syntax lookup.
The active Syntect backend maps imports to preproc; Studio theme now assigns it
the keyword color. Actual TSX/TSR imports are visibly distinct. Initial unrelated
Tree-sitter hypothesis was discarded. Its host build ran0tests after4m43s setup
and proves nothing; no grammar/backend/dependency change remains. An obsolete
compiler build was stopped. Do not rerun that experiment.

Native close testing found a stale parent tab click could reselect a removed path
and blank the editor. selectFile rejects closed paths; native close now restores
main.tsrx without another click. Full model suite11/11PASS6.47ms execution,
120,031.5ms setup; final affected workspace5/5PASS4.23ms,93,605.9ms setup includes
stale selection. Six document cases retained unchanged after that affected rerun.

Shared Node.each uses distinct nonempty document IDs as stable row identities;
unlabelled rows keep positional semantics. Mixed insertion cannot consume keyed
rows. Duplicate IDs fail before old ownership is discarded. Dynamic class values
do not reset row identity. Final new regression1/1PASS0.00s,3.53s setup; existing
list control1/1PASS0.01s on the final test binary. Tree forwards text size to its
ListItem and uses ellipsis. Shortcut/Ticker are now absolute/zero-size: ordinary
empty flex children had created80px trailing action-row gaps. Commands also sit
outside the visual action row.

Workspace/all-target checkPASS1m31s and separate backend compiler/LSP all-target
checkPASS1m20s before the final mixed-row/native-only repair. Warnings are confined
to unchanged test helpers, including unused/dead code and fetch_update deprecation.
Final repaired Node code passes its test/control; compiler buildPASS2m16s.
Candidate CLI SHA25642db19302c4f46245262af62e6e2351c6044e6d4cb7e22957d323b1171a88519.
Final native buildPASS (session28153 terminal); retained dev6863/supervisor41326 still
uses ebf586f9 renderer. Installed CLI/LSP remain prior accepted images. No live
supervisor restart is authorized for this new renderer yet.

CUA phase “Verified compact editor shell controls” completed: header/no app label,
rail divider, left-aligned exact workspace filename filtering/open, tab close
restoration, collapse/full editor width, actual divider resize (image coordinates
588,500 to788,500), settings font actions and separate Cmd+=/Cmd+- editor zoom.
Right action inset is corrected. Light appearance, Tree zoom/ellipsis, native state
across moves, drag reorder, full AX/IME and layout persistence are still unverified.
Live appPID76014 after normal source rebuild (child changes on source saves).

Owner explicitly requires configurable locale/i18n, appearance/text sizes, navigation
and activity-rail sides, terminal and AI-chat docking. These are required scope
in the plan/AGENTS, not implemented by the current temporary settings menu. Defaults:
rail/tree left, terminal in a resizable bottom panel under editor, chat right.
Use existing std:ui locale, Dock/Resizable/Settings and one validated persisted
settings model. Do not make one-off side toggles or empty panel placeholders.

A one-time supervisor restart question is pending through request_user_input_async;
no reply yet. Keep session6863 running. Prior goal turn made progress with local
checkpoint acbcb8ed and removal of the redundant language lookup.

Lifecycle checkpoint79c37206 suppresses unchanged dev replacements before editable buffers.
Running now owns its complete native cache input identity and, on macOS, the
generated Info.plist bytes. A successful rebuild retains the child only when
those exact inputs match and try_wait confirms it is alive. This uses full input
bytes, not the hash directory name. Exited children still relaunch on the next
edit; failed builds retain the previous app. Candidate directories drop on the
unchanged path. No arbitrary memory cap or source/watch exclusion was added.

Identity unit passes1/1 in0.00s. Initial Cargo invocation accidentally used an
unqualified exact filter and ran0tests (2m12s build); direct final test-binary
invocation used the qualified path and ran the intended witness. Extended existing
process lifecycle test covers test-only edit/PID retention, actual dependency edit,
failed build, exited-child relaunch and interrupt cleanup. Its final run passes1/1 in3.22s execution,12.51s build/setup
(session48574 terminal). Backend compiler/LSP all-target compilation passes1m19s
(session27102 terminal); warnings remain in unchanged host test helpers and
reference controls (unused/dead code and existing fetch_update deprecation). No
changed dev/cache source warning. Repository CLI SHA256 is now
074d80e8c16f828d279e44d00c9df866d3baf8b1fd3e7c8f61693ac7b7a5b426.
Installed consumers and the retained supervisor still use prior images. No live supervisor replacement yet. Settings/save work is
still pending; this fixes a concrete draft-loss prerequisite.

Current section: std:fs.writeTextFileAtomic for storage, recovery and settings.
Implemented bindings/registry/bridge, separately selected native module, full
IntelliSense docs/example, public std:testing suite, compiler selection/type
witnesses and affected LSP rows/help. Native implementation stages in the same
directory, syncs file and parent, rejects symlinks/read-only/multiple links, checks
optional exact baseline plus observed inode/metadata revisions, and preserves
macOS metadata with fcopyfile (explicit ownership/mode verification). New files
are private. committed-not-durable means bytes changed despite Err and must be
handled explicitly by Studio. A final rename can still race an uncooperative
external writer; no cross-process CAS or filesystem sandbox claim. Windows is
explicitly unsupported before mutation for now; that remains required later.
Other Unix metadata preservation is mode-only; Linux product delivery is deferred.

No app save/edit action is enabled yet. Recovery and model integration must handle
commit/durability outcomes before editable buffers. Atomic backend host witnesses
cover deterministic staging failure, concurrent changed/identical-byte replaced
files, post-commit sync failure, ACL/xattr/mode/owner preservation and unsafe
paths. Public suite covers UTF-8/BOM/CRLF, chunk boundaries, first-class/optional
calls, empty files, missing parents/targets and conflicts. Whole formatter changed
one fixture and one doc snippet, with token/idempotence/fingerprint checks passing.
Atomic backend host units pass3/3 in0.03s (3m17s setup, session20685 terminal).
Selected runtime extraction/isolation witness passes1/1 in0.23s (2m14s setup,
session29648 terminal); type-rejection/ownership control passes1/1 in0.02s
(0.95s setup, session30780 terminal). Initial public generated suites fail before
execution: new operation was missing from the closed audited native-effect list,
so direct/first-class calls propagated an opaque contract into std:testing. Added
the audited Result-only contract without weakening unknown-call rejection, plus
a mutating-observer exclusion witness. Original direct/alias cases stay intact.
Corrected effect/audit witness passes1/1 in0.01s (3m16s setup, session79824
terminal); mutating-observer exclusion control passes1/1 in0.00s on that binary.
The repaired public suite and generated example compile together in one program.
Their disposable working directory was /private/tmp/tsrust-atomic-suite.FrjHFj
(now removed). No native execution pass is claimed for either failed source check. Corrected public suite and actual
catalog example pass3/3 together in63.47ms execution,1,561.3ms build/setup
(session45044 terminal). CLI/LSP pair build passes2m28s (session20261 terminal).
Affected completion matrix2rows x4modes passes2.37s; filesystem consistency8cases
x4modes including new callable hover/parameter help passes8.78s; catalog contract
help passes1/1 in0.03s. No new receiver type; existing FsError agreement is retained.

Final macOS metadata strengthening verifies copied BSD flags before publication;
extended host witness also proves UF_HIDDEN retention and fresh modification time.
That affected witness passes1/1 in0.03s (3m17s setup, session85303 terminal). Other
backend/effect/editor witnesses are unchanged and retained. Final pair rebuild
passes2m27s (session35496 terminal). Final public native suite
and generated example pass3/3 in45.58ms execution,1,874.7ms build/setup (session57991
terminal). Backend compiler/LSP all-target compilation passes1m18s (session52425
terminal), with only unchanged host helper/reference unused/dead code, HTTP import
and fetch_update deprecation warnings. No changed source warning. Installed CLI
and LSP were atomically refreshed and SHA256-verified independently:
CLI f5e31107f51340b11e6a512c63a50551cfb858e6e43a1861ab88612a001fa292;
LSP 6d55dbde9e6143242f57132cb8067c9cf6f47169b1a2b00cc9f67c38a011963a.
Repository overrides point at the same new target/debug pair. VS Code is running
but has an empty editor and no tsrust-lsp process; CUA “Verified no active editor
documents” completed. No server restart or open-document diagnostic claim is needed.
Studio supervisor41326/app76014/session6863 remains running on the earlier image;
latest live poll confirms the PTY remains active. Pending restart question has no
reply. No Studio restart occurred. Removed the disposable generated catalog tree,
empty private test cwd, obsolete pre-flags native application cache variant, and
171,774,617B of obsolete test executables/dependency files. Kept current tools,
final compiler unit binary, current native cache and the pending Studio preview.
The macOS storage primitive has focused acceptance. The live Studio app does not
yet use it; Studio saving/recovery remain unaccepted until their integration and
workflow checks pass. This is not full application or Windows acceptance.

Next: implement Studio
save/recovery orchestration in .tsr using exact disk baselines, explicit handling
of committed-not-durable, and retained drafts before enabling editable buffers.
While the restart reply is pending, continue that independent storage/settings
work. Load the renderer only if the pending restart is approved, and verify
Tree zoom/ellipsis and editor state during close/reorder. Keep6863 running until
replacement is authorized. Continue validated settings/i18n/docking alongside
safe file-save/recovery primitives and revisioned editing. Test-only replacement
is fixed in79c37206 but awaits loading into the retained supervisor. Draft
preservation across legitimate dev reloads remains required before editable buffers. Full saving, recovery, PTY and chat are
not implemented; model guards alone do not prove file safety.
Native checked resource/streaming/session ownership, held-out AI work,
all editor workflows, cloud/external agents and Windows remain required. Use the
actual apps/studio package for visible acceptance as implementation proceeds.

### Balanced global search spacing — 2026-11-10 (accepted with this checkpoint)

Owner reported the close button overlapping the command panel. Final change is a
single p-4 View around the gallery search Command in src/search.tsrx, adding equal
padding on all four sides while retaining Kit's original rounded border/background.
The first local attempt used top-only padding and bordered=false; owner rejected
the squared panel and unbalanced gap, so both were corrected before checkpoint.
No renderer, public API, search ranking, generated files or new dependencies change.

Final native gallery build passes. Whole formatter0files/snippets/fingerprints and
diff check pass. Native CUA verifies equal inset on all four sides, rounded panel,
and unobstructed Close in light/dark themes with the exact Custom input menu query.
Close works and reopening retains the query. Prior intermediate Enter navigation
also opened Input; handlers are unchanged. Independent app bundle atomically
refreshed and one preview left open on dark search for inspection. Completion:
Verified balanced rounded search spacing. No CUA phase open. No broad regression,
web interaction, performance measurement, push or remote validation for this
reversible gallery-only layout change.

### Composed gallery examples — 2026-11-10 (accepted with this checkpoint)

Owner requested readable shadcn-style example composition instead of giant title,
description and escaped code props. `src/example.tsrx` now exports ExampleTitle,
ExampleDescription, ExampleTabs, ExamplePreview and ExampleCode alongside Example.
Example only lays out children. ExampleTabs reuses native Tabs/Tab/TabsContent
selection and retained child nodes; no separate signal/context API. ExampleCode
accepts string children, displays Markdown source and owns the Copy control inside
the code panel. Guide live examples use the same tabs/preview/code composition;
the duplicate LiveExample implementation is removed. The gallery generator emits
readable text children and multiline template children, escaping backslashes,
backticks and interpolation openers without changing displayed source. Regenerated
all71 component pages/358 examples and guide live blocks; README explains sources.

Completed: full gallery project check; both generators --check; whole formatter
0files/snippets/fingerprints; diff check; bounded Table authoring regression1/1,
18.75s execution,4m40 build/setup including shared-target queue. Existing composition
cases8/8 passed,2.69ms entire initial run. New native composition witness initially
asserted Markdown source was in document HTML, but Kit props are omitted by that
serializer; a labeled rerun confirmed precisely this unsupported assertion. The
source assertions were removed; headings, preview and Copy remain asserted. Final
focused rerun1/1 passed,1.51ms execution; setup/build3m40 including queue. This
preserves the existing8/8 evidence instead of rerunning unchanged cases. Independent
Node VM evaluation of all373 generated template literals matches original spec/guide
code byte for byte. Native gallery build passed; regular app-bundle executable was
atomically replaced, old preview terminated, and one independent preview reopened.
Desktop proof: Basic Accordion initially expands first item; second item remains
expanded after Code -> Preview switching. Code panel shows highlighted source;
Copy pastes the complete expected18-line BasicAccordion function into a disposable
unsaved Code buffer, which was closed without saving. Gallery left on Accordion
preview. CUA completion: Verified composed example tabs and copy. No CUA phase open.
VS Code full rescan settled
idle and CUA reports no workspace problems. No compiler/runtime/LSP source changes,
new dependency, performance measurement, push or remote validation. Web interaction
was not rerun for this example-only refactor; native/structural proof is above.
Pruned four superseded failed-test cache entries; kept final native cache entries,
shared runtime build support, shared Cargo dependencies and independent app bundle.

### Native text, parser and packaging finish — 2026-11-10 (accepted with this checkpoint)

Latest accepted checkpoints: fd803813 resolves all16 reported workspace diagnostics;
2bdecffe preserves native behavior for utility-styled buttons. This final family:
- Native Text uses GPUI StyledText/TextRun through a small RenderOnce adapter, reading
  inherited window text style at layout. Nested weight/italic/family/colors/solid
  background/underline/strike stay in one paragraph; explicit no-underline and sans
  reset work. Root hover inheritance remains native. Text/UTF8/order preservation
  is asserted against the existing content_text control. Rich Kit button labels use
  the same runs while retaining an explicit accessible name; ordinary labels keep
  the upstream label path. Unstyled href links reuse native Kit Button link behavior
  with no padding/automatic height, replacing the mouse-only upstream Link Div.
- Inline validation now admits italic/strike/solid background. The first desktop
  attempt exited because the old shared whitelist rejected those paint effects;
  corrected at that common boundary. A regression accepts the paint combination
  and still rejects padding, positioning and flex layout inside text.
- The new legitimate guide found raw TSRX punctuation after nested tags being read
  as expression operators. Parser::is now excludes JsxText like String/template
  tokens. Native punctuation suite proves exact café/punctuation/as/in/satisfies
  output and normal postfix .length outside markup. Malformed Unicode/nested markup
  returns valid diagnostics without panicking in the targeted project witness.
- Styling guide/live examples and hover help now state actual GPUI limits instead
  of claiming universal parity. Nested links inside Text still lack separate native
  hit/focus targets; use a Link beside Text. One paragraph has one size/line height.
  Tracking/balance/transforms/underline offsets/animations/pointer-selection helpers/
  field-sizing/justify-self/auto-row-min/table CSS/nonuniform tracks/placeholder and
  inline interaction-state mappings remain explicit limitations. No full CSS or
  upstream Kit parity claim; prior catalog count1939/2147,20 incomplete is unchanged.
- Added isolated KeyboardLinks guide example using createLocation. Native AX exposes
  Open local docs as Link; Shift+Tab focus ring then Enter changes /links/home to
  /links/docs, keeping the gallery on the same page and opening no other window.

Acceptance (build/setup separate from test execution):
- UI lib137/137,.04s; build2m37s including queue. Prior136 also passed before whitelist
  repair. Compiler ui_authoring6/6,19.35s, including bounded Table replay and malformed
  markup checks; LSP TSRX6/6,1.88s. Public punctuation suite2/2,.27ms execution,
  compiled together once. Fixture initially had an unrelated optional-array join
  inference error; required children fits every call and preserves exact oracles.
- Both consumers rebuilt2m36s for the final runtime validation; parser build2m49s.
  Native gallery build passes. Workspace/all-target1m49s and separate backend/all-
  target3m18s pass for current Rust changes. Existing shared test-helper warnings
  remain reviewed/unchanged; no warning suppression or new runtime timing/SQLite.
- Desktop visual proof: bold café rose -> sky on signal update; italic monospace
  code on amber; no-underline clears only its span; sans resets within mono; bold
  rich button name is exposed. Stock Kit buttons intentionally avoid mouse focus:
  after Shift+Tab gives keyboard focus, Space activates the rich button and changes
  sky -> rose. Earlier styled Base Button click/Space/Enter/disabled proof remains.
- Live editor final server25028 is idle (CPU0%,RSS2160KiB at33m15 after restart) and
  CUA reports no problems. Old accessible panic message is historical: Tsrust log
  has three project/link.rs:431 failures before the4:20:05PM restart, none after.
  This replaces the earlier transient-cleared count. Last CUA completions: Verified
  native inline text styling; Verified native link keyboard navigation; Verified
  final editor diagnostics cleared. No CUA phase open.
- Whole formatter after new fixture/code:0 files/snippets/fingerprints on final
  pass; both generators and diff passed. This record is included in the local source checkpoint.

Unsigned public packaging smoke passed: a real release build made ZIP, DMG and
SHA256SUMS. Both hashes verified, archive plist/app.json confirm CLI name Override
Smoke, config id dev.tsrust.smoke/version1.2.3/repo example/smoke; CLI cwd-relative
artifacts overrides configured-artifacts (absent). No signing/notarization or
publication. hdiutil only emitted a host deprecation notice. Temporary smoke tree
and raw verification data/logs are disposable; do not retain as canonical records.

Cleanup: removed unused target/debug/incremental4.1GB and secondary gpui-stable3.1GB
only after verifying no active consumers. Keep original target/gpui-starter file.
Current independent bundle /private/tmp/Tsrust Gallery.app is a regular copied
executable, parent launchd; one preview process43849 at last inventory. Final prose-only guide rebuild97964 passed; bundle atomically replaced and
reopened on Styling and Themes, inline example visible. CUA completion: Left the
updated gallery open. Official native cache cleanup removed25 artifacts/8,574,772,414 bytes; temp cleanup completed
before the local checkpoint; preserve this regular bundle and compiler/LSP bins.
Do not restart unchanged heavy matrices or push. All latest logs are /tmp/tsrust-ui-*.log and are disposable after this record.

### Workspace diagnostics repair — 2026-11-10 (accepted fd803813)

Owner screenshot disproved the transient cleared Problems pane:16 diagnostics
remained after the prior scan. All are now resolved without diagnostic exclusions.
- Generated both private server SDKs with their checked API configs; resolved the
  rust-owned-words binding lock offline. API-contract README includes its server
  config. Ignored generated SDKs are local setup; the dependency lock is retained.
- Updated dock/gallery/gallery2/gallery3 to current ListItem, string-valued Tabs,
  InputGroup composition, InputChangeEvent.target.value, Combobox array events and
  JSON DataTable rows. List retains initial Drafts selection and reports the row.
  All four check. Realtime SSE handler is async, allowing its runtime-driven
  producer start; it checks, along with both server witnesses and words/main.
- Removed unreachable return after an unconditional returning loop from both
  scoped composition counterparts; explicitly reviewed/updated source hashes.
  Prior timing remains historical; this run makes no new performance claim.
- Loop completion now retains newly stored native-view loans on fallthrough and
  continue without propagating back-edge moves. Recursive view record stores use
  their already-checked whole-body carrier rather than unrelated nullable join
  restrictions. All four failing byte-view workloads check and execute correctly.
- Extended exact-path/exact-byte bundled_entry mapping to split UI facade parts
  and logging; observes all parts before trusting either UI fragment. LSP uses it
  only for entries, never imports. Unsaved changes revoke the mapping. All three
  compiler-owned files check. No private-binding access granted to user modules.

Acceptance:
- New public loop-provenance suite2/2 native tests,0.74ms execution. Whole formatter
  proves stable formatted source. All30 std_bytes host tests pass (49.84s), including
  owner-mutation/short-lifetime rejection on fallthrough and continue and existing
  native runtime family at both required profiles. Host build3m49s incl queue.
- Five affected workloads pass native and Node-after-tsc correctness-only checks,
  five input cases each, matching independent oracles. All12 Node source/oracle
  registration guards pass (57.21s). No timing or SQLite row added.
- Full compiler lib1541 passed,0 failed,8 existing ignores,147.55s execution;
  build7m43s incl queue. LSP lib26/26 (.16s); first run25/26 failed only because the
  new fixture used a forbidden absolute import; corrected to a relative import.
- LSP workspace6/6 (.01s) and workspace_scan8/8 (.02s) pass; build4m49s incl queue. Both workspace/all-target and backend/all-target
  compile. Existing unused members in shared runtime/test controls still warn;
  all warning sources outside the removed unused native_threads host_id helper
  are unchanged. No blanket warning suppression. Affected native_threads check
  rerun passes with no warnings (59.69s incl queue).
- Both consumers rebuilt7m09s incl queue. Live VS Code restarted onto PID89997;
  after the entire scan settled (CPU0%, RSS6048KiB at10m39s after restart), Problems
  pane reports no problems. CUA completion: Verified a clean workspace scan.
  This is settled verification, replacing the earlier transient cleared count.
- Whole formatter0 files/snippets/fingerprints; both UI generators and diff pass.

Styled Button WIP also has a successful native gallery build. Bundle atomically
replaced and reopened, retaining launchd ownership. Desktop proof: click count1;
click+Space+Enter count4; disabled click remains4 with50% opacity; re-enable,Tab,
Enter reaches5 with native focus ring. Ordinary buttons now expose button roles
rather than implicit toggles. CUA does not expose the final page controls in its
AX dump; manual keyboard/visual proof plus upstream Base Button role contract,
not a claim of a separate AX audit. Last native CUA completion: Verified native
styled button behavior. Current preview remains open on that example, enabled,
count5. Styled changes are validated for their separate source checkpoint. Remaining
style work: audit unsupported native utility effects and inline text styling; do
not claim browser CSS parity. No CUA phase open now.

Remaining sessions: unsigned local macOS package smoke94211 (release GPUI build).
Other checks completed; collect remaining sessions as needed. Logs in /tmp are
disposable after this evidence. Package source/config in /tmp/tsrust-package-smoke;
config defaults id dev.tsrust.smoke/version1.2.3/repo example/smoke, CLI overrides
name to Override Smoke and output to cwd-relative artifacts. No signing/publishing.

### Gallery LSP index repair — 2026-11-10 (accepted8a3c74a8)

Latest accepted local source da2b668b contains Questionnaire/Settings/custom menus;
84c64577 contains Speech. Current LSP changes validated as follows:
- Real header gallery index/request: 1/1,8.33s total including90.43ms header setup.
- Real checked gallery index/request: 1/1,58.29s harness including48.198s compiler
  graph setup. Header build5.996s, facts77ms, application1.686s, references2.179s.
  Both retain Route hover and its bundled source definition. Index deadline stays
  30s after setup; compiler setup has a separate180s watchdog. No benchmark claim.
- LSP lib25/25 (.19s), checker agreement5/5 (295.91s), typing matrix3/3 (226.18s),
  editor consistency30/30 (84.17s), intelligence331 relevant tests passed, TSRX6/6
  (.77s). 400 ordinary editor tests plus the two real gallery tests pass.
- The native backend catalog also ran because the initial skip named its helper,
  not its #[test] wrapper backend_catalog_examples_match_native_output. It failed
  at HttpApp.listen under the known sandbox restriction (147.38s harness including
  native builds). This runtime-only gate is unaffected by LSP changes. Every
  catalog key outside std:ui is byte-equivalent as parsed JSON to 4e766220, whose
  retained full native catalog passed (including resumed HttpApp onward1492.20s).
  Reuse that completed evidence; do not rerun the unchanged heavy runtime catalog.
- Both consumers rebuilt successfully (15.64s); whole formatter zero files/snippets/
  fingerprints, generators and diff check pass. No new runtime measurements or CI.

Root causes/fixes: importing each prop field rescanned duplicate record owners and
remapped every record in its dependency. Unique record names are collected once,
field identifiers once, and only referenced types are remapped/imported. Successful
nominal declaration spellings cache per SourceIndex revision/source/span; generic
arguments still resolve separately. Declaration/field/native docs borrow one raw
catalog entry instead of cloning the module. Generic receiver substitution stays
in builtins_for. Project source indices consume the project graph's checked facts
and skip redundant standalone compiler checks; ordinary single-file construction
is unchanged. Checked fields apply only in their owning module, with foreign
shapes imported on demand; copying all std:ui fields into each importer cost~4.3s
per module. Structural fields filter by their own source; named inherited fields
follow the owner declaration. No artificial size caps or weaker type coverage.

Evidence: old live server84953 stayed99% CPU >30min, footprint/peak809MB. Its sample
was ProjectIndex.checked -> apply_project_checked_facts -> project_type_spelling.
Separate samples localized duplicate standalone checks and per-page record imports.
New tests/gallery_index.rs use the actual84-module gallery once per case, separate
setup from a bounded index/request phase, and run no separate native programs.

Live refresh: VS Code restarted onto the repaired binary (new PID20030, old84953
exited). Input.onValidate shows current behavioral help and No Problems. Inspection
keystrokes briefly typed into a selected attribute; all were undone, unsaved flag
cleared, original onValidate restored and git diff confirms Input.tsrx unchanged.
Last CUA completion: Verified refreshed TSRX editor help. A subsequent live sample
of20030 showed workspace_scan.Cache.check_cancellable -> compiler analysis, not
indexing. Whole-workspace compiler checking is still active; do not claim idle or
use transient No Problems as proof it settled. Monitor later; no CUA phase open.

Owner noticed two previews. Confirmed processes: original53885 ./target/gpui-starter
(~7h old) and current1217 /private/tmp/Tsrust Gallery.app/Contents/MacOS/gpui-starter.
Explained that the original was preserved and Tsrust Gallery has current changes.
Do not close the original unless the owner asks. Current gallery remains open on
Input, custom menu field='Hello from the menu', independently of Claude.

Next: finish formatter/local checkpoint for this LSP section, then native classed
Button keyboard/focus/a11y and native text/style limits, unsigned packaging smoke,
final all-target checks and obsolete output cleanup. Existing full compiler/LSP
milestones remain evidence. No push/release/agent work authorized or needed.
Research: gpui-base-0.9.0/src/button.rs provides an unstyled native Button that owns
keyboard/focus/a11y and implements Styled, InteractiveElement and
StatefulInteractiveElement. Use it for the raw-Div classed core Button path, sharing
utility rule application with styled_rules. Styled Kit Button installs its own
hover and debug-asserts if a second one is supplied. Preserve href/labels/ids,
disabled/tab settings, press handlers and rich children. kit_button also calls
.toggled(false) even when toggled is absent: guard on presence to restore normal
button accessibility semantics. No UI renderer source changes for this next slice
yet. Microphone capture, other OS targets and signing remain unverified; full Kit
coverage remains1939/2147,20 types incomplete, not complete parity.

### Forms desktop acceptance — 2026-11-10

Speech checkpoint 84c64577 is local. Current Questionnaire/Settings section adds
QuestionInput freeform answers, initial selected choices, live disabled questions
and choices, root sizes/shortcuts/default question and committed/full submit events.
Legacy onSubmit remains a map to choice arrays; detailed events include confirmed
freeform and status for enabled questions only. Schema rejects duplicates, multiple
single-answer defaults and multiple freeform inputs. Definition changes are mount
semantics, not a live schema editor.

Settings adds sizing, sidebar width/range, initial page/group, group variants,
sidebar/search-header and page-heading styles, page icons/default expansion,
search keywords, vertical rows, dirty/onReset, title suffix and group footer slots.
Reset reuses SettingField.on_reset, retaining weak root references. Arbitrary child
controls keep their own size/disabled/value semantics. Input custom menus compose
existing ContextMenu around contextMenu=false and reuse controlInput actions.
225 wrappers, 71 pages/357 examples. Mapping 1939/2147 across 185 Kit types;
20 types remain incomplete. This is substantial coverage, not full API parity.

Acceptance: UI runtime 136/136, 0.04s (build/setup 4.33s); compiler/LSP binary
build passed 3m06s; TSRX LSP 6/6, 3.26s. Public suites compiled together once:
10/10, 2.65ms. The initial new fixture incorrectly created state inside its test
callback; moved construction into a normal component per the state-owner contract,
then reran the affected public suites. Native gallery build passed and its detached
bundle was atomically replaced/reopened. Whole formatter: one public fixture first,
then zero files/snippets/fingerprints on the final run; generators and diff clean.

Desktop proof: Free default survives size/disabled updates; Team becomes enabled,
selection advances to the input; blank required text blocks submit with an error;
confirmed notes plus Team appear in detailed JSON. Settings startup keyword keeps
its row visible; dirty Switch exposes Reset All, reset restores on; numeric 14 ->15
then reset ->14. Icon, sidebar, footer/title slots and vertical layout render.
Speech unsupported/disabled buttons expose disabled accessibility states, and size
changes render without capture. Actual recording/OS permissions remain unverified.
Input right-click shows custom actions; Replace with greeting updates the same field.
Gallery currently open on Input, custom menu field='Hello from the menu'.

New issue found while verifying the editor: existing server PID84953 remained busy
for >22min. Bounded one-second sample /tmp/tsrust-ui-forms-lsp.sample shows
ProjectIndex::checked -> apply_project_checked_facts -> project_type_spelling,
not compiler diagnostics. Physical footprint 809MB, peak809MB; RSS15MB reflects
compression, not real footprint. No memory-limit workaround applied. Inspect
project_generic_intelligence.rs Named spelling (repeated LineIndex/symbol searches)
and global fact application before refreshing server; transient No Problems did
not prove settled analysis. Fix this narrow performance defect next, with a real
gallery project witness and existing editor agreement tests. Preserve all form
passes; no need to repeat native acceptance for an LSP-only repair. Still pending:
classed-button accessibility/native text/style limits, unsigned packaging smoke,
final all-target checks and obsolete output cleanup. No pushes/agents/measurements.

### Forms continuation — 2026-11-10 (native acceptance pending)

Local accepted input/replay checkpoint is f5673ab7. Current Speech section adds
size/disabled/showWhenUnsupported, mount-time systemFallback/stopTimeout, waveform
bars/liquid and started/cancelled callbacks. systemFallback=false takes precedence
over an explicit locale, leaving no system recognizer. stopTimeout rejects invalid
durations before mount and finishes with the transcript so far (onFinal), matching
Kit; it does not emit Cancelled. New gallery availability preview never starts
capture. Button/Waveform public builder coverage is complete; SpeechState input
injection and cancellation remain unmapped. No full API parity claim.

Compiler/LSP binary build passed 2m59s. UI runtime 134/134 passed, 0.04s; LSP TSRX
6/6 passed, 2.98s. Public navigation+composition compiled together once: 9/9 passed,
2.34ms execution. Whole formatter changed one public suite, zero snippets and
fingerprints; generators/diff check pass. Native compilation and visual proof are
pending and will be batched with Questionnaire/Settings form changes. Microphone
capture, OS permissions and platform recognition remain unverified; do not start
recording or accept permission prompts silently. Keep detached gallery open.

Next implementation: Questionnaire freeform inputs, sizes, defaults/disabled,
shortcut/navigation/events and practical Settings sizing/sidebar/layout/reset/slots;
custom input menu via existing ContextMenu composition. Then rebuild both consumers,
focused checks, one native gallery build and desktop proof before acceptance.

### Latest accepted section — 2026-11-10

This entry supersedes the older in-progress input/Table notes below. Local WIP
945a0ae9 contains synchronous Input/Textarea/CodeEditor paste decisions, validation,
NumberInput stepping and source grammar aliases. The follow-up avoids replaying
successfully checked diagnostic roots; the real borrowed Table regression still
reports ownership and independent return errors within its unchanged 20s bound.
The shared replay observer uses Arc/Mutex to preserve Program's Send property.
Unsized NumberInput now retains a usable 240px input width; constrained controls
fill their allocated width.

Acceptance: compiler/LSP binaries rebuilt (2m45s); compiler recovery groups 37/37
passed (14 recovery, 6 suppression, 10 project, 2 terminal, 5 UI authoring), with
execution 0.85/0.06/2.71/0.04/23.82s respectively and setup/build 2m44s. LSP TSRX
6/6 passed in 3.00s. Public navigation+composition suites compiled together once:
8/8 passed, 2.31ms execution. Runtime UI 133 active passes across retained runs.
Final native gallery build passed, bundle replaced atomically and reopened.
Whole formatter changed zero files/snippets/fingerprints; generators and diff check
passed. Earlier full compiler/LSP milestone passes remain valid evidence; final
all-target compilation after the shared Program change remains pending.

Desktop proof: validation rejects the ninth character, permits clearing; paste
reports text/path/image-count metadata; consume=true prevents insertion while
updating callback state. Custom stepping gives .5 -> .6 -> .5 -> 0 (clamped).
The final stepper width displays its value. Diff highlights .tsr keywords/strings.
VS Code restarted onto the refreshed server and shows No Problems. The separately
accepted hover fence alias fix remains installed and visually verified. Gallery
is running independently of Claude. No pushes, remote checks or measurements.

Next: finish practical Questionnaire/Settings/Speech options and custom input menu
composition, native style/accessibility gaps, unsigned packaging archive smoke,
final affected checks and output cleanup. Full upstream API parity remains open;
1843/2147 methods mapped, 27 types incomplete. Do not describe this as complete.

### Owner review continuation — 2026-11-10 (in progress)

Active request: resolve all seven gallery/platform comments and finish missing Kit
coverage, including the outstanding language defects. Starting source is local
9df57e42 plus Claude's interrupted TSRX wording and Tailwind edits; preserve them.
One writer, no push/release authorized. The latest owner test rule excludes build
and setup from the under-one-hour execution budget, superseding older workflow prose.

Checklist (acceptance pending unless explicitly recorded):
- TSRX wording throughout documentation, diagnostics and editor help.
- Tailwind utilities/arbitrary and reactive values; actual native rendering,
  dark/responsive/data selectors, interaction states and documented native limits.
- Route/path parameters/state, independent per-window state and shared opt-in state.
- Packaging highlighting and checked config defaults with CLI precedence.
- Status-bar edges, sidebar badge clipping, compact title-bar controls.
- Missing Diff, Editor, Questionnaire, Settings, Speech, AppMenuBar and Plot tooltip;
  input callbacks and honest full public-method coverage.
- String-field ownership ergonomics, pathological Table checking, early invalid
  control nesting diagnostics and lazy reactive-list captures.
- Affected suites, whole-project format/token/idempotence proof, gallery interaction
  verification, local accepted checkpoints and output cleanup.

Starter reopened independently of Claude as /tmp/Tsrust Gallery.app. Its executable
is a regular copy of examples/gpui-starter/target/gpui-starter; replace it atomically
when rebuilding, then close/reopen this app through CUA. Do not stop the owner's
pre-existing ./target/gpui-starter process (historically PID 53885). Desktop launch
works; sandbox-only launch fails in NSPasteboard. Gallery remains open for inspection.

Source checkpoints: 36149186 preserves Claude's interrupted edits and the first
review batch; ccfae53b accepts navigation/window ownership and diagnostic recovery;
e981bfbc preserves the compiled application-menu/editor section. All are local.
Current uncommitted section is reactive Kit classes and native viewport/styles.

Implemented since the WIP:
- createLocation(initial) gives each window an independent navigation/history owner;
  routeParam(path, pattern, name) and Route support named nonempty path segments.
  Node captures navigation context; command queues and asynchronous replies retain
  their originating window through weak owner identities. Closing a window discards
  only its pending commands. Full builtin help and navigation guide examples added.
- Signal writes advance a runtime revision; affected roots refresh bindings and
  request other windows to redraw. Signal/Computed function parameters now enter
  the existing reactive lowering scope without becoming additional state owners.
  Manual testing found the missing parameter lowering; the rebuilt gallery now
  passes shared-state redraw in both directions (child increments to 1; owner to 2).
- Editor fallback imports documented bundled source functions without waiting for
  project analysis, preserves their true source definition and parameter docs, and
  respects lexical shadowing. No native identity is fabricated.
- Known built-in controls/blocks inside TSRX Text/Button now reject before an
  unvisited route runs, using linked component identity. Custom components retain
  runtime validation; Button render replacements retain their own child contract.
- Table checker slowdown reproduced with the real gallery page after removing
  string clones: cloned source checked in 1.997s; borrowed version exceeded a 20s
  stop and was killed. A one-second sample showed semantic diagnostic replay
  repeatedly starting in bundled wrappers, not unbounded allocation (about 63 MB
  RSS at 4s). Recovery now prioritizes user-source functions only, while bundled
  bodies still check through real calls. The bounded CLI regression preserves both
  the Table ownership error and an unrelated boolean return error.

Completed acceptance (build/setup recorded separately from execution):
- UI runtime 122/122, 0.04s execution; setup/build 47.62s including lock wait.
- Packaging config defaults/CLI precedence/path/error unit test passed (<0.01s;
  original build/setup 2m19s). Actual archive/signing flow not rerun.
- Compiler diagnostic recovery 14/14, 0.58s; UI authoring 3/3, 15.27s including
  real bounded CLI diagnostics. Build/setup 2m33s. These cover static nesting,
  received Signal getters and the real Table replay regression.
- Latest affected LSP run: lib 25/25, editor_consistency 30/30 (29.74s), TSRX
  editor 5/5 (0.04s). Build/setup 4m05s including shared-target contention. The
  focused routing transport test also passed independently (9.85s).
- Public navigation suite 2/2 on formatted source: 1.284ms total execution,
  individual cases 0.019/1.038ms. Build/setup 146802.7ms (including shared-target
  contention), checking 749.2ms and native/link 145719.3ms. No performance claim.
- Native gallery build passed with the navigation/window/revision renderer.
  The further build including received Signal lowering also passed.
- Earlier workspace/all-target and separate backend/all-target checks passed,
  with existing test-helper/deprecation warnings. Fresh workspace/all-target
  checking passed after the language repairs (1m38s). The separate backend-only
  all-target check also passed (2m07s); only existing warnings observed.
- Whole-project formatter: last run 0 files, 0 snippets, 0 fingerprints changed.
  Gallery generation --check passes. Final diff check and checkpoint pending.

Manual CUA proof: two independent counter windows started at zero/first route;
first increment/navigation did not affect second. Notification and close targeted
second only. Surviving first retained state, back worked, and closing it left the
main route intact. Compact title controls and shell highlighting visually verified.
Five component pages added (Diff, Editor, Questionnaire, Settings, Speech); currently
69 component pages / 349 examples plus generated guide pages. These pages are not
full API acceptance. Coverage remains 1686/2147; do not inflate covers metadata.

Latest public navigation+composition suite: 3/3 across two files compiled once;
1.448ms total execution, cases 0.058/0.927/0.126ms. Build/setup 232836.8ms
(checking 1684.2ms, native/link 230575.0ms including target contention). The
composition witness exercises a custom LazyChildren factory containing a reactive
list inside DialogContent and passes native compilation and assertions. Current
DialogContent is eager Child[]; the old lazy-part wording is stale. No remaining
deferred-capture failure reproduced with this direct witness. Latest CUA confirms
shared-window redraw in both directions and child close preserving owner state.
Main gallery is raised on Navigation and Windows, shared count 2, no child windows.

Next: checkpoint the accepted routing/editor/language-recovery section, then
finish Tailwind native gaps (animation,
text transform/tracking/balancing, nonuniform grid, interaction effect composition,
viewport sizing), missing Kit APIs and remaining ownership ergonomics. Strings use
Cow<'static,str>; retaining dynamically owned row text requires an owned copy, while
literal Cow clones retain static borrows. Do not hide an allocation with a global
string representation redesign. Original string clones remain an accepted interim
workaround, not a claimed ergonomic fix. No benchmark measurements or remote runs.

### Application menu and editor section — 2026-11-10 (implementation, acceptance pending)

Accepted routing/editor-recovery checkpoint is ccfae53b (local, not pushed), after
36149186 WIP. Working tree now implements the next section:
- AppMenuBar/AppMenu typed wrappers and a new Application Menu gallery page.
  macOS uses GPUI system menus; Windows/Linux use Kit AppMenuBar. Existing
  MenuItem/Submenu entries dispatch named Shortcut commands in the active window;
  checked/disabled changes reload menus. Popup-only custom rows/onSelect/href
  reject explicitly. Hiding the owning bar clears installed menus. Native state
  and renderer mappings are not compiled yet; no desktop acceptance claimed.
- CodeEditor supports uncontrolled defaultValue without resetting edits on render,
  controlled value, editing/display/folding/indent/search props, focus/blur/submit
  events and shared controlInput/readInput routing by id. Snapshots add language.
  Native shared input macros are reused; no second editor implementation.
- Editor byte counts/tab sizes validate at the Kit prop/binding boundary, including
  binding refresh. Added public composition test for invalid counts. Gallery adds
  a local-buffer editor with focus/select/find/read actions and a line-number toggle.
- Catalog/typed wrappers generated; currently 216 components and gallery 70 pages,
  351 examples. Covers entries record real newly wired methods only; substantial
  Editor low-level APIs and all remaining Diff/Settings/etc gaps stay open.
- Before-project-analysis property completion needed real bundled parameter record
  fields. The source-function fallback now shares one immutable source index per
  bundled module, rather than synthesizing signatures without their records. Uses
  existing import_declaration field/definition/help logic. Regression added for
  CodeEditor alias completion, behavioral hover and source definition. Not yet run.

Section validation so far:
- UI runtime 122/122 passed (0.04s execution, build/setup 2.36s).
- Public navigation+composition suite 4/4 passed, two files compiled once. Total
  execution 1.369ms; cases 0.056/0.890/0.110/0.039ms. Build/setup 189567.5ms,
  check 1974.1ms, native/link 186963.6ms including shared-target contention.
- TSRX editor integration 6/6 passed (0.97s, cold immutable bundled index included;
  build/setup 2m13s). Full affected LSP lib 25/25 (0.13s) and
  editor_consistency 30/30 (29.35s) also passed; build/setup 1m04s.
- Generator checks pass. Whole-project formatter changed 0 files/snippets/
  fingerprints. API coverage implementation count is 1776/2147, not accepted yet.
- Native gallery build initially failed at two bridges: private GlobalState module
  access, and EditorMode does not implement inline-token insertion. Corrected the
  import to the public GlobalState re-export and specialized the shared input
  command macro's token operation; CodeEditor reports an explicit UI error for
  insertToken, as now documented. Other shared input commands still use one macro.
  Compiler rebuild passed (2m22s); the repaired native gallery retry passed.
- Workspace/all-target check passed (1m30s); separate backend/all-target check
  passed (2m53s). Both report the existing test/runtime-helper warning set.
- Local WIP checkpoint e981bfbc preserves this section. Gallery restarted successfully
  from its independent app bundle. macOS File > Save example updates the page;
  View > Show guides toggles state; Disable Save reloads its menu definition.
  Navigating to Editor removes the installed File/View menus.
- Desktop Editor proof: replacing the local buffer with `fn edited() { println!("Kept"); }`
  and toggling line numbers preserves the text. Read editor state reports language
  rust and current selection/buffer. Find fn opens the search field with fn.
  Disabled-menu click rejection/checkmark appearance still needs direct verification;
  AX omits their disabled/checked attributes. Other desktop platforms remain pending.

### Reactive Kit styles and native utility rendering — 2026-11-10 (in progress)

Working tree adds ReactiveValue<string> className to all generated Kit wrappers,
using existing nativeBindClass. Style refresh retains control identity, local state,
handlers and weak ownership. Kit disabled props now participate in disabled utility
styles. Guide/gallery and public composition witnesses cover the new behavior.
Native viewport lengths use actual window viewport pixels; Kit internal raw class
surfaces now filter dark/responsive predicates and custom spacing. New runtime
witnesses cover viewport resolution and reactive Kit lifecycle (124/124 passed,
0.04s; compiler rebuild passed 2m28s).

Further renderer fixes: stable identities for styled surfaces and active styles;
preserve axis overflow and scoped SVG color; compose hover/focus ring/shadow/gradient
inputs over base styles (including clearing a ring without losing the base shadow).
Shared interaction-merge regression passes. Text child bridge moves an owned Cow
string with into_owned rather than copying it again; explicit borrowed-field clones
still remain.

Validation so far:
- UI runtime 125/125, 0.04s execution; setup/build 1m52s including compiler lock.
- Generated helper initially failed compound negative-guard narrowing. Replaced it
  with separate undefined/string branches; compiler rebuild passed 2m31s.
- Public navigation+composition suites 5/5, one runner for both files. Execution
  1.64ms; setup/build 18474.9ms (check 2066.2ms, emit 334.7ms, backend 54.2ms,
  native/link 15721.7ms). Reactive Kit className and disabled state pass.
- LSP lib 25/25 (0.14s), editor_consistency 30/30 (29.93s), TSRX editor 6/6
  (0.99s), setup/build 9.30s. Whole-project formatting changed zero files/snippets/
  fingerprints; both generators --check and diff --check pass.
- Native gallery build passed. Bundle replaced atomically and reopened. Desktop
  proof: Tag padding/background/foreground change live; Switch disabled opacity
  paints, and its local on value survives disabling/re-enabling and class changes.
  The runner used its default example-local shared native Cargo target rather
  than the repository target because CARGO_TARGET_DIR was omitted. Future native
  and host invocations must explicitly set CARGO_TARGET_DIR to the repository
  target. Prune redundant example-local native outputs after active builds end.
  Example target is about 45GB, repository target about 156GB (logical du sizes).

This section was checkpointed as caa69ba1. Continue text-transform/unsupported-native diagnostics, Plot
tooltip, Diff and remaining Kit API families plus string-field ergonomics. Full Kit
API coverage and ownership ergonomics remain open. No performance measurements,
remote runs, pushes or releases. Gallery is open on Styling and Themes, live styles off and Switch locally on.

### Table diagnostic repair follow-up — 2026-11-10 (active)

Input callback native gallery build PASSED (/tmp/tsrust-ui-input-native.log); it
has not yet been copied/reopened or manually verified. Final validation script
12403 stopped at ui_authoring: 4/5 passed, only the existing bounded Table error
case hit 20s. Preserve the four passes. LSP/public steps did not run. Compiler
final rebuild passed 2m48s; added alias unit 1/1 passed (<0.01s), so runtime total
133 active tests passes across retained runs. No source implementation changed
after that build; only the public suite gained nonfinite NumberInput/negative step
witnesses (suite step still unrun).

A standalone exact Table-shaped retry (14406, CLI 69870) also exceeded 20s and
was killed, so this is not merely parallel-test contention. One-second sample
/tmp/tsrust-ui-table-check.sample shows semantic_diagnostics::collect ->
analyze_recursive_with_proofs; RSS is bounded, but replay is real. Current
semantic_diagnostics.rs still has the earlier bundled-root exclusion. Investigate
which skipped diagnostic/root triggers recursive inference/repeated replay; do
not raise the timeout or weaken ownership/independent-error coverage. Preserve the
valid native callback build while repairing this compiler regression. A WIP
checkpoint of callbacks is appropriate before changing diagnostic recovery; whole
formatter currently session 22925, log /tmp/tsrust-ui-input-wip-format.log.
Next: finish format/generator checks and local WIP checkpoint, add bounded/failure
trace if needed, diagnose/fix replay, rerun only this failing test and affected
compiler recovery tests, resume unrun LSP/public checks. Then manual Input/paste/
step/Diff grammar checks and accepted checkpoint. No pushes/agents.

### Synchronous input decisions — 2026-11-10 (implementation, checks running)

Latest local source checkpoint 701a74dc accepts expanded Diff display/review slots.
Current dirty section adds reusable synchronous predicates to Kit, Input.onValidate,
Input/Textarea/CodeEditor.onPaste (borrowed JSON {text,paths,imageCount}, no image
bytes), NumberInput.onValidate and stepBy(value, increment). Callbacks preserve
errors; paste consumes when true and redraws after the decision. Pure validation
keeps Kit's empty-string allowance. Numeric steps reject negative/nonfinite output.
Weak captured command contexts target the originating window without retaining it;
Node predicates enter their weak navigation context. NumberInput no longer resets
custom stepping during sync and numeric options retain f64 precision. Boundary
validation rejects nonfinite NumberInput options and negative fixed steps.

Also fixes source grammar aliases: tsr/tsrust -> typescript, tsrx -> tsx in editor
and Diff. Diff selects the native highlighting feature even without an editor or
Markdown. New parse_diff helper sets per-file grammar overrides without changing
file names. Full API help, generated wrappers and LSP fields updated; gallery adds
Input validation/paste and value-dependent NumberInput stepping (71 pages /354
examples). Mapping 1843/2147, 27 types incomplete; no completeness claim.

Completed: runtime 132/132 in 0.04s before the last source-alias unit was added;
includes callback errors, released state, invalid steps and weak window-context
routing. Input and NumberInput actual gallery pages check with zero diagnostics.
Initial compiler rebuild passed 2m49s. Current final coherent validation script
session 12403, set -e, explicit CARGO_INCREMENTAL=0/CARGO_BUILD_JOBS=1/repo target:
1. compiler CLI rebuild /tmp/tsrust-ui-input-build-final.log;
2. added alias unit /tmp/tsrust-ui-input-alias.log;
3. ui_authoring 5 tests /tmp/tsrust-ui-input-authoring.log;
4. LSP TSRX 6 tests /tmp/tsrust-ui-input-lsp.log;
5. public navigation+composition suites compiled once (8 tests expected),
   /tmp/tsrust-ui-input-public.log.
At last observation step 1 was still compiling. Preserve terminal passes; on a
failure repair/rerun affected checks and resume unrun steps. Native GPUI gallery
compile + desktop proof still required for the new macros/closures, then formatter,
generator checks, checkpoint. Do not modify source mid-build absent a real repair.

Original broad catalog retry 62502 FINISHED PASS: 1/1, 1492.20s harness including
many native setup/builds. Earlier 477 LSP tests and remaining 61 targets passed;
full LSP milestone is now complete. Compiler library milestone remains 1541 active
pass, 8 unchanged ignored; workspace/backend checks passed. No need to restart
these full passes for wrapper-only edits. Current callback bridge needs its focused
checks and native proof. Retain owner test execution <1h, setup/build separately.

Gallery /tmp/Tsrust Gallery.app currently runs 701a74dc, on Diff split/details off;
code in the working tree is newer and not desktop-accepted yet. VS Code hover fix
00a44a19 remains installed/loaded. Do not close the owner's other starter process.
Next after acceptance: custom input menus/practical Questionnaire, Settings, Speech
options; native style/accessibility limits; packaging archive smoke and cleanup.
Research already localized: native menu builder is gpui_base input/base/native.rs,
NativeMenu.menu/menu_with_disabled/separator; existing TsrustCommand + Shortcut can
reuse application commands. Questionnaire base supports optional InputState answers,
disabled/default-selected choices, sizes and change/completed events; existing
wrapper only choices. Settings builder options and Speech size/disabled/fallback/
timeout/waveform are still mostly absent. Read pinned sources before implementing.

### Diff display and review surface — 2026-11-10 (accepted: 701a74dc)

Hover fence fix checkpoint is 00a44a19, local/not pushed. The installed local
VS Code extension is 0.2.1 with tsr/tsrx aliases, visually verified and loaded.
Current dirty section expands Diff with line numbers, syntax toggle, file headers,
hover/highlight/marker/wrapping choices, context/expansion/inline comparison limits,
line click/hover and add-annotation JSON callbacks, plus DiffAnnotation,
DiffDecoration, DiffGutterMarker and DiffHeader child slots. Counts and positions
validate at the runtime boundary. Malformed patches now report parse errors instead
of silently becoming empty. Retained state setters skip unchanged values. Header
slots repeat read-only content per file; per-file interactive headers require
separate Diff nodes. State command/query APIs remain incomplete. No full Kit claim.

222 wrappers, 71 component pages / 352 examples, mapping 1837/2147, 27 incomplete
types. Native compile passed and the detached gallery binary was atomically
updated/reopened. UI count boundary test 1/1 passes (<0.01s; setup/build 1m16s),
LSP TSRX 6/6 passes (2.31s execution). Compiler rebuild passed (2m50s final); native
gallery build passed. One-second sample during the build showed Command::output
waiting on native Cargo, not semantic replay; remove disposable sample later.
Whole-project formatter 0 files/snippets/fingerprints; both generators --check and
diff --check pass. Original broad catalog retry remains active (62502) and has
progressed into UI example native compilation; retain earlier passes below.

Desktop proof: annotation text, custom header suffix and translucent range render;
marker click reports Comment opened. Switching split/unified preserves annotation
and state. Line numbers/change backgrounds turn off while explicit range coloring
remains. Clicking modified source line 2 reports {path:greeting.tsr,side:modified,
line:2}; annotation button reports inclusive modified 2..2. Gallery left on Diff,
split on, details off. Pinned Kit does not infer .tsr syntax highlighting in Diff;
source grammar alias registration should be fixed in the next native/editor slice.

Next: input synchronous callbacks and remaining practical Questionnaire/Settings/
Speech options, native style/accessibility gaps and packaging smoke; then final
acceptance/cleanup. Borrowed row string clone is still the accepted interim
workaround; dynamically retained strings need an owned copy with current Cow
storage. No new performance claims or remote validation.

### Hover example highlighting — 2026-11-10 (accepted)

Plot/live-array/menu section checkpoint is 4e766220 (local, not pushed).
Owner reported plain white TSRX hover examples. Builtin docs already carry fenced
Markdown; VS Code only registered the tsrust language name. Added tsr/tsrx aliases
to that same grammar; no documentation rewrite or alternate syntax engine.
Extension compile, executable tests 4/4 and real TextMate tests 2/2 pass (0.29s
combined test execution). Packaged local 0.2.1 VSIX, installed over local 0.2.0,
reloaded the owner's VS Code window, and visually verified Route hover: keywords,
component tags, attributes and strings are colored. Source and open documents
unchanged. Gallery remains open independently. The broad catalog retry below is
still running and making progress through stdlib native example compilation.

### Plot tooltip and live array props — 2026-11-10 (final checks running)

Latest accepted checkpoint is caa69ba1 (reactive Kit classes/native viewport styles),
local and not pushed. Current working section is uncommitted. /tmp/Tsrust Gallery.app
now runs this working section’s successfully compiled binary, copied atomically
and reopened through CUA. It is on Plot, offset 1, glide off. One writer; no pushes,
releases or performance measurements.

Implementation:
- PlotTooltip/PlotTooltipRow use GPUI Kit’s Plot hover state and overlay: title,
  automatic colored series rows, explicit plain/colored/value-colored rows,
  crosshairs, dots, glide/fade, progress, gap and appearance. Marks add label.
  Missing trailing series values are omitted; no hover work without a tooltip.
- Plot labels use internal JSON to preserve empty/newline labels. Mark values
  reject NaN/infinity; stroke/gap reject invalid geometry. Painting and hover
  share normalization for finite extreme ranges and grouped-bar coordinates.
- All generated Kit array props accept ReactiveValue<array>, retain evaluated
  getters for state expressions, and keep the constant path for plain arrays.
  Live props admit scalar arrays/nested arrays; Computed/useProp stay scalar-only;
  record/resource arrays still reject. Ordinary array bindings remain invariant.
- Array literal/Array.of/spread contexts retain a sole array alternative through
  union injection, without selecting an ambiguous array or tuple alternative.
- Native empty callback captures use their concrete uninhabited type, without
  unused generic bounds. Empty/live callback calls share the existing receiver
  logic, preserving captured mutable reborrows. Public regressions exercise both.
- Fixed disabled native application menus: GPUI macOS validation follows action
  availability. Disabled entries/labels now use NoAction, matching the pinned
  upstream set_menus example; parent disabled state propagates to children.
- Complete generated help and a new Plot gallery page. 218 wrappers; 71 component
  pages / 352 examples. Mapping count 1787/2147, 28 types incomplete. Tooltip’s two
  upstream type names share one inventory row; no full Kit coverage claim.

Completed validation (setup/build separate from observed execution):
- UI runtime 128/128, 0.05s; setup/build 41.18s including lock wait.
- Public navigation+composition, one shared runner: 7/7 in 1.80ms. Individual
  cases ~0.09/0.91/0.15/0.06/0.04/0.05/0.01ms. Build/setup 77226.2ms
  (check 2478.8ms, emit 422.5ms, backend 66.2ms, native/link 73896.7ms).
- Full compiler library: 1541 active tests passed; 8 unchanged opt-ins ignored.
  Initial sandbox run: 1511 pass, 30 permissions failures, 256.22s harness time;
  only those 30 rerun with local listener/filesystem permissions, all pass 0.57s.
  Cargo setup/build 6m13s. No code failure in the library run.
- CLI 49/49: 48 passed in the initial 3.22s run; the one outdated Plot wrapper
  export expectation was extended for the two new item wrappers and passes 0.01s.
- ui_authoring 4/4, 21.96s (setup/build 18.79s), including the new array contexts,
  invariant existing arrays, record/computed rejection and original bounded Table
  diagnostic recovery. Earlier accepted routing/nesting/shared-signal witnesses retained.
- Whole-project formatter last changed one public-suite file; zero documentation
  snippets/fingerprints. Both generators --check and diff --check passed.
- Native gallery build passed using the explicit repository Cargo target.
- Desktop CUA: disabled Save is marked disabled, stays inert and preserves its
  original message; re-enabling restores Save. Navigating away clears File/View.
  Plot Mon shows Requests 2/Completed 1/Updates 0; Wed shows Requests 4 only plus
  Updates 0. Update data changes Tue to Requests 8/Completed 7/Updates 1 and Mon
  Requests 3. Disabling glide preserves values. Crosshairs, colored hover dots,
  plain row/value color and tooltip edge flipping visibly work.

Running / pending:
- Full LSP initial run: 477 tests passed before the intelligence target’s catalog
  example test hit sandbox-blocked HttpApp.listen. Other 331 intelligence tests
  passed. Preserve these passes. Setup/build 49.31s. Resume only that catalog
  test with TSRUST_CATALOG_RESUME_FROM=HttpApp and local permissions: session
  62502, /tmp/tsrust-ui-lsp-catalog-retry.log.
- Remaining 61 LSP targets after intelligence: all passed (exit 0), including
  the expanded TSRX completion/hover/definition regression.
  /tmp/tsrust-ui-lsp-remaining.log. Initial tail run accidentally lacked resource
  variables; stopped its own Cargo/children before any tests ran and restarted
  with explicit limits. Prune the aborted target/debug/incremental outputs later.
- Workspace/all-target and separate backend/all-target checks both passed;
  /tmp/tsrust-ui-workspace-check.log and /tmp/tsrust-ui-backend-check.log.
- Whole-project format rerun: 0 files/snippets/fingerprints; both generators
  --check and git diff --check passed. Catalog retry still running at checkpoint.
  Do not rerun already passed groups unless changed inputs justify it.

Resource hygiene: official cache clean removed 2092 root artifacts (105371441251
bytes) and 99 example artifacts (49078328212 bytes), preserving running private
copies and the repository Cargo target. Every future command explicitly sets
CARGO_TARGET_DIR=/Users/seancotter/Desktop/dev/learning/tsrust/target,
CARGO_INCREMENTAL=0 and CARGO_BUILD_JOBS=1. Raw /tmp logs are disposable: summarize
terminal results here, then delete them before handoff; no imported raw receipts.

Next: collect the three running sessions, repair only affected failures and
checkpoint this section. Continue Diff, remaining input/editor callbacks,
Questionnaire/Settings/Speech APIs, borrowed string-field ergonomics, and feasible
native typography/interaction styling. Native style gaps remain: text case,
tracking/balance, animations, nonuniform grids and flattened nested text; do not
claim full Tailwind rendering. Core classed-button focus/keyboard behavior also
needs review: raw classed Div buttons may lack a focus handle. No changes for
those gaps have been implemented yet. Other desktop platforms/web and signing/
archive packaging are unverified. Keep the gallery available for owner inspection.

### Gallery as Kit docs mirror; API coverage tracking — 2026-11-09

Owner: the starter must mirror GPUI Kit's component docs (left nav, every Kit
example live with copyable .tsrx) and every component must cover GPUI Kit's
full public API, not just the docs. `tools/kit-coverage.py` inventories public
methods from the locked gpui-component/gpui-base sources (184 types, 1765
methods; gpui-component types shadow same-named gpui-base primitives) against
kit.json `covers`; `--check` gates completeness. Done so far: Button (all but
tooltip_with_action, which needs a std:ui actions/keybinding model shared with
menus), ButtonGroup (Button children), Toggle, ToggleGroup. Gallery: generated
from examples/gpui-starter/pages.spec by tools/generate-ui-gallery.py; custom
title bar (renderer now honors titlebar: 'custom'), tabbed Preview/Code cards,
Gruvbox theme set via new registerThemes/useTheme; std:ui utility colors now
follow the active Kit theme. Icons: all 1830 Kit catalog names (IconName is a
string; a literal union stalled rustc); programs naming non-default icons
register AllAssets. Display batch: Badge, Tag, Label, Kbd (command prop reads
Shortcut bindings), Separator, Skeleton, Spinner, Progress/ProgressCircle,
Avatar/AvatarGroup, Alert, Icon (svg, rotate), ShimmerText, Empty/EmptyMedia,
GroupBox/GroupBoxFooter, Clipboard and Link (disabled) are complete, with 15
gallery pages. Forms batch: Checkbox, Switch, Radio (also standalone),
RadioGroup and Rating are complete with 4 pages (coverage 374/1765). Styled Kit components in STYLED_KIT take
className as their own style refinement instead of a wrapper. Fixed: every
ScrollArea shared one scroll offset (Kit keys Scrollable state by call site
without an explicit id); gallery routes now each own a scroll area; sidebar
groups merge across the spec; utilities gained rounded-md and rounded-full.
Owner review fixes (examples must work when exercised and read like shadcn):
Kit text content comes from children (`<Label>Name <Text variant="secondary">
(optional)</Text></Label>`, `<Checkbox>Accept</Checkbox>`, `<MenuItem>Copy
</MenuItem>`); kit.json `textChildren` mirrors kit.rs `text_children_prop`
(generate-ui-kit.py checks agreement). Text gained `variant`. Alert takes
AlertTitle/AlertDescription; Empty takes EmptyHeader/EmptyTitle/
EmptyDescription/EmptyContent and EmptyMedia variant="icon". Form controls are
uncontrolled when `checked`/`value`/`selected` is unset (defaultChecked,
defaultValue, defaultSelected set the start). DropdownMenu is a plain button
that opens the menu; onClick or split makes a split button. Remote image src
loads through a reqwest HTTP client (gpui-http feature, selected for Avatar and
Image). Empty Avatar names no longer hide the placeholder icon. Colors: utilities
and Kit color props accept the Tailwind 4.1 palette (gamut-mapped to sRGB hex)
and arbitrary [#hex]/[oklch(...)] values. Every gallery page was exercised
(menus, split button, context menu, checkboxes, switches, toggles, ratings,
alerts). Inputs batch: Input and Textarea map GPUI Kit's Input/Textarea and
InputState/TextareaState (masking, masks, number formatting, regex pattern,
search and replace, rows/auto-grow, wrapping, tabs, focus/blur/submit events,
inline tokens); imperative calls go through controlInput(id, action) and
readInput(id, onRead) over the command queue. InputGroup is now shadcn-style
composition (Input/Textarea child plus InputGroupAddon > InputGroupText /
InputGroupButton, which opens a menu from MenuItem children); NumberInput gained
min/max/step (default 1), affix icons, sizes and defaultValue. Four new pages,
all exercised. tools/kit-coverage.py now matches braces lexically (its
test-module stripper had swallowed real impls) and maps InputBaseState aliases,
so the honest total is 701/2147. Still open on inputs: validate/step_by and
on_paste need synchronous TS callbacks; custom context menus and custom token
rendering. Select and Combobox are complete with pages: options are SelectItem
children (value + text label, disabled) optionally in SelectGroup with a
SelectLabel, or the items prop; SelectEmpty supplies empty-list text (Kit renders
it from &App, so text only); Combobox adds multiple, query, checkIcon and
ComboboxTrigger / ComboboxFooter slots. Language limits found: reactive props
must be scalars (array props are static; Combobox value is newline-separated)
and owned callbacks are callable only as parameters (kitOnList adapts
string[] events). Tabs (Tab/TabsContent by value,
prefix/suffix/empty-space slots, folder variant, menu), Tooltip (keys, command,
TooltipContent), Popover (open/defaultOpen/onOpenChange, anchor, offset, arrow,
right-click, overlayClosable) and Accordion (value/defaultValue,
AccordionTrigger/AccordionContent, styles) are complete with pages; the
Example card now uses value-based Tabs. Coverage 856/2147 (Tooltip's open
methods belong to the Plot chart tooltip). Owner direction 2026-11-09: finish a
page for every component, then add semantic search to the gallery, rebuild
Getting Started from the real docs as the single source of truth, and add an
animated showcase page (use the design-taste-frontend skill). Since then: Dialog and Sheet
(Trigger/Content/Header/Title/Description/Footer/Close parts), AlertDialog,
notify options and events, HoverCard, Collapsible, Breadcrumb, Pagination,
Stepper, DescriptionList, Slider, OTP, TimeField, Calendar, DatePicker,
ColorPicker, and a layout batch (Sidebar with nested items and item menus,
SidebarToggleButton, Resizable, Toolbar groups, Form/Field, StatusBar,
TitleBar close, WindowBorder) — each with exercised pages. Utilities gained
justify-end/around/evenly and items-end/baseline. Coverage 1300/2147. Remaining
types: Table/DataTable/List/Tree states, Command, Carousel, Diff, Editor,
Attachment, Bubble/Message/MessageScroller, Marker, Questionnaire, Settings,
Speech, AppMenuBar, the Plot tooltip, and input sync callbacks (validate,
onPaste, step_by) plus custom input context menus and token rendering. Then
semantic search, Getting Started from real docs, and the showcase page.
Next batch (2026-11-10): Command palette (groups, separators, shortcuts,
header/footer/empty, controlled query, loading, highlight vs select events),
Carousel (CarouselContent/CarouselItem/Previous/Next/Pagination, loop,
vertical, controlled `selected`), composable Table (header/body/footer/caption,
per-part className, align, colSpan, size) and List over a custom ListDelegate
(ListItem rows render their own children, ListItemSuffix, ListSeparator,
ListGroup sections, ListEmpty, search, confirmed/check icon, controlled
selection/query, hasMore/onLoadMore) and Tree (TreeItem text children,
icons, defaultExpanded, disabled, controlled `selected` that reveals hidden
items, onExpand/onCollapse, TreeMenu right-click menu with onContextMenu). TSRX
children accept spread arrays (`{...rows.map((row) => <Row />)}`); a spread
that reads component state lowers to a reactive list slot (runtime
`Node::each`: transparent like branches, rebuilt on refresh, rows whose shape
is unchanged keep their identity and state; `visible_children` flattens
slots). Spec sections may define interfaces before the example function, and
generate-ui-gallery.py rejects unknown icon names (a typo rendered nothing).
Fixed while exercising: List section headers overlapped rows (Kit measures
header height from section 0), disabled List rows still reported, confirmed
rows drew no check (Kit needs an explicit check icon), Tree `selected` could not
reveal collapsed items, Carousel parts ignored className, Sidebar header/footer
`collapsed` needed Kit's Collapsible trait in scope. Open defect: in the gallery Table page, consuming a
read-only callback parameter's string field twice (`name={person.name}` and
`{person.name}` inside nested cells) made `tsrust check` run for minutes in
analyze_recursive_with_proofs instead of reporting callback-access; small
repros report the error in under a second, so the pages use clone() for now.
Reading a string field in a consuming position should copy rather than
require clone().
Second batch (2026-11-10): DataTable is composable (DataTableColumn children
with key/width/min/max/sortable/defaultSort/align/fixed/resizable/movable/
selectable/padding; rows as JSON; row, column or cell selection; sort, column
move/resize, right-click and DataTableMenu, empty/loading/load-more events;
controlled selectedRow). Chat family: Message parts (MessageAvatar/Header/
Content/Footer, MessageGroup, a Bubble inside follows the message side),
Bubble variants/BubbleContent/BubbleReactions/BubbleGroup, Marker parts,
loading style and shimmer, Attachment parts (AttachmentMedia src or icon,
AttachmentContent/Title/Description, AttachmentActions, tooltip, vertical,
size), AttachmentGroup edge fade, MessageScroller styling props. New Ticker
component (GPUI timer, onTick(frame), interval, running) drives animation.
Command gained CommandItem `value` and a Command-level onSelect(value).
Gallery: Getting Started pages are generated from docs/guides/ui/*.md and
docs/guides/tooling/desktop-apps.md (single source; ```tsx live Name``` blocks
compile into the gallery as live examples); title-bar search (⌘K) ranks
pages and examples over a generated index with concept synonyms ("modal"
finds Dialog); an animated Showcase page (60 Hz plot, 160-cell wave field,
spectrum, orbits, auto carousel). Fixes: `tsrust new --ui` template did not
compile against the current API (now checked by a scaffold test); runtime UI
errors exited silently with status 0 because GPUI exits inside its run loop
(now printed as Uncaught and exit 1); Plot colors ignored palette names;
arbitrary pixel offsets (left-[150px]) are supported. Coverage 1686/2147.
Remaining Kit types: Diff, Editor, Questionnaire, Settings, Speech,
AppMenuBar, Plot tooltip, SearchableListItemElement, input sync callbacks.
Known limits found: Kbd/controls inside Text or Button are rejected only at
runtime; lazy part children (DialogContent) cannot own a reactive list's
captures, so build rows in a helper function.

### Plot and Speech — 2026-11-09

Owner: wrap the whole GPUI surface (low-level included) and trust GPUI behavior;
verify our wrapper. Plot/PlotLine/PlotArea/PlotBars paint through GPUI Kit's plot
primitives (shared band x and linear y scale, axis labels, dashed grid).
SpeechButton (locale, waveform; onPartial/onFinal/onError) uses SpeechState with
SystemRecognizer; `gpui-speech` (Kit `speech`: cpal and platform audio, 36 locked
additions) is selected only when a program uses it, alongside `gpui-highlight`.
`tsrust package` writes NSMicrophoneUsageDescription and
NSSpeechRecognitionUsageDescription. `ui add` follows multi-item children lists.
Plot rendering checked; dictation itself not exercised. 112 components.

### GPUI highlighting, charts and setup — 2026-11-09

`gpui-highlight` feature (GPUI Kit syntect: +syntect, two-face, fancy-regex,
bincode, bit-set 0.8, bit-vec 0.8; existing pins unchanged) is selected only when
the emitted program uses CodeEditor (runtime marker); CodeEditor and Markdown code
blocks highlight. Chart gains radar, candlestick (open/high/low/close) and sankey
(links). Fixed: `setup --dependencies` fetched only the backend set, so fresh
machines could not build std:ui apps offline; it now fetches the GPUI set too, and
the starter's dev.sh runs setup first. Windows/Linux GPUI
runs remain unverified here. Tests: the test harness now defaults
CARGO_TARGET_DIR (repository target) and TSRUST_SOURCE_PACK_DIR (Cargo home packs)
like CI; four shared_sources tests had failed on every local run without them.

### GPUI CLI, Questionnaire, Svg and doc rewrite — 2026-11-09

Owner: no signing or published release; users run the source. The starter has
`tsrust.config.json` and `examples/gpui-starter/dev.sh` (builds this checkout's
compiler, then `tsrust dev`: ~20 s rebuild and restart on save; cold first build
~3 min). `tsrust new <name> --ui` copies the starter; `tsrust ui list|add` writes
editable wrappers (component plus its item components) into `src/components/`;
all 75 wrappers check and a generated project builds. Questionnaire/Question/
QuestionChoice (submit emits JSON answers) verified by input. Fixed: Svg/Path drew
nothing on GPUI; they now render as SVG images with resolved theme colors
(`Node::svg_markup`). Route, Link, navigate, back and CopyButton verified on GPUI.
The 27 std:ui IntelliSense entries and api.tsr comments no longer describe the
removed browser/Vello renderer; every std:ui entry now has a tsrx example, and
the LSP catalog tests accept tsrx fences and optional console examples for std:ui
(they had failed since std:ui returned). The runtime_build dependency-reuse test
only asserts a shared Cargo target when CARGO_TARGET_DIR is set (it failed on
every local run without it). 107 components.

### GPUI services, packaging, updates and starter — 2026-11-09

Branch `codex/gpui-ui`. Since the catalog pass: application services (notify,
window control, clipboard, file dialogs, extra windows, appearance, theme),
Icon/Animate, font utilities, i18n (`t`/`tWith`/`translate`/`translateWith`,
JSON catalogs, plurals, live locale switching, Kit strings follow), catalog
batch 2 (Markdown … ButtonGroup) and batch 3 (InputGroup, Diff, Marker,
Attachment/AttachmentGroup, DataTable, Settings pages/groups/items,
MessageScroller) and Dock/DockPanel (drag, stack and split verified): 104 generated components. Kit 0.9.0 has no GridLayout (docs
are ahead of the pinned crate). Not wrapped: Plot
(closure drawing API; Chart covers data plots), Questionnaire, Speech (needs the
`speech` feature and cpal), and code highlighting (syntect/tree-sitter features).
`tsrust package` (cli/app_package_cli.rs) builds release, writes the `.app`
(icon, Info.plist, app.json), optional codesign/notarytool/stapler, zip and DMG on
macOS, zip on Windows, tar.gz and .deb on Linux, plus SHA256SUMS. The std:ui
updater (tsrust-ui/src/update.rs; `appInfo`, `checkForUpdates`, `installUpdate`,
`relaunch`) reads GitHub `releases/latest` through system curl, verifies the
archive digest and macOS signing team, and swaps in place. Check verified against
the live API; install path verified up to the missing asset; packaged `.app`
launches. `examples/gpui-starter` is the demo (seven pages, en/fr, dark mode,
About & updates), verified with real input. Guide: docs/guides/tooling/desktop-apps.md.
Open: Developer ID identity and notarytool profile choice, a release repository
for the demo (owner approval before publishing), end-to-end install from a real
release, the remaining components above, CLI `new --ui`/`ui add`, and rewriting
the restored std:ui docs.

### GPUI Kit catalog pass — 2026-11-09

Branch `codex/gpui-ui`. One spec (`stdlib/ui/kit.json`, 62 components including
items and slots) drives `tools/generate-ui-kit.py`, which writes the typed `std:ui`
wrappers (`kit.tsr`, formatted) and their IntelliSense entries; `--check` guards
drift. The runtime has one generic Kit node (string props, live props through
text bindings, typed events `kitOn{,Boolean,Int,Number,Void}`); `gpui_kit.rs`
holds the renderer arms by pattern (display, controlled, Kit state entities,
overlays through a weak root handle, item containers). Compiler fixes found on the
way: literal and literal-union template interpolation (`TsrTemplateText`), and
text bindings over literal-typed getters. Native cache readers now wait up to 2 s
for short copies by other processes (dev reuse flake under parallel tests).
`experiments/gpui-kit-admission/tsrust/gallery.tsrx` renders every component
(screenshots checked). Lib 1541/47/25/6, tsrust-ui 108.
Open: event and overlay interaction proof (headless GPUI test platform, since OS
input injection is unavailable while the owner uses the machine); GPUI rendering
for Svg/Path, Link navigation, CopyButton and visibility callbacks; rewrite 27
restored std:ui docs that still describe the removed browser/Vello renderer; then
CLI (`new --ui`, `ui add/list`) and the starter.

### GPUI bridge: first slice — 2026-11-09

Branch `codex/gpui-ui`. Owner chose to restore the checked reactive lowering
(signals, computed, text bindings, branches, controlled inputs) rather than add
generic typed native callbacks first. `std:ui` keeps its name; `crates/tsrust-ui`
again holds the Node model, utility classes and theme without any renderer;
`stdlib/ui/gpui.rs` renders Nodes through one GPUI root view (Kit Button/Input
for unclassed controls, utility classes mapped to GPUI styles). `gpui-dependencies/`
is the backend manifest plus GPUI Kit with its own lock (892 packages, backend pins
unchanged). UI window programs keep the OS main thread. `check` hides bundled
`std:` contracts. Slice in `experiments/gpui-kit-admission/tsrust` matches the
Rust reference; startup 244 ms vs 253 ms median. Lib 1541/47/25/6, tsrust-ui 107.
Next: Textarea, focus/active states, Kit theme sync, then CLI (`new --ui`,
`ui add/list`), catalog and starter. Raise CI job timeouts to ~3 h with the next
CI change (owner: only test execution has the one-hour limit).

### Rust 1.99.1 pin — 2026-11-09

Branch `codex/gpui-ui`. The backend and repository pin move from 1.96.1 to exactly
1.99.1 (GPUI Kit needs post-1.96 APIs). Exact, not minimum: the rustdoc JSON reader
is format-specific, now format 61 with a regenerated trimmed shlex 2.0.1 fixture.
Active docs, scripts and tests moved; measurement records keep their recorded
toolchain. Also moved the ownership witnesses used by tests out of the deleted UI
experiments into `crates/tsrust-compiler/tests/fixtures/owner-handles`, and
recorded the GPUI bridge design in the platform plan. Local acceptance on 1.99.1:
clippy correctness; lib 1540/47/25/6; host targets acceptance, async_io,
native_dependency_bundle, project_commands, rust_adapter_generation, rust_bridge,
rust_data_boundaries, toolchain_installation, named_handler_specialization,
tracked_resources; seven pinned LSP targets; 134 runtime suites (415 s, cold cache).

### GPUI section: earlier UI removal — 2026-11-09

Branch `codex/gpui-ui`. Both earlier UI attempts are removed: `crates/tsrust-ui`,
`std:ui`/`std:ui-runtime`, UI analysis/emit, `ui_project`, the UI CLI, dev web
and docs app (`docs/src`, `docs/content`), the UI catalog, UI examples, tests and
benchmarks, and the React/Electron/Expo/shadcn traces. The `.tsrx` parser,
formatter and TSRX syntax stay; TSRX lowers to plain component calls. The native
dependency manifest dropped its UI crates (571 to 326 locked packages) and native
support always builds from the shared backend set. `tsrust ui`/`new --ui` report
GPUI Kit support in development. Fixed in passing: LSP snapshots record
`tsrust.config.json` as a loaded source (previously implied by UI config reads);
`testing_runtime` stubs the capture reset; four test helpers lex with the new
`tsrust_compiler::source_tokens`.

Local acceptance: compiler lib 1540, CLI 47, LSP lib 25 and bin 6; 134 runtime
suites (370 s, 0 failed); affected host targets (acceptance, interactive_cli,
native_cache, native_dependency_bundle, native_package_export, package_management,
project_commands, router_contract, testing_runtime, toolchain_installation,
url_features) and LSP targets (stdio, api_project, editor_consistency,
completion_matrix, checker_agreement, project_intelligence) pass; API audit PASS
at 1,583 identities (102 UI rows removed); clippy correctness, format check and
tooling tests pass. Next: toolchain minimum-version policy, then the GPUI bridge.

PR #16 CI fix `f324a669` (Windows `normalize` output, UTF-8 suite output, no `tsc`
in host tests) is pushed; its CI run is pending.

### Outstanding-issue fixes and v0.2.0 preparation — 2026-11-09

- Fixed: `captureOutput` now ends its capture in `finally`, so a throwing
  callback no longer swallows later output in the same test (the earlier
  error-effect failure was the missing effect-table entries, now present).
- Fixed: no compiler or library-test warnings remain (unused console imports;
  test-only runtime includes marked as such; `pin_project!` import annotated).
- Fixed: `tsrust dev` uses `sigaction` (persistent handler, explicit restore)
  instead of raw `signal()`, removing libc-dependent reset semantics behind the
  unexplained Linux dev-session exit failure.
- Fixed: HTTP server tests retried no rebind after shutdown; parallel tests can
  receive the freed ephemeral port. All five sites wait up to 1s.
- Doctests: both crates have zero; executable example coverage is the generated
  catalog suites, so CI does not spend a doctest build.
- Ledger: 1,685 identities PASS (five new testing identities reviewed).
- Version: tsrust-compiler 0.2.0, tsrust-lsp 0.3.0, extension 0.2.1 unchanged.
  Release notes and the status page are updated. All v0.2.0 plan boxes are checked.
- Local revalidation: compiler library 1,546 (twice), CLI 54, project commands 6,
  toolchain installation 25, acceptance 8, std:testing catalog suite 12.

### CI build-once restructure — 2026-11-09

Owner: CI must compile once per OS and run `tsrust test`; consolidation instead
of 1:1 is fine; no PRs/pushes until the restructure is done; pushes exist to
merge toward release. Plan: `docs/plans/ci-build-once.md`.

- New `std:testing` output capture: `captureOutput`, `startOutputCapture`,
  `finishOutputCapture` (runner resets unfinished captures; effect table updated).
- `tools/generate-catalog-suites.py`: 1,066 generated tests from verified catalog
  examples (42 host-only by rule), one directory per group.
- `tools/run-suites.py`: 133 suites, one native build each, PASS locally
  (cold 17.7 min, warm 301s). It found and fixed a real defect:
  `tsr_console_table` was private, so `console.table` failed in test programs.
  The rust-regex example's stale bindings lock was re-locked.
- `tools/host-tests.py`: 21 boundary targets. CI rewritten: lint, lib, build per
  OS (release artifact), suites and host per OS on that build, editor, CI gate.
  Removed: shard runner, platform runner/evidence chain, supporting-host timing.
- About 530 legacy per-case integration targets remain as local deep-regression
  witnesses, outside CI.

### Full runs 3–4 — 2026-11-08 21:45 UTC

- E0460 root cause: relocated vendored builds recompile shared crates
  (proc-macro2, cfg_if) in place in the shared target. Direct-rustc tests now
  link a private nix closure (`target/test-externs`); the CI libnix step was
  removed. JSON staging test: the runtime rlib is hash-named; test fixed.
  Relocated/doc budgets raised. Every shard has passed in at least one run; the
  last failures were cold-build timeouts.
- Owner flagged the hourly CI loop. Do not dispatch another full run without
  approval; one confirming run on the final commit remains the only
  full-suite item. Platform acceptance is complete (C9).

### C9 complete; full run 2 — 2026-11-08 20:00 UTC

- C9 42511473967 at 81f7beed: focused platform acceptance COMPLETE on all four
  hosts (61 targets each, zero failures). The Windows Ctrl+C fix worked. B00
  admission, B01/B03 Windows rows, B20 native acceptance and "finish B01–B19"
  are checked.
- Full run 2 42511475678: shards 0, 1, 3, 4 and 7 PASS; every shard finished
  within its hour (max 56 min). Remaining (all pass locally):
  native_buffered_file_backend E0460 (shared-target `cfg_if` rebuilt by tests
  that run their own Cargo builds), native_json_build staging,
  project_commands dev exit (status now printed), and the frozen-UI emit budget
  (40→120s). Next: full run 3; fix whatever recurs.
- Remaining plan: clean complete pass, release archives in the notice audit,
  prune, owner approval.

### First sharded full regression — 2026-11-08 19:30 UTC

- Run 42503200899 (8 shards, 2fd6e5e1-era HEAD on `codex/stdlib-0.2.0-full`):
  every shard finished within its one-hour job (slowest 54 min). Failures,
  diagnosed and fixed locally:
  - map-epoch `while(true)` loops emitted `while !stop && (true)`, so a function
    ending in such a loop failed rustc with `()` instead of i64. A literal-true
    loop with no break leaving it now emits `loop` (`breaks_out` uses resolved
    control targets). Test sources with genuinely dead statements after
    divergent loops were corrected (3 scoped-record cases).
  - Stale fixtures: exception_handling inlined runtime lacked `report.rs`;
    cache callback rejection is now `cache-payload`; `Option<T>` is supported
    (B19), so it is no longer a rejection case; json.test.tsr now expects the
    early 413; the empty generic call diagnostic is `array-type`.
  - Pretty reporter printed absolute assertion paths; sites now render relative
    to the working directory.
  - CI environment: crates.io `url` was absent offline (the workspace patches
    it), so it is now in `tests/dependencies`; a restored stale `libnix` broke
    direct-rustc tests (dropped before the shards run); "Complete regressions"
    was skipped after an earlier step failure (`!cancelled()` added); two cold
    budgets raised.
  - Local: all affected targets PASS, including scoped_record_composition 207
    and the map suites.
- C8 42503198395 sealed (Linux/ARM/Intel complete). The Windows helper died
  with STATUS_CONTROL_C_EXIT. It now ignores Ctrl+C process-wide and stays
  attached for one second after sending, so its own copy of the event is
  delivered while ignored.

### C6 review and measurement completion — 2026-11-08 16:45 UTC

- C6 42494556962 at 3972fa49: Linux, ARM and Intel macOS repairComplete=true
  (61 targets, packaged smoke PASS). Windows' only failure is the SIGINT witness:
  the owned console helper now dies on its own event, because it inherits the
  restored normal Ctrl+C processing. The helper now also sets the process-wide
  ignore. Sealed in LATER_RUNS (Linux 9e39f0fc…, ARM 5a4fc8a2…, Intel 98908cee…,
  Windows af07e6fb…).
- Ingested 240 entries (80 workloads × 3 engines); integrity ok. The page has
  1,165 comparisons; the generator uses the manifest `rustReference`.
- Owner: benchmark only Node-comparable operations, not integration wrappers.
  HTTP upload/middleware timing and the native HTTP refresh are dropped.
- Site 78 pages, links PASS (128 pages).
- Open: Windows SIGINT (C7), the sharded full regression run, package admission,
  B01/B03 Windows rows, prune, release approval.

### Plan completion pass — 2026-11-08 15:30 UTC

Owner: keep going until the whole v0.2 plan is implemented; CI matters but is
not the main goal. v0.2.0 merge/publication is still not approved.

- C5 42485215015 at a341f7ff terminal: Linux/ARM repairComplete=true (61 targets).
  Windows' only remaining failure is the real SIGINT witness (listener never saw
  Ctrl+C although the helper's GenerateConsoleCtrlEvent succeeded). Intel reached
  41 targets, then the relocated realtime case hit the 55-minute scope stop. All
  four results sealed in `LATER_RUNS` (Linux 33f3a296…, ARM 1be2637e…, Intel
  86cca64f…, Windows ef36ee03…); chain verified locally.
- Windows SIGINT: the CI launcher starts the test owner with Ctrl+C ignored, an
  attribute every child inherits even with CREATE_NEW_CONSOLE. The test owner now
  installs its own ignore handler and restores normal processing
  (`SetConsoleCtrlHandler(None,0)`) before spawning the listener. Unproven until
  the next Windows run.
- Generic sibling regression fixed: generic argument preparation retried only
  `generic-inference` errors, so `identity([])` aborted before the sibling
  `identity([7])` supplied T. A context-free `array-type` error is now deferred
  the same way. Witness `empty_array_generic_arguments_take_context_from_generic_siblings`.
- Host export ledger: `docs/plans/host-export-ledger.md`; verify-api-audit PASS
  for all 1,680 identities. Language inventory reconciled (80 probe-backed
  reclassifications, 1,355 disposition notes; generator links repaired).
- Probe defects fixed: `'a'.localeCompare('b','en')` panicked (receiver rewrite
  reached std:locale without preparing its records). Locale and regex receiver
  rewrites now share `prepare_receiver_binding`, which also reports arity by the
  public member name instead of `__stringCompare expects 3 to 3`.
- Notice audit: five Android/wasm-only backend crates had no license file in
  their archives; exact upstream MIT supplements are added by packaging and
  portable export (distribution tests 15 PASS).
- Full regression fit: CI `full_regressions=true` now runs eight parallel shards
  (`tools/regression-shards.py`, weights from the 965de83f run, every target on
  exactly one shard; shard 0 owns lib/bins/doc/format/lint). Not yet run.
- Measurement gap found: 80 registered runtime workloads (75 B02 byte codec/
  operation, 5 B01 file) and 16 B10 upload/middleware HTTP rows have never been
  measured; HTTP request/CPU native pins date from 2026-11-04.
- Measured (not yet ingested): all 80 runtime workloads verified (795s) and timed
  at 3×31 on native/node-tsc/rust-hand, results
  `benchmarks/runtime/results/2026-11-08T15-45-57-371Z.json` (36 codec) and
  `…T15-56-53-554Z.json` (44 operation/file). NEXT: `npm run store:ingest` in
  benchmarks/runtime, verify, add the runs to website/tools/performance-runs.json.
- B10 upload reference fixed: the hand-written Rust server never compiled
  (`form_urlencoded::Serializer` held across an await made `forward` non-Send).
  The serializer is now scoped; manifest pin updated; guard PASS. NEXT: run
  `--uploads --verify` then `--uploads --prepared`, the same for `--middleware`,
  then the native-only requests/CPU refresh (scripts http-*.sh pattern in this
  note's commands), ingest, update pins.
- C6 focused platform run 42494556962 at 3972fa49 dispatched (later chain
  42480586761 42485215015). Review all four artifacts when terminal; then
  dispatch `full_regressions=true` (eight shards) — same-ref runs cancel each other.

### C4 review and C5 dispatch — 2026-11-08 13:45 UTC

- C4 42480586761 at 3d6767c5 terminal: ARM and Linux repairComplete=true (all 61
  targets, zero failures). Windows reached all 61 targets with 118 new passes;
  remaining failures: the real SIGINT delivery witness and LSP async_tasks/
  native_threads/rust_bridge. Intel reached 41 targets, 17 new passes, then scope
  stop. All four results sealed in `LATER_RUNS` (Linux 720c3bd4…, ARM 377d65d1…,
  Intel 13a54555…, Windows e14c5b60…).
- Windows SIGINT root cause: the runner started test binaries with
  `CREATE_NEW_PROCESS_GROUP`, so every descendant inherited "ignore Ctrl+C" and the
  generated listener could never observe CTRL_C. Windows cleanup never used the
  group (owned handle plus exact PID/birth rows), so the flag is removed on
  Windows; POSIX sessions are unchanged. Guard test added.
- Windows LSP failures: tests used Unix-only absolute ids (`/async/main.tsr`,
  `file:///…`). The three files now derive ids from host-absolute URIs (Windows
  gains `C:`), unchanged on Unix; local async_tasks 30+3, native_threads 18+12,
  rust_bridge 8+8, service_sections 3+3 PASS (96.8s).
- Intel root cause: the compiler test harness counted time spent waiting for the
  serialized heavy-build lock against each command's own timeout (180s). The
  command budget now starts after the lock is held; the wait has its own 1800s
  bound. Platform scope 55 min/job 60 min are unchanged.
- Runner: ordered `LATER_RUNS` chain (`--later-repair-plan`, workflow input
  `platform_later_prior_runs`) requires every earlier seal unchanged. Python 27 PASS.
- C5 dispatched: run 42485215015 at a341f7ff (prior/second/third + later
  42480586761). PRs/Dependabot were empty before push. Do not dispatch another
  run while it is live; download all four artifacts when terminal, seal them in
  `LATER_RUNS`, and repair only diagnosed failures.
- Remaining B00–B20 work after C5: four-host package admission and B01/B03
  Windows rows from C5 evidence; the generic sibling-context regression above;
  making the complete compiler/LSP suites fit one hour (historical 11,754s case
  time plus per-target builds; settle repair saves about 1,000s; next levers are
  consolidating integration targets into fewer binaries and the 777s
  checker_agreement case); the per-export host ledger (1,247 inventory identities
  lack a review entry; `tools/verify-api-audit.py`); measured HTTP metric refresh;
  docs/help/install/update/archive audits; pruning; then release approval.

### Suite-time and catalog repairs — 2026-11-08 13:20 UTC

- LSP test `Session::settle` now stops 50ms after every open document's current
  version is published, keeping the old 900/400ms quiet window as fallback when a
  version publication is not observed. Assertions are unchanged. Actual local
  timings versus the 965de83f record: completion_matrix 341.5s (was 732s, 3 PASS),
  editor_consistency 36.5s (was 234s, 38 PASS), intelligence 176.1s (was 570s),
  checker_agreement 777.4s (unchanged 759s; not settle-bound, 6 PASS).
- Catalog: 65 non-UI entries had verified `examples` but no documentation fence,
  failing `backend_catalog_examples_match_native_output` at HttpClientOptions.ca.
  Each now shows its first verified example, formatted by the current compiler,
  with `Expected output:`. The test then PASSED alone in 679.3s.
- OPEN regression found, not yet fixed: `contextual_generic_array_calls_keep_checked_parameter_and_result_help`
  fails because `check` rejects
  `function identity<T>(value:T):T{return value;}function first<T>(left:T,right:T):T{return left;}function main(){const values:number[]=identity([1,2]);const selected=first(identity([]),identity([7]));console.log(values.length,selected.length);}`
  with `array-type: An empty array needs an array annotation or call/return context`
  at `identity([])`. Sibling generic-argument context no longer reaches the nested
  generic call. Candidate commits touching generic inference: 8b1d4e9f, 91428ba4,
  e339008f (2026-11-07). Bisect or trace `generic_call_work.rs` parameter contexts.
- C4 run 42480586761 at 3d6767c5: ARM and Linux SUCCESS, repairComplete true,
  all 61 targets, zero failures (new passes 6/5; retained 546/552). Results
  SHA prefixes 377d65d1f095 / 720c3bd41c0d. Intel/Windows pending at this entry.

### Resumed continuation — 2026-11-08 12:30 UTC

Owner resumed the goal. Hosted run 42474075588 at a7795eea is terminal FAIL on all
four hosts; all four artifacts were downloaded, SHA-sealed and reviewed. This
continuation repairs its diagnosed causes. Full B00–B20 stays incomplete;
v0.2.0 merge/publication is not approved.

- Actual C3 results (focused repair, not full-source acceptance): Linux 4 new
  passes (relocated logging 357.8s, relocated realtime 392.9s, nominal projection
  2.1s, LSP IO docs 11.8s); ARM 0; Intel 0; Windows 232 new passes across 22
  targets, reaching 41 of 61 targets before the 55-minute scope stop. Windows
  build-helper cleanup now works on the host. Results SHA256: Linux 3c28afa0…,
  ARM 849cf27d…, Intel 8ef7dc2d…, Windows 5f3be802…; each retains the C1/C2 seals.
- Root cause of almost every failure on all hosts: `Validating native support
  workspace …: URL patch payload is incomplete.` The Actions cache restores
  `target/tsrust-native` without some registered URL patch files, and
  `backend_sources::restore_missing` restored only the four base files. Repair:
  `native_url_patch::restore_missing` recreates only absent registered payload
  files under the existing workspace lock; changed files and links are rejected
  before any write. Unit witnesses cover partial/whole payload loss and changed
  files. This resolves the earlier unexplained remote ENOENT/missing-path class.
- Actual Windows stack receipt: relocated exported process `program.exe` (debug)
  exits 0xc00000fd with main-thread stack overflow. Reproduced locally: the same
  debug process example aborts under `ulimit -s 1024` (exit 134). Repair: an
  async entrypoint now runs the whole generated `main` (hook setup, result
  adapter, exit code) on a scoped thread named `main` with an 8 MiB stack; only
  `()`/`ExitCode` crosses threads. Local repro then exits 0 with identical output.
  The thread name keeps panic text unchanged. Witness:
  `async_entrypoints_run_on_an_explicit_main_thread_stack`.
- Remaining non-URL timeouts were cold per-command budgets on slow hosts: Intel
  `emit --package` for logging/realtime exceeded 60s (now 180s, matching IO);
  Intel IO relocated build and Windows logging relocated release build exceeded
  300s (all three relocated cargo builds now 600s per profile). Scope 55 min and
  job 60 min are unchanged.
- Runner/workflow: optional reviewed third input `platform_third_prior_run=
  42474075588` (`--third-repair-plan`) retains only C3's own successful cases at
  their a7795eea identity, and requires both earlier seals. Python 25 PASS 0.14s;
  workflow/release policy node tests 23 PASS.
- B19 optimization: equality between a record-union literal tag field and a
  string literal now selects variants (`match &(base) { ViewUnionN::Vk(_) => true,
  … }`) in both emitter paths instead of comparing text. Nested-option workload
  emits zero tag text comparisons; untimed oracle 505733888 matches. Not timed.
- Local actual checks at the working tree (shared target, inc0, jobs1, exact
  prior feature graph http-websocket+logging): compiler library 1544 PASS/8
  ignored; LSP library 27 PASS; CLI 54 PASS (requires explicit CARGO_TARGET_DIR,
  as on CI); schema_unions 14 incl. new witness, rust_bridge 8+8,
  owned_iterators 12, recursive_callable_results 16, generic_value_adapters 92,
  string_operators 6, async_tasks 30+3, async_io 17+4, process 12+4,
  task_ergonomics 14, service_sections 3+3, structured_logging 4+1 all PASS.
  The two relocated export cases need `TSRUST_SOURCE_PACK_DIR=target/portable-source-packs`
  locally, as CI sets it. Each compiler rebuild took about 430s.
- Next: commit, check PRs/Dependabot, push, dispatch one focused run with all
  three prior inputs. Continue remaining B00–B20 work locally while it runs.

### Owner-requested stop — 2026-11-08 11:14:58 UTC

The owner explicitly requested a handoff and stop. Pause the goal; do not continue
implementation or start more validation until the owner resumes it. Full B00–B20
remains incomplete; v0.2.0 merge/publication is not approved.

- Canonical writer: `/Users/seancotter/Desktop/dev/learning/tsrust`, branch
  `codex/stdlib-0.2.0`, HEAD `a7795eea73d631a9f4340c0c5d898a107ac047db`, pushed.
  Preserve the protected app worktree at
  `/Users/seancotter/.codex/worktrees/stdlib-platform-acceptance/tsrust`.
- Pending changes are ONLY `docs/agent-progress.md` and
  `docs/plans/stdlib-0.2.0.md`: actual OS leak/allocation findings and current CI
  evidence. Preserve them. No source edits or new builds followed a7795eea.
- Verified at stop: no local Cargo/rustc/native/leaks process, no live agents.
  Historical build PIDs and vanished sessions must not be restarted.
- Existing hosted run **42474075588** remains LIVE at exact a7795eea, all four
  hosts in bounded native-family/package acceptance. It was already dispatched;
  left running, not cancelled. No additional run was launched for this handoff.
  https://github.com/Forever-Frameworks-LLC/tsrust/actions/runs/42474075588
- First action after resume: inspect this SAME run to terminal status, download
  its host artifacts, inspect actual failures and preserve successful scopes
  with their real identities. Prior two sealed inputs are42460717406@469 and
  42470181942@f46. Current repair retains both separately and is NOT full-source
  acceptance. Windows stack/signal phases and missing-support-path diagnosis
  remain unresolved until actual new host evidence arrives.
- Accepted local runtime/cache checkpoint7ff214ea: corrected one no-run build
  31.080s/697811784B; CLI54PASS; IO18 both profiles plus raw stdioPASS; shared55
  both profilesPASS. Existing compiler1544/LSP27/export proofs retained. Actual
  113 timing-history entries are committed, original93 samples each; unchanged
  Rust/Node controls must not be rebuilt or retimed without a diagnosed reason.
- Latest additional evidence (details below): all four password workloads on
  native/Rust have zero leaked blocks/bytes under macOS leaks at exit, using
  existing binaries. The original inside-runtime allocator rejection remains
  recorded. Current deep-data allocation count18 versus2066; collections5
  unchanged. Material collection timing drift and nested-adapter7x Rust gap
  remain explicitly recorded, not hidden by passing allocation observations.
- Remaining complete-goal work includes final backend binaries/all integration/
  doctest/editor/package/install/update/rollback/archive acceptance, public metric
  and inventory reconciliation, and outstanding plan requirements. Do not claim
  B00–B20 complete from the current focused scopes.
- Full-suite budget audit is READ-ONLY, no optimization implemented or run:
  old `/tmp/tsrust-965de83f-full-integration.json` has544 targets/4619 cases;
  non-UI-prefix509/4503 accumulated10530s case time plus6288s per-target build
  overhead. These are historical accumulated times, not a current wall-clock
  estimate. Do not launch a known oversized suite or split it into serial hourly
  chunks. One Cargo build and complete coherent coverage must fit one hour.
  Largest historical groups: native_package_export, checker_agreement,
  completion_matrix. `Session::settle` still waits900/400ms for quiet; replacing
  it was only considered, NOT implemented or validated. Any future change must
  preserve actual document-version/phase agreement and all existing assertions.
- Public metric refresh entry points located, not executed:
  `website/tools/generate-performance.mjs`,
  `website/tools/performance-runs.json`, and `website/package.json` reference/build
  scripts. Generator reads SQLite and explicitly curated Rust baselines. Add
  accepted current measurements with cached controls; do not fabricate new runs.
- No pushes/releases after a7795eea. Last prepush PR/Dependabot checks were empty.
  Check both again before any future push. Keep this as the single handoff.


Current source checkpoint a7795eea73d631a9f4340c0c5d898a107ac047db, pushed on
codex/stdlib-0.2.0. Runtime/cache repair7ff214ea is locally accepted; a7795eea adds
reviewed two-run evidence reuse and bounded cold-export command adjustments.
Working-tree changes after this checkpoint are plan/progress evidence only.
Hosted42460717406@469 and42470181942@f46 areterminalFAIL. Focused repair
42474075588@a7795eea is RUNNING on all four hosts; do not dispatch another run.
Full B00–B20 goal remains active. v0.2.0 publication/merge is not approved.

- ACTUALC3dispatch42474075588 atEXACTa7795eea, usingbothreviewedpriorruns.
  https://github.com/Forever-Frameworks-LLC/tsrust/actions/runs/42474075588.
  LatestprepushPRs[]/Dependabot[], oldrunsallterminal. All4hostsnowinbounded
  nativefamily/packageacceptance. Scope55min inclusiveofsetup/job60min. Thisis
  focusedrepair, neverfullcurrent-sourceacceptance; publicationstillunapproved.
- OSleakdiagnosisusedEXISTINGcompiledprobesonly, zeronativecompilation:
  all4passwordworkloads×native/Rust report0leakedblocks/0bytes atprocessExit.
  Eightownedscopes1.071–1.590s, same120saggregate ceiling. The3additional
  measuredprogramsperenginehadexactacceptedbinarySHA verified beforeexecution.
  /tmp/tsrust-password-os-leaks.json. This doesNOTerase earlierstrictallocator
  inside-runtime zero-retentionfailure orprovezero livebackendallocations there.
- CurrentB18allocationdiagnosticsACTUALPASS4.213s/283885568B: threeobservations
  each retain0B. Deepdata18allocations/1728requestedB vsold2066/132800,
  removing2048temporaryrecordworkallocations. Collections5allocations/2400B,
  unchangedfromoldprofile. /tmp/tsrust-testing-current-allocations.json;
  benchmarks/runtime/diagnostics/2026-11-08T11-08-53-828Z-native-allocations.json.
  Noallocation-countcause forcollectiontimingdrift established; no controlretime.
- Secondfocusedrepair preparation: all4 C2 artifacts SHA/host/source/parentseal
  verified locally. ActualC2passesLinux2/Windows0/ARM6/Intel127 remainf46;
  C1retained447/0/540/399 remain469. Newoptionalexplicitsecondartifactinput
  preservesbothidentities, ignoresnestedretainedasC2 proof, andskipsONLYactual
  successfulok/noScopeStopcases; oldforcedC1controlsstillrequireactualC2pass.
  Fourhostoriginal61targetselection/guards/package+smoke unchanged; focused
  completefalse staysfalse. Python24PASS; actualall4sealedartifactsPASS.
- TerminalC2Intel47.6min (jobcap60min/scope55min): mostnewnativefailuresbare
  ENOENT plusIOpackageemit60s/logging+realtimeDEBUGCargo180s timeouts. Record
  /tmp/tsrust-repair-42470181942-mac-intel. Logging/realtimeCargo budgets now
  300s EACHprofile, IOpackageemit180s; onlypercommandcoldworklimits changed,
  scope55min/job60min stays. Unchangedlocallyacceptedpayloadsnotrerun.
- Nextremote decision: candidateafter7ffcheckpoint uses54CLI/IO18both+stdio/
  shared55both accepted, priorcompiler1544/LSP27/exportproofretained. Need
  Windowsactualstack/signal+successfulownedhelpercleanup and actualmissingpath
  receipt/resolution onhost caches; selectplatform_acceptance=true,
  platform_prior_run=42460717406, platform_second_prior_run=42470181942.
  OldequivalentrunterminalFAIL; no duplicateactivevalidation, nofullCI/release.
  CheckfreshPRs/Dependabot immediatelybeforepush. Publicationunapproved.
- Focused cache/build repair ACTUALPASS: exactpriorCargo featuregraph restored;
  onecorrectedno-run31.080s/697811784B, terminalbuild-finished success.
  CLI54PASS10.598s includingmissingfile/changedmetadata/symlinkcontrols;
  registeredIO18EACHprofile+rawstdioPASS193.470s; shared55EACHprofile
  PASS130.231s. /tmp/tsrust-partial-support-repair-corrected.json.
  Existing compiler1544/LSP27/relocatedexportsremainaccepted; notrerun.
- Rootmistakepreserved: firstrepaircommand omitted priorhttp-websocket/logging
  features andstartedredundantcompilerlibraryrebuild. Stoppedownsupervisor77436
  /Cargo77519, verifiedsettlement, thenusedEXACTpriorgraph. Bothattemptscharged
  sameoriginal1770sdeadline, noreset. /tmp/tsrust-partial-support-wrong-features-stop.json.
- Closed-outputcleanup removed791files343910222logicalbytes and recovered
  361938944observedfreebytes: aborted/closedloosecodegenobjects plusobsolete
  profilingbinaries/sources only. CurrentmanifestconsumersSHAverifiedunchanged;
  sourcepacks/shareddeps/currentdiagnostics preserved. Togetherwithprior
  8958775296B cleanup, observedrecoverythiscontinuation9320714240B.
  /tmp/tsrust-measured-repair-closed-output-cleanup.json.
- Allocation follow-up finished within ORIGINAL1770s deadline: native8
  iterator/adapter probesPASS5.829s; fourRustallocationcontrolsPASS1.607s.
  All3observations strictzero. Adaptercounts/bytes IDENTICALnative/Rust;
  nested4095allocations/77805B, so no allocationvolumecause for7.00x timinggap.
  /tmp/tsrust-post-allocation-gates.json and
  benchmarks/runtime/diagnostics/2026-11-08T10-37-09-708Z-native-allocations.json.
- Original37allocation scopeFAILED755.902s/588445688B atfirstpasswordprobe;
  noownedprocessesremaining.25syncprobes passedbeforefailure. Strictpassword
  rejection remains: recovered3samplesretain4376–4424B; matchedunchangedRust
  control4264–4376B. Backendstate/retirement plausible, leak/eventualcleanup
  unproved. Remaining3passwordallocationprobesunrun. Existingbinary-onlycounter
  recovery26rowsPASS2.151s afterEXACTemission/instrumentedsourcecomparison;
  initialrecoveryfailed .trim() mismatch BEFOREexecution, preserved.
  /tmp/tsrust-crypto-allocation-counter-recovery-v2.json;
  /tmp/tsrust-password-rust-allocation-control.json (4.299s scopedbuild/run).
- B18native-only originalprotocolPASS fivecorrectnesscases perworkload then
  8warmups/3processes/31samples, quietguardwaited forload<4. Native2groups93samples
  ingestedONCE at2026-11-08T10:38:29.075Z; cachedRust/Nodecontrolsunchanged.
  deepdata .011042ms versusprior .045500, 4.12xfaster/Rustgap1.53x. Collections
  .137666ms versusprior .070750, regression1.95x; no deferredrecordstack in
  emittedcollections, relevantMap/Set/NumericArray/helpersunchanged. Three
  processmedians .146625/.137666/.100292 drift; actualcauseunproved. Do not
  relabelregressionasnoise/pass orretimeunchangedcontrols. Planrecordsboth.
- Partialcache restoration repair nowLOCALLYPASSED: underexisting
  ownedworkspacelock restoresONLYmissingbasefiles; changedmetadata/symlinks
  rejected beforemutation. Contextualvalidationerrors includeactualpath.
  RemoteENOENTcachecauseremainshypothesisuntilcontextualreceipt. Loggingcold
  releasecommandbudget300s afterobserved180s compilationtimeout; scope55min/job
  60min unchanged. Nootherpassedexportsorbackendregressionsrestarted.
- Windows build-helper cleanup repair is in source: only code0 selected-test
  builds may terminate exact-owned background helpers and then require fresh
  empty-tree inspection. Runtime/test scopes still fail on unsettled children;
  no executable-name exception. buildCleanup preserves before/after evidence.
  Existing Python platform suite now23PASS0.125s. Hosted behavior is notyetproved.
- Public37 measurement ACTUALPASS219.179s/404946944B: all555 correctness
  comparisons passed before timing; original8warmups/3processes/31samples,
  quietguard enabled/systemallocator. Actual111 engine/workloadgroups93samples
  ingested ONCE with npm run store:ingest; SQLite integrityok. Canonical minimal
  histories timestamp2026-11-08T10:14:12.271Z. Preserve unchanged Rust/Nodecontrols.
  Raw disposable benchmarks/runtime/results/2026-11-08T10-14-12-271Z.json;
  /tmp/tsrust-current-public-37-measurements.json. B15/B16/B19 plan nowrecords
  actualmedians, including nestedoption7.00x Rustgap andrandomInt8.22x Nodegap.
- Allocation scope is CLOSED; no profiling/native process remains. See actual
  rejected/password and completed iterator/adapter evidence above. Do not
  restart25passed synchronous probes orremeasureunchangedRust/Nodecontrols.
- Focused42470181942 actualLinuxfailure: IO2/task1 instantfailures; logging
  scope319.804s endedwithunsettledchild. Windowsbuildcode0/748.391s blockedby
  genuinecurrentMSVCvctipdescendant, notoldPIDreuse. No nativeWindowscase ran.
  Artifacts /tmp/tsrust-repair-42470181942-{linux,windows}; agentsreviewlogs
  sourceonly. Keepownership/cleanup assertions; no blankettelemetryexemption.

- ACTUAL current compiler library1543PASS/0FAIL/8unchanged opt-ins ignored,
  96.406s/1352483320B. Current LSP library27PASS/0FAIL1.053s.
  /tmp/tsrust-final-extraction-test.json and /tmp/tsrust-current-final-lsp-library.json.
- Native family consolidation: task13, asyncIO18, network18, compression16,
  HTTP110 =175 registered cases. IO/network/compression pass BOTHprofiles;
  task/HTTP retain original default-profile protocols. Strict exact case names,
  original output/typed error oracles, source correspondence, cleanup and no
  unknown output retained. True stdio, no-runtime, cache, TLS, ABI and relocated
  export boundaries stay separate. All owned processes settled; no build live.
- Network BOTHprofiles ACTUALPASS16.694s/753188864B after closed native record/
  finite plain literal option result-discovery repair. Minimal real2file projects
  preserve listen alias and DNS conditionalalias/relay/undefined family; genuine
  void/wrong native argument/invalid literal remain rejected. Recursive-summary
  scalar eligibility unchanged. /tmp/tsrust-final-extraction-test-network-native.out.
- HTTP110ACTUALPASS10.447s/1289913592B, nativebuild8180.098333ms. HTTP emitted
  arguments ONCE; raw text reuses prepared operands, no abandoned conditional
  owner. Compression16EACHprofile ACTUALPASS25.503s/713097216B, nativebuild
  3513.819ms debug/20032.778ms optimized; worker7/5 outputs now correctly tagged.
  /tmp/tsrust-shared-native-context-functional.json.
- Existing6case testing fixture ACTUALPASS6.809s; FIVE child paths (TokioJob,
  async pool, sync pool, dedicated, scoped thread) retain exact case/source
  location/order, no orphan/duplicate output. Profile secondrun cachehit/native0.
  ONE ArcMutex collector shared across extractedruntime; Tokio poll/drop and
  sync worker/drop scope restored. Ended scopes reject late capture to existing
  visible unattributed reporting, never silentlydiscard or leak into next case.
- Complete runtimevariant factories reused by emission/dependency detection/
  extraction. Pure tests prove ONE runtime SCOPE/collector and actual selected
  modules, including async-only runners. No new dependencies or unsafe code.
  Non-runner runtime hooks remain passthrough. console_async.rs is committed at04f3df1f.
- Repair failures preserved honestly: originalbuild160.956s failed NEWtesthelper
  unwrap on InputDiagnostic(noDebug); corrected to render API. Sharedbuild
  399.154s then3guardsPASS/1FAIL: test wrongly expected bindingfeatureasync;
  actual async selection follows Task origins/runtime module. Secondbuild
  388.352s then3PASS/1FAIL: test string matcher missed pub/comment/mod syntax.
  Original1770s aggregate closed INCOMPLETE, no fake full acceptance.
  /tmp/tsrust-shared-native-context-repair{,-resume}.json and
  /tmp/tsrust-native-result-final.json retain failures.
- Final correction is cfg(test) ONLY: parses Rust AST for exact public module
  identity/visibility, not formatting. New600s gate rebuilt ONLYcompiler test
  harness226.206s (no tool binaries), corrected guardPASS2.617s, full library
  PASS96.406s. Earlier production consumers unchanged; native network is current.
- CI cache defect fixed: shared Cargo target alone did NOT share standalone
  native support. Platform runner AND CI env now explicitly use
  TSRUST_CACHE_DIR=target/tsrust-native; deliberate per-command isolation still
  overrides. Actual subprocess-environment regression + runner tests16PASS0.081s.
  No trigger/timeout changes;55min platform/60min job remains. Not dispatched yet.
- Contract catalog+guides now document direct imported aliases/stored callbacks
  versus unsupported stored generic all/extracted Iterator method values. Exact
  thread-call negativePASS. Public logging/realtime reference mapping/sitebuild
  PASS4.752s and128page rendered links/assets/anchors PASS1.605s (not deployed).
- Cleanup removed newly-created loose codegen objects only, preserving current
  libraries, metadata, binaries and shared caches. Exact count/bytes:
  /tmp/tsrust-native-context-object-cleanup.json. Prior net recovery oldhost
  1430007808B +obsoletebasepack766394368B remains separate.
- Generated minimal/unused vs ICU consumer build/size proof ACTUALPASS at04f3df1f:
  520872B vs2179640B (delta1658768B), exactoutputs7 and ICU4lines verified;
  first appcachemiss nativebuild2056.4ms/24974.8ms, warmhitsbothnative0.
  Existingstable sources, sharedtarget/cache, no directcontrols/11sample replay,
  no sampled-benchmark/SQLite claim. Temporarybinaries/emittedscratch removed.
  /tmp/tsrust-generated-minimal-footprint.json. B00feature-selection row closed.
- Public37 original30min scope CLOSED INCOMPLETE before timing after genuine
  preparation failures, no child/resource runaway and no accepted timing rows.
  Corrected9stream endindices,3Rust HKDFreference slice calls and4Argon2reference
  PHCstr calls; all29crypto+4iterator workloads then passed5independentcases on
  native/Rust/Node. FirstpureRustadapter failed unconditional injected Rayon;
  rootcompiler dependency-selection repair is required before timing can proceed.
  Preserved validated prepared artifacts with exact compiler/source/ABI/binary
  hashes, not a claimed measurement pass. No samples/SQLite rows were ingested.
  Attempts: /tmp/tsrust-public-iterator-crypto-adapter-{measurements,resume,final,
  password-repair}.json. Originaldeadline was never reset.
- Historicallaunchrecord (nowcompletedabove): Newpublic37 firstactualtiming atf46d7400 afterrealcompilerdependency
  repair. Existing29crypto preparednative/Rust/Node artifacts mustmatchNEWCLI
  emission/ABI/sourcebefore reuse; all37×5cases×3engines nowpassedindependent
  correctness before original8warmups/3processes/31samples began. No oldfailed
  scope reset or fabricated samples. Original publicpreparation gate remains
  incomplete; newsourcecandidate supportsall4pureRustadapterprograms.
  /tmp/tsrust-current-public-37-measurements.json; ingestONLYsuccessfulactualrun.
- Hostedfocusedrepair42470181942RUNNING atf46d7400, all4hosts; priorrun42460717406
  isexplicitinput withfixed4resulthashes. Failed/missing/new/unreachedcasesand
  reviewedforcedaffectedcontrols rerun; oldpasseskeeporiginal469identity.
  https://github.com/Forever-Frameworks-LLC/tsrust/actions/runs/42470181942.
  LatestprepushPRs/Dependabotempty; no duplicatefullCI/publicationdispatched.
- Final focused host classifierPASS0.569s; crypto source/negative guards5PASS,
  platform budget/ownership/repair-evidence guards22PASS0.110s. Whole-project
  format0changes/tokenfingerprints0 anddiff-checkPASS beforecheckpoint.
- Explicit1MiB Rustthread frontendprobe ACTUALPASS: load/check/emit/drop current
  process example,226778emittedbytes; compile1.072s/execute0.550s. This islocal
  frontend evidence, notproof Windowsnative/runtime stackfailurefixed.
  /tmp/tsrust-process-frontend-stack-probe.json. Windowscommands/stagesnowlogged.
- Cleanup recoveredadditional2627801088bytes from64 source/CRC-provenobsolete
  host/cacheoutputs; currentacceptedartifacts SHAverifiedunchanged. Removed1156
  closedlooseRustobjects702278040logical/722604032observedfreebytes. Combinedwith
  earlier5608370176bytes consolecleanup, sessionobserved recovery8958775296bytes.
  Currentarchives/binaries/metadata/shareddeps preserved. Rootcache remainsat
  target/; do notpurge dependencygraphs ordelete swapfiles.
  /tmp/tsrust-platform-repair-{obsolete-artifact,loose-object}-cleanup.json.
- Priorf46 remotevalidation decision (supersededbyactualhostresultsabove): local1544compiler/27LSP/
 52CLI/4editorphases/IO18bothprofiles/rawstdio/pure+genuineRust/logging+realtime
  relocatedbothprofilesPASS. NativeWindowsstack/signal phases andcoldhosttimeouts
  remainunresolved. SmallestreviewedCI inputplatform_prior_run=42460717406 retains
  oldpasseswithOLDsourceidentity, listsfreshbinaries, rerunsallfailed/missing/new/
  unreachedandforcedaffectedcases, thenpackagesmoke. It isFOCUSEDrepair, nevera
  fullcurrent-sourcepass. Last4hostrun42460717406terminalFAIL; no liveCIrun.
  CheckfreshPRs/Dependabotimmediatelybeforepush; publicationstillunapproved.
- ACTUALcurrent compiler1544PASS/0FAIL/8unchangedopt-ins ignored96.012s;
  LSP27PASS1.099s; CLI52PASS3.196s. PurestdownedadapterO0/O3 PASS2.690s
  WITHOUTRayon; genuineRayon/TaskContextbridgePASS2.137s. Newdependencyselection
  factworkswithoutremovingactualRayonavailability. /tmp/tsrust-platform-coherent-repair.json.
- ACTUALcorrectedIO18 EACHprofile+stable rawstdio witnessPASS116.644s.
  30min aggregate thenexhaustedduringloggingexport; no scopepass claimed and
  remainingfield retainsPRE-killchildren. Liveinspectionconfirmedallownednative
  processesdead. Passedcompiler/editor/CLI/IO/adapterswerenotrestarted.
- Remainingexports continuedasdistincttrees underABSOLUTEonehourfromtheoriginal
  repairbuildstart, notanewhour. ACTUALloggingrelocatedBOTHprofilesPASS216.862s;
  realtimeBOTHprofilesPASS248.588s. Consolehelperrootnamespacefixnowhasactualraw
  portable/nativeproof. /tmp/tsrust-platform-export-continuation.json. Original
 30minscope remainsincomplete; theseactualexportchecksareseparatecompletedproofs.
- B00new/changed namespace+receivercontractaudit complete, includingbinding
  signatures/access/results, catalog/reference docs, aliasclasses andcancel/drop.
  Twoactualdocgapsfixed; no missingnewAPIidentityfound. Nativefourhostandhistorical
  pre-facadeprofile chronology remainseparateunchecked requirements.
- Coherent currentrepair build ACTUALPASS416.124s/3386036472B, oneCargo,
  terminal build-finished success; normalCLI+LSP+selectedharnessescurrent. Original
 1770s repairdeadline retained. Watchdogsetup firstfailed on UnixThursday being
  mistakenforISO(T); correctedformat detection,21Python testsPASS. Laterfocused
  repair-selection tests bringcurrentPython count22PASS0.092s.
- Current3compiler selection/worker guardsPASS2.143s. LSP4phaseIO firstrepair
  fixedcompletiondetail but nothover's checked-call syntheticheader; focused
  LSP-only rebuild11.583s/529908528B, correctedexistinghoverpath now4testsPASS
 12.619s. Originalfailedphasewitnessretained, no assertionweakening.
- IOsharedsuiteguardfirstrejected blanklines; exacttoken mapping nowignores
  formattingtrivia while preserving tokens/stringcontents. Thenactual19case run
  failedNDJSON because stored-call witness writesrawstdout, a genuineprocess/
  stdioboundary. Revertedthat registration tooriginal18; keepchecked-in stable
  sourceandexecuteoriginalrawstdoutoracle separately, no tempsource/no library
  interception. This corrects earliermisclassification, not anI/O runtimefix.
- Additional hostedterminal evidence: ARM544passing/Intel401passing/Windows174
  retained. Macs' I/O failures are180/300s setup/run timeouts without captured
  diagnostics, not proven EOF/loan failures; Intel55min budget exhausted in HTTP,
  so missing later cases are incomplete rather than independent defects. Global
  CI60min cap remains; setup-inclusive55min accounting patch nowpasses20Python
  tests with creation-ordered ownership changes included.
- ConfirmedWindows watchdog bug: old vctip1508 born07:56 predates08:42root7032,
  yet reused intermediatePPID1476 made it appear owned. Creation-ordered ancestry
  and exact identity kills now exclude old processes; Windows taskkill/T removed
  because it follows PID-only trees independently.20Python testsPASS0.092s.
  Five raw process cases didpass; seven have genuine stack overflows whose CLI/
  emitted-program phase is unlocated. No stack-limit increase or semantic pass
  inferred. Source fixturetime nowuses100ns exactWindows precision, retains1ns
  onotherhosts; rooted-path probe usesWindowsdrivequalified paths so onlyintended
  relative call sees injectedcwd failure. Nativeplatformrepairs remainunvalidated.
- Hosted469 Linux finished35min;551 exactpassing cases retained,4 realfailed
  scopes. Consolehook namespace breaks raw logging/realtime exports; editor native
  alias facts overwrite recovered hover shape; pure nominalRustadapter failsRayon
  linkage. Windows finished with174passing cases and additionalnative issues;
  full case/attempt evidence downloaded to /tmp/tsrust-platform-42460717406-{linux,
  windows}/. Macs stillrunning; no duplicate dispatch. No fullplatform pass claimed.
- Pending coherent source repairs: sharedconsole helper root aliases preserveONE
  collector for raw+extracted programs; keepnative alias signatures in checked
  generic-fact application. Docscontract audit additionally found7genericWebSocket
  type summaries and missing direct-only operation-value limits for signals/
  threads/delay/yield. Narrowdocs fixes prepared/applied; no newAPIbehavior.
- Public37 measurement first attempt FAILED before timing258.136s/488713952B
  at crypto_sha256_stream_262144. Actual native checked error exposed all9
  streaming benchmark sources passing length instead of bytes.view end index.
  Rust/Node peers/oracles were correct. Fixed9 Tsrust sources and exact hashes;
  rehashed bad-length negative now rejects it. Crypto guard3PASS0.701s;
  whole-project format0changes. No timing rows were produced/ingested.
- SAME original1770s scope RESUMED with1336.0s remaining, exact6 unchanged
  one-shotSHA native/Rust/Node prepared artifacts hash-validated/reused by existing
  prepared-prefix path. No budget reset, sample reduction or new runner.
  /tmp/tsrust-public-iterator-crypto-adapter-resume.json records deadline;
  original failure /tmp/tsrust-public-iterator-crypto-adapter-measurements.json.
  Source/helper/ABI/executable validation remains strict before reuse.
- Allocation profiler crypto hooks were missing despite existing async driver
  support. Added existing crypto source/reference validation, frozen runtime
  rechecks and diagnostic evidence. Strict zero-retention assertion UNCHANGED;
  no allocation pass or password-retention exception inferred before execution.
- ACTUAL stored realtime factory acceptance PASS162.370s: shared55 EACHprofile,
  strict original identities/outputs, cache hit witness native0. Existing host and
  oracle source hashes match their accepted builds; current stable suites are
  loaded from disk. Only test fixtures/examples changed, no production compiler
  code repair or new standalone case programs. Entire affected gate reused one
  original1770s deadline, including build/repairs/cleanup; no budget reset.
  /tmp/tsrust-observer-alias-acceptance.json retains failed attempts and passes.
- Current source adds B03 renamed/stored observer coverage to existing example
  and controlled process fixtures, plus9 checker-only invalid-argument/readonly
  controls.17catalog entries now map to15stable std:testing modules, one native
  runner with exact module/case/output identity and private output. Four existing
  path testsPASS; new catalog guard initially failed only a blank-line expectation,
  repaired and catalog15PASS2.629s. Original4passed scopes preserved.
- Current CLI unit binary52PASS2.626s after ONE26.004s affected-host build.
  Process11PASS initially; unchanged relocated export failed missing source-pack
  env, then ONLYthat scope PASS29.153s with current prepared packs. Do not infer
  a process implementation defect from that setup failure.
- Compression renamed/stored buffer/stream factories now execute inside existing
  cases without extra programs. Typed invalid-level assertion required evaluating
  fallible byte creation before the owner-taking call (correct existing compiler
  rejection, no weakening). Original failed cheap attempts retained. Current
 16casesEACHprofile PASS25.126s with strict old source/output oracles.
- Complete backend doctest command ACTUALPASS156.752s/2258586768B; both crates
  register0doctests. This closes command execution only, not executable examples.
  /tmp/tsrust-final-backend-doctests.json.
- Remaining public timing audit: B15four+B16twentynine+B19four workloads have
  ZERO public SQLite histories. Private admission profiles are different work;
  first public run must measure all3engines, then preserve unchanged controls.
  Existing8warmups/3processes/31samples and quiet-machine rules remain unchanged.
- ACTUAL new obsolete console-cache cleanup recovered5608370176bytes (71 exact
  entries;5592688011logical bytes). Embedded old console collector identifies
  obsolete source, not age/size. Namespace locks held; no native process active.
  Current worker-context runtime artifacts, stable build workspaces and shared
  dependency caches retained. /tmp/tsrust-obsolete-console-runtime-cleanup.json.
- Hosted FOURHOST validation RUNNING at469e8bb3:
  https://github.com/Forever-Frameworks-LLC/tsrust/actions/runs/42460717406.
  Each host completed setup and entered bounded native acceptance. No full
  regression/publication run dispatched. Preserve local passed scopes; monitor
  terminal host results and actual packaged consumers before any acceptance claim.
- Remote validation decision: checkpoint includes only platform selector/CI/docs
  changes after5c3414e7. Local runner tests17PASS0.092s, release-policy9PASS,
  whole-project format0changes and diff-checkPASS. Hosted OS file/signal/process/
  TLS/export behavior and55-minute expanded fit cannot be proved locally. Smallest
  available workflow is CI platform_acceptance=true, four native hosts, no full
  regression/benchmark/publication dispatch. Previous equivalent full platform
  run42259225163 at7d67daf8 was cancelled;42279800898 is UDP-only supporting
  timing, not platform acceptance. Current PRs/Dependabot empty and no live run.
- Pending platform selection now includes45 compiler/16LSP targets (61 intended,
  64 validated Cargo artifacts); existing shared55, crypto24 and regex catalog
  programs are reused. No newly ignored tests or runtime benchmark replay. Fit
  on four hosts remains unmeasured under the unchanged55-minute aggregate.
- Full CI regression scope corrected to complete compiler/LSP lib/bin/integration
  and doctests, per development-workflow.md. Removed duplicate frozen UI runs;
  test threads3, jobs1/incremental0 unchanged. Job ceiling remains60minutes.
- B00 historical prototype audit found real pre-facade evidence for B03,
  B05–B17 and B19, with retained SSE semantics qualification. B01/B02 retained
  receipts are incomplete; B04/B18 retained profiling is post-facade. Global
  historical-order checkbox remains open; later measurements do not establish
  earlier chronology. No speculative prototype rebuild was launched.
- Defaults audit found no additional implicit v0.2 limits. Process spawn inherits
  stdio; run pipes output and closes absent input, matching spawn/output modes.
  Drop cleanup is an explicit documented ownership difference. Existing bounded
  date-format/Decimal/threadpool contracts remain intentionally preserved.
- NEXT: reconcile remaining B00contracts/
  defaults/feature clauses; final backend all-target/bin/doctest acceptance and
  FOURHOST native/package/install proof still pending. Check current PRs/
  Dependabot immediatelybeforepush; lastread empty/no live run. Do not dispatch
  before a concrete current scope that fits55min. Full means full_regressions=true.

## Earlier retained acceptance (superseded failures are not a work queue)

- ACTUAL PASS currentCLI publicfrontend declared/inferredfactory4controls plus
  genuineMissing_response error; originalservicefrontendPASS; inventoryPASS1680
  rows/671bindings/174types/43sourceexports/78retainedexamplewitnesses.
  /tmp/tsrust-service-unwind-repair-results.json.
- ACTUAL integratedservicePASS78.033s/1042023408B: ONEappbuild thenSAMEexecutable
  TLS/auth/sessionrevocation/persistentSQLite restart/forms/multipart/CORS/bodylimit/
  staticallowlist/redaction/SIGTERM/SIGINT. /tmp/tsrust-service-unwind-repair-service.out.
  NativeDarwin only; offlinePackagefalse, notLinux/Windows/exportproof.
- Main focusedharness launchfailed0.558s BEFORECargo becauseold /tmp/tsrust-watch.py
  vanished. Restoredthinwrapperusingrepo platform.bounded, no newwatchframework.
  ONLYpendingRustharness resumedinsideSAMEoriginal30minscope; actualPASS240.363s/
  3371849128B; exactprojectregressionPASS0.613s including4variants+realerror.
  /tmp/tsrust-service-unwind-host-tests-results.json and -d2e204380b48.json.
  Session66923/83979closed. No build live. Preserveallpassednativeapp checks.
- Current repairedsource retainsbeforeBOTHdiscovery→fullentries, restoresONLY
  failedtask'sdraineduncheckedemptyfunctions BEFOREpendingerrorretry. Diagnostics
  removed. Earlier03ec1538compiler/27LSP/Docnative/fullHTTP3passesretained; final
  wholebackend/platform acceptance stillpending. Do notclaimallv0.2done.
- B00newpackageaudit vs8efaaee:30native/31backenddirectadditions,17alreadylocked;
  44newregistryidentities checksumverified;42activehaveexactretainedadmission;
  2dormant. Currentbackendlock0vulns/0warnings; optionalUI warnings remainexplicit.
- Attribution/export05ea0131 ACTUAL PASS: existing CLI build26.545s/654639920B,
  terminal Cargo build-finished success; focused CLI notice test1PASS/0FAIL1.092s.
  Actual structured-logging portable export13.131s/57999960B, no native build:
  exact upstreamMIT1065B/SHAed60d479...b44 at root attributions/dependencies/,
  all23 vendored Valuable files match unchanged Cargo checksums; disposable
  export deleted. /tmp/tsrust-valuable-cli-build.json and
  /tmp/tsrust-valuable-export-result.json. JSfocused2tests remainPASS0.355s.
  No active Cargo/rustc processes at initial process inspection.
- Accepted alias/feature assertions from a3d9a82e plus narrow physical-default
  annotation repair: host build4.715s/413991704B; feature selection14rowsPASS1.564s;
  iterator private/extracted-method negativesPASS0.555s. Original shared native
  build failed10.462s with E0282, preserved in /tmp/tsrust-alias-feature-acceptance.json.
  Repaired deferred borrowed union owner now names INITIALIZER physical type;
  ordinary non-union defaults/ownership/evaluation unchanged. Existing unit guard
  retains physical enum check and verifies complete owner annotation.
- ACTUAL repair PASS /tmp/tsrust-alias-feature-repair.json: build385.896s/
  3387248888B, terminal Cargo success; focused physical-default guard1PASS1.067s.
  TWO existing std:testing files share ONE native runner:18PASS/0FAIL/0SKIP,
  native build/link2522.47175ms, check1275.527208ms, total setup4079.676458ms;
  test execution3.244125ms. Outer runner5.225s/392691712B, remaining[]/stopnull.
  Same original900s aggregate scope, not a new budget or full regression claim.
  Removed61 newly-created loose codegen objects23723224 logical bytes after
  all owned processes settled; kept libraries, metadata, binaries and shared caches.
  /tmp/tsrust-alias-feature-object-cleanup.json. No build active.
- B00 pin/wiring checklist reconciled with current54 common direct manifest
  declarations/lock identities, reserved-package regression and actual relocated
  exports. Broader package admission, optional-selection/minimal-program footprint
  and four-host acceptance remain open. Reuse already-passed regex/locale/process/
  network/compression/hostname selection controls; do not replay their nativeapps.
- Platform selector repair: all7 existing B01 filesystem targets now selected,
  46 intended integration targets/49 validated Cargo artifacts. Existing buffered
  backend harness includes actual fs.rs O0/O3 tests, including Windows exclusive
  open conflicts after explicit close/scope drop. Selector/watchdog unit tests
  ACTUAL PASS15/0FAIL0.093s. No native family rerun or new remote execution yet;
  55-minute aggregate fit and four-host execution remain pending. Guide corrected
  to describe one Cargo build and per-target pending-case batches.
- Owner permits profiling/additional test methods. Existing public test --profile
  separates check/emit/backend/cache/native build and per-case execution timings;
  use stable std:testing suites and cache, not independent programs per case.
  Single observations remain distinct from sampled benchmark claims. No new
  profiling framework or repeated accepted app builds were added.
- B20localintegratedapp rownowcompletedwithqualifiedMac evidence; nativefourhost/
  finalregressions/remainingB00review remainunchecked. Namedfs/HTTPdefaultmigrations
  documentedascompletedwithoutnarrowingparentcross-exportdefaultrequirement.
- Netcleanup oldhost1430007808B +obsoletebasepack766394368B. Currentbase/Shlexpacks
  protected; rawnewobjectcleanupseparate. Shareddepsretained. No push/publication.
- TERMINAL PASS current03ec4f13 repair: selectedbuild394.945s/3276963KiB,
  two focused compiler guards PASS, real documentation native case PASS119.67s.
  /tmp/tsrust-fixed-callback-repair-76986fc9793f.json.
  Complete compiler library1538PASS/8unchanged opt-ins96.549s; LSP27PASS0.559s.
  /tmp/tsrust-fixed-callback-family-libraries.json terminal-success. Outer public
  phase518.307s, library98.167s; stopnull/remaining[] both. Session65004 closed.
  No compiler rebuild currently running. Preserve all these accepted scopes.
- Integratedservice ACTUALfrontendFAIL0.559s BEFORE nativeapp build:
  /tmp/tsrust-current-integrated-service.json +.err. False `routes must return App`
  at examples/service-observability/main.tsr45; function explicitly returnsapp.
  Adding UNUSED exercise async try/finally calling routes reproducesfalseerror;
  same routes/body withoutunusedexercise checks. B06/B14 investigate consumed-body
  retry/checked-function invalidation, not an outdatedApp API. No speculative
  productionfix yet; do not inline/removeobservability or weaken service checks.
  No proven SERVICE_BINARY exists; afterfrontendrepair the existing harness needs
  ONEappbuild thenrestarts SAMEexecutable foractualservicecontracts.
- Existingpublic_api_inventory example build reused compiler/dependencies and
  finished4.158s; no compilerlib rebuild. Tool execution exposed private internal
  NativeTypeId lookup assumptions: PoolSubmission explicitly classified pending
  commit; IteratorCursor remains tool-only reconciliation gap. No completed current
  inventory/per-export acceptance claim. Root is only canonical writer.
- Second repair terminal: selected build PASS396.529s/3269491KiB; overall
 573.204s/3379566016B, stopnull, remaining[]. Session54291 closed; no build live.
 /tmp/tsrust-physical-carrier-repair-resume-6a381ec5ec6a.json.
- Positional cache key, coalescing context and owned-field replacement witnesses
 passed. Generic-layout fixture still failed because its base callback was owned
 rather than scoped; readonly-base fixture correction is pending native validation.
- Actual sharedHTTP std:testing suite PASS all3 cases: existing request streaming,
 real4097-byte route/syntax rejects, exact maxAge34560001/i64MAX headers and
 negative values/cleanup. One public runner/build; host58.71s including setup,
 not a comparable benchmark. Test now prints existing profile/build/test JSON
 events for future visibility. Do not rerun passed native cases for logging only.
- Documentation remains FAIL107E0308, all spread-boundary field mismatches; five
 positional call errors resolved. f858 physical-spread lookup did not fix actual
 failures and was removed, preserving canonical constructor repair. Tiny valid
 public TSRX repro proves missing conversion at FIXED callback argument boundary.
 Reviewed replacement uses existing checked_carrier only after fixed invocation
 resolution; named/forwarded functions and methods retain physical specialization.
 Exact new uppercase TSRX fixture frontend-check PASS before build.
- All1530 compiler/27LSP earlier library, rust_bridge8/nullish7/LSPbridge8 and5/6
 earlier docs passes retained. Actual generic fixture + new fixed-boundary guard
 and failed docs case next; complete libraries at this ownership-family milestone
 reuse the same build. Eight existing opt-ins remain unchanged, never forced.
- Three current production lock audits ACTUAL0vulnerabilities. Backend0warnings;
 workspace/native each four unmaintained +two yanked retained optionalUI packages.
 DB309392b18f6b17022f82c701f300433126901068, updated2026-11-06. Exact lock hashes,
 activation chains and qualifications are in backend-distribution-inventory.md.
 /tmp/tsrust-current-production-audit-summary.json. No warnings suppressed.
- Net obsolete host cleanup reclaimed1430007808B. Current selected artifacts and
 application dependencies preserved. Per-build new-object cleanup30068736B and
 130850816B are separate, not additional net historical recovery.
- Original30minute resume guard refused before launching another process when
 insufficient time remained. Original failed scope unchanged. Subsequent repair
 was independently capped30minutes AND first repair start+onehour. Next driver
 /tmp/tsrust-fixed-callback-repair-run.py keeps that same absolute one-hour ceiling.
 Do not extend it or claim budget exhaustion passed. After real doc acceptance,
 resume existing integrated service and remaining B00/B20/native platforms.
Only root writes canonical; app worktree owner WIP stays protected.
- ACTUAL PASS /tmp/tsrust-http-factory-portable-repair-422a7d392a0e.json:
  HTTP forwarder4, shared55 EACH debug/release plus filtered cachehit/nativeBuildMs0
  EACH, focused HTTP LSP3, middleware alias/native+relocated exports, real Shlex
  generated adapter+relocated empty-Cargo-home locked/offline exports at both profiles.
  Shared55 execution162.818459/67.557667ms; setup68631.7925/30725.667ms. Correctness
  observations, NOT comparable timings. No new skips/weakening/unsafe.
- Initial compiler build terminalPASS636.673s/4163284KiB; orchestration omitted
  LSP package and failed artifact assertion before tests. Original artifacts SHA
  retained. LSP-only alternate selection interrupted50.988s before compiler rebuild.
  Correct shared package selection reused compiler and added LSP14.488s/711952KiB.
  Resume terminalPASS400.573s/1668041368B,stopnull,remaining[]. All within SAME
  original1800s scope /tmp/tsrust-http-factory-portable-repair-scope.json,
  startedUnix1794096899.864186. Sessions53382/95717/24361 all terminal; no run live.
- Source fixes accepted: existing checked factory forwarding now includes stored
  secureHeaders/compress, plainMiddleware direct return and reentry-safe reserved
  Symbol dedup. Shlex bindingsresolve explicitly prepares --portable; export
  remains Cargo-free. All B10 API boxes now reflect actual passed evidence.
- B19 generator/customization/rustdoc57/fallback/editor3/3 retained19b; nominal
  nested String pointer/capacity moves O0/O3 retained19c. RealShlex paired with
  compiler-owned regex backend5/export passes at965 closes package-witness scope.
  Historical19b/19c nested bridge failure is now repaired: all8 compiler rust_bridge
  cases passed in the ownership scope. Preserve that pass; do not replay the
  already-passed adapter/native/export controls.
- Retain byte68 EACHdebug/release PASS /tmp/tsrust-nullable-storage-proof-front-30bc05cbad9c.json;
  actual10002node ABI/pointers/deepdrop/O0O3 PASS /tmp/tsrust-abi-witness-repair-c5252aa3173b.json;
  full compiler library1528+repairedHTTPpair =all1529runnable,8unchanged opt-ins;
  LSP27PASS /tmp/tsrust-closed-runtime-library-milestone.json plus
  /tmp/tsrust-http-body-limit-proof-28fed979d29f.json. Do not replay for harness edits.
- Cleanup quiet/locked /tmp/tsrust-closed-http-caches-cleanup.json recovered
  1603506176B actual:3source-proven obsolete native cache entries+closed old
  target/http-realtime-public-package. Current suites/shareddeps/controls retained.
  Per-build rcgu cleanup recovered1140387840B+96890880B+334708736B newly created
  objects; do not sum as net historical recovery.
- Reviewed pending harness changes reuse explicit validated TSRUST rather than
  rebuilding release compiler in run.mjs/profile-native.mjs. Logging allocation
  driver now supports existing checked Result ABI/required pin_project_lite and
  source guards. Pure focused checks6PASS; whole-project format0changes before
  latest evidence docs; repeat required formatter before source checkpoint.
- Existing coverage guide now lists245 added catalog namespace identities since
  preplan8efaaee21bcc;520 current backend namespace rows documented. This does NOT
  close receiver/source-only/per-overload/platform/default/advisory acceptance.
- ACTUAL B11 native measurement/ingest PASS: /tmp/tsrust-b11-fresh-public-native-sourcepack-resume.json,
  session24482 terminal0. First source-pack environment mistake failed1.288s before
  build; correctedresume used target/portable-source-packs/SAME original30minbudget.
  Emit14.153s,releasebuild130.698s/peak554713856B,publicprotocol198.282s/508788736B,
  ingest0.539s. Resume344.550s; no quiet bypass. Strict guard waited for Spotlight
  load13->below4, thenall6nativecases original3x31x128 +separate resourcepasses.
  SQLite18workload/engine rows/history18; each93samples/11904latencies.
  Native at2026-11-08T00:43:50.142Z, unchangedRust/Node at2026-11-07T10:50:14.417Z.
  WSbatch/nativeRust ratio0.91–1.04,Node0.83–0.92; SSE slower1.12–1.20Rust/
  1.06–1.10Node. DirectSSEbackendfloor lacks requiredpublicchannelproducerbridge;
  no hiddencontractweakening/optimizationclaim. Actualstore:ingest Added18entries.
  Buildpackage target/http-realtime-public-package nowclosed/disposable; controls
  and shareddeps retained. No indexing settings/path changes were made.
- Harness6purechecksPASS,whole-project format0changes/diffclean beforeddc61ed6.
- URL ACTUAL measurement/ingest PASS, current source75027f23:
  /tmp/tsrust-url-family-resume.json session33695 terminal0, protocol157.180s/
  peak571290008B,ingest0.558s,resume158.085s; SAME original30minscope as initial
  failed373.490s preparation. Initial10th TSquerysnapshot lackedDOM.Iterable;
  affected tsc --noEmit PASS0.201s, fixcheckpoint75027f23. Sealed first9artifact
  prefix receipt /tmp/tsrust-url-prepared-prefix.json: currentCLI/input/generated/
  binaryhashes +priorfailedscope; exactfrontendregenerationmatched, zerofirst9
  recompiles; existing5-shape correctness rerun, remaining2preparednormally.
  All11native/Rust/Node cases165correctness +3x31after8warmups; SQLite33rows/
  histories33,each93samples,at2026-11-08T01:00:48.357Z. Native authority/path
  scaling8192->65536 doubles1.85–2.11long/1.93–1.98short;65536medians1.645375/
  1.156375ms vsRust1.638458/1.188250,Node0.579333/0.497042. Boundedlinear
  corpus,notuniversalcomplexity/WHATWGclaim. B00URLinvestigation checkboxclosed.
  Generalnative/Rust components/query/setters1.06/0.92/0.96;Node0.66/0.71/0.12.
- Realtime closed-package cleanup /tmp/tsrust-b11-closed-package-cleanup.json
  recovered773849088B actual newly materialized vendor output; controls retained.
- B17 disabled-event allocation proofACTUAL: original profilebuilt30.313s then
  FAILED genericzero-retention validation. SAMEcompiledbinary,3freshprocesses
  eachn0/1/32/1024 all6alloc/0realloc/33656requested+peak/776retained+afterDrop.
  /tmp/tsrust-disabled-logging-first-call-counts.json +source/binaryseals in
  /tmp/tsrust-logging-family-resume.json. Independentchecksums +factoryneverinvoke;
  eventincrement0. B17checkboxclosednarrowly; NOT zero-whole-workloadretention/
  leakproof. Genericprofileassertunchanged/explicitlyFAILED, no probe rebuild.
- Matched logging3 firstattempt stopped32.924s at handwrittenRust ABI adaptation,
  beforetiming. Preserveauthored fnfunction0(n:i64,seed:i64) source/hash unchanged;
  reuseexistingreferenceexception pattern with explicitguard, nativeABI staysstrict.
  Focusedpure4PASS,format0changes. Resume SAME original30minscope aftercheckpoint;
  currentCLI/compilerRust unchanged. No logging timings/ingest claimed.
- Logging matchedsession70736 stopped31.879s on secondworkload's actual movedpath
  error; firstdisabledworkload prepared/allcorrectnesspassed. Repair ONLY benchmark
  sources: retain declaredLoggerOptions, narrowoptionalpath/readsamefieldafterclose;
  child typedJSONspan usesexistingnamedSpanFields alias,notunsupportedinlineobject
  type. BothsourcechecksPASS/focusedpure4PASS. Sourceoracles/fingerprintsupdated;
  Rust/Nodecontrols unchanged. Existingsealed-prefixguard nowalsoallowslogging
  family withfullsource/native/TS/hand hashes andfrontendregeneration. Sealed
  completeddisabledcase /tmp/tsrust-logging-prepared-prefix.json; no disabled
  program/probe/compiler rebuildneeded. Remaining2prepare normally.
- Logging sourcesession82879 stopped34.027s beforetiming: secondcontrolimports
  node:fs/os/path butfamilyomittedfromexistingNode typeRoots/types selector.
  One-linefamilyselectionrepair; BOTH filecontrol TypeScript --noEmit preflights
  PASS,whole-projectformat0changes/diffclean. Reuse SAME sealeddisabledprefix
  and SAME original30minscope; no compiler/probe rebuild or controlsource edits.
- B17 ACTUAL MATCHEDTIMING/INGEST PASS: /tmp/tsrust-logging-family-node-resume.json,
  session78133 terminal0,protocol164.558s/525921992B,ingest0.561s,resume165.554s.
  SAMEoriginal30minscope includingearlierprofile/wiring repairs; no quietbypass.
  All3workloads45correctness +3x31after8warmups; SQLite9rows/history9 each93samples,
  at2026-11-08T01:24:07.067Z. Native disabled/file/child0.009375/3.058417/
  5.668209ms; Rust0.005666/3.198125/6.186750;Node0.027625/3.622375/4.101875.
  Wholepipelines includecreate/close/checksum; child1.38xNodeslower retained.
  Zero-eventallocationdelta and776Bfixedfirstcallretention remainseparate,strict
  genericprofileFAILEDnotretroactivelypassed. Rust/Nodecontrols unchanged.
- ACTUAL affectedownership scope: /tmp/tsrust-final-ownership-acceptance-788f019d501b.json,
  session37998 terminal1/stopnull/remaining[]. SharedbuildPASS648.082s/4175300KiB;
  phase796.959s/4297971928B; cleanup1330298880B newlycreatedobjectsrecovered.
  All8compilerRustbridge (inclpreviousfailednestedtags/drops),7nullish,8LSPbridge,
  5of6documentationcontentPASS. B19firstcheckboxclosed; preserveallpassedcases.
  SoleFAIL authored_content_api_inventory_and_exact_examples_build_into_checked_native_data:
  RustmissingRecord261/290/... constructoraliases,247reportederrors/cascades.
- SAMEoriginalscope librarymilestone ACTUAL PASS, session36182 terminal0:
  /tmp/tsrust-stdlib-family-final-libraries.json,compiler1530PASS/8UNCHANGEDoptins
  106.487s/1262978632B; LSP27PASS0.566s/5915080B. No newbuild forlibraryruns.
  Scopeoriginalstarted1794101250.8819342,budget1800; completedwithoutoverrun.
- Reviewed pending rootfix /tmp/tsrust-object-spread-native-declaration.patch
  SHA1a50d8bff6371a29ee5f18afc42d4a6e25ce580727276111a088567a095a3756:
  object_spread_value_with constructorusesexistingnative_record_declaration(result),
  matchingdeclarationloop/plainliterals. Resultfieldlayout/coercions/orderunchanged.
  Existingnative_view_once_family_emits_real_alternative_lifetimes assert_family
  nowchecksnormalizedconstructornames/bothmutablemodes/singleinput/keyorder.
  SourcepeerreviewPASS; nativeDocfreshdiagnosticsstillrequired,notall247fixedclaimed.
- NEXT targeted source-repair scope,30min/5GB AND originalmilestoneabsolute1hceiling:
  buildnormalcompiler/LSP+updatedlibonce; runONLYupdatedscoped-family unit +failed
  documentationcase, preserve1530/27/8/7/8passes. OriginalscopeAretainsdocFAIL;
  don'treset/labelitpassed orrerunwhole4targets. Oldprepared command belowisNOTnext:
  prepared
  /tmp/tsrust-final-ownership-acceptance-run.py +gates/scopes/cleanup sameprefix.
  One shared compiler+LSP selectedbuild,explicitbothpackages+samefeatures,rootlibs
  buildcacheonlyNOTexecuted; whole rust_bridge/documentation_content/nullish_coalescing
  compiler targets +LSPrust_bridge. Includes lastfailednestedOption/Result/Vec tags/
  drops. Fixed30min/5GB/jobs1/incremental0/threads3; oldactualexecution~161s.
  Preserve1529library/27LSP/55/68/24/42passes; thisisaffectedblock,NOTfullregression.
  Existingfullintegrationoldtotals doNOTprove55minfit; oldbatcheddriverhascohort
  editorfilters andmustNOTbeusedunchangedforfullclaims.
- Remaining: B17 logging_disabled public
  allocation profile (existing driver, separate setup/event costs). Remaining
  B00 ledger/package/platform, B19nested integration, B20 apps/final regression/
  platform/measurements/release acceptance stay open. Integrated persistent API
  control tools/check-service.mjs supports SERVICE_BINARY reuse; otherwise build
  ONE app. Fullgoalactive; no push/merge/v0.2publication approved or claimed.

Latest accepted section — public55case native runtime; full v0.2.0 remains active:

- Current canonical checkpoint7c171bb8; uncommitted targeted compiler repairs
  are under bounded affected validation detailed below. Accepted runtime fixes are retained
  atc7e8eced. Only root writes canonical; app worktree edits stay protected.
- ACTUAL PASS: /tmp/tsrust-upgrade-handoff-4366908885f1.json. Selected fresh normal
  compiler CLI build terminal success,226.058s/peak2791138KiB. Shared public host
  public_registered_suites_share_one_native_build PASS; all55cases/5files,
  zero failures/skips/hooks. Exact logging10rows/order/concurrent request fields,
  realtime18cases,iterator32cases/catalog outputs and55names all verified by host.
  Execution155.242166ms; setup17567.628416ms (check8311.2005,emit348.149792,
  backend45.198542,cache0.302,nativeBuild8832.318167ms/cachemiss). These are test
  observations,not comparable benchmark measurements or SQLite runs.
- Final bounded run246.728s/peak2889995040B,stopnull,remaining[]. It and automatic
  cleanup fit /tmp/tsrust-upgrade-handoff-scope.json's original15minute ceiling.
  LSP was not rebuilt for backend-only fixes; existing editor passes retained.
- Accepted fixes: retain computed readonly inputs of checked Task factories and
  optional borrowed-union payload owners; preserve ordinary mutable union ABI.
  Enable Hyper serve_connection_with_upgrades for actual socket handoff. Stop
  close draining on acknowledged peer Close,keeping prior transport errors real.
  No TLS verification weakening,unsafe,hidden clones or raised API limits.
- Logging fixture corrected to Rust cancellation semantics: cancelled future drops
  owners; source finally only runs when reached. Explicit cleanup log follows
  awaited cancellation settlement in caller; original requested/cancelled asserts
  and exact ten-row oracle preserved. Public futures reference documents this.
- Earlier native executions50/55 then54/55 are preserved in prior gate JSONs;
  actual failures drove these repairs. Two filtered diagnostic reruns used the
  same stable55registered runner (18.255s/19.782s),not per-case native sources.
  Temporary stage logs removed. Existing tagged-literal,WebSocket literal,
  17core/12iterator/12editor receiver passes remain retained and unrerun.
- All source is whole-project formatted,diff--check clean. Cleanup THIS
  continuation recovered5172674560B actual free space: obsolete linked objects,
 105compiler/LSP executable consumers older than changed emitter source,and stale
  compiler metadata. Current consumers/shared dependencies/independent Rust-Node
  controls retained. Quiet processes/shared locks checked before deletion. Reports
  /tmp/tsrust-*-objects-cleanup.json and
  /tmp/tsrust-task-input-obsolete-executables-cleanup.json. Earlier1.02GB/5.71GB/
 491MB recoveries are separate; do not double count.
- Portable source setup PASSED offline15.1s. Corrected benchmark SSE callback to
  async before starting its producer Job; original runtime protocols unchanged.
  Public SSE/WebSocket server exports and release-builds offline PASS134.948s,
  peak553583384B. Existing Rust/Node controls remain unchanged/cached; exact Node
  ws8.18.3 archive materialized offline and package proof passes.
- Comparable measurement deliberately INTERRUPTED with zero network/resource
  groups: quiet guard saw sustained load12–14>=4 (Spotlight indexing and Codex
  renderer). No timings/SQLite ingestion claimed; no --allow-busy bypass. Driver
  /tmp/tsrust-b11-public-native-resume-measure.py ended -2,remaining[],404.275s;
  wrapper /tmp/tsrust-b11-accepted-public-resume-results.json. Optimized public
  package retained at target/http-realtime-public-package. Its normal program
  alias will need fresh validation/proof after exports overwrite that alias.
- Relocated offline exports PASS: /tmp/tsrust-upgrade-handoff-d970ae7bc9e7.json;
  realtime18cases at debug+release,empty Cargo home,local TLS fixtures,forbid unsafe;
  logging5cases/10exact rows at both profiles. Hosts247.074s/221.528s; total
 474.781s,peak884623928B,stopnull,remaining[],inside30min scope. Cleanup339607552B.
- Temporary TLS runtime paths validated: all55pass156.562333ms, but changing
  TSRUST_REALTIME_TLS_DIR every host invocation caused cache misses. Actual field
  comparison proved generated Rust(1988485B SHA0f836a623062...) and graph(103933B
  SHAdafb1961435e...) identical; only backend_sources application environment
  identity differed. Cache identity deliberately includes all build-script env.
- FIXED IN HARNESS,not weakened cache key: stable private temporary fixture path
  hashes workspace+six fixture contents; standard file lock protects staging.
  Runtime never reads Desktop certs. Reusing SAME compiler command with a filter
  proves cache hit/nativeBuildMs0,keeping initial55JSON rows intact for benchmarks.
  /tmp/tsrust-stable-family-6135303b69f6.json structured_logging PASS28.056s;
  all55nativecases147.86ms,then one filtered cached case. No arbitrary env excluded.
- All three prepared migrations below are now APPLIED+whole-project formatted:
 42collection cases(default),24crypto(debug/release),67bytes(debug/release).
  Original rejections/ABI controls/profile requirements retained. Formatter112files
  then0/idempotent; blank-only Rust trailing whitespace corrected. Acceptance
  remains pending; do not claim177former native launches are already replaced
  with accepted passes until actual new suites finish.
- Latest 30min family scope /tmp/tsrust-stable-family-run.py stopped MEMORY after
 45.355s at5233890872B. Structured logging/cache passed; collections failed native
  generic spread/join emission; crypto failed a source guard (export formatting
  inserts a blank separator); bytes check grew beyond5GB before native build.
  No higher limits/restarts. Cleanup quiet process check passed,recovered26673152B.
- Focused diagnostic ONLY: /tmp/tsrust-bytes-memory-profile.py paused checker at
 800408728B,sampled1s then killed it;2.483s total,no native compile/acceptance.
  Stack /tmp/tsrust-bytes-memory-profile.sample.txt43–67 proves CHECK phase:
  discover_record_callbacks -> prepare_record_child -> absent_nullable_record_annotation
  -> empty_environment_work -> absent_callback. empty_environment_work follows
  recursive Holder.next?:Holder without active/memo state,unbounded postorder
  frames; absent_callback identities themselves are cached. B14 reviewer owns
  finite recursive empty-environment repair; keep all67cases intact.
- Current source checkpoint c057307c preserves all147migration/export/cache files;
  it is NOT family acceptance. Uncommitted compiler repairs now APPLIED: collection
  spread/join373602cb...,crypto sourceguard336270ff... (public lexer-backed
  documentation highlight API used because syntax module is private),recursive
  empty environments3ec1e44c.... Root strengthened new mutual-view fixture with
  exact3/7/3 bytes in addition to lengths; bytes suite now68,all original67 intact.
- Recursive repair uses operation-local SCC traversal with record/map modes,
  finite reserve/remap for all component members,one full signature cache key,
  unchanged-layout reuse and already-absent callback reuse. No new global registry,
  depth caps or weakened ownership. Four focused private controls cover self,
  mutual both-callback,map/optional-map/maps-only,cache aliasing and idempotence;
  prior16k unchanged-chain/deep callback controls retained.
- Recursive-family gate TERMINAL: /tmp/tsrust-recursive-family-400197d26d77.json.
  Fresh lib/CLI/hosts build531.961s,peak4109828KiB; selected10empty/SCC/recursive
  unit controls PASS0.629s;55/cache PASS27.443s;collection42 PASS3.203s. Crypto
  source-count guard expected7 but actual8(seven field examples+shared header);
  fixed with independent8hash/9reference proof755c08c4.... Bytes now checks1.31s/
 113936KiB,then fails existing case14 native-view-call at child.read,not memory.
  End-to-end gates569.347s/peak4243298616B,remaining[];cleanup933388288B.
- Real failure-only metadata trace added under EXISTING TSRUST_TRACE_CARRIERS.
  CLI-only diagnostic build267.624s,then same68suite check2.121s/113311744B,
  original30min scope unchanged and actual work stopped before it expired.
  /tmp/tsrust-view-guard-diagnostic-results.json. All discarded narrow pairs were
  legitimate Result Err(95<-94 and101<-100);NO flow drops. Final holder106/child106
  bindings have NO loans. Therefore speculative guard-remapping patch5f827da...
  is explicitly NOT APPLIED; do not revive it as the current repair.
- Proven producer bug: named function native summaries defer discovering_results,
  but closure/family registration did not. Early callback summaries enable loan
  checking before named read/build factories publish owner data. Applied ONLY
  two matching discovery guards +one focused definitive-missing-owner control:
  /tmp/tsrust-native-discovery-summary.patch SHA04afd8c6b90a8d63903adfff9fe2400004288483f91e9907d301f6b44c9be621.
  No lifetime/narrow check relaxed. Crypto actualcount8guard fixed too.
- Native-summary scope TERMINAL: preflight60.369s; selected lib/CLI/hosts build
  PASS532.596s/peak4141412KiB; three native-view-callee controls PASS and55/cache
  PASS. Crypto now reaches real await-task error at imported catalog async main;
  bytes still rejects existing case14 child.read lifetime. Full gate567.01s,
  peak4288731352B,remaining[]; cleanup932052992B. Evidence:
  /tmp/tsrust-native-summary-42b94eaef168.json. Do not repeat passed scopes.
- Post-discovery frontend-only trace1.475s confirms final holder/child loansNone,
  no physical guard loss. /tmp/tsrust-native-summary-post-discovery.err.
- Imported async main root proven: parser synthesizes a synchronous executable
  main wrapper even for imported modules. Applied link-boundary normalization
  restoring imported Task factory, preserving actual entry wrappers. Named/default
  import and wrong-result-type checks added; not yet built/accepted. Patch
  /tmp/tsrust-imported-async-main-factory.patch SHA26a4df4883ffd07efbaa64279304f614aa4504a7d1f04242580cb55b876e6e87.
- Applied effects-only phase repair82a969c6: return-summary recording already
  skips access inference; named/closure/family registration now matches it. This
  prevents real callback summaries consuming intentionally absent inferred loans.
  Definitive missing-owner rejection retained; unit extension compile-preflighted
  but not independently replayed. Failure trace now identifies effects_only.
- Native-effects scope TERMINAL /tmp/tsrust-native-effects-75f59227bec7.json:
  preflight63.612s PASS; one CLI/three-host build227.184s PASS,peak2837810KiB.
  Crypto24cases PASS BOTH debug/release,all source guards retained; host36.294s.
  Actual debug setup10070.018ms/native9630.549ms/runtime2474.350ms; release
  setup22471.437ms/native22088.604ms/runtime138.365ms. Test observations only,
  not controlled benchmark measurements. Named/default async-import+wrong-type
  regression PASS0.573s. No library or previously passed suites replayed.
- Bytes advances past original case14 lifetime failure. New mutual-recursion
  witness fails frontend at root.next = second(read(right)): Expected Second |
  undefined,found Second. Preserve all68 and3/7/3 oracle; next repair is concrete
  physical-layout assignment compatibility,not the already-fixed missing loans.
  Total gates270.352s,peak2967212712B,stopnull,remaining[]. Quiet automatic
  cleanup123rcgu objects recovered433356800B; heavy slot free. Agent B14 reviewing
  new actual failure. Collections42,55/cache,10recursive/3callee units and portable
  exports retained; no new SQLite timings or full-goal completion claimed.
- Applied new checked-native-record eligibility patch b5fa0513 after source
  checkpoint2459529f. Exact nominal/generic arguments and nullish domains retained;
  recognizes physical scoped-native callable families with actual view lifetimes,
  rather than broadening aggregate ownership facts. Optional injection,nullable
  record compatibility and conditional join share this narrow condition. New parsed
  factory unit covers physical retention,wrong nominal and mismatched nullish domain.
- Native-carrier affected driver TERMINAL: /tmp/tsrust-native-carrier-abf5dc0317df.json.
  Preflight compiler+LSP tests PASS73.383s; selected CLI/LSP/lib/hosts build
  PASS547.739s/peak4109172KiB. Four focused compiler units PASS;27LSP library
  tests PASS; byte callable/editor agreement PASS; std_bytes25/28 PASS.
- Remaining bytes: all68 now pass frontend,then native_capture_storage.rs206
  panics at resolved native environment during EMIT. B06 investigates emitter.
  Two existing negative hosts(nullable recursive factory joins/mutable recursive
  stores) still reject,but incorrectly report missing Holder return +missing
  capture lifetime instead of required native-view-escape; B14 investigates phase
  consistency. Neither error assertion weakened; original cases/oracles retained.
- Affected gates568.461s,peak4225407168B,stopnull,remaining[]. Automatic quiet
  cleanup135rcgu objects recovered1007636480B. No further build live.
- Full compiler library milestone TERMINAL,NO Cargo build: reused exact source
  abf5dc0317df/binaryed18f6bb... under original30minute carrier scope. Actual
 1514PASS,4FAIL,8known opt-in probes/external-SQL ignores,zero filtered.111.546s,
  peak1256932096B,stopnull,remaining[]. Same exact eight ignores as prior baseline;
  /tmp/tsrust-native-carrier-abf5dc0317df-full-library.json and .out/.err. Preserve
 1514passes; do NOT replay full library after routine repairs.
- Four full-library failures diagnosed and repairs APPLIED:
  native_capture_storage panic shared with byte suite: physical lifetime queries
  traversed symbolic recursive fields omitted by native generic renderer. Reuse
  exact scoped_record_native_field predicate at all four query callers; source
  capture analysis remains full. No count fallback,cap or unsafe. ac982d44 patch.
  Dependency guard omitted6runtime names: futures-util,tokio-tungstenite,
  webpki-roots,pin-project-lite,tracing,tracing-subscriber. ce8a3ecc reserves all
  runtime names and tests bothschemas for all65native/54backend manifest entries.
  Two stale test expectations corrected by3f0bb5dd: actual Task result is canonical
  Callable contract; unreachable return after always-returning loop is now an
  explicit negative,while storage assertions stay intact. Root used err().unwrap()
  for CheckedProgram (does not implement Debug),avoiding a private-test compile bug.
- Existing byte negative errors traced to swallowed discovery failures consuming
  source bodies. c880cd6d restores ONLY active unchecked consumed functions from
  existing discovery_body_sources before resetting speculative stack. Definitive
  ownership rejection unchanged. New test proves original return/error survives.
- Runtime-family-repair driver TERMINAL: preflight72.963s PASS; selected build
 554.258s PASS/peak4126676KiB. Six of eight targeted compiler controls PASS:
  canonical Task contract,returning cleanup loop,all runtime pin reservations,
  native-capture three controls. New restore-body unit and existing recursive
  native-family Rust witness still FAIL; byte three hosts still FAIL; editor PASS.
  /tmp/tsrust-runtime-family-repair-faecbb5eb2da.json; gates574.75s,
  peak4259010848B,stopnull,remaining[];cleanup1005764608B recovered.
- Concrete remaining causes: function continuation drains function_stack BEFORE
  discovery receives the ignored error,so prior active-stack restoration could
  never repair failed bodies. Corrected44c4e4c restores unchecked consumed bodies
  from existing retained map; failing regression unchanged. Applied.
- Native count panic resolved; recursive native witness now reaches Rust E0425
  missing Record2: native declaration chose nominal ID for lifetime-free cycles,
  while Drop/Link/Array used concrete ID. Applied9904689e uses cyclic(record),
  matching the existing all-cyclic concrete-field invariant. Same patch omits
  ONLY unchanged nested array identity passthroughs for cyclic concrete storage
  from block tuple shadowing. The owning Map leaf panic was an ARRAY epoch caller,
  not a Map operation. Added failure-only function/binding/path/types metadata
  if that unobserved cyclic classification proves insufficient.
- CLI-first corrected-source driver TERMINAL: normal CLI/host build223.187s
  PASS/peak2764738KiB. BOTH byte ownership-negative hosts now PASS with their
  original native-view-escape diagnostics. Byte68 progresses past earlier emitter
  panics,then compiler STACK OVERFLOW before native compilation. No unit binary
  rebuild was attempted. /tmp/tsrust-runtime-family-frontend-f86a8673b358.json;
  total236.654s/peak2877837992B,stopnull,remaining[];cleanup400375808B.
- Actual profiling: /tmp/tsrust-byte-cyclic-stack-paused.sample.txt. Emit-only
  CLI paused at256590592B,one-second stack sample then killed(total11.309s).
 756samples show record_carrier_conversion repeatedly alternating Record/Union;
  no native program compilation or raised limits. Earlier two-second sample
  captured analysis only and was inconclusive; do not use it as root-cause proof.
- Applied480cb5a0: reuse existing absent_record_field guard to evaluate impossible
  record source once and match its empty required callback enum BEFORE expanding
  recursive sibling fields. Root retained INFERRED local type so boxed/RecordLink
  union payloads retain their actual physical representation; no forced RecordN
  annotation. Remaining cyclic conversions expose type IDs only under existing
  TSRUST_TRACE_CARRIERS. No new registry/cap/unsafe or relaxed ownership.
- Original runtime-family30minute scope is closed INCOMPLETE; both real byte
  negative failures are repaired,while byte68 native and two private units remain
  required. All its build/test/diagnostic processes are terminal. No pass claimed
  for stackoverflow and no reset of that scope. Next source-changed acceptance
  covers the remaining one byte host,then two private failures only after byte
  frontend/emission succeeds; preserve all earlier passes.
- Public status now lists actual source-build v0.2 feature families and links
  test/profiling guide; platform/package/performance/final acceptance explicitly
  remains incomplete. No publication/push/merge approved or performed.
- Byte-family closeout first stage TERMINAL: source e339008f/7864fd53ca4b.
  NormalCLI/host build PASS221.954s/peak2851442KiB;byte68 still STACK OVERFLOW,
  so private-unit build correctly NOT started. /tmp/tsrust-byte-family-closeout-
  front-7864fd53ca4b.json;total235.349s/peak2973078208B,remaining[],cleanup400326656B.
  Existing absent-record guard is sound but did not cover actual repeating pair.
- Exact trace /tmp/tsrust-byte-closeout-cycle-trace.err: source139→target140 once,
  source138→target140 repeated453432times; both nominal declaration70. Inhabited
  physical layouts need inspection; do not invent a general recursive converter
  or weaken scope without actual field/family evidence.
- LLDB diagnostic stalled before target launch (inferior688KB/suspended). Stopped
  after~50s; verified and killed owned debugger31843/debugserver31933/target31932.
  No debugger layout proof; no further debugger attempts. Public Analysis hides
  raw physical records,so an external probe would not provide private metadata.
- Carrier-layout diagnostic build TERMINAL: /tmp/tsrust-byte-carrier-layout-
  build.json PASS220.162s/peak2962182944B; actual Cargo build218.881s,cleanup
 400519168B. Native/user tests NOT run. --timings report shows compiler library
 197.23s(frontend13.91/codegen183.32),CLI18.42s,host2.33s; cached dependencies0s.
- Captured ONLY3layout rows then killed emit-only compiler after10.297s:
  /tmp/tsrust-byte-cyclic-layout-metadata.txt. Module52Holder is ORIGINAL byte
  case50 nullable-recursive-callbacks-join-factories-9 (reversed declarations).
  Source138readUnion86,next104;target140readUnion106,next107. Read is OPTIONAL,
  so source is inhabited. No safe nominal alias or absence shortcut exists.
- Applied reviewed bounded owned carrier worklist patch091ede6a (4files,new
  record_carrier_worklist.rs). Finite expression-local type-pair Enter/Finish
  tasks move records/unions/nested arrays with heap worklists,not runtime recursive
  calls. Existing union_move_payload/union_store_payload handle Link ownership;
  identical payloads transfer directly. Field order/key_order and partial-owner
  Drop retained; no cloning,unsafe,caps or globalregistry. Task generics use only
  source types,Node onlytarget,function either,avoiding zero→one unused lifetime.
- Existing recursive nativefamily witness extended with10002convertednodes on
  its EXISTING64KiB thread and O0/O3 loop; asserts actual distinctsource/target,
  CarrierTask generation,and every original first-owner pointer; retains second
  callback-owner pointer and escape negative. B14 reviewed exact091ede candidate,
  no blocker. Root frontend-only check caught test helper reusing moved first;
  fixed helper to THREE distinct Window inputs(empty(first),second,third). Same
  adopt specialization/probe intact. Equivalent source check nowPASS0.018s with
  currentCLI; Rust/private native validation still pending.
- Quiet old blocking-timeout fixture cleanup retained main.tsr/gate.rs/witness
  files,removed ONLY generated target after no-open-consumer/liveness/locks proof.
  /tmp/tsrust-blocking-fixture-target-cleanup.json recovered63438848B actual.
- Bounded-carrier first stage TERMINAL: preflightPASS68.714s; CLI/host build
  PASS223.337s/peak2887618KiB. Byte68nowpassesfinitegenerationbutstopsEMIT at
  StoredCallable149→StoredCallable0; privatebuildcorrectlyNOTstarted. No native
  pass. /tmp/tsrust-bounded-carrier-front-2b402dcab4c3.json;total235.843s,
  peak3019100936B,remaining[],cleanup400580608B.
- Actual parenttrace /tmp/tsrust-bounded-callable-boundary.err proves failure is
  NEWmutual module1Second224(read149,next205)→Second222(read0,next203). Earlier
  original module52/49 optionalrecursive conversions nowprogress. Source149is
  actualsecond(value) captured environment; no opaque boxing/empty-family shortcut
  justified. Candidate66e37187 remainsUNAPPLIED/unconfirmed.
- Proven registry cause: materialize_recursive normalizedSecond224toitsnominal
  beforeplanning,thenclonedSeconddeclaration(read0) whileonlypreservingFirstroot
  overrides. Appliedreviewed replacementd8383f7f: multi-member path retainsall
  suppliedphysicalnodeplans, exact-leaf/app coinductive candidate reuse, existing
  fullcomponentreservation/cache, ordinaryparameterregistration. Selfpathunchanged.
- Ghostretarget ONLYdirectnonoptionalCallbackIDconfirmedbyexistingAnalyzer
  absent_callback_payloads; two productioncallers passdisjointfieldpredicate(&dyn
  Fn,no newregistry). Optionalunions/unknowncallbacks doNOTalias. Peerreviewcaught
  optionalinhabited FirstGhost.nextSecond149 beingerasedbyRootSecond150; newunit
  assertsboth149+150,root148andinhabitedoptionalghost remainreachable. Otherunit
  covers148/149,backedge,repeatcache,concrete-inputfixedpoint/noarenagrowth,150distinct.
  B06peerreviewd838clear; no executionyet.
- Mutual-carrier first stage TERMINAL: preflight70.921sPASS; CLI/host build
 218.399sPASS/peak2895554KiB. Byte68advancestoEMITStored149→Callback72(verified
  emptyghosttarget),no nativepass/privatebuild. /tmp/tsrust-mutual-carrier-front-
  b70ec615d43b.json,total231.194s/peak2966901560B,remaining[],cleanup400171008B.
- Exactconsumerproof: start_record_work records.rs130 chooses recursive_parameter
  BEFOREliteralfields/defaults; resolved_prepared_call analysis/mod.rs8666 does
  samefornominalfactoryresults. Newmulti plannerregisteredALLmembers,sofirst148
  constructorregisteredrequiredemptySecond72 as soleSecondcandidate. LaterSecond
  selectsghost72. Wrapperdirectionisconsistentwithwrongselectedtarget; doNOT
  loosenNative149→absent72 conversion.
- Appliedbdc8ac2d filtersBOTHordinaryregistration sites usingexisting confirmed
  DIRECTabsentCbIDs. Layout/cache/linksretained; requireduninhabitedmembersnot
  recursiveparametercandidates. Rootexplicitlychecks!field.optional too; optional
  properties/unionsstayeligible. Regressionchecks first148ghostSecond72 retained,
  constructor/resultfallbackremainsnominalSecond1,thenliveSecond149isselected.
  Bothagentsconfirmedconsumerpath; no opacity/conversionworkaroundor66eguard.
- CURRENT /tmp/tsrust-live-recursive-run.py at7c171bb8, fixed30minuteacceptance: compiler/LSP
  testpreflight,remainingbyte68bothprofilesfirst,thenfailedrestoration/deepnative
  plusaffectedregistry/emptycontrols andfreshLSPbyteagreement. Preserve earlier
 1514lib/6focused/25byte/27LSP/2escape-negative/crypto24/collections42/public55/export
  passes. Source remainsnotaccepted until actualnative. No push/merge/publication.
- Quiet cleanup /tmp/tsrust-stale-worker-target-cleanup.json removed ONLY ignored
  generated targets from stdlib-bytes(native cache) and stdlib-path(rustc info)
  managed worktrees; source/WIP branches untouched. Recovered167149568B actual.
- Deterministic/customization-safe adapter generation actual PASS retained at
  /tmp/tsrust-utilities-http-repair19b-66af5f02808e.json. Only middleware positive
  and real-generated-shlex offline export hosts remain unproved in that pair.
- Additional cleanup /tmp/tsrust-obsolete-http-cache-cleanup.json pruned9cache
  entries containing replaced serve_connection(io,service) template under quiet
  build/cache locks; ACTUAL recovered2102931456B. No shared dependencies removed.
- Additional cleanup pruned6native cache entries with obsolete dependency identity,
  recovered155926528B; /tmp/tsrust-accepted-runtime-obsolete-cache-cleanup.json.
- NEXT: finish bytes mutual-layout assignment,then coherent compiler/LSP
  milestone and still-unproved B10 middleware/B19 shlex offline export boundaries
  LAST after compiler work. Middleware already has a genuine relocated example
  host; no new harness is needed. Public runtime TLS staging/migrations are done.
  Resume comparable timings only when quiet,retaining unchanged controls/proofs.
  Remaining full B00–B20 milestone/platform acceptance stays active. No push,
  merge or v0.2.0 publication approved by this local checkpoint.

Source migrations now APPLIED/formatted; acceptance pending for all three:
- B16: /tmp/tsrust-crypto-runtime-suite-migration.patch
  SHA538e29224c353b64560a90fdd060751f6ab21fa092e39a607b624cd6f18240ef;
  25old launches→24unique registered cases, debug/release batches,29catalogrefs/
  20verbatim examples.27paths; exact hashes/case map in matching -inputs/-cases.
- B14 collections: /tmp/tsrust-collections-stable-suite.patch
  SHAcac1163fcaf5b34739f6d9dd44922043df24cf32fb379eb8e74de5a0994ecc2b;
  37run sites→42programs→42registered cases; original bodies/comments,25checker
  sites/all8Rustc controls retained.44paths, matching -inputs case map.
- B02: /tmp/tsrust-bytes-registered-runtime.patch
  SHAc90267eb33bf5757710f5a806592177baca427199de300db489d4f74a8bd9eb3;
  67positive programs (24default-only,43bothprofiles)→one67case suite in both
  profiles. Original builders/negative checks retained; source bodies change only
  exported entry and final newline. Matching -inputs includes exact mapping.
All actual canonical apply-checks passed when prepared against0d090688; recheck
before applying. None touches current55cohort/core compiler. Format/source-check/
execution still required; never count prepared patches as accepted implementation.

B11 benchmark historical-root guard selfcheck PASS after0d checkpoint: both old
and current roots must match15sealed control/protocol files and actual Cargo
manifest/source paths. Private default unchanged. No controls retimed/relabelled.
/tmp/tsrust-b11-public-native-measure.py UNRUN, requires actual successful55case
host log/gate JSON and current CLI SHA before public release-server build,
protocol/native-only timings, cached-control reuse and npm run store:ingest.
Ensure portable source preparation prerequisite before emit package. Old actual
Rust/Node controls remain intact; no public native timings/ingestion claimed.

Prior cleanup5711986688B and491126784B remain separate from this continuation.
Full-goal next scopes remain B20 integrated applications using existing B10/B19
examples, affected acceptance/exports, actual matched measurements, B00 ledger/
default/minimal features, full regressions and four-host native packaging gates.
Future GPUI/Kit, .tsr client codegen/shared frontend/backend roots follow v0.2.0.
No push/merge/publication. Never redefine the goal around the current cohort.

## Earlier evidence (retained; current state above supersedes continuation advice)

## Current source checkpoint preparation — 2026-11-07

Canonical codex/stdlib-0.2.0 after a96b1d79. Previous turn made source and acceptance
progress, not completion. No build/test process is live. Full B00–B20 goal active;
no push/merge/publication. Source is formatted; native acceptance remains incomplete.

B17 integrated compiler/runtime/editor/reference/examples + three matched workloads.
Use stable std:testing suites, no standalone backend probes. All compiler/LSP test
harnesses TYPE-CHECKED with logging enabled: 67.028s, peak2181745904B, stopnull,
remaining[] (/tmp/tsrust-logging-unit-harness-typecheck.*); this includes cfg(test),
not just selected integration targets. Earlier selected cargo check --lib did NOT
cover unit bodies, so final preflight explicitly uses --tests. New pin-project-lite
PinnedDrop wrapper requires literal Pin type syntax, now corrected and checked.
Existing exact pin-project-lite0.2.17 direct edges, no new package/version/boxing.

Original public logging scope /tmp/tsrust-public-logging-scope-start.json:
started1794052719.0877151,3300s absolute. CLOSED INCOMPLETE; do not reset/restart
its oversized repair sequence. Last owned check finished with remaining[] before
55-minute deadline. No exports or timings claimed. Preserve actual passes below.
- First source2b6f build17.65s failed const parameter slice promotion; fixed.
- Source32d14 build229.715s failed runtime-test super import; fixed to self.
- Source4319 build204.330s failed Vec/reference assertion; fixed as_slice.
- Source1b4d build/gates380.247s, peak4184348840B, stopnull/remaining[]:
  seven passes/six failures. PASS file-append/scalars, interleaved contexts,
  Logger/LoggerOptions/LogError agreement,12 completion phases,module coverage,
  catalog reachability,negative sources. Cancellation bare-backend drop failed.
- Source223b build203.407s failed pinned-drop macro qualified type; fixed Pin.
- Source74d2d8ddc03c, attempt6 terminal231.012s, peak2871235288B,
  build-finishedtrue,stopnull/remaining[], four passes/three failures. PASS logging
  hover, unused-module/type-import/shadowing selection, both old callback-error
  guards. Unknown stdlog/iterator helper targets are repaired by preserving only
  trusted stateless wrapper symbols outside module-owner lift. Ordinary module
  functions retain program-owned capture storage (new structural witness unrun).
  FAIL two iterator cases: first typed catch48 remains unresolved, including
  effect-only nominal stored-field summary candidate (not accepted as a fix).
  FAIL logging suite first reached moved logger in a Once factory; fixture now
  creates/closes logger inside factory, reports actual error value through Channel.
  Direct same-suite check0.550s then reached unsettled-job at example48. Proven
  fixture gap: fallible readiness assertions occurred before cleanup try. Fixed
  source by guaranteed cancel+await in finally, then asserts requested/cancelled.
  Tracker consumes await operands BEFORE failure snapshots; no checker weakening.
  Corrected full suite has NOT been rerun. Runtime acceptance is still pending.

Actual docs traces used ordinary docs command, no native probe:
/tmp/tsrust-real-docs-carrier-trace.*3.158s pinned inhabitedRecord145/family9 being
replaced by absentRecord261 in a mixed union. Narrow direct-required-absent fold
repair applied, no populated-to-empty conversion permission. Fresh74d docs check
9.338s then advanced to Unknown example http-middleware (no carrier rejection).
Added actual docs/src/examples/http-middleware.tsr +config registration, formatted,
unrun. This is progress beyond former error, NOT full docs/native acceptance.

Unrun internal witnesses: cancellation/interleaved context after new drop owner,
trusted wrapper/module-state structural control, mixed-union fold controls,
nominal stored-field unknown/cycle controls and direct typed iterator diagnostic.
The diagnostic captured with OLD1b4d library binary0.556s remains
/tmp/tsrust-iterator-direct-effect-diagnostic.*. New trace adds closure captures
and actual call targets, but no new library test binary executed that trace yet.
Next milestone must include these guards, full corrected public suite, same-suite
relocated O0/O3 export LAST and matched timings with actual SQLite ingestion.
Do not replay unchanged accepted editor/negative scopes after fixture-only fixes.

Storage cleanup (locked/quiet, linked objects only) this turn:1120702464B +
469340160B reclaimed; previous profiling-turn906670080B separate, avoid doublecount.
Reports /tmp/tsrust-public-logging-before-repair-cleanup.json and
/tmp/tsrust-public-logging-closed-cleanup.json. target/debug14G/release3.7G;
preserve current consumers/libs and unchanged Rust/Node controls. Object pruning
is separate storage maintenance after terminal process cleanup (all remaining[]).

B11 coherent SOURCE-ONLY patch ready, NOT applied. Latest
/tmp/tsrust-b11-public-source.patch SHAb906740b56400aaddcd80bff21ea0f3b58c70640fa3cbc30611e0e01c9e45e83;
exact inputs /tmp/tsrust-b11-public-source-inputs.json. Central manifest proposal
/tmp/tsrust-b11-http-websocket-manifests-current.patch SHAc9518dc85506049fe7eb20c2af37cc44fa1c70f32224314274a062220eb0a628.
Includes compiler/runtime/editor/docs,18-case stable suite, strict JSON/exactnames,
prepared-native benchmark hook preserving old controls. Review caught E0716
parameter indexing, missing futures-util direct edges (all3manifests, exact
already-locked0.3.34 std+sink), bare bytes dependency (uses axum::body::Bytes now).
No locks/builds/formatter/measurements in worker. Root owns production locks and
must revalidate current baselines before integrating. Workers idle with preserved
isolated changes. Single writer canonical; one shared target/inc0/jobs1.

## Historical logging integration preparation — 2026-11-07

Canonical HEADa96b1d79, dirty source preserves owner progress/Owned predicate.
Integrated corrected complete B17 public patch3fc9507acc935fc653319477e754c0b68a0558285a15e43311449927e5d063e2
(excludes private admission), central19e6proposal then Cargo-managed offline locks.
Proposal lacked valuable cfg edge: offline metadata minimally adds valuable0.1.1;
all3locks now pass offline --locked resolution, old package versions preserved.
Logging compiler/runtime/editor/docs/examples +3matchedworkloads now in canonical.
Normal stable five-case std:testing suite first; relocated same-suite export is a
separate last gate. New unused-import/factory dependency-selection witness pending.
Root also integrated stable B15 registered iterator suite; changed wrapper to
stable canonical source/cwd (agent copied temp sources; root removed those copies),
retained ordinary app main +unnamed-catch emit witness +all existing assertions.
Existing carrier-rejection trace now opt-in CLI TSRUST_TRACE_CARRIERS=1, default
unchanged; use real docs command after fresh CLI, no standalone native reproducer.

Whole formatter6sourcefiles/24snippets/3fingerprints then stable0/0/0; Node2logging
source/oracle checks PASS. First selected build terminal17.65s/peak1440829088B,
stopnull/remaining[]: E0716 const EMIT borrowed ENABLED.parameters[1]. Root replaces
only that expression with same direct parameter declaration; no semantics change.
First evidence /tmp/tsrust-public-logging-budget.* and fingerprint2b6f675def43.
LIVE resume session54504; supervisor64615,driver64699,watch64712,CargoPG64713.
Do not duplicate/restart. Current source fingerprint32d14e03e52d..., build output
/tmp/tsrust-public-logging-32d14e03e52d-selected-build-artifacts.jsonl +build.log.
Watch /tmp/tsrust-public-logging-attempt2-budget.{out,err,json}; aggregate original
/tmp/tsrust-public-logging-scope-start.json start timestamp is in that JSON,
3300s absolute including firstfail/repair (never reset). Selected compiler logging
runtime3unitcases +normal suite/negative/unused-selection, stableiterator2cases,
LSP logging/catalog/12matrixphases/Logger,LoggerOptions,LogErroragreement.
One shared target/inc0/jobs1, --features tsrust-compiler/logging, threads3;
no offline exports in first pass. Full private units/old unaffected gates excluded.
B03 source-only suite patch integrated/corrected; B06 review finds no further proven
bug, bound method extraction contract source review pending. B14 realtime coherent
source still underway (runtime/compiler/catalog/suite/docs drafted; matched driver
uses explicit prepared-native mode preserving unchanged controls), no heavy lease.
No new public measurements ingested; all three new logging workload rows measured:false.

## Public test profiling accepted — 2026-11-07

New `tsrust test --profile` reuses existing build-stage timers, adds an optional
initial NDJSON build event, text profile on stderr, default JSON unchanged. Stable
examples/testing is one native runner: threefiles/sixcases. New host orchestration
checks duplicate/non-test flag rejection, default report, then profiled same
runner with cache=hit/nativeBuildMs0, six actual test events and summary. Full
focused scope TERMINAL27.691s, peak930432272B, stopnull/remaining[], build-finished
true; CLI/test-target build22.828s, host execution4.162s incl suite executions.
Actual profiled setup213.303166ms, test summary0.7395ms. These are correctness
observations, not protocol benchmarks/SQLite measurements. Artifacts:
/tmp/tsrust-test-profile-build.jsonl, /tmp/tsrust-test-profile-scope.{json,out,err}.
Text reporter sixpasses too; separately launched environments caused cache misses,
so an initial smoke assertion expecting cross-process cachehit was invalid. Repeat
plain scope2.126s, no stop/children, actual native/link1528.4ms; recorded rather
than hiding/restarting previous suite. Env hashes intentionally include build
script inputs; no cache-identity weakening. Both whole-project format passes
0changes, snippets0/fingerprints0; diff whitespace clean. Cleanup locked/quiet:
/tmp/tsrust-test-profile-objects-cleanup.json,147linked objects removed,
906670080freebytes reclaimed. Current compiler includes --profile; LSP unchanged.

B17 public source patch SHA619a6de1dd6e04899881189b3273a6b19489315533624ce8bad0c717cda946ea
passes git apply --check only, not yet integrated. Central proposal review caught
missing TOML commas; worker repaired and tomllib-validated all6proposed files:
/tmp/tsrust-logging-central.patch newSHA19e6e4b88a6af396aaf69bd58c8c675050d590aa3458d7c27b59e117afc96983.
Old f78b6proposal superseded. No production manifests/locks changed yet.
Fixture explicitOwned annotation remains pending acceptance; retain uncommitted.
B03 confirms docs span is receiver `props` (5chars), not `props.id`; actual source/
target record layouts require evidence before repair. No scalar widening guesses.

## Owner test-workflow correction — 2026-11-07

Use stable std:testing suites through tsrust test, one native runner for related
cases, same suite for export, unchanged Rust/Node controls reused. No further
private standalone backend probe builds. Owner explicitly permits testing-library
profiling/method extensions. Existing durationMs already reports test execution;
new --profile CLI separates discovery/setup/check/emit/backend/cache/native build.
Implementation + stable examples/testing six-case orchestration witness pending.
B17 frozen public source patch /tmp/tsrust-logging-public-source.patch includes
five registered cases and matched support, not yet integrated/validated.

Root late-factory gate is TERMINAL (session8471 closed), ac8bec50/3bbecb368457:
225.277s, peak2,934,641,248B, build-finishedtrue, stopnull/remaining[],2PASS3FAIL.
Both existing callback-error controls PASS. Named/unnamed iterator fixtures now
reach callback-escape on an unannotated named predicate. B03 independently proved
this fixture requires explicit Owned<Function<boolean,[value:Owned<TrackedResource>]>>
at creation; its resource-drop oracle expects a moved environment. Annotation
applied without weakening guards; no rerun yet. Docs still fails std:ui128,
actual differing owning callback record layout remains unresolved. Earlier
535.595s compiler build/5PASS4FAIL evidence preserved. Diagnostic scope original
1794048868.3530629/3300s closes incomplete; do not reset/replay its repair sequence.
A separate owner-requested public profiling feature scope will build the CLI and
run only the stable testing example suite and option checks; no old suite replay.

## Current state — 2026-11-07

Canonical: /Users/seancotter/Desktop/dev/learning/tsrust, codex/stdlib-0.2.0.
HEADac8bec50 (new specialization queue +uninhabited record/scalar compatibility repairs).
Last successful executed compiler consumer ac8bec50. Historical root session8471 (TERMINAL):
/tmp/tsrust-late-factory-repair-budget.*, same original diagnostic scope start
/tmp/tsrust-blocker-diagnostics-scope-start.json,3300s absolute including repairs.
This is affected integration-only refresh, no --lib/no replay of prior passing
B19/crypto/Task/LSP groups. Queue repair source-reviewed independently by B03/B14:
postponed catch retries can discover fresh specializations, which must drain
through the existing sequence before retry/finalization. Retry with no newwork
returns original unresolved error; no empty-family/template/effect suppression.
Named full native iterator fixture, unnamed-catch no-panic witness, existing
constructed/unknown callback-error controls and authored docs are selected.
New private named/unnamed unit variants await a later meaningful library gate.

Earlier root diagnostic ec11431e/a29c8df89d97 TERMINAL546.107s, peak4,116,551,776B,
build-finishedtrue, stopnull/remaining[],5PASS4FAIL. Fresh compiler-only build535.595s.
/tmp/tsrust-blocker-diagnostics-a29c8df89d97.json and budget.json. PASS fullnested
Rust adapter actual native, imported docs source-range unit, direct callback modes,
Job iterator ownership unit, literal scalar widening unit. Remaining UI traces
prove source records have registered empty TsrAbsentCallable field; new coalesce
uses Rust empty matches only for direct required ordinary-record absent fields
(!optional, no arrays/opaque/type-only guesses). New scalar compatibility mirrors
existing emitter rule all members must satisfy target.accepts_scalar_value.
Fresh diagnostic family22 trulyempty decl12 whilefamily23=[43] pure; source-only
unnamed-terminalcatch diagnostic input panicked all-functions-analyzed, proving
newfunction scheduling hole. Canonical acceptance fixture never weakened.


Current continuation: previous turn was concrete progress (native5passes, source
834f fix, actual5,357,232,128B cleanup), not a stalled/restarted build. Initial
process inspection found no live compiler/build/watchdog. Canonical dirty edits:
README v0.2 feature/example overview and docs_cli imported diagnostic renderer
(reuses project_cli renderer; new formatted source-range witness). Neither is
compiled; no canonical source checkpoint until whole-project formatting slot.

B17 heterogeneous retained-field extension also PASSED3.774s, peak270,368,768B,
stopnull/remaining[], exact old actual optimized rlibs/source/binary hashes.
/tmp/tsrust-b17-child-fields-result.json. Only6newgroups/93samples per generic/
erased shape after8; no replay36controls. Pointer/capacity and parent/child
Drop0whileheld→1afterfinalrelease, typedJSON and2request ancestryPASS. Erased
closure/nativegeneric allocations equal; signed+677B is warmed cache observation,
not an after-drop/leak claim. Backend choice opaqueLogger/Arc static-span closure
now has direct evidence; public integration remains UNVALIDATED.

B17 PRIVATE backend admission now PASSED in clean original logging tree3139162e.
Normal build/guards from attempt1 preserved; exact host-filtered metadata repaired
overapproximation without any source/runtime change or passed-normal rebuild.
Only unrun allocation/stress builds7.787s, profile5.212s, stopnull/remaining[].
Original55m deadline unchanged. All6processgroups/36timing invocations1116samples,
24allocationrows, original3x31after8; quietmax3.18359375. Disabled0alloc/retained;
format1036 andchild10252allocs equal directRust, retained217088/362496B are still
ownedoutputbuffers, not leaks. Correctness includes final sink/owner release.
/tmp/tsrust-b17-logging-attempt2-result.json and supervisor.json. Private results
are NOT public workloads or SQLite ingestion. Public B17 implementation underway
in same managedloggingtree newcodex/stdlib-logging-implementation at834f; only
old admission/logging restored from3139162e, original branch preserved. Source-only
opaqueLogger, static tracing scalar fields, lazy checked Once callback facade,
append-safe synchronous sinks, per-poll async instrumentation; no new generic
publicowner/dynamicmetadata/customformatter. Heterogeneous child probe pending.

B14 now owns existing managed realtime tree, CLEAN99c9admission preserved then
newcodex/stdlib-realtime-implementation at834f with only admission/realtime restored.
Existing full SSE/WS admission was found here; reuse it, no duplicated preparation.
B11 private admission TERMINAL35.455s profile, peak598,952,424B, stopnull/remaining[].
Originalstart1794048144.843 completed~8m40 including preparation/repairs, no reset.
All3correctnessgroups+2engines actualPending-send shutdownPASS.36timingrows×31=1116
samples,142848requestlatencies,36separateCPU/RSSrows, originalprotocols intact.
/tmp/tsrust-b11-profile-attempt8-result.json; historical terminal-success-with-
unresolved-sse-semantics / publicBackendAccepted:false retained honestly. Owner
Rust/backend contract selects exact Axum empty/CRLF behavior, not WHATWG parity.
B14 now publicsource-only implementation; no heavy lease. SSE/channel +explicit
sseWithProducer owns Job, converts post-header failure to existing HttpError and
drops typed E exactlyonce after settlement; detailed E handled inside producer.
WS uses reserved listener-Scope Job beforeupgrade and backendowned consuming split.
Compiler/editor/examples/matched/TLS/copy acceptance not yet executed.
Historical admission phases below describe preparation, not current live processes.
/tmp/tsrust-b11-attempt1.json and later attemptN-supervisor/out/err hold actual
handles. Cached exact Nodews8.18.3 installed+integrity verified; RustSec309392
no-fetch audit0vulns/warnings; hostmetadata/archive/source and optimizedactual
closure proofPASS. Actual-artifact features are checked subsets of metadata
optional/target upperbounds, exactrootfeatures. Original protocols unchanged.
SSE10framing/3lifetime controlsPASS; WS malformed/UTF8/limit/backpressure/cancel
controlsPASS. Delimiter/guard/invalid-close-fixture repairs preserved actual
passed scopes. Simultaneous-close fixture must move endpoints into join futures
so completed side drops its socket; no backendprotocol change. No timing yet.
Owner's explicit Rust/backend semantics select exact Axum empty/CR/LF behavior;
raw+WHATWG differences remain documented, no normalization/handmade encoder.
Private backend choice is NOT public generated facade acceptance. Root/B06 do
source-only work while B14 owns timing/build slot. No rootcompiler gate started.

All compiler/build/watchdog processes verified terminal before source edits and cleanup.
Repair19d is TERMINAL at6ebee54b/cffa06ffa9c3:236.293s total,
peak2,880,983,600B, stopnull/remaining[], build-finishedtrue;5PASS2FAIL.
Build224.246s; only compiler/affected integration consumers, no compiler --lib.
Results /tmp/tsrust-utilities-http-repair19d-cffa06ffa9c3.json; outer
/tmp/tsrust-utilities-http-repair19d-budget.json. Fresh normal CLI/LSP6ebee54b.
PASS: actual mapped startedJob identity→toArray→all nativeoutput3,3; nullable
iterator state and real ByteView escape/mutation guards; O0/O3 nullish fixture;
borrowed nestedResult payload discriminator read +mutation/move negatives;
Rust signatures/source ownership reject controls. Earlier crypto and nominal
pointer/capacity native passes retained.
Source-only after closure: shared commonfield receiver now reuses existing
conditional_arm(Read) owner staging instead of expr(Read), retaining temporary
function return through assertion borrowing. Wholeprojectformatter0/0/0; no new
redundant source-shape test, existing native E0716 witness remains required.
This one-line fix is NOT rebuilt/executed; CLI/LSP still6ebee54b.

Remaining FAIL: fullnested adapter now reaches Rust E0716 from borrowing
unit(none).kind/unit(some).kind temporary (B06 shared receiver staging repair);
authored documentation advances through prior coalesce panic to std:ui owned
callback record-family rejection (B03 exact source diagnosis). Typederror source
family22 remainsopaque from19c; do not rerun until provenance/repair changes.
New privateunits/familytraces are not executed; full family milestone remains open.
Middleware actual offline/native and generated shlex offline exports remain
required and UNRUN; no passes invented. Original19 scope CLOSED with cleanup at
~3240/3300seconds, no resource stop or surviving children, incomplete acceptance.
Do not reset or replay this same oversized selection as serial hour-long chunks.
Future checks require coherent new source changes and affected scope only.

Final quiet cleanup removed410linked rcgu.o leftovers:1,126,985,728 actual bytes
recovered (/tmp/tsrust-repair19d-objects-cleanup.json). This continuation reclaimed
5,357,232,128 actual bytes across three cleanup operations. Target is now18G
(debug13G/release3.5G), down from23G observed at this continuation's start.
All source/control/current consumers preserved; no cargo clean/refill.

Repair19c is TERMINAL: selected build549.707s, peak4,012,964KiB,
build-finishedtrue. Initial preflight used wrong task_diagnostics::tests prefix;
correct cursor_tests prefix discovered from actual --list. No rebuild: exact same
22e002fc/d2cd59ed7ca1 artifacts resumed direct execution40.789s,
peak795,459,584B, stopnull/remaining[],6PASS10FAIL. Results:
/tmp/tsrust-utilities-http-repair19c-d2cd59ed7ca1.json; original build outer
/tmp/tsrust-utilities-http-repair19c-budget.json, execution outer
/tmp/tsrust-utilities-http-repair19c-execution-budget.json. Original19start unchanged.
Fresh passes: all3remaining crypto native/catalog cases; middleware negatives;
nullable capture/nativecounter +realborrowescape/live-mutation guards; generated
nominal record O0/O3 pointer/capacity preservation. UI checks now progress to emit
record remapping (Alert Record107→54, Keyboard219→119); two new UI witnesses
need valid nominal/lifetime construction. Iterator Job diagnostic proves missing
direct-call origin mode; reviewed narrow source delta applied, not yet rebuilt.
Resource forEach typedcatch remainsopaque. NestedResult Option payload field
lookup rejects legitimate borrowedunion; B06 tracing checker+emitter repair.
All test processes terminal; no heavy lease live. No exports claimed passed.

Latest completed repair19b: source8c54386f, fingerprint66af5f02808e,
629.219s, peak4,140,701,720B, stopnull/remaining[], build-finishedtrue,
69PASS11FAIL. Exact results /tmp/tsrust-utilities-http-repair19b-66af5f02808e.json;
outer /tmp/tsrust-utilities-http-repair19b-budget.json. CLI/LSP and selected
binaries are fresh for8c54386f. The prior Argon2 verifier correction passed actual
module-password execution; optional CORS units, nullable iterator counter native,
named callback inference, literal-hint guards, Rust editor agreement and all
Task-array/pool transports passed. Preserve those completed scopes.

Reviewed source repairs now applied, execution pending: literal-to-scalar carrier
and nullish fallback mapping; Rust record-array collection through its concrete
alias; explicit generated Option seed tags including SomeUnit; borrowed password
options extraction (previous output silently selected all backend defaults);
project execution of imported crypto field examples; valid scoped ByteView callback
positive plus true escape/live-owner-mutation negatives. Password example now
asserts actual encoded custom cost and output length. Original repair19 start is
/tmp/tsrust-repair19-scope-start.json,3300s absolute including repairs and cleanup;
never reset it. Remaining unresolved: started Job iterator-map responsibility and
resource forEach typed-error metadata after a prior catch. B14 source-only diagnosis.
Required middleware native/offline and real generated shlex/offline exports remain
queued LAST after compiler checks; neither has been accepted by repair19b.

At verified quiet point, lock-protected cleanup removed3040 leftover rcgu.o files:
2,931,077,120 actual free bytes recovered. Report /tmp/tsrust-repair19-objects-cleanup.json.
Additional quiet-point cleanup validated543 exact obsolete runtime files from181
closed temporary workspaces, currentnamed/value/pathconsumer counts0 with1546
positive controls, preserving41live-source and26unknown/non-temporary roots.
Deleted543files; actual freebytes43953741824−42654572544=1,299,169,280.
/tmp/tsrust-runtime-refresh-applied.json. This continuation actualrecovery4,230,246,400B.
Prior cleanup recovered6,951,444,480bytes; do not count those again or claim this is
net growth reduction. Preserve linked consumers, reusable libraries and controls.
No push, merge, release or public timing/SQLite ingestion in this continuation.
Live remote read: no open PRs (including Dependabot); CI49 cancelled, CI50
startup_failure, CI51 success at794c839735cf is supporting-host UDP timing ONLY.
Its compiler/LSP/native-platform families were skipped, not full acceptance.
GitHub connector search unavailable; gh read-only fallback used. No workflow dispatched.
One writer per tree, one shared Cargo target, jobs1/incremental0; tests threads3.
Every scope <=1h, original55m aggregate and <=30m/5GB inner guards.

## Latest exact acceptance

All repair18/18b/18c/18d/18e belong to ONE original55m absolute budget, including
repair gaps; no reset. Final18e finished beforedeadline, not resource-stopped.
Private library witnesses changed after18c were NOT rebuilt/executed in18e;
they remain explicitly pending. Do not use stale library binaries to prove fixes.

- repair18 build-only301.95s: test Arc movedcounter error; fixed.
- repair18b c7cdaf14/f304ca82101c:881.223s, peak4,138,145,816B,
  stopnull/remaining[], build-finishedtrue;48PASS19FAIL.
- repair18c aaf1ab3a/d995e2bfd2d5:690.857s, peak4,140,570,576B,
  stopnull/remaining[], build-finishedtrue;42PASS13FAIL.
- repair18d build-only11.513s: Capture reference.clone producedVec<&Capture>;
  fixed using explicit copied7metadatafields, no new clone capability.
- repair18e895205bb/b2cedc3354d1:267.889s, peak2,919,469,664B,
  stopnull/remaining[], build-finishedtrue;19PASS7FAIL. Results:
  /tmp/tsrust-utilities-http-repair18e-b2cedc3354d1.json;
  /tmp/tsrust-utilities-http-repair18e-budget.json. No surviving child processes.

Preserved native passes: forms/multipart relocated vendored offline O0/O3 with
forbid(unsafe_code), uploads O0/O3, fixed digest/Hasher/HMAC/HKDF all readable
positions O0/O3, legacy owned mixedunion/Mutable default actualnative, four callable
record positives plus allnegativeguards,9Taskarray tests and5pool/error transports.
B16 lifecycle/typedargument/opaqueHasher controls pass; password export manifest
now selects crypto-password and full async pool support. Publicpassword execution
stillfails the above Argon2 &String call; d33fix pendingverification.

Preserved editor: crypto hover/signatures/7optionfields/2Hashermethods in4phases,
4crypto/CORS receiver checkeragreement, CORS newrow4phases; earlierHTTP2rows8phases
andcrypto4rows16phases actuallyobserved (oldmatrixdriver countbug corrected without
replay). Full affected editorhelp and catalogcompilerwitness PASS. Runtime/private
middleware construction/contextcompression/bodydrop/error/sourceorder controls pass;
new marker/prelude extraction assertion andoptionalCors privateunits awaitfreshlib.
Actual defaultTLS HTTP2 ALPN/privateCA/loopback transport passedrepair16.

## Superseded repair18 diagnosis (latest results above take precedence)

- Password runtime extraction, explicit crypto-password feature in nativeCLI AND
  standaloneexport, and asyncPool selection are now implemented; actualexecution
  progressed to admitted verifiergeneric API mismatch, fixedsourceonlyd33.
- Middleware positive example now stages await c.req.text() before c.text(body),
  and CLIchecks beforeportable setup. Actual typedCorsFactory=cors fails during
  asyncdiscovery: provisional function_value hardcodes Required/restfalse. Source8b1d4e9f repairs
  fallbackmetadata using sourceinitializer omission/rest and async typed-first
  regression; freshnative/privateproof pending. Do not add explicitundefined workaround.
- Iteratorretry now reaches strictEqual(error.code,'selected') but coarseString
  hint conflicts with alreadyboundliteralT. Source8b1d4e9f adds exactboundliteralhintdeferral
  preservingT and finalwrongliteral/typedString rejection; no blanketwidening.
  Freshnative/privateproof pending.
- Stored nullable current.next loses its selectednonnull fact after namedassignment
  and branchjoin. Rootlocalized: namedstore invalidatesselectedRHSfact, thenbranchjoin loses
  nonnullproof. Anyfix must normalizephysicalrecord to existingcheckedpayload
  withnominal/item/owned/no-view guards, notallow nullable.next. No guessedflowfix.
- Retainedkeyboard UI check+emit path repaired; actual documentation_content now
  reaches docs/src/components/ui/alert.tsrx ownedcallbackcarrier failure. B03 owns
  focusedprivatewitness/actualshape diagnosis, not UIfeature expansion.
- Freshprivate Taskmethod responsibility/contextualRead, namedcallbackgenericresult,
  PWruntimeextraction and UI/marker units remain required. Newtracker analyzes
  actual non-template polymorphicmethods instead of emptydeclaration bodies;
  preserves abandonment negatives. Namedcallbackinference preserves shadowing,
  explicitnon-generic guard and exactparameterchecking.

A separate <=60s CHECK-ONLY9probe diagnostic completed~0.4s after18/cleanup:
/tmp/tsrust-18e-literal-optional-check-probes.json. CLIhashb8c1278ba18a43991d4cd0046da6283ed7804bae31314f9139b85d8758a067a4.
Not an acceptance reset or native proof. Workers may do source/lightdiagnosis;
NONE has a heavy build/profile lease. Root alone edits canonical.

## Other dependency-ready work

B11 source audit confirms SSE/WebSocket public APIs absent; no acceptance claimed.
Prepared SSE backend admission only (UNRUN) in existing HTTP worker:
/Users/seancotter/.codex/worktrees/stdlib-http-admission/tsrust/benchmarks/runtime/admission/http-sse.
Review patch /tmp/tsrust-http-sse-admission-source.patch SHA256
5b29fc84d0247737a09116f76aadf1a34d0a24f372f434b4fbaadf4f36f8791b.
Exact file inventory /tmp/tsrust-http-sse-admission-source-inputs.json.
Pins existing Axum0.8.9/Tokio1.53.3/http-body-util0.1.5 stack; backend framing,
independent multiline/Unicode oracle, bounded channel/drop/KAT and18profilegroups.
README queues exact lock/metadata/build/profile commands, shared target/no downloads.
No heavy lease yet; profile first, then public Channel/Scope/compiler/editor/example/
matchedbenchmark implementation. WebSocket needs dependency admission plus real
upgrade-capable connection serving/scoped upgrade-task cleanup; not just a route.


B19 integrated sourcecheckpoint9ae0c2b9 after37fileexactoverlaycc8070 (private
admissioninputsunchanged). RecursiveOption/Vec/String/record/outputResult adapters,
move-onlynominalprojections, existingbindingsCLI selectedgeneration, hashguarded
regen/customization-safeeditableuncompiledstubs, realformat57reader/sourcefallback,
exactstd/coreDropalias/qualifiedselfguards, editor/docs/examples/4matchedworkloads.
No newproductiondependencies/manifests/locks. Rootreserves5admittedcrypto names.
NominalfixturelinksactualTsrust-emittedapp toseparatebackendrlib O0/O3; exactRayon
librarycomesfromselectedCargo fingerprint, notnewestmtime. Native/editor/export/
measurementUNRUN. TwoindependentNodeoracle/sourceguardsPASS0.238s;wholeproject
format5sources/4fingerprints then0/0/0. Alloldbenchmarkrows/protocolsunchanged.

Current19primaryscope12targets includesfreshprivate library witnesses, fullRustbridge,
2nonexportgeneratorcases, affectedRustLSP/nativecrypto/HTTP/Task/iterator/docs.
TWO requiredexports explicitlyqueuedLAST underSAMe19deadline: middlewarepositive
andrealgeneratedshlex relocatedoffline. Neither isclaimedcoveredbyprimaryscope.
Do not buildportableexports beforeplannedcompilerrepairs; fixesmayresumebudget.

Guardednullableassignmentflow89c integrated9ae; checksameownedknown/no-view
scopednominal+exactapplicationargs, onlyprovenrecord-onlyfacts normalize tocurrent
physicalpayload. Undefined/unknown writesdo notpublishfact. Actualcounter+5negative
unit/nativeexistingfixture pendingfresh19. ActualAlert privatecheck+emitwitnessand
failure-onlytraces887e integrated; no Alertproductionacceptanceguardchanged.

B19 privateprofile PASS47.454s:114groups/3534timings/114allocationrows, retained0;
/tmp/tsrust-b19-profile.json. Actualpinned1.96.1 rustdocshlexfixture PASS2.120s,
166166528B, stopnull/remaining[], format57; /tmp/tsrust-b19-rustdoc-lease.json.
B17 cleanloggingtree3139162e audit309392PASS; privateprofile stillUNRUN.
Prepared /tmp/tsrust-b17-logging-supervisor.py (original36groups1116samples/
24allocationrows31after8), requires explicitexclusivelease. Publiclogging notstarted.
B11 SSE/WebSocket andremainingB20 nativeWindows/Linux/macOS/package/install/full
regressions remain mandatory. No partialbundle substitutes forfullB00–B20 goal.

## Storage / validation order

Quiet lockedcleanup this continuation reclaimed2,832,830,464actualbytes:
3obsoletevendorpacks2,270,629,888B;15obsoletegeneratedworkspaces562,200,576B.
Reports /tmp/tsrust-repair18-obsolete-source-packs-cleanup.json and
/tmp/tsrust-repair18-obsolete-workspaces-cleanup.json. Currentvendorpack preserved;
unknownworkspaceidentity preserved. Cacheprograms23current/unknown retained.
Including preceding1,672,269,824Bobjects +2,446,344,192Bobsoletecache cleanup,
recordedactualreclaimed total6,951,444,480B. This is removedoutput, notnetdiskgrowth.
Keepcurrentdebugcompiler/LSP/libraries/controlbinaries/source/chats/toolchains.
No cargo clean/refill and no deleting currentdebugsymbols blindly.

Repeatedbuildroot proven:18c145dirtyunits/95rebuiltartifacts vs18b9nonfresh.
InterveningportableCargo builds made shareddependencies/buildscripts newer;
historical.d alsoholddeletedvendorpaths. Executecompiler/editor/directchecks first;
portablevendorexports LAST, withno plannedcompilerrefresh afterward. Sameprofiles/
target retained; no newcacheframework. Preserve successfulscopeevidence.

ExistingSQLite acceptedmeasurements remain: B03sevenworkloads93samples;
B14/B18nineworkloads27enginegroups93samples; I/O48histories13206samples;
compression36histories10044samples. B18focused24native accepted, deepassertnative
remeasurepending; B15fourreadyunmeasured. NewHTTPuploads10/middleware10/B16crypto29
matchedworkloads ready, independentoraclesPASS, actualpublictimingsUNRUN. Private
profiles are NOT publicmeasurements. Measure acceptedfamilies, reuseunchangedRust/
Nodecontrols, ingest actualtimings with npm run store:ingest. Fullregressions only
at meaningfulfeaturefamilymilestones; preservepasses/resumeunrun afterrealrepairs.

Next: finish provenoptional/literal/nullable/UI source repairs, reviewB19draft,
formatwholeproject, then ONE coherentlyscoped normalbuild+affectedproof withfresh
budget andexportsatend. Do not restart the expired18scope or call itpassed.
Check currentPRs/Dependabot beforeanypush. OldCI49cancelled; neverdispatchobsolete
330mworkflow. Publication stillunapproved. Postv0.2 GPUI/Kit signals/.tsrx, .tsrSDK
andsharedbackend/frontend roots follow docs/plans/gpui-application-platform.md.
