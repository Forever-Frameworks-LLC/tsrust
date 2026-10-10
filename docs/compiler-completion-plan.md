# Tsrust compiler completion: sustained execution handoff

## Owner request and execution goal

The owner wants one agent to attempt to finish the remaining compiler in one sustained run. Work through the entire remaining compiler scope, using small, testable modules and accepted checkpoints. Do not stop merely because one gate or family finishes. One sustained run does not mean one giant patch or skipping verification. Do not promise completion before auditing what remains.

Another agent is currently merging into main. This handoff is intentionally outside the repository to avoid concurrent tree mutation. Do not edit, format, build into shared targets, refresh consumers, stage or commit in main until the merge owner explicitly transfers exclusive ownership. Read-only inspection and planning may proceed. Once ownership transfers, inspect the new HEAD and merge handoff, then copy this document into docs/compiler-completion-plan.md and reconcile its baseline. Do not assume a724dbc is still HEAD. Do not message another chat without direct user authorization.

## Required reading and current baseline

Repository: /Users/seancotter/Desktop/dev/learning/tsrust.

Read AGENTS.md and docs/development-workflow.md first, then:

- docs/handoff-2026-10-08-resumable-summaries-accepted.md
- docs/resumable-callable-summaries-plan.md
- docs/milestones/resumable-callable-summaries.md
- docs/handoff-2026-10-08-named-declaration-family.md
- docs/roadmap.md and docs/feature-coverage.md
- benchmarks/coverage/language.json, inventory.json, status.json and checkpoints.json
- docs/standard-library-roadmap.md and docs/standard-library-coverage.md
- docs/nested-function-declarations-plan.md
- The latest integration handoff produced by the merging agent.

Last compiler-family publication before the active merge: a724dbc. Invocation sessions are 366e0b0, bounded wrapper completion is 974fa8e, and complete source acceptance is 294af82. Gates 172/173 and the named-declaration family are accepted. Earlier roadmap paragraphs are chronological history, not a reliable list of current failures: reconcile them against newer accepted records and actual source. Gate 174 benchmark layout contracts also have accepted records; inspect the latest numbered milestones before assigning new gate numbers.

The scheduling family has complete compiler acceptance: 2,342 checks pass, zero failures, three intentional ignores. The ignores are the opt-in worklist cost diagnostic, module frontend measurement and subprocess contention worker. Complete editor acceptance is 360 checks, zero failures or ignores. There are no retained baseline correctness failures at this checkpoint.

All 102 focused flow controls and eight depth canaries pass. Three representative matched workloads verify all five native and Node-after-tsc cases. Their emitted programs match historical hashes using the harness's trimmed-output convention. Five consumers are refreshed and smoke-tested. Exact evidence is in benchmarks/records.sqlite. Cleanup removed 339 closed artifacts totaling 6,505,287,564 bytes; do not rebuild just to warm caches.

## Protected owner changes

Fifteen owner fixtures were modified before this handoff. Preserve their exact bytes and index state. Never stage them as part of implementation acceptance or merge cleanup. Reconcile this list with status after the integration handoff:

- crates/tsrust-compiler/tests/fixtures/array-callback-errors/values.tsr
- crates/tsrust-compiler/tests/fixtures/array-fields/catalog.tsr
- crates/tsrust-compiler/tests/fixtures/array-from-collections/documented.tsr
- crates/tsrust-compiler/tests/fixtures/array-from-collections/values.tsr
- crates/tsrust-compiler/tests/fixtures/comma-contexts/guards.tsr
- crates/tsrust-compiler/tests/fixtures/generic-value-adapters/array-parameter-pattern.tsr
- crates/tsrust-compiler/tests/fixtures/generic-value-adapters/sort-search.tsr
- crates/tsrust-compiler/tests/fixtures/generic-value-adapters/unused-alias-contracts.tsr
- crates/tsrust-compiler/tests/fixtures/interfaces/shapes.tsr
- crates/tsrust-compiler/tests/fixtures/map-for-each/values.tsr
- crates/tsrust-compiler/tests/fixtures/map-group-by/documented.tsr
- crates/tsrust-compiler/tests/fixtures/set-algebra/precision.tsr
- crates/tsrust-compiler/tests/fixtures/sets/documented.tsr
- crates/tsrust-compiler/tests/fixtures/string-iteration/values.tsr
- crates/tsrust-compiler/tests/fixtures/type-assertions/assignable.tsr

