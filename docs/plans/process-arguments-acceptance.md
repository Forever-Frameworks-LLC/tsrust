# Process arguments acceptance

`std:env.args(): Result<string[], EnvError>` returns fresh owned arguments in order,
excluding the executable name. Empty values and Unicode are preserved. Direct
native invocation with a non-Unicode OS argument returns `NotUnicode` without
including the argument in the error. The CLI forwarding path accepts Unicode
arguments. No raw-byte argument API is claimed.

The environment runtime module is emitted once for programs using both `get` and
`args`. Both functions have explicit native error-effect and observer contracts.
The editor catalog documents behavior, errors, examples and ownership.

Focused acceptance on 2026-10-18:

- Existing three environment checks pass; two new native process-argument checks
  pass after correcting an optional-value fixture and registering error effects.
- Process-argument hover and environment import/signature checks pass.
- Three completion matrix checks pass, including the new row in all four modes.
- All environment catalog examples compile and execute at O0 and O3.
- Four independent argument vectors pass on native and Node-after-tsc, before and
  after whole-project formatting. Source hashes are recorded. These are untimed
  process-boundary witnesses, not throughput measurements.
- Whole-project formatting and Git whitespace checks pass.

Closed evidence uses `tsrust-process-arguments-*` in canonical SQLite, including
original failure and repaired results. No complete regression, SQL suite or timing
was rerun. Live editor restart/visual acceptance remains unverified because this
session has no editor UI control.
