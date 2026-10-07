function strictEqual(actual: number, expected: number): void { if (actual !== expected) throw new Error('assertion'); }

function next(): number { console.log("default"); return 4; }
function sum(value: number = next(), ...tail: number[]): number {
    let result = value;
    for (const item of tail) { result += item; }
    return result;
}
function make() {
    let calls = 0;
    function identity<T>(value: T): T { calls += 1; console.log(calls); return value; }
    return identity;
}
function main() {
    const fixed: (action: (value?: number, ...tail: number[]) => number) => (value?: number, ...tail: number[]) => number = make();
    const action = fixed(sum);
    strictEqual(action(), 4);
    strictEqual(action(undefined, 1, 2), 7);
    strictEqual(action(7, ...[3, 4]), 14);
}

main();
