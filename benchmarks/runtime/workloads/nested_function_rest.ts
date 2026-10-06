function workload(n: number, seed: number, input: string): number[][] {
    const output: number[][] = [];
    let i = 0;
    while (i < n) {
        const base = i % 5 + seed % 5 + 1;
        function sum(...values: number[]): number {
            let total = base;
            for (const value of values) { total += value; }
            return total;
        }
        const action = sum;
        const empty = action();
        const supplied = action(2, 3);
        const pack = [4, 5];
        const spread = action(...pack);
        function front(first: number = base, ...tail: number[]): number { return first + tail.length; }
        const prefix = front;
        const omitted = prefix();
        const undefinedFirst = prefix(undefined, 1, 2);
        const explicit = prefix(3, 4);
        output.push([empty, supplied, spread, omitted, undefinedFirst, explicit, input.length]);
        i += 1;
    }
    return output;
}
function main() {}
