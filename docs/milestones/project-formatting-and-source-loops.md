# Source loops and automatic project formatting

The source-loop checkpoint preserves finite generic callable contracts through while, for, do, for-of, updates, labeled control, break and continue. Definite assignments narrow the possible targets; saved aliases retain their earlier contracts. Literal zero-iteration and single-element controls are covered. Exception regions, expression writes and capture effects remain conservative. Full Gate173 is not accepted.

Project formatting is now mandatory before every source checkpoint. Run `node tools/format-project.mjs` with `TSRUST` pointing to the current verified compiler. The command formats the entire project and refreshes exact benchmark source hashes only after proving token preservation against pinned pre-format bytes or the committed baseline. Comments, literal contents, template chunks and operators are preserved. Semantic changes require ordinary acceptance.

The project-wide proof covers 1,060 existing changed files with identical tokens, including comments. Fifteen protected owner fixtures remain uncommitted. The Node-only BigInt reference now has a `.ts` extension so recursive Tsrust formatting no longer attempts to parse it. Its output matches the retained numeric literal reference.

Formatting exposed layout-sensitive benchmark and editor checks. Token-based benchmark checks now tolerate layout while preserving mutation rejection, exact source hashes, independent oracles and operation/sample counts. Three historical manifest prefix checks freeze the protocol projection rather than raw source hashes. All 540 refreshed source fingerprints are layout-only; no timing is claimed.

Focused evidence includes 171 compiler tests, all eight depth canaries within that count, the formatted fixture corpus, 322 editor tests across the preserved original and repaired/resumed runs, 132 benchmark registration/helper tests and seven final affected helper tests. The matched generic adapter family verifies native and Node-after-tsc results untimed. Original failures and repaired checks are retained in SQLite rather than erased.
