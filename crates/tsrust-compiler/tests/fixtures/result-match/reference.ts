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

function counter(start: number): Result<() => number, string> {
    let value = start;
    return { kind: "Ok", value: () => ++value };
}

function wrap<T>(value: T): Result<T, string> {
    return { kind: "Ok", value };
}

const first = counter(10);
const second = counter(20);
if (first.kind !== "Ok" || second.kind !== "Ok") throw new Error("Unexpected failure");
strictEqual(first.value(), 11);
strictEqual(first.value(), 12);
strictEqual(second.value(), 21);

const callback = wrap<() => number>(() => 7);
if (callback.kind !== "Ok") throw new Error("Unexpected failure");
strictEqual(callback.value(), 7);

const callableError: Result<number, () => string> = { kind: "Err", error: () => "error" };
if (callableError.kind !== "Err") throw new Error("Unexpected success");
strictEqual(callableError.error(), "error");
