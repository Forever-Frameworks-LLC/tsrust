# Handoff: continuing B5 after the carrier joins

Date: 2026-10-09. HEAD when written: `c5272b2b`.

Read these first, in this order:

1. `AGENTS.md` and `docs/development-workflow.md` (project rules; they override habits).
2. This file (how the work is done, and what is next).
3. `docs/b5-scoped-aggregate-acceptance-map.md` (status of every storage form).
4. `docs/scoped-callable-transport.md` (user-facing rules; keep it true).
5. `docs/handoff-2026-10-09-b5-carrier-joins.md` (what the last eight commits did).

**B5 is not complete.** Five forms still report source diagnostics. Do not describe B5
as done, and do not treat a diagnostic as support.

## 1. State of the tree

- All work is committed. `git status --short` shows exactly fifteen modified fixtures
  under `crates/tsrust-compiler/tests/fixtures/`. They are the owner's edits. Never
  stage, format, or restore them from Git. The list is in
  `docs/handoff-2026-10-09-b5-nullable-fields-review.md`.
- The five installed consumers (`target/debug/tsrust`, `target/debug/tsrust-lsp`,
  `target/editor/debug/tsrust`, `target/editor/debug/tsrust-lsp`,
  `target/release/tsrust`) were refreshed from `c5272b2b`.
- Whole compiler crate (2,575 tests) and whole editor crate (424 tests) passed at
  `2e49f92b`. Commit `c5272b2b` changed one emitter condition and two documents and ran
  the affected suites only. Run both whole crates before the next acceptance.
- No timing run exists for this work. `scoped_record_composition` keeps
  `measured: false` in `benchmarks/runtime/manifest.json` until B5 closes.

## 2. Owner decisions in force

Taken on 2026-10-09 through direct questions. Do not reopen them without asking.

1. **Scope-end release for control-flow stores.** A nullable callback record local, or a
   nullable field of a local record, that is stored inside a branch, loop, exception
   region or labelled block, or whose record is replaced as a whole, or that belongs to
   an array element, keeps every stored environment borrowed until the local's scope
   ends. Locals with only direct stores in a normal lexical sequence keep precise
   release. This is deliberately coarser than Rust liveness so it can be relaxed later.
2. **Structural record keys reject `number` fields.**
3. **Structural record keys reject callable fields.** `Set<Record>` for a capturing
   record is therefore a policy rejection, not a gap.
4. **Nullable owned results are allowed.**
5. Unary signs bind to their operand in formatted code (`-1`). Decided 2026-10-08.

Earlier standing decisions still apply: no implicit clone; local captures borrow;
arrows copy scalars when created (so `(): int => i` reads the value `i` had then, which
differs from Node); ask only for necessary design decisions.

Still open, ask before acting: B14 `switch` fallthrough; whether the formatter may
rewrite the fifteen protected fixtures.

## 3. The model you are extending

Two regimes exist for a nullable callback record. Keep them separate.

**Precise regime** (previous agent, `array_storage_epochs.rs`,
`nullable_record_storage.rs`). A local with only direct stores gets a new native type
at each store. The emitter shadows the variable (`drop(place); let mut place: New = …`).
Captures are released at the store.

**Carrier regime** (this session). A *sticky* local has one type for the whole
function: the join of every environment stored into it. Stores convert into that type;
captures are never released by a store.

Where each part lives:

