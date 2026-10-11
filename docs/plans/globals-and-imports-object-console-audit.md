# Object and console namespace audit

Status: 27 members reviewed (five Object, 22 console). Keep both familiar global
namespaces; no imported synonyms or removals are proposed. General object
reflection and diagnostic output are discoverable language utilities. Their native
limits remain explicit; names do not imply a dynamic JavaScript object model,
browser DOM, inspector or callable reflection.

## Shared contracts

`unknown` in the catalog is a signature placeholder for checked supported input
shapes, not dynamically typed unchecked storage. Object operations borrow inputs
and return owned results. Snapshot values are explicitly cloned; borrowed views
and other non-cloneable fields cannot silently become independent owned values.
Object methods require their exact documented argument counts and statically
reject unsupported receivers/access. Property reflection checks native capability
metadata; opaque resource types do not gain reflection through their representation.

Console returns void and borrows arguments, evaluating them once left-to-right.
Earlier argument loans prevent conflicting later mutation/transfer. Inspection
creates temporary representations; these allocations do not transfer or clone the
user's ownership graph into a reusable public value. Output write failures are
ignored, not returned as Result. Callback/callable values and opaque native types
without inspection capability are rejected. Inspection of recursive data is bounded.

The current console checker enforces table's two-argument limit and dir option
types. Other listed signatures describe meaningful parameters: additional printable
arguments are accepted/evaluated and may be ignored by fixed-parameter runtime
operations. This exception must not be confused with Object's exact arity checks.
No new argument rejection is introduced by this audit.

## Object members

| Symbol | Signature | Purpose, result, errors and ownership |
|---|---|---|
| `Object.is` | `is(left: unknown, right: unknown): boolean` | Borrows both values, evaluated once left-to-right; SameValue equality: NaN equals NaN, signed zeros differ, int/number compare numerically, aggregate data compares storage identity. Callable/void values rejected; returns boolean. |
| `Object.keys` | `keys(value: unknown): string[]` | Borrows record/tuple/array/string/number/boolean. Independent names in each record constructor order, positional names for arrays/tuples, UTF-16 positions for strings; numeric/boolean values yield empty arrays. Nullable/functions rejected. |
| `Object.values` | `values(value: unknown): Value[]` | Borrows record/tuple/array; clones independent field/element snapshot. Mixed fields infer union element type. Empty records, function fields, non-cloneable payloads and unsupported receivers rejected. |
| `Object.entries` | `entries(value: unknown): [string, Value][]` | Borrows record/tuple; clones independent name/value tuples in constructor order. Mixed values infer union. Arrays rejected (use array.entries); same clone/type restrictions as values. |
| `Object.hasOwn` | `hasOwn(value: unknown, key: string \| int \| number \| boolean \| null \| undefined): boolean` | Borrows receiver then primitive query key; converts key to text. Tests declared record fields, canonical position names and length on positional values. Strings use UTF-16 length. Numeric/boolean receivers return false; nullable/union receivers and functions rejected. |

Record key order follows each value's constructor literal; clone/replacement keep
that order. This differs from assuming type-declaration order. Object.hasOwn uses
own fields only; inherited names do not count. A present field whose value is
undefined remains present. Positional strings such as "01" and "-0" are not
canonical indices; numeric -0 converts to "0". Object string reflection counts
UTF-16 units even though other native string operations can use different units.

## Console members

