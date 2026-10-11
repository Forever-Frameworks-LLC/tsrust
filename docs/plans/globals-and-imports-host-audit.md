# Host-service symbol placement review

Status: all thirteen current exports across `std:path`, `std:os`, `std:env` and
`std:time` have a per-symbol source review. Preserve their imported placement and
existing names. These APIs expose host conventions or state; explicit imports
keep dependencies visible and support meaningful local aliases. No global aliases
or renaming are proposed.

## Per-symbol coverage

All parameters below are required unless marked optional or rest. Wrong argument
counts/types are compile-time errors. Path/string parameters are borrowed; string
results are independent values. None of these functions mutates its arguments.
These general statements supplement, rather than replace, the individual rows.

| Symbol | Purpose and signature | Errors and effects | Runtime dependency and disposition |
|---|---|---|---|
| `std:path.basename` | `(path: string, suffix?: string): string`; final component, optional exact suffix removal | Lexical; no filesystem access; borrows both strings | Bundled path helper; retain import and alias support |
| `std:path.dirname` | `(path: string): string`; lexical parent | Empty/bare filename yields dot; no filesystem lookup; borrowed input | Bundled path helper; retain import |
| `std:path.extname` | `(path: string): string`; last component's final suffix | Leading-dot name and absent extension yield empty text; no filesystem lookup | Bundled path helper; retain import |
| `std:path.isAbsolute` | `(path: string): boolean`; test platform root syntax | No filesystem access or containment guarantee; scalar result | Bundled path helper; retain import |
| `std:path.normalize` | `(path: string): string`; collapse separators and resolve lexical dot components | Empty becomes dot; preserves applicable trailing separator; no symlink resolution or containment guarantee | Bundled path helper; retain import; document platform/device spelling limitations |
| `std:path.join` | `(...paths: string[]): string`; join and normalize path segments | Zero parts allowed; skips empty segments; borrows each supplied string; no filesystem lookup | Bundled path helper; retain import; native rest arguments remain checked strings |
| `std:path.relative` | `(from: string, to: string): string`; lexical route after resolving relative locations | Borrows both strings. Required cwd lookup can throw Error with code `path-relative`, including non-UTF-8 cwd. Identical and fully qualified inputs avoid unnecessary lookup | Bundled path plus working-directory access; retain import and explicit throwing documentation |
| `std:os.platform` | `(): string`; executable target OS name | Constant during execution; unsupported target selector causes native compilation failure, not runtime Result | Per-export target-selected bridge; retain import |
| `std:os.arch` | `(): string`; executable target architecture name | Constant during execution; unsupported target selector causes native compilation failure | Per-export target-selected bridge; retain import |
| `std:os.endianness` | `(): string`; executable target byte order, BE or LE | No runtime mutation or handle; unsupported selector causes native compilation failure | Per-export target-selected bridge; retain import |
| `std:env.get` | `(name: ReadOnly<string>): Result<string \| undefined, EnvError>`; execution-time environment lookup | Missing is Ok(undefined), empty is Ok(""). Invalid name and non-Unicode value are Err. Borrows name; present output owns its UTF-8 storage; does not mutate environment | Environment bridge and host lookup; retain import and checked Result contract |
| `std:env.EnvError` | `{code: string, message: string}`; InvalidName or NotUnicode lookup failure | Ordinary owned record, explicit independent clone. Error text excludes the environment value | Ordinary record layout; retain imported type |
| `std:time.monotonicNow` | `(): number`; nonnegative elapsed milliseconds from first-call process-local origin | No arguments, handle or cleanup. Readings may be equal; no calendar/cross-process meaning. Copies number; initializes shared clock origin once | Selected Instant/OnceLock helper; retain import rather than adding an ambient clock |

Path behavior follows the executable platform's separator/drive conventions.
Names do not imply filesystem identity, existence, symlink resolution or a sandbox
boundary. OS identity describes the executable target, including translated
execution, rather than physical hardware. Environment names follow host matching
rules; name validation rejects empty names, equals and NUL. Monotonic suspension
behavior is platform-dependent and floating-point precision decreases over long
runtimes. These observable differences are already described in the catalog.

## Source reconciliation

Bindings: `crates/tsrust-compiler/src/stdlib/path/bindings.rs`,
`os/bindings.rs`, `env/bindings.rs` and `time/mod.rs`. Their parameter modes,
optionality, results and feature tags agree with the corresponding catalog groups
in `crates/tsrust-lsp/src/builtins.json`. Native aliases resolve to the same binding
identity; these imports do not reserve ambient utility names.

Implementations: `stdlib/path/runtime.rs` and `relative.rs` provide lexical
operations and cwd resolution; `stdlib/os/mod.rs::bridge` selects target strings
and unsupported-target diagnostics; `stdlib/env/runtime.rs::get` plus `emit.rs`
convert missing/invalid/non-Unicode results without exposing values in errors;
`stdlib/time/runtime.rs` owns the monotonic origin. All paths in this paragraph
are relative to `crates/tsrust-compiler/src`.

`emit.rs` emits selected native calls and their bridges. The existing clock
witness proves an unused clock import adds no clock helper. No claim is made here
that every unused native type/import is eliminated. There is no user-supplied
linker flag requirement.

## Focused audit evidence

The environment and monotonic integration tests cover owned results, invalid
calls, non-Unicode environment data, alias/stored-call behavior, ordered finite
clock readings and unused clock imports. The relative-path witness covers
borrowed inputs, independent output and cwd failure/avoidance. Existing imported
path/OS editor witnesses cover signatures and documented behavior. Audit logs
are retained in SQLite under `tsrust-api-host-*`; this review adds no timing claim
or broader platform acceptance.

The throwing `relative` contract is an existing, documented exception to the
Result-based resource APIs. Preserving it avoids an unrelated breaking migration.
Its visible effects reinforce the approved policy: familiar names must carry
accurate error documentation, while JSON uses explicitly distinct names for its
two approved error-delivery styles.