| Concern | Location |
| --- | --- |
| Which locals are sticky | `nullable_record_fields::control_stores` (syntactic scan per block), `Analyzer::sticky_nullable`, `Context::control_stored`, `mark_control_stored` (recomputed when a context is rebuilt) |
| Loans are kept | `capture_loan_binding::replace_target_capture_loans` uses `extend_from` instead of `transfer` |
| Type join | `record_branch_carriers::join_nullable_record_unions`, `join_sticky_record_types`, `join_record_types` |
| Callback families in record fields | `borrowed_conditional_carriers::record_field_callback_family` (joins owned and borrowing callbacks) |
| Planning for locals and fields | `record_branch_carriers::record_sequence_carriers` (runs once per block, innermost first) |
| Planning for array elements | `nullable_record_fields::nullable_array_slot_carriers` |
| Stored values take the carrier type | `record_branch_carriers::retarget_nullable_store` (retypes tagged values, record literals and array literals in place; wraps everything else) |
| Reads take the carrier type | `nullable_record_fields::retype_carrier_read` (names, fields, indexes, narrowing, optional chains, calls through callback fields, `pop`/`shift`) |
| Results | `nullable_record_result`, `joined_record_result`, `joined_array_result`, `borrows_local_storage` |
| Conditional values | `conditional_record_join` |
| Native conversion | `record_carrier_emit::record_carrier_conversion` |
| Native enum identity | `emit.rs::nominal_union_alias`: every concrete environment of one nullable record type is a type alias of the first such enum. Any new per-union `impl` must skip aliased unions |
| Parameters | `scoped_record_containers.rs` decides what is allowed; `emit.rs::callback_bounds` declares the generic callbacks; `map_emit.rs` shares Map entry layout |
| Diagnostics for unplanned forms | `nullable_record_fields::reject_unplanned_nullable_field_stores` |

Facts that cost time to discover:

- **Absent environments.** An absent value has a payload record whose callback field is
  `Type::Callback(absent id)`. In a join, the absent environment and the bare
  declaration are both "bottom": the other side wins. In generated code a conversion
  from an absent callback is `unreachable!`.
- **`aggregate_callable_facts(instance).scoped` is false for an empty environment and
  for an environment that owns its captures.** Ask the *declaration*
  (`scoped_records.declaration(id)`) when you mean "is this a capturing record type".
  Several fixes this session were exactly this substitution.
- **Analysis types are point types.** After a store the binding's type is the type of
  that store. Reads checked earlier or later carry stale types and stale call targets.
  Every carrier plan must retype reads with `retype_carrier_read`.
- **An injected value and its inner expression share one span.** When you match values
  by span in a walk, remove the entry on the first hit.
- **`record_storage_writes::collect` skips some values.** It does not report arrays of
  unions or stores whose value type it does not select. Scan declarations yourself when
  you need them (see `nullable_array_slot_carriers`).
- **`record_sequence_carriers` runs again on a rebuilt context** in
  `record_parameter_carriers.rs`. State kept on `Context` must be recomputed there.
- **A callee is generic over callbacks named `C{contract id}`.** One name per contract,
  so two parameters with the same nullable record type would be forced to one
  environment. That case is diagnosed, not supported.

## 4. How each step was done

Follow this loop. It is what kept the tree green.

1. **Probe before coding.** Write one-line programs to a scratch file, run them with
   the debug compiler, and compare with the pinned Node reference:

   ```sh
   export PATH=/Users/seancotter/.nvm/versions/node/v24.11.1/bin:/opt/homebrew/bin:$PATH
   export CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target CARGO_INCREMENTAL=0
   cargo build --offline --locked --bin tsrust
   $CARGO_TARGET_DIR/debug/tsrust run file.tsr
   node tools/node-reference/run.mjs /absolute/path/file.tsr
   ```

   Classify each result: correct, source diagnostic, wrong output, or "Rust rejected
   generated code" / compiler panic. The last two kinds are defects; fix them or turn
   them into a source diagnostic in the same step. Keep the probe scripts and rerun all
   of them after every change; earlier forms regress easily.

2. **Know the baseline.** Before blaming new code, check whether the form fails without
   nullable types or at the previous commit. Three "new" failures this session were old
   regressions that subset test runs had missed.

3. **Ask when a rule changes.** If support requires a new ownership rule, a policy for
   an edge, or a scope reading of the ledger, stop and ask the owner with concrete
   options and a recommendation. Implementation details are yours to decide.

4. **Prefer one shared mechanism.** Extend the join, the retargeting or the read
   retyping instead of adding a planner per form. A special case is acceptable only as
   a source diagnostic for a form you are not supporting yet.

5. **Tests.** Add them to `crates/tsrust-compiler/tests/scoped_record_composition.rs`.
   Pattern: a `run(body, expected)` closure over `support::execute` with
   `support::format_round_trip`, several programs per test, and a `rejected(body, code)`
   closure for diagnostics. Include at least one capture-conflict witness that proves
   the ownership rule, and one diagnostic witness per form still unsupported. When a
   form becomes supported, convert its rejection row into a positive row; never delete
   it silently.