| Symbol | Signature | Observable effects and failures |
|---|---|---|
| `console.log` | `log(...values: unknown[]): void` | Formats values and newline to stdout; empty call emits blank line. |
| `console.info` | `info(...values: unknown[]): void` | Same stdout behavior as log. |
| `console.debug` | `debug(...values: unknown[]): void` | Same stdout behavior as log. |
| `console.warn` | `warn(...values: unknown[]): void` | Formats values and newline to stderr. |
| `console.error` | `error(...values: unknown[]): void` | Same stderr behavior as warn; does not throw. |
| `console.table` | `table(data?: unknown, properties?: string[]): void` | Prints box-drawn table to stdout; optional column names select order; omitted data prints undefined; primitive data prints normally. More than two arguments rejected. |
| `console.assert` | `assert(condition?: unknown, ...values: unknown[]): void` | False-like condition prints Assertion failed and optional formatted message to stderr; does not throw. Omitted condition is false-like. |
| `console.dir` | `dir(value?: unknown, options?: {depth?: number, showHidden?: boolean, colors?: boolean}): void` | Inspects one value without format substitutions. Omitted value is undefined; depth defaults 2, clamps -1..128; null depth selects 128; showHidden/colors default false; unknown options ignored. |
| `console.dirxml` | `dirxml(...values: unknown[]): void` | Alias of log; no DOM visualization. |
| `console.group` | `group(...values: unknown[]): void` | Optional formatted label, then adds two spaces of indentation to subsequent stdout/stderr console lines. |
| `console.groupCollapsed` | `groupCollapsed(...values: unknown[]): void` | Same terminal behavior as group; no interactive collapse. |
| `console.groupEnd` | `groupEnd(): void` | Removes one indentation level, saturating at zero; no output. |
| `console.count` | `count(label?: unknown): void` | Converts optional label to text (omitted/undefined default); increments independent counter and prints label: count. |
| `console.countReset` | `countReset(label?: unknown): void` | String label resets existing counter; omitted/undefined selects default. Non-string labels warn without reset even if count converted that value to text. Missing counter warns on stderr. |
| `console.time` | `time(label?: unknown): void` | Starts monotonic timer for converted label (default if omitted/undefined); duplicate warns and preserves existing timer. |
| `console.timeLog` | `timeLog(label?: unknown, ...values: unknown[]): void` | Prints elapsed milliseconds plus additional inspected values without substitution; retains timer; missing label warns. |
| `console.timeEnd` | `timeEnd(label?: unknown): void` | Prints elapsed milliseconds and removes timer; missing label warns. |
| `console.clear` | `clear(): void` | ANSI clear/home only for terminal stdout when TERM is not dumb; redirected output unchanged; preserves indentation. |
| `console.trace` | `trace(...values: unknown[]): void` | Prints formatted message and captured native stack to stderr; does not throw. |
| `console.profile` | `profile(label?: unknown): void` | Evaluates arguments; no profiler/output is implemented. |
| `console.profileEnd` | `profileEnd(label?: unknown): void` | Evaluates arguments; no profiler/output is implemented. |
| `console.timeStamp` | `timeStamp(label?: unknown): void` | Evaluates arguments; no profiler/output is implemented. |

Log-style formatting supports %s/%d/%i/%f/%j/%o/%O, consumes %c style arguments,
and handles %% literally. Missing substitutions remain literal; extra values use
space separation. Top-level text is unquoted; nested values are inspected with
quoted strings, default depth two and at most 100 displayed array elements.
Groups/counters/timers use thread-local state. Native tracing reports a native
stack, not a promised JavaScript stack. Test execution captures console streams
with source location metadata through the existing test reporter.

The countReset parameter documentation previously described text conversion
without its exception, while a later paragraph already stated the exception.
The parameter description now agrees with runtime behavior; no behavior changes.
A focused native witness checks numeric-label rejection and explicit string reset,
and editor hover verifies the corrected parameter description.

## Source reconciliation and dependencies

All compiler anchors below are under crates/tsrust-compiler/src:

- analysis.rs::object_is and object_keys.rs::object_keys/object_has_own check
  borrowing, argument types, native capabilities and exact arity.
- object_values.rs::object_values checks reflection, field unions, cloning,
  empty-record rejection and the distinct array support of values versus entries.
- emit.rs ObjectIs/ObjectKeys targets, object_has_own/object_values and record
  observation metadata implement identity, UTF-16 positions, field order and copies.
- analysis.rs console call dispatch explicitly recognizes all 22 catalog names,
  checks table/dir shapes and console_inspectable values, and records read loans.
- emit.rs::console_call/console_value and scoped_record_console_emit.rs build
  temporary inspection values and preserve source locations for test output.
- runtime_console.rs::tsr_console_call/write implements all streams, formatting,
  state, timing, trace, terminal clear and explicit inspector no-ops.
- emit.rs includes console runtime when generated output references console
  values/helpers or test capture. This is family-level dependency selection;
  scalar fast paths avoid constructing general inspection trees where supported.
  Object reflection adds record order metadata only for observed record types.
  No extra imported package or user linker flag is required. This audit does not
  claim per-method binary elimination or measure allocation/throughput changes.

The catalog and independently inspected dispatch agree on all 27 names. Existing
static namespace/name resolution remains in force; this review does not make
Object or console dynamically replaceable namespace objects. Keep current global
placement, document limitations and retain the explicit native service imports.

## Focused acceptance

SQLite evidence `tsrust-api-object-console-*.log` retains the failing documentation
witness, corrected editor checks, native console output/state/lifetime checks and
Object reflection/identity/ownership checks. Whole-project formatting verifies
idempotence and token preservation before checkpoint. No full-family regression or
new benchmark timing is claimed.
