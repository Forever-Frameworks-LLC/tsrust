import { strictEqual } from 'node:assert';
function make() {
    let calls = 0;
    function identity<T>(value: T): T { calls += 1; console.log(calls); return value; }
    return identity;
}
function main() {
    const fixed: (value: (value: number) => number) => (value: number) => number = make();
    const first = fixed((value: number): number => value + 1);
    const second = fixed((value: number): number => value + 2);
    strictEqual(first(7), 8);
    strictEqual(second(7), 9);
}

main();
