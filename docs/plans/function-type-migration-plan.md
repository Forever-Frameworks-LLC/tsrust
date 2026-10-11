# Function and access-wrapper migration plan

Status: implementation has focused acceptance, 2026-10-04. Checkpoint101 regressions and measurements remain pending.
Follow [development-workflow.md](../development-workflow.md). Preserve current Gate100 work,
existing editor fixes, running checks and deferred feature-family measurements. These phases do
not allocate or renumber compiler gates.

## Progress (2026-10-04)

Decisions taken with the user:

- `Once<Function<...>>` is the single-call owned function; it replaces `once` and `owned once`. `Owned<T>` marks an argument the function takes over and replaces `consume`; `Owned<Function<...>>` equals plain `Function<...>` and replaces the `owned` alias.
- Function argument lists accept optional labels, optional positions and a rest position: `Function<int, [text: ReadOnly<string>, limit?: int, ...rest: int[]]>`.
- Arrow type syntax stays as a second spelling. Wrappers compose with it: `Mutable<(row: Mutable<Row>) => void>`.
- Access wrappers are used wherever access is written: in Function argument lists, arrow type parameters and arrow function parameters.

Implemented with focused acceptance:

- Parsing in the shared annotation parser (`type_syntax.rs`). The new spelling lowers to the existing callable and access contracts; no new runtime representation. For matched programs the generated Rust is identical apart from source byte offsets.
- The five names are reserved type names. Diagnostics name the wrappers instead of the keywords.
- IntelliSense: hover and completion documentation for `Function`, `Mutable`, `ReadOnly`, `Once` and `Owned` in `builtins.json`, each with native-verified examples.
- Tests: `crates/tsrust-compiler/tests/function_types.rs` (5 tests) and an editor test in `crates/tsrust-lsp/tests/intelligence.rs`. Evidence logs: `target/editor/function-types-compiler.log` (33 suites, 398 passed) and `target/editor/function-types-editor.log` (148 passed).

Further work completed the same day:

- Access wrappers are accepted on function declaration parameters and on arrow parameters. Stated access is a contract: the body may use the value less, never more, and callers lend or give what is stated (`parameter-access` when the body exceeds it).
- Inferred types render in the new spelling in compiler facts, hover and signature help. The language server takes `Function<...>` apart structurally and keeps the built-in names when remapping imported types.
- VS Code grammar highlights the five built-in types; snippets added. The keyword highlighting rule was removed.
- `tools/migrate_function_types.py` converts keyword spellings in raw text. Applied to compiler and editor tests (71 files), compiler fixtures, the two benchmark support sources and 56 workload sources. Every converted `.tsr` file produces identical generated Rust apart from source-offset-derived names; workload fingerprints in `manifest.json` and three registration digests were updated. Pre-conversion copies are in `archive/function-type-migration-start`.
- Current guides use the new spelling.

Retirement and editor repair completed:

- Remaining source templates, compiler unit sources, native signature consumers and catalog strings were audited and migrated. Old access spellings now produce `retired-access-keyword` with the replacement type. They are absent from completion suggestions; internal ownership enum names remain implementation details.
- `tools/audit_function_type_spellings.py` checks live source consumers; immutable history is preserved. Snapshot: `archive/function-keyword-retirement-20261004-101613`.
- Formatter recognizes generic type spans, including nested `>>`, without changing comparison/shift spacing. Exact reported signature has an idempotence regression. `function-types-formatting-final.log`: 7 passed.
- VS Code grammar checks, compile and package passed. Rebuilt `editors/vscode/tsrust-0.2.0.vsix` installed; current workspace server binaries rebuilt. Reloaded VS Code and verified highlighted Function/Mutable and tight formatting in the open document.
- Focused compiler suites passed after retirement repairs; see `function-keyword-retirement-{second,third,final}.log`, `function-types-type-syntax-final.log`, `function-types-spread-preservation.log`. All 149 existing editor tests plus the new keyword-completion rejection passed.

Remaining:

- Gate101 array-literal spread has started; acceptance matrix, editor documentation,
  benchmark registration and checkpoint remain open. See its milestone.

Harness repair evidence:

