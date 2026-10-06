import { strictEqual } from 'node:assert';

function make(start: number) {
    let calls = start;
    function select<T>(value: T): T {
        calls += 1;
        console.log(calls);
        return value;
    }
    return select;
}

function main() {
    const action = make(10);
    {
        const integer: (value: number) => number = action;
        strictEqual(integer(7), 7);
    }
    strictEqual(action('generic'), 'generic');
    const text: (value: string) => string = action;
    strictEqual(text('owned'), 'owned');
}

main();