Use explicit Git paths. Never use git add -A crates, git commit -a, reset, stash, snapshots or archive directories to handle owner work.

## Established design decisions

Keep existing ownership rules. Local captures borrow; escaped owned callables transfer their captured owners. ReadOnly, Mutable and Once contracts describe callable use. Do not introduce shared JavaScript capture cells, implicit clone, a fourth callable variant or a new ownership model without an owner decision.

Optional omitted record fields are undefined. Interfaces and extends form separate nominal record types; no implicit structural subtyping. String indexing uses Unicode character positions and out-of-range errors consistently with arrays. Explicit rest/object spread may copy; ordinary assignment does not imply clone. Preserve selected-entry main behavior across modules. Named imports/exports and aliases are an accepted minimum module direction; default import/export policy must be reconciled before expanding it.

The standard-library direction is bundled native modules, Rust package backends, explicit async/await with Tokio and native threads. No custom goroutine scheduler. Node console parity is the approved console reference. Hover documentation explains observable behavior and examples, not runtime reference versions or compiler provenance.

## Stage 1: derive the actual remaining compiler scope

Do this before choosing only the easiest next feature. Build a requirement ledger in docs/compiler-completion-plan.md. Audit all current language inventory rows, compiler-related standard-library prerequisite gates, open plans, retained rejection witnesses, TODO/unimplemented/panic paths and source diagnostics. A TODO is a lead, not proof of a defect. Reproduce suspected gaps with source witnesses.

For each requirement record: source of the requirement, current behavior, intended behavior, owner policy, dependencies, focused red witness, native/editor/formatter coverage, benchmark support, acceptance evidence and status. Use explicit states: accepted, incomplete, blocked by a decision or intentionally outside compiler scope. Unsupported diagnostics do not close an implementation requirement. Accepted policy limitations must be supported by an owner decision, not invented to shrink the task.

Distinguish compiler functionality from the enormous standard-library API backlog. Compiler work includes parsing, types, inference, ownership/lifetimes, control flow, lowering, native binding contracts, modules and project tooling. The merging agent owns its actual library slice. Do not duplicate its exports or count unimplemented HTTP/database/schema APIs as completed merely because their compiler prerequisites exist. Audit and implement compiler-side built-in gaps that are already part of the approved language scope; coordinate shared registries and catalog changes.

Audit these known leads first, without assuming old descriptions remain accurate:

1. Scoped aggregate composition: borrowing callable records in arrays, Maps, Sets, nullable/union carriers and nested aggregates; ownership transfer, replacement, extraction, reinsertion, borrowing iterators, escape rejection and exact disposal.
2. Fallible borrowing callback fields behind record parameters: typed error transport, catches/finally, replacement, retained loans and cleanup.
3. Named function expressions: private recursive name scope, inferred and annotated signatures, captures, generic/default/rest variants and ordinary expression use.
4. Recursive result inference and checking of unreachable catch bodies: determine current support; avoid breaking provisional re-analysis or dropping source checking accidentally.
5. Module/project completion: unsaved imported files, invalidation, alias/private/rename boundaries, incomplete-source assistance and checked disk edits; reconcile S04 and any merged changes.
6. Compiler type/error foundations in S06: generic functions and containers, tagged Result representation, exhaustiveness and typed propagation. Existing accepted generics must not be redone because this older ledger is stale.
7. Native resource and byte foundations in S07: opaque handles, explicit close/deterministic drop, invalid-use diagnostics, borrowed views, bounds and failure cleanup.
8. Native thread prerequisites and S13 async language lowering: capture transfer/sharing restrictions, async/await types, suspension-safe loans/resources, async entry behavior and native futures. Runtime task/server/database APIs remain separate library families, but their compiler requirements must be accounted for.
9. Remaining approved syntax/type/control-flow families in the inventory, including classes/object features, pattern/type constructs, iterators/generators and other open rows where applicable. Enumerate them; do not assume these examples are already approved full JavaScript parity or silently omit them.
10. Diagnostics, formatter, CLI and editor gaps discovered by the audit: inference failures, malformed arrow/type/generic syntax, quick fixes, highlighting and run/build/test behavior must remain consistent with implemented language features.

