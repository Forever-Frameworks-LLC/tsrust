import { strictEqual } from 'node:assert';
function make() {
    let calls = 0;
    function select<T>(value: T): T { calls += 1; console.log(calls); return value; }
    return select;
}
function missing(): ((value: number) => number) | undefined { return undefined; }
function main() {
    const present: ((value: number) => number) | undefined = make();
    const first = present?.(7);
    const second = present?.(8);
    if (first === undefined || second === undefined) { throw 99; }
    strictEqual(first, 7);
    strictEqual(second, 8);
    const absent: ((value: number) => number) | undefined = missing();
    if (absent?.(9) !== undefined) { throw 99; }
}

main();
