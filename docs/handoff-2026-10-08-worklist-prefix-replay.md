# Handoff: worklist prefix replay isolated

Production behavior remains accepted at 6cc985a. This checkpoint adds test-only invocation counting and an intentionally failing scheduling budget. Do not describe the gate as accepted. Fifteen owner fixtures remain uncommitted and protected.

For 8/16/32/64 independent operator calls, evaluations are 17/33/65/129 and invocation visits are 44/152/560/2144. Requests grow linearly while completed call prefixes are replayed quadratically. The red witness fails at eight calls with 17 evaluations against a budget of ten. Existing immediate identity-return handling does not cover return value+1. See milestones/generic-state-join-efficiency.md for the exact next implementation and acceptance scope.

Continue by extracting a conservative invocation-free scalar-return eligibility helper and using the existing contextual evaluator. Preserve errors, capture effects and ownership. Add negative controls before broadening eager scheduling. Do not reintroduce the rejected scalar-state equality shortcut. Test-only counter instrumentation has no production binary effect. Installed consumers remain accepted; warm release still contains the rejected prior experiment and must be rebuilt before further acceptance.

Read AGENTS.md and docs/development-workflow.md. Use explicit staging, Node 24.11.1, /opt/homebrew/bin/python3, one Cargo target and CARGO_INCREMENTAL=0. Closed diagnostics are stored in SQLite under tsrust-worklist-* identities. Parallel std:path.relative publication requires exclusive main ownership and reconciliation against the latest checkpoint. Give a physical handoff after the implementation gate.