- Retained prefix counts/order/full-object digests, while updating stale live metadata
  baselines. Original tests and available transient manifests are archived intact.
  `target/editor/function-types-harness-rebaseline.json` records old/new identities.
- Metadata guards test membership inside oracle input arrays instead of requiring
  an entry to be first. Real compiler emission replaces synthetic native ABI fixtures.
  Same-signature workload/helper binding now uses checked workload-first declaration
  evidence, retaining fail-closed ambiguity checks.
- Report categories include later callable/generic/path work. Exact counterpart tests
  route through the actual result-family driver and observer; approximate Math uses
  immutable references. Added independent path and nested-generic arithmetic oracles.
  Trailing original oracle tests are retained. No measured records were changed.
- Complete final guard suite: **282 pass, 0 fail**, `function-types-benchmark-guards-complete.log`.
- Full regressions and matched native-versus-Node-after-tsc measurements at a completed feature-family milestone, under the updated implementation-first workflow. Focused checks above are implementation evidence, not checkpoint timing.

## Agreed public direction

```tsrust
Function<int>                         // () => int
Function<int, [string]>               // (text: string) => int
Function<boolean, [int, int]>         // two int arguments, boolean result
Mutable<Function<int>>                // scoped reusable mutable callback borrow
ReadOnly<Function<int>>               // scoped readonly callback borrow
Mutable<int[]>                        // scoped mutable array access
ReadOnly<string>                      // scoped readonly string access
```

Return type comes first; an omitted argument tuple means zero arguments, not arbitrary
arguments. Function signatures and access wrappers are compiler-recognized types.
They must not add runtime wrappers, implicit cloning, sharing, allocation or dynamic
dispatch. Preserve the existing ownership, lifetime and invocation checks.
AsyncFunction will eventually follow the same shape; async implementation is outside
this migration. Arrow expressions remain ordinary function expressions.

## Phase 1: freeze the complete syntax and baseline

- [ ] Produce a contract table for every existing data access and callback ownership
  state. Map read and mutate to ReadOnly<T> and Mutable<T>; explicitly resolve consume,
  once, owned once and the owned alias before claiming keyword retirement is complete.
  Replacement names are settled in Progress above. Bare Function owns a reusable
  environment, following the current bare callable default.
- [ ] Specify argument labels, optional positions and rest positions. Evaluate
  `Function<int, [text: ReadOnly<string>, limit?: int]>` and
  `Function<int, [...values: int[]]>` as accepted forms; see Progress above.
  Default expressions belong to declarations, not signature types. Preserve omission
  versus explicit undefined, required-position arity and runtime supplied counts.
- [ ] Define wrapper placement/composition: nullable borrowed callback versus callback
  returning nullable data, arrays of callbacks, wrapped array elements, nested wrappers,
  invalid ReadOnly<Mutable<T>>, borrow-return restrictions and scalar permissions.
  Do not conflate data access, invocation access and ownership transfer.
- [ ] Specify Function<void>, Function<Function<int>>, generic return/argument types,
  aliases and imported names. Decide whether arrow *type* syntax remains a supported
  spelling; its retention must not imply retaining access keywords in the final API.
- [ ] Decide built-in-name reservation/shadowing and diagnose incorrect generic arity.
  ReadOnly is the agreed spelling; explicitly distinguish its borrowing meaning from
  TypeScript's structural Readonly utility. Function is an exact signature, not the
  loose JavaScript/TypeScript Function constructor/type. Do not silently accept
  AsyncFunction before futures and scheduling exist.
- [ ] Record current compiler/editor identities and focused baseline results. Build a
  file-and-location ledger from searches AND dependency tracing, classifying each hit
  as live source, current docs/examples, compatibility test, generated file, dependency
  or immutable historical evidence. Record a disposition and verifying check per hit.

## Phase 2: compiler recognition and canonical contracts

