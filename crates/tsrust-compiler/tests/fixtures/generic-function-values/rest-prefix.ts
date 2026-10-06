import { strictEqual } from "node:assert";

function main() {
    let calls = 0;
    function first<T>(fallback: T, offset: number = 0, ...values: T[]): T {
        calls += 1;
        if (values.length === 0) { return fallback; }
        return values.splice(offset, 1)[0];
    }
    {
        const action = first;
        strictEqual(action(7), 7);
        strictEqual(action("fallback", undefined, "a", "b"), "a");
        strictEqual(action(0, 1, 4, 5), 5);
    }
    strictEqual(calls, 3);
}

main();
