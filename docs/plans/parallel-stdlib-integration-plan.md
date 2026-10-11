# Standard-library worker: integrate accepted work into main

## Goal

Finish the current path joining and endianness checkpoint, then integrate the complete accepted standard-library series into main, including canonical evidence and executable consumers. The owner now explicitly requests integration; this supersedes the earlier worker prompts that prohibited merging into main. Branch acceptance alone is not completion of this goal.

Read AGENTS.md, docs/development-workflow.md, the lexical-path, OS-identity and path-join handoffs, and the latest main callable handoff. Reinspect both worktrees and their HEADs before starting. The observed worker tip is 7713745, with coverage and the path-join handoff still uncommitted. Finish the active measurement/evidence work before integration. Never discard its completed results or rerun successful groups merely for presentation.

## Exclusive integration ownership

The worker may own main integration after receiving this prompt. The main agent will leave compiler source work at its checkpoint while integration runs. Only one agent may edit main or write its SQLite database at a time. Confirm the current main checkpoint and protected changes at takeover. Do not merge while another agent has resumed source writes; request handoff if ownership is unclear. Do not reset, stash, copy or commit the fifteen protected owner fixture edits in either tree. Keep their exact bytes and index state intact.

Do conflict resolution in the isolated worker tree. First integrate the current main commit into the worker branch, preserving the complete standard-library history and main's compiler work. Resolve additive overlaps narrowly, especially analysis.rs, emit.rs, syntax.rs, error_effects.rs, native bindings, catalog, tests and benchmark registrations. Preserve retained checked invocation IDs, independent declaration preparation and invocation failure propagation. Do not simplify either implementation to resolve conflicts. Format the entire project before source checkpoints, preserving protected owner edits and proving tokens and idempotence. Use explicit Git staging.

## Combined acceptance

Verify all accepted standard-library APIs: path dirname, extname, isAbsolute, normalize and join; OS platform, arch and endianness. Cover native imports, aliases, stored calls, variadic evaluation/borrowing/spread, errors, documented examples and editor signatures. Run affected registration/source-contract checks and all new or changed workloads untimed against native and Node-after-tsc. Preserve branch measurements and their original source/toolchain identities; combined correctness checks do not rewrite historical timing.

Run all eight depth canaries after shared analyzer/dispatch conflict resolution. At the combined library milestone run the complete compiler and LSP crates, preserving completed groups and repairing/rerunning only affected failures. Classify against both branch and main evidence. Main currently retains two distinct-declaration replacement integration witnesses and one mutable-factory library witness; preserve intentional witnesses and identify them explicitly. Do not hide new failures as baseline. Repair integration regressions and coordinate a genuine unrelated compiler dependency rather than silently weakening tests.

Actual native coverage remains macOS ARM64 unless another platform really executes. Windows lexical helper checks and cross-target metadata checks must remain separately described. OS constant identity workloads remain untimed. Do not retime prior accepted path workloads without a demonstrated changed measurement requirement.

## Publish source, evidence and consumers

Once combined checks pass with explicitly classified intentional witnesses, commit the integrated branch. Fast-forward main to that verified branch tip when possible. If main has moved, stop main mutation, refresh the exclusive handoff and integrate that new checkpoint in the isolated tree before publishing. Never force-update main, reset owner edits or merge across active source writers. Source integration is explicitly authorized by this goal; no additional owner confirmation is required for the routine merge.

Serialize SQLite integration. Export branch records with repository storage tools into an empty temporary root; preserve logical identities, kinds, immutable bytes and original analytics rows. Compare existing bindings, deduplicate identical bytes and preserve differing bytes under explicit new identities. Never replace main records.sqlite with branch records.sqlite. Import each record, read back byte for byte, verify analytics/sample counts and database integrity, then remove verified loose exports. Store combined verification evidence in main SQLite too.

Rebuild the five main executable consumers from the integrated commit using the documented main target and CARGO_INCREMENTAL=0. Copy to sibling .refresh-tmp files, then mv -f atomically. Finish refresh before checks execute them. Smoke-test the refreshed CLI and editor consumers with imported library fixtures. Reinspect source status and ensure only protected owner edits remain. Prune obsolete scratch outputs without rebuilding merely to warm caches.

## Deliverable and stopping point

Write an integration handoff listing branch/main commits, conflict decisions, exact checks, preserved intentional failures, native platform limits, measurement boundaries, merged SQLite identities/counts and refreshed consumers. State plainly whether source, evidence and consumers are integrated. Stop after this integrated checkpoint; propose the next bounded API goal afterward. Do not begin filesystem resources, JSON, databases or networking in this goal.

Use Node 24.11.1 first on PATH and /opt/homebrew/bin/python3 for store.py. Keep the worker Cargo target during isolated checks and the documented main target for main consumer rebuilding; both disable incremental compilation.

## Worker prompt

Finish your active path join/endianness checkpoint, then complete docs/plans/parallel-stdlib-integration-plan.md from the main tree. Integration into main is now explicitly authorized and required. Take exclusive main integration ownership; preserve fifteen owner fixture edits exactly. Integrate current main into your isolated branch, resolve overlaps without losing compiler or stdlib behavior, verify combined compiler/editor/native/benchmark contracts, then publish the verified branch to main. Merge immutable SQLite evidence through storage tools, refresh all five consumers atomically from the integrated commit, verify their behavior, and provide a committed integration handoff. Do not stop at branch acceptance. Do not start another API family before integration is complete.
