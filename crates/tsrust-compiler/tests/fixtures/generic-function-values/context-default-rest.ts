import { strictEqual } from 'node:assert';

function offset<T>(value: T, index: number = 0): T {
    return value;
}

function first<T>(...values: T[]): T {
    return values.shift()!;
}

function main() {
    const choose: (value: number, index?: number) => number = offset;
    strictEqual(choose(7), 7);
    strictEqual(choose(8, undefined), 8);
    const pick: (...values: number[]) => number = first;
    strictEqual(pick(3, 4), 3);
}

main();
