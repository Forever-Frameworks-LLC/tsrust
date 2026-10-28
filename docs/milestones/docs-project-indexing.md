# Checked project indexing and documentation diagnostics

The local LSP checked the docs entry in 16.9 seconds, then spent the rest of a
120-second acceptance window applying checked type information without publishing
diagnostics. Tracing separated compiler checking, declaration indexing, checked
facts, references and diagnostic conversion. Workspace-scan isolation reproduced
the stall, excluding background scanning as the sole cause.

The checked index now applies named record fields to visible declarations rather
than importing every unrelated project record into each generated loader. Checked
dependency indexes still supply other types when a value references them. Completed
structural expansions are reused within the immutable project revision. Validated
JSON schema symbols retain their expanded field help instead of being overwritten
with private JsonData names. No compiler diagnostics or required type information
are suppressed, and no indexing quota is introduced.

Acceptance:

- 178 project-intelligence, 3 interface, 15 schema-editing and 2 text-import cases
  passed (198 total). A focused inherited-help unit case also passed earlier.
- Existing Windows-incompatible virtual file URIs were corrected in the affected
  fixtures; production absolute-URI validation remains strict. One obsolete
  parameter-label whitespace expectation was updated to current display formatting.
- A controlled baseline with the original field-fact implementation reproduced
  the same three fixture/JSON-hover failures. The rejected line-table experiment
  was reverted; it did not resolve the large-project stall.
- Actual local LSP on the 1,668-module docs application: clean entry diagnostics,
  deliberate in-memory type error reported, repair clears diagnostics, disk source
  unchanged. Fresh backend-async entry/operation diagnostics and error/repair also
  passed with the ordinary 15-second per-publication deadline.
- The large docs case uses an explicit 120-second deadline; the default remains
  15 seconds. The first clean publication took about 75 seconds: checking 17.2s,
  declaration indexing 15.7s, applying facts 41.2s, references 0.74s. Startup remains
  slow and cooperative interruption remains open. These are diagnostic observations,
  not matched benchmark measurements or proof that all VS Code Problems are clear.

Opt-in TSRUST_LSP_TRACE_TIMINGS writes stage observations to local stderr. The
protocol verifier supports projects without API config, checks an in-memory error
independent of operation signatures, and reports observed publications/counters on
failure. It never controls the owner's editor. Disposable logs live under target:
lsp-project-help-current.log, docs-editor-visible-facts-current.log and
backend-editor-current.log. Full compiler/LSP matrices, matched benchmarks and other
platform executions were not performed for this editor section.

The native documentation startup stack overflow and 16 authoring-only UI dependency
warnings remain separate open issues. Server-owned API registration, shared Cargo
source caches, removal of embedded vendoring and trusted CLI/LSP installation/update
remain required backend work.
