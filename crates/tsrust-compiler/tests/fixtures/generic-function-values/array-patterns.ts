import { strictEqual } from "node:assert";

function main() {
    let calls = 0;
    function first<T>(values: T[]): T {
        calls += 1;
        return values.shift()!;
    }
    {
        const action = first;
        const numbers = [7, 8];
        strictEqual(action(numbers), 7);
        strictEqual(numbers.length, 1);
        const strings = ["a", "b"];
        strictEqual(action(strings), "a");
        strictEqual(strings.length, 1);
    }
    strictEqual(calls, 2);
}

main();