Publish the audited scope and dependency order before major implementation. Ask concise direct questions only when a real semantic/product decision is necessary. Continue independent work while an optional answer is pending. Stop dependent work when a required decision has no answer; document the exact blocker. Do not choose a different language behavior merely to avoid asking.

## Stage 2: sustained implementation through numbered gates

Assign the next unused gate number after inspecting the latest integrated milestone records. Keep descriptive active directory names. Gates are reviewable chunks within meaningful feature families. Every progress report should identify the gate number, feature, state, passed checks and next requirement; label defect repairs and family milestones explicitly.

For each gate:

1. Add a focused failing source test demonstrating the missing behavior or defect. Include ownership/failure cases that distinguish the intended design.
2. Implement in isolated modules. Keep shared recursive dispatch and stack frames small. Do not increase worker stacks, add implicit clones, bypass source analysis or special-case only the witness.
3. Verify native behavior with std:testing where possible and an independent Node/Rust reference where relevant. Preserve order, borrowing, transfers, errors and exact disposal. Use project/module execution for imported fixtures.
4. Complete editor facts, hover/completion/signature help/diagnostics, grammar and formatting coverage. Successful modified fixtures must pass token preservation, formatter idempotence and formatted execution. Every new built-in needs complete builtins.json documentation and executable examples before acceptance.
5. Keep matched benchmark support ready: TypeScript counterpart, independent oracle, source contract, manifest/registration, ABI/driver and runner/profile/report hooks. Append manifest rows at the end. Run representative untimed checks as support lands; preserve existing timing protocols and original samples.
6. Run bounded affected checks. Run all eight depth canaries after shared parser/analyzer dispatch changes. Do not launch full regressions after routine edits.
7. Format the whole project before source checkpoints with the current verified compiler. Preserve owner fixture semantics. Commit explicit paths and update the ledger, milestone, coverage status/generated inventory, roadmap and handoff evidence.
8. Continue to the next gate without asking for routine permission or treating the completed gate as overall completion.

Fix every real defect discovered. Classify stale assertions only after independent native/source evidence establishes the new intended behavior. Never remove a correctness witness just to obtain a green run.

## Stage 3: meaningful family acceptance and final compiler audit

At each completed family, run complete compiler/editor regressions and the affected benchmark comparisons. Preserve completed groups and original records; repair/rerun affected groups and resume unrun work instead of restarting the whole matrix. Measure new or changed generated programs under the original protocol; retain previous measurements for unchanged programs. Compiler checking observations are not runtime throughput measurements.

Do not repeat the just-accepted scheduling milestone before changing relevant source. Preserve its one-shot reference controls, per-site prefix/finally witnesses, unchanged operation budgets, bounded eager wrapper depth and capture/default invalidation rules. Never reintroduce blanket State equality shortcuts: equal states can still require named-target resolution and provenance canonicalization.

Final compiler completion requires a requirement-by-requirement audit of the initial ledger. Every in-scope row needs current source/native/editor/formatter evidence and appropriate benchmark support. All compiler-related prerequisites in approved plans must be addressed. No known correctness failure, Rust generation panic, stale accepted-feature rejection or missing promised editor contract may be hidden in a baseline bucket. Policy exclusions and remaining library/platform work must be explicit. Cross-platform acceptance requires actual evidence; macOS success is not Windows native acceptance.

If the scope is too large to finish within one run, preserve the full objective, continue while meaningful progress is available, and leave a precise handoff on interruption. Do not relabel a callable-only subset as the completed compiler. Stop for unresolved required decisions or genuine external blockers, with exact evidence and remaining requirements.

## Environment and publication

Reuse /private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0 and offline/locked Cargo. Node: /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node, first on PATH. Storage Python: /opt/homebrew/bin/python3. Shell is zsh; do not rely on word splitting. Serialize Cargo operations and shared SQLite publication.

