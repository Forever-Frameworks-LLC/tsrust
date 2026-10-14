# Editor agreement with the compiler: handoff to integration

Scope: `crates/tsrust-lsp/**` and its tests. Branch
`codex/claude-lsp-agreement`, based on `06deeecf`. Worktree
`/Users/seancotter/Desktop/dev/learning/tsrust-claude-lsp`. The compiler crate
and `editors/vscode` are unchanged. Installed and editor binaries were not
replaced and VS Code was not restarted. Not merged.

## Commits

| Order | Commit | Content |
| --- | --- | --- |
| 1 | `6721c75b` | Request routing, cursor repair, one member list, new completion positions, typing harness and three guard suites |
| 2 | this commit | Workflow rule and this handoff |

## What was wrong

**Stale routing (confirmed).** `main.rs` answered with
`projects.request(...).or_else(|| documents.index(uri)...)`. A project index
is used only while its input snapshot is current. After each keystroke it is
not current until the 200 ms debounce and the analysis finish. In that window
the request fell back to the single-document index, which cannot resolve
imports, and returned an empty list. The baseline matrix shows it: every
imported name and member is empty when typed without a pause and when typed
at 80 ms per key, and correct when settled.

**No facts for the text being typed.** `value.` makes the document invalid,
so the checker produces nothing. The server had two probes for this, one for
a member at the end of the file and one for a member that is a whole
statement followed by `}` or `;`. Other positions were answered from the
editor's own inference, or not at all.

**The editor contradicted the checker.** Completion offered nothing on a
`Result`, and the hover text said "Match before accessing either payload".
The checker accepts `result.payload`. An agent that investigated this report
read the silence as a language rule and wrote it down as fact.

## What changed

**Routing.** A request for a document with imports waits for the project
analysis of its current text. The analysis starts at once. The request is
answered when that analysis arrives, when no analysis is queued or running,
or after three seconds; a late answer is marked `isIncomplete` and counted.
`$/cancelRequest` releases a waiting request. Hover, signature help,
definition and references use the same path.

**Cursor repair** (`cursor_repair.rs`). When the document does not check and
its own index offers nothing at a member position, the server removes the
unfinished access at the cursor, closes the brackets that the unfinished code
left open, and checks the repaired text: with `tsrust_compiler::check` for a
single document, with `check_graph` on a copy of the module graph for a
project. A receiver that is a whole statement is read through `typeof`, as
the earlier probes did, so the checker reports its narrowed view. The
receiver type and its members come from that check.

**Members** (`member_items`). One function builds member completions for
both the direct and the repaired path. After a member separator the server
offers members or nothing.

- `Result<T, E>`: `Ok` and `Err` each store one `payload`, and the checker
  reads a field that every alternative has with the union of its types. The
  editor offers `payload` with that type when neither alternative is `void`,
  and a `match` snippet that replaces the whole access. Hover on the Result
  and on `payload` states the field.
- A native record that another native record mentions is declared from the
  compiler's `native_types` declarations, so `error.issues[0].` offers the
  `SchemaIssue` fields without an import of that name.

**New positions** (`position_completion.rs`): import paths (standard modules
and the `.tsr` files beside the document), fields of a record literal after
an annotated declaration, and the missing arms of a `match` on a Result.

## Rows from the report

Measured with `completion_matrix` in four modes: typed without a pause, typed
at 80 ms per key, settled, settled with a syntax error elsewhere.

| Position | `06deeecf` | `6721c75b` |
| --- | --- | --- |
| Imported value, imported member, import names of a project module, at typing speed | empty | same as settled |
| `parsed.` on a Result | empty | `payload`, `match` |
| `console.log(parsed.pa` | 54 scope names | `payload` |
| `parsed.payload` hover | none | `(property) payload: string \| ValidationError` |
| `parsed.payload.` | empty | empty; the union has no common member (declared silent) |
| `error.issues[0].` | empty | six issue fields |
| Nullable narrowed by `if`, then `maybe.` | empty | record fields |
| Callback parameter in an open closure | empty | members |
| `const other: Item = { ` | scope names | `id`, `name` |
| `function take(a: It` | scope names included | `Item` |
| `import { x } from '` | empty | standard and project modules |
| `match (parsed) { ` | scope names | `Ok`, `Err` arms |
| `catch (error) { error.` | empty | empty; a caught value has no declared type (declared silent) |

