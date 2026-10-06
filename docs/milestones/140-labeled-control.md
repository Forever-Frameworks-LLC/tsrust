# Gate 140: labeled statements and loop control

Status: focused native/compiler, editor, formatter, guide, refreshed-consumer and matched benchmark acceptance complete. Full regressions and measurements remain deferred.

## Behavior

A label names an enclosing statement. `break name;` leaves it. `continue name;` targets an enclosing loop, running its update or post-test condition where applicable. Blocks, conditional statements, empty statements and loops support labels; several labels can name one statement. Labels have a separate namespace from values and cannot cross a function or closure boundary. Duplicate active names are rejected. A statement supports at most 32 labels.

Ordinary ownership rules still apply: an outer owner must be reinitialized before continuing its loop after a transfer. An outer break retains the transferred state at its destination. Loop keys and elements retain their existing access. No implicit clone was added.

See [the guide](../labeled-control.md) for examples, target scope and diagnostics.

## Implementation

- `Stmt` retains label declaration/reference tokens and a checked `ControlTarget`. Parser helpers collect label aliases without recursing for each alias. Classic-for labels attach to the lowered loop rather than its initializer block. Non-loop labeled statements use a labeled block. Switch clause lowering keeps labeled breaks separate from unnamed clause-ending breaks.
- Analyzer loop frames retain labels and their native target kind. `labeled_control.rs` owns target resolution, duplicate checks, labeled-block joins and targeted availability/flow snapshots outside the recursive dispatch frame. Unnamed control skips labeled-block frames and still targets the nearest loop. Outer continue states are stored on the target frame, so the target update and ownership checks use the actual continuing state.
- Native emission labels while/for-of loops and labeled blocks. A tail-loop continue exits its body block, allowing the target classic-for update or do-while condition to execute; intervening loop updates are skipped.
- Checked label facts expose declaration/reference identities through source and project analysis. Editor hover, definition, references and highlights use a label namespace distinct from values. Project linking relocates label token spans without renaming label text.

## Defects repaired during implementation

The original definite-assignment walker treated every break/continue as leaving the remaining function statements. A break leaving a labeled block could therefore suppress checks of later unassigned reads. It now retains targeted exits and their assignment state until the enclosing statement consumes them. Conditional exits and ordinary fallthrough are joined separately.

A continue could also skip assignment checks in the target loop's update or do-while condition. Continuing paths now join before checking that tail. Red witnesses for both defects are retained; unassigned uses report frontend errors rather than reaching the native compiler. The existing deferred-initialization suite remains passing.

Project references initially omitted labels and could mix a same-named value reference into the label namespace. Checked reference locations now separate them. A focused red project witness and repaired source/project reference checks are retained.

## Updated expectations and fixture repairs

`loops` previously expected `break outer;` with no matching label to be `syntax`; it is now `loop-control`. An initial owned-transfer witness used a function that never consumed its parameter; it was corrected to return the parameter. The post-break diagnostic expectation is `transferred`, matching the actual rejected operation. A project fixture was corrected to import a supported exported function instead of an unsupported exported constant. These are fixture corrections, not additional language features.

## Evidence

- Exact snapshots: `archive/labeled-control-start`, `archive/labeled-control-benchmark-start`, `archive/labeled-control-consumers-start`, `archive/labeled-control-records-start`. Directory paths preserve same-named compiler/editor files separately.
- `target/editor/labeled-control-red.log`: initial compiler failures. Assignment, continue-assignment, editor and project-reference red logs remain alongside repaired evidence.
- `labeled-control-current.log`, `labeled-control-edge.log` and `labeled-control-final-native.log`: eight focused compiler suites account for 60 passing checks across retained passes and affected supplements: labeled control3, deferred initialization2, loops14, control flow11, higher-order function values6, type syntax8, switch2 and formatting14. The label contract test contains thirteen static rejection rows; a separate test checks rejection of 33 aliases and acceptance of 32.
- `labeled-control-editor-references.log` and `labeled-control-project-editor-references.log`: two focused editor checks pass, including hover, definition, references, same-named value separation and linked project locations. No complete editor suite was run.
- `labeled-control-final-node.json`, `labeled-control-guide-node.json`, `labeled-control-guide-native.log`: executable fixture and guide assertions pass independently in Node and natively with empty stdout.
- Debug compiler/LSP and release builds pass. Five consumers were atomically replaced after preserving gate139 consumers and before execution checks.
- `labeled-control-harness.log`: 363/363 checks pass in one run.
- `benchmarks/runtime/results/2026-10-05T13-46-27-781Z-verify.json`: `labeled_control`, `deferred_initialization` and `classic_for_control` verify untimed for native and Node-after-tsc. An earlier invocation contained an unknown placeholder workload ID and was rejected before preparation; the corrected selection and original log are preserved.
- The matched `labeled_control` pair retains seven complete observed cells, an independent formula oracle, source contracts, registrations, runtime/profiling/report hooks, current hashes and the string[][] ABI. One complete scenario is the operation denominator. Measurements remain pending.

Coverage uses the existing `control.labelledBreak` and `control.labelledContinue` identities. No full compiler/LSP regression, timing or SQLite measurement was performed.
