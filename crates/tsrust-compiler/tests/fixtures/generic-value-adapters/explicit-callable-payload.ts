function strictEqual(actual: number, expected: number): void { if (actual !== expected) throw new Error('assertion'); }

function make() {
    let calls = 0;
    function identity<T>(value: T): T {
        calls += 1;
        console.log(calls);
        return value;
    }
    return identity;
}

function main() {
    const identity = make();
    const first = identity<(value: number) => number>((value: number): number => value + 1);
    const second = identity<(value: number) => number>((value: number): number => value + 2);
    strictEqual(first(7), 8);
    strictEqual(second(7), 9);
}

main();
