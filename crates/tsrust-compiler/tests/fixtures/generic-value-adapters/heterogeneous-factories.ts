import { strictEqual } from 'node:assert';
function left() {
    let calls = 0;
    function select<T>(value: T): T { calls += 1; console.log(calls); return value; }
    return select;
}
function right() {
    let calls = 0;
    function select<T>(value: T): T[] { calls += 1; console.log(calls); return [value]; }
    return select;
}
function main() {
    const first: ((value: number) => number | number[]) = (true ? left : right)();
    const second: ((value: number) => number | number[]) = (false ? left : right)();
    const scalar = first(7);
    const array = second(8);
    if (typeof scalar === 'number') { strictEqual(scalar, 7); } else { throw 99; }
    if (typeof array === 'number') { throw 99; } else { strictEqual(array[0], 8); }
}

main();
