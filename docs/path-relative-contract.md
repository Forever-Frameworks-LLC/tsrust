# Lexical relative path contract

`std:path.relative(from: string, to: string): string` computes the lexical route from one location to another using the executable target's path convention. It requires exactly two strings, borrows both inputs and returns an independent owned string. There are no default arguments. Arity and type errors are compiler diagnostics.

Empty arguments denote the current working directory. Relative arguments resolve against that directory; dot and parent segments collapse without crossing an absolute root, and redundant/trailing separators disappear. Identical input strings return an empty string without observing cwd. Fully qualified inputs also avoid cwd. The operation neither visits the input locations nor resolves symlinks and is not a containment check.

POSIX paths use `/` and case-sensitive components. Windows paths accept both separators, emit `\`, compare case-insensitively, preserve destination spelling and use drive-specific working directories for drive-relative inputs. Different drive or UNC server roots normally return the resolved destination. Length-changing Unicode lowercase mappings follow the pinned reference's component comparison behavior, including UNC and unusual unresolved drive contexts.

If required cwd lookup fails or cannot produce UTF-8 text, the checked native error mechanism throws an `Error` with code `path-relative`. No lossy conversion or fabricated cwd is used. The host helper caches one successful cwd observation for the pair; internal helpers accept injected cwd and per-drive observations for deterministic tests.

```tsr
import { relative } from "std:path";
import { strictEqual } from "std:testing";
function main() {
    strictEqual(relative("reports", "reports/2026"), "2026");
    strictEqual(relative("é😊", "é😊"), "");
}
```

Ownership: both arguments remain usable after calls; the returned string can be changed independently. Direct imports, aliases and stored native calls use the same contract.

## Reference and acceptance scope

Behavior is audited against [Node 24.11.1 path documentation](https://nodejs.org/download/release/v24.11.1/docs/api/path.html#pathrelativefrom-to) and its [official path implementation](https://github.com/nodejs/node/blob/v24.11.1/lib/path.js). The Rust cwd boundary follows [`std::env::current_dir`](https://doc.rust-lang.org/std/env/fn.current_dir.html). Runtime versions belong here, not editor help.

The deterministic reference fixture records every ordered pair of 288 paths under three contexts: POSIX `/workspace/project`, Windows `C:\work`, and Windows UNC `\\host\share\work`, with a separate `D:` drive directory. This gives 248,832 independently observed comparisons, including roots, siblings, prefixes, dots, repeated/trailing separators, Unicode case expansion, drive-relative and UNC locations. Windows coverage is internal-helper/reference coverage; actual generated native execution on this machine is macOS. Deleted-cwd errors execute against the host. Non-UTF-8 conversion is injectable on Unix; actual invalid-byte directory creation is additionally exercised where supported on Linux.

The matched `std_path_relative` workload alternates sibling and descendant destinations containing Unicode. Its independent oracle spells expected values without calling either relative implementation. Cwd lookup, lexical resolution, output retention and observation are included. Source mutation guards, fallible native ABI, runner and allocation-profile hooks are registered. Timing acceptance is recorded separately in SQLite; source readiness alone is not a measurement claim.
