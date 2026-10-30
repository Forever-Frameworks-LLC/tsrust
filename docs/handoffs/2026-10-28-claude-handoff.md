# Tsrust handoff to Claude — 2026-10-28

The owner requested this handoff and stopped Codex continuation. The goal is
unfinished. This document supersedes the next-step instructions at the top of
`docs/agent-progress.md`; its older entries remain historical evidence.

## Start here

Repository: `/Users/seancotter/Desktop/dev/learning/tsrust`.
Branch: `codex/language-cli-docs-polish`.
Last implementation/docs checkpoint before this handoff: `ec92fbc75e28ed21092ac647f4e60783300ec1be`.
The handoff itself is a separate documentation-only checkpoint. No push or release
was performed during cleanup.

Read `AGENTS.md`, `docs/development-workflow.md`, `docs/agent-goal.md`, then this
handoff. Use `docs/agent-progress.md` to locate detailed evidence, rather than
repeating its old continuation instructions. It is long and includes superseded
plans. Inspect current source/status before acting.

## Most recent owner correction — highest immediate priority

The owner said:

> im basically just seeing a lot of example how to documents appearing instead of
> actual documentation of the std lib and how the language works - it would be
> better to have an interactive guide of how tos than to just have a bunch of
> articles hogging up the search results

The current site over-fragments language topics into many small walkthroughs,
especially callable/generic features. Do not continue producing those articles.
Their verified facts/examples are reusable evidence, not an approved final docs
architecture. The initial `docs/plans/public-documentation-site.md` already proposed
broad language-reference topics; implementation drifted from that intent.

The requested direction is:

- Make language semantics and standard-library API reference the primary docs.
- Consolidate how-tos into an interactive guided learning experience; do not index
  every tutorial step as another competing reference article.
- Keep examples within reference entries where they explain an operation, without
  making examples the whole documentation model.
- Plan the revised structure in Markdown, then implement it incrementally. Preserve
  useful audited contracts and migrate links/anchors safely. No empty placeholders.
- Every page still needs factual and presentation review. Reorganization does not
  establish correctness, and a successful build does not establish a completed audit.

No revised navigation, consolidated reference, interactive guide or API-reference
generator has been implemented. Codex stopped after inspecting options. The
information-architecture skill was read, but no `.design` artifacts were created.

Useful implementation facts: Astro 7.4.2, Starlight 0.42.7; site configuration is
`website/astro.config.mjs`, content is `website/src/content/docs/`. Installed
Starlight schema supports frontmatter `pagefind: false`. Reuse its navigation,
styles and accessibility instead of adding another framework. Do not hide useful
reference content before its replacement exists. Preserve old URLs through the
migration, then remove superseded public source copies.

`crates/tsrust-lsp/src/builtins.json` has 103 groups / 894 entries, including
signatures, parameter documentation, behavior, ownership and examples. It is a
useful canonical source for a synchronized stdlib reference. Some group keys are
internal names; do not expose them as public types without mapping them. The
existing library overview lists 23 public module names, but detailed API reference
coverage is still incomplete. `std:path` (7 exports) and `std:env` (3 entries) were
only inspected as possible starting points; no generation work was done.

## Full goal and remaining work

Finish the finite backend-only language/CLI/docs/platform polish from PR #2. Do
not expand into native UI or arbitrary TypeScript compatibility. The original fan
stop was explicitly lifted: the owner is using a different computer.

- Async testing registration and HTTP body/SDK work have substantial accepted
  evidence in `docs/agent-goal.md` and the progress record; preserve that evidence.
- Shared dependency caches and paired CLI/LSP installation/update have implemented
  workflows and focused macOS/Linux/Windows evidence. Remaining distribution
  checks must be reconciled, not assumed complete.
- Windows and Linux installer/locked-binary checks passed previously. The newer
  isolated editor UTF-16 fixture and fresh toolchain smoke/acquisition paths still
  need their outstanding platform executions. Exact release-build commands and
  actual published acquisition remain unproven where marked pending.
- GitHub Pages was selected and configured, but a successful production deployment
  and public URLs remain pending. Local preview/build is not deployment evidence.
- Complete the finite remaining docs inventory in the migration/audit plans, using
  the corrected reference/learning architecture. Old pending/unsupported claims
  often contradict newer compiler behavior; check source before copying them.
- Retain completed family correctness/performance records. Do not restart full
  matrices or timing protocols after routine documentation changes.

A reported LSP retention issue was repaired and a bounded 30-minute macOS stress
check passed. The old blanket warning was removed. This is not proof of every
possible long-session behavior. HTTP performance evidence and original SQLite
records already exist; no new measurements were taken during recent docs edits.

