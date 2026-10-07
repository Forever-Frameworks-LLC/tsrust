import { strictEqual } from 'node:assert';

function defaults() {
    let calls = 0;
    function next(): number {
        calls += 1;
        console.log(calls);
        return calls;
    }
    function select<T>(value: T, offset: number = next()): T {
        console.log(offset);
        return value;
    }
    return select;
}

function rest() {
    let calls = 0;
    function select<T>(fallback: T, ...values: T[]): T {
        calls += 1;
        console.log(calls, values.length);
        return fallback;
    }
    return select;
}

function main() {
    const action = defaults();
    {
        const view: (value: number, offset?: number) => number = action;
        strictEqual(view(7), 7);
        strictEqual(view(8, 9), 8);
        strictEqual(view(10, undefined), 10);
    }
    strictEqual(action('after'), 'after');
    const packed = rest();
    {
        const view: (fallback: number, ...values: number[]) => number = packed;
        strictEqual(view(1), 1);
        strictEqual(view(2, 3, 4), 2);
    }
    strictEqual(packed('after', 'one'), 'after');
}

main();
