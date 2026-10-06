# Gate 144: nominal enums

Status: accepted with focused compiler/editor checks and matched untimed verification.
No full regression or performance measurements were run.

## Behavior

Enums create distinct, copyable types with numeric or string members. Numeric
values start at zero and omitted values increment the previous value. Constant
arithmetic, bitwise expressions, and earlier member references are supported.
Duplicate values compare equal; duplicate names and mixed value categories are
rejected. Members are readonly. Raw primitive values and other enum types cannot
be assigned implicitly. Explicit `Number` and `String` conversion exposes values.

Enum values work in record fields, arrays, callbacks, generics, unions and imported
aliases. `typeof` follows the backing value category. The namespace remains static;
reverse lookup, runtime enum objects, runtime initializers and `const enum` are
unsupported. See [the guide](../enums.md).

## Implementation and repairs

A separate parser helper evaluates constant members. Checked enum types retain
nominal identity through annotation resolution, generic inference, module linking
and expression facts. Nonempty native enums have an explicit 32-bit representation;
record layout accounts for it. Arrays retain ordinary storage and enum access copies
values without allocating or cloning surrounding owners.

Focused witnesses found and repaired record-layout rejection, array search
classification, generic argument annotation, and native assertion equality. Enum
identity uses backing values rather than record pointer identity. Numeric assertion
identity preserves NaN and signed-zero behavior. Enum metadata registration occurs
after generic record templates are removed, preserving declaration indices.

Editor support includes declarations and members, hover, definitions, completion,
formatting, highlighting and imported aliases. An imported-member witness found
and repaired the declaration-kind filter and alias receiver identity.

Exact originals are retained in `archive/enums-*-start/`, including the entire
compiler source directory and previous consumer bytes. Other feature changes remain
preserved.

## Evidence

Logs are retained under `target/editor/enums-*.log`.

- Enum compiler suite: 5/5 (`enums-final-native.log`), with assertion-based native
  execution, invalid declarations/assignments and exported aliases. The expanded
  fixture also passes direct native execution and independent pinned Node reference.
- Focused compiler canaries: for-of 10, higher-order function values 6, nullish
  coalescing 4, assertions 6, type syntax 8 (`enums-focused-batch.log`). Generic
  default scope 3 passes in the later generic batch. These canaries precede the
  final enum-only generic annotation and identity repairs; only the enum suite
  was rerun after those repairs. Total passing coverage is 42 distinct tests.
- Enum editor suite: 3/3 (`enums-import-editor-final.log`). Adjacent exception
  and assertion editor suites each pass 2/2 in `enums-final-editor.log`; that
  batch retains its initial imported-enum failure. Only the affected enum suite
  was rerun after repair. No complete editor run was performed.
- Runtime harness: 369 of 370 passed before consumer refresh
  (`enums-harness.log`). The sole failure used a stale editor compiler against
  the new enum workload. Its affected prepared-rank-driver check passes on the
  refreshed compiler (`enums-rankdriver-final.log`). All 370 checks are accounted
  for across the original pass and focused rerun.
- Source/oracle registration passes, including fingerprint mutations, denominator
  guards, driver hooks and preservation of previous manifest rows.
- Five consumers were refreshed atomically. Hashes are in `enums-consumers.log`;
  previous bytes are in `archive/enums-consumers-start/`.
- Guide example prints `5` then `long` in native and Node execution.
- Native and node-tsc verification passes five cases per engine:
  `benchmarks/runtime/results/2026-10-06T05-09-10-382Z-verify.json`.
  Startup, compilation and execution timing arrays are empty. Existing SQLite
  measurements remain unchanged.

## Benchmark readiness

`enum_values` contains matched native and TypeScript sources, an independent oracle,
source contracts, registration checks, runtime/profile hooks and reporting category.
It retains a numeric discriminant, selected string member and dynamic input text.
The workload is accepted but unmeasured, pending a meaningful type-syntax milestone.