The last retained nested-function guide still contains unaudited owned/scoped
selection, single-use call scopes, generic replacement and mutable factory arrays.
Its source-flow paragraph was investigated, but is not a mandate for another page.
Other language/library/operational inventory remains pending too.

## GitHub and release constraints

Original PR #2 was bootstrapped with owner approval. Last recorded draft delivery
PR is https://github.com/Forever-Frameworks-LLC/tsrust/pull/4, base `stag`, remote
branch `codex/backend-polish-staging`. Last recorded remote head was
`61ee67a44152437848a53d0fea6499e7ffec56ec`; its tree matched local `1b375f06`.
Later local work, including the recent docs commits, has not been pushed.
These are last-recorded values, not a fresh remote-state check. Revalidate before
any push. Do not force-push or trigger duplicate CI runs.

Owner-approved policy: branch-to-stag PRs run tests and require one review, with
owner review bypass; stag-to-main promotion compiles macOS/Linux/Windows artifacts,
then auto-merges/publishes if successful. New tags may be created by repository
writers; existing tags/releases must remain immutable. Prefer caching, minimal
permissions and dependable artifact reuse. Avoid unnecessary GitHub Actions usage.
No concrete new release version was selected; do not invent one or publish a new
release as part of this handoff.

## Validation and cleanup state

Recent accepted docs evidence is indexed in
`docs/plans/public-documentation-content-audit.md` and
`docs/plans/public-documentation-migration.md`. Their content-audit acceptance does
not mean the owner approved the fragmented article architecture.

The unfinished `language/callable-control-flow.md` article was deleted; its sidebar
addition and source-guide edits were reverted. No unfinished implementation remains.
Before deletion its two intentional rejection examples returned `arity`, and two
native examples printed `4 1` / `7` and `7 1`. Focused compiler flow checks passed
15 tests and editor diagnostics passed 4. Disposable logs, if still present:
`/tmp/tsrust-flow-doc-contracts.log` and `/tmp/tsrust-flow-doc-editor.log`. These are
optional current-run evidence, not durable analytics or required deliverables.

Cleanup rebuilt the restored 66-page site successfully (`npm --prefix website run
build`), removing the deleted article from generated output. The last full internal
link/anchor pass on that same committed site also passed. The agent-started docs
preview on port 4322 was stopped, its browser tab closed and viewport override reset.
No known compiler/test job is running. Restart preview if needed with
`npm --prefix website run dev -- --host 127.0.0.1 --port 4322`.

198 disposable native program cache entries (274 MiB) were pruned from
`target/tsrust-native/v2`; root lock files, shared native runtime cache and Cargo
target were preserved. Native programs rebuild when needed. Source, owner edits,
retained UI, fixtures and historical evidence were not removed.

## Working rules

One agent in this tree at a time. Inspect status first. Use the existing Cargo
target, `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`, and focused section-level checks.
Do not touch the owner's VS Code instance. Format the entire project before each
source checkpoint with `TSRUST="$PWD/target/debug/tsrust" node tools/format-project.mjs`.
Keep native behavior, diagnostics, IntelliSense and examples consistent when code
changes. No new heavy suite for prose alone.

No source snapshots/archive directories. SQLite remains canonical analytics;
never publish ignored `benchmarks/records.sqlite`. Public measurements must use
`benchmarks/storage/public.py` and its provenance allowlist. Preserve original
records and distinguish unmeasured work from measured work. Keep the handoff and
finite acceptance checklist current; do not claim completion from partial evidence.

## Claude progress — 2026-10-28

Architecture recorded in `docs/plans/documentation-architecture.md` (reference
first, one interactive Learn track per journey, chapter merge table and rules).

- Standard-library and built-in reference: `website/tools/generate-api-reference.mjs`
  renders all 103 catalog groups (minus frozen `std:ui` and the internal testing
  runtime) into 28 generated pages before every `dev`/`build`. Output is ignored
  by Git. The docs workflow now also triggers on `builtins.json` changes.
- First language chapter merged: `language/optional-values/` replaces
  borrowed-optional-fallback, optional-chaining, nullish-coalescing and
  nullish-assignment. Old routes redirect to chapter anchors. All ten examples ran
  with the current debug compiler with matching output; five rejection claims
  were checked with `tsrust check`.
- Link checker now validates redirect pages against their destination anchor
  (unit test added). Site build: 91 pages + redirects; link/anchor check passed.

Not done: remaining nine chapter merges, the first Learn track, folding library
guides and backend how-tos into tracks, and the CI/installation/platform/deployment
reconciliation listed above. Nothing was pushed; no GitHub run was triggered.

## Distribution, CI and deployment reconciliation — 2026-10-28 (rows updated 2026-10-29)

Checked against workflow source and read-only `gh` queries; no run was triggered.