| Surface to audit | Concrete starting points | Required acceptance |
|---|---|---|
| Lexing and declaration parsing | `syntax.rs`, `type_syntax.rs`, `source_walk.rs` | Nested generic closers, argument tuples, spans, grouped/union types, local/parameter/result/field annotations; preserve expression parsing |
| Annotation conversion and resolution | `callable_annotations.rs`, `callable_resolution.rs`, `generic_validation.rs` | Lower new spelling to existing checked callable/access contracts; resolve aliases and errors consistently |
| Generic inference and specialization | `type_substitution.rs`, `generic_inference.rs`, `generic_functions.rs`, `generic_calls.rs`, `generic_defaults.rs`, `generic_default_scope.rs`, `generic_records.rs` | Traverse return types, argument tuples and wrappers recursively; stable contract identities and finite specialization |
| Ownership and invocation | `analysis.rs`, `flow.rs`, `callable_compatibility.rs`, `callable_contracts.rs`, `callable_dispatch.rs` | Capture mutation, reuse after borrowing, readonly rejection, moves, once calls, overlapping loans and nonescaping borrows |
| Storage and layout | `callable_storage.rs`, `callable_layout.rs`, `callable_size.rs`, `storage.rs`, `cloneability.rs`, `record_layout.rs`, `projections.rs` | Local/field/array/nullable callable transport preserves origins, layout, lifecycle and explicit clone restrictions |
| Optional, rest, spread and nullable calls | `parameters.rs`, `parameter_arity.rs`, `rest_calls.rs`, `spread_calls.rs`, `spread_builtins.rs`, `optional_chain.rs`, `unions.rs`, `logical_values.rs` | Ordinary, optional/default/rest/spread, builtin and higher-order calls retain arity, ordering, ownership and lazy evaluation |
| Imports and project interfaces | `project.rs`, `project_link.rs`, `project_facts.rs`, `native_bindings.rs`, `native_path.rs` | Export/import aliases, std signatures and cross-file generic calls resolve identically |
| Public type rendering and formatting | `syntax.rs`, `analysis.rs`, `diagnostic.rs`, `formatting.rs`, all signature-building callers | New canonical spelling in inferred types, diagnostics and formatting; formatting idempotence and accurate spans |
| Native emission and caching | `emit.rs`, `native_emit.rs`, `spread_emit.rs`, `spread_builtin_emit.rs`, `logical_values_emit.rs`, `native_cache.rs` | Existing ReadCallback/Callback/OnceCallback contracts and ABI preserved; no added wrapper costs or stale-cache reuse |

Paths in this table are under `crates/tsrust-compiler/src/`. These are audit targets,
not an assertion that every file requires an edit. Follow callers and exhaustive
matches when adding annotation variants; compilation alone does not establish coverage.
The existing structured AST has Applied/Callable/Array/Union forms but no tuple form;
signature argument tuples need deliberate representation and traversal. Supporting
signature tuples does not require enabling general tuple runtime values.

- [ ] Introduce new syntax alongside the old syntax for a bounded migration period.
- [ ] Prefer one shared normalization path; do not independently implement ownership
  semantics in each parser. Assert equivalent old/new checked contracts during transition.
- [ ] Keep internal Mode/CallbackKind names where useful. Public keyword retirement
  is not a requirement to rename every internal Read/Mutate enum or Rust trait.

## Phase 3: LSP and installed editor

- [ ] Audit `crates/tsrust-lsp/src/intelligence.rs`, `generic_intelligence.rs`,
  `project_intelligence.rs`, `project_generic_intelligence.rs`, workspace/session,
  diagnostics and code-action modules for independent parsing and rendered type text.
- [ ] Update completion, hover, parameter help, nested-call active-parameter selection,
  aliases/import navigation, diagnostics, fixes and type suggestions together. Labels
  and optional/rest argument metadata must survive tuple normalization.
- [ ] Audit every entry in `crates/tsrust-lsp/src/builtins.json`: signatures, parameter
  labels/help, documentation and executable examples. Include std:path function values,
  array callbacks and inferred/stored/factory signatures. Explain behavior first and
  ownership afterward; keep migration provenance out of user help.
- [ ] Add hover/completion documentation for Function, Mutable and ReadOnly in the
  catalog or explicitly designed equivalent built-in-type catalog path. Document arity,
  wrapper restrictions, example output and ownership. No AsyncFunction completion yet.
- [ ] Update VS Code grammar, snippets, extension-generated source and docs:
  `editors/vscode/syntaxes/tsrust.tmLanguage.json`, `snippets/tsrust.json`,
  `src/extension.ts`, `README.md`, `scripts/grammar.test.cjs`, `src/test/suite.ts`,
  `src/test/modules.ts` and packaging checks. Audit `.vscode/` examples/settings.
