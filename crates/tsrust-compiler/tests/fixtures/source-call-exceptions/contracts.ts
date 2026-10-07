import { strictEqual } from "node:assert";
function identity<T>(value: T): T { return value; }
function forward<T>(fail: boolean, value: T, trace: number[]): T {
    let action = identity;
    action = action;
    const saved = action;
    try {
        try {
            if (fail) { throw 1; }

        } finally { trace.push(1); }
    } catch (error) {
        trace.push(2);

    } finally { trace.push(3); }
    return saved<T>(value);
}
function override(trace: number[]): number {
    try { throw 1; } finally { trace.push(4); return 7; }
}
function returned(trace: number[]): number { try { return 8; } finally { trace.push(5); } }
function main() {
    const normal: number[] = [];
    strictEqual(forward(false, 4, normal), 4);
    strictEqual(normal.join(","), "1,3");
    const failed: number[] = [];
    strictEqual(forward(true, "owned", failed), "owned");
    strictEqual(failed.join(","), "1,2,3");
    strictEqual(override(failed), 7);
    strictEqual(failed.join(","), "1,2,3,4");
    strictEqual(returned(failed), 8);
    strictEqual(failed.join(","), "1,2,3,4,5");
}

main();