| Requirement | Current state | Evidence |
| --- | --- | --- |
| Release artifacts for macOS, Linux, Windows | All four targets built and smoke-tested; `v0.1.0-rc.1` published as a prerelease (updated 2026-10-29). | Release run 40861023720, Promote run 40864132322 |
| Windows paired install, locked binaries, shared caches | Passed on hosted Windows. An earlier failure (40724567590, 90382caa) was fixed by 9b658857. | Runs 40731485167 (9b658857), 40748893795 (86435b88) |
| Async testing on three platforms | Passed. | Run 40672007920 (c5b3d4a9) |
| PR CI on the staging branch | Linux compiler/LSP, VS Code extension and docs build passed for PR head 61ee67a4. | Runs 40761578381, 40761578389 |
| macOS | Local focused acceptance only (see `docs/agent-goal.md`). | Local evidence |
| GitHub Pages | Deployed at https://forever-frameworks-llc.github.io/tsrust/ (updated 2026-10-29). | Documentation run 40864208241 |
| Install/self-update from a published release | rc.1 archive installed by hand on macOS arm64 (version, doctor, native run). Managed install and self-update need the first stable release: pending `v0.1.0`. | See the release-candidate status section |
| Windows editor UTF-16 fixture | Passed on Linux and Windows (updated 2026-10-29). | Editor acceptance run 40816862762 |

Remote branch `codex/backend-polish-staging` (PR #4, draft, base `stag`) is still at
61ee67a4. Local history is not a descendant of it (the PR branch was rebuilt
from the same trees), so compare trees before any push. All local commits since
then, including this documentation restructure, are unpushed.

## Release candidate status — 2026-10-29

- Editor acceptance passed on Linux and Windows (CI run 40816862762), including
  the surrogate-pair UTF-16 diagnostic fixture.
- The five package-export tests pass locally once `tsrust setup --portable
  --offline --yes` has prepared source packs.
- Owner chose a pre-release. Version is `0.1.0-rc.1`. The toolchain installer
  now accepts `MAJOR.MINOR.PATCH-rc.N` (ordered before the stable version);
  update discovery still considers only stable releases, so self-update from a
  published release still needs a stable release to verify.
- PR #4 (`codex/backend-polish-staging` → `stag`) was merged by the owner with
  the review bypass (squash commit cf4e729c). Authors cannot approve their own
  PRs; the bypass is the owner's path for agent-opened PRs.
- PR #5 (`stag` → `main`) built all four targets and the extension (Release run
  40861023720). Promote (run 40864132322) merged it and published the
  `v0.1.0-rc.1` prerelease with four archives, `tsrust-0.2.0.vsix` and
  `SHA256SUMS`. The Documentation workflow (run 40864208241) deployed
  https://forever-frameworks-llc.github.io/tsrust/.
- Release smoke test on macOS arm64: the archive matched `SHA256SUMS`;
  `tsrust --version` printed `0.1.0-rc.1`, `tsrust-lsp --version` printed
  `0.2.0`; `doctor` and `doctor --native` passed; `tsrust run` executed a native
  program correctly. `tsrust toolchain install --root` refused as designed: "No
  published stable official toolchain is available yet".
- Pages smoke test: the home page, Learn, Language, globals guide, generated
  globals and `std:sql` reference returned 200; the old
  `reference/builtins/globals/` path redirects to `reference/globals/functions/`.
- Still open: managed install and self-update need the first stable release.
  Linux, Windows and Intel macOS archives were built and checked in CI, not
  installed by hand. This handoff commit is local only and reaches `stag` with
  the next PR.

## 0.1.0 milestone — 2026-10-29

Owner direction: no rc.2; finish the work, then publish `0.1.0`; run the
performance milestone and put the measurements in the docs.

- **Legacy UI out of the release CLI.** `tsrust ui`, `new --ui`, `dev --target
  web` and the UI menu are behind the `legacy-ui` Cargo feature (off in release
  builds). The bundled component catalog and `shadcn-MIT.txt` are not in the
  archive. Compiler `std:ui` support and the `tsrust-ui` crate remain, frozen.
  UI CLI tests run only with `--features legacy-ui`.
- **Docs inventory finished.** New chapters `language/operators` and
  `language/errors`; numbers, enums and recursive records in `language/types`;
  `for...in` and labels in `language/control-flow`; diagnostic suppression in
  `tools/cli`. 24 retained guides merged or deleted; research and developer
  guides labeled. See `docs/plans/documentation-architecture.md`.
- **Defects fixed on the way** (each with a regression test): formatter spacing
  for `c ? (): T => a : b`; misleading diagnostics for non-boolean `||=` on an
  array element and for rebinding a borrowed-view `let`; `setup --portable`
  failing on a damaged cached pack (now rebuilt and replaced); benchmark
  harness hiding prepared source packs behind its isolated Cargo home; three
  benchmark policies rejecting formatting-only source changes (now use
  registered token-identical proofs).