SQLite is canonical. Import closed evidence, read it back byte for byte, then remove disposable copies. Preserve immutable identities and protocols. Do not replace the database with a worker database. Refresh five consumers from accepted commits by copy to sibling .refresh-tmp and atomic rename: target/debug/tsrust, target/editor/debug/tsrust, target/debug/tsrust-lsp, target/editor/debug/tsrust-lsp and target/release/tsrust. Smoke-test combined compiler/library/editor behavior. Prune closed generated/native/test outputs and do not rebuild merely to warm caches afterward.

Final deliverables: reconciled compiler requirement ledger, accepted numbered gates and family records, complete relevant editor/docs/formatter coverage, working benchmark support and immutable analytics, integrated source commits, current five consumers, cleanup receipt and a final handoff stating what is proven complete and what genuinely remains.

## Ready-to-copy execution prompt

Continue Tsrust at /Users/seancotter/Desktop/dev/learning/tsrust using /private/tmp/tsrust-compiler-completion-handoff-2026-10-08.md. Goal: attempt to finish the entire remaining compiler in one sustained run, not stop after one gate. Another agent is currently merging: wait for explicit exclusive tree ownership before any repository mutation or shared-target build. After transfer, inspect latest HEAD and integration handoff, read AGENTS.md and docs/development-workflow.md, then copy and reconcile this document as docs/compiler-completion-plan.md.

First audit actual remaining compiler requirements across the current language inventory, plans, source and witnesses. Publish a complete dependency-ordered ledger; old roadmap paragraphs are historical and must be reconciled. Preserve approved ownership/native semantics and all fifteen owner fixture edits. Do not narrow completion to callable gaps or claim the Node standard library is complete. Implement all in-scope compiler requirements through the next unused numbered gates with TDD, isolated modules, native assertions, complete editor/formatter/docs and matched benchmark support. Report gate number and state clearly. Continue autonomously between gates; ask only for real unresolved design decisions and stop dependent work if no answer arrives.

Run bounded affected checks per gate, all eight depth canaries after shared dispatch changes, and complete regressions/performance review at meaningful family milestones. Preserve completed evidence, exact SQLite records and existing runtime measurements for unchanged programs. Never weaken witnesses, hide baseline defects, add implicit clones, increase stacks or bypass checking. Final completion requires a requirement-by-requirement audit, accepted source commits, five refreshed/smoke-tested consumers, cleanup and a physical final handoff. If interrupted or blocked, preserve the full remaining ledger and exact continuation instructions. Do not spawn extra agents without authorization. Current pre-merge compiler publication is a724dbc: scheduling family accepted, compiler 2,342/editor 360 pass, no retained correctness failures.

---

# Requirement ledger (audited 2026-10-08, baseline 5120aad)

Baseline reconciliation: the merge owner returned main with
`docs/handoff-2026-10-08-path-relative.md`. HEAD is 5120aad, not a724dbc. Combined
acceptance there is 2,350 compiler and 362 editor checks with no failures. The latest
numbered milestone is 174; new gates start at 175. The fifteen protected owner
fixtures match the list above.

Every row below was probed against the installed compiler built from 411acbe. A row
is `accepted` only when a source witness ran natively. `stale record` means the
behavior is accepted but `benchmarks/coverage/status.json` still says pending.
`decision` means the behavior is not in any approved plan; an owner decision is
needed before it counts as in scope or as a policy exclusion.

## A. Accepted behavior with stale inventory records

These inventory rows say `pending` but execute natively. Each needs its status row,
evidence link and (where missing) a matched workload, not new compiler work.

`array.literalSpread`, `control.forOfIterable`
(arrays, strings, Map, Set, entries/keys), `control.void`,
`syntax.defaultParameters`, `syntax.export`, `syntax.import`, `syntax.hoisting`,
`syntax.optionalCatchBinding`, `string.iteration`, `null.narrowing`,
`undefined.narrowing`, `nullish.assignment`, `optional.field`, `optional.return`,
`optional.value`, `int.parse`, `object.propertyEnumeration` (`Object.keys`,
`Object.entries`, `for in`), `type.generic` (functions and record types),
`type.nonNullAssertion`, labeled control, tuples, enums, `satisfies`, object spread,
destructuring, optional chaining, `in`.

