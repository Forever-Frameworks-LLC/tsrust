# Gate 170: nested function declaration foundation

Status: implementation in progress; capture-free native foundation works. Not accepted.

The complete scope and owner capture decision are in
[nested function declarations plan](../nested-function-declarations-plan.md).
Gate 170 covers lexical declaration representation, hoisting, scope, capture-free
recursion and synchronous first-class use. Subsequent capture, escape and signature
gates are required to finish the full feature; their witnesses remain explicit.

The hoisting fixture exercises forward use, factorial recursion, even/odd mutual
recursion, inner block shadowing and use as an Array.map callback. Other fixtures
cover later initialized captures, mutable borrowed state, returned counters and
local generic/default/rest signatures. Pinned Node reference runs pass all three.
The focused compiler run is expected red at the existing statement syntax diagnostic.
See the plan for exact compiler/editor/benchmark acceptance gates and dependency
seams. No timing, consumer refresh or broad regression is part of this red checkpoint.

A preparatory source chunk shares top-level named declaration parsing through the
non-inlined named_function_syntax helper. Statement-consumer obligations are listed
in the full plan. It does not yet add nested declaration syntax. A rest-call depth
failure found during focused checking is repaired by separating generic and
conditional preparation state from ordinary declared call frames, without changing
worker-stack or source-depth limits. Feature acceptance remains pending.

Preparatory checks pass: 137 affected compiler checks, including all eight depth
canaries and generic/optional/rest contracts, and all 267 editor checks. The direct
declared-call dispatch frame is now 288 bytes versus its former 4,672 bytes. Detailed
failure, LLDB, compiled-frame and successful check logs are retained in SQLite under
target/editor/nested-function-parser-* as gate-log records, verified by exact
read-back. Five consumers are atomically refreshed and hash-verified. Generated
example and scratch test executables are pruned; libraries remain cached and tests
will relink. No timing or full compiler regression was performed. Nested declaration
implementation and its matched benchmarks remain pending; gate 170 is not accepted.

The current lexical implementation adds a real LocalFunction AST node and a dedicated
local_functions resolver before checking and module linking. It registers declarations
before uses, resolves shadowing, diagnoses conflicts, lifts capture-free functions for
static execution and retains declaration identities and source spans through linking
and checked program construction. Four focused checks pass: the hoisting/recursive
fixture, generic/default/rest fixture, scope/conflict/local-main checks and closure
parameter shadowing. The capture witness remains genuinely red at an explicit capture
use/declaration diagnostic. A raw-parser depth witness exposed signature temporaries
retained across recursive bodies; header parsing now ends before body recursion and
the original 128-level source limit is preserved. See nested-functions.md for current
behavior. Editor acceptance and matched benchmark support remain pending.
