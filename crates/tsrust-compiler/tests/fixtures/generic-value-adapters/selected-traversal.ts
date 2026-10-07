import { strictEqual } from 'node:assert';

function first() {
    function value<T>(value: T): T { return value; }
    return value;
}

function second() {
    function value<T>(value: T): T { return value; }
    return value;
}

function truthFirst() {
    function value<T>(value: T): boolean { return true; }
    return value;
}

function truthSecond() {
    function value<T>(value: T): boolean { return true; }
    return value;
}

function reduceFirst() {
    function value<T>(left: T, right: T): T { return left; }
    return value;
}

function reduceSecond() {
    function value<T>(left: T, right: T): T { return right; }
    return value;
}

function flatFirst() {
    function value<T>(value: T): T[] { return [value]; }
    return value;
}

function flatSecond() {
    function value<T>(value: T): T[] { return [value]; }
    return value;
}

function compareFirst() {
    function compare<T>(left: T, right: T): number { return 0; }
    return compare;
}

function compareSecond() {
    function compare<T>(left: T, right: T): number { return 0; }
    return compare;
}

function main() {
    const mapped = [3, 4].map((true ? first : second)());
    strictEqual(mapped[1], 4);
    const found = [3, 4].findIndex((false ? truthFirst : truthSecond)());
    strictEqual(found, 0);
    const total = [3, 4].reduce((false ? reduceFirst : reduceSecond)(), 1);
    strictEqual(total, 4);
    const reversed = [3, 4].reduceRight((false ? reduceFirst : reduceSecond)(), 1);
    strictEqual(reversed, 3);
    const flattened = [3, 4].flatMap((true ? flatFirst : flatSecond)());
    strictEqual(flattened.length, 2);
    strictEqual(flattened[1], 4);
    const values = [3, 4];
    values.sort((false ? compareFirst : compareSecond)());
    strictEqual(values[0], 3);
    const copied = values.toSorted((true ? compareFirst : compareSecond)());
    strictEqual(copied[1], 4);
}

main();
