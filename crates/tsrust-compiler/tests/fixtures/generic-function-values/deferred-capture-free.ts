import { strictEqual } from 'node:assert';

function identity<T>(value: T): T { return value; }
function first<T>(...values: T[]): T { return values.shift()!; }
function size<T>(values: T[]): number { return values.length; }
function reject<T>(value: T): T { throw 'reject'; }
function mark(): number { console.log('default'); return 0; }
function offset<T>(value: T, index: number = mark()): T { return value; }

function main() {
    const action = identity;
    strictEqual(action(7), 7);
    strictEqual(action('text'), 'text');
    const values = [3, 4];
    const length = size;
    strictEqual(length(values), 2);
    const returned = action(values);
    strictEqual(returned[1], 4);
    const pick = first;
    strictEqual(pick(8, 9), 8);
    strictEqual(pick('left', 'right'), 'left');
    const choose = offset;
    strictEqual(choose(5), 5);
    strictEqual(choose('other', 1), 'other');
    const fail = reject;
    let caught = false;
    try { fail(1); } catch (error) { caught = true; }
    strictEqual(caught, true);
}

main();