Reconciled in gate 178: nineteen inventory rows above now carry `implemented` with
their existing workloads and milestone records. The remaining names in the list
(labeled control, tuples, enums, `satisfies`, object spread, destructuring, optional
chaining, `in`) already had accepted rows under other identifiers.

## B. Incomplete and inside approved plans

| # | Requirement | Source | Current behavior | Status |
| --- | --- | --- | --- | --- |
| B1 | Named function expressions: private recursive name, annotated and inferred signatures, captures, generic/default/rest forms | nested function plan; lead 3 | accepted natively; parameter-default position rejected | accepted: gate 175 ([record](milestones/175-named-function-expressions.md)) |
| B2 | Recursive result inference | gate 173 limit; lead 4 | inferred from an independent return; diagnostic only when none exists | accepted: gate 176 ([record](milestones/176-recursive-result-inference.md)) |
| B3 | Type checking of unreachable `catch` bodies | record gate limit; lead 4 | body checked at the end of analysis with a runtime error name; not emitted | accepted: gate 177 ([record](milestones/177-unreachable-catch-bodies.md)) |
| B4 | Caught runtime error passed to generic `strictEqual` | named family handoff | repaired before this plan: the internal runtime error record is registered for generic specialization ([record](milestones/caught-generic-assertions.md)); eleven assertion shapes re-verified | accepted; gate 177 adds witnesses |
| B5 | Scoped aggregate composition: arrays, Maps, nullable replacement and parameter transport of records with borrowing callbacks | scoped aggregate plan; lead 1 | scoped forms supported; family regressions and timing pending; mixed-record unions assigned to D2 by owner | accepted: gate 182 ([record](milestones/182-scoped-aggregate-composition.md)); mixed-record unions remain in D2 |
| B6 | Fallible borrowing callback fields behind record parameters | record gate limit; lead 2 | failures propagate with typed handlers; `error-effect` only when no record of the type is constructed | accepted: gate 180 ([record](milestones/180-fallible-record-callbacks.md)) |
| B7 | Callbacks stored through a record parameter that borrow from the caller | record gate limit | data and callback inputs, relays, replacement and clearing, branch, loop and exception-region stores, installer closures called directly or through captured closures; abstract installer invocation and captured-record stores are diagnosed | accepted: gate 183 ([record](milestones/183-caller-borrowed-record-stores.md), [map](b7-caller-borrowed-acceptance-map.md)) |
| B8 | Generic adapter compile time (specialization worklist, state joins) | named family handoff | adapter workload checks in 2.4 s release and 10.4 s debug after resumable summaries; earlier 55 s and 120 s limits no longer reached | accepted: gate 184 ([record](milestones/184-generic-adapter-compile-time.md)); shared binding storage is optional later work |
| B9 | S04 project editor: unsaved imports, invalidation, rename/removal, stale results | standard library roadmap | in progress | incomplete |
| B10 | S06 type foundations: tagged Result, exhaustiveness, typed propagation | standard library roadmap | in progress | incomplete |
| B11 | S07 resources and bytes: opaque handles, close, drop, views | standard library roadmap | pending | incomplete |
| B12 | S12 thread prerequisites: transferred captures, Send-like checks | standard library roadmap | pending | incomplete |
| B13 | S13 async compiler: `async`/`await`, async main, suspend-safe loans | standard library roadmap | `unsupported` at declaration | incomplete |
| B15 | Assignment as a value for strings, arrays and records (`(text = "x").length`) | inventory `control.assignmentExpression` | the value is the target read in place; computed places rejected | accepted: gate 179 ([record](milestones/179-owned-assignment-values.md)) |
| B14 | `switch` fallthrough between non-empty cases | inventory `control.fallthrough` | `unsupported` with guidance | decision (see C) |

## C. Not in an approved plan: owner decision required

Probed behavior is a syntax, type or `unsupported` error for every row.

- **Classes and `this`**: `class.*` (13 rows), `function.this`, `function.bind`,
  `function.call`, `function.apply`, `function.arguments`, object method shorthand,
  getters/setters in object literals, `instanceof`, `new` of user types, `Error`.
