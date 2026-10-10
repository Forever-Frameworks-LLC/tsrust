import { strictEqual } from "node:assert";
type Result<T, E> = { kind: "Ok"; value: T } | { kind: "Err"; error: E };
type State = { finalized: number };
function finish(result: Result<string, string>, state: State): string {
    try {
        switch (result.kind) {
            case "Ok": return result.value;
            case "Err": throw result.error;
        }
    } finally { state.finalized += 1; }
}
function failing(): Result<() => number, string> {
    const message = "failure".repeat(2);
    return { kind: "Ok", value: () => { throw message; } };
}
function invoke(result: Result<() => number, string>): number {
    switch (result.kind) { case "Ok": return result.value(); case "Err": return result.error.length; }
}
function message(text: string): Result<() => string, number> {
    return { kind: "Ok", value: () => text.trim() };
}
function onceFailure(text: string): Result<() => number, string> {
    return { kind: "Ok", value: () => { throw text; } };
}
function loop(result: Result<number[], string>): void {
    for (let index = 0; index < 4; index++) {
        switch (result.kind) {
            case "Ok": result.value.push(index); if (index < 2) { continue; } break;
            case "Err": break;
        }
        break;
    }
    if (result.kind === "Ok") { result.value.push(9); }
}
const state: State = { finalized: 0 };
strictEqual(finish({ kind: "Ok", value: "returned" }, state), "returned");
try { finish({ kind: "Err", error: "thrown" }, state); } catch (error) { strictEqual(error, "thrown"); }
strictEqual(state.finalized, 2);
let replaced: Result<string[], string> = { kind: "Ok", value: ["old".repeat(16)] };
replaced = { kind: "Err", error: "new".repeat(16) };
strictEqual(replaced.error.length, 48);
const callback = failing();
let caught = 0;
for (let index = 0; index < 2; index++) {
    try { invoke(callback); } catch (error) { strictEqual(error, "failurefailure"); caught += 1; }
}
strictEqual(caught, 2);
if (callback.kind === "Ok") {
    try { callback.value(); } catch (error) { strictEqual(error, "failurefailure"); caught += 1; }
}
strictEqual(caught, 3);
let replacedCallback = message("first".repeat(16));
replacedCallback = message("second".repeat(16));
if (replacedCallback.kind === "Ok") { strictEqual(replacedCallback.value().length, 96); } else { strictEqual(1, 0); }
const once = onceFailure("once".repeat(8));
if (once.kind === "Ok") {
    try { once.value(); } catch (error) { strictEqual((error as string).length, 32); }
} else { strictEqual(1, 0); }
const iterated: Result<number[], string> = { kind: "Ok", value: [] };
loop(iterated);
strictEqual(iterated.value.length, 4);
strictEqual(iterated.value[3], 9);
function recursive(depth: number): Result<number, string> {
    if (depth === 0) { return { kind: "Ok", value: 0 }; }
    const result = recursive(depth - 1);
    switch (result.kind) { case "Ok": return { kind: "Ok", value: result.value + 1 }; case "Err": return result; }
}
const counted = recursive(8);
if (counted.kind === "Ok") { strictEqual(counted.value, 8); } else { strictEqual(1, 0); }
const unused = message("unused".repeat(8));