Baseline: 46 failing observations of 144, in 14 rows. Final: none of 144.

## Guard systems

- `tests/support/session.rs`: a real server over stdio. It types one
  full-document change per character and requests at once.
  `TSRUST_LSP_SERVER` runs the same suites against another build.
- `tests/completion_matrix.rs`: one row per receiver kind and position, in
  four modes that must agree. It fails when a kind has no row, when a row has
  no expectation, and when import-path completion lacks a module of the
  compiler's binding table.
- `tests/checker_agreement.rs`: for 15 receiver types, completion fields
  equal the fields the checker accepts, each field shows the checker's type,
  and no offered method is one the checker rejects. Run against the installed
  server it fails on exactly `Result<string, int>.payload` and
  `Result<Item, string>.payload`.
- `tests/editor_consistency.rs`: completion is not empty where hover reports
  a type; every documented member of seven built-in families is reachable;
  typing a project document leaves the counters at zero.
- `src/counters.rs` and the request `tsrust/counters`: `staleAnswers`,
  `emptyKnownReceiver`, `nonMemberAfterSeparator`. Each increment is also
  written to the server log.
- `docs/development-workflow.md`, section "Editor agreement with the
  compiler".

## Validation at `6721c75b`

Private target `/private/tmp/tsrust-claude-lsp-target`, `CARGO_INCREMENTAL=0`,
`CARGO_BUILD_JOBS=1`, serial tests, no `RUST_MIN_STACK`, unoptimized build.

- All 49 `tsrust-lsp` test targets and both unit targets were run. 50 pass.
  One fails: `optional_main::invalid_return_in_dependency_does_not_mask_independent_entry_error`.
  It fails the same way on unmodified `06deeecf`. The test calls
  `check_with_diagnostics` directly and expects two diagnostics; the compiler
  returns one. No editor code is involved.
- One existing assertion changed:
  `project_intelligence::imported_result_matches_keep_nominal_payload_help_and_pattern_navigation`
  required that the Result hover contain no ` | `. The hover now names the
  payload type. The assertion checks the signature for that, and requires the
  payload sentence.
- Answer time for a request sent right after the last keystroke in a
  two-module project: 60 to 144 ms. Before, 1 to 30 ms for an empty answer.
  These are single observations, not timing measurements.
- `node tools/format-project.mjs`: 0 files formatted, 0 fingerprints updated.
- Not run: compiler suites (no compiler change), the VS Code extension tests,
  any check in a running editor.

## Requests to the compiler owner

The editor still infers types in `intelligence.rs` and reads members from
`builtins.json`. Those are checked against the compiler by tests, not
replaced. Three queries would remove the remaining gaps.

1. **Members of a type.** `fn members(&self, ty: Type) -> Vec<Member>` on a
   checked program, with `name`, `kind` (field or method) and the checked
   type or signature. Needed because method agreement is one-directional
   today: the checker validates argument count before the receiver
   (`x.map()` on an `int` answers `Expected 1 arguments, found 0`), so a call
   without arguments cannot prove that a method exists.
2. **Type of an expression at a span**, for names, field reads and index
   reads, in canonical spelling. `CheckedTypeFacts` covers calls, narrowed
   views and parameters. The payload type shows why spelling matters: the
   checker orders union alternatives by an internal key (`int | string` for
   both `Result<int, string>` and `Result<string, int>`), which the editor
   cannot reproduce. The editor shows `T | E`; the agreement test compares
   alternatives as a set.
3. **The list of standard modules.** `native_exports::known_module` answers
   one name and omits `std:testing`. The editor confirms each documented
   module once by loading the headers of a probe module.

## Findings outside the editor

- `optional_main` fails on `06deeecf` (above).
- A type diagnostic spells a native type by its linked name:
  `Type "__tsrust_module_native#std:schema#SchemaIssue[]" has no field …`.
- `Result.payload` follows from the common-field rule for record unions, but
  `docs/result-match.md` does not mention it and no test reads it. With a
  `void` alternative the field does not exist. `console.log(result)` prints
  `{ payload: … }` for both alternatives with no tag. Whether the field is a
  supported surface is the owner's decision. If it is removed, the agreement
  test fails until the editor stops offering it.

## Limits and deferred work

