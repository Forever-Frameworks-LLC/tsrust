# Gate 155: Unicode string iteration

String for-of visits Unicode scalar values in source order, lending each character
as a readonly string. Empty input visits no body. The source expression runs once;
source replacement or transfer is blocked during the loop. Owned character
transfers require explicit `clone(character)`. Readonly string parameters can read
characters, and normal break, continue, throw and finally behavior is preserved.

## Implementation

Iterable typing recognizes string inputs. A dedicated non-inlined emitter helper
walks chars() directly and creates one owning character string per iteration,
borrowed by the body. There is no source clone or full character array snapshot.
This allocation is necessary for the current Cow<'static,str> string ABI; a dynamic
borrowed substring cannot satisfy existing readonly string parameter types. Native
strings represent well-formed UTF-8 Unicode scalar values; character length still
counts UTF-8 bytes. See docs/guides/language/string-iteration.md for examples and ownership rules.

Editor iteration inference exposes the character type and string methods. No new
built-in member is introduced. A stale rejection for iterating "Pat" is now a
positive compiler check; independent Node confirms the supported behavior.

## Focused evidence

- Red CLI check rejects string iteration before implementation. Assertion fixture
  covers ASCII, multibyte/astral text, combining marks, nested loops, explicit
  copies, source loans, factory evaluation once and finally on break/continue.
- Native fixture passes. Independent pinned Node passes after adapting only the
  helper measuring string length to TextEncoder UTF-8 byte length. Character
  traversal and completion remain unchanged in the reference source.
- Compiler: final for_of 10, corrected string_iteration 3, map_iteration 2 and
  nullish_coalescing 4 pass (19 distinct checks across retained runs). The initial
  callback-result witness incorrectly omitted a named function result annotation;
  a baseline probe confirms omitted annotations mean void. The corrected witness
  checks a typed callable containing a string loop.
- Editor: map_iteration 1 and string_iteration 1 pass.
- The guide example, wrapped in main(), prints A|é|🙂| identically natively and
  through pinned Node. Wrapping corrected the first standalone example probe.
- Harness 382/382 passes after correcting the string workload report category;
  the prior report failure remains in the original harness log.
- Matched native/node-tsc verification passes untimed:
  runtime/results/2026-10-06T08-48-23-330Z-verify.json. Array-based independent
  oracle covers traversal order/count, filtering and nested loops. Reporting-only
  category correction followed verification; source fingerprints are unchanged.
- Five consumers rebuilt and atomically refreshed after workload check. Hashes:
  target/editor/string-iteration-consumers.log. Pre-acceptance manifest retained.

Source pair, oracle, contracts, registration and runtime/profiling/report hooks are
complete. Logs are retained under target/editor/string-iteration-*.log; original
sources, records and previous consumers have archive/string-iteration-*-start
snapshots. No full regression or timing. Collection measurements remain pending;
live string iterator objects and the general iterator protocol remain unsupported.
