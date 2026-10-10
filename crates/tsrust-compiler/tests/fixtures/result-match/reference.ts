import { strictEqual } from "node:assert";

type Result<T, E> = { kind: "Ok"; value: T } | { kind: "Err"; error: E };

function divide(a: number, b: number): Result<number, string> {
    if (b === 0) {
        return { kind: "Err", error: "zero" };
    }
    return { kind: "Ok", value: Math.trunc(a / b) };
}

const success = divide(10, 2);
switch (success.kind) {
    case "Ok": strictEqual(success.value, 5); break;
    case "Err": strictEqual(success.error, "unexpected"); break;
}

const failure = divide(10, 0);
switch (failure.kind) {
    case "Ok": strictEqual(failure.value, -1); break;
    case "Err": strictEqual(failure.error, "zero"); break;
}