- **Performance milestone.** All 930 workloads measured native vs Node after
  tsc (3 × 31 samples, 175,620 samples), 880 profiled. Native faster on 613
  (median 1.52×). Median native change vs earlier measurements 1.00×; no
  regression beyond noise. Evidence: `benchmarks/storage/release-0.1.0-analytics/`.
  The public page `project/performance` is generated from
  `benchmarks/measurements.sqlite` at docs build time; publish a different run by
  editing `website/tools/performance-runs.json`.
- **Optimization targets** (native slower than Node): JSON/schema decoding,
  record/object copies, Unicode case conversion, `String.at`, recursive array
  copies, enum and interface records.
- **Known pre-existing issues, not fixed:** `store.py verify` rejects old
  absolute-path log records already in `records.sqlite`; about 55 benchmark
  harness unit tests are stale (hard-coded `target/editor/debug/tsrust`, frozen
  manifest-prefix hashes broken by fingerprint refreshes). Neither affects
  measurement.

## v0.1.0 published — 2026-10-29

- PR #6 (owner bypass merge) into `stag`; PR #7 `stag` → `main` built all four
  targets and the extension (Release run 40930352738); Promote (run
  40934384351) merged it and published `v0.1.0` as a regular release; the
  Documentation workflow redeployed Pages (performance page live).
- Archive check (macOS arm64): checksum OK; attributions hold only the Tsrust
  license and dependency notices (no UI notice); help has no `tsrust ui`;
  `tsrust 0.1.0` / `tsrust-lsp 0.2.0`; `doctor` OK.
- Managed install against the published release: `toolchain install --root`
  selected 0.1.0 and its LSP; `check` reports no newer release; `update` changes
  nothing; `rollback` refuses (no previous version); `doctor` reports a managed,
  intact pair. A console project runs and tests, and a backend project checks.
- Still unverified: updating across two real releases (needs a second release),
  and hand installation on Linux, Windows and Intel macOS (CI smoke only).


Baseline: the 0.1.0 run in `benchmarks/measurements.sqlite`. Every "after" is a
quiet-machine run (runner load guard, limit = half the logical CPUs).

