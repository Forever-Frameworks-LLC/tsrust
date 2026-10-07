export {};
type int = number;
type Owned<T> = T;
type Mutable<T> = T;
type Function<Result, Arguments extends unknown[]> = (...values: Arguments) => Result;
function strictEqual<T>(actual: T, expected: T): void { if (actual !== expected) throw new Error('assertion'); }
type Box<T> = { value: T; };
function make() {
    let calls = 0;
    function take<T>({ value }: Owned<Box<T>>): T { calls += 1; console.log(calls); return value; }
    return take;
}
function main() {
    const action = make();
    {
        const fixed: Mutable<Function<int, [box: Owned<Box<int>>]>> = action;
        strictEqual(fixed({ value: 7 }), 7);
    }
    const box: Box<string> = { value: 'owner' };
    strictEqual(action(box), 'owner');
}

main();
