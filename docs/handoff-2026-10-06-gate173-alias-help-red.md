# Handoff: generic alias editor contracts

The result-context implementation is committed at e6fa6f5. Read AGENTS.md,
docs/development-workflow.md, docs/handoff-2026-10-06-gate173-result-context.md
and both function plans before continuing. Gate173 remains open.

Two focused editor witnesses now fail on otherwise valid compiler input:
deferred_generic_alias_hover_preserves_source_type_relation and
deferred_generic_alias_signature_help_keeps_concrete_call_contract. The alias
hover currently says `const action: T`, losing the callable relation. Parameter
help returns no signature for either the int or string invocation. The focused
command runs only these two tests; both fail. Its original output is retained as
tsrust-generic-alias-help-red.log in benchmarks/records.sqlite.

Implement checked source metadata for deferred origins and their value uses.
Expose the original generic parameter/result relation for alias hover and
completion, including names, access, omission, rest parameters and documentation.
Expose concrete invocation contracts for parameter help without selecting an
arbitrary sibling specialization. The existing CheckedTypeFacts.generic_calls
and checked_generic_signature paths are useful boundaries, but deferred alias
calls currently do not reach parameter help. Keep the empty prototype excluded
from ordinary callable facts; do not use its void result as a wildcard or infer
the generic source contract from a single concrete call.

Extend witnesses to captured factories, default/rest parameters, documentation,
completion and source navigation. Preserve project source coordinates and
ambiguous specialization handling. Run focused checks and the whole editor crate
before accepting this editor chunk. Continue real callback/storage adaptation
and the wider gate173 audit afterward; no family acceptance or timing is claimed.

Four unrelated owner fixtures remain untouched and uncommitted. No compiler
implementation changed in this red checkpoint. The test process is terminal.

Continuation prompt: Continue Tsrust from this handoff. Implement real generic
alias hover/completion and concrete invocation parameter help from checked source
contracts. Preserve ownership, owner fixtures, SQLite evidence and the full
generic-value plan. Then continue callback/storage adaptation and gate173 audit.
