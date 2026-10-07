import { strictEqual } from 'node:assert';

function increment(value: number): number { return value + 1; }
function make() {
    let calls = 0;
    function invoke<T>(action: (value: T) => T, value: T): T {
        calls += 1;
        console.log(calls);
        return action(value);
    }
    return invoke;
}
function main() {
    const fixed: (action: (value: number) => number, value: number) => number = make();
    strictEqual(fixed(increment, 7), 8);
    strictEqual(fixed(increment, 8), 9);
}
main();
