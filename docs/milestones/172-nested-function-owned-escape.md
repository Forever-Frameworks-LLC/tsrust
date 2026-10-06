# Gate 172: owned escape of nested functions

Status: in progress, not accepted. Gate 171 is accepted at 1c6dd58. Preserve the
four owner fixture edits and use the existing development workflow and full nested
function plan. The settled model transfers owned escaping captures, preserves scalar
copy semantics and uses existing loans for borrowed captures. No implicit clones or
shared JavaScript capture cells are introduced.

## Implementation chunks and gates

1. Ownership-taking captures. A named function can return or otherwise transfer a
   captured owner. Replace blanket rejection with existing availability, argument
   loan and transfer checks. Verify independent factory instances, direct owner reuse,
   once-callable reuse and overlapping argument/capture access. Use the arrow wrapper
   and existing owned callable transport, with source capture identities preserved.
2. Retained values and aggregate storage. Verify multiple factory instances, record
   and array storage, explicit moves, borrowed-data escape rejection and disposal.
   Add missing callable transport support if actual witnesses expose gaps.
3. Recursive environments and callback failure. Prove recursive captured groups have
   finite retained storage, no duplicated ownership and correct activation identity.
   Verify exception propagation and disposal on ordinary and failure paths.
4. Correctness and usability. Investigate the known dead-inner-declaration capture
   propagation: checking unused bodies must not create unnecessary runtime loans or
   transfers. Keep invalid unused bodies diagnosed. Complete editor diagnostics,
   hover/signature examples and guide coverage without exposing hidden arguments.
5. Acceptance support. Append a matched owned-escape workload at the END of the
   manifest; add the TypeScript counterpart, independent oracle, source contracts,
   registrations, observer/ABI and run/profile/report hooks. Verify untimed, run all
   eight canaries and whole editor, import/read back evidence into SQLite, refresh
   consumers atomically and commit. No timing or full compiler matrix for this gate.

## First ownership-taking chunk

The native take-owner fixture failed at the ownership rejection. It now returns two
independent captured User owners through the named-function arrow wrapper. The direct
call checks existing capture access against active argument loans and marks consumed
owners transferred. Native emission already supports consuming hidden parameters.
The final lowering validator no longer rejects every consuming capture. An unused
ownership-taking declaration is now typechecked without demanding execution.

Gate acceptance remains owed for aggregate storage, disposal, callback failures,
recursive retained lifetime, dead declaration propagation, editor examples and the
matched workload. Default/optional/generic function values and omitted-result
inference remain gate 173; preserve its genuine red witness.

Checkpoint evidence: capture suite17 and owned-escape tests2 pass, including native
independent factories and precise transfer/once/conflict diagnostics. Node reference
passes the factory fixture. Canaries63 and editor276 pass. Existing matched capture
cases5 pass untimed; new owned-escape workload remains owed before gate acceptance.
Canonical result 2026-10-06T19-52-48-838Z-verify.json has verified source fingerprints
and no timing samples. 11 exact original records are imported/read back in SQLite;
failed assertion and retired-keyword test drafts are retained. Five consumers are
refreshed atomically. 59 scratch executables and runtime output are pruned.
