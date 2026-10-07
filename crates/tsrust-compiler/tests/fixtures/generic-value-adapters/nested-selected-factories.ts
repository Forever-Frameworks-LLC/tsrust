import { strictEqual } from 'node:assert';

function left(start: number) {
    console.log("left", start);
    let calls = start;
    function select<T>(value: T): T {
        calls += 1;
        console.log(calls);
        return value;
    }
    return select;
}

function right(start: number) {
    console.log("right", start);
    let calls = start;
    function choose<T>(value: T): T {
        calls += 2;
        console.log(calls);
        return value;
    }
    return choose;
}

function main() {
    let checks = 0;
    let evaluations = 0;
    const fixed: (value: number) => number = (checks++ === 0 ? (checks++ === 1 ? left : right) : right)(++evaluations);
    strictEqual(fixed(7), 7);
    const mapped = [3, 4].map((checks++ === 0 ? left : (checks++ === 3 ? right : left))(++evaluations));
    strictEqual(mapped[0], 3);
    strictEqual(mapped[1], 4);
    strictEqual(checks, 4);
    strictEqual(evaluations, 2);
}

main();