6. **Editor witness.** Add one test to `crates/tsrust-lsp/tests/intelligence.rs`:
   formatted source has no diagnostics; a small textual replacement produces the
   expected diagnostic code. Replacement strings must match *formatted* text.

7. **Guide.** Update `docs/scoped-callable-transport.md`: behavior first, one short
   example, limits stated directly. Run each documented example natively and on Node.

8. **Workload.** Add a helper to both
   `benchmarks/runtime/workloads/scoped_record_composition.tsr` and `.ts`, call it at
   the end of the `output.push([...])` list, and update:
   - `scoped-record-composition-oracle.mjs`: append the arithmetic formula;
   - `scoped-record-composition-registration.test.mjs`: append the two expected values
     (inputs 1 and 2) and one or two mutation controls;
   - `scoped-record-composition-source-contract.mjs`: add shared snippets at the front
     of `shared`, and extend the `output.push` snippet;
   - `manifest.json`: replace the two SHA-256 values of this workload as text.

   The TypeScript counterpart has no `ReadOnly<>` or `Mutable<>`; use the plain type
   there and keep such signatures out of the shared snippets. Then:

   ```sh
   node --test benchmarks/runtime/scoped-record-composition-registration.test.mjs
   (cd benchmarks/runtime && node run.mjs --verify --engines=native,node-tsc --only=scoped_record_composition)
   ```

   The workload is the best defect finder available: it runs the new form inside a
   large program where callbacks are fallible. It exposed the stale call target defect.

9. **Regression.** Run the composition suite while iterating (about two minutes). Before
   a commit that changes shared analysis or emission, run both whole crates (about
   twenty minutes together) in the background and write the guide or workload meanwhile:

   ```sh
   cargo test --offline --locked -p tsrust-compiler --no-fail-fast > /private/tmp/<name>-whole-compiler.log 2>&1
   cargo test --offline --locked -p tsrust-lsp --no-fail-fast > /private/tmp/<name>-whole-editor.log 2>&1
   ```

   Use `--no-fail-fast` always. Do not wait with a foreground command longer than ten
   minutes; poll the log.

10. **Format with the fixtures protected.**

    ```sh
    S=<scratch dir>; git status --short | grep '^ M crates/tsrust-compiler/tests/fixtures/' | cut -c4- > $S/list
    while read f; do mkdir -p $S/$(dirname $f); cp -p $f $S/$f; done < $S/list
    TSRUST=$CARGO_TARGET_DIR/debug/tsrust node tools/format-project.mjs
    while read f; do cmp -s $f $S/$f || cp -p $S/$f $f; done < $S/list
    ```

    Rerun the registration test and the workload verification after formatting.

11. **Evidence into SQLite, then delete the loose copy only after a byte comparison.**

    ```sh
    PY=/opt/homebrew/bin/python3; DB=benchmarks/records.sqlite
    $PY benchmarks/storage/store.py import --root $PWD/benchmarks --kind runtime-result $DB runtime/results/<file>.json
    $PY benchmarks/storage/store.py read $DB runtime/results/<file>.json | cmp - benchmarks/runtime/results/<file>.json && rm benchmarks/runtime/results/<file>.json
    $PY benchmarks/storage/store.py import --root /private/tmp --kind gate-log $DB <name>.log
    $PY benchmarks/storage/store.py read $DB <name>.log | cmp - /private/tmp/<name>.log && rm /private/tmp/<name>.log
    ```

    Keep logs of failing runs too; they are evidence.

12. **Commit with explicit paths.** Never `git add -A`, `commit -a`, stash, reset, or
    create snapshot directories.

    ```sh
    git status --short | grep -v '^ M crates/tsrust-compiler/tests/fixtures/' | cut -c4- > $S/paths
    xargs git add -- < $S/paths && git commit -m "…"
    ```

    Check the path list first: a format pass can legitimately change other fixtures,
    and the `grep` above would hide a protected fixture only because all fifteen stay
    modified. One commit per coherent step, message in plain prose, what and why.

