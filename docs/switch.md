# Switch statements

`switch` compares one value with each `case` in source order and runs the first clause that matches. `default` runs when no case matches.

```tsrust
import { strictEqual } from "std:testing";
function name(code: int): string {
    switch (code) {
        case 1:
            return "one";
        case 2:
        case 3:
            return "few";
        default:
            return "many";
    }
}
function main() {
    strictEqual(name(1) + name(3) + name(9), "onefewmany");
    let total = 0;
    for (let i = 0; i < 4; i++) {
        switch (i % 2) {
            case 0:
                continue;
            default:
                total += i;
                break;
        }
        total += 100;
    }
    strictEqual(total, 204);
}
```

## Matching

- The value in parentheses is evaluated once.
- A case matches by strict equality (`===`). The value and each case must be comparable: int, number, boolean or string, or a union of those.
- Case expressions run in source order and stop at the first match. Later case expressions are not evaluated.
- Adjacent labels share one clause: `case 2: case 3:` runs the same statements for both values.
- `default` may appear anywhere. It runs only after every case has failed, whatever its position. A switch can have one `default`.
- When the value is a union variable, a matching case narrows it inside the clause, so `case "go": value.toUpperCase();` is accepted.

## Ending a clause

Each clause ends with `break`, `return` or `continue`, or is the last clause. `break` leaves the switch. `continue` continues the enclosing loop. A clause that is one block may end the block with `break`.

Two JavaScript forms are not available, and each has a message that says what to write:

- **Running into the next clause.** A clause with statements cannot fall into the clause after it. End it explicitly, or leave it empty to share the next clause.
- **A `break` inside a nested statement of a clause**, such as `if (done) break;`. Make `break` the last statement of the clause, or restructure with `if` and `else`. A `break` inside a loop in the clause belongs to that loop, as usual.

## Scope and results

Each clause is its own block. A name declared in one clause is not visible in another. A function whose every path must return a value needs a `default` clause that returns, like a final `else`.

A switch supports at most 120 clauses.
