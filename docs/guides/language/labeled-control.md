# Labeled statements, break and continue

A label names a statement so a nested `break` can leave it. A label on a loop also lets a nested `continue` start that loop's next iteration.

```tsrust
import { strictEqual } from "std:testing";

function main() {
    let total = 0;
    let updates = 0;
    outer: for (let i = 0; i < 3; i++, updates++) {
        for (let j = 0; j < 2; j++) {
            if (i === 1) continue outer;
            if (i === 2) break outer;
            total += 1;
        }
    }
    strictEqual(total, 2);
    strictEqual(updates, 2);

    let chosen: int;
    done: {
        chosen = 7;
        break done;
    }
    strictEqual(chosen, 7);
}
```

## Targets and execution

`break name;` leaves the enclosing statement with that label. Blocks, `if`, loop statements and other non-declaration statements can be labeled. Several labels can name one statement: `first: second: while (...) { ... }`.

`continue name;` requires a label on an enclosing loop. It skips the remaining body of that loop. A classic `for` runs its update, then tests its condition. A `do...while` tests its condition. A `while` tests its condition, and a `for...of` or `for...in` advances its iteration. Updates of intervening loops do not run.

Without a name, `break` and `continue` still target the nearest enclosing loop. A labeled block does not capture unnamed loop control. Existing switch restrictions still apply; a labeled break to an enclosing statement remains distinct from the unnamed break ending a switch clause.

Labels occupy a separate namespace from variables. An ordinary value can have the same name. The editor resolves a label reference to the label declaration, not the value.

## Scope and diagnostics

- A label is visible only inside its statement and within the same function or closure. An unknown or out-of-scope target is `loop-control`.
- `continue` targeting a labeled block is `loop-control`.
- Reusing an active label, including in a chain of labels on one statement, is `duplicate`. Disjoint statements may reuse a label.
- A label directly before `let`, `const`, `type` or a function declaration is `syntax`.
- A statement supports at most 32 labels. Additional labels are `unsupported`. Ordinary source-depth limits still apply.
- Labeled control does not add an implicit clone. Transferring an outer owner before continuing its loop is rejected unless the binding is reinitialized on every continuing path. Breaking out of the loop preserves the transferred state on that exit.
- A deferred variable must be assigned on every path reaching its use, including paths that leave a labeled block. A `continue` must also satisfy assignment requirements in the target loop's update or post-test condition. Loop bodies still do not establish assignment for ordinary code after the loop.
