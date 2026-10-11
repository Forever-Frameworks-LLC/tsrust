# Gate 170: nested function declaration foundation

Status: accepted lexical and capture-free foundation. Captures, escape and comprehensive result inference remain subsequent gates in the [full plan](../plans/nested-function-declarations-plan.md).

Nested named declarations are block scoped and hoisted. Capture-free forward calls, self and mutual recursion, inner shadowing and synchronous callable use execute natively. Basic generic, default and rest signatures reuse declared-call support. Real AST declarations retain stable source identities through lifting, specialization and project linking. Local main is an ordinary function.

Checked single-file and project APIs expose local declarations with original names, source locations and checked result types. Completion, hover, definition, references, signature help and hierarchical document symbols preserve lexical visibility and imported nominal result identity. A future block binding masks an outer declaration without premature completion. This revealed and repaired virtual scope isolation for unbraced loops. Formatting preserves nested blocks and blank lines.

Public checking of 100 nested declarations passes on the original worker stack. The preceding parser chunk also verified the existing 128-level source bound and repaired retained parser header state; ordinary rest-call dispatch frames were reduced without increasing stack or source limits.

Acceptance evidence: five focused compiler checks and all 16 formatting checks pass; the entire editor crate passes 271 tests, followed by a stronger project nominal-identity assertion checked separately. Harness and report checks pass 17 tests. The preceding lexical checkpoint retained 121 affected compiler checks including all eight depth canaries, plus three parser/metadata checks. No parser/analyzer dispatch changed in this editor chunk.

The matched nested_function_declarations workload retains six scalar observations per scenario for forward calls, recursion, shadowing and callbacks. Native and Node-after-tsc agree with the independent oracle on all five untimed verification cases. Source contracts, ABI registration, driver/profile/report hooks, manifest and coverage registrations are complete. No workload timing or full compiler regression was performed.

The borrowed/escaping capture witness remains genuinely red for gate 171/172. A new independent Node-positive omitted-result witness is red because unannotated named declarations currently mean void; comprehensive inference remains gate 173. These are explicit staging boundaries, not accepted unsupported behavior or completion of the full nested-function plan.

Detailed successful and failed logs are canonical SQLite gate-log records under target/editor/nested-function-editor-*, with earlier parser/lexical records retained. The exact runtime-result is benchmarks/runtime/results/2026-10-06T16-36-15-075Z-verify.json. Its provisional manifest snapshot predates acceptance flags; source hashes and verified behavior are unchanged. Every record was read back byte for byte before its loose copy was deleted. Five executable consumers are atomically refreshed and hash verified. Coverage contains 1,956 operations across 28 families; this family is implemented and verified, unmeasured.
