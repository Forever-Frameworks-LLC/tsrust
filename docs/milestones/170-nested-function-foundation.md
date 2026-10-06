# Gate 170: nested function declaration foundation

Status: planned; red witnesses added. No implementation or acceptance yet.

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
