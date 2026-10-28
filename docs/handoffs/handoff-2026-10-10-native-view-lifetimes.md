# Gate 189 named native function lifetimes

Read AGENTS.md, docs/development-workflow.md, docs/plans/resources-bytes-plan.md and
docs/milestones/native-byte-views-implementation.md first. This checkpoint follows 5991b8d2.
Gate 189 and S07 remain open; public view exports remain guarded.

native_view_lifetime_emit plans named lifetime slots from carried input provenance
and checked return summaries. A single origin keeps its input lifetime; alternative
origins get an output lifetime constrained by each source. Independent inputs and
independent record fields remain distinct. The shared signature planner carries
these declarations, output types and outlives bounds alongside existing callback
and scoped record requirements.

Private formatted-source witnesses compile generated functions at O0 and O3, with
actual slice pointer checks and exclusive write-through. They cover direct returns,
record fields, arrays, Results and alternative inputs. Positive controls let a first
input or field outlive an unused shorter-lived sibling. A negative Rust control
rejects a selected result escaping one of its possible owners.

The native_view_loans module is crate-visible only to share internal typed origin
identities with emission. No public compiler API or built-in source binding is added.
Stored function values, closures/callback composition, recursive wrappers, forwarding
call provenance, collection transport and exception completion reconciliation remain
pending. Current output lifetime grouping uses value-relative paths. No source
factory execution or accepted end-to-end view support is claimed.

Checks passed: 41 view tests, nine existing callable signature tests, 89 depth
canaries and 16 native editor tests. The extended Result/control witness passed
again after its final additions. Project formatting changed zero files/fingerprints.

Exact red, repair and final logs are stored in records.sqlite under
runtime/results/native-view-lifetimes-2026-10-10-checks.json. Timing and full regressions
remain the completed S07 milestone. Installed consumers remain f569190a; the warm
compiler was rebuilt for whole-project formatting. Use the single Cargo target,
CARGO_INCREMENTAL=0, Node 24.11.1 first on PATH and /opt/homebrew/bin/python3 for store.py.

## Continuation prompt

Continue gate 189 from docs/handoffs/handoff-2026-10-10-native-view-lifetimes.md. Connect checked
function return origins to direct and stored call results without borrowing parameter
slots that merely carry inner lifetimes. Add source/native forwarding witnesses and
complete collection/control-flow/capture provenance. Preserve genuine Rust slices and
independent input lifetimes. Keep public factories guarded until all required analyzer
and backend paths agree. Finish complete editor/docs and matched benchmark support
before gate acceptance. Full regressions and timing belong to S07 completion.
