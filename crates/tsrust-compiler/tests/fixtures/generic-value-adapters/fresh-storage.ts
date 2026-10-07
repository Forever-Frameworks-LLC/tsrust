import { strictEqual } from 'node:assert';

type Holder = { run: (value: number) => number; };
function make(start: number) {
    console.log(start);
    let calls = start;
    function select<T>(value: T): T {
        calls += 1;
        console.log(calls);
        return value;
    }
    return select;
}
function other(start: number) {
    console.log(start);
    let calls = start;
    function select<T>(value: T): T {
        calls += 2;
        console.log(calls);
        return value;
    }
    return select;
}
function main() {
    const holder: Holder = { run: make(10) };
    strictEqual(holder.run(7), 7);
    const actions: ((value: string) => string)[] = [make(20), (false ? make : other)(30)];
    strictEqual(actions[0]('a'), 'a');
    strictEqual(actions[0]('b'), 'b');
    strictEqual(actions[1]('c'), 'c');
}

main();