| Change | Result | Kept |
| --- | --- | --- |
| Runner waits for load < CPUs/2 before each timed process; records max load | Desktop apps alone keep load near 3–4 on this M3; a limit of 2 never starts | Yes |
| `String(stringEnum)` borrows the static member text; whole numbers below 2^53 format as integers | `enum_values` 6.65M → 17.3M ops/s (2.6×; 0.14× → 0.35× of Node); other number workloads within noise | Yes (cc2dc382) |
| Left-nested string `+` appends to the fresh left temporary (exact `reserve_exact` growth) | `string_add_chained_*` 23.2M → 22.8M, `string_concatenation`/`interfaces` unchanged: allocation, not intermediate copying, dominates short strings; V8 uses ropes | No (reverted) |
| Dead `const` array receivers of `concat()`/`toReversed()` are moved (single reference, same statement list, no closures) | concat-zero workloads 1.94–3.01× (record 0.43× → 1.04× of Node); moved `toReversed` 1.27–1.57×; others within noise | Yes |
| Receiver moves extended to `slice()` (plain `Vec` only) and `toSorted()`; closures only block locals they capture | Tested (75 suites); no benchmark workload has a dead `slice()`/`toSorted()` receiver, so unmeasured | Yes |
| String search: first-byte scan for haystacks ≤ 64 bytes (`indexOf`, `lastIndexOf`, `includes`) | `indexOf` 2.45–2.72×, `lastIndexOf` 1.05–1.74×, `includes` 1.26–1.32×, JSON string projection 12.3× (0.33× → 4.1× of Node). A plain byte-window scan was tried first: 0.43–0.63× on `includes`/default `lastIndexOf` (slice compare at every position), so it was replaced | Yes |
| mimalloc as global allocator (owner approved trying it; then paused) | Schema pipeline 1.85× with mimalloc, `record_mutation` 0.91×. Profiles showed time inside macOS `free()`, but this Mac has 8 GB RAM with ~7.4 GB swap in use and ~3 GB compressed, which inflates allocator cost. Not adopted: a dependency in every program needs evidence from a clean machine. Dependency changes reverted; the harness keeps an experimental `--allocator-rlib` option | No |
| Dynamic `JSON.parse` re-scans integer tokens only when serde produced a float (only an integer too large for i64/u64 arrives that way) | JSON workloads up to 1.32× (`string_reads`, `refinement`, `typeof`, `scalars`); 54 JSON/schema workloads median 1.00×; `native_json_raw` 0.84× re-measured alone at 0.99× and does not use the parser | Yes |
| Typed JSON decode: validation pass records floats; integer re-scan only after one | 54 JSON/schema workloads median 1.04× (typed JSONC 1.18×, roundtrip 1.15×, object strip 1.15×); lowest 0.97× (noise). The validation pass itself stays: it keeps malformed input from reaching user callbacks | Yes |
| `JSON.parse` of a string skips UTF-8 re-validation (`parse_valid_utf8`); JSONC and byte inputs keep the check | vs the previous run: 26 JSON workloads median 1.01×, `typeof` 1.29×, scalars 1.18×; lowest 0.98× (noise). Inputs are tiny, so larger documents gain more | Yes |
| `JSON.parse`/`tryParse`/`std:json` `parse` without limits = plain serde_json (owner direction: TypeScript-familiar API, Rust accuracy). No default limits (HTTP `bodyLimit` and serde's 128-level nesting remain); over-range integers still reject; Rust number formatting kept (a JavaScript-style formatter was tried and reverted as a JS quirk). Also fixed a CLI panic when a diagnostic points past the user file | Speed roughly neutral: median 0.97× vs the previous run, 1.05× vs 0.1.0; drops of 0.82–0.84× are in workloads that do not call `JSON.parse`, so run-to-run noise here is about ±15%. Generated/hand-written for `int_bounds` 0.39× → 0.43×. First version regressed `typeof` to 0.33× by allocating in the float check; fixed | Yes |
| Parity engine `rust-hand` (hand-written Rust in `benchmarks/runtime/parity/`) | `native_schema_int_bounds`: generated 0.34× → 0.40× of hand-written. Per iteration: generated 2 allocations vs 1; the rest is parse overhead (UTF-8 re-check, budgeted seed, typed decode parses twice) and issue conversion | Tooling |
| `dynamic_unicode_uppercase` investigated | Node reaches ~3 ns per 20-char Unicode `toUpperCase`, i.e. it reuses the loop-invariant result; native does the real conversion (~75 ns, std ASCII fast path already). Benchmark-design artifact, not a conversion gap | n/a |

Measurement caveat: every local number, including the 0.1.0 baseline, comes from a memory-starved Mac (8 GB RAM, heavy swap). Code-level wins (fewer copies, faster search) reduce work on any machine, but magnitudes need confirming on a clean runner before publishing or adopting dependencies.
| Typed `JSON.parse<T>`/`tryParse<T>` without limits: one serde decode plus the integer-token scan (`typed::decode_plain`), no validation pass. Explicit limits, JSONC and JSONL keep the budgeted path; schema decoding is unchanged | Median duration vs the previous run: `native_json_typed` 1.38×, `native_json_literal_union` 1.35×; Node.js controls 1.01× (comparable conditions). Native ÷ Node.js: typed 0.71× → 0.96×, literal union 0.64× → 0.85×. Load ≤ 4 throughout | Yes |
| Schema rejection path (`native_schema_int_bounds`): pooled the intermediate `Issues` buffer to drop one allocation and free per rejection | No wall-clock change at 40M iterations (3.23–3.58 s vs 3.19–3.37 s): the thread-local pool costs about what the `free` did. A profile shows ~85% of the loop is the rejection path (issue buffer, `error.issues` array, record conversion); the array allocation is required by the API. Generated 79–85 ns/iteration vs hand-written 34 ns. Not adopted | No |
| `text + String(n)` (int, number or numeric enum) writes the digits into the concatenation buffer instead of formatting a temporary string; template interpolations of int and whole numbers append digits directly (`tsr_push_int`, `tsr_push_whole_number`; other numbers keep their existing spelling) | 18 affected workloads, stashed before vs after, back to back, load ≤ 3.9: median 1.07×, none slower; `enum_values` 1.62× (0.33× → 0.54× of Node.js), `array_from_map_string` 1.54× (0.51× → 0.79×), `interfaces` 1.25×, `function_expressions` 1.23× (0.73× → 0.92×). Node.js controls 0.87–1.06× | Yes |
| String-addition chains build one buffer (`6f474114`); int/number/boolean chains emit one running value (`d3f85d41`) | 27 workloads, `3790b8e0` vs `423a8dbd` back to back, load ≤ 4: median 1.04×, none slower; `interfaces` 1.84× (0.21× → 0.39× of Node.js), `map_iteration` 1.76× (0.30× → 0.52×), `string_iteration` 1.41×, `map_for_each` 1.28×, `object_values_entries` 1.23×. Arithmetic and boolean chains 1.00–1.07× (LLVM already folded the nested form; the flat form exists for depth, not speed). Node.js controls 0.95–1.13× | Yes |
| `String(x)` of a dead `const` string local moves it instead of cloning (`92600e26`; receiver-move rule) | 8 record workloads, `7d7b97d6` vs `92600e26`, load ≤ 3.9: `type_aliases` 1.13× (0.19× → 0.23× of Node.js); the other seven do not use the pattern and stay within noise (0.94–1.01×, while Node.js controls drifted 0.88–0.96×, so the second half ran on a slightly slower machine) | Yes |
| In-place `text += value` for uncaptured locals (`4855dbee`, `tsr_string_append`: borrowed → one exact allocation, owned → `push_str`) | Long accumulation (20,000 appends of 3 bytes, ×20): 0.02 s vs 0.20 s CPU for the copying form (10×, and the gap grows with length: linear vs quadratic). Short single appends on the 14 `+=` workloads, `28f66cdb` vs `4855dbee`: 0.88–1.02× (`compound_string_add_ascii` 0.94×, `declaration_lists` 0.94×), within noise or a few percent from amortized capacity retained in output arrays, the same as hand-written `push_str` | Yes (kept for the algorithmic win; parity with Rust `push_str`) |
| `text += a + b` (and longer right-side chains) appends each operand in place after one reserve (`500c09a2`, `tsr_string_reserve`) | 15 workloads, `d6fae277` vs `500c09a2`, load ≤ 4: `labeled_control` 2.37× (0.30× → 0.71× of Node.js), `for_pattern_initializers` 1.19× (0.19× → 0.25×), `in_operator` 1.09×; others 0.94–1.04× (the `compound_string_add_*` single appends are unaffected by this change). Node.js controls 0.91–1.08× | Yes |
| Appended `name.slice(...)` of a local borrows its range (`0800c571`, `tsr_slice_str`); new hand-written parity program `parity/declaration_lists.rs` | `1c652680` vs `0800c571`, load ≤ 3.9: `declaration_lists` 1.24× (0.18× → 0.22× of Node.js; generated/hand-written 0.38× → 0.46×); five other slice workloads 0.98–1.03× (no appended slices). Hand-written Rust itself is 0.53× of Node.js on this workload: V8 shares strings, so `slice()`/`repeat(1)` are free there. Remaining generated overhead: the `words` array allocated only to be destructured, `repeat(1)` and `second.slice()` copies of borrowed elements | Yes |
| Map/Set keys hashed once with a per-process random key; bucket tables use that hash directly (`27bb63d4`) | 11 workloads, `a3959610` vs `27bb63d4`, load ≤ 4: `map_values` 1.23× (0.39× → 0.48× of Node.js), `set_values` 1.14× (0.40× → 0.49×), `set_for_each` 1.13×, `map_for_each` 1.11×, `native_schema_collections` 1.06×; `map_iteration` 1.12× matched by its Node.js control (noise); unrelated controls 0.98–1.04× | Yes |
| Maps and Sets with at most 8 slots scan their slots instead of hashing; the 9th slot builds the index once (`f43235a6`) | 13 collection workloads, `647bd09f` vs `f43235a6`, load ≤ 3.9: 1.17–1.60×, none slower. `set_algebra` 1.60× (0.54× → 0.88× of Node.js), `map_values` 1.57× (0.45× → 0.73×), `set_for_each` 1.47×, `set_relations` 1.44× (0.73× → 1.03×), `set_entries` 1.40× (0.99× → 1.43×), `map_for_each` 1.35×, `array_from_collections` 1.27×, `map_iteration` 1.22×, `map_group_by` 1.21×, `map_snapshots` 1.17×, `native_schema_collections` 1.17×. Node.js controls 0.93–1.11× | Yes |
| `String(boolean)` borrows "true"/"false"; `Object.hasOwn` recognizes canonical index keys without formatting the index (`9ca1e25b`) | 10 workloads, `3aad90d0` vs `9ca1e25b`, load ≤ 4: `object_has_own` 3.16× (0.31× → 1.02× of Node.js), `array_splice_boolean` 2.13× (0.59× → 1.32×), `map_values` 1.20× (0.74× → 0.87×); others 0.99–1.03×. `array_splice_int` first read 0.86× but re-timed alone at 0.98× (noise; it does not use either path) | Yes |
| One-character ASCII strings borrow a static 128-byte table (`charAt`, `split("")`, `for…of` text, spreads, `Set(text)`); `text.at(i)` selects one character instead of `text.split("").at(i)` (`c35e14da`) | 8 workloads, `85912b00` vs `c35e14da`, load ≤ 4: `string_at_characters` 30.1× (0.13× → 3.71× of Node.js), `array_from_string_characters` 2.12× (0.31× → 0.65×), `string_iteration` 1.71× (0.25× → 0.44×), `string_split_empty_delimiter` 1.13×; controls 0.98–1.01×, Node.js controls 0.98–1.02× | Yes |
| Small-string optimization: runtime `String` as `Borrowed`/`Owned`/`Inline(≤15 bytes)` (24 bytes, like `Cow`), inline clones/slices/padding/`+=`, chains and templates through a sink (branch `codex/small-string-experiment`, `53c111b2`…) | 20 workloads vs `85912b00`: `string_at_characters` 2.03×, `array_from_string_characters` 1.57×, splits 1.03–1.20×, but `string_add_chained_*` 0.53–0.56×, `map_iteration` 0.75×, `enum_values` 0.81×, `interfaces` 0.85×, `string_iteration`/`string_concatenation` 0.87–0.88×. Generated code may not contain `unsafe`, so inline text is re-validated as UTF-8 whenever it is read as `&str`; standalone, the chain is 0.66 ms vs 0.39 ms and an unchecked variant still only reached ~0.55×. The character wins were taken by the static-table change above instead | No (branch kept) |
| Schema collections: duplicate keys of ≤8 entries compared pairwise without a HashMap; leaves whose int/number/boolean output equals the input restore through the capture-free identity closure (`283f61cb`) | 12 schema/JSON workloads, `c35e14da` vs `283f61cb`: `native_schema_collections` 1.26× (0.25× → 0.33× of Node.js; 59 → 48 allocations per iteration, `map.safeParse` of one entry 12 → 9). Others 0.90–1.03× with Node.js controls 0.92–1.05×; `native_json_containers`/`native_schema_json_array` re-timed alone at 1.01×/1.00×. Remaining schema allocations are structural: each trial layer (sequence, pair, remap, inject) boxes a `dyn FnOnce` restoration capturing the previous one, and entry/value Vecs are rebuilt between layers; generic restorations are the next fix | Yes |
| `String(int)` formats with `i64::to_string` instead of building a `TsrConsoleValue` and calling `label()` (`2f29f4e3`) | 8 workloads vs `85912b00`: `array_splice_int` 1.18× (0.56× → 0.66× of Node.js), `type_aliases` 1.15×, `labeled_control` 1.10×, `declaration_lists` 1.09× (gen/hand 0.46 → ~0.50), `interfaces` 1.04×, `enum_values` 1.00×; `string_concatenation` 1.07× and `map_iteration` 1.03× had Node.js controls at 0.81×/0.93× so are not conclusive | Yes |
| `String(number)` tries whole-number digits before the number formatter (tried as `cce967c1`, dropped) | 6 workloads vs `8afa1e58`: `array_pop_number` 0.93×, `array_shift_number`/`nullable_number_null` 0.96×, others 0.98–1.01× (Node.js controls 1.00–1.03×). Whole numbers allocate once either way and mixed inputs (0.5, NaN, ±Infinity) pay for the extra check | No |

Allocation profile (2026-10-30, `profile-native.mjs`): `native_schema_recursive`
47 allocations per iteration, `native_schema_export` 25, `native_schema_pipeline`
13, `interfaces` 8, `native_schema_string` 3, `enum_values` 2. Schema paths
build a dynamic `JSON.parse` tree and then copy it into typed output; fusing
`schema.safeParse(JSON.parse(text))` into one typed decode must keep the throw
for malformed JSON. Records now allocate close to hand-written Rust; the rest of
the gap to Node.js is short strings, which V8 keeps inline. A safe
small-string representation was tried on 2026-10-30 and rejected (see the
optimization log): without `unsafe`, reading inline text re-validates UTF-8 and
concatenation-heavy workloads lost up to 47%. Remaining short-string wins should
come from borrowing (static tables, slices of stable receivers) and avoiding
intermediate strings, as the one-character table did.

Schema text decoding (2026-10-30): `schema.safeParseJson(text)` measured about
1.25× slower than `schema.safeParse(JSON.parse(text))` (300,000 nested-object
decodes: 0.27 s vs 0.22 s CPU), so fusing the latter into the former would be a
regression. `source::document` validates the whole document, then each cursor
re-parses its subtree with serde when a field is read. Skipping the integer
re-scan without floats changed nothing measurable. The fix is a single-pass
schema `DeserializeSeed` decoder (as typed `JSON.parse<T>` already uses), which
replaces the cursor emission in `stdlib/schema`.

Next targets: add more `parity/` programs; add workloads with dead `slice()`/`toSorted()` receivers; `with()`/`toSpliced()` moves (need element assignment across array wrappers); JSON/schema decoding (group median 0.78×); record construction (records group 0.51×); `String.at`.

## Measurement store — 2026-10-30

Owner direction: canonical records are not kept. `benchmarks/measurements.sqlite`
is one table, `measurements(workload, engine, history)`, with a JSON history of
`{at, ms, ops, samples}` per run (commit `chore(benchmarks): one row per measured
workload…`). After timing, run `npm run store:ingest` in `benchmarks/runtime`;
`npm run checkpoint` prints the latest native/Node.js ratios. The ignored
`benchmarks/records.sqlite` (596 MB) was deleted, and the old analytics/export
scripts were removed. Raw files in `benchmarks/runtime/results` (ignored, ~0.8 GB)
are disposable once ingested; `BENCHMARKS.md` is still generated from them.

## Nesting limits removed — 2026-10-30

Owner direction (2026-10-29): remove limits like the 128-level nesting limit.

- `d3f85d41`: chains of `+ - * / % && ||` are flat in the checker (shared type
  hints during descent, linear time) and in emitted Rust (one buffer or one
  running value); generated programs allow `unused_parens`.
- Discovery (`benchmarks/compiler/nesting-discovery/run.py --unbounded-parser
  --breadth-first --start 256 --factor 4`, 60 s/case) over all 80 families found
  no crash. With the parser bound lifted, families complete to 4,096–1,048,576
  levels before the case budget. Evidence JSON is session scratch, not imported.
- rustc's own stack was the remaining ceiling (SIGBUS near 1,000 nested
  blocks). Tsrust now runs rustc/cargo with `RUST_MIN_STACK=1073741824` unless
  the user set it (virtual reservation); 16,384 nested blocks then compile.
  Exported packages document the variable instead of setting Cargo `[env]`,
  which would also change thread stacks under `cargo run`.
