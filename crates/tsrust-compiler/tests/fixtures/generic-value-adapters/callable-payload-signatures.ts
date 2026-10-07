import { strictEqual } from 'node:assert';
function make() {
    let calls = 0;
    function identity<T>(value: T): T { calls += 1; console.log(calls); return value; }
    return identity;
}
function main() {
    const source = make();
    {
        const view: (value: (value: number) => number) => (value: number) => number = source;
        const action = view((value: number): number => value + 1);
        strictEqual(action(7), 8);
    }
    {
        const view: (value: (value: string) => string) => (value: string) => string = source;
        const action = view((value: string): string => value + "!");
        strictEqual(action("owner"), "owner!");
    }
}

main();
