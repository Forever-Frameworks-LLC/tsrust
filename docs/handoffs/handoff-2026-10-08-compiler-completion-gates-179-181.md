# Handoff: compiler completion, gates 179 to 181

Date: 2026-10-08. Previous handoff: `docs/handoffs/handoff-2026-10-08-compiler-completion-gates-175-178.md`.
Plan and ledger: `docs/plans/compiler-completion-plan.md`.

## State of the tree

- Gate 180 is `6c993c9`. Later commits hold this handoff and the formatter repair. Gates 179 (`68504c0`) and 180 are accepted and committed with
  milestone records, coverage, SQLite evidence and matched workloads.
- Gate 181 (truthiness, ledger row D1) is **in progress and uncommitted**. Three source files hold
  the work: `crates/tsrust-compiler/src/analysis.rs`, `emit.rs` and `logical_values_emit.rs`.
  Nothing else belonging to gate 181 exists yet: no tests, fixture, editor witness, documentation,
  workload, milestone record or coverage change.
- The fifteen protected owner fixtures under `crates/tsrust-compiler/tests/fixtures/` remain
  modified, unstaged and untouched.
- The five consumers were last refreshed from `e1fb13c`. They do not contain gates 179 and 180.
  Refresh them from an accepted commit once gate 181 is accepted or its source changes are set
  aside.
- The formatter now attaches a unary sign to its operand (`-1`, `[-2, +3]`, `-(a + 1)`,
  `i++ + -a`). The owner chose this on 2026-10-08. The earlier `- 1` output was a formatter defect,
  not a decided style. The change is in `crates/tsrust-compiler/src/formatting.rs` with the witness
  `unary_signs_attach_to_their_operands` in `tests/formatting.rs`. The project format pass rewrote
  308 files with code tokens proven unchanged and refreshed 147 benchmark fingerprints. Checks: the
  formatting suite (26), the eight depth canaries, the editor crate (365) and a native plus Node
  verification of two reformatted workloads all pass.
- Two protected owner fixtures still hold the old spacing because they were kept byte-identical:
  `array-callback-errors/values.tsr` and `map-for-each/values.tsr`. `node tools/format-project.mjs`
  will rewrite them on its next run. Copy them aside first and restore them, or ask the owner to
  let the formatter touch them.
- The session ran no whole-crate regression after the gate 181 edits. The last whole compiler run
  was at gate 180: 2,362 passed, 0 failed, 3 ignored. The editor crate passed 365.

## Gate 181: what is implemented

Owner decision: truthiness is in scope for conditions and logical operators. Loose equality, `var`,
`with`, array holes, writable `length` and `delete` stay excluded.

- `analysis.rs::truthy_condition` runs before every condition check (`if`, `while`, `do`, `for`,
  conditional expression). A boolean or void condition is unchanged. A nullable fixed-path place
  becomes `place !== null && place !== undefined && !!place`, which keeps narrowing. Any other
  condition becomes `!!condition`. `&&` and `||` conditions recurse on the left operand.
- Unary `!` accepts a non-boolean operand. `logical_values_emit.rs::logical_not` emits it and
  `emit.rs` routes to it when the operand type is not boolean, null or undefined. A call without a
  return value is rejected: "A function call without a return value has no truth value."
- `analysis.rs::truthy_logical_assignment` rewrites `target ||= value` and `target &&= value` on a
  non-boolean fixed-path target into `target = target || value` (or `&&`). It runs at the top of
  `expression_other`. The `LogicalAssign` type hint now follows the target type.

Native probes that pass: `!s`, `!n`, `!!s`, NaN, `if (s)`, `if (x)` on `int | null` with narrowing,
conditional expressions, `while (n)`, `do`/`while`, `for (;i;)`, `s && n`, `"" || n`, records and
arrays as conditions, optional parameters, nullable fields, `!x` with narrowing in the else branch,
`u && u.name`, early return on `!x`, `boolean | null`, and `||=`/`&&=` on `int` and `boolean`
targets, both as a statement and as a value.

## Gate 181: known defect, fix first

`||=` and `&&=` on an owned target fail. The rewrite reads the target as the left operand of the
selection, and that read transfers the owned value:

```
let s = ""; s ||= "d";          // error[transferred]: "s" is no longer available
let y: string | undefined = undefined; y ||= "set";   // same
p.name ||= "anon";              // error[partial-transfer] on record field
```

The rewrite must not move the target. Emit the test as a borrowed truthiness read and assign only
when the test selects the right operand: `if (!target) target = value` for `||=` and
`if (target) target = value` for `&&=`, with the expression form yielding the target read in place
(the gate 179 rule). Scalar targets already work and must keep working.

## Gate 181: remaining acceptance work

1. Fix the owned-target defect above.
2. Add `tests/truthiness.rs` and a fixture directory `fixtures/truthiness/`, and compare with the
   pinned Node reference.
3. Existing suites hold rows that expect "Expected boolean, found ..." rejections. They are now
   stale. Check each natively before converting it to a positive witness. Likely suites: operators,
   `logical_assignment`, control flow, conditional expressions.
4. Add an editor witness in `crates/tsrust-lsp/tests/intelligence.rs`.
5. Update `docs/guides/language/logical-operators.md`. It still states that conditions and logical assignment
   require booleans.
6. Add the matched workload, coverage rows `boolean.truthinessConversion` and
   `control.logicalAssignment`, the milestone record `docs/milestones/181-truthiness.md`, the ledger
   row D1 and the roadmap entry.
7. Run the bounded checks with the eight depth canaries and the whole editor crate. Format the
   project, import evidence to SQLite, and commit with explicit paths.

## Remaining ledger

Accepted: B1, B2, B3, B4, B6, B15.

Not started, in the order the owner chose:

- B5: scoped records in arrays, Maps, Sets and unions; nullable replacement.
- B7: callbacks stored through a record parameter that borrow from the caller.
- B8: generic adapter compile time. `generic-state-join-efficiency.md` still reads "in progress".
- B9: S04 project editor. B10: S06 `Result` and exhaustiveness. B11: S07 resources and bytes.
- B12: thread prerequisites. B13: async compiler.
- D2: type-level features. D3: classes and `this`. D4: generators and iterators. D5: BigInt and
  Symbol.

The compiler is not complete. The final requirement-by-requirement audit has not been done.

## Decisions the owner still owes

- B14: `switch` fallthrough.
- Whether the format pass may rewrite the two protected fixtures that still hold `- 1` spacing.
- Gate 179 rule: the value of an owned assignment is the target read in place. It was implemented
  on recommendation and not separately confirmed.
- Not in scope until decided: decorators, tagged templates, regular expression literals, computed
  property names, object identity equality, and the URI, `globalThis`, JSON and Date globals.

## Working rules to keep

Follow `AGENTS.md` and `docs/development-workflow.md`. Stage explicit paths only. Never stage the
protected fixtures. Use the shared Cargo target `/private/tmp/tsrust-gate100-editor-target` with
`CARGO_INCREMENTAL=0`, `--offline --locked`, Node 24.11.1 ahead of Homebrew Node on `PATH`, and
`/opt/homebrew/bin/python3` for `benchmarks/storage/store.py`.