- `NestingPolicy::CONSUMERS` is unbounded: expression, block, conditional,
  match, template, type annotation, JSX and destructuring nesting have no limit.
- The CLI and the LSP analysis worker run the compiler on a
  `COMPILER_STACK_BYTES` (1 GiB, virtual) thread, so passes that still recurse
  (for example `in`, checked by synchronous `object_has_own`) reach tens of
  thousands of levels. Tests use `support::check_deep`/`accept_deep`.
- Owner direction (2026-10-30): "match the rust defaults, if rust doesnt have
  limits then we shouldnt either". rustc has no count limit on
  monomorphizations, type parameters, type arguments or record nesting; its
  only default is `recursion_limit = 128`, which bounds polymorphic
  instantiation depth and nested-type trait solving. Removed: callback origin
  specialization counts, per-target callback instance counts, generic
  specialization counts (128/declaration, 1,024/program), closure environment
  complexity, direct record layout depth, explicit type argument and type
  parameter counts. Inline record and callable storage now fail only past
  Rust's maximum object size (`isize::MAX`). Generic instantiation instead
  fails when an argument nests deeper than 128 (`INSTANTIATION_DEPTH_LIMIT`),
  which is how polymorphic recursion (`f<T>` calling `f<Box<T>>`) still gets a
  diagnostic, as in rustc. Callback specializations get the same bound:
  closures inherit the instantiation depth of the body that creates them
  (`specialization_depths`/`closure_depths`), so wrapping the callback in a
  new closure on each recursive call fails at depth 128 instead of growing
  without end (it reached ~30 GB once before the guard existed). Kept,
  matching `recursion_limit`: array rank 128, callable result discovery
  depth 128. Commit `0be17d66`.
