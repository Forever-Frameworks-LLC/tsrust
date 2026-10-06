import { strictEqual } from 'node:assert';

type Holder = { run: (value: number) => number; };

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
    const first = make(10);
    const holder: Holder = { run: first };
    strictEqual(holder.run(7), 7);
    const second = make(20);
    const actions: ((value: string) => string)[] = [second];
    strictEqual(actions[0]('a'), 'a');
    strictEqual(actions[0]('b'), 'b');
}

main();