- **Type-level features**: string and number literal types, discriminated unions,
  generic constraints and defaults on aliases, `keyof`, `typeof` types, indexed
  access, mapped, conditional, template literal and intersection types, utility types
  (`Partial`, `Pick`, `Record`, `Readonly`, `ReturnType`, …), `never`, `unknown`,
  `as const`, readonly tuples, namespaces, ambient declarations, const enums,
  declaration merging, overloads, anonymous structural record literals.
- **Protocols**: generators (13 rows), custom iterators and `Symbol.iterator`
  (13 rows), `Symbol`, decorators (17 rows).
- **BigInt** (22 rows).
- **JavaScript coercions and legacy syntax**: truthiness in conditions and logical
  assignment on non-booleans, loose equality, `var`, `with`, array holes and sparse
  arrays, writable `length`, `delete`, computed property names, object identity
  equality, tagged templates, regular expression literals.
- **Globals**: `encodeURI` family, `escape`/`unescape`, `globalThis`, `JSON`, `Date`.

`type.any`, `global.eval` and `object.prototypeMutation` already carry a policy
exclusion.

## Owner decisions (2026-10-08)

1. **In scope:** classes and `this`; type-level features; generators and iterators;
   BigInt and Symbol. Each becomes a family in section D below.
2. **Truthiness is in scope:** strings, numbers and nullable values in conditions and
   logical operators. **Excluded by policy:** loose equality and inequality, `var`,
   `with`, array holes and sparse arrays, writable `length`, `delete`. Their
   diagnostics stay; the inventory rows carry `policy`.
3. **Recursive result inference stays a superset of TypeScript** (gate 176).
4. **Order:** B15, then scoped aggregates (B5 to B7), then the remaining B rows and
   the section D families.

Still undecided and therefore not in scope: decorators, tagged templates, regular
expression literals, computed property names, object identity equality, `switch`
fallthrough (B14), and the URI, `globalThis`, `JSON` and `Date` globals. The owner
confirmed nothing about B15; it proceeds on the stated rule (the value of an owned
assignment is a read-only view of the target) and is flagged in its record.

## D. Families added by owner decision

| # | Family | Depends on | Status |
| --- | --- | --- | --- |
| D1 | Truthiness in conditions and logical operators | none | accepted: gate 181 ([record](milestones/181-truthiness.md)) |
| D2 | Type-level features: literal types, discriminated unions, generic constraints and alias parameters, `keyof`/`typeof`/indexed/mapped/conditional types, utility types, `never`/`unknown`, `as const`, readonly tuples, overloads, namespaces, structural record literals | none; discriminated unions need literal types | incomplete |
| D3 | Classes and `this`: declarations, constructors, fields, methods, getters/setters, static, `extends`/`super`, `new`, `instanceof`, object method shorthand, `Error` | ownership rule for `this` (design step of the family) | incomplete |
| D4 | Generators and iterators: `function*`, `yield`, iterator protocol, `Symbol.iterator` | D5 for `Symbol`; async forms need B13 | incomplete |
| D5 | BigInt and Symbol | none | incomplete |

## Dependency order

1. B1–B4: independent of each other; no shared prerequisite. Gates 175–178.
2. B5–B7: build on the accepted record gate. B6 needs an error-contract carrier for
   record parameters; B5 unions depend on it for nullable replacement.
3. B10 before B11; B11 before B12 and B13 (S06 → S07 → S12/S13).
4. B9 is editor-only and independent.
5. Section C families start only after their owner decision. Classes depend on the
   `this` policy; discriminated unions depend on literal types; generators depend on
   the iterator protocol decision; async generators depend on B13.
6. Section A record reconciliation runs alongside each family acceptance.

## Owner scope decision (2026-10-09): discriminated unions

Mixed-record unions containing a capturing record move from B5 to a dedicated D2
feature. This is sequencing, not removal from the compiler objective. Implement
literal discriminants, narrowing and tagged storage together with capture ownership
and failure cleanup. Include typed validation success/error shapes suitable for the
planned schema validation library. B5 can close after its agreed aggregate forms
pass complete compiler/editor regressions and matched performance review.
