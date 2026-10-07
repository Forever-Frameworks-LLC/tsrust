import { strictEqual } from 'node:assert';
type Item = { id: number; };
function make() {
    let calls = 0;
    function identity<T>(value: T): T { calls += 1; console.log(calls); return value; }
    return identity;
}
function main() {
    const fixed: (value: () => Item) => () => Item = make();
    const item: Item = { id: 7 };
    const original: () => Item = (): Item => item;
    const action = fixed(original);
    const result = action();
    strictEqual(result.id, 7);
}

main();
