# Gate 187: native declaration and byte signature foundation

The typed native foundation and initial std:bytes construction/length slice are
accepted on 2026-10-10 at source commit 6e565765. B11/S07 remains in progress; gates 188–190 own wider operations,
views and resource lifecycle. No workload has been timed for this family.

## Supported behavior

Native declaration IDs distinguish opaque Bytes from the public ByteError record.
Named, array and Result signatures project into the existing checked arena and generic
specialization. Native Result payloads use the same alternatives as source Result;
there is no dynamic handle or second tagged-union runtime. Module loading separates
native type exports from callable exports, preserves aliases and privacy, and rejects
record-literal or spread forgery of opaque owners.

fromArray borrows an int array, validates values in 0..255 before copying, and returns
Result<Bytes, ByteError>. Empty input succeeds. Invalid input reports the first invalid
array offset. Checked length and fallible reservation report structured errors;
process-wide memory exhaustion is not universally recoverable. Length borrows Bytes
and returns its int byte count. Native operations return failures through Result and
have explicit nonthrowing effect contracts.

Focused native execution covers direct and stored calls, callback forwarding, generic
identity, direct Bytes arrays and records, Result arrays and records, repeated readonly
observation, empty/boundary inputs and first-invalid offsets. Static witnesses reject
wrong types/arity, later use after transfer and unsupported opaque capabilities.

Editor assistance covers imported and exported native aliases, public field docs,
structured function signatures, parameter help and import completion. Structured
exported aliases preserve hidden native fields without colliding with a local record
of the same public name. Internal identities are hidden from hover and completion.
Checked match payloads have full member help; declaration-only match arms retain the
existing lack of payload type inference.

## Repairs found while closing the gate

Exported Result aliases were qualified as user generic records by the module alias
linker. Result is now preserved as a built-in constructor, with a failing-then-passing
native witness for both primitive and native payload aliases. This repairs an accepted
B10 composition defect rather than introducing a new Result representation.

Structured alias imports previously had no declaration symbol or member help. Their
projection now remaps native names by identity, keeps navigation on the exported alias,
and expands borrowed array aliases before resolving element members. Private editor
symbols no longer appear as global completion candidates or raw names in help.

## Evidence

The closing focused batch has 68 passes: 24 compiler, 41 editor and three benchmark
registration controls. Earlier increments retain signature bounds, opaque capability,
ten normal-stack depth controls and two native documentation examples. Source round
trips preserve tokens and stable formatting. No complete compiler/editor run or timing
was required for this individual gate; those remain due at the S07 family milestone.

The matched native_bytes workload retains six scalar observations per iteration:
two repeated borrowed lengths, the unchanged input element, empty length, first-invalid
offset and error-code length. Its TypeScript counterpart validates before constructing
an independent Uint8Array. An arithmetic oracle, token-based scenario guards,
source fingerprints, registration tests and runner/profile/report hooks are present.
Five cases pass on native and Node after TypeScript compilation without timing.

Exact SQLite records:

- runtime/results/native-byte-storage-workload-2026-10-10-checks.json
- runtime/results/2026-10-10T14-08-14-877Z-verify.json
- runtime/results/native-byte-construction-2026-10-10-checks.json
- Earlier native declaration, projection, import, capability and re-export records
  listed in docs/plans/resources-bytes-plan.md.

The first matched verification precedes the narrow exported-Result-alias repair; its
inputs are retained unchanged. Final source verification after that repair also passes:
runtime/results/2026-10-10T14-17-12-756Z-verify.json. Records are imported with byte-for-byte
readback; loose copies are removed.

All five consumers are refreshed from 6e565765 with atomic sibling replacement and
verified hashes. Installed compiler debug/release storage checks and LSP initialization/
shutdown pass. Publication evidence is in SQLite at
runtime/results/gate187-2026-10-10-publication.json. Obsolete test executables and
generated runtime outputs were pruned; shared libraries and installed consumers remain.
