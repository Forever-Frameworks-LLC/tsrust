function workload(n: number, seed: number, input: string): number[][] {
    function independent(): number {
        function unused(): number { return later.length; }
        return 7;
    }
    const independentResult = independent();
    const later = [1, 2];
    if (independentResult !== 7) { throw 99; }
    const output: number[][] = [];
    let i = 0;
    while (i < n) {
        const value = i % 5 + seed % 5 + 1;
        const actions = [make(value), make(value + 10)];
        const first = actions[0]();
        const second = actions[1]();
        const again = actions[0]();
        const recurse = recursive([value]);
        const take = owner(value + 20);
        const item = take();
        const fail = failure(value);
        let caught = 0;
        try { fail(); } catch { caught = 1; }
        output.push([first, second, again, recurse(4), recurse(3), item.value, caught, input.length]);
        i += 1;
    }
    return output;
}
type Item = { value: number; };
function make(start: number): () => number {
    let value = start;
    function next(): number { value += 1; return value; }
    return next;
}
function recursive(base: number[]): (n: number) => number {
    function even(n: number): number { if (n === 0) { return base[0]; } return odd(n - 1); }
    function odd(n: number): number { if (n === 0) { return base[0] + 1; } return even(n - 1); }
    return even;
}
function owner(value: number): () => Item {
    const item: Item = { value: value };
    function take(): Item { return item; }
    return take;
}
function failure(value: number): () => number {
    function fail(): number { throw value; }
    return fail;
}
function main(): void {}
