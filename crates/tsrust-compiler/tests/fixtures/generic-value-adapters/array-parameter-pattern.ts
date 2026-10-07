export {};
type int = number;
type Owned<T> = T;
type Mutable<T> = T;
type Function<Result, Arguments extends unknown[]> = (...values: Arguments) => Result;
function strictEqual<T>(actual: T, expected: T): void { if (actual !== expected) throw new Error('assertion'); }
function make() {
    let calls = 0;
    function head<T>([first]: Owned<T[]>): T { calls += 1; console.log(calls); return first; }
    return head;
}
function main() {
    const action = make();
    {
        const fixed: Mutable<Function<int, [values: Owned<int[]>]>> = action;
        strictEqual(fixed([7, 8]), 7);
    }
    strictEqual(action(['owner', 'extra']), 'owner');
}

main();