13. **Update the acceptance map and the current handoff in the same commit.** Move the
    row from "Diagnosed" to "Supported" with the test name.

14. **Refresh consumers** from the accepted commit by copy to `.refresh-tmp` and `mv -f`,
    then run one smoke program with each `tsrust` binary.

## 5. What is next

Five rows remain. Suggested order and what is known about each.

### 5.1 Borrowed views of nullable capturing records (closes two rows)

Rows: a parameter typed `ReadOnly<Record> | undefined`, and
`const saved = boxes[0].saved`.

Root cause: data records get a `Type::BorrowedUnion` view in these positions; a
nullable capturing record gets an owned `Type::Union`, so the value would be moved.

Evidence:

- For the parameter, the argument is `ExprKind::Inject` with type `Type::Union(declared)`
  and an inner value of the caller's concrete union. Emission then reaches the owned
  widening path in `emit.rs::union_injection` and moves the caller's variable
  ("use of partially moved value"). The injection is built near
  `analysis.rs` `scoped_record_injection` call sites (search `borrowing_callback`).
- For the element field, emission panicked with "indexed storage cannot move"; a source
  diagnostic in `reject_unplanned_nullable_field_stores` now guards it.
- `function use(p: ReadOnly<P> | undefined)` with a data record `P` is the working
  reference. Compare its checked argument and parameter types with the capturing case.

Probes to start from:

```
type Holder={run:ReadOnly<Function<int>>;};
function use(h:ReadOnly<Holder>|undefined):int{if(h!==undefined)return h.run();return 0;}
function main():void{const a=[3];let h:Holder|undefined=undefined;console.log(use(h));h={run:():int=>a[0]};console.log(use(h));}
```

```
type Holder={run:ReadOnly<Function<int>>;};type Box={saved:Holder|undefined;};
function main():void{const a=[3];const boxes:Box[]=[{saved:undefined}];boxes[0].saved={run:():int=>a[0]};const s=boxes[0].saved;if(s!==undefined)console.log(s.run());}
```

When the parameter works, relax `scoped_record_containers.rs` (`viewed_argument` and
`parameter_bound_nullable_record`) and remove the element-field diagnostic.

### 5.2 Storing a new callback record through a `Mutable<Record>` parameter

Clearing already works. Storing needs the caller's environment to include the callee's
callback. The mechanism exists for plain callback fields:
`record_parameter_carriers.rs` (`Replacement { parameter, path, ty }`),
`record_parameter_replacement_emit.rs` (`From<…>` bounds and conversion), and
`program.record_parameter_replacements`. It must learn a path that passes through a
nullable payload (`saved` then `run`), widen the caller's union payload, add a
`C{id}: From<Closure…>` bound, and emit the conversion inside the record literal. This
overlaps ledger row B7; plan them together. A stored callback must own its captures;
the existing `callback-escape` checks in `record_parameter_carriers.rs` state the rule.

### 5.3 Two parameters with the same nullable callback record type

Needs a generic name per parameter leaf instead of `C{contract id}`. The name is
produced in `emit.rs::native_type` and used by every native type printed inside the
callee, so the change is not local. Lowest priority.

### 5.4 Union of a capturing record and another record type

`Holder | Other`. No narrowing story was established for it. Ask the owner whether it
is wanted before building it.

### 5.5 Closing B5

When the rows above are supported or the owner removes them from scope:

- run both whole crates;
- run the timing protocol for `scoped_record_composition`, review it against the Node
  counterpart, and only then set `measured: true` (see `AGENTS.md` on milestones);
- update `docs/compiler-completion-plan.md` row B5 and the roadmap;
- refresh consumers, prune obsolete build outputs, and write the acceptance handoff.

### 5.6 Not part of B5

- Structural record keys for data records: policy decided (section 2), not started.
  Give it its own ledger row.
- Ledger rows B7 to B13 and D2 to D5 are untouched.
- Arbitrary comma final operands beyond the cases in the array storage tests.

## 6. Reporting

- State counts as measured: tests passed, failed, ignored. Say which suites ran.
- Separate "supported", "diagnosed" and "not started". Name every remaining row.
- Say when a check was not run.
- The owner uses a terse chat style; documents, comments and commit messages stay in
  normal prose.