- With a syntax error elsewhere in the file, nothing checks, so narrowed
  unions and callback parameters in an open closure offer nothing. They never
  offer a wrong name. The rows are marked `needs_check`.
- A caught value has no declared type; `error.` after `catch` stays silent.
- Record literal fields are offered only after an annotated declaration, not
  for arguments or returns. Match arms are offered only for a Result.
- Repair costs up to two checks per member completion in a document that
  does not check. A large project will show that as latency; the three-second
  bound applies to the wait for analysis, not to the repair.
- Hover is not repaired: hover on an unfinished access can still be empty.
- `emptyKnownReceiver` counts single known data types only.

## Evidence

Closed records are outside Git, in
`benchmarks/runtime/results/claude-lsp-evidence/` of this worktree (an
ignored path). `MANIFEST.sha256` lists 128 files; its own SHA-256 is
`5a0c75e5f6deb5aaa409cec092a757f822bf398c6fde778ecf84c6e8377feb5e`.

| Record | Content |
| --- | --- |
| `matrix-baseline-06deeecf.tsv` | The original failures: every row and mode against the unmodified server |
| `matrix-after-routing.tsv` | After the routing change alone: typing-speed differences gone, missing members remain |
| `matrix-6721c75b.tsv` | Final |
| `agreement-installed-server.txt` | The agreement test against the installed server, failing on `Result.payload` |
| `suites-*/` | One log per suite and a summary that counts aborts, at each stage and on the baseline |
| `probe.py`, `project.py`, `user.py` | The stdio scripts of the first investigation |

Server built from `6721c75b`: SHA-256
`4a41014237ff55f781bb09589be058f5bcaf8af681c3066acd0ce1b7ab0dc183`.

## Integration and cleanup

After merge: build and install both consumers, restart the server, and check
`parsed.` in an open project document. Then remove the evidence directory
after its SQLite import, the worktree `tsrust-claude-lsp`, and the target
`/private/tmp/tsrust-claude-lsp-target`.

## Codex integration

Merged into main at `0039df14`, after compiler storage continuations checkpoint
`5c0b2c94`. The merge has no conflicts. All 129 evidence files, including the
manifest, have been imported into `benchmarks/records.sqlite` under
`claude-lsp-evidence/`, verified byte for byte, with SQLite `quick_check` passing.
The original baseline failures and observational latency records are preserved.

The compiler's existing common-field behavior for `Result.payload` is retained.
No language rule is removed to accommodate editor output. The three requested
compiler queries and the limitations above remain open integration follow-ups.

The VS Code extension now has a regression covering immediate completions for
`parsed.`, `console.log(parsed.pa`, matched error fields and indexed issue fields,
plus a payload hover. It types each character into a real extension-host document
and checks the first answer without a settling delay or retry. This passes with
the merged compiler/server, along with the existing extension-host suite and
extension executable-discovery/grammar tests. Whole-project formatting changes
zero files and fingerprints.

The three agreement suites pass against the merged compiler. `optional_main`
reproduces the original failure: an invalid return in an imported function masks
an independent entry-body diagnostic. It remains a compiler diagnostic replay
issue, not a passing test. Inspection identifies eager dependency scheduling as
the reason changing `diagnostic_first` does not visit the entry's earlier invalid
statement before its failing dependency. Repair must preserve real callee
contracts and keep diagnostic replay separate from executable analysis.

Both installed binaries and the workspace editor server now match the merged
build by SHA-256. After restarting the actual server, the owner's open
`tsrust_test/src/main.tsr` offers `match` and `payload` for `parsed.` inside
`console.log`. The temporary dot was undone; the document is clean with zero
problems. The first attempt before a successful restart still used the old
process and returned no suggestions; that deployment observation is retained.

Final merged regression run: 51 targets, 50 passing and `optional_main` failing
only the retained assertion above; 661 individual tests pass, one fails and one
is ignored. The full run is preserved, not restarted or counted as wholly passed.
No formal timing protocol was run. Integration records use
`tsrust-lsp-agreement-integration-*` in SQLite.

After byte verification and live consumer checks, the Claude evidence copies,
merged clean worktree and 2.9 GB private Cargo target were removed. The branch is
retained, and the unrelated `codex/stdlib-path` worktree is untouched. Compiler
nesting architecture, diagnostic replay repair and the remaining roadmap stay
active; this editor integration does not accept those requirements.
