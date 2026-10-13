# Nested call argument compilation

Run the shared compilation driver with `--call-arguments declared`, `bound`,
`inline` or `field`. The selected form has matched Tsrust and TypeScript sources,
a constant independent output oracle (`7\n`) and separate source hashes. The
TypeScript field signature represents the same pure call; it does not claim
Tsrust ownership checking equivalence. Both sources invoke the same number of
identity operations in the same nesting order.

For example, run `python3 benchmarks/compiler/function-dependencies/run.py
--call-arguments field --only 8 32 --output NEW_JSON` from the repository root.
This validates formatting idempotence, compiler checking, Rust emission and
Node/TypeScript strict checking without recording durations. Pass `--measure`
only for an approved milestone run; the original protocol remains two warmups
and eleven samples per selected phase and count. Preserve original records in
SQLite with exact-byte restoration verification.

Registered counts are measurement samples, not language limits or assertions of
support. Sizes 128 and 512 remain pending while parser, checker and emitter
continuations are completed. The discovery driver grows inputs geometrically
without a maximum depth and records actual failures on normal worker stacks.
Current discovery completes parse, analysis, emission and disposal at size 64 in
all four forms after callable emission moves to a work stack. All four encounter
the existing parser expression guard at 128. Those samples do not establish a
support ceiling or completed scaling; parser and remaining cross-phase recursion
still require migration before whole-pipeline nesting acceptance.
