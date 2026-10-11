# B7 native callback payload alternatives

B7/gate 183 remains in progress. This checkpoint continues `8b96e05`. B5 remains
accepted at `00214a11`. Read AGENTS.md, docs/development-workflow.md,
docs/plans/caller-borrowed-record-stores-plan.md and the preceding B7 handoffs first.
Discriminated unions remain a separate required D2 feature. No owner decision is pending.

## Implemented in this checkpoint

Both earlier replacement witnesses now pass: the same installer can replace a
holder's borrowed callback wrapper with a callback having a different native capture
type. Readonly and mutable inputs work. Mutable callback state survives holder
disposal, and the caller can invoke the original callbacks afterward.

The new `borrowed_payload_emit.rs` separates native payload alternatives from source
origins. It follows complete input vectors across named calls and stable aliases,
with an iterative worklist and a visited set. Captures with matching source signatures
keep independent positions. Complete vectors are deduplicated and sorted; the plan
never constructs a Cartesian product of unrelated positions. A unit witness checks
correlation, deterministic output and termination in a call graph containing recursion.
A simple recursive retained-store relay also passes native execution. This does not
establish acceptance for every recursive retained-store summary.

Borrowed-family definitions, inferred local family types, concrete conversions,
dispatch, record-family conversion and record-call reification use the expanded
payload alternatives. Several native arms can invoke the same source origin. Concrete
capture types in conversion implementations avoid overlapping generic Rust
implementations. Source callable contracts, source origins and capture loans are
unchanged. There is no boxing, dynamic dispatch, implicit clone or lifetime cast.

## Verification

Closed logs use `tsrust-b7-payload-` keys in benchmarks/records.sqlite. Failed checks
remain preserved alongside their repaired checks.

- `final-batch.log`: six replacement/representation tests, seventeen caller-store
  tests and nine existing record-parameter replacement tests pass. The outstanding
  holder-capture witness is explicitly excluded from this positive batch.
- `plan-final.log`: the isolated payload planner test passes on the final source.
- `editor-repair.log`: declared hover and parameter help survive replacement, and
  moving the currently retained captured callback reports capture-conflict. The
  original conflict control referred to the replaced callback and was corrected.
- `recursive-store.log`: a terminal-branch recursive relay stores and replaces
  different input environments and passes native execution.
- `holder-capture-red.log`: one intentional source witness still fails in generated
  Rust, as described below.
- `final-registration.log`: independent arithmetic oracle, fourteen observations per
  iteration, source hashes, readonly and mutable replacement mutation guards and execution hooks pass.
- `final-format.log`: whole-project formatting, fingerprint
  preservation and idempotence checks; fifteen unrelated fixtures are preserved.

The expanded matched workload adds readonly and mutable replacement and swapped
callback capture positions. `runtime/results/2026-10-09T17-50-08-166Z-verify.json` verifies five cases
on each of native and Node-after-tsc. It has no execution, startup or compilation
samples. The workload remains measured false and implementationAccepted false.
Previous eleven- and thirteen-observation and current fourteen-observation sources and oracle/guard
bytes are preserved under content-addressed SQLite source-evidence keys. Changed
compiler source bytes at verification are also preserved under their exact recorded
hashes, including the pre-alias-cleanup versions. Exact bytes were read back before
removing the loose result. No complete regressions or timing ran in this slice.

## Next work

First fix `capturing_a_replaced_holder_preserves_its_actual_record_carrier` in
`caller_borrowed_callback_replacement.rs`. It installs two callbacks, then captures
the holder in another closure. Source checking succeeds. Native capture layout still
expects the holder's initial capture-free field type, while the actual field is an
expanded BorrowedCallable carrier. Do not change the valid program into an unsupported
diagnostic. Investigate final record carrier selection and lifetime accounting in
`scoped_record_capture_views.rs`, `scoped_record_emit.rs` and `callable_layout.rs`.
Native named-record rendering still assumes one payload per source origin. Preserve
source point types separately from final storage types when repairing that assumption.

The payload planner currently resolves concrete closure inputs whose captures do not
themselves contain abstract callback inputs. Nested generic capture environments need
recursive payload rendering and provenance transport; their previous generic path
remains, and repeated alternatives are not accepted yet. Recursive retained-store
summaries also remain unfinished. A simple recursive relay passes in
`recursive-store.log`; an earlier multi-input aliased recursive attempt encountered
capture-conflict before backend planning, preserved in `plan.log`.

Then continue the original B7 requirements: direct borrowed callback assignment,
nullable stores, mutable data aliases, nested forwarding, finite retained-store
summaries, failure before/after stores, exact replacement and clearing, branches,
loops, early exits and disposal. Extend matched support as those forms land. Complete
the family before full compiler/LSP regressions, timing and performance review.

## Workspace and consumers

The fifteen unrelated fixture edits remain untouched and uncommitted. Use explicit
Git paths. Private compiler builds include this progress; the five installed consumers
remain at accepted B5. Refresh those consumers atomically only after family acceptance.

Cleanup removed 245 obsolete Cargo test executables (7,008,126,320 bytes) and five
closed caller-store runtime artifacts. Current focused binaries, compiler/LSP binaries,
shared libraries and installed consumers remain. Unrelated test suites will relink
when next requested. No rebuild was run merely to refill caches. Keep the single
shared target with CARGO_INCREMENTAL=0, Node 24.11.1 first on PATH and Homebrew
Python 3.11+ for store.py. All current test/verification processes are terminal.
