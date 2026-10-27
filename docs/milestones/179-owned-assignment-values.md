# Gate 179: assignment as a value for strings, arrays and records

Status: accepted. Requirement B15 of
[the compiler completion ledger](../plans/compiler-completion-plan.md). The rule was
stated to the owner in the gate 178 handoff; it was not separately confirmed and is
the one design choice of this gate to review.

## Behavior

`(target = value)` with a string, array or record target has the target itself as
its value, read in place after the store. No copy is made. Member access, method
calls and read-only arguments leave the target usable; binding the value moves the
target, as naming the target would. The target is a `let` variable or a record
field with a fixed path. The [guide](../guides/language/assignment-values.md) describes it.

## Implementation

- `Analyzer::owned_assignment_value` rewrites such an assignment into the existing
  comma form `(target = value, target)` before it is checked. The store uses the
  accepted discarded-store path; the read is an ordinary place read.
- `Analyzer::owned_assignment_projection` rewrites `(target = value).field` for a
  record target into `(target = value, target.field)`, so the record stays in place
  instead of moving into a temporary.
- Both run in `expression_other`, before sequence dispatch; the thin dispatcher frame
  is unchanged.
- Array elements and other computed places keep an `unsupported` diagnostic for
  owned values, because the place would be evaluated twice. Scalars are unchanged.

## Evidence

- `owned_assignment_values`: native fixture (string, array, record, nested field
  target, read-only argument, loop condition, field of a field) and five rejection
  rows, plus the scalar chain. Formatter-stable; passes the pinned Node reference.
- Bounded compiler batch with unit tests, all eight depth canaries, comma, compound
  assignment, field move and formatting suites: 692 pass, 0 fail.
- Whole editor crate: 365 pass, including member completion and hover on the value
  and the diagnostic for a `const` target.
- Matched `owned_assignment_values` workload at the end of the manifest with oracle,
  token-based source contract, registration test and hooks; native and
  Node-after-tsc agree on five untimed cases. No timing.
- The guide example runs natively and passes the Node reference.
- Correction after the whole-crate run of gate 180: `assignment_values` and
  `logical_assignment` each still expected `unsupported` for
  `console.log(text = "new")`. Both rows are removed and the behavior is a positive
  witness in `owned_assignment_values`. The gate 179 bounded batch did not include
  these two suites.