- Measured follow-up: `scoped-map-catch` check time is cubic in depth (debug
  compiler: 32 → 0.42 s, 64 → 2.9 s, 80 → 5.5 s) while plain `catch-regions`
  and `cleanup-regions` stay at 0.02 s, so the cost is callback-holding Map
  storage across catch regions in analysis (`map_exception_storage_types`,
  `scoped_map_carriers`, `map_storage_edges` are the candidates). Profiling is
  profiled (Developer Mode enabled 2026-10-30): `finish_block` →
  `record_sequence_carriers` runs every carrier pass over the block's whole
  subtree each time any block finishes, and `scoped_map_entry_epochs` then
  filters every function binding through `contains_scoped_map_storage` for
  each statement (`native_inputs`). Work is blocks × statements × bindings.
  A scalar fast path changed nothing (the bindings are Map/record types). The
  fix is incremental: process each statement's epochs once instead of
  re-walking nested blocks from every enclosing block.
- Known failures before 0.1.0 (also in full run 40909244877, frozen UI):
  `retained_ui_catalog_entries_have_current_compiler_witnesses` and
  `retained_ui_catalog_examples_match_native_output` fail because the
  `UiComputed`/`UiSignal` catalog entries have no witness arm and no in-text
  example. Left as is under the backend-only reset; fix only if UI revives.
- Follow-up (performance, not limits): `scoped-map-catch`, `-cleanup`,
  `-catch-cleanup` time out at 256 levels and `scoped-map-mixed` at 1,024 in
  ownership analysis; `builtin-string-chains`, `captured-local-functions`,
  `field-*-cleanup`, `fresh-generic-adapters` time out at 4,096.
