import { strictEqual } from "node:assert";

function one(value: number): number { return value; }
function two(value: number, other: number): number { return other; }
function namedFactory<A>(action: A) {
    function select(): A { return action; }
    return select;
}
function arrowFactory<A>(action: A) { return (): A => action; }

for (const factory of [namedFactory, arrowFactory]) {
    const first = factory(one);
    const second = factory(two);
    const saved = first();
    const wrong = second();
    strictEqual(saved, one);
    strictEqual(wrong, two);
    strictEqual(saved.length, 1);
    strictEqual(wrong.length, 2);
    strictEqual(saved(7), 7);
    strictEqual(wrong(7, 9), 9);
}
console.log("Named and arrow factories preserve distinct callable identities and arities.");