- [ ] Verify nested generic highlighting, type-versus-expression arrows, malformed
  partial typing, multi-file edits, hover and signature help in the installed extension.
  Build packaged output through its normal build path; do not manually edit out/VSIX.

## Phase 4: migrate live consumers and evidence

- [ ] Inventory all compiler integration tests and fixtures, embedded source strings,
  generated-Rust expectations, LSP tests, native-binding signatures, CLI examples and
  source templates. Migrate by parsed type position, not global text substitution.
- [ ] Cover callbacks, captures, local/field/array storage, moves, recursive callable
  results, retained/single-use callbacks, nullable calls, generics, modules, optional
  parameters, rest and Gate100 spread fixtures. Retain negative ownership witnesses.
- [ ] Update current README/VISION, type-system/ownership docs, callable locals/fields/
  arrays, nullable callables, parameter docs, stdlib contracts and current roadmap links.
  Historical accepted milestones retain their original evidence; annotate their current
  replacement where needed rather than retroactively changing what passed.
- [ ] Audit `benchmarks/runtime/workloads/`, harness generators/observers, embedded
  source and native ABI assertions. Keep matched TypeScript counterparts semantically
  equivalent without inserting Tsrust-only wrappers into Node sources.
- [ ] Update live benchmark source hashes/registrations when source changes. Preserve
  prior records, original sources, measured semantic epochs and byte-exact restoration.
  Do not rewrite archives, accepted diagnostic snapshots or canonical SQLite records.
- [ ] Update `benchmarks/coverage/language.json` and `status.json` for native syntax
  entries, then regenerate inventory/docs with `update.mjs`. Do not mark TypeScript
  utility Readonly or general tuples implemented merely because these wrappers exist.
  Separate focused implementation acceptance from pending checkpoint measurements.

## Phase 5: verification and retirement

- [ ] Positive matrix: zero/multiple/named/optional/rest arguments; nested functions;
  generic aliases/imports; primitive/array/record results; nullable callbacks; default,
  optional and spread calls; ReadOnly and Mutable on data and callback environments.
- [ ] Negative matrix: missing/excess type arguments, malformed/non-tuple argument lists,
  invalid wrapper composition/targets, readonly mutation, borrowed escape, overlapping
  mutation, moved-value reuse, repeated once invocation and unsupported AsyncFunction.
- [ ] Execute focused fixtures at O0/O3 where output matters; verify return values,
  side-effect order, ownership/disposal and representative old/new ABI equivalence.
  Verify every changed executable catalog example and editor protocol assertion.
- [ ] Add a maintained public-syntax audit with an explicit allowlist for transition
  fixtures, immutable history and intentional internal names. Search read/mutate/consume/
  owned/once in type positions plus arrow signature emitters, generated snippets and
  source templates. Search Function/ReadOnly/Mutable to find partial implementations.
  Classify every remaining live hit; raw keyword count is not proof of completion.
- [ ] At a completed feature-family milestone, run the full regression and relevant matched
  native-versus-Node-after-tsc comparisons. Preserve passed checks; repair/rerun affected
  suites and resume unrun suites. Retiming only addresses affected/unresolved workloads.
- [ ] Retire old keyword parsing/completion/highlighting once all replacement contracts
  and live consumers pass. Keep explicit migration diagnostics and rejection tests.
  Do not leave the final public language dependent on keyword aliases.

## Completion and recovery

Completion requires a closed disposition ledger, accepted syntax contract, passing
focused compiler/native/editor/example checks, migrated live consumers, and no
unclassified old public syntax. Report checkpoint status separately: unmeasured work
must never be described as measured. This document alone does not close a gate.

Before each implementation phase, retain exact pre-edit files and source identities
because this workspace currently has no usable Git repository. Restore only this
migration's changed files/hunks if necessary, preserving concurrent unrelated work.
During expansion, old syntax remains readable; preserve originals when migrating
consumers. If a phase fails, stop its rollout, repair affected checks or restore its
scoped edits. Historical analytics and accepted-source evidence are never rollback
targets. Complete the requested planning stage before beginning implementation.
