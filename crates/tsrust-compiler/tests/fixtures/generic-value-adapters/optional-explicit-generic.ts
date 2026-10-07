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

function enabled(flag: boolean): boolean { return flag; }

function main() {
    let effects = 0;
    const missing = enabled(false) ? make() : undefined;
    const absent = missing?.<number>(effects++);
    strictEqual(absent ?? -1, -1);
    strictEqual(effects, 0);
    const present = enabled(true) ? make() : undefined;
    const first = present?.<number>(++effects);
    const second = present?.<number>(++effects);
    strictEqual(first ?? -1, 1);
    strictEqual(second ?? -1, 2);
    strictEqual(effects, 2);
}

main();
